//! # MCP Server Implementation
//!
//! This module provides a high-level API for creating and managing MCP servers.
//! It simplifies the process of registering resources, tools, prompts, and other
//! MCP capabilities.
//!
//! ## Overview
//!
//! The `McpServer` struct wraps the lower-level `Server` implementation and provides
//! a more ergonomic API for common server operations. It handles capability registration
//! automatically and maintains registries of resources, tools, and other components.
//!
//! Key features include:
//! * Simplified resource registration with automatic capability management
//! * Tool registration with builder pattern support
//! * Prompt registration and templating
//! * Support for sampling, completion, and roots capabilities
//!
//! ## Examples
//!
//! ```rust
//! use mcp_daemon::server::McpServer;
//! use mcp_daemon::types::Implementation;
//! use url::Url;
//!
//! // Create a new MCP server
//! let mut server = McpServer::new(Implementation {
//!     name: "example-server".to_string(),
//!     version: "1.0.0".to_string(),
//! });
//!
//! // Register a simple resource
//! server.resource(
//!     "Example Resource",
//!     "example://resource",
//!     None,
//!     |url| Box::pin(async move {
//!         Ok(vec![
//!             (url.clone(), "text/plain".to_string(), "Example content".to_string())
//!         ])
//!     })
//! );
//!
//! // Create and register a tool
//! let tool = server.tool_builder("example-tool")
//!     .description("An example tool")
//!     .parameter("param1", "string", "A parameter")
//!     .required("param1")
//!     .handler(|args| Box::pin(async move {
//!         // Tool implementation
//!         Ok(serde_json::json!({ "result": "success" }))
//!     }))
//!     .build();
//!
//! server.register_tool(tool.0, tool.1);
//!
//! // Connect to a transport
//! // server.connect(transport).await.unwrap();
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::server::resource`] - Resource handling
//! * [`crate::server::tool`] - Tool handling
//! * [`crate::server::prompt`] - Prompt handling
//! * [`crate::server::sampling`] - Sampling support
//! * [`crate::server::roots`] - Roots support

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use url::Url;

use crate::server::{
    completion::{CompletionCallback, RegisteredCompletion},
    error::ServerError,
    prompt::{PromptBuilder, RegisteredPrompt},
    resource::{RegisteredResource, RegisteredResourceTemplate, ResourceTemplate, ReadResourceResult, ReadResourceCallbackFn},
    roots::{RegisteredRoots, Root},
    sampling::{RegisteredSampling, SamplingRequest, SamplingResult},
    tool::{RegisteredTool, ToolBuilder},
    Server,
};
use crate::transport::Transport;
use crate::types::{Implementation, Prompt, Resource, ServerCapabilities, Tool};

type Result<T> = std::result::Result<T, ServerError>;

/// High-level MCP server that provides a simpler API for working with resources, tools, and prompts.
///
/// The `McpServer` struct wraps the lower-level `Server` implementation and provides
/// convenience methods for registering and managing various MCP capabilities.
///
/// # Fields
///
/// * `server` - The underlying Server instance
/// * `registered_resources` - Map of registered direct resources
/// * `registered_resource_templates` - Map of registered resource templates
/// * `registered_tools` - Map of registered tools
/// * `registered_prompts` - Map of registered prompts
/// * `registered_sampling` - Optional sampling handler
/// * `registered_roots` - Optional roots handler
/// * `registered_completion` - Optional completion handler
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::McpServer;
/// use mcp_daemon::types::Implementation;
///
/// let mut server = McpServer::new(Implementation {
///     name: "example-server".to_string(),
///     version: "1.0.0".to_string(),
/// });
///
/// // Register resources, tools, prompts, etc.
/// ```
pub struct McpServer {
    /// The underlying Server instance
    pub server: Server,

    registered_resources: HashMap<String, RegisteredResource>,
    registered_resource_templates: HashMap<String, RegisteredResourceTemplate>,
    registered_tools: HashMap<String, RegisteredTool>,
    registered_prompts: HashMap<String, RegisteredPrompt>,
    registered_sampling: Option<RegisteredSampling>,
    registered_roots: Option<RegisteredRoots>,
    registered_completion: Option<RegisteredCompletion>,
}

