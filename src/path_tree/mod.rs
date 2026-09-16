//! Internal path-to-items tree primitives used by [`crate::TagTree`].

mod node;
mod tree;
mod treeview;

pub(crate) use tree::PathTree;
pub use treeview::TagNode;
