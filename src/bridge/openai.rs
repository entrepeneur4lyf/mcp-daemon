//! # MCP to OpenAI Function Call Bridge
//!
//! This module provides conversion between MCP tools and OpenAI function calls.
//! It allows MCP tools to be used with any LLM that supports OpenAI's function calling format.
//!
//! ## Overview
//!
//! The OpenAI bridge enables interoperability between the Model Context Protocol (MCP) tool system
//! and OpenAI's function calling API. This allows MCP servers to expose their tools to LLMs that
//! implement the OpenAI function calling specification.
//!
//! The bridge provides bidirectional conversion:
//! * Converting MCP tools to OpenAI function definitions
//! * Converting OpenAI function calls to MCP tool executions
//! * Converting MCP tool responses to OpenAI function responses
//!
//! ## Examples
//!
//! ```no_run
//! use mcp_daemon::bridge::openai::{mcp_to_function, mcp_to_function_response, ToolResponse};
//! use mcp_daemon::types::Tool;
//! use serde_json::json;
//!
//! // Define an MCP tool
//! let tool = Tool {
//!     name: "weather".to_string(),
//!     description: Some("Get the current weather".to_string()),
//!     input_schema: json!({
//!         "type": "object",
//!         "properties": {
//!             "location": { "type": "string" }
//!         },
//!         "required": ["location"]
//!     }),
//! };
//!
//! // Convert to OpenAI function format
//! let functions = mcp_to_function(&[tool]);
//!
//! // Use with an OpenAI-compatible API
//! // ...
//!
//! // Convert tool response back to OpenAI format
//! let tool_response = ToolResponse {
//!     result: json!({"temperature": 72, "conditions": "sunny"}),
//!     error: None,
//! };
//! let function_response = mcp_to_function_response("weather", &tool_response);
//! assert_eq!(function_response.name, "weather");
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::bridge::ollama`] - Ollama-specific bridge implementation
//! * [`crate::types`] - Core MCP types used in the conversion process

use crate::transport::TransportError;
use crate::types::Tool;
use serde::{Deserialize, Serialize};

/// OpenAI function definition format
///
/// Represents a function definition in the OpenAI function calling API.
///
/// # Fields
///
/// * `name` - The name of the function
/// * `description` - Optional description of the function's purpose
/// * `parameters` - JSON Schema object defining the function's parameters
/// * `strict` - Whether the function should strictly validate parameters (defaults to true)
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function
    pub name: String,
    /// Optional description of the function's purpose
    pub description: Option<String>,
    /// JSON Schema object defining the function's parameters
    pub parameters: serde_json::Value,
    /// Whether the function should strictly validate parameters (defaults to true)
    #[serde(default = "default_strict")]
    pub strict: bool,
}

/// OpenAI function format
///
/// Represents a complete function object in the OpenAI API.
///
/// # Fields
///
/// * `function_type` - The type of the function (always "function")
/// * `function` - The function definition
#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    /// The type of the function (always "function")
    #[serde(rename = "type")]
    pub function_type: String,
    /// The function definition
    pub function: FunctionDefinition,
}

fn default_strict() -> bool {
    true
}

/// OpenAI function response format
///
/// Represents a response from a function call in the OpenAI API.
///
/// # Fields
///
/// * `name` - The name of the function that was called
/// * `content` - The string content of the function's response
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionResponse {
    /// The name of the function that was called
    pub name: String,
    /// The string content of the function's response
    pub content: String,
}

/// Convert MCP tools to OpenAI function format
///
/// Transforms an array of MCP Tool objects into OpenAI Function objects.
///
/// # Arguments
///
/// * `tools` - A slice of MCP Tool objects to convert
///
/// # Returns
///
/// A vector of OpenAI Function objects
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::bridge::openai::mcp_to_function;
/// use mcp_daemon::types::Tool;
/// use serde_json::json;
///
/// let tool = Tool {
///     name: "calculator".to_string(),
///     description: Some("Perform calculations".to_string()),
///     input_schema: json!({
///         "type": "object",
///         "properties": {
///             "operation": { "type": "string" },
///             "a": { "type": "number" },
///             "b": { "type": "number" }
///         },
///         "required": ["operation", "a", "b"]
///     }),
/// };
///
/// let functions = mcp_to_function(&[tool]);
/// assert_eq!(functions.len(), 1);
/// assert_eq!(functions[0].function.name, "calculator");
/// ```
pub fn mcp_to_function(tools: &[Tool]) -> Vec<Function> {
    tools.iter().map(|tool| {
        // Get the base schema
        let mut parameters = tool.input_schema.clone();
        
        // Add additionalProperties: false if it's not already set
        if let Some(obj) = parameters.as_object_mut() {
            if !obj.contains_key("additionalProperties") {
                obj.insert("additionalProperties".to_string(), serde_json::Value::Bool(false));
            }
        }

        Function {
            function_type: "function".to_string(),
            function: FunctionDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters,
                strict: true,
            },
        }
    }).collect()
}

/// Convert OpenAI function to MCP tool execution
///
/// Transforms an OpenAI Function object into an MCP ToolExecution object.
///
/// # Arguments
///
/// * `function` - The OpenAI Function object to convert
///
/// # Returns
///
/// An MCP ToolExecution object wrapped in a Result
///
/// # Errors
///
/// Returns a TransportError if the function parameters cannot be processed
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::bridge::openai::{function_to_mcp, Function, FunctionDefinition};
/// use serde_json::json;
///
/// let function = Function {
///     function_type: "function".to_string(),
///     function: FunctionDefinition {
///         name: "calculator".to_string(),
///         description: Some("Perform calculations".to_string()),
///         parameters: json!({
///             "arg1": "test"
///         }),
///         strict: true,
///     },
/// };
///
/// let execution = function_to_mcp(&function).unwrap();
/// assert_eq!(execution.name, "calculator");
/// ```
pub fn function_to_mcp(function: &Function) -> Result<ToolExecution, TransportError> {
    Ok(ToolExecution {
        name: function.function.name.clone(),
        arguments: function.function.parameters.clone(),
    })
}

