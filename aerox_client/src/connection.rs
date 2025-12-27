//! Client connection management

use crate::config::ClientConfig;
use crate::error::{ClientError, Result};
use aerox_network::Frame;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

// Import MessageCodec from aerox_network
use aerox_network::MessageCodec;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    ShuttingDown,
}

/// Client connection
pub struct ClientConnection {
    /// Framed codec for reading/writing frames
    framed: Framed<TcpStream, MessageCodec>,

    /// Remote server address
    server_addr: SocketAddr,

    /// Sequence ID for messages
    sequence_id: Arc<AtomicU64>,

    /// Connection metadata
    connected_at: Instant,

    /// Last activity timestamp
    last_active: Arc<tokio::sync::RwLock<Instant>>,

    /// Connection state
    state: Arc<tokio::sync::RwLock<ClientState>>,
}

impl ClientConnection {
    /// Connect to server
    pub async fn connect(config: &ClientConfig) -> Result<Self> {
        // Set state to Connecting
        let state = Arc::new(tokio::sync::RwLock::new(ClientState::Connecting));

        // Connect with timeout
        let stream = tokio::time::timeout(
            config.connect_timeout,
            TcpStream::connect(config.server_addr),
        )
        .await
        .map_err(|_| ClientError::Timeout("Connection timed out".to_string()))?
        .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;

        let server_addr = stream.peer_addr().map_err(|e| {
            ClientError::ConnectionFailed(format!("Failed to get peer address: {}", e))
        })?;

        // Create framed codec
        let framed = Framed::new(stream, MessageCodec::new());

        let now = Instant::now();

        // Update state to Connected
        let mut state_guard = state.write().await;
        *state_guard = ClientState::Connected;
        drop(state_guard);

        Ok(Self {
            framed,
            server_addr,
            sequence_id: Arc::new(AtomicU64::new(0)),
            connected_at: now,
            last_active: Arc::new(tokio::sync::RwLock::new(now)),
            state,
        })
    }

    /// Send a frame
    pub async fn send_frame(&mut self, frame: Frame) -> Result<()> {
        // Check connection state
        {
            let state = self.state.read().await;
            if *state != ClientState::Connected {
                return Err(ClientError::NotConnected);
            }
        }

        // Send frame through codec
        self.framed
            .send(frame)
            .await
            .map_err(|e| ClientError::SendFailed(e.to_string()))?;

        // Update last activity
        let mut last_active = self.last_active.write().await;
        *last_active = Instant::now();

        Ok(())
    }

    /// Send protobuf message
    pub async fn send_message<M: prost::Message>(
        &mut self,
        msg_id: u16,
        message: &M,
    ) -> Result<()> {
        // Encode message
        let mut buf = BytesMut::new();
        message.encode(&mut buf)
            .map_err(|e| ClientError::SendFailed(format!("Encoding failed: {}", e)))?;

        // Create frame
        let seq_id = self.sequence_id.fetch_add(1, Ordering::SeqCst) as u32;
        let frame = Frame::new(msg_id, seq_id, buf.freeze());

        // Send frame
        self.send_frame(frame).await
    }

    /// Receive next frame
    pub async fn recv_frame(&mut self) -> Result<Frame> {
        // Check connection state
        {
            let state = self.state.read().await;
            if *state != ClientState::Connected {
                return Err(ClientError::NotConnected);
            }
        }

        // Receive frame through codec
        let frame = self
            .framed
            .next()
            .await
            .ok_or_else(|| ClientError::ReceiveFailed("Connection closed".to_string()))?
            .map_err(|e| ClientError::ReceiveFailed(e.to_string()))?;

        // Update last activity
        let mut last_active = self.last_active.write().await;
        *last_active = Instant::now();

        Ok(frame)
    }

    /// Receive and decode protobuf message
    pub async fn recv_message<M: prost::Message + Default>(
        &mut self,
    ) -> Result<(u16, M)> {
        let frame = self.recv_frame().await?;

        let msg = M::decode(&*frame.body)
            .map_err(|e| ClientError::ReceiveFailed(format!("Decoding failed: {}", e)))?;

        Ok((frame.message_id, msg))
    }

    /// Get connection state
    pub async fn state(&self) -> ClientState {
        *self.state.read().await
    }

    /// Get server address
    pub fn server_addr(&self) -> SocketAddr {
        self.server_addr
    }

    /// Get connected time
    pub fn connected_at(&self) -> Instant {
        self.connected_at
    }

    /// Get last activity time
    pub async fn last_active(&self) -> Instant {
        *self.last_active.read().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == ClientState::Connected
    }

    /// Close connection
    pub async fn close(mut self) -> Result<()> {
        // Update state to ShuttingDown
        {
            let mut state = self.state.write().await;
            *state = ClientState::ShuttingDown;
        }

        // Close the framed codec
        self.framed
            .close()
            .await
            .map_err(|e| ClientError::ConnectionFailed(format!("Close failed: {}", e)))?;

        Ok(())
    }

    /// Get framed codec (for advanced usage)
    pub fn framed(&mut self) -> &mut Framed<TcpStream, MessageCodec> {
        &mut self.framed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_state() {
        let state = ClientState::Disconnected;
        assert_eq!(state, ClientState::Disconnected);

        let state2 = ClientState::Connected;
        assert_ne!(state, state2);
    }

    #[tokio::test]
    async fn test_sequence_id_increment() {
        let seq_id = Arc::new(AtomicU64::new(0));

        assert_eq!(seq_id.fetch_add(1, Ordering::SeqCst), 0);
        assert_eq!(seq_id.fetch_add(1, Ordering::SeqCst), 1);
        assert_eq!(seq_id.load(Ordering::SeqCst), 2);
    }
}
