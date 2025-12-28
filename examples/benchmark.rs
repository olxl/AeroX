//! AeroX 多线程多用户性能测试
//!
//! 测试服务器在高并发下的性能表现
//!
//! 运行:
//! ```bash
//! cargo run --example benchmark
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;

use prost::Message;

// ========== 配置 ==========

/// 测试配置
struct BenchmarkConfig {
    /// 服务器地址
    server_addr: &'static str,
    /// 客户端连接数
    num_clients: usize,
    /// 测试持续时间（秒）
    duration_secs: u64,
}

impl BenchmarkConfig {
    const fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:8080",
            num_clients: 64,  // 全局配置，可随时修改
            duration_secs: 15,
        }
    }
}

// ========== 消息定义 ==========

/// 请求消息
#[derive(Clone, prost::Message)]
struct BenchmarkRequest {
    #[prost(uint64, tag = "1")]
    client_id: u64,
    #[prost(uint64, tag = "2")]
    sequence: u64,
    #[prost(string, tag = "3")]
    data: String,
}

/// 响应消息
#[derive(Clone, prost::Message)]
struct BenchmarkResponse {
    #[prost(uint64, tag = "1")]
    client_id: u64,
    #[prost(uint64, tag = "2")]
    sequence: u64,
    #[prost(string, tag = "3")]
    echo_data: String,
}

// 消息 ID
const MSG_ID_REQUEST: u16 = 2001;
const MSG_ID_RESPONSE: u16 = 2002;

// ========== 服务器实现 ==========

async fn run_server() -> aerox::Result<()> {
    println!("🚀 启动性能测试服务器...");
    println!("📡 监听地址: {}", BenchmarkConfig::default().server_addr);
    println!("👥 预期客户端数: {}", BenchmarkConfig::default().num_clients);
    println!();

    let result = aerox::Server::bind(BenchmarkConfig::default().server_addr)
        .route(MSG_ID_REQUEST, |ctx| {
            Box::pin(async move {
                // 解码请求
                match BenchmarkRequest::decode(ctx.data().clone()) {
                    Ok(request) => {
                        // 创建响应（原路返回）
                        let response = BenchmarkResponse {
                            client_id: request.client_id,
                            sequence: request.sequence,
                            echo_data: request.data,
                        };

                        let response_bytes = prost::Message::encode_to_vec(&response);
                        let _ = ctx.respond(MSG_ID_RESPONSE, response_bytes.into()).await;
                    }
                    Err(e) => {
                        eprintln!("⚠️  解码请求失败: {}", e);
                    }
                }
                Ok(())
            })
        })
        .run()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

// ========== 客户端实现 ==========

/// 单个客户端任务
async fn run_client(
    client_id: u64,
    server_addr: &'static str,
    duration_secs: u64,
    total_counter: std::sync::Arc<AtomicU64>,
    running: std::sync::Arc<tokio::sync::Semaphore>,
    stop_flag: std::sync::Arc<AtomicU64>,
) -> aerox::Result<u64> {
    // 连接服务器
    let mut client = match aerox::Client::connect(server_addr).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 客户端 {} 连接失败: {}", client_id, e);
            return Err(e.into());
        }
    };

    // 克隆计数器用于闭包
    let counter = total_counter.clone();
    let stop_flag_for_handler = stop_flag.clone();

    // 注册响应处理器
    client
        .on_message(MSG_ID_RESPONSE, move |_msg_id, _response: BenchmarkResponse| {
            let counter = counter.clone();
            let stop_flag = stop_flag_for_handler.clone();
            Box::pin(async move {
                // 始终计数响应，即使测试已结束
                // 因为响应可能比发送晚到达
                let count = counter.fetch_add(1, Ordering::Relaxed);
                // 每1000条消息打印一次，且在测试时间内才打印
                // if count % 10000 == 0 && stop_flag.load(Ordering::Relaxed) == 0 {
                //     println!("  客户端已收到 {} 条响应", count + 1);
                // }
                Ok(())
            })
        })
        .await?;

    // 等待一小段时间确保处理器注册完成
    sleep(Duration::from_millis(100)).await;

    // 等待所有客户端准备好
    running.acquire().await.unwrap().forget();
    drop(running.clone());

    // 发送消息循环 - 使用 stop_flag 而不是本地计时
    let mut sequence = 0u64;
    let data = format!("Hello from client {}", client_id);
    let mut sent_count = 0u64;

    eprintln!("  [DEBUG] 客户端 {} 开始发送消息循环", client_id);

    // 持续发送直到收到停止标志
    while stop_flag.load(Ordering::Relaxed) == 0 {
        sequence += 1;

        let request = BenchmarkRequest {
            client_id,
            sequence,
            data: data.clone(),
        };

        if let Err(e) = client.send(MSG_ID_REQUEST, &request).await {
            eprintln!("  [DEBUG] 客户端 {} 发送失败: {}", client_id, e);
            break;
        }

        sent_count += 1;

        // 每1000条消息打印一次（减少输出频率）
        // if sent_count % 10000 == 0 {
        //     eprintln!("  [DEBUG] 客户端 {} 已发送 {} 条消息", client_id, sent_count);
        // }

        // 添加一个极短的yield，让客户端有机会检查stop_flag
        // 如果send阻塞了，这个yield会让出CPU，让stop_flag检查更及时
        tokio::task::yield_now().await;
    }

    eprintln!("  [DEBUG] 客户端 {} 发送循环结束, 共发送 {} 条消息", client_id, sent_count);

    // 打印发送总数
    if sent_count > 0 {
        eprintln!("  客户端 {} 总共发送 {} 条消息", client_id, sent_count);
    }

    // 等待一小段时间让最后的响应到达
    sleep(Duration::from_millis(500)).await;

    Ok(sent_count)
}

