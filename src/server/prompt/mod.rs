use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::completable::Completable;
use crate::types::{Prompt, PromptArgument, MessageContent};

/// A registered prompt with metadata and callbacks
pub struct RegisteredPrompt {
    /// The prompt metadata
    pub metadata: Prompt,
    /// Optional argument completions
    pub argument_completions: HashMap<String, Arc<dyn Completable<Input = str, Output = String>>>,
    /// The callback to execute the prompt
    pub execute_callback: Arc<dyn PromptCallback>,
}

impl std::fmt::Debug for RegisteredPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredPrompt")
            .field("metadata", &self.metadata)
            .field("argument_completions", &"<HashMap>")
            .field("execute_callback", &"<PromptCallback>")
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
/// Result returned when getting a prompt's details
pub struct GetPromptResult {
    /// Optional description of the prompt
    pub description: Option<String>,
    /// List of messages that make up the prompt
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone)]
/// A message within a prompt
pub struct PromptMessage {
    /// The role of the message sender (e.g., 'user', 'assistant')
    pub role: String,
    /// The content of the message
    pub content: MessageContent,
}

/// A callback that can execute a prompt
pub trait PromptCallback: Send + Sync {
    /// Calls the prompt with optional arguments
    fn call(&self, args: Option<HashMap<String, String>>) -> PromptFuture;
}

impl std::fmt::Debug for dyn PromptCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PromptCallback")
    }
}

// Type aliases for complex future and callback types
type PromptFuture = Pin<Box<dyn Future<Output = GetPromptResult> + Send>>;
type PromptCallbackFunc = Box<dyn Fn(Option<HashMap<String, String>>) -> PromptFuture + Send + Sync>;

struct PromptCallbackFn(PromptCallbackFunc);

impl PromptCallback for PromptCallbackFn {
    fn call(&self, args: Option<HashMap<String, String>>) -> PromptFuture {
        (self.0)(args)
    }
}

/// Builder for creating prompts with arguments and completions
pub struct PromptBuilder {
    name: String,
    description: Option<String>,
    arguments: Vec<PromptArgument>,
    argument_completions: HashMap<String, Arc<dyn Completable<Input = str, Output = String>>>,
}

impl PromptBuilder {
    /// Create a new prompt builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: Vec::new(),
            argument_completions: HashMap::new(),
        }
    }

    /// Add a description to the prompt
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a required argument to the prompt
    pub fn required_arg(
        mut self,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            description: description.map(|d| d.into()),
            required: Some(true),
        });
        self
    }

    /// Add an optional argument to the prompt
    pub fn optional_arg<S: Into<String>>(
        mut self,
        name: S,
        description: Option<S>,
    ) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            description: description.map(Into::into),
            required: Some(false),
        });
        self
    }

    /// Add a completion callback for an argument
    pub fn with_completion(
        mut self,
        arg_name: impl Into<String>,
        completable: impl Completable<Input = str, Output = String> + 'static,
    ) -> Self {
        self.argument_completions
            .insert(arg_name.into(), Arc::new(completable));
        self
    }

    /// Build the prompt with the given execution callback
    pub fn build<F, Fut>(
        self,
        callback: F,
    ) -> Result<(Prompt, RegisteredPrompt), String>
    where
        F: Fn(Option<HashMap<String, String>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GetPromptResult> + Send + 'static,
    {
        // Validate arguments
        for arg in &self.arguments {
            if let Some(required) = arg.required {
                if required && arg.name.is_empty() {
                    return Err("Required argument must have a name".to_string());
                }
            } else {
                return Err(format!("Argument '{}' must specify if it's required", arg.name));
            }
        }

        let metadata = Prompt {
            name: self.name.clone(),
            description: self.description.clone(),
            arguments: if self.arguments.is_empty() {
                None
            } else {
                Some(self.arguments.clone())
            },
        };

        let registered = RegisteredPrompt {
            metadata: metadata.clone(),
            argument_completions: self.argument_completions,
            execute_callback: Arc::new(PromptCallbackFn(Box::new(move |args| {
                Box::pin(callback(args))
            }))),
        };

        Ok((metadata, registered))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completable::CompletableString;

    #[tokio::test]
    async fn test_prompt_builder() {
        let (metadata, registered) = PromptBuilder::new("test")
            .description("A test prompt")
            .required_arg("arg1", Some("First argument"))
            .optional_arg("arg2".to_string(), None)
            .with_completion(
                "arg1",
                CompletableString::new(|input: &str| {
                    let input = input.to_string();
                    async move { vec![format!("{}_completed", input)] }
                }),
            )
            .build(|_args| async move {
                GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: "assistant".to_string(),
                        content: MessageContent::Text {
                            text: "Test response".to_string(),
                        },
                    }],
                }
            })
            .expect("Failed to build prompt");

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, Some("A test prompt".to_string()));
        assert_eq!(metadata.arguments.as_ref().unwrap().len(), 2);

        assert!(registered.argument_completions.contains_key("arg1"));
        assert!(!registered.argument_completions.contains_key("arg2"));

        let result = registered
            .execute_callback
            .call(Some(HashMap::new()))
            .await;
        match &result.messages[0].content {
            MessageContent::Text { text } => assert_eq!(text, "Test response"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_prompt_builder_invalid_args() {
        let result = PromptBuilder::new("test")
            .required_arg("", Some("Invalid required arg"))
            .build(|_args| async move { GetPromptResult::default() });

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Required argument must have a name");

        let result = PromptBuilder::new("test")
            .optional_arg("arg", None)
            .build(|_args| async move { GetPromptResult::default() });

        assert!(result.is_ok());
    }
}
