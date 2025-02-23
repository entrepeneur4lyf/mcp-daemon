mod mcp;
/// Module for handling code completion functionality
pub mod completion;
/// Module for error types and error handling
pub mod error;
/// Module for handling server notifications
pub mod notifications;
/// Module for handling prompts and prompt-related functionality
pub mod prompt;
pub use prompt::RegisteredPrompt;
/// Module for handling server requests
pub mod requests;
/// Module for managing resources and resource-related functionality
pub mod resource;
/// Module for managing root directories and workspace roots
pub mod roots;
/// Module for handling sampling functionality
pub mod sampling;
/// Module for managing tools and tool-related functionality
pub mod tool;

pub use mcp::McpServer;
pub use error::{ErrorCode, JsonRpcError, ServerError};
pub use notifications::{Notification, NotificationHandler, NotificationSender};
pub use requests::{Request, RequestHandler};

use crate::types::{Implementation, ServerCapabilities};
use crate::transport::Transport;
use std::sync::Arc;

type Result<T> = std::result::Result<T, ServerError>;

/// A server that implements the Model Context Protocol
pub struct Server {
    #[allow(dead_code)]
    server_info: Implementation,
    capabilities: ServerCapabilities,
    request_handler: Option<Arc<dyn RequestHandler>>,
    notification_handler: Option<Arc<dyn NotificationHandler>>,
    notification_sender: Option<Arc<dyn NotificationSender>>,
}

impl Server {
    /// Create a new server with the given implementation info
    pub fn new(server_info: Implementation) -> Self {
        Self {
            server_info,
            capabilities: Default::default(),
            request_handler: None,
            notification_handler: None,
            notification_sender: None,
        }
    }

    /// Connect to the given transport and start listening for messages
    pub async fn connect(&self, _transport: impl Transport) -> Result<()> {
        if self.request_handler.is_none() {
            return Err(ServerError::new(
                ErrorCode::HandlerNotSet,
                "Request handler not set",
            ));
        }

        if self.notification_handler.is_none() {
            return Err(ServerError::new(
                ErrorCode::HandlerNotSet,
                "Notification handler not set",
            ));
        }

        // TODO: Implement actual server connection logic
        Ok(())
    }

    /// Listen for incoming messages and handle them
    pub async fn listen(&self) -> Result<()> {
        // TODO: Implement actual server listening logic
        Ok(())
    }

    /// Register new capabilities
    pub fn register_capabilities(&mut self, capabilities: ServerCapabilities) {
        self.capabilities = capabilities;
    }

    /// Set the request handler
    pub fn set_request_handler(&mut self, handler: impl RequestHandler + 'static) {
        self.request_handler = Some(Arc::new(handler));
    }

    /// Set the notification handler
    pub fn set_notification_handler(&mut self, handler: impl NotificationHandler + 'static) {
        self.notification_handler = Some(Arc::new(handler));
    }

    /// Set the notification sender
    pub fn set_notification_sender(&mut self, sender: impl NotificationSender + 'static) {
        self.notification_sender = Some(Arc::new(sender));
    }

    /// Send a notification to clients
    pub async fn send_notification(&self, notification: Notification) -> Result<()> {
        if let Some(sender) = &self.notification_sender {
            sender.send(notification).await.map_err(|e| 
                ServerError::new(ErrorCode::InternalError, format!("Failed to send notification: {}", e)))?;
        } else {
            return Err(ServerError::new(
                ErrorCode::HandlerNotSet,
                "Notification sender not set",
            ));
        }
        Ok(())
    }
}
