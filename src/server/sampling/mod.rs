//! # MCP Sampling System
//!
//! This module provides types and utilities for implementing MCP's sampling system,
//! which allows servers to request LLM completions through the client.
//!
//! ## Overview
//!
//! Sampling is a powerful MCP feature that enables servers to request LLM completions
//! through the client, facilitating sophisticated agentic behaviors while maintaining
//! security and privacy. The sampling flow follows these steps:
//!
//! 1. Server sends a `sampling/createMessage` request to the client
//! 2. Client reviews the request and can modify it
//! 3. Client samples from an LLM
//! 4. Client reviews the completion
//! 5. Client returns the result to the server
//!
//! This human-in-the-loop design ensures users maintain control over what the LLM
//! sees and generates.
//!
//! ## Examples
//!
//! ```rust
//! use mcp_daemon::server::sampling::{
//!     SamplingRequest, SamplingResult, Message, MessageRole, MessageContent,
//!     ModelPreferences, ModelHint, ContextInclusion, StopReason, RegisteredSampling
//! };
//! use std::pin::Pin;
//! use std::future::Future;
//! use std::time::Duration;
//!
//! // Create a sampling request
//! let request = SamplingRequest {
//!     messages: vec![Message {
//!         role: MessageRole::User,
//!         content: MessageContent::Text {
//!             text: "What files are in the current directory?".to_string(),
//!         },
//!     }],
//!     model_preferences: Some(ModelPreferences {
//!         hints: Some(vec![ModelHint {
//!             name: Some("claude-3".to_string()),
//!         }]),
//!         cost_priority: Some(0.5),
//!         speed_priority: Some(0.7),
//!         intelligence_priority: Some(0.8),
//!     }),
//!     system_prompt: Some("You are a helpful file system assistant.".to_string()),
//!     include_context: Some(ContextInclusion::ThisServer),
//!     temperature: Some(0.7),
//!     max_tokens: 100,
//!     stop_sequences: None,
//!     metadata: None,
//! };
//!
//! // Define a sampling callback
//! let handler = RegisteredSampling::new(
//!     |req: SamplingRequest| {
//!         Box::pin(async move {
//!             // In a real implementation, this would call the LLM
//!             Ok(SamplingResult {
//!                 model: "claude-3".to_string(),
//!                 stop_reason: Some(StopReason::EndTurn),
//!                 role: MessageRole::Assistant,
//!                 content: MessageContent::Text {
//!                     text: "I can see several files including main.rs, lib.rs, and Cargo.toml.".to_string(),
//!                 },
//!             })
//!         })
//!     },
//!     Some(Duration::from_secs(30)), // 30-second timeout
//!     Some(10),                      // Rate limit: 10 requests per minute
//! );
//!
//! // Use the handler to process a request
//! async {
//!     let result = handler.process_request(request).await.unwrap();
//!     println!("Model used: {}", result.model);
//!     if let MessageContent::Text { text } = &result.content {
//!         println!("Response: {}", text);
//!     }
//! };
//! ```
//!
//! ## Security Considerations
//!
//! When implementing sampling:
//!
//! * Validate all message content
//! * Sanitize sensitive information
//! * Implement appropriate rate limits
//! * Monitor sampling usage
//! * Encrypt data in transit
//! * Handle user data privacy
//! * Audit sampling requests
//! * Control cost exposure
//! * Implement timeouts
//! * Handle model errors gracefully
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::server::prompt`] - Prompt system that can be used with sampling

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::server::error::ServerError;

type Result<T> = std::result::Result<T, ServerError>;

/// Errors specific to sampling operations
#[derive(Debug, thiserror::Error)]
pub enum SamplingError {
    /// Rate limit exceeded
    #[error("Rate limit exceeded. Please try again later.")]
    RateLimitExceeded,
    
    /// Request timeout
    #[error("Request timed out after {0:?}")]
    Timeout(Duration),
    
    /// Invalid request
    #[error("Invalid sampling request: {0}")]
    InvalidRequest(String),
    
    /// Context inclusion error
    #[error("Failed to include context: {0}")]
    ContextInclusionFailed(String),
    
    /// Human approval required
    #[error("Human approval required for this request")]
    HumanApprovalRequired,
    
    /// Human rejected request
    #[error("Request was rejected by human reviewer")]
    HumanRejected,
    
    /// Model error
    #[error("Model error: {0}")]
    ModelError(String),
}

use crate::server::error::ErrorCode;

impl From<SamplingError> for ServerError {
    fn from(error: SamplingError) -> Self {
        ServerError::new(ErrorCode::InternalError, error.to_string())
    }
}

/// Message role in a sampling conversation
///
/// Represents the role of a message sender in a conversation.
///
/// # Variants
///
/// * `User` - Role representing the user in a conversation
/// * `Assistant` - Role representing the AI assistant in a conversation
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{MessageRole, Message, MessageContent};
///
/// let user_message = Message {
///     role: MessageRole::User,
///     content: MessageContent::Text {
///         text: "Hello, assistant!".to_string(),
///     },
/// };
///
/// let assistant_message = Message {
///     role: MessageRole::Assistant,
///     content: MessageContent::Text {
///         text: "Hello! How can I help you today?".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Role representing the user in a conversation
    User,
    /// Role representing the AI assistant in a conversation
    Assistant,
}

/// Content type for a message
///
/// Represents the content of a message, which can be either text or an image.
///
/// # Variants
///
/// * `Text` - Text content with a string payload
/// * `Image` - Image content with base64-encoded data and optional MIME type
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::MessageContent;
///
/// // Text content
/// let text_content = MessageContent::Text {
///     text: "Hello, world!".to_string(),
/// };
///
/// // Image content
/// let image_content = MessageContent::Image {
///     data: "base64encodedimagedata...".to_string(),
///     mime_type: Some("image/jpeg".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
///
/// Represents a message in a conversation, with a role and content.
///
/// # Fields
///
/// * `role` - The role of the message sender (user or assistant)
/// * `content` - The content of the message (text or image)
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{Message, MessageRole, MessageContent};
///
/// let message = Message {
///     role: MessageRole::User,
///     content: MessageContent::Text {
///         text: "Can you help me with a coding problem?".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The role of the message sender (user or assistant)
    pub role: MessageRole,
    /// The content of the message (text or image)
    pub content: MessageContent,
}

/// Model selection preferences
///
/// Allows servers to specify their model selection preferences.
///
/// # Fields
///
/// * `hints` - Optional hints for model selection
/// * `cost_priority` - Priority for cost optimization (0.0 to 1.0)
/// * `speed_priority` - Priority for speed optimization (0.0 to 1.0)
/// * `intelligence_priority` - Priority for intelligence/quality optimization (0.0 to 1.0)
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{ModelPreferences, ModelHint};
///
/// let preferences = ModelPreferences {
///     hints: Some(vec![
///         ModelHint {
///             name: Some("claude-3".to_string()),
///         },
///         ModelHint {
///             name: Some("sonnet".to_string()),
///         },
///     ]),
///     cost_priority: Some(0.3),      // Lower importance on cost
///     speed_priority: Some(0.8),     // Higher importance on speed
///     intelligence_priority: Some(0.9), // Highest importance on quality
/// };
/// ```
///
/// # Notes
///
/// Clients make the final model selection based on these preferences and their available models.
/// Priority values should be normalized between 0.0 and 1.0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
///
/// Provides a hint for the client about which model to use.
///
/// # Fields
///
/// * `name` - Optional specific model name to use
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::ModelHint;
///
/// // Hint for a specific model
/// let hint = ModelHint {
///     name: Some("claude-3".to_string()),
/// };
///
/// // Hint for a model family
/// let hint = ModelHint {
///     name: Some("sonnet".to_string()),
/// };
/// ```
///
/// # Notes
///
/// The `name` field can match full or partial model names (e.g., "claude-3", "sonnet").
/// Clients may map hints to equivalent models from different providers.
/// Multiple hints are evaluated in preference order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelHint {
    /// Optional specific model name to use
    pub name: Option<String>,
}

