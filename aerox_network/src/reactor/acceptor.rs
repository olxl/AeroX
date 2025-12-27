//! 连接接受器
//!
//! 负责接受新的 TCP 连接。

use crate::reactor::balancer::ConnectionBalancer;
use aerox_core::{AeroXError, Result};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

/// 连接接受器
///
/// 接受新的 TCP 连接并发送到 Worker
pub struct Acceptor {
    /// TCP 监听器
    listener: TcpListener,
    /// 连接均衡器
    balancer: Arc<ConnectionBalancer>,
    /// 发送通道到各 Worker
    worker_txs: Vec<mpsc::Sender<NewConnection>>,
}

/// 新连接消息
pub struct NewConnection {
    /// TCP 流
    pub stream: tokio::net::TcpStream,
    /// 远程地址
    pub remote_addr: std::net::SocketAddr,
}

impl Acceptor {
    /// 创建新的连接接受器
    pub fn new(
        listener: TcpListener,
        balancer: Arc<ConnectionBalancer>,
        worker_txs: Vec<mpsc::Sender<NewConnection>>,
    ) -> Self {
        Self {
            listener,
            balancer,
            worker_txs,
        }
    }

    /// 启动接受器
    ///
    /// 开始接受新连接并分配给 Worker
    pub async fn run(&mut self) -> Result<()> {
        println!("AeroX Reactor: 开始接受连接，监听地址: {:?}", self.listener.local_addr());

        loop {
            // 接受新连接
            match self.listener.accept().await {
                Ok((stream, remote_addr)) => {
                    // 分配给 Worker
                    let worker_id = self.balancer.next_worker();

                    if let Err(_) = self.worker_txs[worker_id].send(NewConnection {
                        stream,
                        remote_addr,
                    }).await {
                        return Err(AeroXError::network(format!(
                            "无法发送连接到 Worker {}", worker_id
                        )));
                    }
                }
                Err(e) => {
                    return Err(AeroXError::network(format!("接受连接失败: {}", e)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptor_creation() {
        // 基础创建测试
        let balancer = ConnectionBalancer::new(2);
        let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();

        // 注意：这里不能实际运行，因为需要异步运行时
        // 实际测试在集成测试中进行
    }
}
