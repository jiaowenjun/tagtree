//! Organize application data with hierarchical, slash-separated tags.
//!
//! `tagtree` is useful for bookmarks, documents, tasks, notes, and other items
//! that can belong to more than one nested category. Use [`TagTree`]
//! when the library should keep each item's complete tag assignment in sync:
//!
//! ```
//! use tagtree::{normalize_paths, TagTree};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut tree = TagTree::<u64>::new();
//! tree.set_tags(&42, normalize_paths(["work/rust", "favorite"])?)?;
//!
//! assert_eq!(tree.items_under("work")?, vec![42]);
//! assert_eq!(tree.tags_for(&42), vec!["favorite", "work/rust"]);
//! # Ok(())
//! # }
//! ```
//!
//! A path such as `work/rust/async` is stored as a tree. Queries on `work`
//! include items assigned to any descendant, while `tags_for` reports only the
//! item's actual assignments. An item may appear in several branches at once.
//! Untagged `TagTree` items are kept at the empty root path and can be queried
//! with `items_under("")`.
//!
//! The [`path`] module provides path validation and normalization helpers for
//! external input.

mod error;
mod tag_tree;

pub mod path;
mod path_tree;

pub use error::{Error, Result};
pub use path::{TagPathError, is_within, normalize_path, normalize_paths, validate_path};
pub use path_tree::TagNode;
pub use tag_tree::{ItemTags, TagTree, TagTreeSummary};
