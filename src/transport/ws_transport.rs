//! WebSocket transport implementation for the Model Context Protocol
//!
//! This module provides transport implementations that use WebSockets for bidirectional
//! communication between MCP clients and servers. It includes two main transport types:
//!
//! - `ServerWsTransport`: A transport for server-side WebSocket connections
//! - `ClientWsTransport`: A transport for client-side WebSocket connections
//!
//! WebSockets provide several advantages for MCP communication:
//! - Full-duplex communication (both directions simultaneously)
//! - Low latency and overhead
//! - Standard web technology with wide support
//! - Compatible with firewalls and proxies
//! - Support for secure connections (WSS)
//!
//! # Examples
//!
//! ## Server-side usage
//!
//! ```
//! use mcp_daemon::transport::{ServerWsTransport, Transport};
//! use actix_ws::Session;
//! use tokio::sync::broadcast;
//!
//! async fn example(session: Session) -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a channel for message passing
//!     let (tx, rx) = broadcast::channel(100);
//!     
//!     // Create a server transport
//!     let transport = ServerWsTransport::new(session, rx);
//!     
//!     // Open the transport
//!     transport.open().await?;
//!     
//!     // Send a message
//!     let message = serde_json::json!({
//!         "jsonrpc": "2.0",
//!         "method": "notification",
//!         "params": { "type": "update", "data": "New data available" }
//!     });
//!     transport.send(&message).await?;
//!     
//!     // Close the transport
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Client-side usage
//!
//! ```
//! use mcp_daemon::transport::{ClientWsTransport, Transport};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client transport with a builder
//!     let transport = ClientWsTransport::builder("ws://localhost:8080/ws".to_string())
//!         .with_header("Authorization", "Bearer token123")
//!         .build();
//!     
//!     // Open the transport (connects to the WebSocket server)
//!     transport.open().await?;
//!     
//!     // Send a message
//!     let message = serde_json::json!({
//!         "jsonrpc": "2.0",
//!         "method": "example",
//!         "params": {"hello": "world"},
//!         "id": 1
//!     });
//!     transport.send(&message).await?;
//!     
//!     // Receive a response
//!     if let Some(response) = transport.receive().await? {
//!         println!("Received: {:?}", response);
//!     }
//!     
//!     // Close the transport
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```

use super::{Message, Transport};
use super::Result;
use super::error::{TransportError, TransportErrorCode};
use actix_ws::{Message as WsMessage, Session};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use reqwest::header::{HeaderName, HeaderValue};
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message as TungsteniteMessage};
use tracing::{debug, info};

// Type aliases to simplify complex types
type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures::stream::SplitSink<WsStream, TungsteniteMessage>;
type MessageSender = broadcast::Sender<Message>;
type MessageReceiver = broadcast::Receiver<Message>;

/// WebSocket transport implementation for the server side
///
/// This transport handles WebSocket connections on the server side, allowing
/// bidirectional communication with clients. It uses actix-web's WebSocket
/// implementation and broadcast channels for message distribution.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ServerWsTransport, Transport};
/// use actix_ws::Session;
/// use tokio::sync::broadcast;
///
/// async fn example(session: Session) -> Result<(), Box<dyn std::error::Error>> {
///     // Create a channel for message passing
///     let (tx, rx) = broadcast::channel(100);
///     
///     // Create a server transport
///     let transport = ServerWsTransport::new(session, rx);
///     
///     // Send a message
///     let message = serde_json::json!({
///         "jsonrpc": "2.0",
///         "method": "notification",
///         "params": { "type": "update", "data": "New data available" }
///     });
///     transport.send(&message).await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ServerWsTransport {
    session: Arc<Mutex<Option<Session>>>,
    rx: Arc<Mutex<Option<broadcast::Receiver<Message>>>>,
    tx: Arc<Mutex<Option<broadcast::Sender<Message>>>>,
}

impl std::fmt::Debug for ServerWsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerWsTransport")
            .field("session", &"<Session>")
            .field("rx", &self.rx)
            .field("tx", &self.tx)
            .finish()
    }
}

impl ServerWsTransport {
    /// Create a new server-side WebSocket transport
    ///
    /// This constructor creates a new transport with the given WebSocket session
    /// and broadcast receiver for incoming messages.
    ///
    /// # Arguments
    ///
    /// * `session` - The WebSocket session
    /// * `rx` - Channel receiver for incoming messages
    ///
    /// # Returns
    ///
    /// A new `ServerWsTransport` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ServerWsTransport;
    /// use actix_ws::Session;
    /// use tokio::sync::broadcast;
    ///
    /// // Create a channel for message passing
    /// let (tx, rx) = broadcast::channel(100);
    ///
    /// // Create a server transport (in an actual application, the session would come from actix-web)
    /// # fn example(session: Session) {
    /// let transport = ServerWsTransport::new(session, rx);
    /// # }
    /// ```
    pub fn new(session: Session, rx: broadcast::Receiver<Message>) -> Self {
        Self {
            session: Arc::new(Mutex::new(Some(session))),
            rx: Arc::new(Mutex::new(Some(rx))),
            tx: Arc::new(Mutex::new(None)),
        }
    }
}

