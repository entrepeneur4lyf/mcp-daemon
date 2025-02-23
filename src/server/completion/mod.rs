use serde::{Deserialize, Serialize};

/// A reference to a resource or prompt
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentInfo {
    /// The name of the argument
    pub name: String,
    /// The value of the argument to use for completion matching
    pub value: String,
}

/// A completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The reference to complete against
    pub ref_: Reference,
    /// The argument's information
    pub argument: ArgumentInfo,
}

/// A completion result matching MCP protocol specs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResult(pub Vec<CompletionResultItem>);

#[derive(Debug, Clone, Serialize, Deserialize)]
/// An item in a completion result
pub struct CompletionResultItem {
    /// The completion text suggestion
    pub text: String,
    /// Optional reason why the completion was finished
    pub finish_reason: Option<String>,
}

/// A callback that can provide completions
pub trait CompletionCallback: Send + Sync {
    /// Get completion suggestions for a reference and argument
    fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResult>;
}

/// A registered completion handler
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
