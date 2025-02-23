use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::server::notifications::LoggingLevel;
use crate::server::error::ServerError;
use crate::types::{Implementation, ServerCapabilities};

/// A request message
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelParams {
    /// The logging level to set
    pub level: LoggingLevel,
}

/// Parameters for a cancel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelParams {
    /// The ID of the request to cancel
    pub request_id: String,
    /// Optional reason for cancellation
    pub reason: Option<String>,
}

type Result<T> = std::result::Result<T, ServerError>;

/// A request handler for handling requests
pub trait RequestHandler: Send + Sync {
    /// Handle a request
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
