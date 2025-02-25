//! Server-sent events (SSE) transport implementation for the Model Context Protocol
//!
//! This module provides a transport layer for server-sent events (SSE) using the actix-web-lab crate.
//! SSE is a web technology where a browser receives automatic updates from a server via HTTP connection.
//! It's a standardized way to establish a long-lived, unidirectional connection from server to client.
//!
//! The SSE transport in MCP is primarily used for:
//! - Streaming responses from server to client
//! - Sending real-time updates and notifications
//! - Providing progress information for long-running operations
//! - Maintaining a persistent connection with minimal overhead
//!
//! Unlike WebSockets, SSE is unidirectional (server to client only) and uses standard HTTP,
//! making it simpler to implement and more compatible with existing infrastructure.
//!
//! # Examples
//!
//! ```no_run
//! use mcp_daemon::transport::{ServerSseTransport, Transport};
//! use actix_web::{web, App, HttpServer, Responder};
//! use serde_json::json;
//!
//! async fn sse_handler() -> impl Responder {
//!     // Create transport with 100 message buffer
//!     let (transport, responder) = ServerSseTransport::new_with_responder(100);
//!     
//!     // Send analysis events
//!     transport.send_event("analysis_start", "Beginning code analysis...").await?;
//!     
//!     // Send structured data
//!     transport.send_json(json!({
//!         "analysis": {
//!             "code_quality": {
//!                 "score": 0.85,
//!                 "issues": [{
//!                     "type": "security",
//!                     "severity": "medium",
//!                     "description": "Password hashing should use work factor of 12"
//!                 }]
//!             }
//!         }
//!     })).await?;
//!     
//!     // Send progress updates
//!     transport.send_event("progress", "Generating improvements...").await?;
//!     
//!     // Keep connection alive
//!     transport.send_comment("keep-alive").await?;
//!     
//!     Ok(responder)
//! }
//! ```

use std::time::Duration;
use async_trait::async_trait;
use actix_web_lab::sse;
use bytestring::ByteString;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::transport::{Message, Result, Transport};

/// Server-side SSE transport implementation
///
/// This transport provides a way to send server-sent events (SSE) to clients.
/// It implements the `Transport` trait and provides additional methods for
/// sending different types of SSE messages.
///
/// SSE is unidirectional (server to client only), so the `receive` method
/// always returns `None`. For bidirectional communication, consider using
/// the WebSocket transport instead.
///
/// # Examples
///
/// ```no_run
/// use mcp_daemon::transport::{ServerSseTransport, Transport};
/// use actix_web::{web, App, HttpServer, Responder};
/// use serde_json::json;
///
/// async fn sse_handler() -> impl Responder {
///     // Create transport with keep-alive enabled
///     let (transport, responder) = ServerSseTransport::new_with_responder(100);
///     
///     // Send analysis events
///     transport.send_event("thinking", "Analyzing implementation...").await?;
///     transport.send_event("analysis", "Code review in progress...").await?;
///     
///     // Send structured data
///     transport.send_json(json!({
///         "metadata": {
///             "model": "code-llm-v1",
///             "confidence": 0.92
///         },
///         "analysis": {
///             "issues": [{
///                 "type": "security",
///                 "severity": "medium"
///             }]
///         }
///     })).await?;
///     
///     Ok(responder)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ServerSseTransport {
    sender: mpsc::Sender<Result<sse::Event>>,
}

impl ServerSseTransport {
    /// Creates a new SSE transport with the given channel capacity
    ///
    /// This constructor creates a new SSE transport with a channel of the specified capacity,
    /// but without returning the responder. This is useful when you want to create a transport
    /// without immediately setting up the HTTP response.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The capacity of the channel buffer
    ///
    /// # Returns
    ///
    /// A new `ServerSseTransport` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::transport::ServerSseTransport;
    ///
    /// // Create a new SSE transport with a buffer capacity of 100 messages
    /// let transport = ServerSseTransport::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = mpsc::channel(capacity);
        Self { sender: tx }
    }

    /// Creates a new SSE transport with the given channel capacity and returns the transport and responder
    ///
    /// This constructor creates a new SSE transport and returns both the transport and
    /// an actix-web responder that can be used to send SSE messages to the client.
    /// The responder includes a keep-alive mechanism that sends a comment every 15 seconds
    /// to prevent the connection from timing out.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The capacity of the channel buffer
    ///
    /// # Returns
    ///
    /// A tuple containing the transport and an actix-web responder
    ///
    /// # Examples
    ///
