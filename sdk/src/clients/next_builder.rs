use crate::clients::next_client::{IggyClientNext, IggyClientNextBackgroundConfig};
use crate::error::IggyError;
use crate::http::client::HttpClient;
use crate::http::config::HttpClientConfigBuilder;
use crate::message_handler::MessageHandler;
use crate::next_client::ClientNext;
use crate::partitioner::Partitioner;
use crate::quic::client::QuicClient;
use crate::quic::config::QuicClientConfigBuilder;
use crate::tcp::client::TcpClient;
use crate::tcp::config::TcpClientConfigBuilder;
use crate::utils::crypto::Encryptor;
use std::sync::Arc;
use tracing::error;

/// The next version builder for the `IggyClient` instance, which allows to configure and provide custom implementations for the partitioner, encryptor or message handler.
#[derive(Debug, Default)]
pub struct IggyClientNextBuilder {
    client: Option<Box<dyn ClientNext>>,
    background_config: Option<IggyClientNextBackgroundConfig>,
    partitioner: Option<Box<dyn Partitioner>>,
    encryptor: Option<Box<dyn Encryptor>>,
    message_handler: Option<Box<dyn MessageHandler>>,
}

impl IggyClientNextBuilder {
    /// Creates a new `IggyClientNextBuilder`.
    /// This is not enough to build the `IggyClient` instance. You need to provide the client configuration or the client implementation for the specific transport.
    pub fn new() -> Self {
        IggyClientNextBuilder::default()
    }

    /// Apply the provided client implementation for the specific transport. Setting client clears the client config.
    pub fn with_client(mut self, client: Box<dyn ClientNext>) -> Self {
        self.client = Some(client);
        self
    }

    /// Use the custom partitioner implementation.
    pub fn with_partitioner(mut self, partitioner: Box<dyn Partitioner>) -> Self {
        self.partitioner = Some(partitioner);
        self
    }

    /// Apply the provided background configuration.
    pub fn with_background_config(
        mut self,
        background_config: IggyClientNextBackgroundConfig,
    ) -> Self {
        self.background_config = Some(background_config);
        self
    }

    /// Use the custom encryptor implementation.
    pub fn with_encryptor(mut self, encryptor: Box<dyn Encryptor>) -> Self {
        self.encryptor = Some(encryptor);
        self
    }

    /// Use the custom message handler implementation. This handler will be used only for `start_polling_messages` method, if neither `subscribe_to_polled_messages` (which returns the receiver for the messages channel) is called nor `on_message` closure is provided.
    pub fn with_message_handler(mut self, message_handler: Box<dyn MessageHandler>) -> Self {
        self.message_handler = Some(message_handler);
        self
    }

    /// This method provides fluent API for the TCP client configuration.
    /// It returns the `TcpClientBuilder` instance, which allows to configure the TCP client with custom settings or using defaults.
    /// This should be called after the non-protocol specific methods, such as `with_partitioner`, `with_encryptor` or `with_message_handler`.
    pub fn with_tcp(self) -> TcpClientNextBuilder {
        TcpClientNextBuilder {
            config: TcpClientConfigBuilder::default(),
            parent_builder: self,
        }
    }

    /// This method provides fluent API for the QUIC client configuration.
    /// It returns the `QuicClientBuilder` instance, which allows to configure the QUIC client with custom settings or using defaults.
    /// This should be called after the non-protocol specific methods, such as `with_partitioner`, `with_encryptor` or `with_message_handler`.
    pub fn with_quic(self) -> QuicClientNextBuilder {
        QuicClientNextBuilder {
            config: QuicClientConfigBuilder::default(),
            parent_builder: self,
        }
    }

    /// This method provides fluent API for the HTTP client configuration.
    /// It returns the `HttpClientBuilder` instance, which allows to configure the HTTP client with custom settings or using defaults.
    /// This should be called after the non-protocol specific methods, such as `with_partitioner`, `with_encryptor` or `with_message_handler`.
    pub fn with_http(self) -> HttpClientNextBuilder {
        HttpClientNextBuilder {
            config: HttpClientConfigBuilder::default(),
            parent_builder: self,
        }
    }

