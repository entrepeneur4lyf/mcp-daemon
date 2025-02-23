use super::{Message, Transport};
use super::Result;
use super::error::{TransportError, TransportErrorCode};
use actix_ws::{Message as WsMessage, Session};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use reqwest::header::{HeaderName, HeaderValue};
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message as TungsteniteMessage};
use tracing::{debug, info};

// Type aliases to simplify complex types
type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures::stream::SplitSink<WsStream, TungsteniteMessage>;
type MessageSender = broadcast::Sender<Message>;
type MessageReceiver = broadcast::Receiver<Message>;

#[derive(Clone)]
/// WebSocket transport implementation for the server side
pub struct ServerWsTransport {
    session: Arc<Mutex<Option<Session>>>,
    rx: Arc<Mutex<Option<broadcast::Receiver<Message>>>>,
    tx: Arc<Mutex<Option<broadcast::Sender<Message>>>>,
}

impl std::fmt::Debug for ServerWsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerWsTransport")
            .field("session", &"<Session>")
            .field("rx", &self.rx)
            .field("tx", &self.tx)
            .finish()
    }
}

impl ServerWsTransport {
    /// Create a new server-side WebSocket transport
    ///
    /// # Arguments
    /// * `session` - The WebSocket session
    /// * `rx` - Channel receiver for incoming messages
    pub fn new(session: Session, rx: broadcast::Receiver<Message>) -> Self {
        Self {
            session: Arc::new(Mutex::new(Some(session))),
            rx: Arc::new(Mutex::new(Some(rx))),
            tx: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Clone)]
/// WebSocket transport implementation for the client side
pub struct ClientWsTransport {
    ws_tx: Arc<Mutex<Option<MessageSender>>>,
    ws_rx: Arc<Mutex<Option<MessageReceiver>>>,
    url: String,
    headers: HashMap<String, String>,
    ws_write: Arc<Mutex<Option<WsSink>>>,
}

impl std::fmt::Debug for ClientWsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientWsTransport")
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("ws_tx", &"<MessageSender>")
            .field("ws_rx", &"<MessageReceiver>")
            .field("ws_write", &"<WsSink>")
            .finish()
    }
}

impl ClientWsTransport {
    /// Create a builder for configuring and creating a client WebSocket transport
    ///
    /// # Arguments
    /// * `url` - The WebSocket server URL to connect to
    pub fn builder(url: String) -> ClientWsTransportBuilder {
        ClientWsTransportBuilder::new(url)
    }
}

/// Builder for configuring and creating a client WebSocket transport
pub struct ClientWsTransportBuilder {
    url: String,
    headers: HashMap<String, String>,
}

