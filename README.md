# tagtree

[![Crates.io](https://img.shields.io/crates/v/tagtree.svg)](https://crates.io/crates/tagtree)
[![Documentation](https://docs.rs/tagtree/badge.svg)](https://docs.rs/tagtree)
[![CI](https://github.com/jiaowenjun/tagtree/actions/workflows/ci.yml/badge.svg)](https://github.com/jiaowenjun/tagtree/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/tagtree.svg)](https://github.com/jiaowenjun/tagtree/blob/main/LICENSE)

`tagtree` provides in-memory data structures for organizing items under
hierarchical, slash-separated tag paths.

Use [`TagIndex`](https://docs.rs/tagtree/latest/tagtree/tag_index/struct.TagIndex.html)
when items can have multiple tags and tags may be renamed or deleted. Use
[`Tree`](https://docs.rs/tagtree/latest/tagtree/treebag/struct.Tree.html) when
you need direct control over a path-to-items tree.

## Installation

Add `tagtree` to your `Cargo.toml`:

```toml
[dependencies]
tagtree = "0.1"
```

The crate requires Rust 1.85 or newer.

## Quick start

```rust
use tagtree::tag_index::TagIndex;

let mut index = TagIndex::new();
index.upsert(&42, &["work/rust".into(), "favorite".into()]);
index.upsert(&7, &["work/rust/async".into()]);

assert_eq!(index.tags(&42), vec!["favorite", "work/rust"]);
assert_eq!(index.items("work").unwrap(), vec![42, 7]);

// Upserting replaces the item's previous assignments.
index.upsert(&42, &["personal".into()]);
assert_eq!(index.tags(&42), vec!["personal"]);
```

`TagIndex<T>` requires `T: Eq + Hash + Clone`; querying items additionally
requires `T: Ord` so results can be returned in descending order.

## Paths and tags

Paths use `/` as the separator, for example `work/rust/async`. A path must not
start or end with `/`, contain `//`, or contain a blank segment. The root path
is represented by an empty string (`""`).

Use [`normalize_tag`](https://docs.rs/tagtree/latest/tagtree/tag_path/fn.normalize_tag.html)
and [`normalize_tags`](https://docs.rs/tagtree/latest/tagtree/tag_path/fn.normalize_tags.html)
to trim input, skip blank values, remove duplicates, and validate paths before
storing them:

```rust
use tagtree::tag_path::normalize_tags;

let tags = normalize_tags([" math ", "math/algebra", "", "math"])?;
assert_eq!(tags, vec!["math", "math/algebra"]);
# Ok::<(), tagtree::tag_path::TagPathError>(())
```

## Managing assignments

`TagIndex::items(path)` returns items assigned to `path` or any descendant. An
item with no tags is stored at the root and can be queried with `items("")`.

Path mutations return the affected items with their resulting assignments:

```rust
use tagtree::tag_index::TagIndex;

let mut index = TagIndex::new();
index.upsert(&1, &["work/rust".into()]);

let changed = index.rename_path("work/rust", "work/languages")?;
assert_eq!(changed[0].tags, vec!["work/languages"]);

let removed = index.delete_path("work/languages")?;
assert!(removed[0].tags.is_empty());
# Ok::<(), tagtree::treebag::TreeError>(())
```

Unknown paths return a `TreeError` from `rename_path`, `delete_path`, and the
low-level `Tree` query methods. Removing an item with `TagIndex::remove` is a
no-op when the item is not present.

## Lower-level tree API

`Tree<T>` maps paths to sets of items and supports adding, removing, moving,
and querying items. `Tree::get_items` includes descendants, while
`Tree::get_bag` only returns items directly attached to the requested node.
`Tree::view` produces a read-only `TreeView` with item counts and child nodes
for presentation.

## License

Licensed under the [MIT License](LICENSE).

## Contributing

Run the checks before opening a pull request:

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```
