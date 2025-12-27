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

    /// 环境变量错误
    #[error("环境变量解析失败: {0}")]
    EnvVar(String),
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

    /// 从环境变量加载配置并覆盖
    ///
    /// 支持的环境变量：
    /// - AEROX_BIND_ADDRESS: 绑定地址
    /// - AEROX_PORT: 端口
    /// - AEROX_MAX_CONNECTIONS: 最大连接数
    /// - AEROX_ENABLE_DDOS_PROTECTION: 启用 DDoS 防护 (true/false)
    /// - AEROX_WORKER_THREADS: 工作线程数
    pub fn load_with_env_override(mut self) -> Result<Self> {
        // 绑定地址
        if let Ok(addr) = std::env::var("AEROX_BIND_ADDRESS") {
            self.bind_address = addr;
        }

        // 端口
        if let Ok(port_str) = std::env::var("AEROX_PORT") {
            self.port = port_str.parse()
                .map_err(|_| ConfigError::EnvVar("AEROX_PORT 必须是有效的 u16 数字".to_string()))?;
        }

        // 最大连接数
        if let Ok(max_conn) = std::env::var("AEROX_MAX_CONNECTIONS") {
            self.max_connections = Some(max_conn.parse()
                .map_err(|_| ConfigError::EnvVar("AEROX_MAX_CONNECTIONS 必须是有效的 u32 数字".to_string()))?);
        }

        // DDoS 防护
        if let Ok(ddos) = std::env::var("AEROX_ENABLE_DDOS_PROTECTION") {
            self.enable_ddos_protection = ddos.parse()
                .map_err(|_| ConfigError::EnvVar("AEROX_ENABLE_DDOS_PROTECTION 必须是 true 或 false".to_string()))?;
        }

        // 工作线程数
        if let Ok(threads) = std::env::var("AEROX_WORKER_THREADS") {
            self.worker_threads = Some(threads.parse()
                .map_err(|_| ConfigError::EnvVar("AEROX_WORKER_THREADS 必须是有效的 usize 数字".to_string()))?);
        }

        Ok(self)
    }

    /// 从文件加载并应用环境变量覆盖
    pub fn from_file_with_env<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file(path)?.load_with_env_override()
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        // 端口验证
        if self.port == 0 {
            return Err(ConfigError::Validation("端口不能为 0".to_string()));
        }

        // 地址验证
        if self.bind_address.is_empty() {
            return Err(ConfigError::Validation("绑定地址不能为空".to_string()));
        }

        // 工作线程数验证
        if let Some(threads) = self.worker_threads {
            if threads == 0 {
                return Err(ConfigError::Validation("工作线程数不能为 0".to_string()));
            }
            if threads > 512 {
                return Err(ConfigError::Validation("工作线程数过大 (建议 <= 512)".to_string()));
            }
        }

        // 最大连接数验证
        if let Some(max_conn) = self.max_connections {
            if max_conn == 0 {
                return Err(ConfigError::Validation("最大连接数不能为 0".to_string()));
            }
        }

        // 每连接请求数验证
        if let Some(reqs) = self.max_requests_per_second_per_connection {
            if reqs == 0 {
                return Err(ConfigError::Validation("每连接请求数不能为 0".to_string()));
            }
        }

        // 全局请求数验证
        if let Some(reqs) = self.max_requests_per_second_total {
            if reqs == 0 {
                return Err(ConfigError::Validation("全局请求数不能为 0".to_string()));
            }
        }

        Ok(())
    }

    /// 获取完整的绑定地址字符串
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }

    /// 获取配置摘要信息
    pub fn summary(&self) -> String {
        format!(
            "AeroX 服务器配置:\n  地址: {}\n  最大连接数: {:?}\n  DDoS 防护: {}\n  工作线程: {:?}",
            self.bind_addr(),
            self.max_connections,
            self.enable_ddos_protection,
            self.worker_threads
        )
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

    #[test]
    fn test_validate_empty_address() {
        let config = ServerConfig {
            bind_address: "".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_worker_threads() {
        let config = ServerConfig {
            worker_threads: Some(0),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_too_many_worker_threads() {
        let config = ServerConfig {
            worker_threads: Some(1000),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_max_connections() {
        let config = ServerConfig {
            max_connections: Some(0),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_override_port() {
        std::env::set_var("AEROX_PORT", "9999");
        let config = ServerConfig::default()
            .load_with_env_override()
            .unwrap();
        assert_eq!(config.port, 9999);
        std::env::remove_var("AEROX_PORT");
    }

    #[test]
    fn test_env_override_address() {
        std::env::set_var("AEROX_BIND_ADDRESS", "127.0.0.1");
        let config = ServerConfig::default()
            .load_with_env_override()
            .unwrap();
        assert_eq!(config.bind_address, "127.0.0.1");
        std::env::remove_var("AEROX_BIND_ADDRESS");
    }

    #[test]
    fn test_env_override_invalid_port() {
        std::env::set_var("AEROX_PORT", "invalid");
        let result = ServerConfig::default()
            .load_with_env_override();
        assert!(result.is_err());
        std::env::remove_var("AEROX_PORT");
    }

    #[test]
    fn test_env_override_ddos_protection() {
        std::env::set_var("AEROX_ENABLE_DDOS_PROTECTION", "false");
        let config = ServerConfig::default()
            .load_with_env_override()
            .unwrap();
        assert!(!config.enable_ddos_protection);
        std::env::remove_var("AEROX_ENABLE_DDOS_PROTECTION");
    }

    #[test]
    fn test_env_override_max_connections() {
        std::env::set_var("AEROX_MAX_CONNECTIONS", "5000");
        let config = ServerConfig::default()
            .load_with_env_override()
            .unwrap();
        assert_eq!(config.max_connections, Some(5000));
        std::env::remove_var("AEROX_MAX_CONNECTIONS");
    }

    #[test]
    fn test_config_summary() {
        let config = ServerConfig::default();
        let summary = config.summary();
        assert!(summary.contains("0.0.0.0:8080"));
        assert!(summary.contains("AeroX 服务器配置"));
    }
}
