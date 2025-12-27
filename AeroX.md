# AeroX - 高性能游戏服务器后端框架方案书

## 1. 项目概述

### 1.1 项目背景

AeroX 是一个专注于游戏服务器后端和实时消息转发场景的高性能 Rust 框架。它旨在提供一套完整的、可扩展的解决方案，支持 TCP、KCP 和 QUIC 协议（初期实现 TCP），采用 Reactor 模式实现高并发连接处理，并整合数据驱动的 ECS 架构。

### 1.2 设计理念

- **高性能**: 基于 Tokio 异步运行时，实现零拷贝消息处理
- **模块化**: 插件化设计，功能可插拔
- **易用性**: 仿照 Bevy 的 API 设计，降低学习成本
- **可扩展**: 支持自定义协议、中间件和系统

### 1.3 核心特性

- Reactor 多线程模型
- Protobuf 消息协议
- Bevy ECS 整合
- 可配置的限流和防护机制
- Axum 风格的中间件系统

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────┐

│            Application Layer            │

│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │

│  │ PluginA │ │ PluginB │ │ PluginC │  │

│  └─────────┘ └─────────┘ └─────────┘  │

└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐

│             Framework Core              │

│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │

│  │  Router │ │  ECS    │ │  Config │  │

│  └─────────┘ └─────────┘ └─────────┘  │

└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐

│           Network Abstraction           │

│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │

│  │  TCP    │ │  KCP    │ │  QUIC   │  │

│  └─────────┘ └─────────┘ └─────────┘  │

└─────────────────────────────────────────┘
```

### 2.2 Reactor 模式实现

```
主线程 (Reactor):

├── 监听端口，接受新连接

├── 将连接分配给工作线程

└── 管理连接生命周期


工作线程 (Worker × N):

├── 处理消息读取/写入

├── 执行路由处理函数

├── 触发 ECS 事件

└── 应用中间件
```

## 3. 核心模块设计

### 3.1 Crate 结构

```
aerox/

├── aerox_core/          # 核心运行时和插件系统

├── aerox_network/       # 网络层抽象和协议实现

├── aerox_protobuf/      # Protobuf 编解码支持

├── aerox_ecs/           # Bevy ECS 整合层

├── aerox_router/        # 路由和中间件系统

├── aerox_config/        # 配置管理

└── aerox_plugins/       # 官方插件集合
```

### 3.2 模块职责说明

#### 3.2.1 aerox_core

- 应用启动器和运行时
- 插件系统 (Plugin trait)
- 系统调度器
- 资源管理和共享状态

#### 3.2.2 aerox_network

```
// 协议抽象

pub trait Transport: Send + Sync {

    async fn connect(&self, addr: &str) -> Result<Connection>;

    async fn bind(&self, addr: &str) -> Result<Listener>;

}


// Reactor 实现

pub struct TcpReactor {

    config: ReactorConfig,

    workers: Vec<Worker>,

    connection_manager: ConnectionManager,

}
```

#### 3.2.3 aerox_protobuf

- 自动消息注册和反序列化
- 消息 ID 映射
- 零拷贝缓冲区管理

#### 3.2.4 aerox_ecs

- Bevy ECS 的包装和扩展
- 网络事件到 ECS 事件的转换
- Tick 调度系统

#### 3.2.5 aerox_router

```
// 路由系统

pub struct Router {

    routes: HashMap<MessageId, Route>,

    middlewares: Vec<Box<dyn Middleware>>,

}



// 中间件 trait

pub trait Middleware {

    fn handle(&self, ctx: &mut Context, next: Next) -> impl Future<Output = Result<()>>;

}
```

## 4. 详细设计

### 4.1 配置系统

```
#[derive(Clone, Debug, Deserialize)]

pub struct ServerConfig {

    pub bind_address: String,

    pub port: u16,

    pub max_connections: Option<u32>,

    pub max_requests_per_second_per_connection: Option<u32>,

    pub max_requests_per_second_total: Option<u32>,

    pub enable_ddos_protection: bool,

    pub worker_threads: Option<usize>,

    pub reactor_buffer_size: usize,

}



impl Default for ServerConfig {

