# AeroX 快速开始指南

本指南将帮助你在 5 分钟内启动并运行 AeroX 服务器。

## 前置要求

- Rust 1.75 或更高版本
- 操作系统: Linux / macOS / Windows

## 安装

### 创建新项目

```bash
cargo new my_game_server
cd my_game_server
```

### 添加依赖

编辑 `Cargo.toml`:

```toml
[dependencies]
aerox_core = "0.1"
aerox_network = "0.1"
aerox_ecs = "0.1"
tokio = { version = "1.40", features = ["full"] }
```

## 基础示例

### 1. Hello World 服务器

创建 `src/main.rs`:

```rust
use aerox_config::ServerConfig;
use aerox_network::TcpReactor;
use aerox_core::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = ServerConfig::default();

    // 创建 Reactor
    let mut reactor = TcpReactor::new(config).await?;

    println!("🚀 服务器启动在 {}", reactor.bind_addr());

    // 启动服务器
    let handle = reactor.start()?;

    // 等待 Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\n⏸️  正在关闭服务器...");

    // 优雅关闭
    reactor.shutdown().await?;
    println!("✓ 服务器已关闭");

    Ok(())
}
```

运行服务器：

```bash
cargo run
```

在另一个终端连接：

```bash
telnet 127.0.0.1 8080
```

### 2. 使用配置文件

创建 `config.toml`:

```toml
[server]
bind_address = "127.0.0.1"
port = 9999
max_connections = 1000

[reactor]
worker_threads = 4
```

修改 `main.rs` 使用配置文件：

```rust
use aerox_config::ServerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // 从文件加载配置
    let config = ServerConfig::from_file("config.toml")?;

    let mut reactor = TcpReactor::new(config).await?;
    // ... 其余代码
}
```

### 3. 添加消息处理

使用路由系统处理不同类型的消息：

```rust
use aerox_router::{Router, Context};
use aerox_network::Frame;
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig::default();
    let mut reactor = TcpReactor::new(config).await?;

    // 创建路由器
    let mut router = Router::new();

    // 注册消息处理器
    router.register(1, |ctx: Context| async move {
        println!("收到消息: {:?}", ctx.data);
        // 处理消息...
        Ok(ctx)
    });

    // 启动服务器
    let handle = reactor.start()?;

    tokio::signal::ctrl_c().await?;
    reactor.shutdown().await?;

    Ok(())
}
```

### 4. 使用 ECS

集成 Bevy ECS 处理游戏逻辑：

```rust
use aerox_ecs::{EcsWorld, NetworkBridge};
use aerox_ecs::components::*;
use aerox_network::ConnectionId;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 ECS World
    let mut world = EcsWorld::new();
    world.initialize()?;

    // 创建网络桥接器
    let bridge = NetworkBridge::new();

    // 模拟新玩家连接
    let conn_id = ConnectionId::new(1);
    let addr = "127.0.0.1:8080".parse()?;

    // 桥接连接事件到 ECS
    bridge.on_connected(&mut world, conn_id, addr);

    // 创建玩家实体
    world.spawn_bundle((
        PlayerConnection::new(conn_id, addr),
        Position::origin(),
        Health::full(100.0),
    ));

    println!("✓ ECS 初始化完成，实体数: {}", world.metrics().entity_count);

    Ok(())
}
```

### 5. 使用插件

创建自定义插件：

```rust
use aerox_core::plugin::Plugin;

struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn build(&self) {
        println!("✓ LoggingPlugin 已加载");
    }

    fn name(&self) -> &'static str {
        "logging"
    }

    fn is_required(&self) -> bool {
        true
    }
}

// 在 main 中注册
fn main() -> Result<()> {
    let mut app = aerox_core::App::new();
    app.add_plugin(LoggingPlugin);
    app.build()?;

    Ok(())
}
```

## 高级用法

### 环境变量覆盖

```bash
export AEROX_PORT=9999
export AEROX_MAX_CONNECTIONS=5000
cargo run
```

### 自定义中间件

```rust
use aerox_router::middleware::*;

// 创建中间件栈
let mut stack = MiddlewareStack::new();

// 添加日志中间件
stack.add(LoggingMiddleware::new());

// 添加超时中间件
stack.add(TimeoutMiddleware::new(Duration::from_secs(5)));

// 应用到路由
router.apply_middleware(stack);
```

### Protobuf 消息

```rust
use aerox_protobuf::MessageRegistry;

let registry = MessageRegistry::new();

// 编码消息
let payload = Bytes::from("hello");
let wrapped = registry.wrap_message(1, 100, payload);

// 解码消息
let (msg_id, seq, data) = registry.unwrap_message(&wrapped)?;
```

## 运行示例

项目包含多个示例程序：

```bash
# Echo 服务器 - 回显所有消息
cargo run --example echo_server

# 聊天室 - 多客户端聊天
cargo run --example chat_room

# 配置加载示例
cargo run --example load_config_example

# 环境变量覆盖示例
cargo run --example env_override_example

# 插件系统示例
cargo run --example hello_plugin

# Protobuf 使用示例
cargo run --example protobuf_usage
```

## 测试

运行所有测试：

```bash
# 单元测试
cargo test

# 集成测试
cargo test -p aerox_core --test integration_test

# 带输出的测试
cargo test -- --nocapture

# 运行特定测试
cargo test test_connection_id
```

## 性能优化

### 调整 Worker 线程数

```toml
[reactor]
worker_threads = 8  # 根据 CPU 核心数调整
```

### 配置连接限制

```toml
[server]
max_connections = 10000

[connection]
timeout = 30
keepalive = true
```

### 零拷贝优化

```rust
// 使用 Bytes 而不是 Vec<u8>
use bytes::Bytes;

let data = Bytes::from("hello");

// 零拷贝切片
let slice = data.slice(0..2);
```

## 常见问题

### Q: 如何选择端口？

A: 使用环境变量或配置文件：

```bash
export AEROX_PORT=9999
```

### Q: 如何处理大量连接？

A: 调整 `worker_threads` 和 `max_connections`，参考[配置说明](configuration.md)。

### Q: ECS 如何提升性能？

A: 查看 [ECS 架构文档](architecture.md#ecs) 和组件设计最佳实践。

### Q: 如何调试？

A: 启用日志：

```rust
env_logger::init();

#[tokio::main]
async fn main() -> Result<()> {
    // ...
}
```

运行时：

```bash
RUST_LOG=debug cargo run
```

## 下一步

- 阅读 [架构设计](architecture.md)
- 查看 [API 文档](https://docs.rs/aerox)
- 浏览 [示例代码](../examples/)
- 加入 [社区讨论](https://github.com/aerox/aerox/discussions)

## 获取帮助

- 📖 查看 [文档](docs/)
- 💬 提交 [Issue](https://github.com/aerox/aerox/issues)
- 📧 发送邮件到 support@aerox.rs

---

祝使用愉快！🎉
