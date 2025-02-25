//! # Common Types for MCP Protocol
//!
//! This module defines the core types used throughout the MCP (Model Context Protocol) implementation.
//! It includes message types, capabilities, requests, responses, and other structures that form the
//! foundation of the protocol.
//!
//! ## Overview
//!
//! The MCP protocol defines a standardized way for AI models to interact with external tools and resources.
//! This module provides Rust implementations of the types defined in the MCP specification, with
//! serialization/deserialization support for JSON-RPC communication.
//!
//! ## Key Components
//!
//! * **Protocol Initialization**: Types for establishing connections between clients and servers
//! * **Capabilities**: Structures defining what features clients and servers support
//! * **Tools**: Types for defining, calling, and receiving responses from tools
//! * **Resources**: Types for accessing and managing external resources
//! * **Prompts**: Structures for working with prompt templates
//! * **Progress Tracking**: Types for monitoring long-running operations
//!
//! ## Examples
//!
//! ### Initializing a Connection
//!
//! ```rust,no_run
//! use mcp_daemon::types::{Implementation, InitializeRequest, ClientCapabilities};
//!
//! // Create client information
//! let client_info = Implementation {
//!     name: "example-client".to_string(),
//!     version: "0.1.0".to_string(),
//! };
//!
//! // Create initialization request
//! let request = InitializeRequest {
//!     protocol_version: "2024-11-05".to_string(),
//!     capabilities: ClientCapabilities::default(),
//!     client_info,
//! };
//! ```
//!
//! ### Working with Tools
//!
//! ```rust,no_run
//! use mcp_daemon::types::{Tool, CallToolRequest};
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! // Define a tool
//! let tool = Tool {
//!     name: "calculator".to_string(),
//!     description: Some("Performs basic arithmetic operations".to_string()),
//!     input_schema: json!({
//!         "type": "object",
//!         "properties": {
//!             "operation": {
//!                 "type": "string",
//!                 "enum": ["add", "subtract", "multiply", "divide"]
//!             },
//!             "a": { "type": "number" },
//!             "b": { "type": "number" }
//!         },
//!         "required": ["operation", "a", "b"]
//!     }),
//! };
//!
//! // Create a tool call request
//! let mut arguments = HashMap::new();
//! arguments.insert("operation".to_string(), json!("add"));
//! arguments.insert("a".to_string(), json!(5));
//! arguments.insert("b".to_string(), json!(3));
//!
//! let request = CallToolRequest {
//!     name: "calculator".to_string(),
//!     arguments: Some(arguments),
//!     meta: None,
//! };
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

