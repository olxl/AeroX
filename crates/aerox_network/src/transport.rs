//! 传输层抽象
//!
//! 定义传输协议的统一接口。

use crate::connection::Connection;
use std::net::SocketAddr;
use thiserror::Error;

/// 传输错误
#[derive(Error, Debug)]
pub enum TransportError {
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 连接错误
    #[error("连接错误: {0}")]
    Connection(String),

    /// 超时错误
    #[error("操作超时")]
    Timeout,
}

/// 传输层 Result 类型
pub type Result<T> = std::result::Result<T, TransportError>;

/// 传输层抽象 trait
pub trait Transport: Send + Sync {
    /// 连接到远程地址
    async fn connect(&self, addr: &SocketAddr) -> Result<Connection>;

    /// 绑定到本地地址并监听
    /// 返回一个监听句柄，后续可以使用 accept 接受连接
    async fn bind(&self, addr: &SocketAddr) -> Result<std::net::TcpListener>;
}