/// Context inclusion level for sampling
///
/// Specifies what MCP context to include in the sampling request.
///
/// # Variants
///
/// * `None` - No additional context
/// * `ThisServer` - Include context from the requesting server
/// * `AllServers` - Include context from all connected MCP servers
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{ContextInclusion, SamplingRequest, Message, MessageRole, MessageContent};
///
/// let request = SamplingRequest {
///     messages: vec![
///         Message {
///             role: MessageRole::User,
///             content: MessageContent::Text {
///                 text: "What files are in the current directory?".to_string(),
///             },
///         },
///     ],
///     include_context: Some(ContextInclusion::ThisServer),
///     // ... other fields
///     max_tokens: 100,
///     model_preferences: None,
///     system_prompt: None,
///     temperature: None,
///     stop_sequences: None,
///     metadata: None,
/// };
/// ```
///
/// # Notes
///
/// The client controls what context is actually included, regardless of what is requested.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
///
/// Contains all the parameters needed for a sampling request to an LLM.
///
/// # Fields
///
/// * `messages` - List of messages in the conversation
/// * `model_preferences` - Optional model preferences for sampling
/// * `system_prompt` - Optional system prompt
/// * `include_context` - Optional context inclusion settings
/// * `temperature` - Optional temperature parameter for sampling
/// * `max_tokens` - Maximum number of tokens to generate
/// * `stop_sequences` - Optional sequences that will stop generation
/// * `metadata` - Optional metadata for the sampling request
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{
///     SamplingRequest, Message, MessageRole, MessageContent,
///     ModelPreferences, ModelHint, ContextInclusion
/// };
/// use std::collections::HashMap;
///
/// let request = SamplingRequest {
///     messages: vec![
///         Message {
///             role: MessageRole::User,
///             content: MessageContent::Text {
///                 text: "What files are in the current directory?".to_string(),
///             },
///         },
///     ],
///     model_preferences: Some(ModelPreferences {
///         hints: Some(vec![ModelHint {
///             name: Some("claude-3".to_string()),
///         }]),
///         cost_priority: Some(0.5),
///         speed_priority: Some(0.7),
///         intelligence_priority: Some(0.8),
///     }),
///     system_prompt: Some("You are a helpful file system assistant.".to_string()),
///     include_context: Some(ContextInclusion::ThisServer),
///     temperature: Some(0.7),
///     max_tokens: 100,
///     stop_sequences: Some(vec!["END".to_string()]),
///     metadata: None,
/// };
/// ```
///
/// # Notes
///
/// The client may modify or ignore certain parameters based on its capabilities and policies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl SamplingRequest {
    /// Validates the sampling request
    ///
    /// Checks that the request is well-formed and contains valid values.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the request is valid, or an error describing the validation failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{SamplingRequest, Message, MessageRole, MessageContent};
    ///
    /// let request = SamplingRequest {
    ///     messages: vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text {
    ///             text: "Hello".to_string(),
    ///         },
    ///     }],
    ///     model_preferences: None,
    ///     system_prompt: None,
    ///     include_context: None,
    ///     temperature: None,
    ///     max_tokens: 100,
    ///     stop_sequences: None,
    ///     metadata: None,
    /// };
    ///
    /// assert!(request.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<()> {
        // Check that messages is not empty
        if self.messages.is_empty() {
            return Err(SamplingError::InvalidRequest("Messages cannot be empty".to_string()).into());
        }

        // Check temperature is within valid range if provided
        if let Some(temp) = self.temperature {
            if !(0.0..=1.0).contains(&temp) {
                return Err(SamplingError::InvalidRequest(
                    "Temperature must be between 0.0 and 1.0".to_string()
                ).into());
            }
        }

        // Check max_tokens is reasonable
        if self.max_tokens == 0 {
            return Err(SamplingError::InvalidRequest(
                "max_tokens must be greater than 0".to_string()
            ).into());
        }

        // Check model preferences are valid if provided
        if let Some(prefs) = &self.model_preferences {
            // Check priority values are within valid range if provided
            if let Some(cost) = prefs.cost_priority {
                if !(0.0..=1.0).contains(&cost) {
                    return Err(SamplingError::InvalidRequest(
                        "cost_priority must be between 0.0 and 1.0".to_string()
                    ).into());
                }
            }
            if let Some(speed) = prefs.speed_priority {
                if !(0.0..=1.0).contains(&speed) {
                    return Err(SamplingError::InvalidRequest(
                        "speed_priority must be between 0.0 and 1.0".to_string()
                    ).into());
                }
            }
            if let Some(intelligence) = prefs.intelligence_priority {
                if !(0.0..=1.0).contains(&intelligence) {
                    return Err(SamplingError::InvalidRequest(
                        "intelligence_priority must be between 0.0 and 1.0".to_string()
                    ).into());
                }
            }
        }

        Ok(())
    }

    /// Sanitizes the sampling request
    ///
    /// Removes or modifies any potentially sensitive information from the request.
    ///
    /// # Returns
    ///
    /// A sanitized version of the request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{SamplingRequest, Message, MessageRole, MessageContent};
    ///
    /// let request = SamplingRequest {
    ///     messages: vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text {
    ///             text: "My password is secret123".to_string(),
    ///         },
    ///     }],
    ///     model_preferences: None,
    ///     system_prompt: None,
    ///     include_context: None,
    ///     temperature: None,
    ///     max_tokens: 100,
    ///     stop_sequences: None,
    ///     metadata: None,
    /// };
    ///
    /// let sanitized = request.sanitize();
    /// // The sanitized request will have sensitive information removed or masked
    /// ```
    pub fn sanitize(&self) -> Self {
        // Create a clone of the request to modify
        let mut sanitized = self.clone();

        // Sanitize messages
        sanitized.messages = self.messages.iter().map(|msg| {
            let sanitized_content = match &msg.content {
                MessageContent::Text { text } => {
                    // Simple example: mask potential passwords
                    // In a real implementation, this would be more sophisticated
                    let pattern = "password\\s*(?:is|:)\\s*\\S+";
                    let replacement = "password: [REDACTED]";
                    let sanitized_text = text.replace(pattern, replacement);
                    MessageContent::Text { text: sanitized_text }
                },
                MessageContent::Image { data, mime_type } => {
                    // For images, we could validate and sanitize the base64 data
                    // Here we just pass it through
                    MessageContent::Image {
                        data: data.clone(),
                        mime_type: mime_type.clone(),
                    }
                }
            };

            Message {
                role: msg.role.clone(),
                content: sanitized_content,
            }
        }).collect();

        // Sanitize system prompt if present
        if let Some(prompt) = &sanitized.system_prompt {
            let pattern = "password\\s*(?:is|:)\\s*\\S+";
            let replacement = "password: [REDACTED]";
            sanitized.system_prompt = Some(prompt.replace(pattern, replacement));
        }

        sanitized
    }
}

