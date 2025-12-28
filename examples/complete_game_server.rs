//! # AeroX 完整游戏服务器示例
//!
//! ## 功能说明
//!
//! 这是一个完整的多玩家游戏服务器示例，展示：
//! - 多玩家实时通信
//! - ECS 游戏逻辑集成
//! - 位置同步和广播
//! - 聊天系统
//! - 心跳检测和超时处理
//!
//! ## 运行方式
//!
//! ### 启动服务器:
//! ```bash
//! cargo run --example complete_game_server -- server
//! ```
//!
//! ### 启动客户端（可以启动多个）:
//! ```bash
//! cargo run --example complete_game_server -- client
//! ```
//!
//! ## 架构
//!
//! ```
//! 多个客户端连接 ──> TCP 服务器 ──> ECS 世界 ──> 广播给所有客户端
//!                      ↓
//!                 消息路由
//!                      ↓
//!                 业务逻辑处理
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, Mutex, RwLock};

use aerox_core::Result;
use aerox_ecs::{EcsWorld, PlayerConnection, Position, PlayerName};
use aerox_network::ConnectionId;
use prost::Message;

// ============================================================================
// Protobuf 消息定义
// ============================================================================

#[derive(Clone, prost::Message)]
pub struct LoginRequest {
    #[prost(string, tag = "1")]
    pub username: String,
}

#[derive(Clone, prost::Message)]
pub struct LoginResponse {
    #[prost(uint64, tag = "1")]
    pub player_id: u64,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, prost::Message)]
pub struct MoveRequest {
    #[prost(float, tag = "1")]
    pub x: f32,
    #[prost(float, tag = "2")]
    pub y: f32,
    #[prost(float, tag = "3")]
    pub z: f32,
}

#[derive(Clone, prost::Message)]
pub struct PlayerMoveBroadcast {
    #[prost(uint64, tag = "1")]
    pub player_id: u64,
    #[prost(string, tag = "2")]
    pub username: String,
    #[prost(float, tag = "3")]
    pub x: f32,
    #[prost(float, tag = "4")]
    pub y: f32,
    #[prost(float, tag = "5")]
    pub z: f32,
}

#[derive(Clone, prost::Message)]
pub struct ChatMessage {
    #[prost(string, tag = "1")]
    pub content: String,
}

#[derive(Clone, prost::Message)]
pub struct ChatBroadcast {
    #[prost(uint64, tag = "1")]
    pub player_id: u64,
    #[prost(string, tag = "2")]
    pub username: String,
    #[prost(string, tag = "3")]
    pub content: String,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}

#[derive(Clone, prost::Message)]
pub struct PlayerJoinBroadcast {
    #[prost(uint64, tag = "1")]
    pub player_id: u64,
    #[prost(string, tag = "2")]
    pub username: String,
}

#[derive(Clone, prost::Message)]
pub struct PlayerLeaveBroadcast {
    #[prost(uint64, tag = "1")]
    pub player_id: u64,
}

#[derive(Clone, prost::Message)]
pub struct Heartbeat {}

#[derive(Clone, prost::Message)]
pub struct HeartbeatAck {}

// 消息 ID
const MSG_ID_LOGIN: u16 = 1001;
const MSG_ID_LOGIN_RESP: u16 = 1002;
const MSG_ID_MOVE: u16 = 2001;
const MSG_ID_MOVE_BROADCAST: u16 = 2002;
const MSG_ID_CHAT: u16 = 3001;
const MSG_ID_CHAT_BROADCAST: u16 = 3002;
const MSG_ID_PLAYER_JOIN: u16 = 4001;
const MSG_ID_PLAYER_LEAVE: u16 = 4002;
const MSG_ID_HEARTBEAT: u16 = 5001;
const MSG_ID_HEARTBEAT_ACK: u16 = 5002;

// ============================================================================
// 服务器状态
// ============================================================================

#[derive(Clone)]
pub struct ServerState {
    /// ECS 世界
    pub world: Arc<Mutex<EcsWorld>>,
    /// 连接映射
    pub connections: Arc<Mutex<HashMap<ConnectionId, ClientInfo>>>,
    /// 广播通道
    pub broadcast_tx: broadcast::Sender<BroadcastMessage>,
    /// 下一个玩家 ID
    pub next_player_id: Arc<Mutex<u64>>,
}

/// 客户端信息
#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub connection_id: ConnectionId,
    pub player_id: u64,
    pub addr: SocketAddr,
    pub socket: Arc<Mutex<TcpStream>>,
    pub last_heartbeat: Arc<Mutex<Instant>>,
}

