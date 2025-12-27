//! AeroX 高级客户端 API 示例
//!
//! 这个示例展示了如何使用 HighLevelClient，它是 AeroX 提供的更高级别的客户端 API。
//! HighLevelClient 会在后台自动接收消息，并通过事件系统通知你的应用程序。
//!
//! ## 运行方式
//!
//! 首先在一个终端启动 complete_demo 服务器：
//! ```bash
//! cargo run --example complete_demo -- server
//! ```
//!
//! 然后在另一个终端运行此示例：
//! ```bash
//! cargo run --example highlevel_demo
//! ```

use std::net::SocketAddr;
use std::time::Duration;
use aerox_client::{HighLevelClient, ClientEvent};
use aerox_core::{Result, AeroXError};

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

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════━━━━━━━━━━━━━╗");
    println!("║   AeroX 高级客户端 API 示例            ║");
    println!("╚════════════════════════════━━━━━━━━━━━━━╝\n");

    let server_addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .map_err(|e| AeroXError::validation(format!("Invalid address: {}", e)))?;

    println!("🔗 连接到服务器: {}...\n", server_addr);

    // 连接到服务器
    let client = match HighLevelClient::connect(server_addr).await {
        Ok(c) => {
            println!("✓ 连接成功!\n");
            c
        }
        Err(e) => {
            eprintln!("❌ 连接失败: {}", e);
            eprintln!("\n提示: 请确保服务器正在运行:");
            eprintln!("  cargo run -p aerox_client --example complete_demo -- server\n");
            return Err(e.into());
        }
    };

    // 订阅客户端事件
    let mut event_rx = client.subscribe_events();

    // 启动事件监听任务
    let event_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                ClientEvent::Connected { addr } => {
                    println!("📡 事件: 已连接到 {}", addr);
                }
                ClientEvent::Disconnected { reason } => {
                    println!("📡 事件: 已断开连接 - {}", reason);
                }
                ClientEvent::MessageReceived { msg_id } => {
                    println!("📨 事件: 收到消息 [ID={}]", msg_id);
                }
                ClientEvent::MessageSent { msg_id } => {
                    println!("📤 事件: 发送消息 [ID={}]", msg_id);
                }
                ClientEvent::Error { error } => {
                    println!("❌ 事件: 错误 - {}", error);
                }
                ClientEvent::Reconnecting { attempt } => {
                    println!("🔄 事件: 重连中 (尝试 {})", attempt);
                }
            }
        }
    });

    // 等待连接建立
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试场景: 高级 API 功能演示");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 测试 1: 发送 Ping 消息
    println!("📝 测试 1: 发送 Ping 消息");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let ping = PingRequest {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        message: "Hello from HighLevelClient!".to_string(),
    };

    match client.send(MSG_ID_PING_REQUEST, &ping).await {
        Ok(_) => println!("   ✓ Ping 消息已发送\n"),
        Err(e) => println!("   ❌ 发送失败: {}\n", e),
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 测试 2: 发送聊天消息
    println!("📝 测试 2: 发送聊天消息");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let chat = ChatMessage {
        username: "Bob".to_string(),
        content: "使用高级客户端 API!".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    match client.send(MSG_ID_CHAT, &chat).await {
        Ok(_) => println!("   ✓ 聊天消息已发送\n"),
        Err(e) => println!("   ❌ 发送失败: {}\n", e),
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 测试 3: 批量发送消息
    println!("📝 测试 3: 批量发送消息");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for i in 1..=3 {
        let ping = PingRequest {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message: format!("批量消息 #{}", i),
        };

        match client.send(MSG_ID_PING_REQUEST, &ping).await {
            Ok(_) => println!("   → 消息 {}/3 已发送", i),
            Err(e) => println!("   ❌ 消息 {}/3 发送失败: {}", i, e),
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("   ✓ 批量发送完成\n");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 测试 4: 检查连接状态
    println!("📝 测试 4: 检查连接状态");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let state = client.state().await;
    let is_connected = client.is_connected().await;
    let server = client.server_addr().await;

    println!("   → 状态: {:?}", state);
    println!("   → 已连接: {}", is_connected);
    println!("   → 服务器地址: {}\n", server);

    // 等待事件处理完成
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("等待事件处理完成...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 关闭客户端
    println!("📝 正在关闭客户端...\n");
    client.shutdown().await?;
    event_task.abort();

    println!("✓ 客户端已关闭");

    Ok(())
}
