//! AeroX Server-Client 通信演示
//!
//! 这个示例展示了如何使用 AeroX 的网络组件进行 server-client 通信：
//! - 服务器端使用 AeroX 的 Frame、MessageCodec 等协议组件
//! - 客户端使用 AeroX 的 StreamClient 客户端库
//! - 双方遵循相同的通信协议（Length-Prefix-Message 格式）
//!
//! ## 运行方式
//!
//! ### 启动服务器:
//! ```bash
//! cargo run --example aerox_communication_demo -- server
//! ```
//!
//! ### 启动客户端:
//! ```bash
//! cargo run --example aerox_communication_demo -- client
//! ```

use std::net::SocketAddr;
use bytes::Bytes;
use tokio::net::TcpListener;
use tokio_util::codec::Framed;
use futures_util::{SinkExt, StreamExt};
use aerox_client::StreamClient;
use aerox_network::{Frame, MessageCodec};
use aerox_core::{Result, AeroXError};
use prost::Message;

// Protobuf 消息定义
#[derive(Clone, prost::Message)]
pub struct PingRequest {
    #[prost(uint64, tag = "1")]
    pub timestamp: u64,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, prost::Message)]
pub struct PingResponse {
    #[prost(uint64, tag = "1")]
    pub request_timestamp: u64,
    #[prost(uint64, tag = "2")]
    pub server_timestamp: u64,
    #[prost(string, tag = "3")]
    pub message: String,
}

#[derive(Clone, prost::Message)]
pub struct ChatMessage {
    #[prost(string, tag = "1")]
    pub username: String,
    #[prost(string, tag = "2")]
    pub content: String,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}

#[derive(Clone, prost::Message)]
pub struct BroadcastMessage {
    #[prost(string, tag = "1")]
    pub from_server: String,
    #[prost(string, tag = "2")]
    pub content: String,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}

// 消息 ID 常量
const MSG_ID_PING_REQUEST: u16 = 1001;
const MSG_ID_PING_RESPONSE: u16 = 1002;
const MSG_ID_CHAT: u16 = 2001;
const MSG_ID_BROADCAST: u16 = 2002;

// ==================== 服务器实现 ====================

/// 运行 AeroX 服务器（使用 AeroX 协议组件）
pub async fn run_server() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX Server-Client 通信演示         ║");
    println!("║   服务器端                             ║");
    println!("╚════════════════════════════════════════╝\n");

    let bind_addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .map_err(|e| AeroXError::validation(format!("Invalid address: {}", e)))?;

    println!("🚀 启动 AeroX 服务器...");
    println!("   地址: {}", bind_addr);
    println!("   协议: Length-Prefix-Message Frame\n");

    let listener = TcpListener::bind(bind_addr).await?;
    println!("✓ 服务器启动成功，等待连接...\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("使用 AeroX 网络组件:");
    println!("  - Frame: 消息帧结构");
    println!("  - MessageDecoder: 帧解码器");
    println!("  - MessageEncoder: 帧编码器");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("支持的消息类型:");
    println!("  [1001] PingRequest  -> PingResponse");
    println!("  [2001] ChatMessage  -> BroadcastMessage");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut connection_count = 0;

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                connection_count += 1;
                println!("📥 新连接 #{} 来自: {}", connection_count, addr);

                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, addr, connection_count).await {
                        eprintln!("❌ 连接 #{} 错误: {}", connection_count, e);
                    }
                });
            }
            Err(e) => {
                eprintln!("❌ 接受连接失败: {}", e);
            }
        }
    }
}

