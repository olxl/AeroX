# AeroX 配置系统指南

## 概述

AeroX 提供了灵活的配置管理系统，支持：
- TOML 配置文件
- 环境变量覆盖
- 配置验证
- 默认值

## 配置文件

### 配置文件格式

配置文件使用 TOML 格式，包含 `server` 和 `reactor` 两个部分：

```toml
[server]
bind_address = "0.0.0.0"
port = 8080
max_connections = 10000
max_requests_per_second_per_connection = 1000
max_requests_per_second_total = 100000
enable_ddos_protection = true
worker_threads = 4

[reactor]
reactor_buffer_size = 8192
batch_size = 32
batch_timeout_ms = 10
connection_timeout_secs = 300
```

### 配置项说明

#### Server 配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `bind_address` | String | "0.0.0.0" | 绑定地址 |
| `port` | u16 | 8080 | 监听端口 |
| `max_connections` | Option\<u32\> | None | 最大连接数 |
| `max_requests_per_second_per_connection` | Option\<u32\> | Some(1000) | 每连接每秒请求数 |
| `max_requests_per_second_total` | Option\<u32\> | Some(100000) | 全局每秒请求数 |
| `enable_ddos_protection` | bool | true | 是否启用 DDoS 防护 |
| `worker_threads` | Option\<usize\> | None | 工作线程数 |

#### Reactor 配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `reactor_buffer_size` | usize | 8192 | Reactor 缓冲区大小 |
| `batch_size` | usize | 32 | 消息批处理大小 |
| `batch_timeout_ms` | u64 | 10 | 批处理超时（毫秒） |
| `connection_timeout_secs` | u64 | 300 | 连接超时（秒） |

## 环境变量

支持通过环境变量覆盖配置文件中的设置：

| 环境变量 | 对应配置项 | 示例 |
|----------|-----------|------|
| `AEROX_BIND_ADDRESS` | bind_address | `127.0.0.1` |
| `AEROX_PORT` | port | `9000` |
| `AEROX_MAX_CONNECTIONS` | max_connections | `5000` |
| `AEROX_ENABLE_DDOS_PROTECTION` | enable_ddos_protection | `false` |
| `AEROX_WORKER_THREADS` | worker_threads | `8` |

## 使用示例

### 1. 使用默认配置

```rust
use aerox_config::ServerConfig;

let config = ServerConfig::default();
config.validate().unwrap();
```

### 2. 从文件加载配置

```rust
use aerox_config::ServerConfig;

let config = ServerConfig::from_file("server.toml")?;
config.validate()?;
```

### 3. 从文件加载并应用环境变量覆盖

```rust
use aerox_config::ServerConfig;

let config = ServerConfig::from_file_with_env("server.toml")?;
config.validate()?;
```

### 4. 仅使用环境变量覆盖

```rust
use aerox_config::ServerConfig;

let config = ServerConfig::default()
    .load_with_env_override()?;
config.validate()?;
```

### 5. 查看配置摘要

```rust
use aerox_config::ServerConfig;

let config = ServerConfig::default();
println!("{}", config.summary());
```

## 配置验证

配置系统会自动验证以下内容：

1. **端口验证**: 端口不能为 0
2. **地址验证**: 绑定地址不能为空
3. **工作线程数验证**: 不能为 0，建议不超过 512
4. **最大连接数验证**: 不能为 0
5. **请求数限制验证**: 不能为 0

## 最佳实践

1. **配置文件管理**
   - 将配置文件放在项目根目录或 `/etc/aerox/`
   - 使用版本控制管理示例配置文件
   - 生产环境配置文件不应提交到版本控制

2. **环境变量使用**
   - 在容器化部署中使用环境变量
   - 敏感信息（如密钥）应使用环境变量
   - 环境变量的优先级高于配置文件

3. **配置验证**
   - 启动前始终验证配置
   - 在 CI/CD 流程中包含配置验证
   - 提供清晰的错误信息

## 示例配置文件

完整示例请参考：`examples/config_example.toml`

## 相关文档

- [API 文档](https://docs.rs/aerox-config)
- [总体架构](./architecture.md)
- [开发计划](./aerox_plan.md)
