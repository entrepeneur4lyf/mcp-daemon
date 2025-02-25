//! # MCP Request System
//!
//! This module provides types and utilities for handling MCP requests,
//! which are messages sent from clients to servers expecting a response.
//!
//! ## Overview
//!
//! The request system defines the core request types in the MCP protocol:
//!
//! * Initialization requests for establishing connections
//! * Ping requests for health checks
//! * Logging level control requests
//! * Request cancellation
//!
//! Each request type has associated parameter structures and expected response types.
//!
//! ## Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use mcp_daemon::server::requests::{Request, InitializeParams, RequestHandler};
//! use mcp_daemon::server::error::ServerError;
//! use mcp_daemon::types::Implementation;
//!
//! // Create an initialization request
//! let init_request = Request::Initialize(InitializeParams {
//!     protocol_version: "1.0.0".to_string(),
//!     capabilities: HashMap::new(),
//!     client_info: Implementation {
//!         name: "example-client".to_string(),
//!         version: "1.0.0".to_string(),
//!     },
//! });
//!
//! // Create a simple request handler
//! struct MyRequestHandler;
//!
//! impl RequestHandler for MyRequestHandler {
//!     fn handle(&self, request: Request) -> Result<serde_json::Value, ServerError> {
//!         match request {
//!             Request::Ping => {
//!                 // Handle ping request
//!                 Ok(serde_json::json!({"status": "ok"}))
//!             }
//!             Request::Initialize(params) => {
//!                 // Handle initialization request
//!                 println!("Client {} version {} connected", 
//!                     params.client_info.name, 
//!                     params.client_info.version);
//!                 Ok(serde_json::json!({}))
//!             }
//!             _ => Err(ServerError::MethodNotFound),
//!         }
//!     }
//! }
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::server::error`] - Error handling
//! * [`crate::types`] - Core MCP types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::server::notifications::LoggingLevel;
use crate::server::error::ServerError;
use crate::types::{Implementation, ServerCapabilities};

/// A request message
///
/// Represents a message sent from a client to a server that expects a response.
/// The server must respond to each request with either a result or an error.
///
/// # Variants
///
/// * `Ping` - Simple health check request
/// * `Initialize` - Connection initialization request with client capabilities
/// * `SetLevel` - Request to set the logging level
/// * `Cancel` - Request to cancel a previous request
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use mcp_daemon::server::requests::{Request, InitializeParams, CancelParams, SetLevelParams};
/// use mcp_daemon::server::notifications::LoggingLevel;
/// use mcp_daemon::types::Implementation;
///
/// // Ping request
/// let ping = Request::Ping;
///
/// // Initialize request
/// let init = Request::Initialize(InitializeParams {
///     protocol_version: "1.0.0".to_string(),
///     capabilities: HashMap::new(),
///     client_info: Implementation {
///         name: "example-client".to_string(),
///         version: "1.0.0".to_string(),
///     },
/// });
///
/// // Set logging level request
/// let set_level = Request::SetLevel(SetLevelParams {
///     level: LoggingLevel::Info,
/// });
///
/// // Cancel request
/// let cancel = Request::Cancel(CancelParams {
///     request_id: "req-123".to_string(),
///     reason: Some("User cancelled".to_string()),
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    #[serde(rename = "ping")]
    /// Ping request to check server health
    Ping,

    #[serde(rename = "initialize")]
    /// Initialize request with parameters
    Initialize(InitializeParams),

    #[serde(rename = "logging/setLevel")]
    /// Set logging level request with parameters
    SetLevel(SetLevelParams),

    #[serde(rename = "cancel")]
    /// Cancel request with parameters
    Cancel(CancelParams),
}

/// Parameters for an initialize request
///
/// Contains information about the client's capabilities and implementation
/// details for establishing a connection with the server.
///
/// # Fields
///
/// * `protocol_version` - The protocol version the client is using
/// * `capabilities` - The client's capabilities
/// * `client_info` - Information about the client implementation
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use mcp_daemon::server::requests::InitializeParams;
/// use mcp_daemon::types::Implementation;
///
/// let params = InitializeParams {
///     protocol_version: "1.0.0".to_string(),
///     capabilities: HashMap::new(),
///     client_info: Implementation {
///         name: "example-client".to_string(),
///         version: "1.0.0".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// The protocol version the client is using
    pub protocol_version: String,
    /// The client's capabilities
    pub capabilities: HashMap<String, serde_json::Value>,
    /// Information about the client implementation
    pub client_info: Implementation,
}

/// Result of an initialize request
///
/// Contains information about the server's capabilities and implementation
/// details in response to an initialization request.
///
/// # Fields
///
/// * `protocol_version` - The protocol version the server is using
/// * `capabilities` - The server's capabilities
/// * `server_info` - Information about the server implementation
/// * `instructions` - Optional instructions for the client
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::requests::InitializeResult;
/// use mcp_daemon::types::{Implementation, ServerCapabilities};
///
/// let result = InitializeResult {
///     protocol_version: "1.0.0".to_string(),
///     capabilities: ServerCapabilities::default(),
///     server_info: Implementation {
///         name: "example-server".to_string(),
///         version: "1.0.0".to_string(),
///     },
///     instructions: Some("Connect to resource at example://resource".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// The protocol version the server is using
    pub protocol_version: String,
    /// The server's capabilities
    pub capabilities: ServerCapabilities,
    /// Information about the server implementation
    pub server_info: Implementation,
    /// Optional instructions for the client
    pub instructions: Option<String>,
}

/// Parameters for a set level request
///
/// Contains the logging level to set for the server.
///
/// # Fields
///
/// * `level` - The logging level to set
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::requests::SetLevelParams;
/// use mcp_daemon::server::notifications::LoggingLevel;
///
/// let params = SetLevelParams {
///     level: LoggingLevel::Info,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelParams {
    /// The logging level to set
    pub level: LoggingLevel,
}

/// Parameters for a cancel request
///
/// Contains information about the request to cancel.
///
/// # Fields
///
/// * `request_id` - The ID of the request to cancel
/// * `reason` - Optional reason for cancellation
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::requests::CancelParams;
///
/// let params = CancelParams {
///     request_id: "req-123".to_string(),
///     reason: Some("User cancelled".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelParams {
    /// The ID of the request to cancel
    pub request_id: String,
    /// Optional reason for cancellation
    pub reason: Option<String>,
}

type Result<T> = std::result::Result<T, ServerError>;

/// A request handler for handling requests
///
/// This trait defines the interface for components that can handle
/// MCP requests and produce responses.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::requests::{Request, RequestHandler};
/// use mcp_daemon::server::error::{ServerError, ErrorCode};
///
/// struct MyRequestHandler;
///
/// impl RequestHandler for MyRequestHandler {
///     fn handle(&self, request: Request) -> Result<serde_json::Value, ServerError> {
///         match request {
///             Request::Ping => {
///                 // Handle ping request
///                 Ok(serde_json::json!({"status": "ok"}))
///             }
///             _ => Err(ServerError::new(ErrorCode::MethodNotFound, "Method not found")),
///         }
///     }
/// }
/// ```
pub trait RequestHandler: Send + Sync {
    /// Handle a request
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle
    ///
    /// # Returns
    ///
    /// A Result containing the response value or an error
    fn handle(&self, request: Request) -> Result<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request::Initialize(InitializeParams {
            protocol_version: "1.0.0".to_string(),
            capabilities: HashMap::new(),
            client_info: Implementation {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        });

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();

        match deserialized {
            Request::Initialize(params) => {
                assert_eq!(params.protocol_version, "1.0.0");
                assert_eq!(params.client_info.name, "test-client");
            }
            _ => panic!("Wrong request type"),
        }
    }
}
