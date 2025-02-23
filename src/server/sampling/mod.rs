use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::server::error::ServerError;

type Result<T> = std::result::Result<T, ServerError>;

/// Message role in a sampling conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Role representing the user in a conversation
    User,
    /// Role representing the AI assistant in a conversation
    Assistant,
}

/// Content type for a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MessageContent {
    /// Text content with a string payload
    Text {
        /// The text content
        text: String
    },
    /// Image content with base64-encoded data and optional MIME type
    Image {
        /// The image data (e.g., base64 encoded)
        data: String,
        /// The optional MIME type of the image
        mime_type: Option<String>
    },
}

/// A message in a sampling conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender (user or assistant)
    pub role: MessageRole,
    /// The content of the message (text or image)
    pub content: MessageContent,
}

/// Model selection preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    /// Optional hints for model selection
    pub hints: Option<Vec<ModelHint>>,
    /// Priority for cost optimization (0.0 to 1.0)
    pub cost_priority: Option<f32>,
    /// Priority for speed optimization (0.0 to 1.0)
    pub speed_priority: Option<f32>,
    /// Priority for intelligence/quality optimization (0.0 to 1.0)
    pub intelligence_priority: Option<f32>,
}

/// A hint for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHint {
    /// Optional specific model name to use
    pub name: Option<String>,
}

/// Context inclusion level for sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContextInclusion {
    /// No model preference
    None,
    /// Use the model running on this server
    ThisServer,
    /// Use models from all available servers
    AllServers,
}

/// Parameters for a sampling request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRequest {
    /// List of messages in the conversation
    pub messages: Vec<Message>,
    /// Optional model preferences for sampling
    pub model_preferences: Option<ModelPreferences>,
    /// Optional system prompt
    pub system_prompt: Option<String>,
    /// Optional context inclusion settings
    pub include_context: Option<ContextInclusion>,
    /// Optional temperature parameter for sampling
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    /// Optional sequences that will stop generation
    pub stop_sequences: Option<Vec<String>>,
    /// Optional metadata for the sampling request
    pub metadata: Option<HashMap<String, Value>>,
}

/// Stop reason for a sampling completion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    /// Generation stopped due to end of turn
    EndTurn,
    /// Generation stopped due to stop sequence
    StopSequence,
    /// Generation stopped due to max tokens
    MaxTokens,
    /// Generation stopped for unknown reason
    Unknown,
    #[serde(other)]
    /// Generation stopped for other reason
    Other,
}

/// Result of a sampling request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResult {
    /// The model to use for sampling
    pub model: String,
    /// The reason why generation stopped
    pub stop_reason: Option<StopReason>,
    pub role: MessageRole,
    pub content: MessageContent,
}

/// A callback that can handle sampling requests
pub trait SamplingCallback: Send + Sync {
    /// Calls the sampling function with the given request
    fn call(
        &self,
        request: SamplingRequest,
    ) -> Pin<Box<dyn Future<Output = Result<SamplingResult>> + Send + 'static>>;
}

impl<F, Fut> SamplingCallback for F
where
    F: Fn(SamplingRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<SamplingResult>> + Send + 'static,
{
    fn call(
        &self,
        request: SamplingRequest,
    ) -> Pin<Box<dyn Future<Output = Result<SamplingResult>> + Send + 'static>> {
        Box::pin(self(request))
    }
}

// Type aliases for complex future and callback types
type SamplingFuture = Pin<Box<dyn Future<Output = Result<SamplingResult>> + Send + 'static>>;
type SamplingCallbackFunc = Arc<dyn Fn(SamplingRequest) -> SamplingFuture + Send + Sync>;

/// A registered sampling handler
pub(crate) struct RegisteredSampling {
    /// The callback to handle sampling requests
    #[allow(dead_code)]
    pub callback: SamplingCallbackFunc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sampling_request() {
        let request = SamplingRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text {
                    text: "Hello".to_string(),
                },
            }],
            model_preferences: Some(ModelPreferences {
                hints: Some(vec![ModelHint {
                    name: Some("claude-3".to_string()),
                }]),
                cost_priority: Some(0.5),
                speed_priority: Some(0.8),
                intelligence_priority: Some(0.9),
            }),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            include_context: Some(ContextInclusion::ThisServer),
            temperature: Some(0.7),
            max_tokens: 100,
            stop_sequences: Some(vec!["END".to_string()]),
            metadata: None,
        };

        let callback = |_req: SamplingRequest| {
            Box::pin(async move {
                Ok(SamplingResult {
                    model: "claude-3".to_string(),
                    stop_reason: Some(StopReason::EndTurn),
                    role: MessageRole::Assistant,
                    content: MessageContent::Text {
                        text: "Hi there!".to_string(),
                    },
                })
            }) as Pin<Box<dyn Future<Output = Result<SamplingResult>> + Send>>
        };

        let result = callback(request).await.unwrap();
        assert_eq!(result.model, "claude-3");
        if let MessageContent::Text { text } = result.content {
            assert_eq!(text, "Hi there!");
        } else {
            panic!("Expected text content");
        }
    }
}
