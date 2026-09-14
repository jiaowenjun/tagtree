//! # treebag
//!
//! `treebag` 用于管理“分层路径 -> 数据项”的映射关系，适合标签、分类和多路径归档场景。

mod error;
mod node;
mod tree;
mod treeview;

pub use error::TreeError;
pub use error::TreeResult;
pub use tree::Tree;
pub use treeview::TreeView;