/// The latest supported version of the MCP protocol
///
/// This constant defines the protocol version that this implementation supports.
/// It is used in initialization requests and responses to ensure compatibility
/// between clients and servers.
pub const LATEST_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Information about an MCP implementation (client or server)
///
/// This structure provides identification information about an MCP client or server
/// implementation, including its name and version.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::Implementation;
///
/// let client_info = Implementation {
///     name: "example-client".to_string(),
///     version: "0.1.0".to_string(),
/// };
/// ```
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
///
/// This is the first message sent by a client to establish a connection with an MCP server.
/// It includes the protocol version, client capabilities, and client information.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::{Implementation, InitializeRequest, ClientCapabilities};
///
/// let client_info = Implementation {
///     name: "example-client".to_string(),
///     version: "0.1.0".to_string(),
/// };
///
/// let request = InitializeRequest {
///     protocol_version: "2024-11-05".to_string(),
///     capabilities: ClientCapabilities::default(),
///     client_info,
/// };
/// ```
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
///
/// This is the server's response to an initialization request from a client.
/// It includes the protocol version, server capabilities, server information,
/// and optional instructions for the client.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::{Implementation, InitializeResponse, ServerCapabilities};
///
/// let server_info = Implementation {
///     name: "example-server".to_string(),
///     version: "0.1.0".to_string(),
/// };
///
/// let response = InitializeResponse {
///     protocol_version: "2024-11-05".to_string(),
///     capabilities: ServerCapabilities::default(),
///     server_info,
///     instructions: Some("Connect to the server using the provided credentials".to_string()),
/// };
/// ```
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
///
/// This structure defines the features and capabilities that an MCP server supports.
/// Each field represents a different category of capabilities, such as tools, resources,
/// prompts, etc.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::{ServerCapabilities, ToolCapabilities, ResourceCapabilities};
///
/// let capabilities = ServerCapabilities {
///     tools: Some(ToolCapabilities {
///         list_changed: Some(true),
///     }),
///     resources: Some(ResourceCapabilities {
///         subscribe: Some(true),
///         list_changed: Some(false),
///     }),
///     ..Default::default()
/// };
/// ```
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
///
/// This structure defines the tool-related capabilities that a server supports.
pub struct ToolCapabilities {
    /// Whether the tool list has changed
    ///
    /// If `true`, the client should refresh its list of available tools.
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to logging functionality
///
/// This structure defines the logging-related capabilities that a server supports.
pub struct LoggingCapabilities {
    /// Whether window-based logging is supported
    ///
    /// If `true`, the server supports window-based logging.
    pub window: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to sampling functionality
///
/// This structure defines the sampling-related capabilities that a server supports.
pub struct SamplingCapabilities {
    /// Whether the sampling list has changed
    ///
    /// If `true`, the client should refresh its list of available sampling options.
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to completion functionality
///
/// This structure defines the completion-related capabilities that a server supports.
pub struct CompletionCapabilities {
    /// Whether the completion list has changed
    ///
    /// If `true`, the client should refresh its list of available completion options.
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to progress tracking
///
/// This structure defines the progress tracking capabilities that a server supports.
pub struct ProgressCapabilities {
    /// Whether window-based progress tracking is supported
    ///
    /// If `true`, the server supports window-based progress tracking.
    pub window: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Progress information for a long-running operation
///
/// This structure provides information about the progress of a long-running operation.
/// It includes a unique token, the current progress value, and optional metadata.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::{Progress, ProgressValue};
/// use serde_json::json;
///
/// let progress = Progress {
///     token: "task-123".to_string(),
///     value: ProgressValue {
///         kind: "percentage".to_string(),
///         title: Some("Downloading file".to_string()),
///         message: Some("50% complete".to_string()),
///         percentage: Some(50.0),
///         cancellable: Some(true),
///     },
///     meta: Some(json!({
///         "fileName": "example.zip",
///         "fileSize": 1024000
///     })),
/// };
/// ```
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
///
/// This structure provides detailed information about the current state of a progress operation.
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
///
/// This structure is used to request cancellation of a long-running operation.
pub struct ProgressCancelRequest {
    /// Token of the progress operation to cancel
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to prompt functionality
///
/// This structure defines the prompt-related capabilities that a server supports.
pub struct PromptCapabilities {
    /// Whether the prompt list has changed
    ///
    /// If `true`, the client should refresh its list of available prompts.
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities related to resource functionality
///
/// This structure defines the resource-related capabilities that a server supports.
pub struct ResourceCapabilities {
    /// Whether resource subscription is supported
    ///
    /// If `true`, the server supports subscribing to resource updates.
    pub subscribe: Option<bool>,
    /// Whether the resource list has changed
    ///
    /// If `true`, the client should refresh its list of available resources.
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Capabilities supported by the client
///
/// This structure defines the features and capabilities that an MCP client supports.
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
///
/// This structure defines the root-related capabilities that a client or server supports.
pub struct RootCapabilities {
    /// Whether the root list has changed
    ///
    /// If `true`, the client should refresh its list of available roots.
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Description of a tool that can be called by the client
///
/// This structure defines a tool that can be called by an MCP client. It includes
/// the tool's name, an optional description, and a JSON schema describing the tool's
/// input parameters.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::Tool;
/// use serde_json::json;
///
/// let tool = Tool {
///     name: "calculator".to_string(),
///     description: Some("Performs basic arithmetic operations".to_string()),
///     input_schema: json!({
///         "type": "object",
///         "properties": {
///             "operation": {
///                 "type": "string",
///                 "enum": ["add", "subtract", "multiply", "divide"]
///             },
///             "a": { "type": "number" },
///             "b": { "type": "number" }
///         },
///         "required": ["operation", "a", "b"]
///     }),
/// };
/// ```
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
///
/// This structure is used to request execution of a tool by an MCP server.
/// It includes the tool's name, optional arguments, and optional metadata.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::CallToolRequest;
/// use serde_json::json;
/// use std::collections::HashMap;
///
/// let mut arguments = HashMap::new();
/// arguments.insert("operation".to_string(), json!("add"));
/// arguments.insert("a".to_string(), json!(5));
/// arguments.insert("b".to_string(), json!(3));
///
/// let request = CallToolRequest {
///     name: "calculator".to_string(),
///     arguments: Some(arguments),
///     meta: None,
/// };
/// ```
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
///
/// This structure contains the result of a tool call. It includes the content
/// returned by the tool, whether the response represents an error, and optional metadata.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::{CallToolResponse, Content};
/// use serde_json::json;
///
/// let response = CallToolResponse {
///     content: vec![Content::Text { text: "8".to_string() }],
///     is_error: None,
///     meta: Some(json!({
///         "executionTime": 0.05
///     })),
/// };
/// ```
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
///
/// This enum represents the different types of content that can be returned by a tool.
/// It includes text, images, and resources.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::Content;
///
/// // Text content
/// let text_content = Content::Text {
///     text: "Hello, world!".to_string()
/// };
///
/// // Image content
/// let image_content = Content::Image {
///     data: "base64encodeddata...".to_string(),
///     mime_type: "image/png".to_string()
/// };
/// ```
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
///
/// This structure represents the contents of a resource that can be accessed by a client.
/// It includes the URI where the resource can be accessed, and optional MIME type, text content,
/// and binary content.
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
///
/// This structure is used to request reading a resource from an MCP server.
/// It includes the URI of the resource to read.
pub struct ReadResourceRequest {
    /// URI of the resource to read
    pub uri: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to list items with pagination
///
/// This structure is used to request a list of items from an MCP server.
/// It includes an optional cursor for pagination and optional metadata.
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
///
/// This structure contains a list of tools available from an MCP server.
/// It includes the list of tools, an optional cursor for pagination, and optional metadata.
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
///
/// This structure contains a list of prompts available from an MCP server.
/// It includes the list of prompts, an optional cursor for pagination, and optional metadata.
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
///
/// This structure defines a prompt that can be used by an MCP client. It includes
/// the prompt's name, an optional description, and optional arguments.
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
///
/// This structure defines an argument that is required by a prompt.
/// It includes the argument's name, an optional description, and whether the argument is required.
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
///
/// This structure contains a list of resources available from an MCP server.
/// It includes the list of resources, an optional cursor for pagination, and optional metadata.
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
///
/// This structure defines a resource that can be accessed by an MCP client.
/// It includes the resource's URI, name, optional description, and optional MIME type.
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
///
/// This enum defines the standard error codes used in the MCP protocol.
/// It includes both SDK-specific error codes and standard JSON-RPC error codes.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::ErrorCode;
///
/// // SDK error code
/// let connection_closed = ErrorCode::ConnectionClosed;
/// assert_eq!(connection_closed as i32, -32000);
///
/// // Standard JSON-RPC error code
/// let invalid_params = ErrorCode::InvalidParams;
/// assert_eq!(invalid_params as i32, -32602);
/// ```
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
///
/// This enum represents the different types of content that can be included in messages.
/// It includes text and images.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::MessageContent;
///
/// // Text content
/// let text_content = MessageContent::Text {
///     text: "Hello, world!".to_string()
/// };
///
/// // Image content
/// let image_content = MessageContent::Image {
///     data: "base64encodeddata...".to_string(),
///     mime_type: "image/png".to_string()
/// };
/// ```
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
///
/// This structure represents a message in a prompt, including the role of the sender
/// and the content of the message.
///
/// # Examples
///
/// ```
/// use mcp_daemon::types::{PromptMessage, MessageContent};
///
/// let message = PromptMessage {
///     role: "user".to_string(),
///     content: MessageContent::Text {
///         text: "What is the capital of France?".to_string()
///     },
/// };
/// ```
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
