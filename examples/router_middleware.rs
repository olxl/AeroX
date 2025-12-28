//! # AeroX 路由和中间件示例
//!
//! ## 功能说明
//!
//! 这个示例展示了 AeroX 路由和中间件系统的真实应用，包括：
//! - 自定义中间件实现
//! - 路由分组和权限控制
//! - 请求拦截和响应处理
//!
//! ## 运行方式
//!
//! ### 启动服务器:
//! ```bash
//! cargo run --example router_middleware -- server
//! ```
//!
//! ### 启动客户端:
//! ```bash
//! cargo run --example router_middleware -- client
//! ```
//!
//! ## 架构
//!
//! ```
//! 请求 → [日志中间件] → [认证中间件] → [限流中间件] → [Handler]
//!        ↓ 记录日志       ↓ 检查令牌       ↓ 限流保护         ↓ 业务逻辑
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};

use aerox_core::Result;
use prost::Message;

// 简单的 ID 生成器
fn generate_session_id() -> String {
    use std::time::SystemTime;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("session_{}", timestamp)
}

// ============================================================================
// Protobuf 消息定义
// ============================================================================

/// 认证请求
#[derive(Clone, prost::Message)]
pub struct AuthRequest {
    #[prost(string, tag = "1")]
    pub token: String,
}

/// 认证响应
#[derive(Clone, prost::Message)]
pub struct AuthResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(string, tag = "3")]
    pub session_id: String,
}

/// 公开数据请求
#[derive(Clone, prost::Message)]
pub struct PublicDataRequest {
    #[prost(string, tag = "1")]
    pub query: String,
}

/// 数据响应
#[derive(Clone, prost::Message)]
pub struct DataResponse {
    #[prost(string, tag = "1")]
    pub data: String,
}

/// 管理员请求
#[derive(Clone, prost::Message)]
pub struct AdminRequest {
    #[prost(string, tag = "1")]
    pub command: String,
    #[prost(string, tag = "2")]
    pub params: String,
}

/// 管理员响应
#[derive(Clone, prost::Message)]
pub struct AdminResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub output: String,
}

// 消息 ID 常量
const MSG_ID_AUTH: u16 = 1001;
const MSG_ID_PUBLIC_DATA: u16 = 2001;
const MSG_ID_ADMIN: u16 = 3001;

// ============================================================================
// 中间件系统
// ============================================================================

/// 中间件上下文
#[derive(Clone)]
pub struct MiddlewareContext {
    /// 请求时间戳
    pub request_time: Instant,
    /// 客户端地址
    pub peer_addr: SocketAddr,
    /// 会话数据（认证后设置）
    pub session_id: Option<String>,
    /// 是否已认证
    pub authenticated: bool,
    /// 用户角色
    pub role: Option<String>,
    /// 扩展数据
    pub extensions: HashMap<String, String>,
}

impl MiddlewareContext {
    pub fn new(peer_addr: SocketAddr) -> Self {
        Self {
            request_time: Instant::now(),
            peer_addr,
            session_id: None,
            authenticated: false,
            role: None,
            extensions: HashMap::new(),
        }
    }
}

/// 日志中间件
#[derive(Clone)]
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    pub async fn handle(
        &self,
        ctx: &mut MiddlewareContext,
        msg_id: u16,
        payload: &[u8],
    ) -> Result<()> {
        let elapsed = ctx.request_time.elapsed().as_millis();
        println!(
            "📝 [LOG] {} | MSG_ID: {} | Payload: {} bytes | Time: {}ms",
            ctx.peer_addr,
            msg_id,
            payload.len(),
            elapsed
        );

        // 记录到上下文
        ctx.extensions
            .insert("logged_at".to_string(), format!("{:?}", ctx.request_time));

        Ok(())
    }
}

