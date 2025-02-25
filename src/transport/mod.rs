//! # Transport Layer for MCP Protocol
//!
//! The transport module provides implementations of various transport mechanisms for MCP communication.
//! It defines a common interface for all transport types and implements specific transport strategies.
//!
//! ## Overview
//!
//! The transport layer is responsible for the actual transmission of MCP messages between clients and servers.
//! It abstracts away the details of the underlying communication protocol, allowing the rest of the system
//! to work with a consistent interface regardless of the transport mechanism being used.
//!
//! ## Transport Types
//!
//! This module provides several transport implementations:
//!
//! * **Stdio Transport**: Uses standard input/output for communication, ideal for command-line tools and process-based communication.
//! * **In-Memory Transport**: Uses Tokio channels for efficient inter-process communication within the same application.
//! * **SSE Transport**: Implements Server-Sent Events for unidirectional server-to-client communication with automatic keep-alive.
//! * **WebSocket Transport**: Provides full-duplex communication with comprehensive error handling and connection management.
//! * **HTTP Transport**: Provides a base implementation for HTTP-based transports like SSE and WebSockets.
//!
//! ## Message Types
//!
//! The transport layer uses JSON-RPC 2.0 for message exchange, with these main message types:
//!
//! * **Requests**: Messages that expect a response, containing a method name and optional parameters.
//! * **Responses**: Replies to requests, containing either a result or an error.
//! * **Notifications**: One-way messages that don't expect a response.
//!
//! ## Error Handling
//!
//! All transport implementations provide detailed error information through the `TransportError` type.
//! This includes specific error codes and messages for various failure scenarios, such as connection
//! failures, timeout errors, and protocol violations.
//!
//! ## Usage Examples
//!
//! ### Client-Side WebSocket Transport
//!
//! ```rust,no_run
//! use mcp_daemon::transport::ClientWsTransportBuilder;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a WebSocket transport
//! let transport = ClientWsTransportBuilder::new("ws://localhost:3004/ws".to_string())
//!     .with_timeout(Duration::from_secs(30))
//!     .build();
//!
//! // Open the transport connection
//! transport.open().await?;
//!
//! // Use the transport...
//!
//! // Close the transport when done
//! transport.close().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Server-Side Stdio Transport
//!
//! ```rust,no_run
//! use mcp_daemon::transport::{StdioServerTransport, Transport, Message};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a stdio transport
//! let transport = StdioServerTransport::new();
//!
//! // Open the transport
//! transport.open().await?;
//!
//! // Receive a message
//! if let Some(message) = transport.receive().await? {
//!     // Process the message...
//!
//!     // Send a response
//!     transport.send(&message).await?;
//! }
//!
//! // Close the transport when done
//! transport.close().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Related Modules
//!
//! * [`client`](crate::client): Uses transports to communicate with MCP servers
//! * [`server`](crate::server): Uses transports to receive and respond to client requests
//! * [`protocol`](crate::protocol): Defines the protocol layer that sits on top of the transport

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod error;
pub use error::{TransportError, TransportErrorCode};

/// Result type for transport operations
pub type Result<T> = std::result::Result<T, TransportError>;

mod stdio_transport;
pub use stdio_transport::*;
mod inmemory_transport;
pub use inmemory_transport::*;
mod sse_transport;
pub use sse_transport::*;
mod ws_transport;
pub use ws_transport::*;
mod http_transport;
pub use http_transport::*;

/// Message type used for MCP protocol communication
///
/// Currently, only JsonRpcMessage is supported, as specified in the MCP specification:
/// <https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/messages/>
pub type Message = JsonRpcMessage;

#[async_trait]
/// Transport layer trait for handling MCP protocol communication
///
/// Implementations of this trait handle the sending and receiving of messages
/// over various transport protocols (stdio, websockets, SSE, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// use mcp_daemon::transport::{Transport, Message};
/// use async_trait::async_trait;
///
/// struct MyCustomTransport {
///     // Transport-specific fields
/// }
///
/// #[async_trait]
/// impl Transport for MyCustomTransport {
///     async fn send(&self, message: &Message) -> Result<(), mcp_daemon::transport::TransportError> {
///         // Implementation for sending a message
///         Ok(())
///     }
///
///     async fn receive(&self) -> Result<Option<Message>, mcp_daemon::transport::TransportError> {
///         // Implementation for receiving a message
///         todo!()
///     }
///
///     async fn open(&self) -> Result<(), mcp_daemon::transport::TransportError> {
///         // Implementation for opening the transport
///         Ok(())
///     }
///
///     async fn close(&self) -> Result<(), mcp_daemon::transport::TransportError> {
///         // Implementation for closing the transport
///         Ok(())
///     }
/// }
/// ```
pub trait Transport: Send + Sync + 'static {
    /// Send a message to the transport
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the message was sent successfully, or a `TransportError` if it failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - `TransportErrorCode::ConnectionClosed` if the connection is closed
    /// - `TransportErrorCode::MessageTooLarge` if the message exceeds size limits
    /// - `TransportErrorCode::MessageSendFailed` if sending fails
    async fn send(&self, message: &Message) -> Result<()>;

    /// Receive a message from the transport
    ///
    /// This is a blocking call that returns None when the connection is closed
    ///
    /// # Returns
    ///
    /// A `Result` containing `Option<Message>` if a message was received successfully, or a `TransportError` if it failed.
    /// Returns `None` when the connection is closed normally.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - `TransportErrorCode::ConnectionClosed` if the connection is closed unexpectedly
    /// - `TransportErrorCode::MessageReceiveFailed` if receiving fails
    /// - `TransportErrorCode::InvalidMessage` if message parsing fails
    async fn receive(&self) -> Result<Option<Message>>;

    /// Open the transport connection
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the connection was opened successfully, or a `TransportError` if it failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - `TransportErrorCode::ConnectionFailed` if connection fails
    /// - `TransportErrorCode::HandshakeFailed` if protocol handshake fails
    /// - `TransportErrorCode::AuthenticationFailed` if auth fails
    async fn open(&self) -> Result<()>;

    /// Close the transport connection
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the connection was closed successfully, or a `TransportError` if it failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - `TransportErrorCode::ConnectionClosed` if already closed
    /// - `TransportErrorCode::InvalidState` if in invalid state
    async fn close(&self) -> Result<()>;
}

