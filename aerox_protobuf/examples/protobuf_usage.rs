//! Protobuf 编解码示例
//!
//! 演示如何使用 AeroX Protobuf 系统进行消息序列化和反序列化。

use aerox_protobuf::{decode_message, encode_message, wrap_message, unwrap_message, MessageRegistry};
use bytes::Bytes;

// 定义一些示例消息类型
#[derive(Clone, PartialEq, prost::Message)]
struct ChatMessage {
    #[prost(string, tag = "1")]
    sender_id: String,
    #[prost(string, tag = "2")]
    content: String,
    #[prost(uint64, tag = "3")]
    timestamp: u64,
}

#[derive(Clone, PartialEq, prost::Message)]
struct UserPosition {
    #[prost(string, tag = "1")]
    user_id: String,
    #[prost(float, tag = "2")]
    x: f32,
    #[prost(float, tag = "3")]
    y: f32,
    #[prost(float, tag = "4")]
    z: f32,
}

#[derive(Clone, PartialEq, prost::Message)]
struct HeartBeat {
    #[prost(uint64, tag = "1")]
    timestamp: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AeroX Protobuf 编解码示例 ===\n");

    // 示例 1: 消息注册
    println!("1. 消息注册:");
    let mut registry = MessageRegistry::new();
    registry.register(1001, "ChatMessage".to_string())?;
    registry.register(1002, "UserPosition".to_string())?;
    registry.register(1003, "HeartBeat".to_string())?;
    println!("   ✓ 注册了 {} 种消息类型", registry.count());
    println!();

    // 示例 2: 编码聊天消息
    println!("2. 编码聊天消息:");
    let chat_msg = ChatMessage {
        sender_id: "user_123".to_string(),
        content: "Hello, AeroX!".to_string(),
        timestamp: 1735297600,
    };
    let chat_encoded = encode_message(&chat_msg)?;
    println!("   原始消息: sender={}, content={}", chat_msg.sender_id, chat_msg.content);
    println!("   编码后大小: {} bytes", chat_encoded.len());
    println!();

    // 示例 3: 解码聊天消息
    println!("3. 解码聊天消息:");
    let chat_decoded: ChatMessage = decode_message(chat_encoded.clone())?;
    println!("   解码成功: sender={}, content={}", chat_decoded.sender_id, chat_decoded.content);
    assert_eq!(chat_msg, chat_decoded);
    println!("   ✓ 消息完整性验证通过");
    println!();

    // 示例 4: 编码位置消息
    println!("4. 编码用户位置消息:");
    let pos_msg = UserPosition {
        user_id: "player_456".to_string(),
        x: 100.5,
        y: 200.3,
        z: 50.0,
    };
    let pos_encoded = encode_message(&pos_msg)?;
    println!("   位置: ({}, {}, {})", pos_msg.x, pos_msg.y, pos_msg.z);
    println!("   编码后大小: {} bytes", pos_encoded.len());
    println!();

    // 示例 5: 消息包装（添加消息头）
    println!("5. 消息包装（添加消息头）:");
    let wrapped = wrap_message(1001, 12345, chat_encoded)?;
    println!("   消息 ID: 1001 (ChatMessage)");
    println!("   序列号: 12345");
    println!("   总大小: {} bytes (包含 16 字节头部)", wrapped.len());
    println!();

    // 示例 6: 消息解包
    println!("6. 消息解包:");
    let (msg_id, seq_id, payload) = unwrap_message(wrapped)?;
    println!("   消息 ID: {}", msg_id);
    println!("   序列号: {}", seq_id);
    println!("   负载大小: {} bytes", payload.len());

    // 验证消息类型
    if let Some(name) = registry.get_name(msg_id) {
        println!("   消息类型: {}", name);
    }
    println!();

    // 示例 7: 列出所有注册的消息
    println!("7. 已注册的消息类型:");
    for id in registry.list_ids() {
        if let Some(name) = registry.get_name(id) {
            println!("   - ID {}: {}", id, name);
        }
    }
    println!();

    // 示例 8: 性能测试
    println!("8. 性能测试:");
    let iterations = 10000;
    let test_msg = HeartBeat {
        timestamp: 1735297600,
    };

    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let encoded = encode_message(&test_msg)?;
        let _: HeartBeat = decode_message(encoded)?;
    }
    let elapsed = start.elapsed();
    println!("   编解码 {} 次耗时: {:?}", iterations, elapsed);
    println!(
        "   平均每次: {:.2} μs",
        elapsed.as_micros() as f64 / iterations as f64
    );
    println!();

    println!("✅ 所有示例运行完成！");

    Ok(())
}