impl McpServer {
    /// Create a new MCP server with the given implementation info
    ///
    /// Initializes a new `McpServer` with the specified server name and version.
    ///
    /// # Arguments
    ///
    /// * `server_info` - Implementation details including name and version
    ///
    /// # Returns
    ///
    /// A new `McpServer` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    ///
    /// let server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    /// ```
    pub fn new(server_info: Implementation) -> Self {
        Self {
            server: Server::new(server_info),
            registered_resources: HashMap::new(),
            registered_resource_templates: HashMap::new(),
            registered_tools: HashMap::new(),
            registered_prompts: HashMap::new(),
            registered_sampling: None,
            registered_roots: None,
            registered_completion: None,
        }
    }

    /// Register a completion handler
    ///
    /// Registers a callback for handling completion requests and enables the completion capability.
    ///
    /// # Arguments
    ///
    /// * `handler` - The completion callback implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    /// use mcp_daemon::server::completion::CompletionRequest;
    ///
    /// let mut server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// server.register_completion(|request: CompletionRequest| {
    ///     // Handle completion request
    ///     Ok("Completion result".to_string())
    /// });
    /// ```
    pub fn register_completion(&mut self, handler: impl CompletionCallback + 'static) {
        self.registered_completion = Some(RegisteredCompletion {
            callback: Box::new(handler),
        });

        // Register completion capability
        self.server.register_capabilities(ServerCapabilities {
            completion: Some(Default::default()),
            ..Default::default()
        });
    }

    /// Register a sampling handler
    ///
    /// Registers a callback for handling sampling requests and enables the sampling capability.
    /// Sampling allows the server to request LLM completions through the client.
    ///
    /// # Arguments
    ///
    /// * `callback` - The sampling callback function
    /// * `timeout` - Optional timeout for sampling requests
    /// * `max_requests_per_minute` - Optional rate limit (requests per minute)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    /// use mcp_daemon::server::sampling::{SamplingRequest, SamplingResult, MessageRole, MessageContent};
    /// use std::time::Duration;
    ///
    /// let mut server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// server.register_sampling(
    ///     |request: SamplingRequest| {
    ///         Box::pin(async move {
    ///             // Process sampling request
    ///             Ok(SamplingResult {
    ///                 model: "model-name".to_string(),
    ///                 role: MessageRole::Assistant,
    ///                 content: MessageContent::Text {
    ///                     text: "I can help with that!".to_string(),
    ///                 },
    ///                 stop_reason: None,
    ///             })
    ///         })
    ///     },
    ///     Some(Duration::from_secs(30)), // 30-second timeout
    ///     Some(10),                      // Rate limit: 10 requests per minute
    /// );
    /// ```
    pub fn register_sampling(
        &mut self,
        callback: impl Fn(SamplingRequest) -> Pin<Box<dyn Future<Output = Result<SamplingResult>> + Send + 'static>>
            + Send
            + Sync
            + 'static,
        timeout: Option<Duration>,
        max_requests_per_minute: Option<usize>,
    ) {
        self.registered_sampling = Some(RegisteredSampling::new(
            callback,
            timeout,
            max_requests_per_minute,
        ));

        // Register sampling capability
        self.server.register_capabilities(ServerCapabilities {
            sampling: Some(Default::default()),
            ..Default::default()
        });
    }

    /// Register a roots handler
    ///
    /// Registers a callback for handling roots requests and enables the roots capability.
    /// Roots define the boundaries where the server can operate, typically filesystem paths.
    ///
    /// # Arguments
    ///
    /// * `list_callback` - Callback function that returns the list of available roots
    /// * `supports_change_notifications` - Whether the server supports notifications when roots change
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    /// use mcp_daemon::server::roots::Root;
    /// use url::Url;
    ///
    /// let mut server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// server.register_roots(
    ///     || Box::pin(async move {
    ///         Ok(vec![
    ///             Root {
    ///                 uri: Url::parse("file:///home/user/projects").unwrap(),
    ///                 name: Some("Projects".to_string()),
    ///             }
    ///         ])
    ///     }),
    ///     true, // Support change notifications
    /// );
    /// ```
    pub fn register_roots(
        &mut self,
        list_callback: impl Fn() -> Pin<Box<dyn Future<Output = Result<Vec<Root>>> + Send>>
            + Send
            + Sync
            + 'static,
        supports_change_notifications: bool,
    ) {
        let wrapped_callback = move || -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<Root>>> + Send>> {
            let fut = list_callback();
            Box::pin(async move { fut.await.map_err(|e| anyhow::anyhow!("{}", e)) })
        };

        self.registered_roots = Some(RegisteredRoots::new(
            wrapped_callback,
            supports_change_notifications,
        ));

        // Register roots capability
        self.server.register_capabilities(ServerCapabilities {
            roots: Some(Default::default()),
            ..Default::default()
        });
    }