/// OpenAI tool call format
///
/// Represents a tool call in the OpenAI API response.
///
/// # Fields
///
/// * `id` - The unique identifier for this tool call
/// * `function` - The function call details
#[derive(Debug, Deserialize)]
pub struct ToolCall {
    /// The unique identifier for this tool call
    pub id: String,
    /// The function call details
    pub function: FunctionCall,
}

/// OpenAI function call format in responses
///
/// Represents a function call in the OpenAI API response.
///
/// # Fields
///
/// * `name` - The name of the function being called
/// * `arguments` - A JSON string containing the function arguments
#[derive(Debug, Deserialize)]
pub struct FunctionCall {
    /// The name of the function being called
    pub name: String,
    /// A JSON string containing the function arguments
    pub arguments: String,
}

/// Convert OpenAI tool call to MCP tool execution
///
/// Transforms an OpenAI ToolCall object into an MCP ToolExecution object.
///
/// # Arguments
///
/// * `tool_call` - The OpenAI ToolCall object to convert
///
/// # Returns
///
/// An MCP ToolExecution object wrapped in a Result
///
/// # Errors
///
/// Returns a TransportError if the function arguments cannot be parsed as valid JSON
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::bridge::openai::{tool_call_to_mcp, ToolCall, FunctionCall};
///
/// let tool_call = ToolCall {
///     id: "call_123".to_string(),
///     function: FunctionCall {
///         name: "calculator".to_string(),
///         arguments: r#"{"operation":"add","a":1,"b":2}"#.to_string(),
///     },
/// };
///
/// let execution = tool_call_to_mcp(&tool_call).unwrap();
/// assert_eq!(execution.name, "calculator");
/// ```
pub fn tool_call_to_mcp(tool_call: &ToolCall) -> Result<ToolExecution, TransportError> {
    // Parse the arguments string as JSON
    let arguments = serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
        TransportError::new(
            crate::transport::TransportErrorCode::InvalidMessage,
            format!("Failed to parse function arguments: {}", e)
        )
    })?;

    Ok(ToolExecution {
        name: tool_call.function.name.clone(),
        arguments,
    })
}

/// Convert MCP tool response to OpenAI function response
///
/// Transforms an MCP ToolResponse into an OpenAI FunctionResponse.
///
/// # Arguments
///
/// * `tool_name` - The name of the tool that was executed
/// * `response` - The MCP ToolResponse object
///
/// # Returns
///
/// An OpenAI FunctionResponse object
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::bridge::openai::{mcp_to_function_response, ToolResponse};
/// use serde_json::json;
///
/// let response = ToolResponse {
///     result: json!({"sum": 3}),
///     error: None,
/// };
///
/// let function_response = mcp_to_function_response("calculator", &response);
/// assert_eq!(function_response.name, "calculator");
/// assert_eq!(function_response.content, r#"{"sum":3}"#);
/// ```
pub fn mcp_to_function_response(tool_name: &str, response: &ToolResponse) -> FunctionResponse {
    FunctionResponse {
        name: tool_name.to_string(),
        content: if let Some(error) = &response.error {
            format!("Error: {}", error)
        } else {
            serde_json::to_string(&response.result).unwrap_or_else(|_| "{}".to_string())
        },
    }
}

/// MCP message format for tool execution
///
/// Represents a tool execution request in the MCP protocol.
///
/// # Fields
///
/// * `name` - The name of the tool to execute
/// * `arguments` - The arguments to pass to the tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolExecution {
    /// The name of the tool to execute
    pub name: String,
    /// The arguments to pass to the tool
    pub arguments: serde_json::Value,
}

/// MCP message format for tool response  
///
/// Represents a tool execution response in the MCP protocol.
///
/// # Fields
///
/// * `result` - The result of the tool execution
/// * `error` - Optional error message if the tool execution failed
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    /// The result of the tool execution
    pub result: serde_json::Value,
    /// Optional error message if the tool execution failed
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_to_function() {
        let tools = vec![Tool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "arg1": { "type": "string" }
                },
                "required": ["arg1"]
            }),
        }];

        let functions = mcp_to_function(&tools);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].function_type, "function");
        assert_eq!(functions[0].function.name, "test_tool");
        assert_eq!(functions[0].function.description, Some("A test tool".to_string()));
        assert!(functions[0].function.strict);
    }

    #[test]
    fn test_function_to_mcp() {
        let function = Function {
            function_type: "function".to_string(),
            function: FunctionDefinition {
                name: "test_tool".to_string(),
                description: Some("A test tool".to_string()),
                parameters: serde_json::json!({
                    "arg1": "test"
                }),
                strict: true,
            },
        };

        let execution = super::function_to_mcp(&function).unwrap();
        assert_eq!(execution.name, "test_tool");
        assert_eq!(execution.arguments, serde_json::json!({"arg1": "test"}));
    }

    #[test]
    fn test_mcp_to_function_response() {
        let response = ToolResponse {
            result: serde_json::json!({"output": "test"}),
            error: None,
        };

        let function_response = mcp_to_function_response("test_tool", &response);
        assert_eq!(function_response.name, "test_tool");
        assert_eq!(function_response.content, r#"{"output":"test"}"#);

        let error_response = ToolResponse {
            result: serde_json::Value::Null,
            error: Some("Test error".to_string()),
        };

        let function_response = mcp_to_function_response("test_tool", &error_response);
        assert_eq!(function_response.content, "Error: Test error");
    }
}
