//! # MCP Server Implementation
//!
//! This module provides the server-side implementation of the Model Context Protocol (MCP).
//! It includes the core server functionality, request and notification handling, and various
//! submodules for specific server features.
//!
//! ## Overview
//!
//! The MCP server is responsible for:
//! - Handling client connections and protocol initialization
//! - Processing requests and sending responses
//! - Managing notifications
//! - Exposing tools, resources, and other capabilities to clients
//! - Implementing server-side protocol operations
//!
//! ## Key Components
//!
//! * **Server**: The main server implementation that manages client connections
//! * **McpServer**: A concrete implementation of the MCP server
//! * **Request/Response Handling**: Processing client requests and generating responses
//! * **Notification Handling**: Managing and sending notifications to clients
//! * **Error Handling**: Standardized error types and handling mechanisms
//!
//! ## Submodules
//!
//! * **completion**: Handles code completion functionality
//! * **error**: Defines error types and error handling mechanisms
//! * **notifications**: Manages server notifications
//! * **prompt**: Handles prompts and prompt-related functionality
//! * **requests**: Processes client requests
//! * **resource**: Manages resources and resource-related functionality
//! * **roots**: Manages root directories and workspace roots
//! * **sampling**: Handles sampling functionality
//! * **tool**: Manages tools and tool-related functionality
//!
//! ## Examples
//!
//! ```rust,no_run
//! use mcp_daemon::server::{Server, McpServer};
//! use mcp_daemon::types::Implementation;
//! use mcp_daemon::transport::ServerStdioTransport;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a server implementation
//! let server_info = Implementation {
//!     name: "example-server".to_string(),
//!     version: "0.1.0".to_string(),
//! };
//!
//! // Create a new server
//! let mut server = Server::new(server_info);
//!
//! // Create a transport
//! let transport = ServerStdioTransport::default();
//!
//! // Connect the server to the transport
//! server.connect(transport).await?;
//!
//! // Start listening for messages
//! server.listen().await?;
//! # Ok(())
//! # }
//! ```

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
pub use resource::RegisteredResource;
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

/// Result type for server operations
type Result<T> = std::result::Result<T, ServerError>;

/// A server that implements the Model Context Protocol
///
/// The `Server` struct is the main entry point for creating an MCP server.
/// It manages client connections, request and notification handling, and
/// provides the core server functionality.
///
/// # Examples
///
/// ```rust,no_run
/// use mcp_daemon::server::Server;
/// use mcp_daemon::types::Implementation;
///
/// # fn example() {
/// // Create a server implementation
/// let server_info = Implementation {
///     name: "example-server".to_string(),
///     version: "0.1.0".to_string(),
/// };
///
/// // Create a new server
/// let mut server = Server::new(server_info);
/// # }
/// ```
pub struct Server {
    /// Information about the server implementation
    #[allow(dead_code)]
    server_info: Implementation,
    
    /// Capabilities supported by the server
    capabilities: ServerCapabilities,
    
    /// Handler for processing client requests
    request_handler: Option<Arc<dyn RequestHandler>>,
    
    /// Handler for processing client notifications
    notification_handler: Option<Arc<dyn NotificationHandler>>,
    
    /// Sender for sending notifications to clients
    notification_sender: Option<Arc<dyn NotificationSender>>,
}

