use crate::{
    protocol::{Protocol, ProtocolBuilder, RequestOptions},
    transport::{Transport, TransportError},
    types::{
        ClientCapabilities, Implementation, InitializeRequest, InitializeResponse,
        LATEST_PROTOCOL_VERSION,
    },
};

use tracing::debug;

type Result<T> = std::result::Result<T, TransportError>;

/// A client for interacting with an MCP server.
///
/// The `Client` struct provides a high-level interface for communicating with MCP servers.
/// It handles protocol details, message formatting, and error handling, allowing you to
/// focus on the application logic.
///
/// # Examples
///
/// ```rust,no_run
/// use mcp_daemon::client::Client;
/// use mcp_daemon::transport::{ClientStdioTransport, Transport, TransportError};
/// use mcp_daemon::types::Implementation;
/// use mcp_daemon::protocol::RequestOptions;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a transport
/// let transport = ClientStdioTransport::new("mcp-server", &[])?;
/// transport.open().await?;
///
/// // Create a client
/// let client = Client::builder(transport.clone()).build();
///
/// // Initialize the client
/// let client_info = Implementation {
///     name: "example-client".to_string(),
///     version: "0.1.0".to_string(),
/// };
/// client.initialize(client_info).await?;
///
/// // Make a request
/// let response = client.request(
///     "tools/list",
///     None,
///     RequestOptions::default()
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
///
/// Most methods return a `Result` that will contain a `TransportError` if something
/// goes wrong. This could be due to connection issues, protocol errors, or server-side
/// problems.
#[derive(Clone)]
pub struct Client<T: Transport> {
    protocol: Protocol<T>,
}

impl<T: Transport> Client<T> {
    /// Creates a new `ClientBuilder` with the given transport.
    ///
    /// This is the recommended way to create a new `Client` instance.
    ///
    /// # Arguments
    ///
    /// * `transport` - The transport to use for communication with the server.
    ///
    /// # Returns
    ///
    /// A new `ClientBuilder` instance that can be used to configure and build a `Client`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mcp_daemon::client::Client;
    /// # use mcp_daemon::transport::ClientStdioTransport;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = ClientStdioTransport::new("mcp-server", &[])?;
    /// let client = Client::builder(transport).build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(transport: T) -> ClientBuilder<T> {
        ClientBuilder::new(transport)
    }

    /// Initializes the client with the server.
    ///
    /// This method sends an initialization request to the server and processes the response.
    /// It also sends a notification that the client has been initialized.
    ///
    /// # Arguments
    ///
    /// * `client_info` - Information about the client implementation.
    ///
    /// # Returns
    ///
    /// Returns the server's response to the initialization request.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The transport fails to send or receive messages
    /// * The server returns an error response
    /// * The server's protocol version is not compatible
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::client::Client;
/// # use mcp_daemon::transport::{ClientStdioTransport, Transport, TransportError};
/// # use mcp_daemon::types::Implementation;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = ClientStdioTransport::new("mcp-server", &[])?;
/// transport.open().await?;
/// let client = Client::builder(transport).build();
///
/// let client_info = Implementation {
///     name: "example-client".to_string(),
///     version: "0.1.0".to_string(),
/// };
/// let response = client.initialize(client_info).await?;
/// # Ok(())
/// # }
/// ```
    pub async fn initialize(&self, client_info: Implementation) -> Result<InitializeResponse> {
        let request = InitializeRequest {
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info,
        };
        let response = self
            .request(
                "initialize",
                Some(serde_json::to_value(request).map_err(TransportError::Json)?),
                RequestOptions::default(),
            )
            .await?;
        let response: InitializeResponse = serde_json::from_value(response)
            .map_err(TransportError::Json)?;

        if response.protocol_version != LATEST_PROTOCOL_VERSION {
            return Err(TransportError::new(
                crate::transport::TransportErrorCode::InvalidMessage,
                format!("Unsupported protocol version: {}", response.protocol_version),
            ));
        }

        debug!(
            "Initialized with protocol version: {}",
            response.protocol_version
        );
        self.protocol
            .notify("notifications/initialized", None)
            .await?;
        Ok(response)
    }

    /// Sends a request to the server.
    ///
    /// # Arguments
    ///
    /// * `method` - The method name for the request.
    /// * `params` - Optional parameters for the request.
    /// * `options` - Request options.
    ///
    /// # Returns
    ///
    /// Returns the server's response as a JSON value.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The transport fails to send or receive messages
    /// * The server returns an error response
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::client::Client;
/// # use mcp_daemon::transport::{ClientStdioTransport, Transport, TransportError};
/// # use mcp_daemon::protocol::RequestOptions;
/// # use serde_json::json;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let transport = ClientStdioTransport::new("mcp-server", &[])?;
/// # transport.open().await?;
/// # let client = Client::builder(transport).build();
/// // List available tools
/// let tools_response = client.request(
///     "tools/list",
///     None,
///     RequestOptions::default(),
/// ).await?;
///
/// // Call a specific tool
/// let call_response = client.request(
///     "tools/call",
///     Some(json!({
///         "name": "example-tool",
///         "arguments": {
///             "param1": "value1"
///         }
///     })),
///     RequestOptions::default(),
/// ).await?;
/// # Ok(())
/// # }
/// ```
    pub async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
        options: RequestOptions,
    ) -> Result<serde_json::Value> {
        let response = self.protocol.request(method, params, options).await?;
        response.result.ok_or_else(|| {
            TransportError::new(
                crate::transport::TransportErrorCode::InvalidMessage,
                format!("Request failed: {:?}", response.error),
            )
        })
    }

    /// Starts listening for messages from the server.
    ///
    /// This method should be called to begin processing incoming messages.
    /// It will run until the transport is closed or an error occurs.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transport is closed normally, or an error if
    /// something goes wrong.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The transport encounters an error while receiving messages
    /// * The protocol encounters an error while processing messages
    ///
    /// # Examples
    ///