/// 认证中间件
#[derive(Clone)]
pub struct AuthMiddleware {
    /// 公开路由（不需要认证）
    pub public_routes: Vec<u16>,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            public_routes: vec![MSG_ID_AUTH, MSG_ID_PUBLIC_DATA],
        }
    }

    pub async fn handle(
        &self,
        ctx: &MiddlewareContext,
        msg_id: u16,
    ) -> Result<()> {
        // 检查是否是公开路由
        if self.public_routes.contains(&msg_id) {
            println!("   ↳ [AUTH] 公开路由，跳过认证: {}", msg_id);
            return Ok(());
        }

        // 检查是否已认证
        if !ctx.authenticated {
            println!("   ↳ [AUTH] 未认证，拒绝访问: {}", msg_id);
            return Err(aerox_core::AeroXError::validation(
                "Authentication required".to_string(),
            ));
        }

        println!("   ↳ [AUTH] 已认证用户: {:?}", ctx.session_id);
        Ok(())
    }
}

/// 限流中间件
#[derive(Clone)]
pub struct RateLimitMiddleware {
    /// 每个客户端的请求计数
    pub client_counts: Arc<Mutex<HashMap<SocketAddr, ClientRateInfo>>>,
}

/// 客户端限流信息
#[derive(Clone, Debug)]
struct ClientRateInfo {
    count: u32,
    window_start: Instant,
}

impl ClientRateInfo {
    fn new() -> Self {
        Self {
            count: 0,
            window_start: Instant::now(),
        }
    }
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        Self {
            client_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    const MAX_REQUESTS: u32 = 10; // 每分钟最多 10 个请求
    const WINDOW_DURATION: Duration = Duration::from_secs(60);

    pub async fn handle(
        &self,
        ctx: &MiddlewareContext,
    ) -> Result<()> {
        let mut counts = self.client_counts.lock().await;
        let info = counts
            .entry(ctx.peer_addr)
            .or_insert_with(ClientRateInfo::new);

        // 检查是否需要重置窗口
        if info.window_start.elapsed() >= Self::WINDOW_DURATION {
            println!("   ↳ [RATE] 重置限流窗口: {}", ctx.peer_addr);
            info.count = 0;
            info.window_start = Instant::now();
        }

        // 检查限流
        if info.count >= Self::MAX_REQUESTS {
            println!("   ↳ [RATE] 限流触发: {} (请求数: {})", ctx.peer_addr, info.count);
            return Err(aerox_core::AeroXError::validation(
                "Rate limit exceeded".to_string(),
            ));
        }

        info.count += 1;
        println!(
            "   ↳ [RATE] 请求计数: {} ({}/{})",
            ctx.peer_addr, info.count, Self::MAX_REQUESTS
        );

        Ok(())
    }
}

/// 管理员权限中间件
#[derive(Clone)]
pub struct AdminMiddleware;

impl AdminMiddleware {
    pub async fn handle(&self, ctx: &MiddlewareContext, msg_id: u16) -> Result<()> {
        // 只有管理员路由需要检查
        if msg_id != MSG_ID_ADMIN {
            return Ok(());
        }

        match ctx.role.as_deref() {
            Some("admin") => {
                println!("   ↳ [ADMIN] 管理员权限验证通过");
                Ok(())
            }
            _ => {
                println!("   ↳ [ADMIN] 权限不足: {:?}", ctx.role);
                Err(aerox_core::AeroXError::validation(
                    "Admin role required".to_string(),
                ))
            }
        }
    }
}

// ============================================================================
// 服务器状态
// ============================================================================

#[derive(Clone)]
pub struct ServerState {
    /// 中间件实例
    logging: LoggingMiddleware,
    auth: AuthMiddleware,
    rate_limit: RateLimitMiddleware,
    admin: AdminMiddleware,
    /// 活跃会话
    pub sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

/// 会话信息
#[derive(Clone, Debug)]
struct SessionInfo {
    session_id: String,
    role: String,
    created_at: Instant,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            logging: LoggingMiddleware,
            auth: AuthMiddleware::new(),
            rate_limit: RateLimitMiddleware::new(),
            admin: AdminMiddleware,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 执行中间件链
    pub async fn execute_middleware(
        &self,
        ctx: &mut MiddlewareContext,
        msg_id: u16,
        payload: &[u8],
    ) -> Result<()> {
        // 1. 日志中间件
        self.logging.handle(ctx, msg_id, payload).await?;

        // 2. 认证中间件
        self.auth.handle(ctx, msg_id).await?;

        // 3. 限流中间件
        self.rate_limit.handle(ctx).await?;

        // 4. 管理员权限中间件
        self.admin.handle(ctx, msg_id).await?;

        Ok(())
    }

