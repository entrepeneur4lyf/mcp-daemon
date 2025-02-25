//! # mcp_daemon
//!
//! `mcp_daemon` is a comprehensive implementation of the Model Context Protocol (MCP) in Rust.
//! It provides a robust, async-first approach to building MCP-compatible servers and clients.
//!
//! ## Overview
//!
//! The Model Context Protocol (MCP) enables communication between AI models and external tools,
//! allowing models to access real-time information, perform actions, and interact with various
//! systems. This implementation goes beyond the standard specification to provide:
//!
//! - **Full Specification Coverage**: Implements every feature from the latest MCP spec
//! - **Production-Grade Error Handling**: Comprehensive error system with recovery mechanisms
//! - **Advanced Transport Layer**: Robust implementations of all transport types with detailed error tracking
//! - **Type-Safe Architecture**: Leveraging Rust's type system for compile-time correctness
//! - **Real-World Ready**: Production-tested with Claude Desktop compatibility
//!
//! ## Example
//!
//! ```rust,no_run
//! use mcp_daemon::client::Client;
//! use mcp_daemon::transport::{ClientStdioTransport, Transport};
//! use mcp_daemon::types::Implementation;
//! use mcp_daemon::protocol::RequestOptions;
//! use serde_json::json;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a transport
//!     let transport = ClientStdioTransport::new("mcp-server", &[])?;
//!     transport.open().await?;
//!     
//!     // Create a client
//!     let client = Client::builder(transport.clone()).build();
//!     
//!     // Initialize the client
//!     let client_info = Implementation {
//!         name: "example-client".to_string(),
//!         version: "0.1.0".to_string(),
//!     };
//!     client.initialize(client_info).await?;
//!     
//!     // Make a request to list available tools
//!     let response = client.request(
//!         "tools/list",
//!         None,
//!         Default::default()
//!     ).await?;
//!     
//!     // Call a tool
//!     let tool_response = client.request(
//!         "tools/call",
//!         Some(json!({
//!             "name": "example-tool",
//!             "arguments": {
//!                 "param1": "value1"
//!             }
//!         })),
//!         Default::default()
//!     ).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Main Components
//!
//! - [`client`]: MCP client implementation for interacting with servers
//! - [`server`]: MCP server implementation for handling client requests
//! - [`transport`]: Various transport layer implementations (Stdio, In-Memory, SSE, WebSockets)
//! - [`protocol`]: Core MCP protocol definitions and message handling
//! - [`types`]: Common types used throughout the crate
//! - [`completable`]: Implementations for completable items
//! - [`registry`]: Registry for MCP components
//! - [`sse`]: Server-Sent Events implementation
//! - [`bridge`]: Bridges to integrate with various LLM providers
//!
//! ## Transport Layer
//!
//! The transport layer provides multiple implementations for different use cases:
//!
//! - **Stdio Transport**: Uses standard input/output for communication, ideal for command-line tools
//! - **In-Memory Transport**: Uses Tokio channels for efficient inter-process communication
//! - **SSE Transport**: Implements Server-Sent Events for unidirectional server-to-client communication
//! - **WebSocket Transport**: Provides full-duplex communication with comprehensive error handling
//!
//! ## Error Handling
//!
//! All components include detailed error handling with specific error types and recovery mechanisms.
//! The transport layer provides comprehensive error information through the `TransportError` type,
//! including specific error codes and messages for various failure scenarios.
//!
//! ## Feature Flags
//!
//! This crate uses Rust's feature flags to enable or disable certain functionality:
//!
//! - `default`: Includes all stable features
//! - `unstable`: Enables experimental features that may change in future releases
//!
//! For more detailed information on each component, please refer to their respective module documentation.

#![warn(missing_docs)]

/// Client implementation for interacting with MCP servers
pub mod client;

/// Trait and implementations for completable items in MCP
pub mod completable;

/// Core protocol definitions and message types for MCP
pub mod protocol;

/// Registry for managing MCP components and their relationships
pub mod registry;

/// Server implementation for handling MCP requests and responses
pub mod server;

/// Server-Sent Events (SSE) implementation for real-time communication
pub mod sse;
pub use sse::http_server::run_http_server;

/// Transport layer implementations for different communication protocols
pub mod transport;

/// Common types and structures used throughout the crate
pub mod types;

/// Bridge implementations for integrating with various LLM providers
pub mod bridge;

// Re-export main components for easier access
pub use client::Client;
pub use server::Server;

// # Transport Layer Examples
//
// ### Server-Side Stdio Transport
//
// ```rust,no_run
// use mcp_daemon::transport::{ServerStdioTransport, Message};
// ```
//
// ### Client-Side WebSocket Transport
//
// ```rust,no_run
// use mcp_daemon::transport::ClientWsTransportBuilder;
// use std::time::Duration;
//
// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
// // Create a WebSocket transport
// let transport = ClientWsTransportBuilder::new("ws://localhost:3004/ws".to_string())
//     .build();
// # Ok(())
// # }
// ```
