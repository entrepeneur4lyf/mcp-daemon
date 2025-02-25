//! # Tool Registry for MCP
//!
//! This module provides a registry for MCP tools, allowing tools to be registered,
//! listed, and called by name. It serves as a central repository for all available
//! tools in an MCP server implementation.
//!
//! ## Overview
//!
//! The tool registry manages:
//! - Tool registration and storage
//! - Tool lookup by name
//! - Tool execution with proper argument handling
//! - Tool listing for discovery
//!
//! ## Key Components
//!
//! * **Tools**: The main registry that stores and manages tool handlers
//! * **ToolHandler**: Internal structure that pairs a tool definition with its implementation
//!
//! ## Examples
//!
//! ```rust,no_run
//! use mcp_daemon::registry::Tools;
//! use mcp_daemon::types::{Tool, CallToolRequest, CallToolResponse, Content};
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a tool registry
//! let mut registry = Tools::new();
//!
//! // Register a calculator tool
//! let calculator_tool = Tool {
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
//! // Register the tool with its implementation
//! registry.register_tool(calculator_tool, |request| {
//!     Box::pin(async move {
//!         // Extract arguments
//!         let args = request.arguments.unwrap_or_default();
//!         let operation = args.get("operation").and_then(|v| v.as_str()).unwrap_or("add");
//!         let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
//!         let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
//!         
//!         // Perform calculation
//!         let result = match operation {
//!             "add" => a + b,
//!             "subtract" => a - b,
//!             "multiply" => a * b,
//!             "divide" => a / b,
//!             _ => return Err(anyhow::anyhow!("Unknown operation: {}", operation)),
//!         };
//!         
//!         // Return response
//!         Ok(CallToolResponse {
//!             content: vec![Content::Text { text: result.to_string() }],
//!             is_error: None,
//!             meta: None,
//!         })
//!     })
//! });
//!
//! // Get a list of all tools
//! let tools = registry.list_tools();
//! assert_eq!(tools.len(), 1);
//! assert_eq!(tools[0].name, "calculator");
//!
//! // Call the tool
//! let mut args = HashMap::new();
//! args.insert("operation".to_string(), json!("add"));
//! args.insert("a".to_string(), json!(5));
//! args.insert("b".to_string(), json!(3));
//!
//! let request = CallToolRequest {
//!     name: "calculator".to_string(),
//!     arguments: Some(args),
//!     meta: None,
//! };
//!
//! let response = registry.call_tool(request).await?;
//! # Ok(())
//! # }
//! ```

use crate::types::{CallToolRequest, CallToolResponse, Tool};
use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

// Type alias for tool handler map
type ToolHandlerMap = HashMap<String, ToolHandler>;

/// A registry of tools that can be called by the server
///
/// The `Tools` struct serves as a central registry for all available tools in an MCP server.
/// It allows tools to be registered, listed, and called by name.
///
/// # Examples
///
/// ```rust,no_run
/// use mcp_daemon::registry::Tools;
/// use mcp_daemon::types::{Tool, CallToolRequest, CallToolResponse, Content};
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Create a tool registry
/// let mut registry = Tools::new();
///
/// // Register a simple echo tool
/// let echo_tool = Tool {
///     name: "echo".to_string(),
///     description: Some("Echoes back the input".to_string()),
///     input_schema: json!({
///         "type": "object",
///         "properties": {
///             "message": { "type": "string" }
///         },
///         "required": ["message"]
///     }),
/// };
///
/// // Register the tool with its implementation
/// registry.register_tool(echo_tool, |request| {
///     Box::pin(async move {
///         let args = request.arguments.unwrap_or_default();
///         let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
///         
///         Ok(CallToolResponse {
///             content: vec![Content::Text { text: message.to_string() }],
///             is_error: None,
///             meta: None,
///         })
///     })
/// });
/// # Ok(())
/// # }
/// ```
pub struct Tools {
    /// Map of tool handlers indexed by tool name
    tool_handlers: ToolHandlerMap,
}