/// ```rust,no_run
/// # use mcp_daemon::client::Client;
/// # use mcp_daemon::transport::{ClientStdioTransport, Transport, TransportError};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let transport = ClientStdioTransport::new("mcp-server", &[])?;
/// # transport.open().await?;
/// let client = Client::builder(transport).build();
/// let client_clone = client.clone();
///
/// // Start listening for messages in a separate task
/// tokio::spawn(async move {
///     if let Err(e) = client_clone.start().await {
///         eprintln!("Client error: {}", e);
///     }
/// });
///
/// // Use the client to make requests...
/// # Ok(())
/// # }
/// ```
    pub async fn start(&self) -> Result<()> {
        self.protocol.listen().await
    }
}

/// A builder for creating `Client` instances.
///
/// This builder provides a fluent interface for configuring and creating
/// a new `Client` instance.
///
/// # Examples
///
/// ```rust,no_run
/// # use mcp_daemon::client::Client;
/// # use mcp_daemon::transport::ClientStdioTransport;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = ClientStdioTransport::new("mcp-server", &[])?;
/// let client = Client::builder(transport).build();
/// # Ok(())
/// # }
/// ```
pub struct ClientBuilder<T: Transport> {
    protocol: ProtocolBuilder<T>,
}

impl<T: Transport> ClientBuilder<T> {
    /// Creates a new `ClientBuilder` with the given transport.
    ///
    /// # Arguments
    ///
    /// * `transport` - The transport to use for communication with the server.
    ///
    /// # Returns
    ///
    /// A new `ClientBuilder` instance.
    pub fn new(transport: T) -> Self {
        Self {
            protocol: ProtocolBuilder::new(transport),
        }
    }

    /// Builds and returns a new `Client` instance.
    ///
    /// # Returns
    ///
    /// A new `Client` instance configured with the settings from this builder.
    pub fn build(self) -> Client<T> {
        Client {
            protocol: self.protocol.build(),
        }
    }
}
