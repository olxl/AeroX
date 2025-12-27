//! AeroX 框架统一错误处理
//!
//! 提供框架级别的错误类型定义和处理机制。

pub mod framework;
pub mod context;

// 重新导出主要类型
pub use framework::{AeroXError, AeroXErrorKind};
pub use context::ErrorContext;

/// AeroX 统一 Result 类型
pub type Result<T> = std::result::Result<T, AeroXError>;
