//! 简单聊天室示例
//!
//! 演示如何使用 AeroX 创建一个多客户端聊天室。
//! 支持用户名设置、广播消息、用户列表等功能。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, Mutex};
use aerox_core::Result;

/// 聊天室服务器
#[derive(Clone)]
struct ChatServer {
    /// 广播通道
    tx: broadcast<String>,
    /// 在线用户
    users: Arc<Mutex<HashMap<SocketAddr, String>>>,
}

impl ChatServer {
    /// 创建新的聊天室服务器
    fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加用户
    async fn add_user(&self, addr: SocketAddr, username: String) {
        let mut users = self.users.lock().await;
        users.insert(addr, username.clone());

        // 广播用户加入消息
        let msg = format!("*** {} 加入了聊天室", username);
        let _ = self.tx.send(msg);
    }

    /// 移除用户
    async fn remove_user(&self, addr: SocketAddr) {
        let mut users = self.users.lock().await;
        if let Some(username) = users.remove(&addr) {
            // 广播用户离开消息
            let msg = format!("*** {} 离开了聊天室", username);
            let _ = self.tx.send(msg);
        }
    }

    /// 广播消息
    async fn broadcast(&self, username: &str, message: &str) {
        let msg = format!("{}: {}", username, message);
        let _ = self.tx.send(msg);
    }

    /// 获取用户列表
    async fn list_users(&self) -> Vec<String> {
        let users = self.users.lock().await;
        users.values().cloned().collect()
    }

    /// 获取在线用户数
    async fn user_count(&self) -> usize {
        let users = self.users.lock().await;
        users.len()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== AeroX 简单聊天室示例 ===\n");

    // 配置服务器地址
    let bind_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    println!("🚀 启动聊天室服务器...");
    println!("   地址: {}\n", bind_addr);

    // 创建聊天室服务器
    let server = ChatServer::new();

    // 创建 TCP 监听器
    let listener = TcpListener::bind(bind_addr).await?;
    println!("✓ 服务器启动成功");
    println!("✓ 等待用户连接...\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("提示: 使用 telnet 或 nc 连接");
    println!("  telnet 127.0.0.1 8080");
    println!("  nc 127.0.0.1 8080");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 接受连接
    let mut conn_id = 0;
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                conn_id += 1;
                let server_clone = server.clone();

                println!("📥 新连接 #{} 来自: {}", conn_id, addr);

                // 为每个连接 spawn 一个任务
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, addr, server_clone, conn_id).await {
                        eprintln!("❌ 连接 #{} 错误: {}", conn_id, e);
                    }
                });

                // 显示在线用户数
                let count = server.user_count().await;
                if count > 0 {
                    println!("   (在线用户: {})\n", count);
                }
            }
            Err(e) => {
                eprintln!("❌ 接受连接失败: {}", e);
            }
        }
    }
}

/// 处理单个客户端连接
async fn handle_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    server: ChatServer,
    conn_id: usize,
) -> Result<()> {
    println!("   ↳ 连接 #{} 已建立", conn_id);

    // 订阅广播频道
    let mut rx = server.tx.subscribe();

    // 发送欢迎消息
    let welcome = "╔══════════════════════════════════════╗\n\
                    ║   欢迎来到 AeroX 聊天室! 🎉        ║\n\
                    ╠══════════════════════════════════════╣\n\
                    ║ 命令:                               ║\n\
                    ║   /name <用户名>  - 设置用户名      ║\n\
                    ║   /list           - 查看在线用户    ║\n\
                    ║   /quit           - 退出聊天室      ║\n\
                    ╚══════════════════════════════════════╝\n\
                    请输入你的用户名: ";

    socket.write_all(welcome.as_bytes()).await?;

    // 读取用户名
    let mut buffer = [0u8; 1024];
    let username = match socket.read(&mut buffer).await {
        Ok(0) => return Ok(()), // 连接关闭
        Ok(n) => {
            let input = String::from_utf8_lossy(&buffer[..n]);
            let name = input.trim().to_string();
            if name.is_empty() {
                format!("User_{}", conn_id)
            } else if name.starts_with('/') {
                // 用户输入了命令而不是名字
                format!("User_{}", conn_id)
            } else {
                name
            }
        }
        Err(e) => {
            eprintln!("   ↳ 读取错误 #{}: {}", conn_id, e);
            return Err(e.into());
        }
    };

    // 添加用户
    server.add_user(addr, username.clone()).await;

    // 发送加入确认和当前在线用户
    let users = server.list_users().await;
    let msg = format!(
        "\n✓ 你已加入聊天室，用户名: {}\n\
         当前在线用户 ({}): {}\n\n",
        username,
        users.len(),
        users.join(", ")
    );
    socket.write_all(msg.as_bytes()).await?;

    // 克隆 socket 用于发送广播
    let mut socket_clone = socket.try_clone()?;

    // 启动任务接收广播消息
    let addr_clone = addr;
    let username_clone = username.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // 不发送给自己
            if let Err(e) = socket_clone.write_all(format!("{}\n", msg).as_bytes()).await {
                eprintln!("   ↳ 发送广播失败: {}", e);
                break;
            }
        }
    });

    // 主循环：处理用户输入
    loop {
        buffer.fill(0);
        match socket.read(&mut buffer).await {
            Ok(0) => {
                // 连接关闭
                break;
            }
            Ok(n) => {
                let input = String::from_utf8_lossy(&buffer[..n]);
                let cmd = input.trim();

                // 处理命令
                if cmd.starts_with('/') {
                    match cmd {
                        "/quit" => {
                            socket.write_all(b"*** 再见!\n").await?;
                            break;
                        }
                        "/list" => {
                            let users = server.list_users().await;
                            let msg = format!(
                                "\n*** 在线用户 ({}): {}\n",
                                users.len(),
                                users.join(", ")
                            );
                            socket.write_all(msg.as_bytes()).await?;
                        }
                        cmd if cmd.starts_with("/name ") => {
                            let new_name = cmd[6..].trim();
                            if !new_name.is_empty() {
                                // 移除旧用户
                                server.remove_user(addr_clone).await;
                                // 添加新用户
                                server.add_user(addr_clone, new_name.to_string()).await;
                                socket.write_all(
                                    format!("\n*** 用户名已更改为: {}\n", new_name).as_bytes()
                                ).await?;
                            }
                        }
                        _ => {
                            socket.write_all(b"\n*** 未知命令\n").await?;
                        }
                    }
                } else if !cmd.is_empty() {
                    // 广播消息
                    server.broadcast(&username_clone, cmd).await;
                }
            }
            Err(e) => {
                eprintln!("   ↳ 读取错误 #{}: {}", conn_id, e);
                break;
            }
        }
    }

    // 移除用户
    server.remove_user(addr).await;
    println!("   ↳ 连接 #{} ({}) 已关闭", conn_id, username);

    Ok(())
}
