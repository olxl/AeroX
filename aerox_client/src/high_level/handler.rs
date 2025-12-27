//! Message handler trait and registry

use crate::error::Result;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

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

/// Handler registry (simplified - tracks which message IDs have handlers)
pub struct HandlerRegistry {
    // For now, we just track which message IDs have handlers registered
    // The actual handling is done by callbacks registered elsewhere
    handlers: tokio::sync::RwLock<HashMap<u16, bool>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Register that a handler exists for this message ID
    pub async fn register<M, H>(&self, msg_id: u16, _handler: H) -> Result<()>
    where
        M: prost::Message + Default + Send + 'static,
        H: MessageHandler<M> + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers.insert(msg_id, true);
        Ok(())
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