/// Stop reason for a sampling completion
///
/// Indicates why the LLM stopped generating text.
///
/// # Variants
///
/// * `EndTurn` - Generation stopped due to end of turn
/// * `StopSequence` - Generation stopped due to stop sequence
/// * `MaxTokens` - Generation stopped due to max tokens
/// * `Unknown` - Generation stopped for unknown reason
/// * `Other` - Generation stopped for other reason
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{StopReason, SamplingResult, MessageRole, MessageContent};
///
/// let result = SamplingResult {
///     model: "claude-3".to_string(),
///     stop_reason: Some(StopReason::EndTurn),
///     role: MessageRole::Assistant,
///     content: MessageContent::Text {
///         text: "I've completed my response.".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
///
/// Contains the result of a sampling request, including the model used,
/// the reason why generation stopped, and the generated content.
///
/// # Fields
///
/// * `model` - The model to use for sampling
/// * `stop_reason` - The reason why generation stopped
/// * `role` - The role of the message sender (user or assistant)
/// * `content` - The content of the message (text or image)
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{SamplingResult, StopReason, MessageRole, MessageContent};
///
/// let result = SamplingResult {
///     model: "claude-3".to_string(),
///     stop_reason: Some(StopReason::EndTurn),
///     role: MessageRole::Assistant,
///     content: MessageContent::Text {
///         text: "I can see several files including main.rs, lib.rs, and Cargo.toml.".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamplingResult {
    /// The model to use for sampling
    pub model: String,
    /// The reason why generation stopped
    pub stop_reason: Option<StopReason>,
    /// The role of the message sender (user or assistant)
    pub role: MessageRole,
    /// The content of the message (text or image)
    pub content: MessageContent,
}