/// ```
/// use mcp_daemon::transport::ServerSseTransport;
/// use actix_web::{web, App, HttpServer, Responder};
///
/// async fn sse_handler() -> impl Responder {
///     let (transport, responder) = ServerSseTransport::new_with_responder(100);
    ///     
    ///     // Store the transport somewhere for later use
    ///     // ...
    ///     
    ///     // Return the responder
    ///     responder
    /// }
    /// ```
    pub fn new_with_responder(capacity: usize) -> (Self, impl actix_web::Responder) {
        let (tx, rx) = mpsc::channel(capacity);
        let transport = Self { sender: tx };
        let responder = sse::Sse::from_stream(ReceiverStream::new(rx))
            .with_keep_alive(Duration::from_secs(15));
        (transport, responder)
    }

    /// Sends a message through the SSE channel
    ///
    /// This method serializes the message to JSON and sends it as an SSE data event.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    ///
    /// # Examples
    ///
/// ```
/// use mcp_daemon::transport::{ServerSseTransport, Message};
/// use mcp_daemon::transport::{JsonRpcMessage, JsonRpcNotification};
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let (transport, _) = ServerSseTransport::new_with_responder(100);
///     
///     let message = Message::Notification(JsonRpcNotification {
///         method: "notification".to_string(),
///         params: Some(serde_json::json!({
///             "type": "update",
///             "data": "New data available"
///         })),
///         jsonrpc: Default::default(),
///     });
///     
///     transport.send_message(message).await?;
///     
///     Ok(())
/// }
/// ```
    pub async fn send_message(&self, message: Message) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        self.sender
            .send(Ok(sse::Event::Data(sse::Data::new(json))))
            .await
            .map_err(|e| e.into())
    }

    /// Sends a data message through the SSE channel
    ///
    /// This method sends a string as an SSE data event.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    ///
    /// # Examples
    ///
/// ```no_run
/// use mcp_daemon::transport::ServerSseTransport;
/// use serde_json::json;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let (transport, _) = ServerSseTransport::new_with_responder(100);
///     
///     // Send raw data
///     transport.send_data("Processing file: main.rs").await?;
///     
///     // Send structured data
///     transport.send_json(json!({
///         "file": "main.rs",
///         "status": "processing",
///         "progress": 45
///     })).await?;
///     
///     Ok(())
/// }
/// ```
    pub async fn send_data(&self, data: impl Into<String>) -> Result<()> {
        let data = ByteString::from(data.into());
        self.sender
            .send(Ok(sse::Event::Data(sse::Data::new(data))))
            .await
            .map_err(|e| e.into())
    }

    /// Sends a JSON-serialized data message through the SSE channel
    ///
    /// This method serializes the data to JSON and sends it as an SSE data event.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize and send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    ///
    /// # Examples
    ///
/// ```no_run
/// use mcp_daemon::transport::ServerSseTransport;
/// use serde_json::json;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let (transport, _) = ServerSseTransport::new_with_responder(100);
///     
///     // Send analysis results
///     transport.send_json(json!({
///         "analysis": {
///             "code_quality": {
///                 "score": 0.85,
///                 "issues": [{
///                     "type": "security",
///                     "severity": "medium",
///                     "description": "Password hashing configuration"
///                 }]
///             },
///             "performance_insights": [{
///                 "type": "database",
///                 "description": "Consider adding index",
///                 "impact": "high"
///             }]
///         }
///     })).await?;
///     
///     Ok(())
/// }
/// ```
    pub async fn send_json<T: Serialize>(&self, data: T) -> Result<()> {
        let json = serde_json::to_string(&data)?;
        let data = ByteString::from(json);
        self.sender
            .send(Ok(sse::Event::Data(sse::Data::new(data))))
            .await
            .map_err(|e| e.into())
    }

    /// Sends a named event with data through the SSE channel
    ///
    /// This method sends a string as an SSE data event with a specified event name.
    /// Named events can be used to categorize different types of messages.
    ///
    /// # Arguments
    ///
    /// * `event` - The event name
    /// * `data` - The data to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    ///
    /// # Examples
    ///
