//! HTTP transport types for the Model Context Protocol
//!
//! This module provides the transport layer for HTTP-based communication between MCP clients and servers.
//! It includes implementations for both Server-Sent Events (SSE) and WebSocket transports, allowing
//! for flexible communication patterns depending on the requirements of the application.
//!
//! The HTTP transport layer serves as a wrapper around the specific transport implementations,
//! providing a unified interface for sending and receiving messages regardless of the underlying
//! transport mechanism.
//!
//! # Features
//!
//! - Support for Server-Sent Events (SSE) for server-to-client streaming
//! - Support for WebSockets for bidirectional communication
//! - Unified Transport trait implementation for consistent API
//! - Async/await support for non-blocking I/O
//!
//! # Examples
//!
//! ```
//! use mcp_daemon::transport::{ServerHttpTransport, Transport};
//! use mcp_daemon::transport::sse_transport::ServerSseTransport;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an SSE transport
//!     let (sse_transport, _) = ServerSseTransport::new_with_responder(100);
//!     
//!     // Wrap it in an HTTP transport
//!     let transport = ServerHttpTransport::Sse(sse_transport);
//!     
//!     // Use the transport
//!     transport.open().await?;
//!     
//!     // Send a message
//!     let message = serde_json::json!({
//!         "jsonrpc": "2.0",
//!         "method": "example",
//!         "params": {}
//!     });
//!     transport.send(&message).await?;
//!     
//!     // Close the transport
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```

use super::{Message, Result, Transport};
use super::ws_transport::{ClientWsTransport, ServerWsTransport};
use super::sse_transport::ServerSseTransport;
use async_trait::async_trait;

/// Server-side HTTP transport variants
///
/// This enum represents the different HTTP-based transport mechanisms
/// that can be used on the server side of an MCP connection. It currently
/// supports Server-Sent Events (SSE) and WebSockets.
///
/// The enum implements the `Transport` trait, providing a unified interface
/// for sending and receiving messages regardless of the underlying transport.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ServerHttpTransport, Transport};
/// use mcp_daemon::transport::sse_transport::ServerSseTransport;
///
/// // Create an SSE transport
/// let (sse_transport, _) = ServerSseTransport::new_with_responder(100);
/// 
/// // Wrap it in an HTTP transport
/// let transport = ServerHttpTransport::Sse(sse_transport);
/// ```
#[derive(Debug, Clone)]
pub enum ServerHttpTransport {
    /// Server-Sent Events transport
    Sse(ServerSseTransport),
    /// WebSocket transport
    Ws(ServerWsTransport),
}

/// Client-side HTTP transport variants
///
/// This enum represents the different HTTP-based transport mechanisms
/// that can be used on the client side of an MCP connection. It currently
/// supports WebSockets, with potential for additional transport types in the future.
///
/// The enum implements the `Transport` trait, providing a unified interface
/// for sending and receiving messages regardless of the underlying transport.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{ClientHttpTransport, Transport};
/// use mcp_daemon::transport::ws_transport::ClientWsTransport;
///
/// // Create a WebSocket transport
/// let ws_transport = ClientWsTransport::new("ws://localhost:8080/ws");
/// 
/// // Wrap it in an HTTP transport
/// let transport = ClientHttpTransport::Ws(ws_transport);
/// ```
#[derive(Debug, Clone)]
pub enum ClientHttpTransport {
    /// WebSocket transport
    Ws(ClientWsTransport),
}

#[async_trait]
impl Transport for ServerHttpTransport {
    /// Sends a message through the transport
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn send(&self, message: &Message) -> Result<()> {
        match self {
            Self::Sse(transport) => transport.send(message).await,
            Self::Ws(transport) => transport.send(message).await,
        }
    }

    /// Receives a message from the transport
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Returns
    ///
    /// A Result containing an Option with the received message, or None if no message is available
    async fn receive(&self) -> Result<Option<Message>> {
        match self {
            Self::Sse(transport) => transport.receive().await,
            Self::Ws(transport) => transport.receive().await,
        }
    }

    /// Opens the transport connection
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn open(&self) -> Result<()> {
        match self {
            Self::Sse(transport) => transport.open().await,
            Self::Ws(transport) => transport.open().await,
        }
    }

    /// Closes the transport connection
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn close(&self) -> Result<()> {
        match self {
            Self::Sse(transport) => transport.close().await,
            Self::Ws(transport) => transport.close().await,
        }
    }
}

#[async_trait]
impl Transport for ClientHttpTransport {
    /// Sends a message through the transport
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn send(&self, message: &Message) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.send(message).await,
        }
    }

    /// Receives a message from the transport
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Returns
    ///
    /// A Result containing an Option with the received message, or None if no message is available
    async fn receive(&self) -> Result<Option<Message>> {
        match self {
            Self::Ws(transport) => transport.receive().await,
        }
    }

    /// Opens the transport connection
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn open(&self) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.open().await,
        }
    }

    /// Closes the transport connection
    ///
    /// This method delegates to the underlying transport implementation
    /// based on the variant of the enum.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn close(&self) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.close().await,
        }
    }
}
