//! 消息注册表
//!
//! 管理消息类型和消息 ID 的映射。

use std::collections::HashMap;
use thiserror::Error;

/// 消息注册错误
#[derive(Error, Debug)]
pub enum RegistryError {
    /// 消息未注册
    #[error("消息未注册: {0}")]
    MessageNotRegistered(u32),

    /// 消息已存在
    #[error("消息已存在: {0}")]
    MessageAlreadyExists(u32),
}

/// 消息注册表
pub struct MessageRegistry {
    /// 消息 ID 到消息名称的映射
    messages: HashMap<u32, String>,
}

impl MessageRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    /// 注册消息
    pub fn register(&mut self, id: u32, name: String) -> Result<(), RegistryError> {
        if self.messages.contains_key(&id) {
            return Err(RegistryError::MessageAlreadyExists(id));
        }
        self.messages.insert(id, name);
        Ok(())
    }

    /// 检查消息是否已注册
    pub fn contains(&self, id: u32) -> bool {
        self.messages.contains_key(&id)
    }
}

impl Default for MessageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let mut registry = MessageRegistry::new();
        registry.register(1001, "TestMessage".to_string()).unwrap();
        assert!(registry.contains(1001));
        assert!(!registry.contains(1002));
    }
}