    /// 创建会话
    pub async fn create_session(&self, session_id: String, role: String) {
        let info = SessionInfo {
            session_id: session_id.clone(),
            role,
            created_at: Instant::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), info);
        println!("   ↳ [SESSION] 创建会话: {}", session_id);
    }

    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
}

// ============================================================================
// 服务器实现
// ============================================================================

/// 运行路由和中间件服务器
pub async fn run_server() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX 路由和中间件示例 - 服务器      ║");
    println!("╚════════════════════════════════════════╝\n");

    let bind_addr: SocketAddr = "127.0.0.1:8081"
        .parse()
        .map_err(|e| aerox_core::AeroXError::validation(format!("Invalid address: {}", e)))?;
    println!("🚀 启动服务器...");
    println!("   地址: {}\n", bind_addr);

    let state = ServerState::new();
    let listener = TcpListener::bind(bind_addr).await?;
    println!("✓ 服务器启动成功，等待连接...\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("支持的消息类型:");
    println!("  [1001] AuthRequest     - 认证（公开）");
    println!("  [2001] PublicData      - 公开数据（公开）");
    println!("  [3001] AdminRequest    - 管理员操作（需认证+管理员权限）");
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

    let mut buffer = [0u8; 8192];
    let mut messages_received = 0u64;

    loop {
        // 读取 AeroX Frame 格式
        match socket.read_exact(&mut buffer[..10]).await {
            Ok(_) => {}
            Err(e) => {
                println!("   ↳ 连接 #{} 已关闭 (接收 {} 条消息)", conn_id, messages_received);
                break;
            }
        }

        // 解析 AeroX Frame 头（小端序）
        let frame_len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
        let msg_id = u16::from_le_bytes([buffer[4], buffer[5]]);
        let _seq_id = u32::from_le_bytes([buffer[6], buffer[7], buffer[8], buffer[9]]);

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

            // 创建中间件上下文
            let mut ctx = MiddlewareContext::new(addr);

            // 执行中间件链
            if let Err(e) = state.execute_middleware(&mut ctx, msg_id, payload).await {
                // 中间件返回错误，发送错误响应
                let error_msg = format!("Middleware error: {}", e);
                let error_response = DataResponse {
                    data: error_msg,
                };
                send_message(&mut socket, msg_id, &error_response).await?;
                continue;
            }

            // 路由到对应的 handler
            match msg_id {
                MSG_ID_AUTH => handle_auth(&mut socket, &state, &mut ctx, payload).await?,
                MSG_ID_PUBLIC_DATA => handle_public_data(&mut socket, payload).await?,
                MSG_ID_ADMIN => handle_admin(&mut socket, &state, &ctx, payload).await?,
                _ => {
                    println!("   ↳ 连接 #{} 未知消息类型: {}", conn_id, msg_id);
                }
            }
        }
    }

    Ok(())
}

/// 处理认证请求
async fn handle_auth(
    socket: &mut TcpStream,
    state: &ServerState,
    ctx: &mut MiddlewareContext,
    payload: &[u8],
) -> Result<()> {
    if let Ok(req) = AuthRequest::decode(payload) {
        println!("   ↳ [AUTH] 收到认证请求: token={}", req.token);

        // 简化的认证逻辑
        let (success, session_id, role) = if req.token == "admin_token" {
            (
                true,
                generate_session_id(),
                "admin".to_string(),
            )
        } else if req.token == "user_token" {
            (
                true,
                generate_session_id(),
                "user".to_string(),
            )
        } else {
            (false, "".to_string(), "".to_string())
        };

        let response = AuthResponse {
            success,
            message: if success {
                "Authentication successful".to_string()
            } else {
                "Invalid token".to_string()
            },
            session_id: session_id.clone(),
        };

        send_message(socket, MSG_ID_AUTH, &response).await?;

        // 如果认证成功，创建会话并更新上下文
        if success {
            state.create_session(session_id.clone(), role.clone()).await;
            ctx.session_id = Some(session_id);
            ctx.authenticated = true;
            ctx.role = Some(role);

            println!("   ↳ [AUTH] 认证成功: role={}", ctx.role.as_ref().unwrap());
        }
    }

    Ok(())
}

