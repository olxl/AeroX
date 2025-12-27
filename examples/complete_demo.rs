//! AeroX 完整示例 - 服务器和客户端通信演示
//!
//! 这个示例展示了如何使用 AeroX 客户端库与服务器进行完整的通信。
//!
//! ## 运行方式
//!
//! ### 启动服务器:
//! ```bash
//! cargo run --example complete_demo -- server
//! ```
//!
//! ### 启动客户端:
//! ```bash
//! cargo run --example complete_demo -- client
//! ```

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use aerox_client::StreamClient;
use aerox_core::{Result, AeroXError};
use prost::Message;

// Protobuf 消息定义 (简化版本，实际项目中应该在 .proto 文件中定义)
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

/// 运行 AeroX 服务器
pub async fn run_server() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║     AeroX 完整示例 - 服务器           ║");
    println!("╚════════════════════════════════════════╝\n");

    let bind_addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .map_err(|e| AeroXError::validation(format!("Invalid address: {}", e)))?;
    println!("🚀 启动服务器...");
    println!("   地址: {}\n", bind_addr);

    let listener = TcpListener::bind(bind_addr).await?;
    println!("✓ 服务器启动成功，等待连接...\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
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

/// 处理客户端连接
async fn handle_client(
    mut socket: tokio::net::TcpStream,
    addr: SocketAddr,
    conn_id: usize,
) -> Result<()> {
    println!("   ↳ 连接 #{} 已建立", conn_id);

    let mut buffer = [0u8; 8192];
    let mut messages_received = 0u64;

    // 读取消息头 (8字节: msg_id(2) + seq_id(4) + length(2))
    loop {
        // 读取消息头
        match socket.read_exact(&mut buffer[..8]).await {
            Ok(_) => {}
            Err(e) => {
                println!("   ↳ 连接 #{} 已关闭 (接收 {} 条消息)", conn_id, messages_received);
                break;
            }
        }

        // 解析消息头
        let msg_id = u16::from_be_bytes([buffer[0], buffer[1]]);
        let _seq_id = u32::from_be_bytes([buffer[2], buffer[3], buffer[4], buffer[5]]);
        let payload_len = u16::from_be_bytes([buffer[6], buffer[7]]) as usize;

        // 读取消息体
        if payload_len > 0 {
            socket.read_exact(&mut buffer[..payload_len]).await?;
            let payload = &buffer[..payload_len];

            messages_received += 1;

            // 根据消息ID处理不同类型的消息
            match msg_id {
                MSG_ID_PING_REQUEST => {
                    if let Ok(ping) = PingRequest::decode(payload) {
                        println!("   ↳ [PING] 来自 {}: {}", addr, ping.message);

                        // 构造响应
                        let response = PingResponse {
                            request_timestamp: ping.timestamp,
                            server_timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            message: format!("PONG from server (conn #{})", conn_id),
                        };

                        // 发送响应
                        send_message(&mut socket, MSG_ID_PING_RESPONSE, &response).await?;
                        println!("   ↳ [PONG] 发送响应");
                    }
                }
                MSG_ID_CHAT => {
                    if let Ok(chat) = ChatMessage::decode(payload) {
                        println!("   ↳ [CHAT] {}: {}", chat.username, chat.content);

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
                        send_message(&mut socket, MSG_ID_BROADCAST, &broadcast).await?;
                        println!("   ↳ [BROADCAST] 消息已广播");
                    }
                }
                _ => {
                    println!("   ↳ [UNKNOWN] 收到未知消息类型: {}", msg_id);
                }
            }
        }
    }

    Ok(())
}

/// 发送消息到客户端
async fn send_message<M: prost::Message>(
    socket: &mut tokio::net::TcpStream,
    msg_id: u16,
    message: &M,
) -> Result<()> {
    let mut buf = Vec::new();

    // 编码消息体
    message.encode(&mut buf)
        .map_err(|e| AeroXError::protocol(format!("Encoding failed: {}", e)))?;

    let payload_len = buf.len() as u16;

    // 写入消息头
    socket.write_all(&msg_id.to_be_bytes()).await?;
    socket.write_all(&0u32.to_be_bytes()).await?; // seq_id (简化为0)
    socket.write_all(&payload_len.to_be_bytes()).await?;

    // 写入消息体
    if !buf.is_empty() {
        socket.write_all(&buf).await?;
    }

    Ok(())
}

// ==================== 客户端实现 ====================

/// 运行 AeroX 客户端
pub async fn run_client() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║     AeroX 完整示例 - 客户端           ║");
    println!("╚════════════════════════════════════════╝\n");

    let server_addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .map_err(|e| AeroXError::validation(format!("Invalid address: {}", e)))?;
    println!("🔗 连接到服务器: {}\n", server_addr);

    // 连接服务器
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

    // 等待一下
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 场景 2: 聊天消息测试
    if let Err(e) = test_chat_message(&mut client).await {
        eprintln!("❌ 聊天消息测试失败: {}", e);
    }

    // 等待一下
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
        message: "Hello from client!".to_string(),
    };

    println!("   → 发送 PING 请求: {}", ping.message);
    client.send_message(MSG_ID_PING_REQUEST, &ping).await?;

    // 接收 Pong
    let (msg_id, pong) = client.recv_message::<PingResponse>().await?;

    if msg_id == MSG_ID_PING_RESPONSE {
        println!("   ← 收到 PONG 响应: {}", pong.message);
        println!("   → 往返时间: {} ms",
            pong.server_timestamp.saturating_sub(pong.request_timestamp));

        // 验证时间戳匹配
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
        println!("║     AeroX 完整示例                      ║");
        println!("╚════════════════════════════════════════╝\n");
        println!("用法:");
        println!("  启动服务器: cargo run --example complete_demo -- server");
        println!("  启动客户端: cargo run --example complete_demo -- client\n");
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
