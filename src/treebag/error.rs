//! 错误类型定义
//!
//! 提供标签树操作中可能出现的错误类型。

use std::fmt;

/// 标签树操作的结果类型别名
///
/// 用于表示可能失败的操作结果，成功时返回 `T`，失败时返回 `TreeError`。
pub type TreeResult<T> = Result<T, TreeError>;

/// 标签树错误类型
///
/// 表示标签树操作中可能出现的各种错误。
#[derive(Debug)]
pub enum TreeError {
    /// 合并操作错误，包含错误描述信息
    MergeError(String),
    /// 路径错误，包含错误描述信息
    PathError(String),
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeError::MergeError(msg) => write!(f, "Merge error: {}", msg),
            TreeError::PathError(msg) => write!(f, "Path error: {}", msg),
        }
    }
}

impl std::error::Error for TreeError {}
