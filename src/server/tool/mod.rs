//! Tool system for exposing executable functionality to clients
//!
//! This module provides the infrastructure for defining, registering, and executing
//! tools in the Model Context Protocol. Tools are a core primitive that allow servers
//! to expose executable functionality to clients, enabling LLMs to perform actions
//! in the real world through a standardized interface.
//!
//! The tool system is designed to be:
//! - Type-safe: Tools can be defined with strongly-typed arguments
//! - Flexible: Tools can be implemented with various callback patterns
//! - Asynchronous: Tool execution is non-blocking
//! - Discoverable: Tools provide metadata for client discovery
//!
//! # Architecture
//!
//! The tool system consists of several key components:
//!
//! - `Tool`: Metadata describing a tool (name, description, input schema)
//! - `RegisteredTool`: A tool with its execution callback
//! - `ToolCallback`: Trait for tool execution callbacks
//! - `ToolBuilder`: Builder pattern for creating tools
//!
//! # Examples
//!
//! ## Creating a simple tool
//!
//! ```
//! use mcp_daemon::server::tool::ToolBuilder;
//! use mcp_daemon::types::CallToolResponse;
//!
//! // Create a simple calculator tool
//! let (metadata, registered_tool) = ToolBuilder::new("calculator")
//!     .description("Performs basic calculations")
//!     .input_schema(serde_json::json!({
//!         "type": "object",
//!         "properties": {
//!             "operation": { "type": "string" },
//!             "a": { "type": "number" },
//!             "b": { "type": "number" }
//!         },
//!         "required": ["operation", "a", "b"]
//!     }))
//!     .build(|args| async move {
//!         // Parse arguments
//!         let args = args.unwrap_or(serde_json::json!({}));
//!         let operation = args.get("operation").and_then(|v| v.as_str()).unwrap_or("add");
//!         let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
//!         let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
//!         
//!         // Perform calculation
//!         let result = match operation {
//!             "add" => a + b,
//!             "subtract" => a - b,
//!             "multiply" => a * b,
//!             "divide" => {
//!                 if b == 0.0 {
//!                     return CallToolResponse::error("Division by zero");
//!                 }
//!                 a / b
//!             },
//!             _ => return CallToolResponse::error("Unknown operation")
//!         };
//!         
//!         // Return result
//!         CallToolResponse::success(format!("Result: {}", result))
//!     });
//! ```
//!
//! ## Creating a tool with typed arguments
//!
//! ```
//! use mcp_daemon::server::tool::ToolBuilder;
//! use serde::Deserialize;
//!
//! // Define typed arguments
//! #[derive(Deserialize)]
//! struct CalculatorArgs {
//!     operation: String,
//!     a: f64,
//!     b: f64,
//! }
//!
//! // Create a calculator tool with typed arguments
//! let (metadata, registered_tool) = ToolBuilder::new("calculator")
//!     .description("Performs basic calculations")
//!     .input_schema(serde_json::json!({
//!         "type": "object",
//!         "properties": {
//!             "operation": { "type": "string" },
//!             "a": { "type": "number" },
//!             "b": { "type": "number" }
//!         },
//!         "required": ["operation", "a", "b"]
//!     }))
//!     .build_typed(|args: CalculatorArgs| {
//!         Box::pin(async move {
//!             // Perform calculation
//!             let result = match args.operation.as_str() {
//!                 "add" => args.a + args.b,
//!                 "subtract" => args.a - args.b,
//!                 "multiply" => args.a * args.b,
//!                 "divide" => {
//!                     if args.b == 0.0 {
//!                         return CallToolResponse::error("Division by zero");
//!                     }
//!                     args.a / args.b
//!                 },
//!                 _ => return CallToolResponse::error("Unknown operation")
//!             };
//!             
//!             // Return result
//!             CallToolResponse::success(format!("Result: {}", result))
//!         })
//!     });
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::types::{CallToolResponse, Tool, Content};

