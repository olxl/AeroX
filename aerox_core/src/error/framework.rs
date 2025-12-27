//! AeroX 框架核心错误类型
//!
//! 定义所有框架级别的错误类型。

use super::context::ErrorContext;
use std::io;
use thiserror::Error;

/// AeroX 框架核心错误类型
#[derive(Error, Debug)]
pub enum AeroXError {
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    Network(String),

    /// 协议错误
    #[error("协议错误: {0}")]
    Protocol(String),

    /// 路由错误
    #[error("路由错误: {0}")]
    Router(String),

    /// 插件错误
    #[error("插件错误: {0}")]
    Plugin(String),

    /// 序列化/反序列化错误
    #[error("序列化错误: {0}")]
    Serialization(String),

    /// 连接错误
    #[error("连接错误: {0}")]
    Connection(String),

    /// 超时错误
    #[error("操作超时")]
    Timeout,

    /// 未实现的特性
    #[error("未实现的特性: {0}")]
    Unimplemented(String),

    /// 验证错误
    #[error("验证失败: {0}")]
    Validation(String),

    /// 带上下文的错误
    #[error("{0}")]
    WithContext(#[source] Box<AeroXError>, ErrorContext),
}

impl AeroXError {
    /// 获取错误类型
    pub fn kind(&self) -> AeroXErrorKind {
        match self {
            AeroXError::Io(_) => AeroXErrorKind::Io,
            AeroXError::Config(_) => AeroXErrorKind::Config,
            AeroXError::Network(_) => AeroXErrorKind::Network,
            AeroXError::Protocol(_) => AeroXErrorKind::Protocol,
            AeroXError::Router(_) => AeroXErrorKind::Router,
            AeroXError::Plugin(_) => AeroXErrorKind::Plugin,
            AeroXError::Serialization(_) => AeroXErrorKind::Serialization,
            AeroXError::Connection(_) => AeroXErrorKind::Connection,
            AeroXError::Timeout => AeroXErrorKind::Timeout,
            AeroXError::Unimplemented(_) => AeroXErrorKind::Unimplemented,
            AeroXError::Validation(_) => AeroXErrorKind::Validation,
            AeroXError::WithContext(_, _) => AeroXErrorKind::Other,
        }
    }

    /// 添加上下文信息
    pub fn with_context<C>(self, context: C) -> Self
    where
        C: Into<ErrorContext>,
    {
        AeroXError::WithContext(Box::new(self), context.into())
    }

    /// 创建配置错误
    pub fn config(msg: impl Into<String>) -> Self {
        AeroXError::Config(msg.into())
    }

    /// 创建网络错误
    pub fn network(msg: impl Into<String>) -> Self {
        AeroXError::Network(msg.into())
    }

    /// 创建协议错误
    pub fn protocol(msg: impl Into<String>) -> Self {
        AeroXError::Protocol(msg.into())
    }

    /// 创建路由错误
    pub fn router(msg: impl Into<String>) -> Self {
        AeroXError::Router(msg.into())
    }

    /// 创建插件错误
    pub fn plugin(msg: impl Into<String>) -> Self {
        AeroXError::Plugin(msg.into())
    }

    /// 创建序列化错误
    pub fn serialization(msg: impl Into<String>) -> Self {
        AeroXError::Serialization(msg.into())
    }

    /// 创建连接错误
    pub fn connection(msg: impl Into<String>) -> Self {
        AeroXError::Connection(msg.into())
    }

    /// 创建超时错误
    pub fn timeout() -> Self {
        AeroXError::Timeout
    }

    /// 创建未实现错误
    pub fn unimplemented(msg: impl Into<String>) -> Self {
        AeroXError::Unimplemented(msg.into())
    }

    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        AeroXError::Validation(msg.into())
    }
}

/// 错误类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AeroXErrorKind {
    /// IO 错误
    Io,
    /// 配置错误
    Config,
    /// 网络错误
    Network,
    /// 协议错误
    Protocol,
    /// 路由错误
    Router,
    /// 插件错误
    Plugin,
    /// 序列化错误
    Serialization,
    /// 连接错误
    Connection,
    /// 超时错误
    Timeout,
    /// 未实现特性
    Unimplemented,
    /// 验证错误
    Validation,
    /// 其他错误
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AeroXError::config("test error");
        assert!(matches!(err, AeroXError::Config(_)));
        assert_eq!(err.kind(), AeroXErrorKind::Config);
    }

    #[test]
    fn test_error_with_context() {
        let err = AeroXError::network("connection failed")
            .with_context(("peer_address", "127.0.0.1:8080"));
        assert!(matches!(err, AeroXError::WithContext(_, _)));
    }

    #[test]
    fn test_error_kinds() {
        assert_eq!(AeroXError::config("").kind(), AeroXErrorKind::Config);
        assert_eq!(AeroXError::network("").kind(), AeroXErrorKind::Network);
        assert_eq!(AeroXError::timeout().kind(), AeroXErrorKind::Timeout);
    }
}
