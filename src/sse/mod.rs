//! # Server-Sent Events (SSE) Module
//!
//! This module provides Server-Sent Events (SSE) implementation for the MCP protocol,
//! enabling real-time, unidirectional communication from server to client over HTTP.
//! 
//! SSE is a standard that allows a web server to push real-time updates to a client
//! over a single HTTP connection. It's simpler than WebSockets and is ideal for
//! scenarios where the server needs to send updates to the client without requiring
//! bidirectional communication.
//!
//! The SSE implementation in MCP consists of:
//! 
//! 1. An HTTP server that handles SSE connections and message routing
//! 2. Authentication middleware for securing SSE endpoints
//! 3. Transport implementations that integrate with the MCP protocol
//!
//! ## Key Features
//!
//! - HTTP-based server implementation with SSE streaming
//! - JWT authentication for securing connections
//! - Support for both SSE and WebSocket connections
//! - TLS support for secure communications
//! - CORS configuration for cross-origin requests
//!
//! ## Usage
//!
//! The SSE module is typically used when implementing MCP servers that need to
//! communicate with clients over HTTP, especially in web browser environments
//! where WebSockets might be restricted.

/// HTTP server implementation for SSE
pub mod http_server;
/// Middleware for SSE authentication
pub mod middleware;