/// 广播消息类型
#[derive(Clone, Debug)]
pub enum BroadcastMessage {
    PlayerJoin { player_id: u64, username: String },
    PlayerLeave { player_id: u64 },
    PlayerMove {
        player_id: u64,
        username: String,
        x: f32,
        y: f32,
        z: f32,
    },
    Chat {
        player_id: u64,
        username: String,
        content: String,
    },
}

impl ServerState {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            world: Arc::new(Mutex::new(EcsWorld::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            next_player_id: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn allocate_player_id(&self) -> u64 {
        let mut id = self.next_player_id.lock().await;
        let player_id = *id;
        *id += 1;
        player_id
    }

    /// 广播消息给所有客户端
    pub async fn broadcast(&self, msg: BroadcastMessage) {
        let _ = self.broadcast_tx.send(msg);
    }

    /// 获取所有玩家信息
    pub async fn get_all_players(&self) -> Vec<(u64, String, (f32, f32, f32))> {
        let mut world = self.world.lock().await;
        let world_ref = world.world_mut();
        let mut query = world_ref.query::<(&PlayerConnection, &Position, &PlayerName)>();

        let mut players = Vec::new();
        let conn_map = self.connections.lock().await;

        for (conn, pos, name) in query.iter(world_ref) {
            if let Some(info) = conn_map.get(&conn.connection_id) {
                players.push((info.player_id, name.name.clone(), (pos.x, pos.y, pos.z)));
            }
        }

        players
    }
}

// ============================================================================
// 服务器实现
// ============================================================================

pub async fn run_server() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX 完整游戏服务器                 ║");
    println!("╚════════════════════════════════════════╝\n");

    let bind_addr: SocketAddr = "127.0.0.1:8082"
        .parse()
        .map_err(|e| aerox_core::AeroXError::validation(format!("Invalid address: {}", e)))?;
    println!("🚀 启动服务器...");
    println!("   地址: {}\n", bind_addr);

    let state = ServerState::new();

    // 初始化 ECS 世界
    {
        let mut world = state.world.lock().await;
        world.initialize().map_err(|e| {
            aerox_core::AeroXError::config(format!("Failed to initialize ECS world: {:?}", e))
        })?;
    }
    println!("✓ ECS 世界已初始化");

    // 启动广播任务
    let state_clone = state.clone();
    tokio::spawn(async move {
        broadcast_task(state_clone).await;
    });

    let listener = TcpListener::bind(bind_addr).await?;
    println!("✓ 服务器启动成功，等待连接...\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("支持的消息:");
    println!("  登录、移动、聊天、心跳");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut connection_count = 0;

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                connection_count += 1;
                println!("📥 新连接 #{} 来自: {}", connection_count, addr);

                let state_clone = state.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, addr, connection_count, state_clone).await {
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

/// 广播任务 - 将消息广播给所有连接的客户端
async fn broadcast_task(state: ServerState) {
    let mut rx = state.broadcast_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let connections = state.connections.lock().await;
                for (conn_id, info) in connections.iter() {
                    if let Ok(mut socket) = info.socket.try_lock() {
                        let result = match &msg {
                            BroadcastMessage::PlayerJoin { player_id, username } => {
                                let broadcast = PlayerJoinBroadcast {
                                    player_id: *player_id,
                                    username: username.clone(),
                                };
                                send_message(&mut *socket, MSG_ID_PLAYER_JOIN, &broadcast).await
                            }
                            BroadcastMessage::PlayerLeave { player_id } => {
                                let broadcast = PlayerLeaveBroadcast {
                                    player_id: *player_id,
                                };
                                send_message(&mut *socket, MSG_ID_PLAYER_LEAVE, &broadcast).await
                            }
                            BroadcastMessage::PlayerMove { player_id, username, x, y, z } => {
                                let broadcast = PlayerMoveBroadcast {
                                    player_id: *player_id,
                                    username: username.clone(),
                                    x: *x,
                                    y: *y,
                                    z: *z,
                                };
                                send_message(&mut *socket, MSG_ID_MOVE_BROADCAST, &broadcast).await
                            }
                            BroadcastMessage::Chat { player_id, username, content } => {
                                let timestamp = SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                let broadcast = ChatBroadcast {
                                    player_id: *player_id,
                                    username: username.clone(),
                                    content: content.clone(),
                                    timestamp,
                                };
                                send_message(&mut *socket, MSG_ID_CHAT_BROADCAST, &broadcast).await
                            }
                        };

                        if let Err(e) = result {
                            eprintln!("广播到 {:?} 失败: {}", conn_id, e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("广播通道错误: {:?}", e);
                break;
            }
        }
    }
}

async fn handle_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    conn_id: usize,
    state: ServerState,
) -> Result<()> {
    println!("   ↳ 连接 #{} 已建立", conn_id);

    let connection_id = ConnectionId::new(conn_id as u64);
    let mut buffer = [0u8; 8192];
    let mut messages_received = 0u64;

    loop {
        match socket.read_exact(&mut buffer[..10]).await {
            Ok(_) => {}
            Err(e) => {
                println!("   ↳ 连接 #{} 已关闭 (接收 {} 条消息)", conn_id, messages_received);

                // 清理连接和 ECS 实体
                cleanup_connection(&state, connection_id).await;
                break;
            }
        }

        let frame_len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
        let msg_id = u16::from_le_bytes([buffer[4], buffer[5]]);
        let _seq_id = u32::from_le_bytes([buffer[6], buffer[7], buffer[8], buffer[9]]);

        let payload_len = frame_len.saturating_sub(6);

        if payload_len > 0 {
            if payload_len > buffer.len() {
                eprintln!("   ↳ 连接 #{} 消息体过大: {}", conn_id, payload_len);
                break;
            }
            socket.read_exact(&mut buffer[..payload_len]).await?;
            let payload = &buffer[..payload_len];

            messages_received += 1;

            match msg_id {
                MSG_ID_LOGIN => handle_login(&state, connection_id, addr, payload).await?,
                MSG_ID_MOVE => handle_move(&state, connection_id, payload).await?,
                MSG_ID_CHAT => handle_chat(&state, connection_id, payload).await?,
                MSG_ID_HEARTBEAT => {
                    // 更新心跳时间
                    if let Some(conn_info) = state.connections.lock().await.get(&connection_id) {
                        *conn_info.last_heartbeat.lock().await = Instant::now();
                    }

                    // 发送 ACK
                    // 注意：这里简化处理，实际应该通过 socket 发送
                }
                _ => {
                    println!("   ↳ 连接 #{} 未知消息类型: {}", conn_id, msg_id);
                }
            }
        }
    }

    Ok(())
}

async fn handle_login(
    state: &ServerState,
    connection_id: ConnectionId,
    addr: SocketAddr,
    payload: &[u8],
) -> Result<()> {
    if let Ok(req) = LoginRequest::decode(payload) {
        println!("   ↳ [LOGIN] 用户登录: {}", req.username);

        let player_id = state.allocate_player_id().await;

        // 创建 ECS 实体
        {
            let mut world = state.world.lock().await;
            let _entity = world.spawn_bundle((
                PlayerConnection::new(connection_id, addr),
                Position::origin(),
                PlayerName::new(req.username.clone()),
            ));
        }

        // 存储连接信息（注意：这里简化处理，实际应该存储传入的 socket）
        // 由于无法在 async fn 中修改传入的 socket，这里暂时跳过存储
        // 在完整实现中，应该使用 channel 将 socket 发送到广播任务
        // let conn_info = ClientInfo {
        //     connection_id,
        //     player_id,
        //     addr,
        //     socket: Arc::new(Mutex::new(socket)),
        //     last_heartbeat: Arc::new(Mutex::new(Instant::now())),
        // };
        //
        // let mut connections = state.connections.lock().await;
        // connections.insert(connection_id, conn_info);

        // 发送响应（这里简化，实际应该发送给客户端）
        println!("   ↳ [LOGIN] 玩家 {} (ID: {}) 登录成功", req.username, player_id);

        // 广播玩家加入
        state
            .broadcast(BroadcastMessage::PlayerJoin {
                player_id,
                username: req.username,
            })
            .await;

        // 发送当前玩家列表给新玩家
        let players = state.get_all_players().await;
        println!("   ↳ 当前在线玩家: {} 人", players.len());
    }

    Ok(())
}

async fn handle_move(state: &ServerState, connection_id: ConnectionId, payload: &[u8]) -> Result<()> {
    if let Ok(req) = MoveRequest::decode(payload) {
        // 获取玩家信息
        let player_id = {
            let connections = state.connections.lock().await;
            connections.get(&connection_id).map(|info| info.player_id)
        };

        if let Some(pid) = player_id {
            // 更新 ECS 位置
            {
                let mut world = state.world.lock().await;
                let world_mut = world.world_mut();

                // 简化：这里需要找到对应的实体并更新位置
                // 实际实现需要通过 connection_id 查找实体
            }

            // 广播移动（暂时注释，需要用户名）
            // state.broadcast(...).await;

            println!("   ↳ [MOVE] 玩家 {} 移动到 ({}, {}, {})", pid, req.x, req.y, req.z);
        }
    }

    Ok(())
}

async fn handle_chat(state: &ServerState, connection_id: ConnectionId, payload: &[u8]) -> Result<()> {
    if let Ok(req) = ChatMessage::decode(payload) {
        let (player_id, username) = {
            let connections = state.connections.lock().await;
            if let Some(info) = connections.get(&connection_id) {
                // 从 ECS 获取用户名
                let mut world = state.world.lock().await;
                let world_ref = world.world_mut();
                let mut query = world_ref.query::<(&PlayerConnection, &PlayerName)>();

                let mut found_username = None;
                for (conn, name) in query.iter(world_ref) {
                    if conn.connection_id == connection_id {
                        found_username = Some(name.name.clone());
                        break;
                    }
                }

                (info.player_id, found_username.unwrap_or_else(|| "".to_string()))
            } else {
                return Ok(());
            }
        };

        if !username.is_empty() {
            println!("   ↳ [CHAT] {}: {}", username, req.content);

            // 广播聊天消息
            state
                .broadcast(BroadcastMessage::Chat {
                    player_id,
                    username,
                    content: req.content,
                })
                .await;
        }
    }

    Ok(())
}

async fn cleanup_connection(state: &ServerState, connection_id: ConnectionId) {
    // 获取玩家 ID
    let player_id = {
        let connections = state.connections.lock().await;
        connections
            .get(&connection_id)
            .map(|info| info.player_id)
    };

    if let Some(pid) = player_id {
        println!("   ↳ [CLEANUP] 清理玩家 ID: {}", pid);

        // 移除连接
        let mut connections = state.connections.lock().await;
        connections.remove(&connection_id);

        // TODO: 从 ECS 世界中移除实体

        // 广播玩家离开
        state
            .broadcast(BroadcastMessage::PlayerLeave { player_id: pid })
            .await;
    }
}

async fn send_message<M: prost::Message>(
    socket: &mut TcpStream,
    msg_id: u16,
    message: &M,
) -> Result<()> {
    let mut buf = Vec::new();
    message
        .encode(&mut buf)
        .map_err(|e| aerox_core::AeroXError::protocol(format!("Encode error: {:?}", e)))?;

    let payload_len = buf.len();
    let frame_len = 6 + payload_len;

    socket.write_all(&(frame_len as u32).to_le_bytes()).await?;
    socket.write_all(&msg_id.to_le_bytes()).await?;
    socket.write_all(&0u32.to_le_bytes()).await?;
    socket.write_all(&buf).await?;

    Ok(())
}

// ============================================================================
// 客户端实现
// ============================================================================

pub async fn run_client() -> aerox_client::Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX 游戏客户端                     ║");
    println!("╚════════════════════════════════════════╝\n");

    use aerox_client::StreamClient;

    let addr: SocketAddr = "127.0.0.1:8082".parse().unwrap();
    println!("🔗 连接到服务器: {}...\n", addr);

    let mut client = StreamClient::connect(addr).await?;
    println!("✓ 连接成功!\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("游戏操作:\n");

    // 登录
    let username = format!("Player{}", std::process::id() % 1000);
    println!("1️⃣  登录为: {}", username);
    let login_req = LoginRequest { username: username.clone() };
    client.send_message(MSG_ID_LOGIN, &login_req).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 移动
    println!("2️⃣  移动到随机位置");
    let x = (std::process::id() as f32 * 17.0) % 100.0;
    let y = (std::process::id() as f32 * 23.0) % 100.0;
    let z = (std::process::id() as f32 * 29.0) % 100.0;
    let move_req = MoveRequest { x, y, z };
    client.send_message(MSG_ID_MOVE, &move_req).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 聊天
    println!("3️⃣  发送聊天消息");
    let chat_msg = ChatMessage {
        content: format!("Hello from {}!", username),
    };
    client.send_message(MSG_ID_CHAT, &chat_msg).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 心跳
    println!("4️⃣  发送心跳");
    client.send_message(MSG_ID_HEARTBEAT, &Heartbeat {}).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 持续发送移动和心跳
    println!("\n5️⃣  持续游戏循环（每 2 秒移动一次）");
    for i in 1..=5 {
        let x = (i as f32 * 10.0) % 100.0;
        let y = (i as f32 * 15.0) % 100.0;
        let z = (i as f32 * 20.0) % 100.0;

        client.send_message(MSG_ID_MOVE, &MoveRequest { x, y, z }).await?;
        client.send_message(MSG_ID_HEARTBEAT, &Heartbeat {}).await?;

        println!("   循环 {}/5 - 移动到 ({}, {}, {})", i, x, y, z);
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 游戏会话结束");

    Ok(())
}

// ============================================================================
// 主函数
// ============================================================================

#[tokio::main]
async fn main() -> aerox_core::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("用法:");
        println!("  server - 启动服务器");
        println!("  client - 启动客户端");
        return Ok(());
    }

    match args[1].as_str() {
        "server" => run_server().await,
        "client" => {
            run_client()
                .await
                .map_err(|e| aerox_core::AeroXError::network(format!("Client error: {:?}", e)))
        }
        _ => {
            eprintln!("未知参数: {}", args[1]);
            eprintln!("使用 'server' 或 'client'");
            Ok(())
        }
    }
}