// ========== 性能测试主逻辑 ==========

async fn run_benchmark() -> aerox::Result<()> {
    let config = BenchmarkConfig::default();

    println!("╔══════════════════════════════════════════╗");
    println!("║   AeroX 性能测试 (Benchmark)           ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("📊 测试配置:");
    println!("  • 客户端数量: {}", config.num_clients);
    println!("  • 测试时长: {} 秒", config.duration_secs);
    println!("  • 服务器地址: {}", config.server_addr);
    println!();

    // 全局消息计数器
    let total_counter = std::sync::Arc::new(AtomicU64::new(0));

    // 停止标志
    let should_stop = std::sync::Arc::new(AtomicU64::new(0));

    // 启动服务器
    let server_handle = tokio::spawn(run_server());

    // 等待服务器启动
    sleep(Duration::from_millis(500)).await;

    println!("🔗 开始创建客户端连接...");
    let start_time = std::time::Instant::now();

    // 用于同步所有客户端同时开始
    let running = std::sync::Arc::new(tokio::sync::Semaphore::new(0));
    let mut client_handles = Vec::new();

    // 创建所有客户端任务
    for i in 0..config.num_clients {
        let client_counter = total_counter.clone();
        let client_running = running.clone();
        let client_should_stop = should_stop.clone();

        let handle = tokio::spawn(async move {
            match run_client(
                i as u64,
                config.server_addr,
                config.duration_secs,
                client_counter,
                client_running,
                client_should_stop,
            )
            .await
            {
                Ok(sent_count) => Some(sent_count),
                Err(e) => {
                    eprintln!("❌ 客户端 {} 错误: {}", i, e);
                    None
                }
            }
        });

        client_handles.push(handle);

        // 每10个客户端打印一次进度
        if (i + 1) % 10 == 0 {
            print!("  已创建 {} 个客户端...\r", i + 1);
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }

    println!();
    println!("✅ {} 个客户端已创建完成", config.num_clients);
    println!();

    // 等待一小段时间确保所有客户端都已准备好
    sleep(Duration::from_millis(500)).await;

    // 开始测试：释放所有permit让所有客户端同时开始
    println!("🚀 开始性能测试...");
    println!("⏱️  测试进行中...\n");
    running.add_permits(config.num_clients);

    // 启动进度显示任务
    let progress_counter = total_counter.clone();
    let progress_handle = tokio::spawn(async move {
        let mut last_count = 0u64;
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            let current = progress_counter.load(Ordering::Relaxed);
            let ops = current - last_count;

            if ops > 0 {
                println!("  📊 实时 OPS: {:.2} ops/s (总计: {} 条消息)",
                         ops as f64, current);
                last_count = current;
            }
        }
    });

    // 使用 interval 进行精确计时
    let mut interval = tokio::time::interval(Duration::from_secs(config.duration_secs));
    interval.tick().await; // 第一次tick立即返回，第二次才是duration_secs后

    // 使用 select! 同时等待计时器和 Ctrl+C
    tokio::select! {
        _ = interval.tick() => {
            println!("⏹️  测试时间到（{} 秒）", config.duration_secs);
            progress_handle.abort();
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n⚠️  测试被用户中断");
            progress_handle.abort();
        }
    }

    // 设置停止标志，告诉客户端停止计数
    should_stop.store(1, Ordering::Relaxed);
    println!("⏹️  设置停止标志，等待客户端完成...");

    // 给一点时间让响应处理器完成最后的工作
    sleep(Duration::from_millis(500)).await;

    eprintln!("  [DEBUG] 开始等待 {} 个客户端完成...", config.num_clients);

    // 等待所有客户端完成
    println!("⏳ 等待客户端完成...");
    let mut completed = 0;
    let mut total_sent_count = 0u64;
    let mut total_failed = 0;

    let elapsed = start_time.elapsed();
    for (idx, handle) in client_handles.into_iter().enumerate() {
        match tokio::time::timeout(Duration::from_millis(10), handle).await {
            Ok(Ok(Some(sent_count))) => {
                total_sent_count += sent_count;
                completed += 1;

                if (completed + total_failed) % 10 == 0 {
                    print!("  已完成 {} 个客户端...\r", completed + total_failed);
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
            }
            Ok(Ok(None)) => {
                // 客户端失败，但没有发送数据
                completed += 1;
                total_failed += 1;
            }
            Ok(Err(e)) => {
                eprintln!("  客户端 {} 任务错误: {:?}", idx, e);
                total_failed += 1;
            }
            Err(_) => {
                // eprintln!("  客户端 {} 任务超时（5秒）", idx);
                total_failed += 1;
            }
        }
    }

    println!();
    eprintln!("  [DEBUG] 客户端完成统计: 成功={}, 失败={}", completed, total_failed);
    println!("✅ 已完成: {} 个客户端", completed);
    if total_failed > 0 {
        println!("⚠️  失败/超时: {} 个客户端", total_failed);
    }

    // 停止服务器
    server_handle.abort();

    
    let total_messages = total_counter.load(Ordering::Relaxed);

    println!();
    println!("╔══════════════════════════════════════════╗");
    println!("║           测试结果统计                   ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("  ⏱️  实际运行时间: {:.2} 秒", elapsed.as_secs_f64());
    println!("  📤 总发送消息: {}", total_sent_count);
    println!("  📥 总接收消息: {}", total_messages);
    println!("  📊 发送 QPS: {:.2}", total_sent_count as f64 / elapsed.as_secs_f64());
    println!("  📊 接收 QPS: {:.2}", total_messages as f64 / elapsed.as_secs_f64());
    println!("  📊 总 OPS: {:.2}", (total_sent_count + total_messages) as f64 / elapsed.as_secs_f64());
    println!();

    // 性能评级（基于总OPS）
    let total_ops = total_sent_count + total_messages;
    let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();
    let rating = if ops_per_sec >= 200000.0 {
        "🏆 优秀 (Excellent)"
    } else if ops_per_sec >= 100000.0 {
        "👍 良好 (Good)"
    } else if ops_per_sec >= 50000.0 {
        "✓ 及格 (Acceptable)"
    } else {
        "⚠️  需要优化 (Needs Optimization)"
    };
    println!("  性能评级: {}", rating);

    println!();
    println!("✅ 性能测试完成！");

    Ok(())
}

#[tokio::main]
async fn main() -> aerox::Result<()> {
    run_benchmark().await
}
