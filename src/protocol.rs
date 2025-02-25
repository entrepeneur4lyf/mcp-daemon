//! # MCP Protocol Implementation
//!
//! This module provides the core protocol implementation for the Model Context Protocol (MCP).
//! It handles the bidirectional communication between clients and servers, managing requests,
//! responses, and notifications according to the JSON-RPC 2.0 specification.
//!
//! ## Overview
//!
//! The protocol layer sits between the transport layer and the application logic, providing:
//!
//! - Message routing and dispatching
//! - Request/response correlation
//! - Typed request and notification handlers
//! - Timeout management
//! - Error handling
//!
//! ## Key Components
//!
//! * **Protocol**: The main protocol implementation that manages communication
//! * **ProtocolBuilder**: A builder for creating and configuring Protocol instances
//! * **RequestHandler**: Trait for handling incoming requests
//! * **NotificationHandler**: Trait for handling incoming notifications
//!
//! ## Examples
//!
//! ### Creating a Protocol Instance
//!
//! ```rust,no_run
//! use mcp_daemon::protocol::{Protocol, ProtocolBuilder};
//! use mcp_daemon::transport::ServerStdioTransport;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a transport
//! let transport = ServerStdioTransport::default();
//!
//! // Create a protocol builder
//! let builder = ProtocolBuilder::new(transport)
//!     .request_handler("ping", |_: serde_json::Value| {
//!         Box::pin(async move {
//!             Ok(json!({ "result": "pong" }))
//!         })
//!     })
//!     .notification_handler("log", |params: serde_json::Value| {
//!         Box::pin(async move {
//!             println!("Log: {:?}", params);
//!             Ok(())
//!         })
//!     });
//!
//! // Build the protocol
//! let protocol = builder.build();
//! # Ok(())
//! # }
//! ```
//!
//! ### Sending Requests and Notifications
//!
//! ```rust,no_run
//! use mcp_daemon::protocol::{Protocol, RequestOptions};
//! use mcp_daemon::transport::ClientStdioTransport;
//! use serde_json::json;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let transport = ClientStdioTransport::new("example", &[])?;
//! # let protocol = Protocol::builder(transport).build();
//! // Send a request with a timeout
//! let response = protocol.request(
//!     "get_data",
//!     Some(json!({ "id": 123 })),
//!     RequestOptions::default().timeout(Duration::from_secs(30))
//! ).await?;
//!
//! // Send a notification
//! protocol.notify(
//!     "status_update",
//!     Some(json!({ "status": "ready" }))
//! ).await?;
//! # Ok(())
//! # }
//! ```

use super::transport::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, Message,
    Transport, TransportError, TransportErrorCode,
};
use super::types::ErrorCode;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::debug;

/// Result type for protocol operations
type Result<T> = std::result::Result<T, TransportError>;

/// The main protocol implementation for handling MCP messages
/// 
/// Manages bidirectional communication between client and server,
/// handling requests, notifications, and their respective responses.
///
/// The Protocol struct is responsible for:
/// - Sending requests and notifications to the remote endpoint
/// - Receiving and routing incoming messages
/// - Managing request/response correlation
/// - Handling timeouts
///
/// # Examples
///
/// ```rust,no_run
/// use mcp_daemon::protocol::Protocol;
/// use mcp_daemon::transport::ClientStdioTransport;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = ClientStdioTransport::new("example", &[])?;
/// let protocol = Protocol::builder(transport).build();
///
/// // Send a request
/// let response = protocol.request(
///     "get_data",
///     Some(json!({ "id": 123 })),
///     Default::default()
/// ).await?;
///
/// // Send a notification
/// protocol.notify(
///     "status_update",
///     Some(json!({ "status": "ready" }))
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Protocol<T: Transport> {
    /// The transport layer used for communication
    transport: Arc<T>,

    /// Counter for generating unique request IDs
    request_id: Arc<AtomicU64>,
    /// Map of pending requests awaiting responses
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
    /// Map of request handlers registered for different methods
    request_handlers: Arc<Mutex<HashMap<String, Box<dyn RequestHandler>>>>,
    /// Map of notification handlers registered for different methods
    notification_handlers: Arc<Mutex<HashMap<String, Box<dyn NotificationHandler>>>>,
}