/// 处理客户端连接（使用 AeroX Frame 和 Codec）
async fn handle_client(
    socket: tokio::net::TcpStream,
    addr: SocketAddr,
    conn_id: usize,
) -> Result<()> {
    println!("   ↳ 连接 #{} 已建立", conn_id);

    // 使用 AeroX 的 MessageCodec 创建 Framed
    // Framed 会自动处理帧的边界，我们只需处理完整的 Frame
    let mut framed = Framed::new(socket, MessageCodec::new());
    let mut messages_received = 0u64;

    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                messages_received += 1;

                // 使用 AeroX Frame 处理消息
                println!("   ↳ 连接 #{} 收到 Frame: {}", conn_id, frame);

                match frame.message_id {
                    MSG_ID_PING_REQUEST => {
                        handle_ping_request(&frame, addr, conn_id, &mut framed).await?;
                    }
                    MSG_ID_CHAT => {
                        handle_chat_message(&frame, addr, conn_id, &mut framed).await?;
                    }
                    _ => {
                        println!("   ↳ 连接 #{} 收到未知消息类型: {}", conn_id, frame.message_id);
                    }
                }
            }
            Some(Err(e)) => {
                eprintln!("   ↳ 连接 #{} 解码错误: {}", conn_id, e);
                break;
            }
            None => {
                println!("   ↳ 连接 #{} 已关闭 (接收 {} 条消息)", conn_id, messages_received);
                break;
            }
        }
    }

    Ok(())
}

/// 处理 Ping 请求（使用 AeroX Frame）
async fn handle_ping_request(
    frame: &Frame,
    addr: SocketAddr,
    conn_id: usize,
    framed: &mut Framed<tokio::net::TcpStream, MessageCodec>,
) -> Result<()> {
    // 解码 Protobuf 消息
    if let Ok(ping) = PingRequest::decode(&frame.body[..]) {
        println!("   ↳ [PING] 连接 #{} 来自 {}: {}", conn_id, addr, ping.message);

        // 构造响应
        let response = PingResponse {
            request_timestamp: ping.timestamp,
            server_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message: format!("PONG from AeroX server (conn #{})", conn_id),
        };

        // 使用 AeroX Frame 编码响应
        send_frame(framed, MSG_ID_PING_RESPONSE, &response).await?;
        println!("   ↳ [PONG] 连接 #{} 发送响应", conn_id);
    }

    Ok(())
}

