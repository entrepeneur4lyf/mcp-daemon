use std::fmt;
use thiserror::Error;
use std::error::Error as StdError;

/// Transport-specific error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportErrorCode {
    // Connection errors
    ConnectionFailed = -1000,
    ConnectionClosed = -1001,
    ConnectionTimeout = -1002,

    // Message errors
    MessageTooLarge = -1100,
    InvalidMessage = -1101,
    MessageSendFailed = -1102,
    MessageReceiveFailed = -1103,

    // Protocol errors
    ProtocolError = -1200,
    HandshakeFailed = -1201,
    
    // Transport operation errors
    SendError = -1300,
    OpenError = -1301,
    CloseError = -1302,
    ReceiveError = -1303,
    AuthenticationFailed = -1202,

    // Session errors
    SessionExpired = -1310,
    SessionInvalid = -1311,
    SessionNotFound = -1312,

    // WebSocket specific
    WebSocketUpgradeFailed = -1400,
    WebSocketProtocolError = -1401,
    WebSocketFrameError = -1402,

    // SSE specific
    SseConnectionFailed = -1500,
    SseStreamError = -1501,
    SseParseError = -1502,

    // Generic errors
    InternalError = -1900,
    Timeout = -1901,
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
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("{code}: {message}")]
    Transport {
        code: TransportErrorCode,
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("JWT error: {0}")]
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
    pub fn new(code: TransportErrorCode, message: impl Into<String>) -> Self {
        Self::Transport {
            code,
            message: message.into(),
            source: None,
        }
    }

    /// Create a new transport error with source
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
