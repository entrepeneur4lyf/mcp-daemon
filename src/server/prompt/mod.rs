//! # MCP Prompt System
//!
//! This module provides types and utilities for implementing MCP's prompt system,
//! which allows servers to define and execute prompt templates with dynamic arguments.
//!
//! ## Overview
//!
//! The prompt system enables servers to define reusable prompt templates that can:
//!
//! * Accept dynamic arguments from clients
//! * Provide auto-completion for argument values
//! * Generate structured messages for LLM interactions
//! * Support both required and optional arguments
//! * Include rich content types (text, images, etc.)
//!
//! ## Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use mcp_daemon::server::prompt::{PromptBuilder, GetPromptResult, PromptMessage};
//! use mcp_daemon::types::MessageContent;
//! use mcp_daemon::completable::CompletableString;
//!
//! // Create a simple greeting prompt with a required 'name' argument
//! let (metadata, registered_prompt) = PromptBuilder::new("greeting")
//!     .description("A friendly greeting prompt")
//!     .required_arg("name", Some("Person's name"))
//!     .with_completion(
//!         "name",
//!         CompletableString::new(|input: &str| {
//!             // Simple completion that suggests common names
//!             let input = input.to_string();
//!             async move {
//!                 vec!["Alice", "Bob", "Charlie"]
//!                     .into_iter()
//!                     .filter(|name| name.to_lowercase().starts_with(&input.to_lowercase()))
//!                     .map(|name| name.to_string())
//!                     .collect()
//!             }
//!         }),
//!     )
//!     .build(|args| async move {
//!         // Extract the name from arguments or use a default
//!         let name = args
//!             .and_then(|args| args.get("name").cloned())
//!             .unwrap_or_else(|| "friend".to_string());
//!
//!         // Return a prompt result with a greeting message
//!         GetPromptResult {
//!             description: Some(format!("Greeting for {}", name)),
//!             messages: vec![
//!                 PromptMessage {
//!                     role: "user".to_string(),
//!                     content: MessageContent::Text {
//!                         text: format!("Hello, {}! How are you today?", name),
//!                     },
//!                 },
//!             ],
//!         }
//!     })
//!     .expect("Failed to build prompt");
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::completable`] - Completion system for arguments
//! * [`crate::types`] - Core MCP types including Prompt and MessageContent

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::completable::Completable;
use crate::types::{Prompt, PromptArgument, MessageContent};

/// A registered prompt with metadata and callbacks
///
/// Represents a prompt that has been registered with the server,
/// including its metadata, argument completions, and execution callback.
///
/// # Fields
///
/// * `metadata` - The prompt metadata (name, description, arguments)
/// * `argument_completions` - Optional completions for prompt arguments
/// * `execute_callback` - The callback to execute the prompt
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use mcp_daemon::server::prompt::{PromptBuilder, RegisteredPrompt};
/// use mcp_daemon::completable::CompletableString;
///
/// // Create a registered prompt using PromptBuilder
/// let (_metadata, registered_prompt) = PromptBuilder::new("example")
///     .description("Example prompt")
///     .required_arg("arg1", Some("First argument"))
///     .with_completion(
///         "arg1",
///         CompletableString::new(|input: &str| {
///             let input = input.to_string();
///             async move { vec![format!("{}_suggestion", input)] }
///         }),
///     )
///     .build(|_args| async {
///         // Prompt execution logic
///         Default::default()
///     })
///     .expect("Failed to build prompt");
/// ```
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

/// Result returned when getting a prompt's details
///
/// Contains the generated prompt messages and optional description.
///
/// # Fields
///
/// * `description` - Optional description of the prompt
/// * `messages` - List of messages that make up the prompt
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::prompt::{GetPromptResult, PromptMessage};
/// use mcp_daemon::types::MessageContent;
///
/// let result = GetPromptResult {
///     description: Some("A greeting prompt".to_string()),
///     messages: vec![
///         PromptMessage {
///             role: "user".to_string(),
///             content: MessageContent::Text {
///                 text: "Hello, world!".to_string(),
///             },
///         },
///     ],
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetPromptResult {
    /// Optional description of the prompt
    pub description: Option<String>,
    /// List of messages that make up the prompt
    pub messages: Vec<PromptMessage>,
}

/// A message within a prompt
///
/// Represents a single message in a prompt, with a role and content.
///
/// # Fields
///
/// * `role` - The role of the message sender (e.g., 'user', 'assistant')
/// * `content` - The content of the message
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::prompt::PromptMessage;
/// use mcp_daemon::types::MessageContent;
///
/// let message = PromptMessage {
///     role: "user".to_string(),
///     content: MessageContent::Text {
///         text: "What is the capital of France?".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PromptMessage {
    /// The role of the message sender (e.g., 'user', 'assistant')
    pub role: String,
    /// The content of the message
    pub content: MessageContent,
}

