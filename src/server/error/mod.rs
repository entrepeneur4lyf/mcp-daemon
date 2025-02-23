use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// Standard JSON-RPC error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Standard JSON-RPC error codes
    /// Invalid JSON was received by the server
    ParseError = -32700,
    /// The JSON sent is not a valid Request object
    InvalidRequest = -32600,
    /// The method does not exist / is not available
    MethodNotFound = -32601,
    /// Invalid method parameter(s)
    InvalidParams = -32602,
    /// Internal JSON-RPC error
    InternalError = -32603,

    // MCP-specific error codes
    /// The connection was closed unexpectedly
    ConnectionClosed = -1,
    /// The request timed out
    RequestTimeout = -2,

    // Server-specific error codes
    /// The server has not been initialized
    ServerNotInitialized = -1000,
    /// The server capabilities are invalid
    InvalidCapabilities = -1001,
    /// A required handler has not been set
    HandlerNotSet = -1002,
    /// An error occurred during server shutdown
    ShutdownError = -1003,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::ParseError => write!(f, "Parse error"),
            ErrorCode::InvalidRequest => write!(f, "Invalid request"),
            ErrorCode::MethodNotFound => write!(f, "Method not found"),
            ErrorCode::InvalidParams => write!(f, "Invalid parameters"),
            ErrorCode::InternalError => write!(f, "Internal error"),
            ErrorCode::ConnectionClosed => write!(f, "Connection closed"),
            ErrorCode::RequestTimeout => write!(f, "Request timeout"),
            ErrorCode::ServerNotInitialized => write!(f, "Server not initialized"),
            ErrorCode::InvalidCapabilities => write!(f, "Invalid capabilities"),
            ErrorCode::HandlerNotSet => write!(f, "Required handler not set"),
            ErrorCode::ShutdownError => write!(f, "Error during shutdown"),
        }
    }
}

impl TryFrom<i32> for ErrorCode {
    type Error = String;

    fn try_from(code: i32) -> Result<Self, Self::Error> {
        match code {
            -32700 => Ok(ErrorCode::ParseError),
            -32600 => Ok(ErrorCode::InvalidRequest),
            -32601 => Ok(ErrorCode::MethodNotFound),
            -32602 => Ok(ErrorCode::InvalidParams),
            -32603 => Ok(ErrorCode::InternalError),
            -1 => Ok(ErrorCode::ConnectionClosed),
            -2 => Ok(ErrorCode::RequestTimeout),
            -1000 => Ok(ErrorCode::ServerNotInitialized),
            -1001 => Ok(ErrorCode::InvalidCapabilities),
            -1002 => Ok(ErrorCode::HandlerNotSet),
            -1003 => Ok(ErrorCode::ShutdownError),
            _ => Err(format!("Invalid error code: {}", code)),
        }
    }
}

/// A JSON-RPC error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// The error code
    pub code: i32,
    /// A short description of the error
    pub message: String,
    /// Additional information about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code as i32,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new JSON-RPC error with additional data
    pub fn with_data(
        code: ErrorCode,
        message: impl Into<String>,
        data: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            code: code as i32,
            message: message.into(),
            data: Some(data.into()),
        }
    }
}

/// Server-specific error type
#[derive(Debug)]
pub enum ServerError {
    /// JSON-RPC protocol error
    JsonRpc(JsonRpcError),
    /// Transport error
    Transport(crate::transport::TransportError),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// I/O error
    Io(std::io::Error),
    /// Server error with code and message
    Server {
        /// The error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Optional error source
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

impl ServerError {
    /// Create a new server error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Server {
            code,
            message: message.into(),
            source: None,
        }
    }

    /// Create a new server error with source
    pub fn with_source(
        code: ErrorCode,
        message: impl Into<String>,
        source: impl Into<Box<dyn StdError + Send + Sync>>,
    ) -> Self {
        Self::Server {
            code,
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Get the error code if this is a server error
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::Server { code, .. } => Some(*code),
            Self::JsonRpc(e) => ErrorCode::try_from(e.code).ok(),
            _ => None,
        }
    }
}

impl fmt::Display for ServerError {
    /// Format the error message
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonRpc(e) => write!(f, "JSON-RPC error: {} (code {})", e.message, e.code),
            Self::Transport(e) => write!(f, "Transport error: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Server { code, message, .. } => write!(f, "{}: {}", code, message),
        }
    }
}

impl StdError for ServerError {
    /// Get the error source
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::JsonRpc(_) => None,
            Self::Transport(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Server { source, .. } => source.as_ref().map(|s| s.as_ref() as &(dyn StdError + 'static)),
        }
    }
}

impl From<JsonRpcError> for ServerError {
    /// Convert a JSON-RPC error to a server error
    fn from(err: JsonRpcError) -> Self {
        Self::JsonRpc(err)
    }
}

impl From<crate::transport::TransportError> for ServerError {
    /// Convert a transport error to a server error
    fn from(err: crate::transport::TransportError) -> Self {
        Self::Transport(err)
    }
}

impl From<serde_json::Error> for ServerError {
    /// Convert a JSON error to a server error
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<std::io::Error> for ServerError {
    /// Convert an I/O error to a server error
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorCode::ParseError as i32, -32700);
        assert_eq!(ErrorCode::InvalidRequest as i32, -32600);
        assert_eq!(ErrorCode::MethodNotFound as i32, -32601);
        assert_eq!(ErrorCode::InvalidParams as i32, -32602);
        assert_eq!(ErrorCode::InternalError as i32, -32603);
        assert_eq!(ErrorCode::ServerNotInitialized as i32, -1000);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(ErrorCode::ParseError.to_string(), "Parse error");
        assert_eq!(ErrorCode::InvalidRequest.to_string(), "Invalid request");
        assert_eq!(ErrorCode::MethodNotFound.to_string(), "Method not found");
        assert_eq!(ErrorCode::InvalidParams.to_string(), "Invalid parameters");
        assert_eq!(ErrorCode::InternalError.to_string(), "Internal error");
        assert_eq!(ErrorCode::ServerNotInitialized.to_string(), "Server not initialized");
    }

    #[test]
    fn test_json_rpc_error() {
        let error = JsonRpcError::new(ErrorCode::ParseError, "Failed to parse JSON");
        assert_eq!(error.code, -32700);
        assert_eq!(error.message, "Failed to parse JSON");
        assert!(error.data.is_none());

        let error = JsonRpcError::with_data(
            ErrorCode::InvalidParams,
            "Invalid parameters",
            serde_json::json!({
                "missing": ["param1", "param2"]
            }),
        );
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid parameters");
        assert!(error.data.is_some());
    }

    #[test]
    fn test_server_error() {
        let error = ServerError::new(ErrorCode::ServerNotInitialized, "Server not ready");
        assert_eq!(error.code(), Some(ErrorCode::ServerNotInitialized));
        assert_eq!(error.to_string(), "Server not initialized: Server not ready");

        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
        let error = ServerError::with_source(
            ErrorCode::InternalError,
            "Internal server error",
            io_error,
        );
        assert_eq!(error.code(), Some(ErrorCode::InternalError));
        assert!(error.source().is_some());
    }

    #[test]
    fn test_error_code_conversion() {
        assert_eq!(ErrorCode::try_from(-32700), Ok(ErrorCode::ParseError));
        assert_eq!(ErrorCode::try_from(-32600), Ok(ErrorCode::InvalidRequest));
        assert!(ErrorCode::try_from(0).is_err());
    }
}
