//! In-memory transport implementation for the Model Context Protocol
//!
//! This module provides an in-memory transport implementation that uses Tokio channels
//! for communication between clients and servers. It's primarily used for testing and
//! for scenarios where both the client and server are in the same process.
//!
//! The in-memory transport consists of two main components:
//! - `ServerInMemoryTransport`: The server-side transport that receives messages from a channel
//! - `ClientInMemoryTransport`: The client-side transport that communicates with a spawned server task
//!
//! # Examples
//!
//! ```
//! use mcp_daemon::transport::{ClientInMemoryTransport, Transport};
//! use tokio::spawn;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an echo server function
//!     async fn echo_server(transport: mcp_daemon::transport::ServerInMemoryTransport) {
//!         while let Ok(Some(message)) = transport.receive().await {
//!             let _ = transport.send(&message).await;
//!         }
//!     }
//!
//!     // Create a client transport with the echo server
//!     let transport = ClientInMemoryTransport::new(|t| spawn(echo_server(t)));
//!
//!     // Open the transport
//!     transport.open().await?;
//!
//!     // Send a message
//!     let message = serde_json::json!({
//!         "jsonrpc": "2.0",
//!         "method": "test",
//!         "params": {"hello": "world"},
//!         "id": 1
//!     });
//!     transport.send(&message).await?;
//!
//!     // Receive the echoed message
//!     let response = transport.receive().await?;
//!     assert_eq!(Some(message), response);
//!
//!     // Close the transport
//!     transport.close().await?;
//!
//!     Ok(())
//! }
//! ```

use super::{Message, Transport};
use super::error::{TransportError, TransportErrorCode};
use super::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::debug;

/// Server-side transport that receives messages from a channel
///
/// This transport implementation is used on the server side of an in-memory
/// connection. It receives messages from a channel and sends responses back
/// through another channel.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ServerInMemoryTransport, Transport};
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a server transport
///     let transport = ServerInMemoryTransport::default();
///
///     // Open the transport
///     transport.open().await?;
///
///     // Process messages
///     while let Ok(Some(message)) = transport.receive().await {
///         // Echo the message back
///         transport.send(&message).await?;
///     }
///
///     // Close the transport
///     transport.close().await?;
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ServerInMemoryTransport {
    rx: Arc<Mutex<Option<Receiver<Message>>>>,
    tx: Sender<Message>,
}

impl Default for ServerInMemoryTransport {
    /// Creates a new server transport with default settings
    ///
    /// This creates a new transport with a channel buffer size of 100 messages.
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(100); // Default buffer size of 100
        Self {
            rx: Arc::new(Mutex::new(Some(rx))),
            tx,
        }
    }
}

#[async_trait]
impl Transport for ServerInMemoryTransport {
    /// Receives a message from the client
    ///
    /// This method waits for a message to be received from the client.
    /// If the channel is closed, it returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - A message was received
    /// * `Ok(None)` - The channel was closed
    /// * `Err(error)` - An error occurred
    async fn receive(&self) -> Result<Option<Message>> {
        let mut rx_guard = self.rx.lock().await;
        let rx = rx_guard
            .as_mut()
            .ok_or_else(|| TransportError::new(TransportErrorCode::InvalidState, "Transport not opened"))?;

        match rx.recv().await {
            Some(message) => {
                debug!("Server received: {:?}", message);
                Ok(Some(message))
            }
            None => {
                debug!("Client channel closed");
                Ok(None)
            }
        }
    }

