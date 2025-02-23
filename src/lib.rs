//! # mcp_daemon
//!
//! `mcp_daemon` is a comprehensive implementation of the Model Context Protocol (MCP) in Rust.
//! It provides a robust, async-first approach to building MCP-compatible servers and clients.
//!
//! ## Features
//!
//! - Full implementation of the MCP specification
//! - Async support using Tokio
//! - Multiple transport layers: Stdio, In-Memory, SSE, WebSockets
//! - Comprehensive error handling and recovery mechanisms
//! - Support for tools, prompts, resources, completion, sampling, and roots
//! - Integration with various LLM providers through bridges
//!
//! ## Main Components
//!
//! - `client`: MCP client implementation
//! - `server`: MCP server implementation
//! - `transport`: Various transport layer implementations
//! - `protocol`: Core MCP protocol definitions
//! - `types`: Common types used throughout the crate
//! - `completable`: Implementations for completable items
//! - `registry`: Registry for MCP components
//! - `sse`: Server-Sent Events implementation
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

// Re-export main components for easier access
pub use client::Client;
pub use server::Server;