/// ```no_run
/// use mcp_daemon::transport::ServerSseTransport;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let (transport, _) = ServerSseTransport::new_with_responder(100);
///     
///     // Send analysis events
///     transport.send_event("thinking", "<thinking>Analyzing code...</thinking>").await?;
///     transport.send_event("analysis", "Let me explain the implementation...").await?;
///     
///     // Send progress updates
///     transport.send_event("progress", "Generating improvements...").await?;
///     
///     // Send completion
///     transport.send_event("completion", "Analysis complete").await?;
///     
///     Ok(())
/// }
/// ```
    pub async fn send_event(&self, event: impl Into<String>, data: impl Into<String>) -> Result<()> {
        let data = ByteString::from(data.into());
        let event = ByteString::from(event.into());
        self.sender
            .send(Ok(sse::Event::Data(sse::Data::new(data).event(event))))
            .await
            .map_err(|e| e.into())
    }

    /// Sends a comment through the SSE channel
    ///
    /// This method sends a comment as an SSE comment event.
    /// Comments are useful for keep-alive messages and debugging.
    ///
    /// # Arguments
    ///
    /// * `comment` - The comment to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    ///
    /// # Examples
    ///
/// ```no_run
/// use mcp_daemon::transport::ServerSseTransport;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let (transport, _) = ServerSseTransport::new_with_responder(100);
///     
///     // Send keep-alive during long analysis
///     transport.send_comment("keep-alive").await?;
///     
///     // Send debug info
///     transport.send_comment("Debug: Processing file 42 of 100").await?;
///     
///     Ok(())
/// }
/// ```
    pub async fn send_comment(&self, comment: impl Into<String>) -> Result<()> {
        let comment = ByteString::from(comment.into());
        self.sender
            .send(Ok(sse::Event::Comment(comment)))
            .await
            .map_err(|e| e.into())
    }
}

