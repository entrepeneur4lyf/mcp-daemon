use actix_web::middleware::Logger;
use actix_web::web::Payload;
use actix_web::web::Query;
use actix_web::{web, App, HttpResponse, HttpServer, Either, Responder};
use actix_cors::Cors;
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use anyhow::Result;

use uuid::Uuid;

use crate::server::Server;
use crate::sse::middleware::{AuthConfig, JwtAuth};
use crate::transport::ServerHttpTransport;
use crate::transport::{handle_ws_connection, Message, ServerWsTransport};
use crate::transport::ServerSseTransport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::{debug, error, info};

/// Server-side SSE transport that handles HTTP POST requests for incoming messages
/// and sends responses via SSE
#[derive(Debug, Serialize, Deserialize)]
/// Claims in a JWT token
pub struct Claims {
    /// Expiration timestamp
    /// Expiration timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
}

#[derive(Deserialize)]
/// Query parameters for SSE messages
pub struct MessageQuery {
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Clone)]
/// Configuration for the SSE HTTP server
pub struct ServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Optional CORS configuration
    pub cors: Option<CorsConfig>,
    /// Optional TLS configuration
    pub tls: Option<TlsConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            cors: None,
            tls: None,
        }
    }
}

#[derive(Clone)]
/// Configuration for CORS
pub struct CorsConfig {
    /// Allowed origin for CORS
    pub allowed_origin: String,
    /// Whether to allow credentials
    pub allow_credentials: bool,
    /// Maximum age for CORS preflight requests
    pub max_age: Option<usize>,
}

#[derive(Clone)]
/// Configuration for TLS
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Clone)]
/// State for managing SSE sessions
pub struct SessionState {
    sessions: Arc<Mutex<HashMap<String, ServerHttpTransport>>>,
    port: u16,
    build_server: Arc<
        dyn Fn(
                ServerHttpTransport,
            )
                -> futures::future::BoxFuture<'static, Result<Server>>
            + Send
            + Sync,
    >,
}

/// Run a server instance with the specified transport
pub async fn run_http_server<F, Fut>(
    config: ServerConfig,
    jwt_secret: Option<String>,
    build_server: F,
) -> Result<()>
where
    F: Fn(ServerHttpTransport) -> Fut + Send + Sync + 'static,
    Fut: futures::Future<Output = Result<Server>> + Send + 'static,
{
    let protocol = if config.tls.is_some() { "https" } else { "http" };
    info!("Starting server on {}://127.0.0.1:{}", protocol, config.port);
    info!("WebSocket endpoint: {}://127.0.0.1:{}/ws", protocol.replace("http", "ws"), config.port);
    info!("SSE endpoint: {}://127.0.0.1:{}/sse", protocol, config.port);

    let sessions = Arc::new(Mutex::new(HashMap::new()));

    // Box the future when creating the Arc
    let build_server =
        Arc::new(move |t| Box::pin(build_server(t)) as futures::future::BoxFuture<_>);

    let auth_config = jwt_secret.map(|jwt_secret| AuthConfig { jwt_secret });
    // Configure and run the server
    let mut server = HttpServer::new(move || {
        let cors = if let Some(cors_config) = &config.cors {
            Cors::default()
                .allowed_origin(&cors_config.allowed_origin)
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .max_age(cors_config.max_age.unwrap_or(3600))
        } else {
            Cors::default()
        };

        App::new()
            .wrap(Logger::default())
            .wrap(JwtAuth::new(auth_config.clone()))
            .wrap(cors)
            .app_data(web::Data::new(SessionState {
                sessions: sessions.clone(),
                build_server: build_server.clone(),
                port: config.port,
            }))
            .route("/sse", web::get().to(sse_handler))
            .route("/message", web::post().to(message_handler))
            .route("/ws", web::get().to(ws_handler))
    });

    // Add TLS if configured
    if let Some(tls_config) = &config.tls {
        use std::fs::File;
        use std::io::BufReader;
        
        // Load TLS keys
        let cert_file = File::open(&tls_config.cert_path)?;
        let key_file = File::open(&tls_config.key_path)?;
        let cert_reader = &mut BufReader::new(cert_file);
        let key_reader = &mut BufReader::new(key_file);

        // Parse TLS keys
        let cert_chain = certs(cert_reader)?.into_iter().map(Certificate).collect();
        let mut keys = pkcs8_private_keys(key_reader)?;
        if keys.is_empty() {
            anyhow::bail!("No private keys found");
        }

        // Create TLS config
        let tls_config = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))?;

        server = server.bind_rustls(("127.0.0.1", config.port), tls_config)?;
    } else {
        server = server.bind(("127.0.0.1", config.port))?;
    }

    server.run().await?;
    Ok(())
}

