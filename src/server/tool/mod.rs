use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::types::{CallToolResponse, Tool, Content};

/// A registered tool with metadata and callbacks
pub struct RegisteredTool {
    /// The tool metadata
    pub metadata: Tool,
    /// The callback to execute the tool
    pub execute_callback: Arc<dyn ToolCallback>,
}

/// A callback that can execute a tool
pub trait ToolCallback: Send + Sync {
    /// Calls the tool with optional arguments
    fn call(&self, args: Option<Value>) -> ToolFuture;
}

// Type aliases for complex future and callback types
type ToolFuture = Pin<Box<dyn Future<Output = CallToolResponse> + Send>>;
type ToolCallbackFunc = Box<dyn Fn(Option<Value>) -> ToolFuture + Send + Sync>;

struct ToolCallbackFn(ToolCallbackFunc);

impl ToolCallback for ToolCallbackFn {
    fn call(&self, args: Option<Value>) -> ToolFuture {
        (self.0)(args)
    }
}

/// Builder for creating tools with typed arguments
pub struct ToolBuilder {
    name: String,
    description: Option<String>,
    input_schema: Option<Value>,
}

impl ToolBuilder {
    /// Create a new tool builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: None,
        }
    }

    /// Add a description to the tool
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add an input schema to the tool
    pub fn input_schema(mut self, schema: Value) -> Self {
        self.input_schema = Some(schema);
        self
    }

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
