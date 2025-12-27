//! 环境变量覆盖示例
//!
//! 演示如何使用环境变量覆盖配置

use aerox_config::ServerConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AeroX 环境变量覆盖示例 ===\n");

    // 示例 1: 设置环境变量
    println!("1. 设置环境变量:");
    std::env::set_var("AEROX_PORT", "9999");
    std::env::set_var("AEROX_BIND_ADDRESS", "127.0.0.1");
    std::env::set_var("AEROX_MAX_CONNECTIONS", "5000");
    println!("   AEROX_PORT=9999");
    println!("   AEROX_BIND_ADDRESS=127.0.0.1");
    println!("   AEROX_MAX_CONNECTIONS=5000");
    println!();

    // 示例 2: 加载默认配置并应用环境变量覆盖
    println!("2. 加载默认配置并应用环境变量覆盖:");
    let config = ServerConfig::default().load_with_env_override()?;

    println!("   ✓ 配置加载成功:");
    println!("     - 地址: {}", config.bind_addr());
    println!("     - 最大连接数: {:?}", config.max_connections);
    println!();

    // 示例 3: 验证配置
    println!("3. 验证配置:");
    match config.validate() {
        Ok(_) => println!("   ✓ 配置有效"),
        Err(e) => println!("   ✗ 配置无效: {}", e),
    }
    println!();

    // 示例 4: 查看配置摘要
    println!("4. 配置摘要:");
    println!("{}", config.summary());
    println!();

    // 清理环境变量
    std::env::remove_var("AEROX_PORT");
    std::env::remove_var("AEROX_BIND_ADDRESS");
    std::env::remove_var("AEROX_MAX_CONNECTIONS");

    println!("5. 清理环境变量后:");
    let config = ServerConfig::default().load_with_env_override()?;
    println!("   地址: {}", config.bind_addr());
    println!("   端口: {}", config.port);

    Ok(())
}
