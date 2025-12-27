//! 配置管理系统
//!
//! 提供灵活的配置管理，支持服务器配置和环境变量。

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// 配置错误类型
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 解析错误
    #[error("解析配置文件失败: {0}")]
    Parse(String),

    /// 验证错误
    #[error("配置验证失败: {0}")]
    Validation(String),
}

/// 配置 Result 类型
pub type Result<T> = std::result::Result<T, ConfigError>;

/// 服务器配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 绑定地址
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// 最大连接数限制
    #[serde(default)]
    pub max_connections: Option<u32>,

    /// 每个连接每秒最大请求数
    #[serde(default = "default_max_requests_per_second_per_connection")]
    pub max_requests_per_second_per_connection: Option<u32>,

    /// 全局每秒最大请求数
    #[serde(default = "default_max_requests_per_second_total")]
    pub max_requests_per_second_total: Option<u32>,

    /// 是否启用 DDoS 防护
    #[serde(default = "default_enable_ddos_protection")]
    pub enable_ddos_protection: bool,

    /// 工作线程数量（None 表示使用 CPU 核心数）
    #[serde(default)]
    pub worker_threads: Option<usize>,
}

/// Reactor 模式配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReactorConfig {
    /// Reactor 缓冲区大小
    #[serde(default = "default_reactor_buffer_size")]
    pub reactor_buffer_size: usize,

    /// 消息批处理大小
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// 批处理超时时间（毫秒）
    #[serde(default = "default_batch_timeout")]
    pub batch_timeout_ms: u64,

    /// 连接超时时间（秒）
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            port: default_port(),
            max_connections: None,
            max_requests_per_second_per_connection: default_max_requests_per_second_per_connection(),
            max_requests_per_second_total: default_max_requests_per_second_total(),
            enable_ddos_protection: default_enable_ddos_protection(),
            worker_threads: None,
        }
    }
}

impl Default for ReactorConfig {
    fn default() -> Self {
        Self {
            reactor_buffer_size: default_reactor_buffer_size(),
            batch_size: default_batch_size(),
            batch_timeout_ms: default_batch_timeout(),
            connection_timeout_secs: default_connection_timeout(),
        }
    }
}

impl ServerConfig {
    /// 从 TOML 文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Parse(format!("读取配置文件失败: {}", e)))?;

        let config: ServerConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(format!("解析配置文件失败: {}", e)))?;

        Ok(config)
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(ConfigError::Validation("端口不能为 0".to_string()));
        }

        if self.port > 65535 {
            return Err(ConfigError::Validation("端口超出范围".to_string()));
        }

        Ok(())
    }

    /// 获取完整的绑定地址字符串
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

// 默认值函数
fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_max_requests_per_second_per_connection() -> Option<u32> {
    Some(1000)
}

fn default_max_requests_per_second_total() -> Option<u32> {
    Some(100_000)
}

fn default_enable_ddos_protection() -> bool {
    true
}

fn default_reactor_buffer_size() -> usize {
    8192
}

fn default_batch_size() -> usize {
    32
}

fn default_batch_timeout() -> u64 {
    10
}

fn default_connection_timeout() -> u64 {
    300
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_bind_addr() {
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 9000,
            ..Default::default()
        };
        assert_eq!(config.bind_addr(), "127.0.0.1:9000");
    }

    #[test]
    fn test_validate_invalid_port() {
        let config = ServerConfig {
            port: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