pub async fn http_server(
    port: u16,
    sessions: Arc<Mutex<HashMap<String, ServerHttpTransport>>>,
    auth_config: Option<AuthConfig>,
    build_server: Arc<
        dyn Fn(
                ServerHttpTransport,
            )
                -> futures::future::BoxFuture<'static, Result<Server>>
            + Send
            + Sync,
    >,
) -> std::result::Result<(), std::io::Error> {
    let session_state = SessionState {
        sessions,
        build_server,
        port,
    };

    let server = HttpServer::new(move || {
        let session_state = session_state.clone();
        App::new()
            .wrap(Logger::default())
            .wrap(JwtAuth::new(auth_config.clone()))
            .app_data(web::Data::new(session_state))
            .route("/sse", web::get().to(sse_handler))
            .route("/message", web::post().to(message_handler))
            .route("/ws", web::get().to(ws_handler))
    })
    .bind(("127.0.0.1", port))?
    .run();

    server.await
}

/// Handles SSE requests
///
/// # Arguments
/// * `req` - The HTTP request
/// * `session_state` - Shared session state
pub async fn sse_handler(
    req: actix_web::HttpRequest,
    session_state: web::Data<SessionState>,
) -> impl Responder {
    let client_ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    info!("New SSE connection request from {}", client_ip);

    // Create new session
    let session_id = Uuid::new_v4().to_string();

    // Create new SSE transport with responder
    let (transport, responder) = ServerSseTransport::new_with_responder(100);

    // Store transport in sessions map
    session_state
        .sessions
        .lock()
        .unwrap()
        .insert(session_id.clone(), ServerHttpTransport::Sse(transport.clone()));

    info!(
        "SSE connection established for {} with session_id {}",
        client_ip, session_id
    );

    // Send initial endpoint info
    let port = session_state.port;
    let endpoint_info = format!("http://127.0.0.1:{port}/message?sessionId={session_id}");
    if let Err(e) = transport.send_event("endpoint", endpoint_info).await {
        error!("Error sending endpoint info: {}", e);
        return Either::Left(HttpResponse::InternalServerError().finish());
    }

    // Create and spawn the server instance
    let transport_for_server = transport.clone();
    let build_server = session_state.build_server.clone();
    tokio::spawn(async move {
        match build_server(ServerHttpTransport::Sse(transport_for_server)).await {
            Ok(server) => {
                if let Err(e) = server.listen().await {
                    error!("Server error: {:?}", e);
                }
            }
            Err(e) => {
                error!("Failed to build server: {:?}", e);
            }
        }
    });

    // Return the SSE responder wrapped in Either::Right
    Either::Right(responder)
}

async fn message_handler(
    query: Query<MessageQuery>,
    message: web::Json<Message>,
    session_state: web::Data<SessionState>,
) -> HttpResponse {
    if let Some(session_id) = &query.session_id {
        let transport = {
            let sessions = session_state.sessions.lock().unwrap();
            sessions.get(session_id).cloned()
        };
        if let Some(transport) = transport {
            match transport {
                ServerHttpTransport::Sse(sse) => match sse.send_message(message.into_inner()).await
                {
                    Ok(_) => {
                        debug!("Successfully sent message to session {}", session_id);
                        HttpResponse::Accepted().finish()
                    }
                    Err(e) => {
                        error!("Failed to send message to session {}: {:?}", session_id, e);
                        HttpResponse::InternalServerError().finish()
                    }
                },
                ServerHttpTransport::Ws(_) => HttpResponse::BadRequest()
                    .body("Cannot send message to WebSocket connection through HTTP endpoint"),
            }
        } else {
            HttpResponse::NotFound().body(format!("Session {} not found", session_id))
        }
    } else {
        HttpResponse::BadRequest().body("Session ID not specified")
    }
}

async fn ws_handler(
    req: actix_web::HttpRequest,
    body: Payload,
    session_state: web::Data<SessionState>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    let client_ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    info!("New WebSocket connection from {}", client_ip);

    // Create channels for message passing
    let (tx, rx) = broadcast::channel(100);
    let transport =
        ServerHttpTransport::Ws(ServerWsTransport::new(session.clone(), rx.resubscribe()));

    // Store transport in sessions map
    let session_id = Uuid::new_v4().to_string();
    session_state
        .sessions
        .lock()
        .unwrap()
        .insert(session_id, transport.clone());

    // Start WebSocket handling in the background
    actix_web::rt::spawn(async move {
        let _ = handle_ws_connection(session, msg_stream, tx.clone(), rx.resubscribe()).await;
    });

    // Spawn server instance
    let build_server = session_state.build_server.clone();
    actix_web::rt::spawn(async move {
        if let Ok(server) = build_server(transport).await {
            let _ = server.listen().await;
        }
    });

    Ok(response)
}