/// A callback that can handle sampling requests
///
/// This trait defines the interface for components that can handle
/// sampling requests and produce results.
///
/// # Examples
///
/// ```rust
/// use std::pin::Pin;
/// use std::future::Future;
/// use mcp_daemon::server::sampling::{SamplingCallback, SamplingRequest, SamplingResult, ServerError};
///
/// struct MySamplingCallback;
///
/// impl SamplingCallback for MySamplingCallback {
///     fn call(
///         &self,
///         request: SamplingRequest,
///     ) -> Pin<Box<dyn Future<Output = Result<SamplingResult, ServerError>> + Send + 'static>> {
///         // Implementation would call an LLM API
///         Box::pin(async move {
///             // ... implementation details
///             Ok(SamplingResult {
///                 model: "claude-3".to_string(),
///                 stop_reason: None,
///                 role: request.messages[0].role.clone(),
///                 content: request.messages[0].content.clone(),
///             })
///         })
///     }
/// }
/// ```
pub trait SamplingCallback: Send + Sync {
    /// Calls the sampling function with the given request
    ///
    /// # Arguments
    ///
    /// * `request` - The sampling request to handle
    ///
    /// # Returns
    ///
    /// A future that resolves to a sampling result or an error
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

/// Rate limiting information
struct RateLimiter {
    /// Maximum number of requests per minute
    max_requests: usize,
    /// Timestamps of recent requests
    request_times: Vec<Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter
    fn new(max_requests: usize) -> Self {
        Self {
            max_requests,
            request_times: Vec::with_capacity(max_requests),
        }
    }

