//! # MCP Notification System
//!
//! This module provides types and traits for MCP's notification system, which enables
//! asynchronous communication between servers and clients.
//!
//! ## Overview
//!
//! Notifications in MCP are one-way messages that don't expect a response. They are used for:
//!
//! * Signaling state changes (resource updates, list changes)
//! * Reporting progress on long-running operations
//! * Logging messages from server to client
//! * Cancellation notifications
//! * Initialization confirmations
//!
//! Unlike requests, notifications don't have an ID and don't expect a response.
//!
//! ## Examples
//!
//! ```rust
//! use mcp_daemon::server::notifications::{
//!     Notification, LoggingMessageParams, LoggingLevel, ResourceUpdatedParams
//! };
//!
//! // Create a logging notification
//! let logging_notification = Notification::LoggingMessage(LoggingMessageParams {
//!     level: LoggingLevel::Info,
//!     message: "Server started successfully".to_string(),
//! });
//!
//! // Create a resource updated notification
//! let resource_notification = Notification::ResourceUpdated(ResourceUpdatedParams {
//!     uri: "file:///path/to/resource".to_string(),
//! });
//!
//! // Create a list changed notification
//! let list_notification = Notification::ResourceListChanged;
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::server::error`] - Error handling
//! * [`crate::protocol`] - Protocol implementation

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::fmt;
use crate::server::error::ServerError;

/// A notification message
///
/// Represents a one-way message from server to client or client to server
/// that doesn't expect a response.
///
/// # Variants
///
/// * `Cancelled` - Request cancelled notification
/// * `Progress` - Progress update notification
/// * `Initialized` - Server initialized notification
/// * `RootsListChanged` - Roots list changed notification
/// * `LoggingMessage` - Logging message notification
/// * `ResourceUpdated` - Resource updated notification
/// * `ResourceListChanged` - Resource list changed notification
/// * `ToolListChanged` - Tool list changed notification
/// * `PromptListChanged` - Prompt list changed notification
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::notifications::{
///     Notification, CancelledParams, ProgressParams, LoggingMessageParams,
///     LoggingLevel, ResourceUpdatedParams
/// };
///
/// // Cancelled notification
/// let cancelled = Notification::Cancelled(CancelledParams {
///     request_id: "req-123".to_string(),
///     reason: Some("User cancelled".to_string()),
/// });
///
/// // Progress notification
/// let progress = Notification::Progress(ProgressParams {
///     request_id: "req-123".to_string(),
///     progress: 0.5,
///     message: Some("50% complete".to_string()),
/// });
///
/// // Initialized notification
/// let initialized = Notification::Initialized;
///
/// // Logging message notification
/// let logging = Notification::LoggingMessage(LoggingMessageParams {
///     level: LoggingLevel::Info,
///     message: "Operation completed".to_string(),
/// });
/// ```
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
///
/// Contains information about a cancelled request.
///
/// # Fields
///
/// * `request_id` - The ID of the request that was cancelled
/// * `reason` - Optional reason for cancellation
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::notifications::CancelledParams;
///
/// let params = CancelledParams {
///     request_id: "req-123".to_string(),
///     reason: Some("User cancelled".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledParams {
    /// The ID of the request that was cancelled
    pub request_id: String,
    /// Optional reason for cancellation
    pub reason: Option<String>,
}

/// Parameters for a progress notification
///
/// Contains information about the progress of a long-running operation.
///
/// # Fields
///
/// * `request_id` - The ID of the request this progress is for
/// * `progress` - Progress value between 0 and 1
/// * `message` - Optional message describing the current progress
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::notifications::ProgressParams;
///
/// let params = ProgressParams {
///     request_id: "req-123".to_string(),
///     progress: 0.75,
///     message: Some("Processing file 3 of 4".to_string()),
/// };
/// ```
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
///
/// Defines the severity level of logging messages.
///
/// # Variants
///
/// * `Debug` - Debug level log message (lowest severity)
/// * `Info` - Info level log message
/// * `Warn` - Warning level log message
/// * `Error` - Error level log message (highest severity)
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::notifications::LoggingLevel;
///
/// let debug_level = LoggingLevel::Debug;
/// let info_level = LoggingLevel::Info;
/// let warn_level = LoggingLevel::Warn;
/// let error_level = LoggingLevel::Error;
///
/// // Convert to string
/// assert_eq!(debug_level.to_string(), "debug");
/// ```
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
///
/// Contains a log message with its severity level.
///
/// # Fields
///
/// * `level` - The logging level
/// * `message` - The message text
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::notifications::{LoggingMessageParams, LoggingLevel};
///
/// let params = LoggingMessageParams {
///     level: LoggingLevel::Info,
///     message: "Server started successfully".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingMessageParams {
    /// The logging level
    pub level: LoggingLevel,
    /// The message text
    pub message: String,
}

/// Parameters for a resource updated notification
///
/// Contains information about an updated resource.
///
/// # Fields
///
/// * `uri` - The URI of the resource that was updated
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::notifications::ResourceUpdatedParams;
///
/// let params = ResourceUpdatedParams {
///     uri: "file:///path/to/resource".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdatedParams {
    /// The URI of the resource that was updated
    pub uri: String,
}

type Result<T> = std::result::Result<T, ServerError>;

/// A notification sender for sending notifications to clients
///
/// This trait defines the interface for components that can send
/// notifications to clients.
///
/// # Examples
///
/// ```rust
/// use async_trait::async_trait;
/// use mcp_daemon::server::notifications::{
///     NotificationSender, Notification, LoggingMessageParams, LoggingLevel
/// };
/// use mcp_daemon::server::error::ServerError;
///
/// struct MyNotificationSender;
///
/// #[async_trait]
/// impl NotificationSender for MyNotificationSender {
///     async fn send(&self, notification: Notification) -> Result<(), ServerError> {
///         // Implementation to send the notification
///         println!("Sending notification: {:?}", notification);
///         Ok(())
///     }
/// }
///
/// // Usage
/// async fn example(sender: &dyn NotificationSender) -> Result<(), ServerError> {
///     let notification = Notification::LoggingMessage(LoggingMessageParams {
///         level: LoggingLevel::Info,
///         message: "Operation completed".to_string(),
///     });
///
///     sender.send(notification).await
/// }
/// ```
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// Send a notification
    ///
    /// # Arguments
    ///
    /// * `notification` - The notification to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn send(&self, notification: Notification) -> Result<()>;
}

/// A notification handler for receiving notifications
///
/// This trait defines the interface for components that can receive
/// and handle notifications.
///
/// # Examples
///
/// ```rust
/// use async_trait::async_trait;
/// use mcp_daemon::server::notifications::{NotificationHandler, Notification};
/// use mcp_daemon::server::error::ServerError;
///
/// struct MyNotificationHandler;
///
/// #[async_trait]
/// impl NotificationHandler for MyNotificationHandler {
///     async fn handle(&self, notification: Notification) -> Result<(), ServerError> {
///         // Implementation to handle the notification
///         match notification {
///             Notification::Initialized => {
///                 println!("Server initialized");
///             }
///             Notification::ResourceListChanged => {
///                 println!("Resource list changed");
///             }
///             _ => {
///                 println!("Received other notification: {:?}", notification);
///             }
///         }
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait NotificationHandler: Send + Sync {
    /// Handle a notification
    ///
    /// # Arguments
    ///
    /// * `notification` - The notification to handle
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
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
