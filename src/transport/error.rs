use std::fmt;
use thiserror::Error;
use std::error::Error as StdError;

/// Transport-specific error codes
///
/// These error codes provide detailed information about the specific type of error
/// that occurred in the transport layer. They are grouped by category for better organization.
///
/// # Error Code Ranges
///
/// - `-1000` to `-1099`: Connection errors
/// - `-1100` to `-1199`: Message errors
/// - `-1200` to `-1299`: Protocol errors
/// - `-1300` to `-1399`: Transport operation errors
/// - `-1400` to `-1499`: WebSocket specific errors
/// - `-1500` to `-1599`: SSE specific errors
/// - `-1900` to `-1999`: Generic errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportErrorCode {
    // Connection errors
    /// Connection to transport failed
    ConnectionFailed = -1000,
    /// Connection was closed
    ConnectionClosed = -1001,
    /// Connection timed out
    ConnectionTimeout = -1002,

    // Message errors
    /// Message size exceeds limit
    MessageTooLarge = -1100,
    /// Message format is invalid
    InvalidMessage = -1101,
    /// Failed to send message
    MessageSendFailed = -1102,
    /// Failed to receive message
    MessageReceiveFailed = -1103,

    // Protocol errors
    /// Protocol error occurred
    ProtocolError = -1200,
    /// Transport handshake failed
    HandshakeFailed = -1201,
    /// Authentication failed
    AuthenticationFailed = -1202,

    // Transport operation errors
    /// Error sending message
    SendError = -1300,
    /// Error opening transport
    OpenError = -1301,
    /// Error closing transport
    CloseError = -1302,
    /// Error receiving message
    ReceiveError = -1303,

    // Session errors
    /// Session has expired
    SessionExpired = -1310,
    /// Session is invalid
    SessionInvalid = -1311,
    /// Session not found
    SessionNotFound = -1312,

    // WebSocket specific
    /// WebSocket upgrade failed
    WebSocketUpgradeFailed = -1400,
    /// WebSocket protocol error
    WebSocketProtocolError = -1401,
    /// WebSocket frame error
    WebSocketFrameError = -1402,

    // SSE specific
    /// SSE connection failed to establish
    SseConnectionFailed = -1500,
    /// Error occurred while streaming SSE events
    SseStreamError = -1501,
    /// Failed to parse SSE event data
    SseParseError = -1502,

    // Generic errors
    /// Internal transport error
    InternalError = -1900,
    /// Transport operation timed out
    Timeout = -1901,
    /// Transport is in an invalid state
    InvalidState = -1902,
}

impl fmt::Display for TransportErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Connection errors
            Self::ConnectionFailed => write!(f, "Failed to establish connection"),
            Self::ConnectionClosed => write!(f, "Connection was closed"),
            Self::ConnectionTimeout => write!(f, "Connection timed out"),

            // Message errors
            Self::MessageTooLarge => write!(f, "Message exceeds size limit"),
            Self::InvalidMessage => write!(f, "Invalid message format"),
            Self::MessageSendFailed => write!(f, "Failed to send message"),
            Self::MessageReceiveFailed => write!(f, "Failed to receive message"),

            // Protocol errors
            Self::ProtocolError => write!(f, "Protocol error"),
            Self::HandshakeFailed => write!(f, "Handshake failed"),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),

            // Session errors
            Self::SessionExpired => write!(f, "Session has expired"),
            Self::SessionInvalid => write!(f, "Invalid session"),
            Self::SessionNotFound => write!(f, "Session not found"),

            // WebSocket specific
            Self::WebSocketUpgradeFailed => write!(f, "WebSocket upgrade failed"),
            Self::WebSocketProtocolError => write!(f, "WebSocket protocol error"),
            Self::WebSocketFrameError => write!(f, "WebSocket frame error"),

            // SSE specific
            Self::SseConnectionFailed => write!(f, "SSE connection failed"),
            Self::SseStreamError => write!(f, "SSE stream error"),
            Self::SseParseError => write!(f, "SSE parse error"),

            // Generic errors
            Self::InternalError => write!(f, "Internal error"),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::InvalidState => write!(f, "Invalid state"),
            Self::SendError => write!(f, "Send error"),
            Self::OpenError => write!(f, "Open error"),
            Self::CloseError => write!(f, "Close error"),
            Self::ReceiveError => write!(f, "Receive error"),
        }
    }
}