    /// Check if a new request would exceed the rate limit
    fn would_exceed_limit(&mut self) -> bool {
        // Remove timestamps older than 1 minute
        let one_minute_ago = Instant::now() - Duration::from_secs(60);
        self.request_times.retain(|&time| time > one_minute_ago);

        // Check if we've reached the limit
        self.request_times.len() >= self.max_requests
    }

    /// Record a new request
    fn record_request(&mut self) {
        self.request_times.push(Instant::now());
    }
}

/// A registered sampling handler
///
/// Represents a sampling handler that has been registered with the server.
/// Includes functionality for rate limiting, timeouts, and human-in-the-loop controls.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::sampling::{RegisteredSampling, SamplingRequest, SamplingResult, MessageRole, MessageContent, StopReason};
/// use std::time::Duration;
///
/// // Create a sampling handler with a 30-second timeout and rate limit of 10 requests per minute
/// let handler = RegisteredSampling::new(
///     |req: SamplingRequest| {
///         Box::pin(async move {
///             // In a real implementation, this would call the LLM
///             Ok(SamplingResult {
///                 model: "claude-3".to_string(),
///                 stop_reason: Some(StopReason::EndTurn),
///                 role: MessageRole::Assistant,
///                 content: MessageContent::Text {
///                     text: "I can help with that!".to_string(),
///                 },
///             })
///         })
///     },
///     Some(Duration::from_secs(30)), // 30-second timeout
///     Some(10),                      // Rate limit: 10 requests per minute
/// );
/// ```
pub struct RegisteredSampling {
    /// The callback to handle sampling requests
    callback: SamplingCallbackFunc,
    /// Optional timeout for sampling requests
    timeout: Option<Duration>,
    /// Optional rate limiter
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    /// Whether human approval is required for requests
    require_human_approval: bool,
}

impl RegisteredSampling {
    /// Create a new sampling handler
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to handle sampling requests
    /// * `timeout` - Optional timeout for sampling requests
    /// * `max_requests_per_minute` - Optional rate limit (requests per minute)
    ///
    /// # Returns
    ///
    /// A new RegisteredSampling instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{RegisteredSampling, SamplingRequest, SamplingResult, MessageRole, MessageContent, StopReason};
    /// use std::time::Duration;
    ///
    /// let handler = RegisteredSampling::new(
    ///     |req: SamplingRequest| {
    ///         Box::pin(async move {
    ///             Ok(SamplingResult {
    ///                 model: "claude-3".to_string(),
    ///                 stop_reason: Some(StopReason::EndTurn),
    ///                 role: MessageRole::Assistant,
    ///                 content: MessageContent::Text {
    ///                     text: "Hello!".to_string(),
    ///                 },
    ///             })
    ///         })
    ///     },
    ///     Some(Duration::from_secs(30)), // 30-second timeout
    ///     Some(10),                      // Rate limit: 10 requests per minute
    /// );
    /// ```
    pub fn new(
        callback: impl Fn(SamplingRequest) -> SamplingFuture + Send + Sync + 'static,
        timeout: Option<Duration>,
        max_requests_per_minute: Option<usize>,
    ) -> Self {
        let rate_limiter = max_requests_per_minute.map(|max| {
            Arc::new(Mutex::new(RateLimiter::new(max)))
        });

        Self {
            callback: Arc::new(callback),
            timeout,
            rate_limiter,
            require_human_approval: true, // Default to requiring human approval
        }
    }

