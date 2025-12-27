# AeroX 架构设计文档

## 版本: v0.1.0
## 更新时间: 2025-12-27

---

## 1. 整体架构

```
┌─────────────────────────────────────────┐
│            Application Layer            │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │ PluginA │ │ PluginB │ │ PluginC │  │
│  └─────────┘ └─────────┘ └─────────┘  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│             Framework Core              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │  Router │ │   ECS   │ │  Config │  │
│  └─────────┘ └─────────┘ └─────────┘  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│           Network Abstraction           │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │   TCP   │ │   KCP   │ │   QUIC  │  │
│  └─────────┘ └─────────┘ └─────────┘  │
└─────────────────────────────────────────┘
```

---

## 2. Reactor 模式设计

### 2.1 主线程 (Reactor)

```
主线程
├── 监听端口，接受新连接
├── 连接限制检查
├── 将连接分配给工作线程
└── 管理连接生命周期
```

### 2.2 工作线程 (Worker)

```
Worker × N
├── 处理消息读取/写入
├── 执行路由处理函数
├── 触发 ECS 事件
└── 应用中间件
```

---

## 3. 模块架构

### 3.1 当前已实现模块 (v0.1.0)

#### config 模块
- **ServerConfig**: 服务器基础配置
- **ReactorConfig**: Reactor 模式配置
- **功能**: 文件加载、验证、环境变量支持（规划中）

#### error 模块
- **AeroXError**: 统一错误类型
- **Result**: 类型别名
- **功能**: 错误转换、错误链

### 3.2 规划中的模块

#### reactor 模块
- **TcpReactor**: 主 Reactor 结构
- **Acceptor**: 连接接受器
- **Worker**: 工作线程
- **ConnectionBalancer**: 连接均衡器

#### connection 模块
- **ConnectionId**: 连接唯一标识
- **Connection**: 连接元数据
- **ConnectionPool**: 连接池管理
- **ConnectionMetrics**: 连接指标

#### protocol 模块
- **Frame**: 消息帧结构
- **Encoder**: 消息序列化
- **Decoder**: 消息反序列化

#### router 模块
- **Router**: 路由表
- **Route**: 路由定义
- **Context**: 请求上下文
- **Handler**: 处理函数 trait

#### middleware 模块
- **Middleware**: 中间件 trait
- **Layer**: 中间件层
- **Next**: 调用链

#### plugin 模块
- **Plugin**: 插件 trait
- **App**: 应用构建器
- **PluginRegistry**: 插件注册表

---

## 4. 数据流设计

### 4.1 连接建立流程

```
客户端连接
    ↓
Acceptor 接受连接
    ↓
连接限制检查
    ↓
分配到 Worker
    ↓
加入 ConnectionPool
    ↓
等待消息
```

### 4.2 消息处理流程

```
接收消息
    ↓
Decoder 解码
    ↓
Middleware 链处理
    ↓
Router 路由
    ↓
Handler 处理
    ↓
ECS 事件触发
    ↓
Encoder 编码响应
    ↓
发送响应
```

---

## 5. 配置系统设计

### 5.1 配置层级

```
默认值
    ↓
配置文件 (TOML)
    ↓
环境变量
    ↓
运行时修改 (规划中)
```

### 5.2 当前配置结构

```rust
ServerConfig {
    bind_address: String,           // 绑定地址
    port: u16,                      // 端口
    max_connections: Option<u32>,   // 最大连接数
    max_requests_per_second_per_connection: Option<u32>,
    max_requests_per_second_total: Option<u32>,
    enable_ddos_protection: bool,   // DDoS 防护
    worker_threads: Option<usize>,  // 工作线程数
}

ReactorConfig {
    reactor_buffer_size: usize,     // 缓冲区大小
    batch_size: usize,              // 批处理大小
    batch_timeout_ms: u64,          // 批处理超时
    connection_timeout_secs: u64,   // 连接超时
}
```

---

## 6. 错误处理策略

### 6.1 错误分类

- **IO 错误**: 网络、文件操作错误
- **配置错误**: 配置解析、验证错误
- **协议错误**: 消息编解码错误
- **网络错误**: 连接、传输错误
- **路由错误**: 消息路由失败
- **序列化错误**: 数据序列化失败
- **连接错误**: 连接管理错误
- **超时错误**: 操作超时
- **未实现特性**: 功能未实现

### 6.2 错误处理原则

1. **使用 thiserror**: 提供清晰的错误信息
2. **错误链**: 保留底层错误上下文
3. **类型安全**: 使用强类型 Result
4. **可恢复性**: 区分可恢复和不可恢复错误

---

## 7. 性能考虑

### 7.1 零拷贝设计

- 使用 `bytes::Bytes` 管理缓冲区
- 消息引用传递
- 避免不必要的内存分配

### 7.2 并发模型

- Tokio 异步运行时
- 多 Worker 线程
- 每个连接绑定到单个 Worker

### 7.3 批处理优化

- 消息批量处理
- 批量反序列化
- 批量序列化响应

---

## 8. 安全设计

### 8.1 防护机制

- 连接频率限制
- 消息大小限制
- 协议校验
- DDoS 防护

### 8.2 认证授权（规划中）

- Token 认证
- IP 白名单
- TLS 支持

---

## 9. 测试策略

### 9.1 单元测试

- 每个模块的单元测试
- 边界条件测试
- 错误处理测试

### 9.2 集成测试

- 完整的连接流程测试
- 消息处理测试
- 并发测试

### 9.3 性能测试

- 基准测试
- 压力测试
- 内存使用分析

---

## 10. 开发里程碑

- [x] Phase 1.0: 项目基础设施
- [ ] Phase 1.1: 配置系统
- [ ] Phase 1.2: 错误处理
- [ ] Phase 1.3: TCP Reactor
- [ ] Phase 1.4: 连接管理
- [ ] Phase 1.5: 消息编解码
- [ ] Phase 1.6: 路由系统
- [ ] Phase 1.7: 中间件系统
- [ ] Phase 1.8: 插件系统
- [ ] Phase 1.9-1.12: 高级特性和发布

---

## 参考资源

- [Tokio 官方文档](https://tokio.rs/)
- [Bevy ECS 文档](https://bevyengine.org/)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
