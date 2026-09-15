//! Organize application data with hierarchical, slash-separated tags.
//!
//! `tagtree` is useful for bookmarks, documents, tasks, notes, and other items
//! that can belong to more than one nested category. Use [`tag_index::TagIndex`]
//! when the library should keep each item's complete tag assignment in sync:
//!
//! ```
//! use tagtree::{tag_index::TagIndex, tag_path::normalize_tags};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut index = TagIndex::<u64>::new();
//! let tags = normalize_tags(["work/rust", "favorite"])?;
//! index.upsert(&42, &tags);
//!
//! assert_eq!(index.items("work")?, vec![42]);
//! assert_eq!(index.tags(&42), vec!["favorite", "work/rust"]);
//! # Ok(())
//! # }
//! ```
//!
//! A path such as `work/rust/async` is stored as a tree. Queries on `work`
//! include items assigned to any descendant, while `tags` reports only the
//! item's actual assignments. An item may appear in several branches at once.
//! Untagged `TagIndex` items are kept at the empty root path and can be queried
//! with `items("")`.
//!
//! Use [`treebag::Tree`] when you need lower-level control over adding,
//! removing, moving, and inspecting path membership. The [`tag_path`] module
//! provides path validation and normalization helpers for external input.

pub mod tag_index;
pub mod tag_path;
pub mod treebag;