/// Request ID type used in JSON-RPC messages
pub type RequestId = u64;

/// JSON RPC version type
///
/// Represents the version of the JSON-RPC protocol being used.
/// The default version is "2.0".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct JsonRpcVersion(String);

impl Default for JsonRpcVersion {
    fn default() -> Self {
        JsonRpcVersion("2.0".to_owned())
    }
}

impl JsonRpcVersion {
    /// Returns the string representation of the JSON-RPC version
    ///
    /// # Returns
    ///
    /// A string slice containing the version (e.g., "2.0")
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A JSON-RPC message that can be either a request, response, or notification
///
/// This enum represents the three possible message types in the JSON-RPC protocol.
/// The MCP protocol uses JSON-RPC 2.0 for message exchange.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// JSON-RPC response message
    ///
    /// Contains either a result or an error in response to a request
    Response(JsonRpcResponse),
    
    /// JSON-RPC request message
    ///
    /// Contains a method name to invoke and optional parameters
    Request(JsonRpcRequest),
    
    /// JSON-RPC notification message
    ///
    /// Similar to a request but does not expect a response
    Notification(JsonRpcNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A JSON-RPC request message
///
/// Contains a method name to invoke and optional parameters.
/// Requests expect a response from the recipient.
pub struct JsonRpcRequest {
    /// The request ID
    ///
    /// Used to match responses to requests
    pub id: RequestId,
    
    /// The method name to invoke
    pub method: String,
    
    /// Optional parameters for the method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    
    /// The JSON-RPC version
    pub jsonrpc: JsonRpcVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(default)]
/// A JSON-RPC notification message
///
/// Similar to a request but does not expect a response.
/// Notifications are one-way messages.
pub struct JsonRpcNotification {
    /// The method name to invoke
    pub method: String,
    
    /// Optional parameters for the method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    
    /// The JSON-RPC version
    pub jsonrpc: JsonRpcVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// A JSON-RPC response message
///
/// Contains either a result or an error in response to a request.
/// The response is matched to the request using the ID.
pub struct JsonRpcResponse {
    /// The request ID this response corresponds to
    pub id: RequestId,
    
    /// The result of the request, if successful
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    /// The error, if the request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    
    /// The JSON-RPC version
    pub jsonrpc: JsonRpcVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// A JSON-RPC error object
///
/// Contains error details when a request fails.
/// Includes a code, message, and optional additional data.
pub struct JsonRpcError {
    /// Error code
    ///
    /// Standard JSON-RPC error codes are in the range -32768 to -32000.
    /// Application-specific error codes should be outside this range.
    pub code: i32,
    
    /// Error message
    ///
    /// A short description of the error
    pub message: String,
    
    /// Optional additional error data
    ///
    /// Can contain more detailed information about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize_initialize_request() {
        let json = r#"{"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"claude-ai","version":"0.1.0"}},"jsonrpc":"2.0","id":0}"#;

        let message: Message = serde_json::from_str(json).unwrap();
        match message {
            JsonRpcMessage::Request(req) => {
                assert_eq!(req.jsonrpc.as_str(), "2.0");
                assert_eq!(req.id, 0);
                assert_eq!(req.method, "initialize");

                // Verify params exist and are an object
                let params = req.params.expect("params should exist");
                assert!(params.is_object());

                let params_obj = params.as_object().unwrap();
                assert_eq!(params_obj["protocolVersion"], "2024-11-05");

                let client_info = params_obj["clientInfo"].as_object().unwrap();
                assert_eq!(client_info["name"], "claude-ai");
                assert_eq!(client_info["version"], "0.1.0");
            }
            _ => panic!("Expected Request variant"),
        }
    }
}