/// WebSocket transport implementation for the client side
///
/// This transport handles WebSocket connections on the client side, allowing
/// bidirectional communication with servers. It uses tokio-tungstenite for
/// WebSocket communication and broadcast channels for message distribution.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ClientWsTransport, Transport};
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client transport with a builder
///     let transport = ClientWsTransport::builder("ws://localhost:8080/ws".to_string())
///         .with_header("Authorization", "Bearer token123")
///         .build();
///     
///     // Open the transport (connects to the WebSocket server)
///     transport.open().await?;
///     
///     // Send a message
///     let message = serde_json::json!({
///         "jsonrpc": "2.0",
///         "method": "example",
///         "params": {"hello": "world"},
///         "id": 1
///     });
///     transport.send(&message).await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ClientWsTransport {
    ws_tx: Arc<Mutex<Option<MessageSender>>>,
    ws_rx: Arc<Mutex<Option<MessageReceiver>>>,
    url: String,
    headers: HashMap<String, String>,
    ws_write: Arc<Mutex<Option<WsSink>>>,
}

impl std::fmt::Debug for ClientWsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientWsTransport")
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("ws_tx", &"<MessageSender>")
            .field("ws_rx", &"<MessageReceiver>")
            .field("ws_write", &"<WsSink>")
            .finish()
    }
}

impl ClientWsTransport {
    /// Create a builder for configuring and creating a client WebSocket transport
    ///
    /// This method returns a builder that can be used to configure and create
    /// a client WebSocket transport with custom options.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket server URL to connect to
    ///
    /// # Returns
    ///
    /// A `ClientWsTransportBuilder` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ClientWsTransport;
    ///
    /// // Create a client transport with a builder
    /// let transport = ClientWsTransport::builder("ws://localhost:8080/ws".to_string())
    ///     .with_header("Authorization", "Bearer token123")
    ///     .build();
    /// ```
    pub fn builder(url: String) -> ClientWsTransportBuilder {
        ClientWsTransportBuilder::new(url)
    }
}

/// Builder for configuring and creating a client WebSocket transport
///
/// This builder allows for configuring various options for a client WebSocket
/// transport before creating it, such as custom headers.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::ClientWsTransport;
///
/// // Create a client transport with a builder
/// let transport = ClientWsTransport::builder("ws://localhost:8080/ws".to_string())
///     .with_header("Authorization", "Bearer token123")
///     .with_header("X-Custom-Header", "custom-value")
///     .build();
/// ```
pub struct ClientWsTransportBuilder {
    url: String,
    headers: HashMap<String, String>,
}

