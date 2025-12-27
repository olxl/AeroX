//! Stream-based client
//!
//! Provides low-level, manual control over message send/receive operations.

use crate::config::ClientConfig;
use crate::connection::ClientConnection;
use crate::error::Result;
use aerox_network::Frame;
use std::net::SocketAddr;

/// Stream-based client
///
/// Provides manual control over message operations. Users explicitly call
/// send/receive methods.
pub struct StreamClient {
    connection: ClientConnection,
}

impl StreamClient {
    /// Connect to server
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let config = ClientConfig::new(addr);
        Self::connect_with_config(config).await
    }

    /// Connect with custom configuration
    pub async fn connect_with_config(config: ClientConfig) -> Result<Self> {
        let connection = ClientConnection::connect(&config).await?;
        Ok(Self { connection })
    }

    /// Send raw frame
    pub async fn send_frame(&mut self, frame: Frame) -> Result<()> {
        self.connection.send_frame(frame).await
    }

    /// Send protobuf message
    pub async fn send_message<M: prost::Message>(
        &mut self,
        msg_id: u16,
        message: &M,
    ) -> Result<()> {
        self.connection.send_message(msg_id, message).await
    }

    /// Receive next frame (blocking)
    pub async fn recv_frame(&mut self) -> Result<Frame> {
        self.connection.recv_frame().await
    }

    /// Receive and decode protobuf message
    pub async fn recv_message<M: prost::Message + Default>(
        &mut self,
    ) -> Result<(u16, M)> {
        self.connection.recv_message().await
    }

    /// Get connection state
    pub async fn state(&self) -> crate::connection::ClientState {
        self.connection.state().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.connection.is_connected().await
    }

    /// Get server address
    pub fn server_addr(&self) -> SocketAddr {
        self.connection.server_addr()
    }

    /// Close connection
    pub async fn close(self) -> Result<()> {
        self.connection.close().await
    }

    /// Get connection reference (for advanced usage)
    pub fn connection(&mut self) -> &mut ClientConnection {
        &mut self.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_client_create() {
        // Test that we can create a StreamClient struct
        // (actual connection tests require a running server)
        let _ = || -> StreamClient {
            // This is a compile-time test
            unimplemented!()
        };
    }
}