/// Transport-specific error type
///
/// This error type provides detailed information about errors that occur in the transport layer.
/// It includes specialized variants for different types of errors, such as JSON parsing errors,
/// I/O errors, WebSocket errors, and more.
///
/// # Examples
///
/// ```
/// use mcp_daemon::transport::{TransportError, TransportErrorCode};
///
/// // Create a simple transport error
/// let error = TransportError::new(
///     TransportErrorCode::ConnectionFailed,
///     "Failed to connect to server"
/// );
///
/// // Create an error with a source
/// let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
/// let error = TransportError::with_source(
///     TransportErrorCode::ConnectionFailed,
///     "Failed to connect to server",
///     Box::new(io_error) as Box<dyn std::error::Error + Send + Sync>
/// );
///
/// // Handle different error types
/// match error {
///     TransportError::Transport { code, message, .. } => {
///         println!("Transport error {}: {}", code, message);
///     }
///     TransportError::Io(err) => {
///         println!("I/O error: {}", err);
///     }
///     TransportError::Json(err) => {
///         println!("JSON error: {}", err);
///     }
///     _ => {
///         println!("Other error: {}", error);
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("{code}: {message}")]
    /// Transport-specific error
    Transport {
        /// The error code
        code: TransportErrorCode,
        /// Error message
        message: String,
        #[source]
        /// Optional error source
        source: Option<Box<dyn StdError + Send + Sync>>,
    },

    #[error("JSON error: {0}")]
    /// JSON serialization/deserialization error
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    /// I/O error
    Io(#[from] std::io::Error),

    #[error("WebSocket error: {0}")]
    /// WebSocket error
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("HTTP error: {0}")]
    /// HTTP error
    Http(#[from] reqwest::Error),

    #[error("Channel error: {0}")]
    /// Channel communication error
    Channel(String),

    #[error("UTF-8 error: {0}")]
    /// UTF-8 encoding/decoding error
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("System time error: {0}")]
    /// System time error
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("JWT error: {0}")]
    /// JWT token error
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for TransportError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Channel(err.to_string())
    }
}

impl<T> From<tokio::sync::broadcast::error::SendError<T>> for TransportError {
    fn from(err: tokio::sync::broadcast::error::SendError<T>) -> Self {
        Self::Channel(err.to_string())
    }
}

impl From<actix_web::Error> for TransportError {
    fn from(err: actix_web::Error) -> Self {
        Self::Transport {
            code: TransportErrorCode::InternalError,
            message: err.to_string(),
            source: None, // Don't store the source since actix_web::Error doesn't implement Send + Sync
        }
    }
}

impl From<tokio::task::JoinError> for TransportError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Transport {
            code: TransportErrorCode::InternalError,
            message: err.to_string(),
            source: None,
        }
    }
}

impl TransportError {
    /// Create a new transport error
    ///
    /// # Arguments
    ///
    /// * `code` - The error code
    /// * `message` - The error message
    ///
    /// # Returns
    ///
    /// A new `TransportError` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::{TransportError, TransportErrorCode};
    ///
    /// let error = TransportError::new(
    ///     TransportErrorCode::ConnectionFailed,
    ///     "Failed to connect to server"
    /// );
    /// ```
    pub fn new(code: TransportErrorCode, message: impl Into<String>) -> Self {
        Self::Transport {
            code,
            message: message.into(),
            source: None,
        }
    }

    /// Create a new transport error with source
    ///
    /// # Arguments
    ///
    /// * `code` - The error code
    /// * `message` - The error message
    /// * `source` - The error source
    ///
    /// # Returns
    ///
    /// A new `TransportError` instance with the specified source
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::{TransportError, TransportErrorCode};
    /// use std::error::Error;
    ///
    /// let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    /// let error = TransportError::with_source(
    ///     TransportErrorCode::ConnectionFailed,
    ///     "Failed to connect to server",
    ///     Box::new(io_error) as Box<dyn Error + Send + Sync>
    /// );
    /// ```
    pub fn with_source(
        code: TransportErrorCode,
        message: impl Into<String>,
        source: impl Into<Box<dyn StdError + Send + Sync>>,
    ) -> Self {
        Self::Transport {
            code,
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Get the error code if this is a transport error
    ///
    /// # Returns
    ///
    /// The error code if this is a transport error, or `None` if this is not a transport error
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::{TransportError, TransportErrorCode};
    ///
    /// let error = TransportError::new(TransportErrorCode::ConnectionFailed, "Failed to connect");
    /// assert_eq!(error.code(), Some(TransportErrorCode::ConnectionFailed));
    ///
    /// let io_error = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
    /// let error = TransportError::Io(io_error);
    /// assert_eq!(error.code(), None);
    /// ```
    pub fn code(&self) -> Option<TransportErrorCode> {
        match self {
            Self::Transport { code, .. } => Some(*code),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(TransportErrorCode::ConnectionFailed as i32, -1000);
        assert_eq!(TransportErrorCode::MessageTooLarge as i32, -1100);
        assert_eq!(TransportErrorCode::ProtocolError as i32, -1200);
        assert_eq!(TransportErrorCode::SendError as i32, -1300);
        assert_eq!(TransportErrorCode::SessionExpired as i32, -1310);
    }

    #[test]
    fn test_error_display() {
        let error = TransportError::new(TransportErrorCode::ConnectionFailed, "Failed to connect");
        assert_eq!(error.to_string(), "Failed to establish connection: Failed to connect");

        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
        let error = TransportError::with_source(
            TransportErrorCode::ConnectionFailed,
            "Failed to connect",
            Box::new(io_error) as Box<dyn StdError + Send + Sync>,
        );
        assert_eq!(error.to_string(), "Failed to establish connection: Failed to connect");
    }

    #[test]
    fn test_error_code() {
        let error = TransportError::new(TransportErrorCode::ConnectionFailed, "Failed to connect");
        assert_eq!(error.code(), Some(TransportErrorCode::ConnectionFailed));

        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "JSON error");
        let error = TransportError::Json(serde_json::Error::io(io_error));
        assert_eq!(error.code(), None);
    }
}
