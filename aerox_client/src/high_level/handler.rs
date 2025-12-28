//! Message handler trait and registry

use crate::error::Result;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use bytes::Bytes;

/// Message handler trait
#[async_trait]
pub trait MessageHandler<M: prost::Message + Default + Send + 'static>: Send + Sync {
    /// Handle a message
    async fn handle(&self, msg_id: u16, message: M) -> Result<()>;
}

/// Function-based handler (simplified - just wraps async functions)
pub struct FnHandler<M, F>
where
    M: prost::Message + Default + Send + 'static,
    F: Fn(u16, M) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync + 'static,
{
    _phantom: std::marker::PhantomData<M>,
    f: Arc<F>,
}

impl<M, F> FnHandler<M, F>
where
    M: prost::Message + Default + Send + 'static,
    F: Fn(u16, M) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync + 'static,
{
    pub fn new(f: F) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            f: Arc::new(f),
        }
    }
}

#[async_trait]
impl<M, F> MessageHandler<M> for FnHandler<M, F>
where
    M: prost::Message + Default + Send + 'static,
    F: Fn(u16, M) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync + 'static,
{
    async fn handle(&self, msg_id: u16, message: M) -> Result<()> {
        let f = self.f.clone();
        f(msg_id, message).await
    }
}

/// Type-erased handler that can decode and handle messages from bytes
type ErasedHandler = Box<dyn Fn(u16, Bytes) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync>;

/// Handler registry
pub struct HandlerRegistry {
    handlers: tokio::sync::RwLock<HashMap<u16, ErasedHandler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Register a handler for a message ID
    pub async fn register<M, H>(&self, msg_id: u16, handler: H) -> Result<()>
    where
        M: prost::Message + Default + Send + 'static,
        H: MessageHandler<M> + 'static,
    {
        // Wrap handler in Arc before moving into closure
        let handler = Arc::new(handler);

        let erased_handler: ErasedHandler = Box::new(move |mid: u16, data: Bytes| {
            let handler = handler.clone();
            Box::pin(async move {
                // Decode the message
                let message = M::decode(data.as_ref())
                    .map_err(|e| crate::error::ClientError::ReceiveFailed(format!("Failed to decode message: {}", e)))?;

                // Call the handler
                handler.handle(mid, message).await
            })
        });

        let mut handlers = self.handlers.write().await;
        handlers.insert(msg_id, erased_handler);
        Ok(())
    }

    /// Dispatch a message to the appropriate handler
    pub async fn dispatch(&self, msg_id: u16, data: Bytes) {
        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(&msg_id) {
            let _ = handler(msg_id, data).await;
        }
    }

    /// Check if a handler exists for a message ID
    pub async fn has_handler(&self, msg_id: u16) -> bool {
        let handlers = self.handlers.read().await;
        handlers.contains_key(&msg_id)
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_registry() {
        let registry = HandlerRegistry::new();

        // Initially no handlers
        assert!(!registry.has_handler(1).await);
    }
}