    /// Connect to the given transport and start listening for messages
    ///
    /// Connects the server to a transport implementation and begins processing messages.
    ///
    /// # Arguments
    ///
    /// * `_transport` - The transport implementation to connect to
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    ///
    /// # Errors
    ///
    /// Returns a ServerError if the connection fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    /// use mcp_daemon::transport::stdio_transport::StdioServerTransport;
    ///
    /// async fn example() {
    ///     let server = McpServer::new(Implementation {
    ///         name: "example-server".to_string(),
    ///         version: "1.0.0".to_string(),
    ///     });
    ///
    ///     let transport = StdioServerTransport::new();
    ///     server.connect(transport).await.unwrap();
    /// }
    /// ```
    pub async fn connect(&self, _transport: impl Transport) -> Result<()> {
        self.server.connect(_transport).await
    }

    /// Register a resource at a fixed URI
    ///
    /// Registers a direct resource with a specific URI and callback for handling read requests.
    /// Automatically enables the resources capability if this is the first resource registered.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for the resource
    /// * `uri` - URI that uniquely identifies the resource
    /// * `metadata` - Optional additional resource metadata
    /// * `read_callback` - Callback function that handles read requests for this resource
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    /// use url::Url;
    ///
    /// let mut server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// server.resource(
    ///     "Example Resource",
    ///     "example://resource",
    ///     None,
    ///     |url| Box::pin(async move {
    ///         Ok(vec![
    ///             (url.clone(), "text/plain".to_string(), "Example content".to_string())
    ///         ])
    ///     })
    /// );
    /// ```
    pub fn resource(
        &mut self,
        name: impl Into<String>,
        uri: impl Into<String>,
        metadata: Option<Resource>,
        read_callback: impl Fn(&Url) -> Pin<Box<dyn Future<Output = ReadResourceResult> + Send + 'static>>
            + Send 
            + Sync
            + 'static,
    ) {
        let uri = uri.into();
        let name = name.into();

        let metadata = metadata.unwrap_or_else(|| Resource {
            uri: Url::parse(&uri).unwrap_or_else(|e| {
                eprintln!("Warning: Invalid URI '{}': {}", uri, e);
                Url::parse("about:invalid").unwrap()
            }),
            name: name.clone(),
            description: None,
            mime_type: None,
        });

        self.registered_resources.insert(
            uri.clone(),
            RegisteredResource::new(
                metadata,
                read_callback,
                false,
            ),
        );

        // Register capabilities if this is the first resource
        if self.registered_resources.len() == 1 {
            self.server.register_capabilities(ServerCapabilities {
                resources: Some(Default::default()),
                ..Default::default()
            });
        }
    }

    /// Register a resource template
    ///
    /// Registers a resource template that can match multiple URIs based on a pattern.
    /// Automatically enables the resources capability if this is the first resource template registered.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for the resource template
    /// * `template` - The resource template defining the URI pattern
    /// * `metadata` - Optional additional resource metadata
    /// * `read_callback` - Callback function that handles read requests for resources matching this template
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    /// use mcp_daemon::server::resource::ResourceTemplate;
    /// use url::Url;
    ///
    /// let mut server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// let template = ResourceTemplate::new("file://{path}");
    ///
    /// server.resource_template(
    ///     "File Resource",
    ///     template,
    ///     None,
    ///     |url| Box::pin(async move {
    ///         // Read file content based on URL
    ///         Ok(vec![
    ///             (url.clone(), "text/plain".to_string(), "File content".to_string())
    ///         ])
    ///     })
    /// );
    /// ```
    pub fn resource_template(
        &mut self,
        name: impl Into<String>,
        template: ResourceTemplate,
        metadata: Option<Resource>,
        read_callback: impl Fn(&Url) -> Pin<Box<dyn Future<Output = ReadResourceResult> + Send + 'static>>
            + Send
            + Sync
            + 'static,
    ) {
        let name = name.into();

        let metadata = metadata.unwrap_or_else(|| Resource {
            uri: Url::parse(template.uri_template()).unwrap_or_else(|e| {
                eprintln!("Warning: Invalid URI template: {}", e);
                Url::parse("about:invalid").unwrap()
            }),
            name: name.clone(),
            description: None,
            mime_type: None,
        });

        self.registered_resource_templates.insert(
            name,
            RegisteredResourceTemplate {
                template,
                metadata,
                read_callback: Arc::new(ReadResourceCallbackFn(Box::new(read_callback))),
            },
        );

        // Register capabilities if this is the first resource template
        if self.registered_resource_templates.len() == 1 {
            self.server.register_capabilities(ServerCapabilities {
                resources: Some(Default::default()),
                ..Default::default()
            });
        }
    }

