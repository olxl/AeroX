//! 配置加载示例
//!
//! 演示如何从文件加载配置并使用环境变量覆盖

use aerox_config::ServerConfig;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AeroX 配置加载示例 ===\n");

    // 示例 1: 使用默认配置
    println!("1. 使用默认配置:");
    let config = ServerConfig::default();
    println!("   地址: {}", config.bind_addr());
    println!("   端口: {}", config.port);
    println!();

    // 示例 2: 从文件加载配置
    println!("2. 从文件加载配置:");
    let config_path = Path::new("examples/config_example.toml");
    match ServerConfig::from_file(config_path) {
        Ok(config) => {
            println!("   ✓ 配置加载成功");
            println!("   地址: {}", config.bind_addr());
            println!("   DDoS 防护: {}", config.enable_ddos_protection);
        }
        Err(e) => {
            println!("   ✗ 配置加载失败: {}", e);
        }
    }
    println!();

    // 示例 3: 配置验证
    println!("3. 配置验证:");
    let valid_config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 9000,
        ..Default::default()
    };
    match valid_config.validate() {
        Ok(_) => println!("   ✓ 配置有效"),
        Err(e) => println!("   ✗ 配置无效: {}", e),
    }
    println!();

    // 示例 4: 无效配置
    println!("4. 无效配置示例:");
    let invalid_config = ServerConfig {
        port: 0,
        ..Default::default()
    };
    match invalid_config.validate() {
        Ok(_) => println!("   ✓ 配置有效"),
        Err(e) => println!("   ✗ 配置无效: {}", e),
    }

    Ok(())
}
