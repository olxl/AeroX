//! Simplified server builder for common use cases
//!
//! Provides a high-level API for building AeroX servers with minimal boilerplate.

use crate::{Error, Result};
use aerox_config::ServerConfig;
use aerox_core::{App, Plugin};
use aerox_router::{Context, Router};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Simplified server builder
///
/// Provides a fluent API for building servers with sensible defaults.
///
/// # Example
///
/// ```rust,no_run,ignore
/// use aerox::Server;
///
/// #[tokio::main]
/// async fn main() -> aerox::Result<()> {
///     Server::bind("127.0.0.1:8080")
///         .route(1001, |ctx| async move {
///             println!("Received: {:?}", ctx.data());
///             Ok(())
///         })
///         .run()
///         .await
/// }
/// ```
pub struct ServerBuilder {
    /// Server configuration
    config: ServerConfig,
    /// Message router (not wrapped in Arc during building)
    router: Router,
    /// Plugins to add
    plugins: Vec<Box<dyn Plugin>>,
}

impl ServerBuilder {
    /// Create a new server builder with default configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// let server = Server::new()
    ///     .route(1001, handler)
    ///     .run()
    ///     .await;
    /// ```
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            router: Router::new(),
            plugins: Vec::new(),
        }
    }

    /// Bind to a specific address
    ///
    /// # Arguments
    ///
    /// * `addr` - Address to bind to (e.g., "127.0.0.1:8080" or "0.0.0.0:9000")
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// let server = Server::bind("127.0.0.1:8080")
    ///     .route(1001, handler)
    ///     .run()
    ///     .await;
    /// ```
    pub fn bind(addr: impl Into<String>) -> Self {
        let addr_str = addr.into();
        let (bind_addr, port) = parse_addr(&addr_str);

        let mut config = ServerConfig::default();
        config.bind_address = bind_addr;
        config.port = port;

        Self {
            config,
            router: Router::new(),
            plugins: Vec::new(),
        }
    }

    /// Set custom server configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom [`ServerConfig`]
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// use aerox::Server;
    /// use aerox::ServerConfig;
    ///
    /// let config = ServerConfig {
    ///     bind_address: "0.0.0.0".to_string(),
    ///     port: 9000,
    ///     max_connections: Some(1000),
    ///     ..Default::default()
    /// };
    ///
    /// let server = Server::new()
    ///     .config(config)
    ///     .run()
    ///     .await;
    /// ```
    pub fn config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a message route handler
    ///
    /// # Arguments
    ///
    /// * `msg_id` - Message ID to handle
    /// * `handler` - Async handler function that takes a [`Context`]
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// Server::bind("127.0.0.1:8080")
    ///     .route(1001, |ctx| async move {
    ///         println!("Received message ID: {}", ctx.message_id());
    ///         println!("Data: {:?}", ctx.data());
    ///         Ok(())
    ///     })
    ///     .run()
    ///     .await;
    /// ```
    pub fn route<F>(mut self, msg_id: u16, handler: F) -> Self
    where
        F: Fn(Context) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        // Wrap the handler to convert from Error to AeroXError
        let wrapped_handler = move |ctx: Context| -> Pin<Box<dyn Future<Output = aerox_core::Result<()>> + Send>> {
            let result = handler(ctx);
            Box::pin(async move {
                result.await.map_err(|e| match e {
                    Error::Core(err) => err,
                    #[cfg(feature = "client")]
                    Error::Client(err) => aerox_core::AeroXError::network(err.to_string()),
                    #[cfg(feature = "server")]
                    Error::Config(err) => aerox_core::AeroXError::config(err.to_string()),
                    Error::Io(err) => aerox_core::AeroXError::network(err.to_string()),
                    Error::Custom(msg) => aerox_core::AeroXError::network(msg),
                })
            })
        };

        let _ = self.router.add_route(msg_id, wrapped_handler);
        self
    }

    /// Add a plugin to the server
    ///
    /// # Arguments
    ///
    /// * `plugin` - Plugin to add (must implement [`Plugin`])
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// use aerox::{Server, Plugin};
    /// use aerox_plugins::HeartbeatPlugin;
    ///
    /// Server::bind("127.0.0.1:8080")
    ///     .plugin(HeartbeatPlugin::default())
    ///     .run()
    ///     .await;
    /// ```
    pub fn plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Build and run the server
    ///
    /// This method consumes the builder and starts the server asynchronously.
    ///
    /// # Example
    ///
    /// ```rust,no_run,ignore
    /// Server::bind("127.0.0.1:8080")
    ///     .route(1001, handler)
    ///     .run()
    ///     .await?;
    /// ```
    pub async fn run(self) -> Result<()> {
        use aerox_config::ReactorConfig;
        use aerox_network::TcpReactor;
        use std::sync::Arc;

        println!("AeroX 服务器启动中...");
        println!("监听地址: {}", self.config.bind_addr());

        // Build the app with plugins
        let mut app = App::new().set_config(self.config.clone());

        for plugin in self.plugins {
            app = app.add_boxed_plugin(plugin);
        }

        // Build app (validates plugins and dependencies)
        let _app = app.build()?;

        // Wrap router in Arc for sharing across workers
        let router = Arc::new(self.router);

        // Create TcpReactor
        let reactor = TcpReactor::new(
            self.config,
            ReactorConfig::default(),
        );

        // Set router
        let reactor = reactor.with_router(router);

        // Start the reactor
        reactor.run().await?;

        Ok(())
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for convenience
pub type Server = ServerBuilder;

/// Parse address string into (host, port) tuple
///
/// # Arguments
///
/// * `addr` - Address string (e.g., "127.0.0.1:8080")
///
/// # Returns
///
/// A tuple of (host, port)
fn parse_addr(addr: &str) -> (String, u16) {
    if let Some((host, port)) = addr.split_once(':') {
        let port = port.parse().unwrap_or(8080);
        (host.to_string(), port)
    } else {
        (addr.to_string(), 8080)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_addr_with_port() {
        let (host, port) = parse_addr("127.0.0.1:8080");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_addr_without_port() {
        let (host, port) = parse_addr("127.0.0.1");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8080); // default port
    }

    #[test]
    fn test_server_builder_creation() {
        let builder = ServerBuilder::new();
        assert_eq!(builder.config.port, 8080);
    }

    #[test]
    fn test_server_builder_bind() {
        let builder = ServerBuilder::bind("127.0.0.1:9000");
        assert_eq!(builder.config.bind_address, "127.0.0.1");
        assert_eq!(builder.config.port, 9000);
    }
}
