//! # AeroX ECS 游戏服务器示例
//!
//! ## 功能说明
//!
//! 这个示例展示了如何使用 AeroX 构建一个完整的游戏服务器，集成 ECS 系统。
//!
//! ## 架构图
//!
//! ```
//! 客户端 1 ──┐
//! 客户端 2 ──┼──> TCP 服务器 ──> 消息路由 ──> ECS 系统 ──> 广播给所有客户端
//! 客户端 3 ──┘
//! ```
//!
//! ## 运行方式
//!
//! ### 启动服务器:
//! ```bash
//! cargo run --example ecs_basics -- server
//! ```
//!
//! ### 启动客户端:
//! ```bash
//! cargo run --example ecs_basics -- client
//! ```
//!
//! ## 消息协议
//!
//! - `1001` - LoginRequest: 登录请求
//! - `2001` - MoveRequest: 移动请求
//! - `3001` - ChatMessage: 聊天消息
//! - `4001` - GetPlayersRequest: 获取玩家列表
//! - `5001` - Heartbeat: 心跳
//!
//! ## ECS 组件
//!
//! - `PlayerConnection`: 玩家连接信息
//! - `Position`: 3D 位置
//! - `PlayerName`: 玩家名称

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, Mutex};

use aerox_core::Result;
use aerox_ecs::{EcsWorld, PlayerConnection, Position, PlayerName};
use aerox_network::ConnectionId;
use prost::Message;

// ============================================================================
// Protobuf 消息定义
// ============================================================================

/// 登录请求
#[derive(Clone, prost::Message)]
pub struct LoginRequest {
    #[prost(string, tag = "1")]
    pub username: String,
}

/// 登录响应
#[derive(Clone, prost::Message)]
pub struct LoginResponse {
    #[prost(uint64, tag = "1")]
    pub player_id: u64,
    #[prost(string, tag = "2")]
    pub message: String,
}

/// 移动请求
#[derive(Clone, prost::Message)]
pub struct MoveRequest {
    #[prost(float, tag = "1")]
    pub x: f32,
    #[prost(float, tag = "2")]
    pub y: f32,
    #[prost(float, tag = "3")]
    pub z: f32,
}

/// 玩家移动广播
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

/// 聊天消息
#[derive(Clone, prost::Message)]
pub struct ChatMessage {
    #[prost(string, tag = "1")]
    pub content: String,
}

/// 聊天广播
#[derive(Clone, prost::Message)]
pub struct ChatBroadcast {
    #[prost(uint64, tag = "1")]
    pub player_id: u64,
    #[prost(string, tag = "2")]
    pub username: String,
    #[prost(string, tag = "3")]
    pub content: String,
}

/// 获取玩家列表请求
#[derive(Clone, prost::Message)]
pub struct GetPlayersRequest {}

/// 玩家信息
#[derive(Clone, prost::Message)]
pub struct PlayerInfo {
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

/// 玩家列表响应
#[derive(Clone, prost::Message)]
pub struct PlayerListResponse {
    #[prost(message, repeated, tag = "1")]
    pub players: Vec<PlayerInfo>,
}

/// 心跳
#[derive(Clone, prost::Message)]
pub struct Heartbeat {}

/// 心跳响应
#[derive(Clone, prost::Message)]
pub struct HeartbeatAck {}

// 消息 ID 常量
const MSG_ID_LOGIN_REQUEST: u16 = 1001;
const MSG_ID_LOGIN_RESPONSE: u16 = 1002;
const MSG_ID_MOVE_REQUEST: u16 = 2001;
const MSG_ID_PLAYER_MOVE: u16 = 2002;
const MSG_ID_CHAT_MESSAGE: u16 = 3001;
const MSG_ID_CHAT_BROADCAST: u16 = 3002;
const MSG_ID_GET_PLAYERS: u16 = 4001;
const MSG_ID_PLAYER_LIST: u16 = 4002;
const MSG_ID_HEARTBEAT: u16 = 5001;
const MSG_ID_HEARTBEAT_ACK: u16 = 5002;

// ============================================================================
// 服务器状态
// ============================================================================

/// 服务器状态
#[derive(Clone)]
pub struct ServerState {
    /// ECS 世界
    pub world: Arc<Mutex<EcsWorld>>,
    /// 连接映射 (connection_id -> player_id)
    pub connection_to_player: Arc<Mutex<HashMap<ConnectionId, u64>>>,
    /// 玩家映射 (player_id -> connection_id)
    pub player_to_connection: Arc<Mutex<HashMap<u64, ConnectionId>>>,
    /// 广播通道
    pub broadcast_tx: tokio::sync::broadcast::Sender<String>,
    /// 下一个玩家 ID
    pub next_player_id: Arc<Mutex<u64>>,
}

impl ServerState {
    /// 创建新的服务器状态
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            world: Arc::new(Mutex::new(EcsWorld::new())),
            connection_to_player: Arc::new(Mutex::new(HashMap::new())),
            player_to_connection: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            next_player_id: Arc::new(Mutex::new(1)),
        }
    }

    /// 分配新的玩家 ID
    pub async fn allocate_player_id(&self) -> u64 {
        let mut id = self.next_player_id.lock().await;
        let player_id = *id;
        *id += 1;
        player_id
    }
}