/// A registered tool with metadata and callbacks
///
/// This struct represents a tool that has been registered with the MCP server.
/// It contains both the tool's metadata (name, description, schema) and the
/// callback function that will be executed when the tool is called.
///
/// # Examples
///
/// ```
/// use mcp_daemon::server::tool::{ToolBuilder, RegisteredTool};
/// use mcp_daemon::types::{CallToolResponse, Content};
/// use std::future::Future;
/// use std::pin::Pin;
///
/// // Create a tool using the builder pattern
/// let (metadata, registered_tool) = ToolBuilder::new("calculator")
///     .description("Performs basic calculations")
///     .input_schema(serde_json::json!({
///         "type": "object",
///         "properties": {
///             "operation": { "type": "string" },
///             "a": { "type": "number" },
///             "b": { "type": "number" }
///         },
///         "required": ["operation", "a", "b"]
///     }))
///     .build(|args| async move {
///         // Tool implementation
///         CallToolResponse {
///             content: vec![Content::Text { text: "Result: 42".to_string() }],
///             is_error: None,
///             meta: None,
///         }
///     });
/// ```
pub struct RegisteredTool {
    /// The tool metadata
    pub metadata: Tool,
    /// The callback to execute the tool
    pub execute_callback: Arc<dyn ToolCallback>,
}

/// A callback that can execute a tool
///
/// This trait defines the interface for tool execution callbacks.
/// Implementations of this trait can be registered with the MCP server
/// to handle tool execution requests.
///
/// The `call` method takes optional JSON arguments and returns a future
/// that resolves to a `CallToolResponse`.
pub trait ToolCallback: Send + Sync {
    /// Calls the tool with optional arguments
    ///
    /// # Arguments
    ///
    /// * `args` - Optional JSON value containing the tool arguments
    ///
    /// # Returns
    ///
    /// A future that resolves to a `CallToolResponse` containing the tool's result
    fn call(&self, args: Option<Value>) -> ToolFuture;
}

// Type aliases for complex future and callback types
/// Type alias for the future returned by tool callbacks
type ToolFuture = Pin<Box<dyn Future<Output = CallToolResponse> + Send>>;
/// Type alias for tool callback functions
type ToolCallbackFunc = Box<dyn Fn(Option<Value>) -> ToolFuture + Send + Sync>;

/// Internal implementation of `ToolCallback` for function-based callbacks
struct ToolCallbackFn(ToolCallbackFunc);

impl ToolCallback for ToolCallbackFn {
    fn call(&self, args: Option<Value>) -> ToolFuture {
        (self.0)(args)
    }
}

/// Builder for creating tools with typed arguments
///
/// This builder provides a fluent API for creating tools with
/// strongly-typed arguments and execution callbacks.
///
/// # Examples
///
/// ```
/// use mcp_daemon::server::tool::ToolBuilder;
/// use mcp_daemon::types::{CallToolResponse, Content};
/// use serde::{Serialize, Deserialize};
/// use std::future::Future;
/// use std::pin::Pin;
///
/// #[derive(Deserialize)]
/// struct CalculatorArgs {
///     operation: String,
///     a: f64,
///     b: f64,
/// }
///
/// let (metadata, registered_tool) = ToolBuilder::new("calculator")
///     .description("Performs basic calculations")
///     .input_schema(serde_json::json!({
///         "type": "object",
///         "properties": {
///             "operation": { "type": "string" },
///             "a": { "type": "number" },
///             "b": { "type": "number" }
///         },
///         "required": ["operation", "a", "b"]
///     }))
///     .build_typed(|args: CalculatorArgs| {
///         Box::pin(async move {
///             let result = match args.operation.as_str() {
///                 "add" => args.a + args.b,
///                 "subtract" => args.a - args.b,
///                 "multiply" => args.a * args.b,
///                 "divide" => args.a / args.b,
///                 _ => return CallToolResponse {
///                     content: vec![Content::Text { text: "Unknown operation".to_string() }],
///                     is_error: Some(true),
///                     meta: None,
///                 }
///             };
///             CallToolResponse {
///                 content: vec![Content::Text { text: format!("Result: {}", result) }],
///                 is_error: None,
///                 meta: None,
///             }
///         })
///     });
/// ```
pub struct ToolBuilder {
    name: String,
    description: Option<String>,
    input_schema: Option<Value>,
}

