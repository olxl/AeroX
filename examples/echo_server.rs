//! Echo Server 示例
//!
//! 演示如何使用 AeroX 网络层创建一个简单的回显服务器。
//! 服务器会将收到的所有消息原样发送回客户端。

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use aerox_core::Result;
use aerox_network::TcpReactor;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== AeroX Echo Server 示例 ===\n");

    // 配置服务器地址
    let bind_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    println!("🚀 启动 Echo Server...");
    println!("   地址: {}\n", bind_addr);

    // 创建 TCP 监听器
    let listener = TcpListener::bind(bind_addr).await?;
    println!("✓ 服务器启动成功，等待连接...\n");

    let mut connection_count = 0;

    // 接受并处理连接
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                connection_count += 1;
                println!("📥 新连接 #{} 来自: {}", connection_count, addr);

                // 为每个连接 spawn 一个任务
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, addr, connection_count).await {
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

/// 处理单个连接
///
/// 读取客户端发送的数据并原样返回
async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    addr: SocketAddr,
    conn_id: usize,
) -> Result<()> {
    println!("   ↳ 连接 #{} 已建立", conn_id);

    let mut buffer = [0u8; 1024];
    let mut bytes_received = 0u64;
    let mut messages_received = 0u64;

    // 持续读取数据
    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                // 连接关闭
                println!("   ↳ 连接 #{} 已关闭 (接收 {} 字节, {} 条消息)",
                    conn_id, bytes_received, messages_received);
                break;
            }
            Ok(n) => {
                bytes_received += n as u64;
                messages_received += 1;

                // 打印接收到的数据
                let data = &buffer[..n];
                if let Ok(text) = std::str::from_utf8(data) {
                    println!("   ↳ 接收 #{}: {}", conn_id, text.trim());
                } else {
                    println!("   ↳ 接收 #{}: {} 字节 (二进制数据)",
                        conn_id, n);
                }

                // 回显数据
                match socket.write_all(data).await {
                    Ok(_) => {
                        println!("   ↳ 发送 #{}: {} 字节", conn_id, n);
                    }
                    Err(e) => {
                        eprintln!("   ↳ 发送失败 #{}: {}", conn_id, e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("   ↳ 读取错误 #{}: {}", conn_id, e);
                break;
            }
        }
    }

    Ok(())
}

/// 使用 AeroX Reactor 的 Echo Server 版本
///
/// 这个版本展示了如何使用 AeroX 的 TcpReactor
pub async fn run_with_reactor() -> Result<()> {
    println!("=== AeroX Echo Server (使用 Reactor) ===\n");

    // 创建 Reactor 配置
    let config = aerox_config::ServerConfig::default();

    // 创建 Reactor
    let mut reactor = TcpReactor::new(config).await?;

    println!("🚀 启动 Echo Server (使用 Reactor)...");
    println!("   地址: {}\n", reactor.bind_addr());

    // 启动服务器
    let handle = reactor.start()?;

    println!("✓ 服务器启动成功，按 Ctrl+C 停止\n");

    // 等待 shutdown 信号
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\n⏸️  收到停止信号，正在关闭服务器...");
        }
        result = handle => {
            if let Err(e) = result {
                eprintln!("❌ 服务器错误: {}", e);
            }
        }
    }

    // 优雅关闭
    reactor.shutdown().await?;
    println!("✓ 服务器已关闭");

    Ok(())
}