impl ClientWsTransportBuilder {
    /// Create a new transport builder
    ///
    /// # Arguments
    /// * `url` - The WebSocket server URL to connect to
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: HashMap::new(),
        }
    }

    /// Add a custom header to the WebSocket connection
    ///
    /// # Arguments
    /// * `key` - Header name
    /// * `value` - Header value
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Build the client WebSocket transport with the configured options
    pub fn build(self) -> ClientWsTransport {
        ClientWsTransport {
            ws_tx: Arc::new(Mutex::new(None)),
            ws_rx: Arc::new(Mutex::new(None)),
            url: self.url,
            headers: self.headers,
            ws_write: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Transport for ServerWsTransport {
    async fn receive(&self) -> Result<Option<Message>> {
        let mut rx = self.rx.lock().await;
        if let Some(rx) = rx.as_mut() {
            match rx.recv().await {
                Ok(message) => Ok(Some(message)),
                Err(broadcast::error::RecvError::Closed) => Ok(None),
                Err(e) => Err(TransportError::new(
                    TransportErrorCode::ReceiveError,
                    format!("Error receiving message: {}", e),
                )),
            }
        } else {
            Ok(None)
        }
    }

    async fn send(&self, message: &Message) -> Result<()> {
        let mut session = self.session.lock().await;
        if let Some(session) = session.as_mut() {
            let json = serde_json::to_string(message)?;
            session
                .text(json)
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::SendError, e.to_string()))?;
            Ok(())
        } else {
            Err(TransportError::new(
                TransportErrorCode::SendError,
                "No active session",
            ))
        }
    }

    async fn open(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        let mut session = self.session.lock().await;
        if let Some(session) = session.take() {
            session
                .close(None)
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::CloseError, e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl Transport for ClientWsTransport {
    async fn receive(&self) -> Result<Option<Message>> {
        let mut rx = self.ws_rx.lock().await;
        if let Some(rx) = rx.as_mut() {
            match rx.recv().await {
                Ok(message) => Ok(Some(message)),
                Err(broadcast::error::RecvError::Closed) => Ok(None),
                Err(e) => Err(TransportError::new(
                    TransportErrorCode::ReceiveError,
                    format!("Error receiving message: {}", e),
                )),
            }
        } else {
            Ok(None)
        }
    }

    async fn send(&self, message: &Message) -> Result<()> {
        let mut ws_write = self.ws_write.lock().await;
        if let Some(ws_write) = ws_write.as_mut() {
            let json = serde_json::to_string(message)?;
            ws_write
                .send(TungsteniteMessage::Text(json))
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::SendError, e.to_string()))?;
            Ok(())
        } else {
            Err(TransportError::new(
                TransportErrorCode::SendError,
                "No active WebSocket connection",
            ))
        }
    }

    async fn open(&self) -> Result<()> {
        let mut request = self.url.as_str().into_client_request()?;
        for (key, value) in &self.headers {
            request.headers_mut().insert(
                HeaderName::from_str(key).map_err(|e| {
                    TransportError::new(TransportErrorCode::OpenError, format!("Invalid header key: {}", e))
                })?,
                HeaderValue::from_str(value).map_err(|e| {
                    TransportError::new(
                        TransportErrorCode::OpenError,
                        format!("Invalid header value: {}", e),
                    )
                })?,
            );
        }

        let (ws_stream, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| TransportError::new(TransportErrorCode::OpenError, e.to_string()))?;

        let (write, mut read) = ws_stream.split();
        let (tx, rx) = broadcast::channel(100);

        let ws_tx = tx.clone();
        let ws_rx = rx;

        *self.ws_tx.lock().await = Some(ws_tx);
        *self.ws_rx.lock().await = Some(ws_rx);
        *self.ws_write.lock().await = Some(write);

        // Spawn a task to handle incoming messages
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(TungsteniteMessage::Text(text)) => {
                        if let Ok(message) = serde_json::from_str::<Message>(&text) {
                            if tx.send(message).is_err() {
                                debug!("All receivers dropped, stopping message handling");
                                break;
                            }
                        }
                    }
                    Ok(TungsteniteMessage::Close(_)) => {
                        info!("WebSocket connection closed by server");
                        break;
                    }
                    Err(e) => {
                        debug!("Error reading from WebSocket: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn close(&self) -> Result<()> {
        if let Some(mut write) = self.ws_write.lock().await.take() {
            write
                .send(TungsteniteMessage::Close(None))
                .await
                .map_err(|e| TransportError::new(TransportErrorCode::CloseError, e.to_string()))?;
        }
        Ok(())
    }
}

/// Handle a WebSocket connection, managing message flow between client and server
///
/// # Arguments
/// * `session` - The WebSocket session
/// * `stream` - Stream of incoming WebSocket messages
/// * `tx` - Channel sender for outgoing messages
/// * `rx` - Channel receiver for incoming messages
pub async fn handle_ws_connection(
    mut session: Session,
    mut stream: actix_ws::MessageStream,
    tx: broadcast::Sender<Message>,
    mut rx: broadcast::Receiver<Message>,
) -> Result<()> {
    // Send messages from rx to the WebSocket
    let mut send_task = actix_web::rt::spawn(async move {
        while let Ok(message) = rx.recv().await {
            let json = serde_json::to_string(&message)?;
            session.text(json).await?;
        }
        Ok::<_, anyhow::Error>(())
    });

    // Receive messages from the WebSocket and send them to tx
    let mut recv_task = actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                WsMessage::Text(text) => {
                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                        if tx.send(message).is_err() {
                            break;
                        }
                    }
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
        Ok::<_, anyhow::Error>(())
    });

    // Wait for either task to complete
    tokio::select! {
        res = (&mut send_task) => match res {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(TransportError::new(
                TransportErrorCode::SendError,
                format!("Send task failed: {}", e)
            )),
            Err(e) => Err(TransportError::new(
                TransportErrorCode::SendError,
                format!("Send task join error: {}", e)
            )),
        }?,
        res = (&mut recv_task) => match res {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(TransportError::new(
                TransportErrorCode::ReceiveError,
                format!("Receive task failed: {}", e)
            )),
            Err(e) => Err(TransportError::new(
                TransportErrorCode::ReceiveError,
                format!("Receive task join error: {}", e)
            )),
        }?,
    }

    Ok(())
}