impl ToolBuilder {
    /// Create a new tool builder with the given name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool
    ///
    /// # Returns
    ///
    /// A new `ToolBuilder` instance
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: None,
        }
    }

    /// Add a description to the tool
    ///
    /// # Arguments
    ///
    /// * `description` - A human-readable description of the tool's purpose and usage
    ///
    /// # Returns
    ///
    /// The builder for method chaining
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add an input schema to the tool
    ///
    /// The input schema defines the expected structure of the tool's arguments
    /// using JSON Schema format.
    ///
    /// # Arguments
    ///
    /// * `schema` - A JSON Schema defining the tool's input parameters
    ///
    /// # Returns
    ///
    /// The builder for method chaining
    pub fn input_schema(mut self, schema: Value) -> Self {
        self.input_schema = Some(schema);
        self
    }

    /// Creates a standardized error response for invalid arguments
    ///
    /// # Arguments
    ///
    /// * `error` - The error message or object
    ///
    /// # Returns
    ///
    /// A `CallToolResponse` with the error message and `is_error` flag set
    #[allow(dead_code)]
    fn error_response(error: impl ToString) -> CallToolResponse {
        CallToolResponse {
            content: vec![Content::Text {
                text: format!("Invalid arguments: {}", error.to_string()),
            }],
            is_error: Some(true),
            meta: None,
        }
    }

    /// Build the tool with the given execution callback
    ///
    /// This method finalizes the tool configuration and creates both the
    /// tool metadata and the registered tool with its execution callback.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that takes optional JSON arguments and returns a future
    ///   that resolves to a `CallToolResponse`
    ///
    /// # Returns
    ///
    /// A tuple containing the tool metadata and the registered tool
    pub fn build<F, Fut>(self, callback: F) -> (Tool, RegisteredTool)
    where
        F: Fn(Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = CallToolResponse> + Send + 'static,
    {
        let metadata = Tool {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.unwrap_or_else(|| {
                serde_json::json!({
                    "type": "object"
                })
            }),
        };

        let registered = RegisteredTool {
            metadata: metadata.clone(),
            execute_callback: Arc::new(ToolCallbackFn(Box::new(move |args| {
                Box::pin(callback(args))
            }))),
        };

        (metadata, registered)
    }

    /// Build the tool with a typed execution callback
    ///
    /// This method allows creating tools with strongly-typed arguments.
    /// The provided callback will receive arguments deserialized into the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize arguments into
    /// * `F` - The callback function type
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that takes a value of type `T` and returns a future
    ///   that resolves to a `CallToolResponse`
    ///
    /// # Returns
    ///
    /// A tuple containing the tool metadata and the registered tool
    ///
    /// # Examples
    ///
/// ```
/// use mcp_daemon::server::tool::ToolBuilder;
/// use mcp_daemon::types::{CallToolResponse, Content};
/// use serde::Deserialize;
/// use std::future::Future;
/// use std::pin::Pin;
///
/// #[derive(Deserialize)]
/// struct GreetingArgs {
///     name: String,
///     formal: Option<bool>,
/// }
///
/// let (metadata, registered_tool) = ToolBuilder::new("greet")
///     .build_typed(|args: GreetingArgs| {
///         Box::pin(async move {
///             let prefix = if args.formal.unwrap_or(false) {
///                 "Hello, "
///             } else {
///                 "Hi, "
///             };
///             CallToolResponse {
///                 content: vec![Content::Text { text: format!("{}{}", prefix, args.name) }],
///                 is_error: None,
///                 meta: None,
///             }
///         })
///     });
/// ```
    #[allow(dead_code)]
    pub(crate) fn build_typed<T, F>(self, callback: F) -> (Tool, RegisteredTool)
    where
        T: for<'de> Deserialize<'de> + Send + 'static, 
        F: Fn(T) -> Pin<Box<dyn Future<Output = CallToolResponse> + Send>> + Send + Sync + 'static, 
    {
        let callback = Arc::new(callback);
        self.build(move |args| {
            let callback = Arc::clone(&callback);
            Box::pin(async move {
                let args_result: Result<T, _> = match args {
                    Some(args) => {
                        serde_json::from_value(args)
                            .map_err(Self::error_response)
                    },
                    None => {
                        serde_json::from_value(serde_json::json!({}))
                            .map_err(Self::error_response)
                    }
                };
                match args_result {
                    Ok(args) => callback(args).await,
                    Err(error_response) => error_response,
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestArgs {
        message: String,
    }

    #[tokio::test]
    async fn test_tool_builder() {
        let (metadata, registered) = ToolBuilder::new("test")
            .description("A test tool")
            .input_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string"
                    }
                }
            }))
            .build_typed(|args: TestArgs| {
                Box::pin(async move {
                    CallToolResponse {
                        content: vec![Content::Text {
                            text: args.message,
                        }],
                        is_error: None,
                        meta: None,
                    }
                })
            });

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, Some("A test tool".to_string()));

        let result = registered
            .execute_callback
            .call(Some(serde_json::json!({
                "message": "Hello"
            })))
            .await;

        if let Content::Text { text } = &result.content[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected text response");
        }
    }
}