/// A callback that can execute a prompt
///
/// This trait defines the interface for prompt execution callbacks.
/// Implementors must provide a `call` method that generates prompt
/// messages based on the provided arguments.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use std::pin::Pin;
/// use std::future::Future;
/// use mcp_daemon::server::prompt::{PromptCallback, GetPromptResult, PromptMessage};
/// use mcp_daemon::types::MessageContent;
///
/// struct MyPromptCallback;
///
/// impl PromptCallback for MyPromptCallback {
///     fn call(&self, args: Option<HashMap<String, String>>) -> Pin<Box<dyn Future<Output = GetPromptResult> + Send>> {
///         Box::pin(async move {
///             let name = args.and_then(|args| args.get("name").cloned()).unwrap_or_else(|| "friend".to_string());
///             
///             GetPromptResult {
///                 description: None,
///                 messages: vec![
///                     PromptMessage {
///                         role: "user".to_string(),
///                         content: MessageContent::Text {
///                             text: format!("Hello, {}!", name),
///                         },
///                     },
///                 ],
///             }
///         })
///     }
/// }
/// ```
pub trait PromptCallback: Send + Sync {
    /// Calls the prompt with optional arguments
    ///
    /// # Arguments
    ///
    /// * `args` - Optional map of argument names to values
    ///
    /// # Returns
    ///
    /// A future that resolves to a GetPromptResult
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
///
/// Provides a fluent interface for constructing prompts with
/// arguments, descriptions, and completion callbacks.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::prompt::{PromptBuilder, GetPromptResult, PromptMessage};
/// use mcp_daemon::types::MessageContent;
/// use mcp_daemon::completable::CompletableString;
///
/// // Create a prompt with required and optional arguments
/// let (metadata, registered) = PromptBuilder::new("code-review")
///     .description("Generate a code review")
///     .required_arg("language", Some("Programming language"))
///     .required_arg("code", Some("Code to review"))
///     .optional_arg("focus", Some("Review focus (e.g., 'security', 'performance')"))
///     .with_completion(
///         "language",
///         CompletableString::new(|_input: &str| {
///             async move {
///                 vec![
///                     "python".to_string(),
///                     "javascript".to_string(),
///                     "rust".to_string(),
///                 ]
///             }
///         }),
///     )
///     .build(|args| async move {
///         // Prompt execution logic
///         GetPromptResult {
///             description: Some("Code review prompt".to_string()),
///             messages: vec![
///                 PromptMessage {
///                     role: "user".to_string(),
///                     content: MessageContent::Text {
///                         text: "Please review this code...".to_string(),
///                     },
///                 },
///             ],
///         }
///     })
///     .expect("Failed to build prompt");
/// ```
pub struct PromptBuilder {
    name: String,
    description: Option<String>,
    arguments: Vec<PromptArgument>,
    argument_completions: HashMap<String, Arc<dyn Completable<Input = str, Output = String>>>,
}

impl PromptBuilder {
    /// Create a new prompt builder with the given name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the prompt
    ///
    /// # Returns
    ///
    /// A new PromptBuilder instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::prompt::PromptBuilder;
    ///
    /// let builder = PromptBuilder::new("greeting");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: Vec::new(),
            argument_completions: HashMap::new(),
        }
    }

    /// Add a description to the prompt
    ///
    /// # Arguments
    ///
    /// * `description` - The description of the prompt
    ///
    /// # Returns
    ///
    /// The updated PromptBuilder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::prompt::PromptBuilder;
    ///
    /// let builder = PromptBuilder::new("greeting")
    ///     .description("A friendly greeting prompt");
    /// ```
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a required argument to the prompt
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the argument
    /// * `description` - Optional description of the argument
    ///
    /// # Returns
    ///
    /// The updated PromptBuilder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::prompt::PromptBuilder;
    ///
    /// let builder = PromptBuilder::new("greeting")
    ///     .required_arg("name", Some("Person's name"));
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the argument
    /// * `description` - Optional description of the argument
    ///
    /// # Returns
    ///
    /// The updated PromptBuilder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::prompt::PromptBuilder;
    ///
    /// let builder = PromptBuilder::new("greeting")
    ///     .optional_arg("style", Some("Greeting style (formal/casual)"));
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `arg_name` - The name of the argument to add completion for
    /// * `completable` - The completion implementation
    ///
    /// # Returns
    ///
    /// The updated PromptBuilder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::prompt::PromptBuilder;
    /// use mcp_daemon::completable::CompletableString;
    ///
    /// let builder = PromptBuilder::new("greeting")
    ///     .required_arg("name", Some("Person's name"))
    ///     .with_completion(
    ///         "name",
    ///         CompletableString::new(|input: &str| {
    ///             let input = input.to_string();
    ///             async move {
    ///                 vec!["Alice", "Bob", "Charlie"]
    ///                     .into_iter()
    ///                     .filter(|name| name.to_lowercase().starts_with(&input.to_lowercase()))
    ///                     .map(|name| name.to_string())
    ///                     .collect()
    ///             }
    ///         }),
    ///     );
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `callback` - The function to execute when the prompt is called
    ///
    /// # Returns
    ///
    /// A Result containing the prompt metadata and registered prompt,
    /// or an error message if validation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use mcp_daemon::server::prompt::{PromptBuilder, GetPromptResult, PromptMessage};
    /// use mcp_daemon::types::MessageContent;
    ///
    /// let result = PromptBuilder::new("greeting")
    ///     .required_arg("name", Some("Person's name"))
    ///     .build(|args| async move {
    ///         let name = args
    ///             .and_then(|args| args.get("name").cloned())
    ///             .unwrap_or_else(|| "friend".to_string());
    ///
    ///         GetPromptResult {
    ///             description: None,
    ///             messages: vec![
    ///                 PromptMessage {
    ///                     role: "user".to_string(),
    ///                     content: MessageContent::Text {
    ///                         text: format!("Hello, {}!", name),
    ///                     },
    ///                 },
    ///             ],
    ///         }
    ///     });
    /// ```
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
