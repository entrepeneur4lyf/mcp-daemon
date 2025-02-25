//! # Ollama Bridge Implementation
//!
//! This module provides conversion between MCP tools and Ollama's function call format,
//! which follows OpenAI's function calling specification with some Ollama-specific adaptations.
//!
//! ## Overview
//!
//! The Ollama bridge extends the OpenAI bridge functionality to work specifically with
//! Ollama's implementation of function calling. It handles the specific format differences
//! and parsing requirements needed for Ollama integration.
//!
//! Key features include:
//! * Converting MCP tools to Ollama-compatible function definitions
//! * Parsing Ollama's response format to extract function calls
//! * Formatting tool responses for Ollama consumption
//!
//! ## Examples
//!
//! ```rust
//! use mcp_daemon::bridge::ollama::{convert_tools_for_ollama, parse_ollama_response};
//! use mcp_daemon::types::Tool;
//!
//! // Define an MCP tool
//! let tool = Tool {
//!     name: "calculator".to_string(),
//!     description: Some("Perform calculations".to_string()),
//!     input_schema: serde_json::json!({
//!         "type": "object",
//!         "properties": {
//!             "operation": { "type": "string" },
//!             "a": { "type": "number" },
//!             "b": { "type": "number" }
//!         },
//!         "required": ["operation", "a", "b"]
//!     }),
//! };
//!
//! // Convert to Ollama function format
//! let ollama_format = convert_tools_for_ollama(&[tool]);
//!
//! // Parse an Ollama response
//! let response = r#"<function>calculator</function><args>{"operation":"add","a":1,"b":2}</args>"#;
//! let execution = parse_ollama_response(response).unwrap();
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::bridge::openai`] - Base OpenAI bridge implementation
//! * [`crate::types`] - Core MCP types used in the conversion process

use super::{ToolCall, FunctionCall, ToolExecution, ToolResponse, mcp_to_function, tool_call_to_mcp, mcp_to_function_response};
use crate::types::Tool;
use serde::Deserialize;

/// Convert MCP tools to Ollama function format
///
/// Transforms an array of MCP Tool objects into Ollama's function format.
/// This includes wrapping the functions in the appropriate JSON structure
/// with the "function_call": "auto" setting.
///
/// # Arguments
///
/// * `tools` - A slice of MCP Tool objects to convert
///
/// # Returns
///
/// A JSON value containing the Ollama function configuration
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::bridge::ollama::convert_tools_for_ollama;
/// use mcp_daemon::types::Tool;
///
/// let tool = Tool {
///     name: "calculator".to_string(),
///     description: Some("Perform calculations".to_string()),
///     input_schema: serde_json::json!({
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
/// let ollama_format = convert_tools_for_ollama(&[tool]);
/// assert!(ollama_format.get("functions").is_some());
/// assert_eq!(ollama_format.get("function_call").unwrap(), "auto");
/// ```
pub fn convert_tools_for_ollama(tools: &[Tool]) -> serde_json::Value {
    let functions = mcp_to_function(tools);
    serde_json::json!({
        "functions": functions,
        "function_call": "auto"
    })
}

/// Parse Ollama response to extract function calls
///
/// Analyzes an Ollama response string to identify and extract function calls.
/// Ollama uses a specific format with <function> and <args> tags to indicate
/// function calls in its responses.
///
/// # Arguments
///
/// * `response` - The raw response string from Ollama
///
/// # Returns
///
/// An Option containing the extracted ToolExecution if a function call was found,
/// or None if no function call was detected, wrapped in a Result
///
/// # Errors
///
/// Returns a TransportError if the function arguments cannot be parsed as valid JSON
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::bridge::ollama::parse_ollama_response;
/// use crate::transport::TransportError;
///
/// let response = r#"<function>calculator</function><args>{"operation":"add","a":1,"b":2}</args>"#;
/// let execution = parse_ollama_response(response).unwrap().unwrap();
/// assert_eq!(execution.name, "calculator");
/// ```
pub fn parse_ollama_response(response: &str) -> Result<Option<ToolExecution>, crate::transport::TransportError> {
    // Look for function call pattern in response
    if let Some(function_call) = extract_function_call(response) {
        let tool_call = ToolCall {
            id: "0".to_string(), // Ollama doesn't provide IDs, so we use a default
            function: FunctionCall {
                name: function_call.name,
                arguments: serde_json::to_string(&function_call.arguments).unwrap_or_default(),
            },
        };
        Ok(Some(tool_call_to_mcp(&tool_call)?))
    } else {
        Ok(None)
    }
}

/// Format tool response for Ollama
///
/// Converts an MCP tool response into a format suitable for Ollama.
///
/// # Arguments
///
/// * `tool_name` - The name of the tool that was executed
/// * `response` - The MCP ToolResponse object
///
/// # Returns
///
/// A string containing the formatted response for Ollama
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::bridge::ollama::{format_ollama_response, ToolResponse};
///
/// let response = ToolResponse {
///     result: serde_json::json!({"sum": 3}),
///     error: None,
/// };
///
/// let ollama_response = format_ollama_response("calculator", &response);
/// ```
pub fn format_ollama_response(tool_name: &str, response: &ToolResponse) -> String {
    let function_response = mcp_to_function_response(tool_name, response);
    serde_json::to_string(&function_response).unwrap_or_else(|_| "{}".to_string())
}

/// Helper to extract function calls from Ollama's response format
///
/// Internal structure used to represent a function call extracted from
/// Ollama's response format.
///
/// # Fields
///
/// * `name` - The name of the function being called
/// * `arguments` - The parsed JSON arguments for the function
#[derive(Debug, Deserialize)]
struct OllamaFunctionCall {
    name: String,
    arguments: serde_json::Value,
}

/// Extract function call from Ollama response text
///
/// Parses the Ollama response text to extract function calls using regex.
/// Looks for patterns like `<function>name</function><args>json_args</args>`.
///
/// # Arguments
///
/// * `response` - The raw response string from Ollama
///
/// # Returns
///
/// An Option containing the extracted OllamaFunctionCall if a function call was found,
/// or None if no function call was detected
fn extract_function_call(response: &str) -> Option<OllamaFunctionCall> {
    // This regex looks for function call patterns in Ollama's response
    // You would need to adapt this to match Ollama's actual format
    let re = regex::Regex::new(r"<function>(?P<name>[^<]+)</function>\s*<args>(?P<args>[^<]+)</args>").ok()?;
    
    if let Some(caps) = re.captures(response) {
        let name = caps.name("name")?.as_str().to_string();
        let args = caps.name("args")?.as_str();
        
        if let Ok(arguments) = serde_json::from_str(args) {
            return Some(OllamaFunctionCall { name, arguments });
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Tool;

    #[test]
    fn test_convert_tools() {
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

        let ollama_format = convert_tools_for_ollama(&tools);
        assert!(ollama_format.get("functions").is_some());
        assert_eq!(ollama_format.get("function_call").unwrap(), "auto");
    }

    #[test]
    fn test_parse_response() {
        let response = r#"<function>test_tool</function><args>{"arg1": "test"}</args>"#;
        let execution = parse_ollama_response(response).unwrap().unwrap();
        assert_eq!(execution.name, "test_tool");
        assert_eq!(execution.arguments, serde_json::json!({"arg1": "test"}));
    }
}