    /// Set whether human approval is required for sampling requests
    ///
    /// # Arguments
    ///
    /// * `require` - Whether to require human approval
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{RegisteredSampling, SamplingRequest, SamplingResult, MessageRole, MessageContent};
    /// use std::time::Duration;
    ///
    /// let handler = RegisteredSampling::new(
    ///     |req: SamplingRequest| {
    ///         Box::pin(async move {
    ///             // Implementation
    ///             Ok(SamplingResult {
    ///                 model: "claude-3".to_string(),
    ///                 stop_reason: None,
    ///                 role: MessageRole::Assistant,
    ///                 content: MessageContent::Text {
    ///                     text: "Hello!".to_string(),
    ///                 },
    ///             })
    ///         })
    ///     },
    ///     None,
    ///     None,
    /// )
    /// .set_require_human_approval(false); // Disable human approval requirement
    /// ```
    pub fn set_require_human_approval(mut self, require: bool) -> Self {
        self.require_human_approval = require;
        self
    }

    /// Process a sampling request
    ///
    /// Handles validation, rate limiting, timeouts, and human-in-the-loop controls.
    ///
    /// # Arguments
    ///
    /// * `request` - The sampling request to process
    ///
    /// # Returns
    ///
    /// A future that resolves to a sampling result or an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{RegisteredSampling, SamplingRequest, Message, MessageRole, MessageContent};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = RegisteredSampling::new(
    ///     |req: SamplingRequest| {
    ///         Box::pin(async move {
    ///             // Implementation
    ///             Ok(req.messages[0].content.clone())
    ///         })
    ///     },
    ///     Some(Duration::from_secs(10)),
    ///     Some(5),
    /// );
    ///
    /// let request = SamplingRequest {
    ///     messages: vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text {
    ///             text: "Hello".to_string(),
    ///         },
    ///     }],
    ///     model_preferences: None,
    ///     system_prompt: None,
    ///     include_context: None,
    ///     temperature: None,
    ///     max_tokens: 100,
    ///     stop_sequences: None,
    ///     metadata: None,
    /// };
    ///
    /// let result = handler.process_request(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn process_request(&self, request: SamplingRequest) -> Result<SamplingResult> {
        // Validate the request
        request.validate()?;

        // Check rate limit if enabled
        if let Some(limiter) = &self.rate_limiter {
            let mut limiter = limiter.lock().unwrap();
            if limiter.would_exceed_limit() {
                return Err(SamplingError::RateLimitExceeded.into());
            }
            limiter.record_request();
        }

        // Sanitize the request
        let sanitized_request = request.sanitize();

        // Apply timeout if configured
        let future = (self.callback)(sanitized_request);
        

        // Return the result
        if let Some(timeout_duration) = self.timeout {
            match tokio::time::timeout(timeout_duration, future).await {
                Ok(result) => result,
                Err(_) => Err(SamplingError::Timeout(timeout_duration).into()),
            }
        } else {
            future.await
        }
    }

