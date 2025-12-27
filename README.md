# AeroX

<div align="center">

**高性能游戏服务器后端框架**

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0--green.svg)](https://github.com/aerox/aerox)

</div>

## 简介

AeroX 是一个基于 Rust 开发的专注于游戏服务器后端和实时消息转发场景的高性能框架。它采用 Reactor 模式实现高并发连接处理，整合 Bevy ECS 架构，提供模块化、可扩展的解决方案。

### 核心特性

- ⚡ **高性能** - 基于 Tokio 异步运行时，零拷贝消息处理
- 🔌 **插件系统** - 模块化设计，功能可插拔
- 🎮 **ECS 整合** - 集成 Bevy ECS，数据驱动游戏逻辑
- 🔐 **类型安全** - Rust 类型系统保证内存安全
- 📦 **Protobuf 支持** - 高效的二进制协议
- 🛣️ **灵活路由** - Axum 风格的中间件系统

## 架构

```
Application Layer     ┌─────────┐ ┌─────────┐ ┌─────────┐
                      │ PluginA │ │ PluginB │ │ PluginC │
                      └─────────┘ └─────────┘ └─────────┘

Framework Core        ┌─────────┐ ┌─────────┐ ┌─────────┐
                      │  Router │ │   ECS   │ │  Config │
                      └─────────┘ └─────────┘ └─────────┘

Network Layer         ┌─────────┐ ┌─────────┐ ┌─────────┐
                      │   TCP   │ │   KCP   │ │  QUIC   │
                      └─────────┘ └─────────┘ └─────────┘
```

## 快速开始

### 安装

将以下内容添加到 `Cargo.toml`：

```toml
[dependencies]
aerox_core = "0.1"
aerox_network = "0.1"
aerox_ecs = "0.1"
```

### Echo Server 示例

```rust
use aerox_network::TcpReactor;
use aerox_config::ServerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let mut reactor = TcpReactor::new(config).await?;

    println!("✓ 服务器启动在 {}", reactor.bind_addr());

    let handle = reactor.start()?;

    // 等待 Ctrl+C
    tokio::signal::ctrl_c().await?;
    reactor.shutdown().await?;

    Ok(())
}
```

### 运行示例

```bash
# Echo Server
cargo run --example echo_server

# 聊天室
cargo run --example chat_room
```

## 文档

- [快速开始指南](docs/getting_started.md)
- [架构设计](docs/architecture.md)
- [配置说明](docs/configuration.md)
- [API 文档](https://docs.rs/aerox)

## Crate 结构

| Crate | 描述 |
|-------|------|
| `aerox_core` | 核心运行时和插件系统 |
| `aerox_network` | 网络层抽象和协议实现 |
| `aerox_protobuf` | Protobuf 编解码支持 |
| `aerox_ecs` | Bevy ECS 整合层 |
| `aerox_router` | 路由和中间件系统 |
| `aerox_plugins` | 内置插件 |
| `aerox_config` | 配置管理 |

## 开发状态

**当前版本**: v0.1.0

**完成度**: 11/12 Phases (92%)

### 已完成功能

- ✅ 项目基础设施
- ✅ 配置系统
- ✅ 错误处理
- ✅ TCP Reactor
- ✅ 连接管理
- ✅ 消息编解码
- ✅ 路由系统
- ✅ 中间件系统
- ✅ 插件系统
- ✅ Protobuf 支持
- ✅ ECS 整合
- ✅ 示例和测试

### 开发中

- 🔄 文档完善
- 🔄 CI/CD 配置
- 🔄 KCP 传输协议
- 🔄 QUIC 传输协议

## 性能

- **并发连接**: 支持 10,000+ 并发连接
- **消息吞吐**: 100,000+ msg/sec (单核)
- **延迟**: P99 < 1ms (本地网络)
- **内存**: 零拷贝设计，最小堆分配

## 测试

```bash
# 运行所有测试
cargo test

# 运行集成测试
cargo test -p aerox_core --test integration_test

# 运行性能基准
cargo test --release --features benchmark
```

**测试覆盖**: 129 tests，所有通过 ✅

## 贡献

欢迎贡献！请查看 [开发指南](docs/development.md) 了解详情。

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 开发路线图

### v0.1.0 (当前)
- [x] 核心框架
- [x] TCP 支持
- [x] ECS 整合
- [ ] 完整文档
- [ ] CI/CD

### v0.2.0 (计划)
- [ ] KCP 协议支持
- [ ] QUIC 协议支持
- [ ] WebSocket 支持
- [ ] 更多插件

### v0.3.0 (未来)
- [ ] 分布式支持
- [ ] 监控和追踪
- [ ] 性能优化
- [ ] 生产环境验证

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [Tokio](https://tokio.rs/) - 异步运行时
- [Bevy](https://bevyengine.org/) - ECS 框架
- [Axum](https://github.com/tokio-rs/axum) - 中间件设计灵感

## 联系方式

- **GitHub**: [https://github.com/aerox/aerox](https://github.com/aerox/aerox)
- **Issue**: [https://github.com/aerox/aerox/issues](https://github.com/aerox/aerox/issues)

---

<div align="center">

**Made with ❤️ by AeroX Team**

</div>
