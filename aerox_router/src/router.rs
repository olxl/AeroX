//! 路由系统
//!
//! 消息 ID 到处理函数的映射。

use crate::context::Context;
use aerox_core::{AeroXError, Result};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// 消息处理器 trait
///
/// 定义消息处理器的接口
pub trait Handler: Send + Sync + 'static {
    /// 处理消息
    fn call(&self, ctx: Context) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;
}

/// 用于函数指针的辅助实现
impl<F> Handler for F
where
    F: Fn(Context) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync + 'static,
{
    fn call(&self, ctx: Context) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        self(ctx)
    }
}

/// 路由器
///
/// 管理消息 ID 到处理器的映射
pub struct Router {
    /// 路由表: message_id -> handler
    routes: HashMap<u16, Box<dyn Handler>>,
}

impl Router {
    /// 创建新路由器
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// 添加路由
    ///
    /// # 参数
    /// - `message_id`: 消息 ID
    /// - `handler`: 消息处理器
    pub fn add_route<H>(&mut self, message_id: u16, handler: H) -> Result<()>
    where
        H: Handler + 'static,
    {
        if self.routes.contains_key(&message_id) {
            return Err(AeroXError::router(format!("路由已存在: {}", message_id)));
        }
        self.routes.insert(message_id, Box::new(handler));
        Ok(())
    }

    /// 查找路由
    ///
    /// # 参数
    /// - `message_id`: 消息 ID
    ///
    /// # 返回
    /// 处理器的引用，如果不存在则返回 None
    pub fn get_route(&self, message_id: u16) -> Option<&dyn Handler> {
        self.routes.get(&message_id).map(|h| h.as_ref())
    }

    /// 处理消息
    ///
    /// 根据消息 ID 找到对应的处理器并调用
    ///
    /// # 参数
    /// - `ctx`: 请求上下文
    pub async fn handle(&self, ctx: Context) -> Result<()> {
        let handler = self
            .get_route(ctx.message_id())
            .ok_or_else(|| AeroXError::router(format!("未找到路由: {}", ctx.message_id())))?;

        handler.call(ctx).await
    }

    /// 获取路由数量
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// 检查路由是否存在
    pub fn has_route(&self, message_id: u16) -> bool {
        self.routes.contains_key(&message_id)
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
    use aerox_network::ConnectionId;
    use bytes::Bytes;

    // 简单的测试处理器
    fn test_handler(ctx: Context) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            println!(
                "处理消息: id={}, data_len={}",
                ctx.message_id(),
                ctx.data().len()
            );
            Ok(())
        })
    }

    // 会修改数据的处理器
    fn echo_handler(ctx: Context) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            println!("Echo: {}", ctx.message_id());
            Ok(())
        })
    }

    #[test]
    fn test_router_creation() {
        let router = Router::new();
        assert_eq!(router.route_count(), 0);
        assert!(!router.has_route(100));
    }

    #[test]
    fn test_add_route() {
        let mut router = Router::new();
        router.add_route(100, test_handler).unwrap();
        assert_eq!(router.route_count(), 1);
        assert!(router.has_route(100));
    }

    #[test]
    fn test_duplicate_route() {
        let mut router = Router::new();
        router.add_route(100, test_handler).unwrap();
        let result = router.add_route(100, echo_handler);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_route() {
        let mut router = Router::new();
        router.add_route(100, test_handler).unwrap();
        assert!(router.get_route(100).is_some());
        assert!(router.get_route(200).is_none());
    }

    #[tokio::test]
    async fn test_handle_message() {
        let mut router = Router::new();
        router.add_route(100, test_handler).unwrap();

        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let data = Bytes::from("test data");

        let ctx = Context::new(conn_id, addr, 100, 1000, data);
        let result = router.handle(ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_unknown_route() {
        let router = Router::new();

        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let data = Bytes::new();

        let ctx = Context::new(conn_id, addr, 999, 1000, data);
        let result = router.handle(ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_routes() {
        let mut router = Router::new();
        router.add_route(100, test_handler).unwrap();
        router.add_route(200, echo_handler).unwrap();

        assert_eq!(router.route_count(), 2);

        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();

        // 测试第一个路由
        let ctx1 = Context::new(conn_id, addr, 100, 1000, Bytes::new());
        assert!(router.handle(ctx1).await.is_ok());

        // 测试第二个路由
        let ctx2 = Context::new(conn_id, addr, 200, 1001, Bytes::new());
        assert!(router.handle(ctx2).await.is_ok());
    }
}