/// 处理聊天消息（使用 AeroX Frame）
async fn handle_chat_message(
    frame: &Frame,
    _addr: SocketAddr,
    conn_id: usize,
    framed: &mut Framed<tokio::net::TcpStream, MessageCodec>,
) -> Result<()> {
    if let Ok(chat) = ChatMessage::decode(&frame.body[..]) {
        println!("   ↳ [CHAT] 连接 #{} {}: {}", conn_id, chat.username, chat.content);

        // 构造广播消息
        let broadcast = BroadcastMessage {
            from_server: format!("User {} via conn #{}", chat.username, conn_id),
            content: chat.content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // 发送广播响应
        send_frame(framed, MSG_ID_BROADCAST, &broadcast).await?;
        println!("   ↳ [BROADCAST] 连接 #{} 消息已广播", conn_id);
    }

    Ok(())
}

/// 发送 AeroX Frame（使用 MessageCodec）
async fn send_frame<M: prost::Message>(
    framed: &mut Framed<tokio::net::TcpStream, MessageCodec>,
    msg_id: u16,
    message: &M,
) -> Result<()> {
    // 编码 Protobuf 消息
    let mut buf = Vec::new();
    message.encode(&mut buf)
        .map_err(|e| AeroXError::protocol(format!("Encoding failed: {}", e)))?;

    // 创建 AeroX Frame
    let frame = Frame::new(msg_id, 0, Bytes::from(buf));

    // 使用 Framed 发送（自动使用 MessageCodec）
    framed.send(frame).await
        .map_err(|e| AeroXError::network(format!("Send failed: {}", e)))?;

    Ok(())
}

// ==================== 客户端实现 ====================

/// 运行 AeroX 客户端（使用 AeroX StreamClient）
pub async fn run_client() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX Server-Client 通信演示         ║");
    println!("║   客户端端                             ║");
    println!("╚════════════════════════════════════════╝\n");

    let server_addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .map_err(|e| AeroXError::validation(format!("Invalid address: {}", e)))?;

    println!("🔗 连接到 AeroX 服务器: {}", server_addr);
    println!("   使用 AeroX StreamClient\n");

    // 连接服务器（StreamClient 内部使用 AeroX 协议）
    let mut client = match StreamClient::connect(server_addr).await {
        Ok(c) => {
            println!("✓ 连接成功!\n");
            c
        }
        Err(e) => {
            eprintln!("❌ 连接失败: {}", e);
            return Err(e.into());
        }
    };

    // 执行测试场景
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("开始执行测试场景...\n");

    // 场景 1: Ping-Pong 测试
    if let Err(e) = test_ping_pong(&mut client).await {
        eprintln!("❌ Ping-Pong 测试失败: {}", e);
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 场景 2: 聊天消息测试
    if let Err(e) = test_chat_message(&mut client).await {
        eprintln!("❌ 聊天消息测试失败: {}", e);
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 场景 3: 批量消息测试
    if let Err(e) = test_batch_messages(&mut client).await {
        eprintln!("❌ 批量消息测试失败: {}", e);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 所有测试完成!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 关闭连接
    client.close().await?;
    println!("✓ 连接已关闭");

    Ok(())
}

/// 测试 Ping-Pong
async fn test_ping_pong(client: &mut StreamClient) -> Result<()> {
    println!("📝 场景 1: Ping-Pong 测试");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 发送 Ping
    let ping = PingRequest {
        timestamp,
        message: "Hello from AeroX client!".to_string(),
    };

    println!("   → 发送 PING 请求: {}", ping.message);
    client.send_message(MSG_ID_PING_REQUEST, &ping).await?;

    // 接收 Pong
    let (msg_id, pong) = client.recv_message::<PingResponse>().await?;

    if msg_id == MSG_ID_PING_RESPONSE {
        println!("   ← 收到 PONG 响应: {}", pong.message);
        println!("   → 往返时间: {} ms",
            pong.server_timestamp.saturating_sub(pong.request_timestamp));

        if pong.request_timestamp == timestamp {
            println!("   ✓ 时间戳验证成功");
        } else {
            println!("   ⚠ 时间戳不匹配");
        }
    }

    println!();
    Ok(())
}

/// 测试聊天消息
async fn test_chat_message(client: &mut StreamClient) -> Result<()> {
    println!("📝 场景 2: 聊天消息测试");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let chat = ChatMessage {
        username: "Alice".to_string(),
        content: "你好，AeroX!".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    println!("   → 发送聊天消息: {}: {}", chat.username, chat.content);
    client.send_message(MSG_ID_CHAT, &chat).await?;

    // 接收广播响应
    let (msg_id, broadcast) = client.recv_message::<BroadcastMessage>().await?;

    if msg_id == MSG_ID_BROADCAST {
        println!("   ← 收到广播: [{}] {}", broadcast.from_server, broadcast.content);
    }

    println!();
    Ok(())
}

/// 测试批量消息
async fn test_batch_messages(client: &mut StreamClient) -> Result<()> {
    println!("📝 场景 3: 批量消息测试");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let messages = vec![
        "消息 1: 测试开始",
        "消息 2: 批量发送",
        "消息 3: 性能测试",
    ];

    for (i, msg_text) in messages.iter().enumerate() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let ping = PingRequest {
            timestamp,
            message: msg_text.to_string(),
        };

        println!("   → 发送消息 {}/3: {}", i + 1, msg_text);
        client.send_message(MSG_ID_PING_REQUEST, &ping).await?;

        // 接收响应
        let (msg_id, _pong) = client.recv_message::<PingResponse>().await?;
        if msg_id == MSG_ID_PING_RESPONSE {
            println!("   ← 收到响应 {}/3", i + 1);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("   ✓ 批量消息测试完成");
    println!();
    Ok(())
}

// ==================== 主函数 ====================

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("╔════════════════════════════════════════╗");
        println!("║   AeroX Server-Client 通信演示         ║");
        println!("╚════════════════════════════════════════╝\n");
        println!("用法:");
        println!("  启动服务器: cargo run --example aerox_communication_demo -- server");
        println!("  启动客户端: cargo run --example aerox_communication_demo -- client\n");
        println!("特性:");
        println!("  ✓ 服务器使用 AeroX Frame 和 MessageCodec");
        println!("  ✓ 客户端使用 AeroX StreamClient");
        println!("  ✓ 双方遵循相同的 Length-Prefix-Message 协议\n");
        println!("请先启动服务器，然后在另一个终端启动客户端。\n");
        return Ok(());
    }

    match args[1].as_str() {
        "server" => run_server().await,
        "client" => run_client().await,
        _ => {
            eprintln!("❌ 未知参数: {}", args[1]);
            eprintln!("   使用 'server' 或 'client'");
            Err(AeroXError::validation("Invalid argument".to_string()))
        }
    }
}
