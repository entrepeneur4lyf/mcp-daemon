# mcp-daemon

![Crates.io Total Downloads](https://img.shields.io/crates/d/mcp_daemon?style=plastic&link=https%3A%2F%2Fcrates.io%2Fcrates%2Fmcp_daemon)![docs.rs](https://img.shields.io/docsrs/mcp_daemon%2F0.2.1?style=plastic&link=https%3A%2F%2Fdocs.rs%2Fcrate%2Fmcp_daemon%2F0.2.1)

## Overview

The most advanced and complete implementation of the Model Context Protocol (MCP) specification. This Rust implementation goes beyond the standard specification to provide:

- **Full Specification Coverage**: Implements every feature from the latest MCP spec
- **Production-Grade Error Handling**: Comprehensive error system with recovery mechanisms
- **Advanced Transport Layer**: Robust implementations of all transport types with detailed error tracking
- **Type-Safe Architecture**: Leveraging Rust's type system for compile-time correctness
- **Real-World Ready**: Production-tested with Claude Desktop compatibility

This library sets the standard for MCP implementations with its comprehensive feature set and robust error handling.

> **Note**: While this implementation provides the most complete coverage of the MCP specification, including features like sampling, roots, and completion that are not yet available in other implementations, it is still under active development.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mcp_daemon = "0.2.1"
```

## Overview
This is an implementation of the [Model Context Protocol](https://github.com/modelcontextprotocol) defined by Anthropic, with extensive modifications and improvements.

## Repository

The mcp-daemon repository can be found at: https://github.com/entrepeneur4lyf/mcp-daemon

## Features

### Supported Transports

#### HTTP-Based Transports
- **Server-Sent Events (SSE)**: Robust unidirectional server-to-client communication with automatic keep-alive
- **WebSocket**: Full-duplex communication with comprehensive error handling and connection management

#### Other Transports
- **Standard IO (Stdio)**: For command-line and process-based communication
- **In-Memory Channel**: Efficient inter-process communication using Tokio channels

## Transport Implementation Details

### HTTP Transport Layer
The HTTP transport layer provides a unified interface for both SSE and WebSocket connections:

#### Server-Side Events (SSE)
- Efficient unidirectional communication from server to client
- Automatic keep-alive with configurable intervals (default: 15 seconds)
- Comprehensive event types support (data, named events, comments)
- JSON serialization for structured messages
- Built on `actix-web-lab` for robust server implementation

#### WebSocket Transport
- Full-duplex communication with message broadcasting
- Header customization support for authentication and session management
- Robust error handling with specific error codes and messages
- Connection lifecycle management (open, close, reconnect)
- Built on `tokio-tungstenite` for async WebSocket support

#### Common Features
- Type-safe message handling using Rust's type system
- Comprehensive error handling with custom error types
- Async/await support throughout the transport layer
- Clean separation between transport types via enum variants

#### Security Features
- **TLS Support**
  - Secure communication with configurable TLS certificates
  - Custom certificate and key path configuration
  - Optional TLS for development environments

- **CORS Configuration**
  - Fine-grained Cross-Origin Resource Sharing control
  - Configurable allowed origins and credentials
  - Customizable preflight cache duration
  - Header allowlist support
  - Default secure CORS policy

## Usage Examples

### OpenAI Function Call Bridge
The implementation provides a seamless bridge between MCP tools and LLM provider function calling formats:

#### Bridge Architecture
The bridge layer automatically handles conversion between MCP tools and provider-specific formats:
- Users only need to define tools using the MCP format
- Bridge handles all provider-specific conversions internally
- No need to know or work with provider-specific formats

#### Function Call Format
- Automatic conversion of MCP tools to OpenAI function format
- Handles function definitions, parameter validation, and response formatting
- Supports strict mode with proper JSON schema validation
- Type-safe conversion with comprehensive error handling

#### Provider Support
- **OpenAI Integration**
  - Full support for all tool choice options (auto/required/none/specific)
  - Parallel function calling support
  - Streaming support for real-time function calls

- **Ollama Integration**
  - Automatic adaptation to Ollama's capabilities
  - Handles Ollama's "auto-only" tool choice limitation
  - Function call extraction with regex-based parsing
  - Compatible response formatting

The bridge abstracts away provider differences, allowing you to write provider-agnostic code while the bridge handles the specific requirements and limitations of each LLM provider.

### Server Implementation

#### Using Stdio Transport
```rust
let server = Server::builder(StdioTransport)
    .capabilities(ServerCapabilities {
        tools: Some(json!({})),
        ..Default::default()
    })
    .request_handler("tools/list", list_tools)
    .request_handler("tools/call", call_tool)
    .request_handler("resources/list", |_req: ListRequest| {
        Ok(ResourcesListResponse {
            resources: vec![],
            next_cursor: None,
            meta: None,
        })
    })
    .build();
```

#### Run HTTP Server with Advanced Configuration
```rust
// Server configuration with TLS and CORS
let config = ServerConfig {
    port: 3004,
    cors: Some(CorsConfig {
        allowed_origin: "https://example.com".to_string(),
        allow_credentials: true,
        max_age: Some(3600),
    }),
    tls: Some(TlsConfig {
        cert_path: "path/to/cert.pem".to_string(),
        key_path: "path/to/key.pem".to_string(),
    }),
    ..Default::default()
};

// Run server with configuration
run_http_server(config, None, |transport| async move {
    let server = build_server(transport);
    Ok(server)
})
.await?;
```

Local Endpoints
```
# With TLS enabled:
WebSocket endpoint: wss://127.0.0.1:3004/ws
SSE endpoint: https://127.0.0.1:3004/sse

# Without TLS:
WebSocket endpoint: ws://127.0.0.1:3004/ws
SSE endpoint: http://127.0.0.1:3004/sse
```

##### Security Features
- **TLS Support**: Secure communication with TLS certificate support
- **CORS Configuration**: Fine-grained control over Cross-Origin Resource Sharing
  - Origin restrictions
  - Credential handling
  - Preflight caching
  - Header allowlists
- **JWT Authentication**: Optional JWT-based authentication for endpoints

### Client Implementation

#### Setting up Transport
```rust
// Stdio Transport
let transport = ClientStdioTransport::new("<CMD>", &[])?;

// In-Memory Transport
let transport = ClientInMemoryTransport::new(|t| tokio::spawn(inmemory_server(t)));

// SSE Transport
let transport = ClientSseTransportBuilder::new(server_url).build();

// WS Transport
let transport = mcp-daemon::transport::ClientWsTransportBuilder::new("ws://localhost:3004/ws".to_string()).build();
```

#### Making Requests
```rust
// Initialize transport
transport.open().await?;

// Create and start client
let client = mcp-daemon::client::ClientBuilder::new(transport.clone()).build();
let client_clone = client.clone();
let _client_handle = tokio::spawn(async move { client_clone.start().await });

// Make a request
client
    .request(
        "tools/call",
        Some(json!({"name": "ping", "arguments": {}})),
        RequestOptions::default().timeout(Duration::from_secs(5)),
    )
    .await?
```

## Using MCP Servers

### Installing Available Servers
MCP servers can be installed and run using npm. For example:
```bash
# Install and run Brave Search MCP server
npx -y @modelcontextprotocol/server-brave-search

# Install and run GitHub MCP server
npx -y @modelcontextprotocol/server-github

# Install and run NPM Search server
npx -y npm-search-mcp-server
```

### Connecting to Servers
Once servers are running, you can connect to them using the client:

```rust
// Example: Using Brave Search server
let transport = ClientSseTransportBuilder::new("http://localhost:3000/sse").build();
let client = mcp-daemon::client::ClientBuilder::new(transport.clone()).build();

// Make a search request
let response = client
    .request(
        "tools/call",
        Some(json!({
            "name": "brave_web_search",
            "arguments": {
                "query": "Rust programming language",
                "count": 5
            }
        })),
        RequestOptions::default(),
    )
    .await?;
```

### Available Servers
Common MCP servers include:
- **Brave Search**: Web search capabilities
- **GitHub**: Repository management and code search
- **NPM Search**: Package search and metadata
- **File System**: Local file operations
- **Memory**: Knowledge graph and data persistence
- **Weather**: Weather data and forecasts
- **Playwright**: Browser automation and testing

Each server provides its own set of tools and resources that can be used through the MCP protocol. Check individual server documentation for specific capabilities and usage details.

## Related SDKs

### Official
- [TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk)
- [Python SDK](https://github.com/modelcontextprotocol/python-sdk)

### Community
- [Go SDK](https://github.com/mark3labs/mcp-go)

For the complete feature set, please refer to the [MCP specification](https://spec.modelcontextprotocol.io/).

## Implementation Status

### Core Protocol Features
- [x] Basic Message Types
- [x] Error and Signal Handling
  - [x] JSON-RPC Error Codes
  - [x] Error Data Support
  - [x] Graceful Shutdown
  - [x] Signal Handlers
  - [x] Transport-specific Errors
- [x] Transport Layer
  - [x] Stdio (with error handling)
  - [x] In-Memory Channel (with error handling)
  - [x] SSE (with error handling)
  - [x] Websockets (with error handling)
  - [x] Detailed Error Codes
  - [x] Error Recovery

### Server Features
- [x] Tools Support
- [x] Prompts Support
  - [x] Arguments
  - [x] Templates
  - [x] List Changed Notifications
- [x] Resources Support
  - [x] Pagination
  - [x] Templates
  - [x] Subscriptions
  - [x] Update Notifications
- [x] Completion Support
  - [x] Resource Completion
  - [x] Prompt Completion
  - [x] Argument Completion
- [x] Sampling Support
  - [x] Model Preferences
  - [x] Context Inclusion
  - [x] System Prompts
- [x] Roots Support
  - [x] URI-based Roots
  - [x] Change Notifications

### Client Features
- [x] Claude Desktop Support
  - [x] Stdio Transport
  - [x] In-Memory Channel
  - [x] SSE Support
  - [x] Websocket Support
- [x] MCP Bridge Protocol
  - [x] Tool Registration Format
  - [x] Tool Execution Format
  - [x] Tool Response Format
  - [x] Message Conversion
  - [x] Error Handling

### Notification Support
- [x] Resource Updates
- [x] Resource List Changes
- [x] Tool List Changes
- [x] Prompt List Changes
- [x] Roots List Changes
- [x] Progress Updates
- [x] Cancellation

### Monitoring
- [x] Logging Support
  - [x] Level Control
  - [x] Message Notifications
- [ ] Metrics

### Utilities
- [x] Cancellation Support
- [x] Progress Tracking
  - [x] Progress Notifications
  - [x] Progress Tokens
  - [x] Progress Values

## License

This project is licensed under the Apache License 2.0, as was the original async-mcp project.

## Acknowledgements

Special thanks to the original author of async-mcp, [v3g42](https://github.com/v3g42), for laying the foundation for this project.