    fn default() -> Self {

        Self {

            bind_address: "0.0.0.0".to_string(),

            port: 8080,

            max_connections: None,

            max_requests_per_second_per_connection: Some(1000),

            max_requests_per_second_total: Some(100000),

            enable_ddos_protection: true,

            worker_threads: None,

            reactor_buffer_size: 8192,

        }

    }

}
```

### 4.2 Reactor 实现细节

#### 4.2.1 主线程事件循环

```
async fn reactor_main(config: &ReactorConfig) -> Result<()> {

    let listener = TcpListener::bind(&config.bind_addr).await?;

    let connection_balancer = ConnectionBalancer::new(config.worker_count);

    

    loop {

        let (socket, addr) = listener.accept().await?;

        

        // 连接限制检查

        if !connection_manager.can_accept_new_connection() {

            continue;

        }

        

        // 分配给工作线程

        let worker_id = connection_balancer.next_worker();

        let worker_tx = worker_channels[worker_id].clone();

        

        worker_tx.send(WorkerMessage::NewConnection(socket, addr)).await?;

    }

}
```

#### 4.2.2 工作线程处理

```
struct Worker {

    id: usize,

    router: Router,

    ecs_world: World,

    connection_pool: ConnectionPool,

    rate_limiter: RateLimiter,

}



impl Worker {

    async fn run(&mut self) -> Result<()> {

        loop {

            // 处理消息

            if let Some(message) = self.connection_pool.next_message().await {

                self.process_message(message).await?;

            }

            

            // 执行 ECS 系统

            self.ecs_world.run_schedule(&self.update_schedule);

        }

    }

}
```

### 4.3 路由系统设计

#### 4.3.1 消息处理注册

```
// 定义消息处理函数

#[message_handler(MessageId = 1001)]

async fn handle_player_login(

    ctx: Context,

    msg: proto::PlayerLoginRequest,

) -> Result<proto::PlayerLoginResponse> {

    // 业务逻辑

    Ok(proto::PlayerLoginResponse::default())

}



// 注册路由

app.add_route::<proto::PlayerLoginRequest, _>(handle_player_login);
```

#### 4.3.2 中间件链

```
pub struct Layer<T> {

    inner: T,

    middleware: Box<dyn Middleware>,

}



impl<T> Layer<T> {

    pub fn new(inner: T) -> Self {

        Self {

            inner,

            middleware: Box::new(IdentityMiddleware),

        }

    }

    

    pub fn layer<M>(self, middleware: M) -> Layer<T>

    where

        M: Middleware + 'static,

    {

        Layer {

            inner: self.inner,

            middleware: Box::new(middleware),

        }

    }

}
```

### 4.4 ECS 整合

#### 4.4.1 网络事件组件

```
// 网络事件组件

#[derive(Component)]

pub struct NetworkEvent {

    pub connection_id: ConnectionId,

    pub message_id: MessageId,

    pub payload: Vec<u8>,

    pub timestamp: Instant,

}



// 连接组件

#[derive(Component)]

pub struct Connection {

    pub id: ConnectionId,

    pub addr: SocketAddr,

    pub last_activity: Instant,

    pub metrics: ConnectionMetrics,

}
```

#### 4.4.2 Tick 调度系统

```
// 定义服务器阶段

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]

pub enum ServerStage {

    NetworkReceive,

    BusinessLogic,

    NetworkSend,

    Cleanup,

}



// 配置调度器

app.add_schedule(Schedule::new()

    .with_stage(ServerStage::NetworkReceive, SystemStage::parallel()

        .with_system(receive_messages_system))

    .with_stage(ServerStage::BusinessLogic, SystemStage::parallel()

        .with_system(handle_player_movement)

        .with_system(handle_chat_messages))

    .with_stage(ServerStage::NetworkSend, SystemStage::parallel()

        .with_system(send_broadcast_messages))

    .with_stage(ServerStage::Cleanup, SystemStage::parallel()

        .with_system(cleanup_disconnected_players)));
```

### 4.5 插件系统

#### 4.5.1 Plugin Trait

```
pub trait Plugin {

    fn build(&self, app: &mut App);

    fn name(&self) -> &'static str;

    fn is_required(&self) -> bool { false }

}
```

#### 4.5.2 默认插件

```
// 心跳插件

pub struct HeartbeatPlugin {

    interval: Duration,

    timeout: Duration,

}



impl Plugin for HeartbeatPlugin {

    fn build(&self, app: &mut App) {

        app.add_system(heartbeat_check_system)

            .add_system(heartbeat_response_system)

            .init_resource::<HeartbeatConfig>();

    }

}



// 限流插件

pub struct RateLimitPlugin {

    config: RateLimitConfig,

}



impl Plugin for RateLimitPlugin {

    fn build(&self, app: &mut App) {

        app.add_layer(RateLimitMiddleware::new(self.config.clone()))

            .init_resource::<RequestCounter>();

    }

}
```

## 5. 使用示例

### 5.1 基本服务器搭建

```
use aerox::prelude::*;

use aerox_plugins::{TcpPlugin, HeartbeatPlugin, RateLimitPlugin};



#[tokio::main]

