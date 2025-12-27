//! 错误上下文
//!
//! 为错误提供额外的上下文信息。

use std::fmt;

/// 错误上下文信息
#[derive(Debug, Clone)]
pub enum ErrorContext {
    /// 键值对上下文
    KeyValue(String, String),
    /// 自定义上下文
    Custom(String),
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorContext::KeyValue(key, value) => write!(f, "{}: {}", key, value),
            ErrorContext::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<(&str, String)> for ErrorContext {
    fn from((key, value): (&str, String)) -> Self {
        ErrorContext::KeyValue(key.to_string(), value)
    }
}

impl From<(&str, &str)> for ErrorContext {
    fn from((key, value): (&str, &str)) -> Self {
        ErrorContext::KeyValue(key.to_string(), value.to_string())
    }
}

impl From<String> for ErrorContext {
    fn from(msg: String) -> Self {
        ErrorContext::Custom(msg)
    }
}

impl From<&str> for ErrorContext {
    fn from(msg: &str) -> Self {
        ErrorContext::Custom(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_from_tuple() {
        let ctx: ErrorContext = ("key", "value").into();
        assert!(matches!(ctx, ErrorContext::KeyValue(_, _)));
        assert_eq!(ctx.to_string(), "key: value");
    }

    #[test]
    fn test_context_from_string() {
        let ctx: ErrorContext = "error message".into();
        assert!(matches!(ctx, ErrorContext::Custom(_)));
        assert_eq!(ctx.to_string(), "error message");
    }
}
