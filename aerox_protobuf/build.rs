//! Protobuf 构建脚本
//!
//! 在编译时自动生成 Rust 代码

use std::io::Result;

fn main() -> Result<()> {
    // 配置 prost-build
    prost_build::Config::new()
        // 生成所有字段
        .enable_type_names()
        // 生成更友好的代码
        .protoc_arg("--proto_path=proto")
        .extern_path(".aerox", "::aerox_protobuf::proto")
        // 编译所有的 proto 文件
        .compile_protos(&["proto/messages.proto"], &["proto/"])?;

    println!("cargo:rerun-if-changed=proto/messages.proto");

    Ok(())
}