/// 处理公开数据请求
async fn handle_public_data(socket: &mut TcpStream, payload: &[u8]) -> Result<()> {
    if let Ok(req) = PublicDataRequest::decode(payload) {
        println!("   ↳ [PUBLIC] 查询: {}", req.query);

        let response = DataResponse {
            data: format!("Public data for query: {}", req.query),
        };

        send_message(socket, MSG_ID_PUBLIC_DATA, &response).await?;
    }

    Ok(())
}

/// 处理管理员请求
async fn handle_admin(
    socket: &mut TcpStream,
    state: &ServerState,
    ctx: &MiddlewareContext,
    payload: &[u8],
) -> Result<()> {
    if let Ok(req) = AdminRequest::decode(payload) {
        println!("   ↳ [ADMIN] 命令: {} {}", req.command, req.params);

        // 验证会话
        if let Some(session_id) = &ctx.session_id {
            if let Some(session) = state.get_session(session_id).await {
                println!("   ↳ [ADMIN] 会话有效: role={}", session.role);

                // 执行管理员命令
                let output = format!("Executed: {} {}", req.command, req.params);
                let response = AdminResponse {
                    success: true,
                    output,
                };

                send_message(socket, MSG_ID_ADMIN, &response).await?;
            } else {
                let response = AdminResponse {
                    success: false,
                    output: "Invalid session".to_string(),
                };
                send_message(socket, MSG_ID_ADMIN, &response).await?;
            }
        }
    }

    Ok(())
}

/// 发送消息（AeroX Frame 格式）
async fn send_message<M: prost::Message>(
    socket: &mut TcpStream,
    msg_id: u16,
    message: &M,
) -> Result<()> {
    let mut buf = Vec::new();
    message.encode(&mut buf).map_err(|e| {
        aerox_core::AeroXError::protocol(format!("Failed to encode message: {:?}", e))
    })?;

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

/// 运行客户端
pub async fn run_client() -> aerox_client::Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   AeroX 路由和中间件示例 - 客户端      ║");
    println!("╚════════════════════════════════════════╝\n");

    use aerox_client::StreamClient;

    let addr: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    println!("🔗 连接到服务器: {}...\n", addr);

    let mut client = StreamClient::connect(addr).await?;
    println!("✓ 连接成功!\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试场景:\n");

    // 1. 测试公开路由
    println!("1️⃣  测试公开数据路由（无需认证）");
    let public_req = PublicDataRequest {
        query: "test_query".to_string(),
    };
    client
        .send_message(MSG_ID_PUBLIC_DATA, &public_req)
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 2. 测试未认证访问受保护路由
    println!("\n2️⃣  测试未认证访问管理员路由（应被拒绝）");
    let admin_req = AdminRequest {
        command: "list_users".to_string(),
        params: "".to_string(),
    };
    client
        .send_message(MSG_ID_ADMIN, &admin_req)
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 3. 认证为普通用户
    println!("\n3️⃣  认证为普通用户");
    let auth_req = AuthRequest {
        token: "user_token".to_string(),
    };
    client.send_message(MSG_ID_AUTH, &auth_req).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 4. 以普通用户身份访问管理员路由
    println!("\n4️⃣  普通用户访问管理员路由（应被拒绝）");
    client
        .send_message(MSG_ID_ADMIN, &admin_req)
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 5. 认证为管理员
    println!("\n5️⃣  认证为管理员");
    let auth_req = AuthRequest {
        token: "admin_token".to_string(),
    };
    client.send_message(MSG_ID_AUTH, &auth_req).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 6. 以管理员身份访问管理员路由
    println!("\n6️⃣  管理员访问管理员路由（应成功）");
    client
        .send_message(MSG_ID_ADMIN, &admin_req)
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 7. 测试限流（快速发送多个请求）
    println!("\n7️⃣  测试限流保护");
    for i in 1..=12 {
        let req = PublicDataRequest {
            query: format!("query_{}", i),
        };
        client
            .send_message(MSG_ID_PUBLIC_DATA, &req)
            .await?;
        println!("   发送请求 {}/12", i);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 测试完成");

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