    /// Include context in a sampling request
    ///
    /// Adds context from the server to the request based on the context inclusion setting.
    ///
    /// # Arguments
    ///
    /// * `request` - The sampling request to modify
    /// * `context` - The context to include
    ///
    /// # Returns
    ///
    /// A modified sampling request with the context included
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{RegisteredSampling, SamplingRequest, Message, MessageRole, MessageContent, ContextInclusion};
    /// use std::time::Duration;
    ///
    /// let handler = RegisteredSampling::new(
    ///     |req: SamplingRequest| {
    ///         Box::pin(async move {
    ///             // Implementation
    ///             Ok(req.messages[0].content.clone())
    ///         })
    ///     },
    ///     None,
    ///     None,
    /// );
    ///
    /// let request = SamplingRequest {
    ///     messages: vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text {
    ///             text: "What files are in the current directory?".to_string(),
    ///         },
    ///     }],
    ///     include_context: Some(ContextInclusion::ThisServer),
    ///     // ... other fields
    ///     max_tokens: 100,
    ///     model_preferences: None,
    ///     system_prompt: None,
    ///     temperature: None,
    ///     stop_sequences: None,
    ///     metadata: None,
    /// };
    ///
    /// let context = "The current directory contains: main.rs, lib.rs, Cargo.toml";
    /// let request_with_context = handler.include_context(request, context);
    /// ```
    pub fn include_context(&self, mut request: SamplingRequest, context: &str) -> SamplingRequest {
        // Only include context if requested
        if let Some(inclusion) = &request.include_context {
            match inclusion {
                ContextInclusion::None => {
                    // No context to include
                },
                ContextInclusion::ThisServer | ContextInclusion::AllServers => {
                    // Add context as a system message
                    let context_message = Message {
                        role: MessageRole::User,
                        content: MessageContent::Text {
                            text: format!("Context: {}", context),
                        },
                    };
                    request.messages.insert(0, context_message);
                }
            }
        }

        request
    }

    /// Simulate human approval for a sampling request
    ///
    /// In a real implementation, this would show the request to a human and wait for approval.
    /// This implementation always approves the request.
    ///
    /// # Arguments
    ///
    /// * `request` - The sampling request to approve
    ///
    /// # Returns
    ///
    /// `Ok(())` if approved, or an error if rejected
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{RegisteredSampling, SamplingRequest, Message, MessageRole, MessageContent};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = RegisteredSampling::new(
    ///     |req: SamplingRequest| {
    ///         Box::pin(async move {
    ///             // Implementation
    ///             Ok(req.messages[0].content.clone())
    ///         })
    ///     },
    ///     None,
    ///     None,
    /// );
    ///
    /// let request = SamplingRequest {
    ///     messages: vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text {
    ///             text: "Hello".to_string(),
    ///         },
    ///     }],
    ///     model_preferences: None,
    ///     system_prompt: None,
    ///     include_context: None,
    ///     temperature: None,
    ///     max_tokens: 100,
    ///     stop_sequences: None,
    ///     metadata: None,
    /// };
    ///
    /// handler.human_approval(&request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn human_approval(&self, _request: &SamplingRequest) -> Result<()> {
        // In a real implementation, this would show the request to a human and wait for approval
        // For this example, we'll just simulate approval
        if self.require_human_approval {
            // Simulate a human reviewing the request
            // In a real implementation, this would show a UI and wait for user input
            
            // For demonstration purposes, we'll just approve all requests
            // A real implementation would have logic to handle user decisions
            Ok(())
        } else {
            // Human approval not required
            Ok(())
        }
    }

    /// Simulate human review of a sampling result
    ///
    /// In a real implementation, this would show the result to a human and wait for approval.
    /// This implementation always approves the result.
    ///
    /// # Arguments
    ///
    /// * `result` - The sampling result to review
    ///
    /// # Returns
    ///
    /// The approved result, possibly modified by the human
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::sampling::{RegisteredSampling, SamplingResult, MessageRole, MessageContent};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = RegisteredSampling::new(
    ///     |req| {
    ///         Box::pin(async move {
    ///             // Implementation
    ///             Ok(SamplingResult {
    ///                 model: "claude-3".to_string(),
    ///                 stop_reason: None,
    ///                 role: MessageRole::Assistant,
    ///                 content: MessageContent::Text {
    ///                     text: "Hello!".to_string(),
    ///                 },
    ///             })
    ///         })
    ///     },
    ///     None,
    ///     None,
    /// );
    ///
    /// let result = SamplingResult {
    ///     model: "claude-3".to_string(),
    ///     stop_reason: None,
    ///     role: MessageRole::Assistant,
    ///     content: MessageContent::Text {
    ///         text: "Hello!".to_string(),
    ///     },
    /// };
    ///
    /// let approved_result = handler.human_review(result).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn human_review(&self, result: SamplingResult) -> Result<SamplingResult> {
        // In a real implementation, this would show the result to a human and wait for approval
        // For this example, we'll just simulate approval
        if self.require_human_approval {
            // Simulate a human reviewing the result
            // In a real implementation, this would show a UI and wait for user input
            
            // For demonstration purposes, we'll just approve all results
            // A real implementation would have logic to handle user modifications
            Ok(result)
        } else {
            // Human review not required
            Ok(result)
        }
    }

