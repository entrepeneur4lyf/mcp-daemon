use crate::types::{CallToolRequest, CallToolResponse, Tool};
use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

// Type alias for tool handler map
type ToolHandlerMap = HashMap<String, ToolHandler>;

/// A registry of tools that can be called by the server
pub struct Tools {
    tool_handlers: ToolHandlerMap,
}

impl Tools {

    /// Get a tool by its name
    ///
    /// # Arguments
    /// * `name` - The name of the tool to retrieve
    ///
    /// # Returns
    /// The tool if it exists, None otherwise
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
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tool_handlers
            .values()
            .map(|tool_handler| tool_handler.tool.clone())
            .collect()
    }
}

// Type aliases for complex future and handler types
type ToolFuture = Pin<Box<dyn Future<Output = Result<CallToolResponse>> + Send>>;
type ToolHandlerFn = Box<dyn Fn(CallToolRequest) -> ToolFuture + Send + Sync>;

// Struct for storing tool handlers
pub(crate) struct ToolHandler {
    pub tool: Tool,
    pub f: ToolHandlerFn,
}
