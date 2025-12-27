//! AeroX 框架统一错误处理
//!
//! 提供框架级别的错误类型定义和处理机制。

pub mod context;
pub mod framework;

// 重新导出主要类型
pub use context::ErrorContext;
pub use framework::{AeroXError, AeroXErrorKind};

/// AeroX 统一 Result 类型
pub type Result<T> = std::result::Result<T, AeroXError>;