impl<T: Transport> Protocol<T> {
    /// Creates a new protocol builder with the given transport
    /// 
    /// # Arguments
    /// * `transport` - The transport layer to use for communication
    ///
    /// # Returns
    ///
    /// A new `ProtocolBuilder` instance that can be used to configure and build a `Protocol`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::protocol::Protocol;
    /// # use mcp_daemon::transport::ClientStdioTransport;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = ClientStdioTransport::new("example", &[])?;
    /// let builder = Protocol::builder(transport);
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(transport: T) -> ProtocolBuilder<T> {
        ProtocolBuilder::new(transport)
    }

    /// Sends a notification to the remote endpoint
    /// 
    /// Notifications are one-way messages that don't expect a response.
    ///
    /// # Arguments
    /// * `method` - The notification method name
    /// * `params` - Optional parameters for the notification
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the notification was sent successfully, or a `TransportError` if it failed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::protocol::Protocol;
    /// # use mcp_daemon::transport::ClientStdioTransport;
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let transport = ClientStdioTransport::new("example", &[])?;
    /// # let protocol = Protocol::builder(transport).build();
    /// // Send a simple notification
    /// protocol.notify("initialized", None).await?;
    ///
    /// // Send a notification with parameters
    /// protocol.notify(
    ///     "status_update",
    ///     Some(json!({ "status": "ready" }))
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn notify(&self, method: &str, params: Option<serde_json::Value>) -> Result<()> {
        let notification = JsonRpcNotification {
            method: method.to_string(),
            params,
            ..Default::default()
        };
        let msg = JsonRpcMessage::Notification(notification);
        self.transport.send(&msg).await?;
        Ok(())
    }

