//! 错误类型定义
//!
//! 提供 AeroX 框架中所有错误类型的统一处理。

use std::io;

/// AeroX 框架的主错误类型
#[derive(thiserror::Error, Debug)]
pub enum AeroXError {
    /// IO 相关错误
    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 协议错误
    #[error("协议错误: {0}")]
    Protocol(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    Network(String),

    /// 路由错误
    #[error("路由错误: {0}")]
    Router(String),

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
}

/// AeroX 的 Result 类型别名
pub type Result<T> = std::result::Result<T, AeroXError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AeroXError::Config("测试配置错误".to_string());
        assert_eq!(err.to_string(), "配置错误: 测试配置错误");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "文件未找到");
        let aerox_err: AeroXError = io_err.into();
        assert!(matches!(aerox_err, AeroXError::Io(_)));
    }
}