    /// Build the `IggyClientNext` instance.
    /// This method returns an error if the client is not provided.
    /// If the client is provided, it creates the `IggyClient` instance with the provided configuration.
    /// To provide the client configuration, use the `with_tcp`, `with_quic` or `with_http` methods.
    pub fn build(self) -> Result<IggyClientNext, IggyError> {
        let Some(client) = self.client else {
            error!("Client is not provided");
            return Err(IggyError::InvalidConfiguration);
        };

        Ok(IggyClientNext::create(
            client,
            self.background_config.unwrap_or_default(),
            self.message_handler,
            self.partitioner,
            self.encryptor,
        ))
    }
}

#[derive(Debug, Default)]
pub struct TcpClientNextBuilder {
    config: TcpClientConfigBuilder,
    parent_builder: IggyClientNextBuilder,
}

impl TcpClientNextBuilder {
    /// Sets the server address for the TCP client.
    pub fn with_server_address(mut self, server_address: String) -> Self {
        self.config = self.config.with_server_address(server_address);
        self
    }

    /// Sets the number of retries when connecting to the server.
    pub fn with_reconnection_retries(mut self, reconnection_retries: u32) -> Self {
        self.config = self.config.with_reconnection_retries(reconnection_retries);
        self
    }

    /// Sets the interval between retries when connecting to the server.
    pub fn with_reconnection_interval(mut self, reconnection_interval: u64) -> Self {
        self.config = self
            .config
            .with_reconnection_interval(reconnection_interval);
        self
    }

    /// Sets whether to use TLS when connecting to the server.
    pub fn with_tls_enabled(mut self, tls_enabled: bool) -> Self {
        self.config = self.config.with_tls_enabled(tls_enabled);
        self
    }

    /// Sets the domain to use for TLS when connecting to the server.
    pub fn with_tls_domain(mut self, tls_domain: String) -> Self {
        self.config = self.config.with_tls_domain(tls_domain);
        self
    }

    /// Builds the parent `IggyClientNext` with TCP configuration.
    pub fn build(self) -> Result<IggyClientNext, IggyError> {
        let client = TcpClient::create(Arc::new(self.config.build()))?;
        let client = self.parent_builder.with_client(Box::new(client)).build()?;
        Ok(client)
    }
}

#[derive(Debug, Default)]
pub struct QuicClientNextBuilder {
    config: QuicClientConfigBuilder,
    parent_builder: IggyClientNextBuilder,
}

impl QuicClientNextBuilder {
    /// Sets the server address for the QUIC client.
    pub fn with_server_address(mut self, server_address: String) -> Self {
        self.config = self.config.with_server_address(server_address);
        self
    }

    /// Sets the number of retries when connecting to the server.
    pub fn with_reconnection_retries(mut self, reconnection_retries: u32) -> Self {
        self.config = self.config.with_reconnection_retries(reconnection_retries);
        self
    }

    /// Sets the interval between retries when connecting to the server.
    pub fn with_reconnection_interval(mut self, reconnection_interval: u64) -> Self {
        self.config = self
            .config
            .with_reconnection_interval(reconnection_interval);
        self
    }

    /// Sets the server name for the QUIC client.
    pub fn with_server_name(mut self, server_name: String) -> Self {
        self.config = self.config.with_server_name(server_name);
        self
    }

    /// Builds the parent `IggyClientNext` with QUIC configuration.
    pub fn build(self) -> Result<IggyClientNext, IggyError> {
        let client = QuicClient::create(Arc::new(self.config.build()))?;
        let client = self.parent_builder.with_client(Box::new(client)).build()?;
        Ok(client)
    }
}

#[derive(Debug, Default)]
pub struct HttpClientNextBuilder {
    config: HttpClientConfigBuilder,
    parent_builder: IggyClientNextBuilder,
}

impl HttpClientNextBuilder {
    /// Sets the server address for the HTTP client.
    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.config = self.config.with_api_url(api_url);
        self
    }

    /// Sets the number of retries for the HTTP client.
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.config = self.config.with_retries(retries);
        self
    }

    /// Builds the parent `IggyClientNext` with HTTP configuration.
    pub fn build(self) -> Result<IggyClientNext, IggyError> {
        let client = HttpClient::create(Arc::new(self.config.build()))?;
        let client = self.parent_builder.with_client(Box::new(client)).build()?;
        Ok(client)
    }
}