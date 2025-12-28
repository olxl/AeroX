//! High-level client with automatic message handling

use crate::config::ClientConfig;
use crate::connection::{ClientConnection, ClientState};
use crate::error::{ClientError, Result};
use crate::high_level::event::ClientEvent;
use crate::high_level::handler::{FnHandler, HandlerRegistry, MessageHandler};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

/// High-level client
///
/// Automatically receives messages in the background and dispatches them to
/// registered handlers.
pub struct HighLevelClient {
    /// Connection for receiving frames
    connection: Arc<tokio::sync::Mutex<ClientConnection>>,
    /// Sender for sending frames (can be cloned and used without locking)
    send_tx: tokio::sync::mpsc::Sender<aerox_network::Frame>,
    handler_registry: Arc<HandlerRegistry>,
    event_tx: broadcast::Sender<ClientEvent>,
    receiver_handle: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    config: ClientConfig,
}

impl HighLevelClient {
    /// Connect to server
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let config = ClientConfig::new(addr);
        Self::connect_with_config(config).await
    }

    /// Connect with custom configuration
    pub async fn connect_with_config(config: ClientConfig) -> Result<Self> {
        // Create connection
        let connection = ClientConnection::connect(&config).await?;

        // Extract the send_tx from the connection
        let send_tx = connection.get_send_tx();

        // Create event channel
        let (event_tx, _) = broadcast::channel(100);

        // Emit connected event
        let _ = event_tx.send(ClientEvent::Connected {
            addr: connection.server_addr(),
        });

        // Wrap connection in Arc<Mutex>
        let connection = Arc::new(tokio::sync::Mutex::new(connection));

        // Create handler registry
        let handler_registry = Arc::new(HandlerRegistry::new());

        // Start background receiver task
        let receiver_handle = Self::start_receiver_task(
            connection.clone(),
            handler_registry.clone(),
            event_tx.clone(),
            config.clone(),
        );

        Ok(Self {
            connection,
            send_tx,
            handler_registry,
            event_tx,
            receiver_handle: Arc::new(tokio::sync::Mutex::new(Some(receiver_handle))),
            config,
        })
    }

    /// Start the background receiver task
    fn start_receiver_task(
        connection: Arc<tokio::sync::Mutex<ClientConnection>>,
        handler_registry: Arc<HandlerRegistry>,
        event_tx: broadcast::Sender<ClientEvent>,
        config: ClientConfig,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            // Message receiver loop
            loop {
                // Check connection state
                let state = {
                    let conn = connection.lock().await;
                    conn.state().await
                };

                if state != ClientState::Connected {
                    if config.auto_reconnect {
                        // TODO: Implement reconnect logic
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    } else {
                        break;
                    }
                }

                // Receive frame
                let frame_result = {
                    let mut conn = connection.lock().await;
                    conn.recv_frame().await
                };

                match frame_result {
                    Ok(frame) => {
                        // Emit message received event
                        let _ = event_tx.send(ClientEvent::MessageReceived {
                            msg_id: frame.message_id,
                        });

                        // Dispatch to handler
                        handler_registry.dispatch(frame.message_id, frame.body).await;
                    }
                    Err(e) => {
                        // Emit error event
                        let _ = event_tx.send(ClientEvent::Error {
                            error: e.to_string(),
                        });

                        if config.auto_reconnect {
                            // TODO: Implement reconnect logic
                        } else {
                            break;
                        }
                    }
                }
            }

            // Emit disconnected event
            let _ = event_tx.send(ClientEvent::Disconnected {
                reason: "Receiver task stopped".to_string(),
            });
        })
    }

    /// Register a message handler
    pub async fn register_handler<M, H>(&self, msg_id: u16, handler: H) -> Result<()>
    where
        M: prost::Message + Default + Send + 'static,
        H: MessageHandler<M> + 'static,
    {
        self.handler_registry.register(msg_id, handler).await
    }

    /// Register a closure-based handler
    /// Note: This is a simplified version - handlers are tracked but not currently dispatched
    pub async fn on_message<M, F>(&self, msg_id: u16, _f: F) -> Result<()>
    where
        M: prost::Message + Default + Send + 'static,
        F: Fn(u16, M) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync + 'static,
    {
        // For now, just register that a handler exists
        // Full handler dispatching will be implemented in a future update
        self.handler_registry.register::<M, FnHandler<M, F>>(msg_id, FnHandler::new(_f)).await
    }

    /// Send a message
    pub async fn send<M: prost::Message>(
        &self,
        msg_id: u16,
        message: &M,
    ) -> Result<()> {
        use bytes::BytesMut;
        use prost::Message;

        // Encode message
        let mut buf = BytesMut::new();
        message.encode(&mut buf)
            .map_err(|e| crate::error::ClientError::SendFailed(format!("Encoding failed: {}", e)))?;

        // Create frame (sequence ID will be 0 for now, could be improved)
        let frame = aerox_network::Frame::new(msg_id, 0, buf.freeze());

        // Send frame through channel (non-blocking, doesn't lock connection)
        self.send_tx
            .send(frame)
            .await
            .map_err(|e| crate::error::ClientError::SendFailed(e.to_string()))?;

        // Emit message sent event
        let _ = self.event_tx.send(ClientEvent::MessageSent { msg_id });

        Ok(())
    }

    /// Subscribe to client events
    pub fn subscribe_events(&self) -> broadcast::Receiver<ClientEvent> {
        self.event_tx.subscribe()
    }

    /// Get connection state
    pub async fn state(&self) -> ClientState {
        let conn = self.connection.lock().await;
        conn.state().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let conn = self.connection.lock().await;
        conn.is_connected().await
    }

    /// Get server address
    pub async fn server_addr(&self) -> SocketAddr {
        let conn = self.connection.lock().await;
        conn.server_addr()
    }

    /// Shutdown the client
    pub async fn shutdown(self) -> Result<()> {
        // Stop receiver task
        let mut handle_guard = self.receiver_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
        }

        // Close connection
        let conn = Arc::try_unwrap(self.connection)
            .map_err(|_| ClientError::ConnectionFailed("Failed to unwrap connection".to_string()))?;
        let conn = conn.into_inner();
        conn.close().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_high_level_client_create() {
        // Test that we can create a HighLevelClient struct
        // (actual connection tests require a running server)
        let _ = || -> HighLevelClient {
            // This is a compile-time test
            unimplemented!()
        };
    }
}
