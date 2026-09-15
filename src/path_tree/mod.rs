//! Advanced path-to-items tree primitives.
//!
//! Most callers should use [`crate::TagTree`]. `PathTree` is available when an
//! application needs direct control over path membership.

mod node;
mod tree;
mod treeview;

pub use tree::PathTree;
pub use treeview::TagNode;