async fn main() -> Result<()> {

    let config = ServerConfig {

        bind_address: "0.0.0.0".to_string(),

        port: 8888,

        max_connections: Some(10000),

        ..Default::default()

    };

    

    App::new()

        .add_plugin(TcpPlugin::with_config(config))

        .add_plugin(HeartbeatPlugin::default())

        .add_plugin(RateLimitPlugin::default())

        .add_route::<LoginRequest, _>(handle_login)

        .add_route::<ChatMessage, _>(handle_chat)

        .add_system(update_player_positions)

        .add_layer(AuthMiddleware::new())

        .add_layer(LoggingMiddleware::new())

        .run()

        .await?;

        

    Ok(())

}
```

### 5.2 自定义消息处理

```
// 定义 Protobuf 消息

syntax = "proto3";

package game;



message PlayerMove {

    float x = 1;

    float y = 2;

    float z = 3;

}



// Rust 处理函数

#[message_handler(MessageId = 1002)]

async fn handle_player_move(

    mut ctx: Context,

    msg: PlayerMove,

    query: Query<&mut Transform, With<Player>>,

) -> Result<()> {

    let player_id = ctx.connection_id();

    

    for mut transform in query.iter_mut() {

        if transform.player_id == player_id {

            transform.position.x = msg.x;

            transform.position.y = msg.y;

            transform.position.z = msg.z;

            break;

        }

    }

    

    Ok(())

}
```

## 6. 高级特性

### 6.1 连接管理

- 连接池和复用
- 连接迁移支持
- 优雅关闭机制

### 6.2 监控和指标

```
#[derive(Resource)]

pub struct ServerMetrics {

    pub connections_total: AtomicU64,

    pub messages_processed: AtomicU64,

    pub avg_response_time: ExponentialMovingAverage,

    pub error_rate: ExponentialMovingAverage,

}



// 提供 Prometheus 端点

pub struct MetricsPlugin;



impl Plugin for MetricsPlugin {

    fn build(&self, app: &mut App) {

        app.init_resource::<ServerMetrics>()

            .add_system(collect_metrics_system)

            .add_route::<MetricsRequest, _>(handle_metrics_request);

    }

}
```

### 6.3 热重载支持

- 配置热重载
- 路由动态更新
- 插件热插拔

### 6.4 测试工具

- 集成测试框架
- 压力测试工具
- 协议模糊测试

## 7. 性能优化策略

### 7.1 零拷贝设计

- 使用 bytes::Bytes 管理缓冲区
- 消息引用传递
- 连接级别的内存池

### 7.2 批处理优化

```
// 消息批处理系统

pub struct BatchProcessor {

    batch_size: usize,

    batch_timeout: Duration,

}



impl BatchProcessor {

    async fn process_batch(&self, messages: Vec<NetworkMessage>) {

        // 批量反序列化

        // 批量执行业务逻辑

        // 批量序列化响应

    }

}
```

### 7.3 连接多路复用

- 单连接多通道支持
- 优先级消息队列
- 流量整形

## 8. 安全设计

### 8.1 防攻击措施

- 连接频率限制
- 消息大小限制
- 协议校验强化
- IP 黑白名单

### 8.2 加密和认证

- TLS 支持
- 自定义加密协议
- Token 认证中间件

## 9. 开发路线图

### Phase 1 (MVP)

1. TCP Reactor 基础实现
2. 基本路由系统
3. Protobuf 消息支持
4.  ECS 整合

### Phase 2 (功能完善)

1. 完整的中间件系统
2. 配置管理和热重载
3. 监控和指标
4. 官方插件集合

### Phase 4 (生态建设)

1. 管理控制台
2. 客户端 SDK
3. 示例项目和模板
4. 性能基准测试套件

### Phase 3 (高级特性)

1. KCP 协议支持
2. QUIC 协议支持
3. 集群和分布式支持
4. WebSocket 支持

## 10. 总结

AeroX 框架旨在为游戏服务器开发提供一个高性能、可扩展的基础设施。通过结合 Reactor 模式、ECS 架构和现代化的异步 Rust 生态系统，它能够在保持高性能的同时提供优秀的开发体验。

框架的核心优势在于：

1. **性能优先**: 零拷贝设计、高效的内存管理
2. **开发友好**: 类似 Bevy 的 API 设计，降低学习成本
3. **高度可扩展**: 插件化架构，可按需组合功能
4. **生产就绪**: 内置监控、限流、防护等企业级特性
5. **协议灵活**: 支持多种传输协议，适应不同网络环境

该框架特别适合需要处理大量并发连接、低延迟要求的实时应用场景，如 MMO 游戏、实时协作工具、物联网平台等。