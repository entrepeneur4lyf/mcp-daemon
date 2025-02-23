use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::fmt;
use crate::server::error::ServerError;

/// A notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Notification {
    #[serde(rename = "notifications/cancelled")]
    /// Request cancelled notification
    Cancelled(CancelledParams),

    #[serde(rename = "notifications/progress")]
    /// Progress update notification
    Progress(ProgressParams),

    #[serde(rename = "notifications/initialized")]
    /// Server initialized notification
    Initialized,

    #[serde(rename = "notifications/roots/list_changed")]
    /// Roots list changed notification
    RootsListChanged,

    #[serde(rename = "notifications/logging/message")]
    /// Logging message notification
    LoggingMessage(LoggingMessageParams),

    #[serde(rename = "notifications/resources/updated")]
    /// Resource updated notification
    ResourceUpdated(ResourceUpdatedParams),

    #[serde(rename = "notifications/resources/list_changed")]
    /// Resource list changed notification
    ResourceListChanged,

    #[serde(rename = "notifications/tools/list_changed")]
    /// Tool list changed notification
    ToolListChanged,

    #[serde(rename = "notifications/prompts/list_changed")]
    /// Prompt list changed notification
    PromptListChanged,
}

/// Parameters for a cancelled notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledParams {
    /// The ID of the request that was cancelled
    pub request_id: String,
    /// Optional reason for cancellation
    pub reason: Option<String>,
}

/// Parameters for a progress notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressParams {
    /// The ID of the request this progress is for
    pub request_id: String,
    /// Progress value between 0 and 1
    pub progress: f32,
    /// Optional message describing the current progress
    pub message: Option<String>,
}

/// Logging level for logging messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    /// Debug level log message
    Debug,
    /// Info level log message
    Info,
    /// Warning level log message
    Warn,
    /// Error level log message
    Error,
}

impl fmt::Display for LoggingLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoggingLevel::Debug => write!(f, "debug"),
            LoggingLevel::Info => write!(f, "info"),
            LoggingLevel::Warn => write!(f, "warn"),
            LoggingLevel::Error => write!(f, "error"),
        }
    }
}

/// Parameters for a logging message notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingMessageParams {
    /// The logging level
    pub level: LoggingLevel,
    /// The message text
    pub message: String,
}

/// Parameters for a resource updated notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdatedParams {
    /// The URI of the resource that was updated
    pub uri: String,
}

type Result<T> = std::result::Result<T, ServerError>;

/// A notification sender for sending notifications to clients
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// Send a notification
    async fn send(&self, notification: Notification) -> Result<()>;
}

/// A notification handler for receiving notifications
#[async_trait]
pub trait NotificationHandler: Send + Sync {
    /// Handle a notification
    async fn handle(&self, notification: Notification) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_serialization() {
        let notification = Notification::Cancelled(CancelledParams {
            request_id: "123".to_string(),
            reason: Some("User cancelled".to_string()),
        });

        let json = serde_json::to_string(&notification).unwrap();
        let deserialized: Notification = serde_json::from_str(&json).unwrap();

        match deserialized {
            Notification::Cancelled(params) => {
                assert_eq!(params.request_id, "123");
                assert_eq!(params.reason, Some("User cancelled".to_string()));
            }
            _ => panic!("Wrong notification type"),
        }
    }

    #[test]
    fn test_logging_level_display() {
        assert_eq!(LoggingLevel::Debug.to_string(), "debug");
        assert_eq!(LoggingLevel::Info.to_string(), "info");
        assert_eq!(LoggingLevel::Warn.to_string(), "warn");
        assert_eq!(LoggingLevel::Error.to_string(), "error");
    }
}