#[async_trait]
impl Transport for ServerSseTransport {
    /// Sends a message through the transport
    ///
    /// This method delegates to `send_message` to send the message as an SSE data event.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    async fn send(&self, message: &Message) -> Result<()> {
        self.send_message(message.clone()).await
    }

    /// Receives a message from the transport
    ///
    /// Since SSE is unidirectional (server to client only), this method always returns `Ok(None)`.
    /// For bidirectional communication, consider using the WebSocket transport instead.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(None)`
    async fn receive(&self) -> Result<Option<Message>> {
        // SSE is unidirectional, server to client only
        Ok(None)
    }

    /// Opens the transport
    ///
    /// For the SSE transport, this is a no-op since the transport is initialized in the constructor.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`
    async fn open(&self) -> Result<()> {
        Ok(())
    }

    /// Closes the transport
    ///
    /// For the SSE transport, this is a no-op since the transport is closed when the client disconnects.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[actix_web::test]
    async fn test_sse_transport() {
        let (transport, _responder) = ServerSseTransport::new_with_responder(100);

        // Test thinking message
        transport.send_event("thinking", "<thinking>Analyzing the snapshot implementation...</thinking>").await.unwrap();

        // Test tool use message
        transport.send_event("tool_use", r#"{
            "tool": "read_file",
            "params": {
                "path": "src/snapshot/manager.rs"
            }
        }"#).await.unwrap();

        // Test large code analysis in chunks
        transport.send_event("analysis", "Let me explain the snapshot implementation in detail.").await.unwrap();
        
        transport.send_event("analysis", r#"
First, let's look at the directory creation process:
1. The code creates directories in the snapshot for directory entries
2. It ensures parent directories exist for all files
3. This is crucial for hard link creation, as target_path must be valid
"#).await.unwrap();

        transport.send_event("analysis", r#"
Performance considerations:
- Testing with large projects (36k files, 756MB)
- Creation time is minimal due to hard links
- Restoration uses fs::copy, which takes longer but is manageable
"#).await.unwrap();

        transport.send_event("analysis", r#"
The exclusion system is comprehensive:
1. Uses global ignore patterns
2. Supports per-directory .gitignore files
3. The ignore crate handles .gitignore automatically
4. walk_builder.git_ignore(true) enables this feature
5. add_global_ignore handles additional exclusions
"#).await.unwrap();

        // Test code snippet with implementation details
        transport.send_event("code", r#"
impl SnapshotManager {
    pub fn create_snapshot(&self, path: &Path) -> Result<()> {
        let walker = WalkBuilder::new(path)
            .git_ignore(true)
            .build();
        
        for entry in walker {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                self.create_directory(&entry)?;
            } else {
                self.create_hard_link(&entry)?;
            }
        }
        Ok(())
    }
}
"#).await.unwrap();

        transport.send_event("analysis", r#"
The restoration process:
1. Uses fs::copy for independence
2. Time-consuming but necessary
3. Could use hard links for space efficiency
4. But requires same filesystem
5. Most users need independent copies
"#).await.unwrap();

        // Test streaming completion
        transport.send_event("completion", "This implementation provides several benefits:").await.unwrap();
        transport.send_event("completion", "\n1. Efficient storage through hard links").await.unwrap();
        transport.send_event("completion", "\n2. Proper handling of exclusions").await.unwrap();
        transport.send_event("completion", "\n3. Flexible restoration options").await.unwrap();

        // Test error handling
        transport.send_event("error", "Warning: Restoration to a different filesystem will use more space").await.unwrap();

        // Test final completion with summary
        transport.send_event("completion_end", r#"{
            "status": "success",
            "message": "Analysis complete. The snapshot system provides efficient storage with proper exclusion handling and flexible restoration options."
        }"#).await.unwrap();

        // Test keep-alive during long analysis
        transport.send_comment("keep-alive").await.unwrap();
    }

    #[actix_web::test]
    async fn test_sse_structured_response() {
        let (transport, _responder) = ServerSseTransport::new_with_responder(100);

        // Send structured analysis response
        transport.send_event("analysis_start", "Beginning code analysis...").await.unwrap();

        transport.send_json(serde_json::json!({
            "response": {
                "metadata": {
                    "model": "code-llm-v1",
                    "timestamp": "2024-02-19T10:30:00Z",
                    "request_id": "req_789xyz",
                    "confidence_score": 0.92,
                    "processing_time_ms": 150
                },
                "context": {
                    "language": "python",
                    "file_type": "source_code",
                    "relevant_symbols": [
                        "UserRepository",
                        "authenticate_user",
                        "hash_password"
                    ],
                    "imports_detected": [
                        "bcrypt",
                        "sqlalchemy"
                    ]
                },
                "analysis": {
                    "code_quality": {
                        "score": 0.85,
                        "issues": [
                            {
                                "type": "security",
                                "severity": "medium",
                                "description": "Password hashing should use work factor of 12 or higher",
                                "line_number": 45,
                                "suggested_fix": "bcrypt.hashpw(password, bcrypt.gensalt(12))"
                            }
                        ]
                    },
                    "performance_insights": [
                        {
                            "type": "database",
                            "description": "Consider adding index on frequently queried user_email column",
                            "impact": "high",
                            "recommendation": "CREATE INDEX idx_user_email ON users(email);"
                        }
                    ]
                },
                "suggestions": {
                    "code_completions": [
                        {
                            "snippet": "def validate_password(password: str) -> bool:\n    return len(password) >= 8 and any(c.isupper() for c in password)",
                            "confidence": 0.88,
                            "context": "password validation helper function",
                            "tags": ["security", "validation", "user-input"]
                        }
                    ],
                    "refactoring_options": [
                        {
                            "type": "extract_method",
                            "description": "Extract password validation logic into separate function",
                            "before": "if len(password) >= 8 and any(c.isupper() for c in password):",
                            "after": "if validate_password(password):",
                            "benefit": "Improves code reusability and testability"
                        }
                    ]
                },
                "references": {
                    "documentation": [
                        {
                            "title": "BCrypt Best Practices",
                            "url": "https://example.com/bcrypt-guide",
                            "relevance_score": 0.95
                        }
                    ],
                    "similar_code_patterns": [
                        {
                            "repository": "auth-service",
                            "file": "auth/security.py",
                            "similarity_score": 0.82,
                            "matched_lines": [42, 43, 44]
                        }
                    ]
                },
                "execution_context": {
                    "memory_usage_mb": 245,
                    "tokens_processed": 1024,
                    "cache_hit_ratio": 0.76,
                    "embeddings_generated": 12
                }
            }
        })).await.unwrap();

        // Send follow-up events to demonstrate streaming with structured data
        transport.send_event("progress", "Generating code improvements...").await.unwrap();
        
        transport.send_event("suggestion", r#"{
            "type": "immediate_fix",
            "code": "bcrypt.hashpw(password, bcrypt.gensalt(12))",
            "priority": "high",
            "apply_to_line": 45
        }"#).await.unwrap();

        transport.send_event("analysis_complete", r#"{
            "summary": "Analysis complete. Found 1 security issue and 1 performance improvement.",
            "total_suggestions": 2,
            "execution_time_ms": 150
        }"#).await.unwrap();

        // Test keep-alive during long analysis
        transport.send_comment("keep-alive").await.unwrap();
    }
}
