# AeroX Server-Client 通信演示

这个示例展示了如何使用 AeroX 的网络组件进行 server-client 通信，**而不是直接使用 tokio tcplistener**。

## 特性

### 服务器端
- ✅ 使用 AeroX 的 `Frame` 结构（消息帧）
- ✅ 使用 AeroX 的 `MessageCodec`（编解码器）
- ✅ 使用 `tokio_util::codec::Framed` 自动处理帧边界
- ✅ 支持 Protobuf 消息序列化
- ✅ 遵循 Length-Prefix-Message 协议格式

### 客户端
- ✅ 使用 AeroX 的 `StreamClient` 客户端库
- ✅ 自动编解码消息
- ✅ 简洁的 API 接口

## 协议格式

```
+--------+--------+--------+----------+
| Length | Msg ID | Seq ID |   Body   |
| 4 bytes| 2 bytes| 4 bytes| variable |
+--------+--------+--------+----------+
```

- **Length**: 4 字节小端序，帧内容大小（不含长度字段）
- **Msg ID**: 2 字节小端序，消息类型 ID
- **Seq ID**: 4 字节小端序，序列号（用于请求匹配）
- **Body**: 变长，Protobuf 编码的消息体

## 运行方式

### 1. 启动服务器

在一个终端运行：

```bash
cargo run --example aerox_communication_demo -- server
```

服务器会：
- 绑定到 `127.0.0.1:8080`
- 显示使用的 AeroX 网络组件
- 显示支持的消息类型
- 等待客户端连接

### 2. 启动客户端

在另一个终端运行：

```bash
cargo run --example aerox_communication_demo -- client
```

客户端会执行三个测试场景：
1. **Ping-Pong 测试**: 发送 PING 请求，接收 PONG 响应
2. **聊天消息测试**: 发送聊天消息，接收广播响应
3. **批量消息测试**: 连续发送多条消息，验证稳定性

## 代码结构

### 服务器端关键代码

```rust
use aerox_network::{Frame, MessageCodec};
use tokio_util::codec::Framed;
use futures_util::{SinkExt, StreamExt};

// 创建 Framed，自动处理编解码
let mut framed = Framed::new(socket, MessageCodec::new());

// 接收消息（自动解码）
match framed.next().await {
    Some(Ok(frame)) => {
        // 处理 AeroX Frame
        match frame.message_id {
            MSG_ID_PING_REQUEST => {
                // 处理消息
            }
            _ => {}
        }
    }
    _ => {}
}

// 发送消息（自动编码）
let frame = Frame::new(msg_id, 0, Bytes::from(buf));
framed.send(frame).await?;
```

### 客户端关键代码

```rust
use aerox_client::StreamClient;

// 连接服务器
let mut client = StreamClient::connect(addr).await?;

// 发送消息（自动编码为 Frame）
client.send_message(MSG_ID_PING_REQUEST, &ping).await?;

// 接收消息（自动解码）
let (msg_id, response) = client.recv_message::<PingResponse>().await?;
```

## 与 tokio tcplistener 的区别

### ❌ 旧方式（complete_demo.rs）
```rust
// 手动读取字节流
let mut buffer = [0u8; 8192];
socket.read_exact(&mut buffer[..8]).await?;

// 手动解析消息头
let msg_id = u16::from_be_bytes([buffer[0], buffer[1]]);
let payload_len = u16::from_be_bytes([buffer[6], buffer[7]]) as usize;

// 手动读取消息体
socket.read_exact(&mut buffer[..payload_len]).await?;
```

### ✅ 新方式（aerox_communication_demo.rs）
```rust
// 使用 AeroX 组件，自动处理
let mut framed = Framed::new(socket, MessageCodec::new());

match framed.next().await {
    Some(Ok(frame)) => {
        // 直接使用 frame.message_id 和 frame.body
    }
    _ => {}
}
```

## 优势

1. **代码更简洁**: 不需要手动处理字节流和帧边界
2. **类型安全**: 使用 AeroX 的 Frame 类型，避免解析错误
3. **协议一致性**: 服务器和客户端使用相同的协议定义
4. **易于维护**: 协议修改只需更新 Frame 定义
5. **可扩展性**: 轻松添加新的消息类型

## 支持的消息类型

| 消息 ID | 请求类型 | 响应类型 |
|---------|----------|----------|
| 1001 | PingRequest | PingResponse |
| 2001 | ChatMessage | BroadcastMessage |

## 测试输出示例

### 服务器端
```
╔════════════════════════════════════════╗
║   AeroX Server-Client 通信演示         ║
║   服务器端                             ║
╚════════════════════════════════════════╝

🚀 启动 AeroX 服务器...
   地址: 127.0.0.1:8080
   协议: Length-Prefix-Message Frame

使用 AeroX 网络组件:
  - Frame: 消息帧结构
  - MessageDecoder: 帧解码器
  - MessageEncoder: 帧编码器
```

### 客户端
```
╔════════════════════════════════════════╗
║   AeroX Server-Client 通信演示         ║
║   客户端端                             ║
╚════════════════════════════════════════╝

🔗 连接到 AeroX 服务器: 127.0.0.1:8080
✓ 连接成功!

📝 场景 1: Ping-Pong 测试
   → 发送 PING 请求: Hello from AeroX client!
   ← 收到 PONG 响应: PONG from AeroX server (conn #1)
   ✓ 时间戳验证成功
```

## 下一步

- 查看 `examples/complete_demo.rs` 了解基本的通信模式
- 查看 `examples/highlevel_demo.rs` 了解高级客户端 API
- 查看 `aerox_network/src/protocol/frame.rs` 了解帧格式
- 查看 `aerox_client/src/lib.rs` 了解客户端 API