// ============================================================================
// 服务器实现
// ============================================================================

/// 运行 ECS 游戏服务器
pub async fn run_server() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX ECS 游戏服务器示例             ║");
    println!("╚════════════════════════════════════════╝\n");

    let bind_addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .map_err(|e| aerox_core::AeroXError::validation(format!("Invalid address: {}", e)))?;
    println!("🚀 启动服务器...");
    println!("   地址: {}\n", bind_addr);

    // 创建服务器状态
    let state = ServerState::new();

    // 初始化 ECS 世界
    {
        let mut world = state.world.lock().await;
        world.initialize().map_err(|e| {
            aerox_core::AeroXError::config(format!("Failed to initialize ECS world: {:?}", e))
        })?;
    }
    println!("✓ ECS 世界已初始化");

    let listener = TcpListener::bind(bind_addr).await?;
    println!("✓ 服务器启动成功，等待连接...\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("支持的消息类型:");
    println!("  [1001] LoginRequest   -> LoginResponse");
    println!("  [2001] MoveRequest     -> PlayerMoveBroadcast");
    println!("  [3001] ChatMessage     -> ChatBroadcast");
    println!("  [4001] GetPlayers      -> PlayerList");
    println!("  [5001] Heartbeat       -> HeartbeatAck");
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

/// 处理客户端连接
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
        // 读取 AeroX Frame 格式: 长度前缀(4) + 消息头(6) + 消息体
        // 先读取 4 字节长度前缀 + 6 字节头 = 10 字节
        match socket.read_exact(&mut buffer[..10]).await {
            Ok(_) => {}
            Err(e) => {
                println!("   ↳ 连接 #{} 已关闭 (接收 {} 条消息)", conn_id, messages_received);

                // 清理玩家实体
                cleanup_player(&state, connection_id).await;
                break;
            }
        }

        // 解析 AeroX Frame 头（使用小端序）
        // 前4字节是帧的长度（不包含长度字段本身）
        // 即: 2(msg_id) + 4(seq_id) + payload_len
        let frame_len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
        let msg_id = u16::from_le_bytes([buffer[4], buffer[5]]);
        let _seq_id = u32::from_le_bytes([buffer[6], buffer[7], buffer[8], buffer[9]]);

        // 计算消息体长度 = 帧长度 - msg_id(2) - seq_id(4)
        let payload_len = frame_len.saturating_sub(6);

        // 读取消息体
        if payload_len > 0 {
            if payload_len > buffer.len() {
                eprintln!("   ↳ 连接 #{} 消息体过大: {}", conn_id, payload_len);
                break;
            }
            socket.read_exact(&mut buffer[..payload_len]).await?;
            let payload = &buffer[..payload_len];

            messages_received += 1;

            // 根据消息ID处理不同类型的消息
            match msg_id {
                MSG_ID_LOGIN_REQUEST => {
                    if let Ok(req) = LoginRequest::decode(payload) {
                        handle_login(&mut socket, &state, connection_id, addr, req).await?;
                    }
                }
                MSG_ID_MOVE_REQUEST => {
                    if let Ok(req) = MoveRequest::decode(payload) {
                        handle_move(&mut socket, &state, connection_id, req).await?;
                    }
                }
                MSG_ID_CHAT_MESSAGE => {
                    if let Ok(req) = ChatMessage::decode(payload) {
                        handle_chat(&mut socket, &state, connection_id, req).await?;
                    }
                }
                MSG_ID_GET_PLAYERS => {
                    handle_get_players(&mut socket, &state).await?;
                }
                MSG_ID_HEARTBEAT => {
                    handle_heartbeat(&mut socket).await?;
                }
                _ => {
                    println!("   ↳ 连接 #{} 收到未知消息类型: {}", conn_id, msg_id);
                }
            }
        }
    }

    Ok(())
}