    /// Sends a message to the client
    ///
    /// This method sends a message to the client through the channel.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The message was sent successfully
    /// * `Err(error)` - An error occurred
    async fn send(&self, message: &Message) -> Result<()> {
        debug!("Server sending: {:?}", message);
        self.tx
            .send(message.clone())
            .await
            .map_err(|e| TransportError::new(TransportErrorCode::MessageSendFailed, format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    /// Opens the transport
    ///
    /// For the server transport, this is a no-op since the transport
    /// is initialized in the constructor.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was opened successfully
    async fn open(&self) -> Result<()> {
        Ok(())
    }

    /// Closes the transport
    ///
    /// This method closes the transport by dropping the receiver.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was closed successfully
    async fn close(&self) -> Result<()> {
        *self.rx.lock().await = None;
        Ok(())
    }
}

/// Client-side transport that communicates with a spawned server task
///
/// This transport implementation is used on the client side of an in-memory
/// connection. It spawns a server task and communicates with it through channels.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ClientInMemoryTransport, Transport};
/// use tokio::spawn;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     // Create an echo server function
///     async fn echo_server(transport: mcp_daemon::transport::ServerInMemoryTransport) {
///         while let Ok(Some(message)) = transport.receive().await {
///             let _ = transport.send(&message).await;
///         }
///     }
///
///     // Create a client transport with the echo server
///     let transport = ClientInMemoryTransport::new(|t| spawn(echo_server(t)));
///
///     // Open the transport
///     transport.open().await?;
///
///     // Send a message
///     let message = serde_json::json!({
///         "jsonrpc": "2.0",
///         "method": "test",
///         "params": {"hello": "world"},
///         "id": 1
///     });
///     transport.send(&message).await?;
///
///     // Receive the echoed message
///     let response = transport.receive().await?;
///
///     // Close the transport
///     transport.close().await?;
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ClientInMemoryTransport {
    tx: Arc<Mutex<Option<Sender<Message>>>>,
    rx: Arc<Mutex<Option<Receiver<Message>>>>,
    server_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    server_factory: Arc<dyn Fn(ServerInMemoryTransport) -> JoinHandle<()> + Send + Sync>,
}

impl ClientInMemoryTransport {
    /// Creates a new in-memory transport with a server factory function
    ///
    /// The server factory function takes a `ServerInMemoryTransport` and returns a `JoinHandle`
    /// that represents the server task. This allows the client to spawn a server task
    /// when the transport is opened.
    ///
    /// # Arguments
    ///
    /// * `server_factory` - A function that takes a `ServerInMemoryTransport` and returns a `JoinHandle`
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ClientInMemoryTransport;
    /// use tokio::spawn;
    ///
    /// // Create an echo server function
    /// async fn echo_server(transport: mcp_daemon::transport::ServerInMemoryTransport) {
    ///     while let Ok(Some(message)) = transport.receive().await {
    ///         let _ = transport.send(&message).await;
    ///     }
    /// }
    ///
    /// // Create a client transport with the echo server
    /// let transport = ClientInMemoryTransport::new(|t| spawn(echo_server(t)));
    /// ```
    pub fn new<F>(server_factory: F) -> Self
    where
        F: Fn(ServerInMemoryTransport) -> JoinHandle<()> + Send + Sync + 'static,
    {
        Self {
            tx: Arc::new(Mutex::new(None)),
            rx: Arc::new(Mutex::new(None)),
            server_handle: Arc::new(Mutex::new(None)),
            server_factory: Arc::new(server_factory),
        }
    }
}

#[async_trait]
impl Transport for ClientInMemoryTransport {
    /// Receives a message from the server
    ///
    /// This method waits for a message to be received from the server.
    /// If the channel is closed, it returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - A message was received
    /// * `Ok(None)` - The channel was closed
    /// * `Err(error)` - An error occurred
    async fn receive(&self) -> Result<Option<Message>> {
        let mut rx_guard = self.rx.lock().await;
        let rx = rx_guard
            .as_mut()
            .ok_or_else(|| TransportError::new(TransportErrorCode::InvalidState, "Transport not opened"))?;

        match rx.recv().await {
            Some(message) => {
                debug!("Client received: {:?}", message);
                Ok(Some(message))
            }
            None => {
                debug!("Server channel closed");
                Ok(None)
            }
        }
    }

    /// Sends a message to the server
    ///
    /// This method sends a message to the server through the channel.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The message was sent successfully
    /// * `Err(error)` - An error occurred
    async fn send(&self, message: &Message) -> Result<()> {
        let tx_guard = self.tx.lock().await;
        let tx = tx_guard
            .as_ref()
            .ok_or_else(|| TransportError::new(TransportErrorCode::InvalidState, "Transport not opened"))?;

        debug!("Client sending: {:?}", message);
        tx.send(message.clone())
            .await
            .map_err(|e| TransportError::new(TransportErrorCode::MessageSendFailed, format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    /// Opens the transport
    ///
    /// This method creates the channels for communication and spawns the server task.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was opened successfully
    /// * `Err(error)` - An error occurred
    async fn open(&self) -> Result<()> {
        let (client_tx, server_rx) = mpsc::channel(100);
        let (server_tx, client_rx) = mpsc::channel(100);

        let server_transport = ServerInMemoryTransport {
            rx: Arc::new(Mutex::new(Some(server_rx))),
            tx: server_tx,
        };

        let server_handle = (self.server_factory)(server_transport);

        *self.rx.lock().await = Some(client_rx);
        *self.tx.lock().await = Some(client_tx);
        *self.server_handle.lock().await = Some(server_handle);

        Ok(())
    }

    /// Closes the transport
    ///
    /// This method closes the transport by dropping the channels and waiting
    /// for the server task to complete.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was closed successfully
    /// * `Err(error)` - An error occurred
    async fn close(&self) -> Result<()> {
        *self.tx.lock().await = None;
        *self.rx.lock().await = None;

        if let Some(handle) = self.server_handle.lock().await.take() {
            handle.await.map_err(|e| TransportError::new(TransportErrorCode::InternalError, format!("Server task failed: {}", e)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{JsonRpcMessage, JsonRpcRequest, JsonRpcVersion};
    use std::time::Duration;

    async fn echo_server(transport: ServerInMemoryTransport) {
        while let Ok(Some(message)) = transport.receive().await {
            if transport.send(&message).await.is_err() {
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_async_transport() -> Result<()> {
        let transport = ClientInMemoryTransport::new(|t| tokio::spawn(echo_server(t)));

        // Create a test message
        let test_message = JsonRpcMessage::Request(JsonRpcRequest {
            id: 1,
            method: "test".to_string(),
            params: Some(serde_json::json!({"hello": "world"})),
            jsonrpc: JsonRpcVersion::default(),
        });

        // Open transport
        transport.open().await?;

        // Send message
        transport.send(&test_message).await?;

        // Receive echoed message
        let response = transport.receive().await?;

        // Verify the response matches
        assert_eq!(Some(test_message), response);

        // Clean up
        transport.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_graceful_shutdown() -> Result<()> {
        let transport = ClientInMemoryTransport::new(|t| {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(5)).await;
                drop(t);
            })
        });

        transport.open().await?;

        // Spawn a task that will read from the transport
        let transport_clone = transport.clone();
        let read_handle = tokio::spawn(async move {
            let result = transport_clone.receive().await;
            debug!("Receive returned: {:?}", result);
            result
        });

        // Wait a bit to ensure the server is running
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Initiate graceful shutdown
        let start = std::time::Instant::now();
        transport.close().await?;
        let shutdown_duration = start.elapsed();

        // Verify shutdown completed quickly
        assert!(shutdown_duration < Duration::from_secs(5));

        // Verify receive operation was cancelled
        let read_result = read_handle.await?;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), None);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_messages() -> Result<()> {
        let transport = ClientInMemoryTransport::new(|t| tokio::spawn(echo_server(t)));
        transport.open().await?;

        let messages: Vec<_> = (0..5)
            .map(|i| {
                JsonRpcMessage::Request(JsonRpcRequest {
                    id: i,
                    method: format!("test_{}", i),
                    params: Some(serde_json::json!({"index": i})),
                    jsonrpc: JsonRpcVersion::default(),
                })
            })
            .collect();

        // Send all messages
        for msg in &messages {
            transport.send(msg).await?;
        }

        // Receive and verify all messages
        for expected in &messages {
            let received = transport.receive().await?;
            assert_eq!(Some(expected.clone()), received);
        }

        transport.close().await?;
        Ok(())
    }
}