    /// Send a JSON-RPC request to the server
    ///
    /// This method sends a request to the remote endpoint and waits for a response.
    /// It automatically generates a unique request ID and handles timeout management.
    ///
    /// # Arguments
    /// * `method` - The name of the method to call
    /// * `params` - Optional parameters for the method
    /// * `options` - Request options like timeout
    ///
    /// # Returns
    ///
    /// A `Result` containing the JSON-RPC response if successful, or a `TransportError` if it failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The transport fails to send the request
    /// * The request times out
    /// * The request is cancelled
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::protocol::{Protocol, RequestOptions};
    /// # use mcp_daemon::transport::ClientStdioTransport;
    /// # use serde_json::json;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let transport = ClientStdioTransport::new("example", &[])?;
    /// # let protocol = Protocol::builder(transport).build();
    /// // Send a request with default options
    /// let response = protocol.request(
    ///     "get_data",
    ///     Some(json!({ "id": 123 })),
    ///     Default::default()
    /// ).await?;
    ///
    /// // Send a request with a custom timeout
    /// let response = protocol.request(
    ///     "get_data",
    ///     Some(json!({ "id": 123 })),
    ///     RequestOptions::default().timeout(Duration::from_secs(30))
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
        options: RequestOptions,
    ) -> Result<JsonRpcResponse> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        // Create a oneshot channel for this request
        let (tx, rx) = oneshot::channel();

        // Store the sender
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id, tx);
        }

        // Send the request
        let msg = JsonRpcMessage::Request(JsonRpcRequest {
            id,
            method: method.to_string(),
            params,
            ..Default::default()
        });
        self.transport.send(&msg).await?;

        // Wait for response with timeout
        match timeout(options.timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                // Clean up the pending request if receiver was dropped
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id);
                Err(TransportError::new(
                    TransportErrorCode::MessageReceiveFailed,
                    "Request cancelled",
                ))
            }
            Err(_) => Err(TransportError::new(
                TransportErrorCode::Timeout,
                "Request timed out",
            )),
        }
    }

    /// Listen for and handle incoming JSON-RPC messages
    ///
    /// This method runs indefinitely, processing incoming messages and routing them
    /// to the appropriate handlers. It will exit when the transport is closed.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the transport is closed normally, or a `TransportError` if an error occurs.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The transport encounters an error while receiving messages
    /// * A handler encounters an error while processing a message
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::protocol::Protocol;
    /// # use mcp_daemon::transport::ClientStdioTransport;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let transport = ClientStdioTransport::new("example", &[])?;
    /// # let protocol = Protocol::builder(transport).build();
    /// // Start listening for messages
    /// protocol.listen().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn listen(&self) -> Result<()> {
        debug!("Listening for requests");
        loop {
            let message: Option<Message> = self.transport.receive().await?;

            // Exit loop when transport signals shutdown with None
            if message.is_none() {
                break;
            }

            match message.unwrap() {
                JsonRpcMessage::Request(request) => self.handle_request(request).await?,
                JsonRpcMessage::Response(response) => {
                    let id = response.id;
                    let mut pending = self.pending_requests.lock().await;
                    if let Some(tx) = pending.remove(&id) {
                        let _ = tx.send(response);
                    }
                }
                JsonRpcMessage::Notification(notification) => {
                    let handlers = self.notification_handlers.lock().await;
                    if let Some(handler) = handlers.get(&notification.method) {
                        handler.handle(notification).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle an incoming request
    ///
    /// This method routes the request to the appropriate handler and sends the response.
    ///
    /// # Arguments
    ///
    /// * `request` - The JSON-RPC request to handle
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the request was handled successfully, or a `TransportError` if it failed.
    async fn handle_request(&self, request: JsonRpcRequest) -> Result<()> {
        let handlers = self.request_handlers.lock().await;
        if let Some(handler) = handlers.get(&request.method) {
            match handler.handle(request.clone()).await {
                Ok(response) => {
                    let msg = JsonRpcMessage::Response(response);
                    self.transport.send(&msg).await?;
                }
                Err(e) => {
                    let error_response = JsonRpcResponse {
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: ErrorCode::InternalError as i32,
                            message: e.to_string(),
                            data: None,
                        }),
                        ..Default::default()
                    };
                    let msg = JsonRpcMessage::Response(error_response);
                    self.transport.send(&msg).await?;
                }
            }
        } else {
            self.transport
                .send(&JsonRpcMessage::Response(JsonRpcResponse {
                    id: request.id,
                    error: Some(JsonRpcError {
                        code: ErrorCode::MethodNotFound as i32,
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    }),
                    ..Default::default()
                }))
                .await?;
        }
        Ok(())
    }
}

/// The default request timeout, in milliseconds
///
/// This constant defines the default timeout for requests, which is 60 seconds (60,000 milliseconds).
pub const DEFAULT_REQUEST_TIMEOUT_MSEC: u64 = 60000;

/// Options for configuring request behavior
///
/// This struct provides options for configuring how requests are sent and processed.
/// Currently, it only supports setting a timeout, but could be extended with other options in the future.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::protocol::RequestOptions;
/// use std::time::Duration;
///
/// // Create options with default timeout (60 seconds)
/// let options = RequestOptions::default();
///
/// // Create options with a custom timeout
/// let options = RequestOptions::default().timeout(Duration::from_secs(30));
/// ```
pub struct RequestOptions {
    /// The timeout duration for the request
    timeout: Duration,
}

impl RequestOptions {
    /// Set the timeout duration for requests
    ///
    /// # Arguments
    /// * `timeout` - The duration after which requests should time out
    ///
    /// # Returns
    ///
    /// A new `RequestOptions` instance with the specified timeout.
    pub fn timeout(self, timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(DEFAULT_REQUEST_TIMEOUT_MSEC),
        }
    }
}

/// Builder for creating and configuring a Protocol instance
///
/// This builder provides a fluent interface for configuring a Protocol instance,
/// allowing you to register request and notification handlers before building the protocol.
///
/// # Examples
///
/// ```rust,no_run
/// use mcp_daemon::protocol::ProtocolBuilder;
/// use mcp_daemon::transport::{ServerStdioTransport, Transport, TransportError};
/// use serde_json::json;
/// use std::pin::Pin;
/// use std::future::Future;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = ServerStdioTransport::default();
///
/// // Create a protocol builder with request and notification handlers
/// let builder = ProtocolBuilder::new(transport)
///     .request_handler("ping", |_: serde_json::Value| {
///         Box::pin(async move {
///             Ok(json!({ "result": "pong" }))
///         })
///     })
///     .notification_handler("log", |params: serde_json::Value| {
///         Box::pin(async move {
///             println!("Log: {:?}", params);
///             Ok(())
///         })
///     });
///
/// // Build the protocol
/// let protocol = builder.build();
/// # Ok(())
/// # }
/// ```
pub struct ProtocolBuilder<T: Transport> {
    /// The transport layer to use for communication
    _transport: T,
    /// Map of request handlers registered for different methods
    request_handlers: HashMap<String, Box<dyn RequestHandler>>,
    /// Map of notification handlers registered for different methods
    notification_handlers: HashMap<String, Box<dyn NotificationHandler>>,
}

