//! # MCP Completion System
//!
//! This module provides types and traits for implementing MCP's completion functionality.
//! Completions allow servers to provide suggestions for argument values based on
//! resources or prompts.
//!
//! ## Overview
//!
//! The completion system enables intelligent auto-completion for arguments in MCP clients.
//! Servers can provide completion suggestions for:
//! * Resource URIs or URI templates
//! * Prompt arguments
//!
//! This allows for a more interactive and user-friendly experience when working with
//! MCP resources and prompts.
//!
//! ## Examples
//!
//! ```rust
//! use mcp_daemon::server::completion::{CompletionRequest, CompletionResult, CompletionResultItem, Reference, ArgumentInfo};
//!
//! // Create a completion request for a resource
//! let request = CompletionRequest {
//!     ref_: Reference::Resource {
//!         uri: "file:///home/user/projects".to_string(),
//!     },
//!     argument: ArgumentInfo {
//!         name: "path".to_string(),
//!         value: "/home/user/pro".to_string(),
//!     },
//! };
//!
//! // Example completion handler implementation
//! fn handle_completion(request: CompletionRequest) -> anyhow::Result<CompletionResult> {
//!     match request.ref_ {
//!         Reference::Resource { uri } if uri.starts_with("file://") => {
//!             // Provide file path completions
//!             Ok(CompletionResult(vec![
//!                 CompletionResultItem {
//!                     text: "/home/user/projects".to_string(),
//!                     finish_reason: None,
//!                 },
//!                 CompletionResultItem {
//!                     text: "/home/user/programs".to_string(),
//!                     finish_reason: None,
//!                 },
//!             ]))
//!         },
//!         Reference::Prompt { name } => {
//!             // Provide prompt argument completions
//!             Ok(CompletionResult(vec![
//!                 CompletionResultItem {
//!                     text: "suggestion".to_string(),
//!                     finish_reason: None,
//!                 },
//!             ]))
//!         },
//!         _ => Ok(CompletionResult(vec![])),
//!     }
//! }
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::server::resource`] - Resource handling
//! * [`crate::server::prompt`] - Prompt handling

use serde::{Deserialize, Serialize};

/// A reference to a resource or prompt
///
/// Represents the target for a completion request, which can be either a resource
/// identified by a URI or a prompt identified by name.
///
/// # Variants
///
/// * `Resource` - Completion based on a resource URI
/// * `Prompt` - Completion based on a prompt name
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::completion::Reference;
///
/// // Resource reference
/// let resource_ref = Reference::Resource {
///     uri: "file:///home/user/documents".to_string(),
/// };
///
/// // Prompt reference
/// let prompt_ref = Reference::Prompt {
///     name: "greeting".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Reference {
    #[serde(rename = "ref/resource")]
    /// Completion based on a resource
    Resource {
        /// The URI or URI template of the resource
        uri: String,
    },
    #[serde(rename = "ref/prompt")]
    /// Completion based on a prompt
    Prompt {
        /// The name of the prompt or prompt template
        name: String,
    },
}

/// Argument information for completion
///
/// Contains details about the argument being completed, including its
/// name and the current partial value.
///
/// # Fields
///
/// * `name` - The name of the argument
/// * `value` - The current value of the argument (may be partial)
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::completion::ArgumentInfo;
///
/// let arg_info = ArgumentInfo {
///     name: "filename".to_string(),
///     value: "doc".to_string(), // Partial value for completion
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentInfo {
    /// The name of the argument
    pub name: String,
    /// The value of the argument to use for completion matching
    pub value: String,
}

/// A completion request
///
/// Represents a request for completion suggestions, containing the reference
/// to complete against and the argument information.
///
/// # Fields
///
/// * `ref_` - The reference to complete against (resource or prompt)
/// * `argument` - The argument's information
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::completion::{CompletionRequest, Reference, ArgumentInfo};
///
/// let request = CompletionRequest {
///     ref_: Reference::Resource {
///         uri: "file:///home/user".to_string(),
///     },
///     argument: ArgumentInfo {
///         name: "path".to_string(),
///         value: "/home/".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The reference to complete against
    pub ref_: Reference,
    /// The argument's information
    pub argument: ArgumentInfo,
}

/// A completion result matching MCP protocol specs
///
/// Contains a list of completion suggestions.
///
/// # Fields
///
/// * `0` - Vector of completion result items
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::completion::{CompletionResult, CompletionResultItem};
///
/// let result = CompletionResult(vec![
///     CompletionResultItem {
///         text: "suggestion1".to_string(),
///         finish_reason: None,
///     },
///     CompletionResultItem {
///         text: "suggestion2".to_string(),
///         finish_reason: Some("exact_match".to_string()),
///     },
/// ]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResult(pub Vec<CompletionResultItem>);

/// An item in a completion result
///
/// Represents a single completion suggestion with optional metadata.
///
/// # Fields
///
/// * `text` - The suggested completion text
/// * `finish_reason` - Optional reason why the completion was finished
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::completion::CompletionResultItem;
///
/// let item = CompletionResultItem {
///     text: "documents".to_string(),
///     finish_reason: Some("exact_match".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResultItem {
    /// The completion text suggestion
    pub text: String,
    /// Optional reason why the completion was finished
    pub finish_reason: Option<String>,
}

/// A callback that can provide completions
///
/// This trait defines the interface for completion handlers.
/// Implementors must provide a `complete` method that generates
/// completion suggestions for a given request.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::completion::{CompletionCallback, CompletionRequest, CompletionResult, CompletionResultItem};
///
/// struct MyCompletionHandler;
///
/// impl CompletionCallback for MyCompletionHandler {
///     fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResult> {
///         // Generate completions based on the request
///         Ok(CompletionResult(vec![
///             CompletionResultItem {
///                 text: "suggestion".to_string(),
///                 finish_reason: None,
///             },
///         ]))
///     }
/// }
/// ```
pub trait CompletionCallback: Send + Sync {
    /// Get completion suggestions for a reference and argument
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request containing the reference and argument information
    ///
    /// # Returns
    ///
    /// A Result containing the completion suggestions or an error
    fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResult>;
}

