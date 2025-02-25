//! HTTP transport types for the Model Context Protocol
//! This module provides the transport layer for HTTP-based communication.

use super::{Message, Result, Transport};
use super::ws_transport::{ClientWsTransport, ServerWsTransport};
use super::sse_transport::ServerSseTransport;
use async_trait::async_trait;

/// Server-side HTTP transport variants
#[derive(Debug, Clone)]
pub enum ServerHttpTransport {
    /// Server-Sent Events transport
    Sse(ServerSseTransport),
    /// WebSocket transport
    Ws(ServerWsTransport),
}

/// Client-side HTTP transport variants
#[derive(Debug, Clone)]
pub enum ClientHttpTransport {
    /// WebSocket transport
    Ws(ClientWsTransport),
}

#[async_trait]
impl Transport for ServerHttpTransport {
    async fn send(&self, message: &Message) -> Result<()> {
        match self {
            Self::Sse(transport) => transport.send(message).await,
            Self::Ws(transport) => transport.send(message).await,
        }
    }

    async fn receive(&self) -> Result<Option<Message>> {
        match self {
            Self::Sse(transport) => transport.receive().await,
            Self::Ws(transport) => transport.receive().await,
        }
    }

    async fn open(&self) -> Result<()> {
        match self {
            Self::Sse(transport) => transport.open().await,
            Self::Ws(transport) => transport.open().await,
        }
    }

    async fn close(&self) -> Result<()> {
        match self {
            Self::Sse(transport) => transport.close().await,
            Self::Ws(transport) => transport.close().await,
        }
    }
}

#[async_trait]
impl Transport for ClientHttpTransport {
    async fn send(&self, message: &Message) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.send(message).await,
        }
    }

    async fn receive(&self) -> Result<Option<Message>> {
        match self {
            Self::Ws(transport) => transport.receive().await,
        }
    }

    async fn open(&self) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.open().await,
        }
    }

    async fn close(&self) -> Result<()> {
        match self {
            Self::Ws(transport) => transport.close().await,
        }
    }
}


