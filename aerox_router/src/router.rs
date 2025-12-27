//! 路由系统
//!
//! 消息 ID 到处理函数的映射。

use std::collections::HashMap;
use thiserror::Error;

/// 路由错误
#[derive(Error, Debug)]
pub enum RouterError {
    /// 路由不存在
    #[error("路由不存在: {0}")]
    RouteNotFound(u32),

    /// 路由已存在
    #[error("路由已存在: {0}")]
    RouteAlreadyExists(u32),
}

/// 路由器
pub struct Router {
    /// 路由表
    routes: HashMap<u32, String>,  // message_id -> handler_name
}

impl Router {
    /// 创建新路由器
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// 添加路由
    pub fn add_route(&mut self, message_id: u32, handler: String) -> Result<(), RouterError> {
        if self.routes.contains_key(&message_id) {
            return Err(RouterError::RouteAlreadyExists(message_id));
        }
        self.routes.insert(message_id, handler);
        Ok(())
    }

    /// 查找路由
    pub fn get_route(&self, message_id: u32) -> Option<&String> {
        self.routes.get(&message_id)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router() {
        let mut router = Router::new();
        router.add_route(1001, "handler1".to_string()).unwrap();
        assert_eq!(router.get_route(1001), Some(&"handler1".to_string()));
        assert_eq!(router.get_route(1002), None);
    }
}
