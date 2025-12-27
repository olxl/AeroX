//! TCP Reactor 实现
//!
//! Reactor 模式的主入口，管理 Acceptor 和多个 Worker。

use crate::reactor::{acceptor::Acceptor, worker::Worker, balancer::ConnectionBalancer};
use aerox_core::{AeroXError, Result};
use aerox_config::{ServerConfig, ReactorConfig};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use aerox_config::ConfigError;

/// TCP Reactor
///
/// 基于 Reactor 模式的 TCP 服务器
pub struct TcpReactor {
    /// 服务器配置
    server_config: ServerConfig,
    /// Reactor 配置
    reactor_config: ReactorConfig,
    /// Worker 任务句柄
    worker_handles: Vec<JoinHandle<Result<()>>>,
}

impl TcpReactor {
    /// 创建新的 TCP Reactor
    pub fn new(server_config: ServerConfig, reactor_config: ReactorConfig) -> Self {
        Self {
            server_config,
            reactor_config,
            worker_handles: Vec::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(ServerConfig::default(), ReactorConfig::default())
    }

    /// 启动 Reactor
    ///
    /// 启动 Acceptor 和多个 Worker
    pub async fn run(mut self) -> Result<()> {
        // 验证配置
        self.server_config.validate()
            .map_err(|e: ConfigError| AeroXError::config(e.to_string()))?;

        // 确定工作线程数
        let worker_count = self.server_config.worker_threads
            .unwrap_or_else(|| num_cpus::get());

        println!("AeroX TCP Reactor 启动:");
        println!("  监听地址: {}", self.server_config.bind_addr());
        println!("  工作线程数: {}", worker_count);
        println!("  缓冲区大小: {}", self.reactor_config.reactor_buffer_size);

        // 创建 TCP 监听器
        let bind_addr = self.server_config.bind_addr();
        let listener = TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| AeroXError::network(format!("绑定地址失败: {}", e)))?;

        // 创建连接均衡器
        let balancer = Arc::new(ConnectionBalancer::new(worker_count));

        // 创建 Worker
        let mut worker_txs = Vec::new();
        for id in 0..worker_count {
            let config = crate::reactor::worker::WorkerConfig {
                id,
                channel_size: self.reactor_config.reactor_buffer_size,
            };

            let (worker, tx) = Worker::new(config);
            let handle = worker.spawn();
            self.worker_handles.push(handle);
            worker_txs.push(tx);
        }

        // 创建并启动 Acceptor
        let mut acceptor = Acceptor::new(listener, balancer, worker_txs);

        // 运行 Acceptor（这会阻塞直到出错）
        acceptor.run().await?;

        // 等待所有 Worker 完成
        for handle in self.worker_handles {
            if let Ok(result) = handle.await {
                if let Err(e) = result {
                    eprintln!("Worker 错误: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 获取服务器配置
    pub fn server_config(&self) -> &ServerConfig {
        &self.server_config
    }

    /// 获取 Reactor 配置
    pub fn reactor_config(&self) -> &ReactorConfig {
        &self.reactor_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactor_creation() {
        let reactor = TcpReactor::with_defaults();
        let config = reactor.server_config();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_reactor_custom_config() {
        let server_config = ServerConfig {
            port: 9999,
            ..Default::default()
        };
        let reactor = TcpReactor::new(server_config, ReactorConfig::default());
        assert_eq!(reactor.server_config().port, 9999);
    }
}
