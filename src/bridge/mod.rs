//! # Bridge Implementations for LLM Providers
//!
//! This module provides bridge implementations that connect the MCP protocol with
//! various LLM providers. These bridges enable MCP tools to be used with different
//! LLM APIs by converting between MCP's tool format and the provider-specific formats.
//!
//! ## Overview
//!
//! The bridge implementations handle:
//! - Converting MCP tools to provider-specific formats
//! - Parsing provider responses to extract tool calls
//! - Formatting tool responses for the provider
//!
//! ## Supported Providers
//!
//! * **OpenAI**: Converts between MCP tools and OpenAI's function calling format
//! * **Ollama**: Converts between MCP tools and Ollama's function calling format (which follows OpenAI's specification)
//!
//! ## Examples
//!
//! ### Using the OpenAI Bridge
//!
//! ```no_run
//! use mcp_daemon::bridge::openai::{mcp_to_function, mcp_to_function_response, ToolResponse};
//! use mcp_daemon::types::Tool;
//! use serde_json::json;
//!
//! // Define MCP tools
//! let tools = vec![
//!     Tool {
//!         name: "calculator".to_string(),
//!         description: Some("Performs basic arithmetic operations".to_string()),
//!         input_schema: json!({
//!             "type": "object",
//!             "properties": {
//!                 "operation": {
//!                     "type": "string",
//!                     "enum": ["add", "subtract", "multiply", "divide"]
//!                 },
//!                 "a": { "type": "number" },
//!                 "b": { "type": "number" }
//!             },
//!             "required": ["operation", "a", "b"]
//!         }),
//!     }
//! ];
//!
//! // Convert MCP tools to OpenAI function format
//! let openai_functions = mcp_to_function(&tools);
//!
//! // Use the converted functions with OpenAI API
//! // ...
//!
//! // Create a tool response
//! let tool_response = ToolResponse {
//!     result: json!({"result": 42}),
//!     error: None,
//! };
//!
//! // Convert to OpenAI function response format
//! let function_response = mcp_to_function_response("calculator", &tool_response);
//! assert_eq!(function_response.name, "calculator");
//! ```
//!
//! ### Using the Ollama Bridge
//!
//! ```no_run
//! use mcp_daemon::bridge::ollama::{convert_tools_for_ollama, parse_ollama_response};
//! use mcp_daemon::types::Tool;
//! use serde_json::json;
//!
//! // Define MCP tools
//! let tools = vec![
//!     Tool {
//!         name: "calculator".to_string(),
//!         description: Some("Performs basic arithmetic operations".to_string()),
//!         input_schema: json!({
//!             "type": "object",
//!             "properties": {
//!                 "operation": {
//!                     "type": "string",
//!                     "enum": ["add", "subtract", "multiply", "divide"]
//!                 },
//!                 "a": { "type": "number" },
//!                 "b": { "type": "number" }
//!             },
//!             "required": ["operation", "a", "b"]
//!         }),
//!     }
//! ];
//!
//! // Convert tools to Ollama format (which follows OpenAI's specification)
//! let ollama_functions = convert_tools_for_ollama(&tools);
//!
//! // Parse Ollama response
//! let response = r#"{"function": "calculator", "arguments": "{\"operation\":\"add\",\"a\":1,\"b\":2}"}"#;
//! let tool_execution = parse_ollama_response(response).unwrap().unwrap();
//! assert_eq!(tool_execution.name, "calculator");
//! ```

/// OpenAI bridge implementation
pub mod openai;
/// Ollama bridge implementation
pub mod ollama;

// Re-export common types and functions
pub use openai::{
    Function, FunctionDefinition, FunctionResponse, ToolCall, FunctionCall,
    ToolExecution, ToolResponse, mcp_to_function, tool_call_to_mcp, mcp_to_function_response
};
