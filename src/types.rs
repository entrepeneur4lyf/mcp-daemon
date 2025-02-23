use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

/// The latest supported version of the MCP protocol
pub const LATEST_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Information about an MCP implementation (client or server)
pub struct Implementation {
    /// Name of the implementation
    pub name: String,
    /// Version of the implementation
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Request to initialize an MCP connection
pub struct InitializeRequest {
    /// Version of the MCP protocol to use
    pub protocol_version: String,
    /// Capabilities supported by the client
    pub capabilities: ClientCapabilities,
    /// Information about the client implementation
    pub client_info: Implementation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Response to an initialization request
pub struct InitializeResponse {
    /// Version of the MCP protocol being used
    pub protocol_version: String,
    /// Capabilities supported by the server
    pub capabilities: ServerCapabilities,
    /// Information about the server implementation
    pub server_info: Implementation,
    /// Optional instructions for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities supported by the server
pub struct ServerCapabilities {
    /// Tool-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
    /// Logging-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapabilities>,
    /// Prompt-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,
    /// Resource-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,
    /// Progress tracking capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressCapabilities>,
    /// Completion-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionCapabilities>,
    /// Sampling-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapabilities>,
    /// Root-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to tool functionality
pub struct ToolCapabilities {
    /// Whether the tool list has changed
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to logging functionality
pub struct LoggingCapabilities {
    /// Whether window-based logging is supported
    pub window: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to sampling functionality
pub struct SamplingCapabilities {
    /// Whether the sampling list has changed
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to completion functionality
pub struct CompletionCapabilities {
    /// Whether the completion list has changed
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to progress tracking
pub struct ProgressCapabilities {
    /// Whether window-based progress tracking is supported
    pub window: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Progress information for a long-running operation
pub struct Progress {
    /// Unique token identifying this progress item
    pub token: String,
    /// Current progress value
    pub value: ProgressValue,
    /// Optional metadata associated with the progress
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Value representing the state of a progress operation
pub struct ProgressValue {
    /// Type of progress (e.g., 'percentage', 'count')
    pub kind: String,
    /// Optional title describing the operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional message about the current state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Progress as a percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
    /// Whether the operation can be cancelled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to cancel a progress operation
pub struct ProgressCancelRequest {
    /// Token of the progress operation to cancel
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to prompt functionality
pub struct PromptCapabilities {
    /// Whether the prompt list has changed
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to resource functionality
pub struct ResourceCapabilities {
    /// Whether resource subscription is supported
    pub subscribe: Option<bool>,
    /// Whether the resource list has changed
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities supported by the client
pub struct ClientCapabilities {
    /// Experimental features
    pub experimental: Option<serde_json::Value>,
    /// Sampling-related capabilities
    pub sampling: Option<SamplingCapabilities>,
    /// Root-related capabilities
    pub roots: Option<RootCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to root functionality
pub struct RootCapabilities {
    /// Whether the root list has changed
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Description of a tool that can be called by the client
pub struct Tool {
    /// Name of the tool
    pub name: String,
    /// Optional description of what the tool does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON schema describing the tool's input parameters
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to call a tool
pub struct CallToolRequest {
    /// Name of the tool to call
    pub name: String,
    /// Optional arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, serde_json::Value>>,
    /// Optional metadata for the request
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Response from a tool call
pub struct CallToolResponse {
    /// Content returned by the tool
    pub content: Vec<Content>,
    /// Whether the response represents an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Optional metadata for the response
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
/// Content that can be returned by a tool
pub enum Content {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String
    },
    /// Image content with data and MIME type
    #[serde(rename = "image")]
    Image {
        /// The image data (e.g., base64 encoded)
        data: String,
        /// The MIME type of the image
        mime_type: String
    },
    /// Resource content
    #[serde(rename = "resource")]
    Resource {
        /// The resource contents
        resource: ResourceContents
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Contents of a resource
pub struct ResourceContents {
    /// URI where the resource can be accessed
    pub uri: Url,
    /// Optional MIME type of the resource content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Optional text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Optional binary content as base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to read a resource
pub struct ReadResourceRequest {
    /// URI of the resource to read
    pub uri: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to list items with pagination
pub struct ListRequest {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Optional metadata for the request
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Response containing a list of tools
pub struct ToolsListResponse {
    /// List of available tools
    pub tools: Vec<Tool>,
    /// Optional cursor for the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Optional metadata for the response
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Response containing a list of prompts
pub struct PromptsListResponse {
    /// List of available prompts
    pub prompts: Vec<Prompt>,
    /// Optional cursor for the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Optional metadata for the response
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Description of a prompt that can be used by the client
pub struct Prompt {
    /// Name of the prompt
    pub name: String,
    /// Optional description of what the prompt does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional list of arguments required by the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Description of an argument required by a prompt
pub struct PromptArgument {
    /// Name of the argument
    pub name: String,
    /// Optional description of what the argument is for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the argument is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Response containing a list of resources
pub struct ResourcesListResponse {
    /// List of available resources
    pub resources: Vec<Resource>,
    /// Optional cursor for the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Optional metadata for the response
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Description of a resource that can be accessed by the client
pub struct Resource {
    /// URI where the resource can be accessed
    pub uri: Url,
    /// Name of the resource
    pub name: String,
    /// Optional description of what the resource contains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional MIME type of the resource content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Error codes for various error conditions in the MCP protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // SDK error codes
    /// Indicates that the connection to the server was closed unexpectedly
    ConnectionClosed = -32000,
    /// Indicates that a request to the server timed out
    RequestTimeout = -32001,

    // Standard JSON-RPC error codes
    /// Indicates an error while parsing the JSON-RPC request
    ParseError = -32700,
    /// Indicates that the JSON-RPC request was invalid
    InvalidRequest = -32600,
    /// Indicates that the method specified in the JSON-RPC request was not found
    MethodNotFound = -32601,
    /// Indicates that the parameters provided in the JSON-RPC request were invalid
    InvalidParams = -32602,
    /// Indicates an internal error occurred while processing the JSON-RPC request
    InternalError = -32603,
}

/// Message content types shared between prompts and tool responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    /// Represents text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String
    },
    /// Represents image content
    #[serde(rename = "image")]
    Image {
        /// The image data (e.g., base64 encoded)
        data: String,
        /// The MIME type of the image
        mime_type: String
    },
}

/// Represents a message in a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptMessage {
    /// The role of the message sender (e.g., "user", "assistant")
    pub role: String,
    /// The content of the message
    pub content: MessageContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_capabilities() {
        let capabilities = ServerCapabilities::default();
        let json = serde_json::to_string(&capabilities).unwrap();
        assert_eq!(json, "{}");
    }
}