impl<T: Transport> ProtocolBuilder<T> {
    /// Creates a new ProtocolBuilder instance
    /// 
    /// # Arguments
    /// * `transport` - The transport layer to use for communication
    ///
    /// # Returns
    ///
    /// A new `ProtocolBuilder` instance.
    pub fn new(transport: T) -> Self {
        Self {
            _transport: transport,
            request_handlers: HashMap::new(),
            notification_handlers: HashMap::new(),
        }
    }

    /// Register a typed request handler
    ///
    /// This method registers a handler function for a specific request method.
    /// The handler function will be called when a request with the specified method is received.
    ///
    /// # Arguments
    ///
    /// * `method` - The method name to handle
    /// * `handler` - The function to handle requests of this type
    ///
    /// # Type Parameters
    ///
    /// * `Req` - The type of request payload
    /// * `Resp` - The type of response payload
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::protocol::ProtocolBuilder;
/// # use mcp_daemon::transport::{ServerStdioTransport, Transport, TransportError};
/// # use serde_json::json;
/// # use std::pin::Pin;
/// # use std::future::Future;
/// # fn example() {
/// # let transport = ServerStdioTransport::default();
/// let builder = ProtocolBuilder::new(transport)
///     .request_handler("ping", |_: serde_json::Value| {
///         Box::pin(async move {
///             Ok(json!({ "result": "pong" }))
///         })
///     });
/// # }
/// ```
    pub fn request_handler<Req, Resp>(
        mut self,
        method: &str,
        handler: impl Fn(Req) -> RequestFuture<Resp> + Send + Sync + 'static,
    ) -> Self
    where
        Req: DeserializeOwned + Send + Sync + 'static,
        Resp: Serialize + Send + Sync + 'static,
    {
        let handler = TypedRequestHandler {
            handler: Box::new(handler),
            _phantom: std::marker::PhantomData,
        };

        self.request_handlers
            .insert(method.to_string(), Box::new(handler));
        self
    }

    /// Check if a handler exists for the given method
    ///
    /// # Arguments
    /// * `method` - The method name to check
    ///
    /// # Returns
    /// `true` if a handler exists for the method, `false` otherwise
    pub fn has_request_handler(&self, method: &str) -> bool {
        self.request_handlers.contains_key(method)
    }

    /// Register a handler for a notification method
    ///
    /// This method registers a handler function for a specific notification method.
    /// The handler function will be called when a notification with the specified method is received.
    ///
    /// # Arguments
    /// * `method` - The notification method to handle
    /// * `handler` - The function to handle notifications of this type
    ///
    /// # Type Parameters
    /// * `N` - The type of notification payload
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::protocol::ProtocolBuilder;
/// # use mcp_daemon::transport::{ServerStdioTransport, Transport, TransportError};
/// # use serde_json::json;
/// # use std::pin::Pin;
/// # use std::future::Future;
/// # fn example() {
/// # let transport = ServerStdioTransport::default();
/// let builder = ProtocolBuilder::new(transport)
///     .notification_handler("log", |params: serde_json::Value| {
///         Box::pin(async move {
///             println!("Log: {:?}", params);
///             Ok(())
///         })
///     });
/// # }
/// ```
    pub fn notification_handler<N>(
        mut self,
        method: &str,
        handler: impl Fn(N) -> NotificationFuture + Send + Sync + 'static,
    ) -> Self
    where
        N: DeserializeOwned + Send + Sync + 'static,
    {
        self.notification_handlers.insert(
            method.to_string(),
            Box::new(TypedNotificationHandler {
                handler: Box::new(handler),
                _phantom: std::marker::PhantomData,
            }),
        );
        self
    }

