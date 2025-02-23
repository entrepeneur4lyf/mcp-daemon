//! Transport layer for the MCP protocol
//! handles the serialization and deserialization of message
//! handles send and receive of messages
//! defines transport layer types
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

/// only JsonRpcMessage is supported for now
/// <https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/messages/>
pub type Message = JsonRpcMessage;

#[async_trait]
/// Transport layer trait for handling MCP protocol communication
///
/// Implementations of this trait handle the sending and receiving of messages
/// over various transport protocols (stdio, websockets, SSE, etc.)
pub trait Transport: Send + Sync + 'static {
    /// Send a message to the transport
    ///
    /// # Errors
    /// - `TransportErrorCode::ConnectionClosed` if the connection is closed
    /// - `TransportErrorCode::MessageTooLarge` if the message exceeds size limits
    /// - `TransportErrorCode::MessageSendFailed` if sending fails
    async fn send(&self, message: &Message) -> Result<()>;

    /// Receive a message from the transport
    /// This is a blocking call that returns None when the connection is closed
    ///
    /// # Errors
    /// - `TransportErrorCode::ConnectionClosed` if the connection is closed
    /// - `TransportErrorCode::MessageReceiveFailed` if receiving fails
    /// - `TransportErrorCode::InvalidMessage` if message parsing fails
    async fn receive(&self) -> Result<Option<Message>>;

    /// Open the transport connection
    ///
    /// # Errors
    /// - `TransportErrorCode::ConnectionFailed` if connection fails
    /// - `TransportErrorCode::HandshakeFailed` if protocol handshake fails
    /// - `TransportErrorCode::AuthenticationFailed` if auth fails
    async fn open(&self) -> Result<()>;

    /// Close the transport connection
    ///
    /// # Errors
    /// - `TransportErrorCode::ConnectionClosed` if already closed
    /// - `TransportErrorCode::InvalidState` if in invalid state
    async fn close(&self) -> Result<()>;
}

/// Request ID type
pub type RequestId = u64;

/// JSON RPC version type
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
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A JSON-RPC message that can be either a request, response, or notification
///
/// This enum represents the three possible message types in the JSON-RPC protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// JSON-RPC response message
    Response(JsonRpcResponse),
    /// JSON-RPC request message
    Request(JsonRpcRequest),
    /// JSON-RPC notification message
    Notification(JsonRpcNotification),
}

// json rpc types
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A JSON-RPC request message
///
/// Contains a method name to invoke and optional parameters
pub struct JsonRpcRequest {
    /// The request ID
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
/// Similar to a request but does not expect a response
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
/// Contains either a result or an error in response to a request
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
/// Contains error details when a request fails
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional error data
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