impl Tools {
    /// Create a new empty tool registry
    ///
    /// # Returns
    ///
    /// A new `Tools` instance with no registered tools.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::registry::Tools;
    ///
    /// let registry = Tools::new();
    /// assert_eq!(registry.list_tools().len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            tool_handlers: HashMap::new(),
        }
    }

    /// Register a new tool with the registry
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool definition
    /// * `handler` - The function that implements the tool's behavior
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::registry::Tools;
    /// # use mcp_daemon::types::{Tool, CallToolRequest, CallToolResponse, Content};
    /// # use serde_json::json;
    /// # fn example() {
    /// let mut registry = Tools::new();
    ///
    /// let echo_tool = Tool {
    ///     name: "echo".to_string(),
    ///     description: Some("Echoes back the input".to_string()),
    ///     input_schema: json!({
    ///         "type": "object",
    ///         "properties": {
    ///             "message": { "type": "string" }
    ///         },
    ///         "required": ["message"]
    ///     }),
    /// };
    ///
    /// registry.register_tool(echo_tool, |request| {
    ///     Box::pin(async move {
    ///         let args = request.arguments.unwrap_or_default();
    ///         let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
    ///         
    ///         Ok(CallToolResponse {
    ///             content: vec![Content::Text { text: message.to_string() }],
    ///             is_error: None,
    ///             meta: None,
    ///         })
    ///     })
    /// });
    /// # }
    /// ```
    pub fn register_tool<F, Fut>(&mut self, tool: Tool, handler: F)
    where
        F: Fn(CallToolRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<CallToolResponse>> + Send + 'static,
    {
        let handler_fn = Box::new(move |req: CallToolRequest| -> ToolFuture {
            Box::pin(handler(req))
        });

        let tool_handler = ToolHandler {
            tool: tool.clone(),
            f: handler_fn,
        };

        self.tool_handlers.insert(tool.name.clone(), tool_handler);
    }

    /// Get a tool by its name
    ///
    /// # Arguments
    /// * `name` - The name of the tool to retrieve
    ///
    /// # Returns
    /// The tool if it exists, None otherwise
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::registry::Tools;
    /// # use mcp_daemon::types::{Tool, CallToolRequest, CallToolResponse, Content};
    /// # use serde_json::json;
    /// # fn example() {
    /// # let mut registry = Tools::new();
    /// # let echo_tool = Tool {
    /// #     name: "echo".to_string(),
    /// #     description: Some("Echoes back the input".to_string()),
    /// #     input_schema: json!({}),
    /// # };
    /// # registry.register_tool(echo_tool, |request| {
    /// #     Box::pin(async move {
    /// #         Ok(CallToolResponse {
    /// #             content: vec![Content::Text { text: "".to_string() }],
    /// #             is_error: None,
    /// #             meta: None,
    /// #         })
    /// #     })
    /// # });
    /// // Get a tool by name
    /// let tool = registry.get_tool("echo");
    /// assert!(tool.is_some());
    /// assert_eq!(tool.unwrap().name, "echo");
    ///
    /// // Try to get a non-existent tool
    /// let tool = registry.get_tool("nonexistent");
    /// assert!(tool.is_none());
    /// # }
    /// ```
    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_handlers
            .get(name)
            .map(|tool_handler| tool_handler.tool.clone())
    }

    /// Call a tool with the given request
    ///
    /// # Arguments
    /// * `req` - The request containing the tool name and arguments
    ///
    /// # Returns
    /// The response from the tool, or an error if the tool doesn't exist or fails
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The requested tool doesn't exist
    /// * The tool implementation returns an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::registry::Tools;
    /// # use mcp_daemon::types::{Tool, CallToolRequest, CallToolResponse, Content};
    /// # use serde_json::json;
    /// # use std::collections::HashMap;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let mut registry = Tools::new();
    /// # let echo_tool = Tool {
    /// #     name: "echo".to_string(),
    /// #     description: Some("Echoes back the input".to_string()),
    /// #     input_schema: json!({}),
    /// # };
    /// # registry.register_tool(echo_tool, |request| {
    /// #     Box::pin(async move {
    /// #         Ok(CallToolResponse {
    /// #             content: vec![Content::Text { text: "Echo response".to_string() }],
    /// #             is_error: None,
    /// #             meta: None,
    /// #         })
    /// #     })
    /// # });
    /// // Create a request
    /// let mut args = HashMap::new();
    /// args.insert("message".to_string(), json!("Hello, world!"));
    ///
    /// let request = CallToolRequest {
    ///     name: "echo".to_string(),
    ///     arguments: Some(args),
    ///     meta: None,
    /// };
    ///
    /// // Call the tool
    /// let response = registry.call_tool(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_tool(&self, req: CallToolRequest) -> Result<CallToolResponse> {
        let handler = self
            .tool_handlers
            .get(&req.name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", req.name))?;

        (handler.f)(req).await
    }

    /// Get a list of all available tools
    ///
    /// # Returns
    /// A vector containing all registered tools
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::registry::Tools;
    /// # use mcp_daemon::types::{Tool, CallToolRequest, CallToolResponse, Content};
    /// # use serde_json::json;
    /// # fn example() {
    /// # let mut registry = Tools::new();
    /// # let tool1 = Tool {
    /// #     name: "tool1".to_string(),
    /// #     description: Some("Tool 1".to_string()),
    /// #     input_schema: json!({}),
    /// # };
    /// # let tool2 = Tool {
    /// #     name: "tool2".to_string(),
    /// #     description: Some("Tool 2".to_string()),
    /// #     input_schema: json!({}),
    /// # };
    /// # registry.register_tool(tool1, |request| {
    /// #     Box::pin(async move {
    /// #         Ok(CallToolResponse {
    /// #             content: vec![Content::Text { text: "".to_string() }],
    /// #             is_error: None,
    /// #             meta: None,
    /// #         })
    /// #     })
    /// # });
    /// # registry.register_tool(tool2, |request| {
    /// #     Box::pin(async move {
    /// #         Ok(CallToolResponse {
    /// #             content: vec![Content::Text { text: "".to_string() }],
    /// #             is_error: None,
    /// #             meta: None,
    /// #         })
    /// #     })
    /// # });
    /// // Get a list of all tools
    /// let tools = registry.list_tools();
    /// assert_eq!(tools.len(), 2);
    /// # }
    /// ```
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tool_handlers
            .values()
            .map(|tool_handler| tool_handler.tool.clone())
            .collect()
    }
}

impl Default for Tools {
    fn default() -> Self {
        Self::new()
    }
}

// Type aliases for complex future and handler types
/// Type alias for a future that returns a Result<CallToolResponse>
type ToolFuture = Pin<Box<dyn Future<Output = Result<CallToolResponse>> + Send>>;
/// Type alias for a function that takes a CallToolRequest and returns a ToolFuture
type ToolHandlerFn = Box<dyn Fn(CallToolRequest) -> ToolFuture + Send + Sync>;

/// Struct for storing tool handlers
///
/// This internal structure pairs a tool definition with its implementation function.
pub(crate) struct ToolHandler {
    /// The tool definition
    pub tool: Tool,
    /// The function that implements the tool's behavior
    pub f: ToolHandlerFn,
}