impl ClientWsTransportBuilder {
    /// Create a new transport builder
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket server URL to connect to
    ///
    /// # Returns
    ///
    /// A new `ClientWsTransportBuilder` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ClientWsTransportBuilder;
    ///
    /// // Create a new builder
    /// let builder = ClientWsTransportBuilder::new("ws://localhost:8080/ws".to_string());
    /// ```
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: HashMap::new(),
        }
    }

    /// Add a custom header to the WebSocket connection
    ///
    /// This method adds a custom header to the WebSocket connection request.
    /// Headers can be used for authentication, custom protocols, or other
    /// application-specific purposes.
    ///
    /// # Arguments
    ///
    /// * `key` - Header name
    /// * `value` - Header value
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ClientWsTransport;
    ///
    /// // Add custom headers
    /// let transport = ClientWsTransport::builder("ws://localhost:8080/ws".to_string())
    ///     .with_header("Authorization", "Bearer token123")
    ///     .with_header("X-Custom-Header", "custom-value")
    ///     .build();
    /// ```
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Build the client WebSocket transport with the configured options
    ///
    /// This method creates a new `ClientWsTransport` instance with the
    /// configured options. The transport is not connected until `open()`
    /// is called.
    ///
    /// # Returns
    ///
    /// A new `ClientWsTransport` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ClientWsTransport;
    ///
    /// // Build the transport
    /// let transport = ClientWsTransport::builder("ws://localhost:8080/ws".to_string())
    ///     .with_header("Authorization", "Bearer token123")
    ///     .build();
    /// ```
    pub fn build(self) -> ClientWsTransport {
        ClientWsTransport {
            ws_tx: Arc::new(Mutex::new(None)),
            ws_rx: Arc::new(Mutex::new(None)),
            url: self.url,
            headers: self.headers,
            ws_write: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Transport for ServerWsTransport {
    /// Receives a message from the WebSocket
    ///
    /// This method waits for a message to be received from the broadcast channel.
    /// If the channel is closed, it returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - A message was successfully received
    /// * `Ok(None)` - The channel was closed
    /// * `Err(error)` - An error occurred while receiving the message
    async fn receive(&self) -> Result<Option<Message>> {
        let mut rx = self.rx.lock().await;
        if let Some(rx) = rx.as_mut() {
            match rx.recv().await {
                Ok(message) => Ok(Some(message)),
                Err(broadcast::error::RecvError::Closed) => Ok(None),
                Err(e) => Err(TransportError::new(
                    TransportErrorCode::ReceiveError,
                    format!("Error receiving message: {}", e),
                )),
            }
        } else {
            Ok(None)
        }
    }

    /// Sends a message through the WebSocket
    ///
    /// This method serializes the message to JSON and sends it through
    /// the WebSocket session.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The message was successfully sent
    /// * `Err(error)` - An error occurred while sending the message
    async fn send(&self, message: &Message) -> Result<()> {
        let mut session = self.session.lock().await;
        if let Some(session) = session.as_mut() {
            let json = serde_json::to_string(message)?;
            session
                .text(json)
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::SendError, e.to_string()))?;
            Ok(())
        } else {
            Err(TransportError::new(
                TransportErrorCode::SendError,
                "No active session",
            ))
        }
    }

    /// Opens the transport
    ///
    /// For the server transport, this is a no-op since the session is
    /// provided in the constructor.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully opened
    async fn open(&self) -> Result<()> {
        Ok(())
    }

    /// Closes the transport
    ///
    /// This method closes the WebSocket session.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully closed
    /// * `Err(error)` - An error occurred while closing the transport
    async fn close(&self) -> Result<()> {
        let mut session = self.session.lock().await;
        if let Some(session) = session.take() {
            session
                .close(None)
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::CloseError, e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl Transport for ClientWsTransport {
    /// Receives a message from the WebSocket
    ///
    /// This method waits for a message to be received from the broadcast channel.
    /// If the channel is closed, it returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - A message was successfully received
    /// * `Ok(None)` - The channel was closed
    /// * `Err(error)` - An error occurred while receiving the message
    async fn receive(&self) -> Result<Option<Message>> {
        let mut rx = self.ws_rx.lock().await;
        if let Some(rx) = rx.as_mut() {
            match rx.recv().await {
                Ok(message) => Ok(Some(message)),
                Err(broadcast::error::RecvError::Closed) => Ok(None),
                Err(e) => Err(TransportError::new(
                    TransportErrorCode::ReceiveError,
                    format!("Error receiving message: {}", e),
                )),
            }
        } else {
            Ok(None)
        }
    }

    /// Sends a message through the WebSocket
    ///
    /// This method serializes the message to JSON and sends it through
    /// the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The message was successfully sent
    /// * `Err(error)` - An error occurred while sending the message
    async fn send(&self, message: &Message) -> Result<()> {
        let mut ws_write = self.ws_write.lock().await;
        if let Some(ws_write) = ws_write.as_mut() {
            let json = serde_json::to_string(message)?;
            ws_write
                .send(TungsteniteMessage::Text(json))
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::SendError, e.to_string()))?;
            Ok(())
        } else {
            Err(TransportError::new(
                TransportErrorCode::SendError,
                "No active WebSocket connection",
            ))
        }
    }

    /// Opens the transport
    ///
    /// This method connects to the WebSocket server and sets up the
    /// message handling infrastructure.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully opened
    /// * `Err(error)` - An error occurred while opening the transport
    async fn open(&self) -> Result<()> {
        let mut request = self.url.as_str().into_client_request()?;
        for (key, value) in &self.headers {
            request.headers_mut().insert(
                HeaderName::from_str(key).map_err(|e| {
                    TransportError::new(TransportErrorCode::OpenError, format!("Invalid header key: {}", e))
                })?,
                HeaderValue::from_str(value).map_err(|e| {
                    TransportError::new(
                        TransportErrorCode::OpenError,
                        format!("Invalid header value: {}", e),
                    )
                })?,
            );
        }

        let (ws_stream, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| TransportError::new(TransportErrorCode::OpenError, e.to_string()))?;

        let (write, mut read) = ws_stream.split();
        let (tx, rx) = broadcast::channel(100);

        let ws_tx = tx.clone();
        let ws_rx = rx;

        *self.ws_tx.lock().await = Some(ws_tx);
        *self.ws_rx.lock().await = Some(ws_rx);
        *self.ws_write.lock().await = Some(write);

        // Spawn a task to handle incoming messages
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(TungsteniteMessage::Text(text)) => {
                        if let Ok(message) = serde_json::from_str::<Message>(&text) {
                            if tx.send(message).is_err() {
                                debug!("All receivers dropped, stopping message handling");
                                break;
                            }
                        }
                    }
                    Ok(TungsteniteMessage::Close(_)) => {
                        info!("WebSocket connection closed by server");
                        break;
                    }
                    Err(e) => {
                        debug!("Error reading from WebSocket: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Closes the transport
    ///
    /// This method sends a close frame to the WebSocket server and
    /// cleans up the message handling infrastructure.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The transport was successfully closed
    /// * `Err(error)` - An error occurred while closing the transport
    async fn close(&self) -> Result<()> {
        if let Some(mut write) = self.ws_write.lock().await.take() {
            write
                .send(TungsteniteMessage::Close(None))
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::CloseError, e.to_string()))?;
        }
        Ok(())
    }
}

/// Handle a WebSocket connection, managing message flow between client and server
///
/// This function sets up bidirectional communication between a WebSocket client
/// and server, using broadcast channels for message distribution. It spawns two
/// tasks: one for sending messages from the server to the client, and one for
/// receiving messages from the client and forwarding them to the server.
///
/// # Arguments
///
/// * `session` - The WebSocket session
/// * `stream` - Stream of incoming WebSocket messages
/// * `tx` - Channel sender for outgoing messages
/// * `rx` - Channel receiver for incoming messages
///
/// # Returns
///
/// * `Ok(())` - The connection was handled successfully
/// * `Err(error)` - An error occurred while handling the connection
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::ws_transport::handle_ws_connection;
/// use actix_ws::{Session, MessageStream};
/// use tokio::sync::broadcast;
///
/// async fn example(session: Session, stream: MessageStream) -> Result<(), Box<dyn std::error::Error>> {
///     // Create channels for message passing
///     let (tx, rx) = broadcast::channel(100);
///     
///     // Handle the WebSocket connection
///     handle_ws_connection(session, stream, tx, rx).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn handle_ws_connection(
    mut session: Session,
    mut stream: actix_ws::MessageStream,
    tx: broadcast::Sender<Message>,
    mut rx: broadcast::Receiver<Message>,
) -> Result<()> {
    // Send messages from rx to the WebSocket
    let mut send_task = actix_web::rt::spawn(async move {
        while let Ok(message) = rx.recv().await {
            let json = serde_json::to_string(&message)?;
            session.text(json).await?;
        }
        Ok::<_, anyhow::Error>(())
    });

    // Receive messages from the WebSocket and send them to tx
    let mut recv_task = actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                WsMessage::Text(text) => {
                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                        if tx.send(message).is_err() {
                            break;
                        }
                    }
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
        Ok::<_, anyhow::Error>(())
    });

    // Wait for either task to complete
    tokio::select! {
        res = (&mut send_task) => match res {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(TransportError::new(
                TransportErrorCode::SendError,
                format!("Send task failed: {}", e)
            )),
            Err(e) => Err(TransportError::new(
                TransportErrorCode::SendError,
                format!("Send task join error: {}", e)
            )),
        }?,
        res = (&mut recv_task) => match res {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(TransportError::new(
                TransportErrorCode::ReceiveError,
                format!("Receive task failed: {}", e)
            )),
            Err(e) => Err(TransportError::new(
                TransportErrorCode::ReceiveError,
                format!("Receive task join error: {}", e)
            )),
        }?,
    }

    Ok(())
}