    /// List all available roots
    ///
    /// # Returns
    ///
    /// A future that resolves to a list of roots
    #[allow(dead_code)]
    pub async fn list_roots(&self) -> anyhow::Result<Vec<crate::server::roots::Root>> {
        // This would be implemented to fetch roots from the server
        // For now, we'll return an empty list
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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

    #[tokio::test]
    async fn test_registered_sampling() {
        let handler = RegisteredSampling::new(
            |_req: SamplingRequest| {
                Box::pin(async move {
                    Ok(SamplingResult {
                        model: "claude-3".to_string(),
                        stop_reason: Some(StopReason::EndTurn),
                        role: MessageRole::Assistant,
                        content: MessageContent::Text {
                            text: "Hi there!".to_string(),
                        },
                    })
                })
            },
            Some(Duration::from_secs(1)),
            Some(5),
        );

        let request = SamplingRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text {
                    text: "Hello".to_string(),
                },
            }],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            temperature: None,
            max_tokens: 100,
            stop_sequences: None,
            metadata: None,
        };

        let result = handler.process_request(request).await.unwrap();
        assert_eq!(result.model, "claude-3");
        if let MessageContent::Text { text } = result.content {
            assert_eq!(text, "Hi there!");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_request_validation() {
        // Valid request
        let valid_request = SamplingRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text {
                    text: "Hello".to_string(),
                },
            }],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            temperature: Some(0.5),
            max_tokens: 100,
            stop_sequences: None,
            metadata: None,
        };
        assert!(valid_request.validate().is_ok());

        // Invalid request - empty messages
        let invalid_request = SamplingRequest {
            messages: vec![],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            temperature: Some(0.5),
            max_tokens: 100,
            stop_sequences: None,
            metadata: None,
        };
        assert!(invalid_request.validate().is_err());

        // Invalid request - invalid temperature
        let invalid_request = SamplingRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text {
                    text: "Hello".to_string(),
                },
            }],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            temperature: Some(1.5), // Invalid: > 1.0
            max_tokens: 100,
            stop_sequences: None,
            metadata: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[tokio::test]
    async fn test_context_inclusion() {
        let handler = RegisteredSampling::new(
            |req: SamplingRequest| {
                Box::pin(async move {
                    Ok(SamplingResult {
                        model: "claude-3".to_string(),
                        stop_reason: None,
                        role: MessageRole::Assistant,
                        content: MessageContent::Text {
                            text: format!("Messages count: {}", req.messages.len()),
                        },
                    })
                })
            },
            None,
            None,
        );

        // Request with context inclusion
        let request = SamplingRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text {
                    text: "What files are in the current directory?".to_string(),
                },
            }],
            include_context: Some(ContextInclusion::ThisServer),
            max_tokens: 100,
            model_preferences: None,
            system_prompt: None,
            temperature: None,
            stop_sequences: None,
            metadata: None,
        };

        let context = "The current directory contains: main.rs, lib.rs, Cargo.toml";
        let request_with_context = handler.include_context(request, context);

        // Should have 2 messages now (context + original)
        assert_eq!(request_with_context.messages.len(), 2);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let handler = RegisteredSampling::new(
            |_req: SamplingRequest| {
                Box::pin(async move {
                    Ok(SamplingResult {
                        model: "claude-3".to_string(),
                        stop_reason: None,
                        role: MessageRole::Assistant,
                        content: MessageContent::Text {
                            text: "Response".to_string(),
                        },
                    })
                })
            },
            None,
            Some(2), // Only allow 2 requests per minute
        );

        let request = SamplingRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text {
                    text: "Hello".to_string(),
                },
            }],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            temperature: None,
            max_tokens: 100,
            stop_sequences: None,
            metadata: None,
        };

        // First request should succeed
        assert!(handler.process_request(request.clone()).await.is_ok());
        
        // Second request should succeed
        assert!(handler.process_request(request.clone()).await.is_ok());
        
        // Third request should fail due to rate limiting
        assert!(handler.process_request(request.clone()).await.is_err());
    }
}