/// A registered completion handler
///
/// Internal structure used by the server to store a registered completion handler.
///
/// # Fields
///
/// * `callback` - The callback to handle completion requests
pub(crate) struct RegisteredCompletion {
    /// The callback to handle completion requests
    #[allow(dead_code)]
    pub callback: Box<dyn CompletionCallback>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_reference() {
        let reference = Reference::Resource {
            uri: "file:///path/to/file".to_string(),
        };

        let json = serde_json::to_string(&reference).unwrap();
        let deserialized: Reference = serde_json::from_str(&json).unwrap();

        match deserialized {
            Reference::Resource { uri } => {
                assert_eq!(uri, "file:///path/to/file");
            }
            _ => panic!("Wrong reference type"),
        }
    }

    #[test]
    fn test_prompt_reference() {
        let reference = Reference::Prompt {
            name: "test-prompt".to_string(),
        };

        let json = serde_json::to_string(&reference).unwrap();
        let deserialized: Reference = serde_json::from_str(&json).unwrap();

        match deserialized {
            Reference::Prompt { name } => {
                assert_eq!(name, "test-prompt");
            }
            _ => panic!("Wrong reference type"),
        }
    }

    #[test]
    fn test_completion_request() {
        let request = CompletionRequest {
            ref_: Reference::Resource {
                uri: "file:///path/to/file".to_string(),
            },
            argument: ArgumentInfo {
                name: "path".to_string(),
                value: "/path/to".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CompletionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(
            match deserialized.ref_ {
                Reference::Resource { uri } => uri,
                _ => panic!("Wrong reference type"),
            },
            "file:///path/to/file"
        );
        assert_eq!(deserialized.argument.name, "path");
        assert_eq!(deserialized.argument.value, "/path/to");
    }
}