    /// Build the protocol instance with the configured handlers
    ///
    /// # Returns
    /// A new Protocol instance ready to handle requests and notifications
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::protocol::ProtocolBuilder;
/// # use mcp_daemon::transport::{ServerStdioTransport, Transport};
/// # fn example() {
/// # let transport = ServerStdioTransport::default();
/// # let builder = ProtocolBuilder::new(transport);
/// let protocol = builder.build();
/// # }
/// ```
    pub fn build(self) -> Protocol<T> {
        Protocol {
            transport: Arc::new(self._transport),
            request_handlers: Arc::new(Mutex::new(self.request_handlers)),
            notification_handlers: Arc::new(Mutex::new(self.notification_handlers)),
            request_id: Arc::new(AtomicU64::new(0)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// Update the handler traits to be async
/// Trait for handling incoming requests
///
/// This trait defines the interface for request handlers.
#[async_trait]
trait RequestHandler: Send + Sync {
    /// Handle an incoming request
    ///
    /// # Arguments
    ///
    /// * `request` - The JSON-RPC request to handle
    ///
    /// # Returns
    ///
    /// A `Result` containing the JSON-RPC response if successful, or a `TransportError` if it failed.
    async fn handle(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse>;
}

/// Trait for handling incoming notifications
///
/// This trait defines the interface for notification handlers.
#[async_trait]
trait NotificationHandler: Send + Sync {
    /// Handle an incoming notification
    ///
    /// # Arguments
    ///
    /// * `notification` - The JSON-RPC notification to handle
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the notification was handled successfully, or a `TransportError` if it failed.
    async fn handle(&self, notification: JsonRpcNotification) -> Result<()>;
}

// Type aliases for complex future types
/// Type alias for a future that returns a Result<T>
type RequestFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;
/// Type alias for a future that returns a Result<()>
type NotificationFuture = RequestFuture<()>;

/// Type alias for async request handler function type
type AsyncRequestHandler<Req, Resp> = Box<dyn Fn(Req) -> RequestFuture<Resp> + Send + Sync>;

/// Type alias for async notification handler function type
type AsyncNotificationHandler<N> = Box<dyn Fn(N) -> NotificationFuture + Send + Sync>;

// Update the TypedRequestHandler to use async handlers
/// A typed request handler that deserializes request parameters and serializes responses
struct TypedRequestHandler<Req, Resp>
where
    Req: DeserializeOwned + Send + Sync + 'static,
    Resp: Serialize + Send + Sync + 'static,
{
    /// The handler function
    handler: AsyncRequestHandler<Req, Resp>,
    /// Phantom data for type parameters
    _phantom: std::marker::PhantomData<(Req, Resp)>,
}

#[async_trait]
impl<Req, Resp> RequestHandler for TypedRequestHandler<Req, Resp>
where
    Req: DeserializeOwned + Send + Sync + 'static,
    Resp: Serialize + Send + Sync + 'static,
{
    async fn handle(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let params: Req = if request.params.is_none() || request.params.as_ref().unwrap().is_null() {
            serde_json::from_value(serde_json::Value::Null).map_err(|e| 
                TransportError::new(TransportErrorCode::InvalidMessage, format!("Failed to deserialize null params: {}", e)))?
        } else {
            serde_json::from_value(request.params.unwrap()).map_err(|e| 
                TransportError::new(TransportErrorCode::InvalidMessage, format!("Failed to deserialize params: {}", e)))?
        };
        let result = (self.handler)(params).await?;
        Ok(JsonRpcResponse {
            id: request.id,
            result: Some(serde_json::to_value(result).map_err(|e| 
                TransportError::new(TransportErrorCode::InvalidMessage, format!("Failed to serialize response: {}", e)))?),
            error: None,
            ..Default::default()
        })
    }
}

/// A typed notification handler that deserializes notification parameters
struct TypedNotificationHandler<N>
where
    N: DeserializeOwned + Send + Sync + 'static,
{
    /// The handler function
    handler: AsyncNotificationHandler<N>,
    /// Phantom data for type parameter
    _phantom: std::marker::PhantomData<N>,
}

#[async_trait]
impl<N> NotificationHandler for TypedNotificationHandler<N>
where
    N: DeserializeOwned + Send + Sync + 'static,
{
    async fn handle(&self, notification: JsonRpcNotification) -> Result<()> {
        let params: N =
            if notification.params.is_none() || notification.params.as_ref().unwrap().is_null() {
                serde_json::from_value(serde_json::Value::Null).map_err(|e| 
                    TransportError::new(TransportErrorCode::InvalidMessage, format!("Failed to deserialize null params: {}", e)))?
            } else {
                serde_json::from_value(notification.params.unwrap()).map_err(|e| 
                    TransportError::new(TransportErrorCode::InvalidMessage, format!("Failed to deserialize params: {}", e)))?
            };
        (self.handler)(params).await
    }
}
