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

type Result<T> = std::result::Result<T, TransportError>;

/// The main protocol implementation for handling MCP messages
/// 
/// Manages bidirectional communication between client and server,
/// handling requests, notifications, and their respective responses.
#[derive(Clone)]
pub struct Protocol<T: Transport> {
    transport: Arc<T>,

    request_id: Arc<AtomicU64>,
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
    request_handlers: Arc<Mutex<HashMap<String, Box<dyn RequestHandler>>>>,
    notification_handlers: Arc<Mutex<HashMap<String, Box<dyn NotificationHandler>>>>,
}

impl<T: Transport> Protocol<T> {
    /// Creates a new protocol builder with the given transport
/// 
/// # Arguments
/// * `transport` - The transport layer to use for communication
pub fn builder(transport: T) -> ProtocolBuilder<T> {
        ProtocolBuilder::new(transport)
    }

    /// Sends a notification to the remote endpoint
/// 
/// # Arguments
/// * `method` - The notification method name
/// * `params` - Optional parameters for the notification
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
    /// # Arguments
    /// * `method` - The name of the method to call
    /// * `params` - Optional parameters for the method
    /// * `options` - Request options like timeout
    ///
    /// # Returns
    /// The JSON-RPC response from the server
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
    /// to the appropriate handlers.
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
pub const DEFAULT_REQUEST_TIMEOUT_MSEC: u64 = 60000;
/// Options for configuring request behavior
pub struct RequestOptions {
    timeout: Duration,
}

impl RequestOptions {
    /// Set the timeout duration for requests
    ///
    /// # Arguments
    /// * `timeout` - The duration after which requests should time out
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
pub struct ProtocolBuilder<T: Transport> {
    _transport: T,
    request_handlers: HashMap<String, Box<dyn RequestHandler>>,
    notification_handlers: HashMap<String, Box<dyn NotificationHandler>>,
}
impl<T: Transport> ProtocolBuilder<T> {
    /// Creates a new ProtocolBuilder instance
/// 
/// # Arguments
/// * `transport` - The transport layer to use for communication
pub fn new(transport: T) -> Self {
        Self {
            _transport: transport,
            request_handlers: HashMap::new(),
            notification_handlers: HashMap::new(),
        }
    }
    /// Register a typed request handler
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
    /// # Arguments
    /// * `method` - The notification method to handle
    /// * `handler` - The function to handle notifications of this type
    ///
    /// # Type Parameters
    /// * `N` - The type of notification payload
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
#[async_trait]
trait RequestHandler: Send + Sync {
    async fn handle(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse>;
}

#[async_trait]
trait NotificationHandler: Send + Sync {
    async fn handle(&self, notification: JsonRpcNotification) -> Result<()>;
}

// Type aliases for complex future types
type RequestFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;
type NotificationFuture = RequestFuture<()>;

/// Type alias for async request handler function type
type AsyncRequestHandler<Req, Resp> = Box<dyn Fn(Req) -> RequestFuture<Resp> + Send + Sync>;

/// Type alias for async notification handler function type
type AsyncNotificationHandler<N> = Box<dyn Fn(N) -> NotificationFuture + Send + Sync>;

// Update the TypedRequestHandler to use async handlers
struct TypedRequestHandler<Req, Resp>
where
    Req: DeserializeOwned + Send + Sync + 'static,
    Resp: Serialize + Send + Sync + 'static,
{
    handler: AsyncRequestHandler<Req, Resp>,
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

struct TypedNotificationHandler<N>
where
    N: DeserializeOwned + Send + Sync + 'static,
{
    handler: AsyncNotificationHandler<N>,
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