/// 处理登录请求
async fn handle_login(
    socket: &mut TcpStream,
    state: &ServerState,
    connection_id: ConnectionId,
    addr: SocketAddr,
    req: LoginRequest,
) -> Result<()> {
    println!("   ↳ [LOGIN] 用户登录: {}", req.username);

    // 分配玩家 ID
    let player_id = state.allocate_player_id().await;

    // 创建 ECS 实体
    {
        let mut world = state.world.lock().await;

        // 创建玩家实体
        let _entity = world.spawn_bundle((
            PlayerConnection::new(connection_id, addr),
            Position::origin(),
            PlayerName::new(req.username.clone()),
        ));

        println!("   ↳ [ECS] 创建实体 (player_id={})", player_id);
    }

    // 更新映射
    {
        let mut conn_to_player = state.connection_to_player.lock().await;
        let mut player_to_conn = state.player_to_connection.lock().await;

        conn_to_player.insert(connection_id, player_id);
        player_to_conn.insert(player_id, connection_id);
    }

    // 发送响应
    let response = LoginResponse {
        player_id,
        message: format!("欢迎, {}!", req.username),
    };
    send_message(socket, MSG_ID_LOGIN_RESPONSE, &response).await?;

    println!("   ↳ [LOGIN] 玩家 {} (ID: {}) 登录成功", req.username, player_id);

    Ok(())
}

/// 处理移动请求
async fn handle_move(
    socket: &mut TcpStream,
    state: &ServerState,
    connection_id: ConnectionId,
    req: MoveRequest,
) -> Result<()> {
    // 获取玩家 ID
    let player_id = {
        let conn_to_player = state.connection_to_player.lock().await;
        conn_to_player.get(&connection_id).copied()
    };

    if let Some(pid) = player_id {
        // 更新位置 - 简化版本，暂时跳过实际的 ECS 更新
        println!("   ↳ [MOVE] 玩家 ID {} 移动到 ({}, {}, {})", pid, req.x, req.y, req.z);

        // TODO: 更新 ECS 中的 Position 组件
        // 需要通过 player_id 找到对应的实体并更新位置

        // TODO: 广播给所有客户端
    } else {
        println!("   ↳ [MOVE] 未知的连接 ID: {:?}", connection_id);
    }

    Ok(())
}

/// 处理聊天消息
async fn handle_chat(
    socket: &mut TcpStream,
    state: &ServerState,
    connection_id: ConnectionId,
    req: ChatMessage,
) -> Result<()> {
    // 获取玩家 ID
    let player_id = {
        let conn_to_player = state.connection_to_player.lock().await;
        conn_to_player.get(&connection_id).copied()
    };

    if let Some(pid) = player_id {
        // 简化：暂时只打印日志
        println!("   ↳ [CHAT] 玩家 ID {}: {}", pid, req.content);

        // TODO: 从 ECS 获取用户名
        // TODO: 广播给所有客户端
    }

    Ok(())
}

/// 处理获取玩家列表
async fn handle_get_players(socket: &mut TcpStream, state: &ServerState) -> Result<()> {
    // 先获取映射关系
    let conn_to_player_map = {
        let conn_to_player = state.connection_to_player.lock().await;
        conn_to_player.clone()
    };

    let mut world = state.world.lock().await;

    // 简化版本：直接查询所有组件
    let world_mut = world.world_mut();
    let mut query = world_mut.query::<(&PlayerConnection, &Position, &PlayerName)>();

    let mut players = Vec::new();
    for (conn, pos, name) in query.iter(world_mut) {
        // 通过 connection_id 映射到 player_id
        let player_id = conn_to_player_map.get(&conn.connection_id).copied().unwrap_or(0);

        players.push(PlayerInfo {
            player_id,
            username: name.name.clone(),
            x: pos.x,
            y: pos.y,
            z: pos.z,
        });
    }

    let response = PlayerListResponse { players };
    send_message(socket, MSG_ID_PLAYER_LIST, &response).await?;

    Ok(())
}