    /// Create a new prompt builder
    ///
    /// Returns a new PromptBuilder for creating and configuring a prompt.
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
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    ///
    /// let server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// let prompt_builder = server.prompt_builder("example-prompt")
    ///     .description("An example prompt")
    ///     .argument("name", "User's name", true);
    ///
    /// // Continue building and register the prompt
    /// ```
    pub fn prompt_builder(&self, name: impl Into<String>) -> PromptBuilder {
        PromptBuilder::new(name)
    }

    /// Register a prompt
    ///
    /// Registers a prompt with the server and enables the prompts capability if needed.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Prompt metadata
    /// * `registered` - The registered prompt implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    ///
    /// let mut server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// let prompt = server.prompt_builder("greeting")
    ///     .description("A greeting prompt")
    ///     .argument("name", "User's name", true)
    ///     .handler(|args| {
    ///         let name = args.get("name").unwrap_or("friend");
    ///         Ok(vec![
    ///             ("user".to_string(), format!("Hello, {}!", name))
    ///         ])
    ///     })
    ///     .build();
    ///
    /// server.register_prompt(prompt.0, prompt.1);
    /// ```
    pub fn register_prompt(&mut self, metadata: impl Into<Prompt>, registered: RegisteredPrompt) {
        let metadata = metadata.into();
        self.registered_prompts
            .insert(metadata.name.clone(), registered);

        // Register capabilities if this is the first prompt
        if self.registered_prompts.len() == 1 {
            self.server.register_capabilities(ServerCapabilities {
                prompts: Some(Default::default()),
                ..Default::default()
            });
        }
    }

    /// Create a new tool builder
    ///
    /// Returns a new ToolBuilder for creating and configuring a tool.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool
    ///
    /// # Returns
    ///
    /// A new ToolBuilder instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    ///
    /// let server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// let tool_builder = server.tool_builder("example-tool")
    ///     .description("An example tool")
    ///     .parameter("param1", "string", "A parameter");
    ///
    /// // Continue building and register the tool
    /// ```
    pub fn tool_builder(&self, name: impl Into<String>) -> ToolBuilder {
        ToolBuilder::new(name)
    }

    /// Register a tool
    ///
    /// Registers a tool with the server and enables the tools capability if needed.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Tool metadata
    /// * `registered` - The registered tool implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::McpServer;
    /// use mcp_daemon::types::Implementation;
    ///
    /// let mut server = McpServer::new(Implementation {
    ///     name: "example-server".to_string(),
    ///     version: "1.0.0".to_string(),
    /// });
    ///
    /// let tool = server.tool_builder("calculator")
    ///     .description("A simple calculator")
    ///     .parameter("operation", "string", "Operation to perform")
    ///     .parameter("a", "number", "First operand")
    ///     .parameter("b", "number", "Second operand")
    ///     .required("operation")
    ///     .required("a")
    ///     .required("b")
    ///     .handler(|args| Box::pin(async move {
    ///         // Tool implementation
    ///         Ok(serde_json::json!({ "result": 42 }))
    ///     }))
    ///     .build();
    ///
    /// server.register_tool(tool.0, tool.1);
    /// ```
    pub fn register_tool(&mut self, metadata: impl Into<Tool>, registered: RegisteredTool) {
        let metadata = metadata.into();
        self.registered_tools.insert(metadata.name.clone(), registered);

        // Register capabilities if this is the first tool
        if self.registered_tools.len() == 1 {
            self.server.register_capabilities(ServerCapabilities {
                tools: Some(Default::default()),
                ..Default::default()
            });
        }
    }
}
