//! Client configuration

use std::net::SocketAddr;
use std::time::Duration;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Server address to connect to
    pub server_addr: SocketAddr,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Enable auto-reconnect
    pub auto_reconnect: bool,

    /// Reconnect delay
    pub reconnect_delay: Duration,

    /// Max reconnect attempts (None = infinite)
    pub max_reconnect_attempts: Option<usize>,

    /// Read buffer size
    pub read_buffer_size: usize,

    /// Write buffer size
    pub write_buffer_size: usize,

    /// Heartbeat interval
    pub heartbeat_interval: Option<Duration>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:8080".parse().unwrap(),
            connect_timeout: Duration::from_secs(5),
            auto_reconnect: false,
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_attempts: None,
            read_buffer_size: 8192,
            write_buffer_size: 8192,
            heartbeat_interval: None,
        }
    }
}

impl ClientConfig {
    /// Create new client config
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            server_addr,
            ..Default::default()
        }
    }

    /// Set connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Enable auto-reconnect
    pub fn with_auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set reconnect delay
    pub fn with_reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    /// Set max reconnect attempts
    pub fn with_max_reconnect_attempts(mut self, max: Option<usize>) -> Self {
        self.max_reconnect_attempts = max;
        self
    }

    /// Set read buffer size
    pub fn with_read_buffer_size(mut self, size: usize) -> Self {
        self.read_buffer_size = size;
        self
    }

    /// Set write buffer size
    pub fn with_write_buffer_size(mut self, size: usize) -> Self {
        self.write_buffer_size = size;
        self
    }

    /// Set heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Option<Duration>) -> Self {
        self.heartbeat_interval = interval;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.server_addr, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert!(!config.auto_reconnect);
    }

    #[test]
    fn test_config_builder() {
        let config = ClientConfig::new("127.0.0.1:9000".parse().unwrap())
            .with_connect_timeout(Duration::from_secs(10))
            .with_auto_reconnect(true)
            .with_reconnect_delay(Duration::from_secs(2));

        assert_eq!(config.server_addr, "127.0.0.1:9000".parse().unwrap());
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert!(config.auto_reconnect);
        assert_eq!(config.reconnect_delay, Duration::from_secs(2));
    }
}