/// 处理心跳
async fn handle_heartbeat(socket: &mut TcpStream) -> Result<()> {
    let ack = HeartbeatAck {};
    send_message(socket, MSG_ID_HEARTBEAT_ACK, &ack).await?;
    Ok(())
}

/// 清理断开的玩家
async fn cleanup_player(state: &ServerState, connection_id: ConnectionId) {
    // 从映射中移除
    let player_id = {
        let mut conn_to_player = state.connection_to_player.lock().await;
        let mut player_to_conn = state.player_to_connection.lock().await;

        let player_id = conn_to_player.remove(&connection_id);
        if let Some(pid) = player_id {
            player_to_conn.remove(&pid);
        }
        player_id
    };

    if let Some(pid) = player_id {
        println!("   ↳ [CLEANUP] 清理玩家 ID: {}", pid);

        // TODO: 从 ECS 世界中移除实体
        // 当前 ECS API 可能不支持按 connection_id 查询实体
    }
}

/// 发送消息到客户端（使用 AeroX Frame 格式）
async fn send_message<M: prost::Message>(
    socket: &mut TcpStream,
    msg_id: u16,
    message: &M,
) -> Result<()> {
    let mut buf = Vec::new();

    // 编码消息体
    message.encode(&mut buf).map_err(|e| {
        aerox_core::AeroXError::protocol(format!("Failed to encode message: {:?}", e))
    })?;

    let payload_len = buf.len();
    // AeroX Frame: 长度字段表示的是 "除了长度字段本身" 的数据长度
    // 即: 2字节(msg_id) + 4字节(seq_id) + payload_len
    let frame_len = 6 + payload_len;

    // 写入 AeroX Frame 头（使用小端序）
    // 4字节：帧长度（不包含长度字段本身）
    socket.write_all(&(frame_len as u32).to_le_bytes()).await?;
    // 2字节：消息ID
    socket.write_all(&msg_id.to_le_bytes()).await?;
    // 4字节：序列ID
    socket.write_all(&0u32.to_le_bytes()).await?;

    // 写入消息体
    socket.write_all(&buf).await?;

    Ok(())
}

// ============================================================================
// 客户端实现
// ============================================================================

/// 客户端状态
pub struct ClientState {
    pub logged_in: bool,
    pub player_id: Option<u64>,
    pub username: Option<String>,
    pub position: (f32, f32, f32),
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            logged_in: false,
            player_id: None,
            username: None,
            position: (0.0, 0.0, 0.0),
        }
    }
}

/// 运行客户端
pub async fn run_client() -> aerox_client::Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX 游戏客户端                     ║");
    println!("╚════════════════════════════════════════╝\n");

    use aerox_client::StreamClient;

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("🔗 连接到服务器: {}...\n", addr);

    let mut client = StreamClient::connect(addr).await?;
    println!("✓ 连接成功!\n");

    // 简化：自动执行一些操作
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("自动演示模式:\n");

    // 登录
    let username = format!("Player{}", 1);
    println!("1. 登录为: {}", username);

    let login_req = LoginRequest { username: username.clone() };
    client.send_message(MSG_ID_LOGIN_REQUEST, &login_req).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 移动
    println!("2. 移动到 (10.0, 20.0, 30.0)");
    let move_req = MoveRequest { x: 10.0, y: 20.0, z: 30.0 };
    client.send_message(MSG_ID_MOVE_REQUEST, &move_req).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 聊天
    println!("3. 发送聊天消息");
    let chat_msg = ChatMessage {
        content: "Hello from AeroX client!".to_string(),
    };
    client.send_message(MSG_ID_CHAT_MESSAGE, &chat_msg).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 获取玩家列表
    println!("4. 获取玩家列表");
    client.send_message(MSG_ID_GET_PLAYERS, &GetPlayersRequest {}).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 心跳
    println!("5. 发送心跳");
    client.send_message(MSG_ID_HEARTBEAT, &Heartbeat {}).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 演示完成");

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
            // 将客户端的 Result 转换为 core Result
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
