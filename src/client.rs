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
#[derive(Clone)]
pub struct Client<T: Transport> {
    protocol: Protocol<T>,
}

impl<T: Transport> Client<T> {
    /// Creates a new `ClientBuilder` with the given transport.
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
    pub async fn start(&self) -> Result<()> {
        self.protocol.listen().await
    }
}

/// A builder for creating `Client` instances.
pub struct ClientBuilder<T: Transport> {
    protocol: ProtocolBuilder<T>,
}

impl<T: Transport> ClientBuilder<T> {
    /// Creates a new `ClientBuilder` with the given transport.
    pub fn new(transport: T) -> Self {
        Self {
            protocol: ProtocolBuilder::new(transport),
        }
    }

    /// Builds and returns a new `Client` instance.
    pub fn build(self) -> Client<T> {
        Client {
            protocol: self.protocol.build(),
        }
    }
}