impl Server {
    /// Create a new server with the given implementation info
    ///
    /// # Arguments
    ///
    /// * `server_info` - Information about the server implementation
    ///
    /// # Returns
    ///
    /// A new `Server` instance with default capabilities and no handlers set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::Server;
    /// use mcp_daemon::types::Implementation;
    ///
    /// let server_info = Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "0.1.0".to_string(),
    /// };
    ///
    /// let server = Server::new(server_info);
    /// ```
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
    ///
    /// This method establishes a connection with the provided transport
    /// and prepares the server to handle client messages.
    ///
    /// # Arguments
    ///
    /// * `_transport` - The transport to use for communication
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the connection was established successfully,
    /// or a `ServerError` if it failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The request handler is not set
    /// * The notification handler is not set
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::server::{Server, RequestHandler, NotificationHandler, NotificationSender, Request, Notification, ServerError};
/// # use mcp_daemon::types::Implementation;
/// # use mcp_daemon::transport::{ServerStdioTransport, Transport};
/// # use async_trait::async_trait;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let server_info = Implementation {
/// #     name: "example-server".to_string(),
/// #     version: "0.1.0".to_string(),
/// # };
/// # let mut server = Server::new(server_info);
/// # struct DummyRequestHandler;
/// # #[async_trait]
/// # impl RequestHandler for DummyRequestHandler {
/// #     async fn handle(&self, _request: Request) -> std::result::Result<serde_json::Value, ServerError> { Ok(serde_json::Value::Null) }
/// # }
/// # struct DummyNotificationHandler;
/// # #[async_trait]
/// # impl NotificationHandler for DummyNotificationHandler {
/// #     async fn handle(&self, _notification: Notification) -> std::result::Result<(), ServerError> { Ok(()) }
/// # }
/// # struct DummyNotificationSender;
/// # #[async_trait]
/// # impl NotificationSender for DummyNotificationSender {
/// #     async fn send(&self, _notification: Notification) -> std::result::Result<(), ServerError> { Ok(()) }
/// # }
/// # let request_handler = DummyRequestHandler {};
/// # let notification_handler = DummyNotificationHandler {};
/// # let notification_sender = DummyNotificationSender {};
    /// # server.set_request_handler(request_handler);
    /// # server.set_notification_handler(notification_handler);
    /// # server.set_notification_sender(notification_sender);
    /// let transport = ServerStdioTransport::default();
    /// server.connect(transport).await?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// This method starts listening for incoming messages from clients
    /// and routes them to the appropriate handlers.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the listening was successful,
    /// or a `ServerError` if it failed.
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::server::{Server, RequestHandler, NotificationHandler, NotificationSender, Request, Notification, ServerError};
/// # use mcp_daemon::types::Implementation;
/// # use mcp_daemon::transport::{ServerStdioTransport, Transport};
/// # use async_trait::async_trait;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let server_info = Implementation {
/// #     name: "example-server".to_string(),
/// #     version: "0.1.0".to_string(),
/// # };
/// # let mut server = Server::new(server_info);
/// # struct DummyRequestHandler;
/// # #[async_trait]
/// # impl RequestHandler for DummyRequestHandler {
/// #     async fn handle(&self, _request: Request) -> std::result::Result<serde_json::Value, ServerError> { Ok(serde_json::Value::Null) }
/// # }
/// # struct DummyNotificationHandler;
/// # #[async_trait]
/// # impl NotificationHandler for DummyNotificationHandler {
/// #     async fn handle(&self, _notification: Notification) -> std::result::Result<(), ServerError> { Ok(()) }
/// # }
/// # struct DummyNotificationSender;
/// # #[async_trait]
/// # impl NotificationSender for DummyNotificationSender {
/// #     async fn send(&self, _notification: Notification) -> std::result::Result<(), ServerError> { Ok(()) }
/// # }
/// # let request_handler = DummyRequestHandler {};
/// # let notification_handler = DummyNotificationHandler {};
/// # let notification_sender = DummyNotificationSender {};
    /// # server.set_request_handler(request_handler);
    /// # server.set_notification_handler(notification_handler);
    /// # server.set_notification_sender(notification_sender);
    /// # let transport = ServerStdioTransport::default();
    /// # server.connect(transport).await?;
    /// // Start listening for messages
    /// server.listen().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn listen(&self) -> Result<()> {
        // TODO: Implement actual server listening logic
        Ok(())
    }

    /// Register new capabilities
    ///
    /// This method sets the capabilities supported by the server.
    ///
    /// # Arguments
    ///
    /// * `capabilities` - The capabilities to register
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mcp_daemon::server::Server;
    /// # use mcp_daemon::types::{Implementation, ServerCapabilities, ToolCapabilities};
    /// # fn example() {
    /// # let server_info = Implementation {
    /// #     name: "example-server".to_string(),
    /// #     version: "0.1.0".to_string(),
    /// # };
    /// # let mut server = Server::new(server_info);
    /// // Create capabilities with tool support
    /// let capabilities = ServerCapabilities {
    ///     tools: Some(ToolCapabilities {
    ///         list_changed: Some(true),
    ///     }),
    ///     ..Default::default()
    /// };
    ///
    /// // Register the capabilities
    /// server.register_capabilities(capabilities);
    /// # }
    /// ```
    pub fn register_capabilities(&mut self, capabilities: ServerCapabilities) {
        self.capabilities = capabilities;
    }

    /// Set the request handler
    ///
    /// This method sets the handler for processing client requests.
    ///
    /// # Arguments
    ///
    /// * `handler` - The request handler to set
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::server::{Server, RequestHandler, Request, ServerError};
/// # use mcp_daemon::types::Implementation;
/// # use async_trait::async_trait;
/// # fn example() {
/// # let server_info = Implementation {
/// #     name: "example-server".to_string(),
/// #     version: "0.1.0".to_string(),
/// # };
/// # let mut server = Server::new(server_info);
/// # struct DummyRequestHandler;
/// # #[async_trait]
/// # impl RequestHandler for DummyRequestHandler {
/// #     async fn handle(&self, _request: Request) -> std::result::Result<serde_json::Value, ServerError> { Ok(serde_json::Value::Null) }
/// # }
/// # let request_handler = DummyRequestHandler {};
/// server.set_request_handler(request_handler);
/// # }
/// ```
    pub fn set_request_handler(&mut self, handler: impl RequestHandler + 'static) {
        self.request_handler = Some(Arc::new(handler));
    }

    /// Set the notification handler
    ///
    /// This method sets the handler for processing client notifications.
    ///
    /// # Arguments
    ///
    /// * `handler` - The notification handler to set
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::server::{Server, NotificationHandler, Notification, ServerError};
/// # use mcp_daemon::types::Implementation;
/// # use async_trait::async_trait;
/// # fn example() {
/// # let server_info = Implementation {
/// #     name: "example-server".to_string(),
/// #     version: "0.1.0".to_string(),
/// # };
/// # let mut server = Server::new(server_info);
/// # struct DummyNotificationHandler;
/// # #[async_trait]
/// # impl NotificationHandler for DummyNotificationHandler {
/// #     async fn handle(&self, _notification: Notification) -> std::result::Result<(), ServerError> { Ok(()) }
/// # }
/// # let notification_handler = DummyNotificationHandler {};
/// server.set_notification_handler(notification_handler);
/// # }
/// ```
    pub fn set_notification_handler(&mut self, handler: impl NotificationHandler + 'static) {
        self.notification_handler = Some(Arc::new(handler));
    }

    /// Set the notification sender
    ///
    /// This method sets the sender for sending notifications to clients.
    ///
    /// # Arguments
    ///
    /// * `sender` - The notification sender to set
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::server::{Server, NotificationSender, Notification, ServerError};
/// # use mcp_daemon::types::Implementation;
/// # use async_trait::async_trait;
/// # fn example() {
/// # let server_info = Implementation {
/// #     name: "example-server".to_string(),
/// #     version: "0.1.0".to_string(),
/// # };
/// # let mut server = Server::new(server_info);
/// # struct DummyNotificationSender;
/// # #[async_trait]
/// # impl NotificationSender for DummyNotificationSender {
/// #     async fn send(&self, _notification: Notification) -> std::result::Result<(), ServerError> { Ok(()) }
/// # }
/// # let notification_sender = DummyNotificationSender {};
/// server.set_notification_sender(notification_sender);
    /// # }
    /// ```
    pub fn set_notification_sender(&mut self, sender: impl NotificationSender + 'static) {
        self.notification_sender = Some(Arc::new(sender));
    }

    /// Send a notification to clients
    ///
    /// This method sends a notification to all connected clients.
    ///
    /// # Arguments
    ///
    /// * `notification` - The notification to send
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the notification was sent successfully,
    /// or a `ServerError` if it failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The notification sender is not set
    /// * The notification sender fails to send the notification
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::server::{Server, NotificationSender, Notification, ServerError};
/// # use mcp_daemon::types::Implementation;
/// # use async_trait::async_trait;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let server_info = Implementation {
/// #     name: "example-server".to_string(),
/// #     version: "0.1.0".to_string(),
/// # };
/// # let mut server = Server::new(server_info);
/// # struct DummyNotificationSender;
/// # #[async_trait]
/// # impl NotificationSender for DummyNotificationSender {
/// #     async fn send(&self, _notification: Notification) -> std::result::Result<(), ServerError> { Ok(()) }
/// # }
/// # let notification_sender = DummyNotificationSender {};
/// # server.set_notification_sender(notification_sender);
/// # let notification = Notification::Initialized;
    /// // Send a notification to clients
    /// server.send_notification(notification).await?;
    /// # Ok(())
    /// # }
    /// ```
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
