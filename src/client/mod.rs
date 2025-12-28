//! High-level client API
//!
//! Provides a simplified client interface for common use cases.

use crate::{Error, Result};
use aerox_client::{HighLevelClient as InnerClient, StreamClient as InnerStream};
use aerox_client::Result as ClientResult;
use std::future::Future;
use std::pin::Pin;

/// High-level client with automatic message handling
///
/// This is a simplified wrapper around [`aerox_client::HighLevelClient`] that provides
/// a cleaner API for common client operations.
///
/// # Example
///
/// ```rust,no_run,ignore
/// use aerox::Client;
///
/// #[tokio::main]
/// async fn main() -> aerox::Result<()> {
///     let mut client = Client::connect("127.0.0.1:8080").await?;
///
///     // Register message handler
///     client.on_message(1001, |id, msg: MyMessage| async move {
///         println!("Received: {:?}", msg);
///         Ok(())
///     }).await?;
///
///     // Send message
///     client.send(1001, &my_message).await?;
///
///     Ok(())
/// }
/// ```
pub struct Client {
    inner: InnerClient,
}

impl Client {
    /// Connect to a server
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "127.0.0.1:8080")
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// let client = Client::connect("127.0.0.1:8080").await?;
    /// ```
    pub async fn connect(addr: impl Into<String>) -> Result<Self> {
        let addr_str = addr.into();
        let socket_addr: std::net::SocketAddr = addr_str.parse().map_err(|e| {
            Error::Custom(format!("Invalid address '{}': {}", addr_str, e))
        })?;

        let inner = InnerClient::connect(socket_addr)
            .await
            .map_err(Error::from)?;

        Ok(Self { inner })
    }

    /// Register a message handler
    ///
    /// # Arguments
    ///
    /// * `msg_id` - Message ID to handle
    /// * `f` - Async handler function
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// client.on_message(1001, |id, msg: MyMessage| async move {
    ///     println!("Received message {}: {:?}", id, msg);
    ///     Ok(())
    /// }).await?;
    /// ```
    pub async fn on_message<M, F>(&mut self, msg_id: u16, f: F) -> Result<()>
    where
        M: prost::Message + Default + Send + 'static,
        F: Fn(u16, M) -> Pin<Box<dyn Future<Output = ClientResult<()>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.inner
            .on_message(msg_id, f)
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    /// Send a message
    ///
    /// # Arguments
    ///
    /// * `msg_id` - Message ID
    /// * `msg` - Message to send (must implement [`prost::Message`])
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// client.send(1001, &my_message).await?;
    /// ```
    pub async fn send<M: prost::Message>(&mut self, msg_id: u16, msg: &M) -> Result<()> {
        self.inner.send(msg_id, msg).await.map_err(Error::from)
    }

    /// Check if connected to the server
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// if client.is_connected().await {
    ///     println!("Still connected");
    /// }
    /// ```
    pub async fn is_connected(&self) -> bool {
        self.inner.is_connected().await
    }

    /// Get the inner client for advanced use cases
    pub fn inner(&self) -> &InnerClient {
        &self.inner
    }

    /// Get a mutable reference to the inner client for advanced use cases
    pub fn inner_mut(&mut self) -> &mut InnerClient {
        &mut self.inner
    }

    /// Consume and return the inner client
    pub fn into_inner(self) -> InnerClient {
        self.inner
    }
}

/// Low-level stream client
///
/// This is a re-export of [`aerox_client::StreamClient`] for users who need
/// fine-grained control over message handling.
///
/// # Example
///
/// ```rust,no_run,ignore
/// use aerox::StreamClient;
///
/// #[tokio::main]
/// async fn main() -> aerox::Result<()> {
///     let mut client = StreamClient::connect("127.0.0.1:8080").await?;
///
///     // Manual send/receive loop
///     client.send_message(1001, &msg).await?;
///     let (msg_id, response) = client.recv_message::<Response>().await?;
///
///     client.close().await?;
///     Ok(())
/// }
/// ```
pub use aerox_client::StreamClient;
