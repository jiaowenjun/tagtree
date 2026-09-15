# tagtree

[![Crates.io](https://img.shields.io/crates/v/tagtree.svg)](https://crates.io/crates/tagtree)
[![Documentation](https://docs.rs/tagtree/badge.svg)](https://docs.rs/tagtree)
[![CI](https://github.com/jiaowenjun/tagtree/actions/workflows/ci.yml/badge.svg)](https://github.com/jiaowenjun/tagtree/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/tagtree.svg)](https://github.com/jiaowenjun/tagtree/blob/main/LICENSE)

`tagtree` is an in-memory index for applications that organize items with
hierarchical tags. It fits bookmarks, documents, tasks, notes, or any other
records that need paths such as `work/rust/async` and may belong to more than
one branch at the same time.

## The core idea

Use a path to describe where an item belongs. A single item can have several
paths, and a query on a parent path includes all of its descendants:

```text
tags
|-- work                 -> item 1
|   `-- rust             -> item 2
|       `-- async        -> item 3
|-- favorite             -> item 1
`-- (untagged)           -> item 4
```

In this example, `items("work")` returns items 1, 2, and 3. The item tagged
`work/rust` is not given a separate `work` tag; `work` is simply its ancestor
for queries. Item 1 demonstrates that the same item can also be assigned to a
different branch.

## Choose an API

- **[`TagIndex`](https://docs.rs/tagtree/latest/tagtree/tag_index/struct.TagIndex.html)**
  is the usual starting point. It keeps each item's complete assignment in
  sync, supports multiple tags, and lets you rename or delete a tag subtree.
- **[`Tree`](https://docs.rs/tagtree/latest/tagtree/treebag/struct.Tree.html)**
  is the lower-level path-to-set structure. Use it when your application needs
  to control item movement itself or only needs a generic tree of values.
- **[`tag_path`](https://docs.rs/tagtree/latest/tagtree/tag_path/index.html)**
  contains helpers for cleaning and validating paths received from users or
  other external input.

## Installation

Add `tagtree` to your `Cargo.toml`:

```toml
[dependencies]
tagtree = "0.1"
```

The crate requires Rust 1.85 or newer.

## Quick start with `TagIndex`

The high-level workflow is: normalize input, assign tags, then query by a
parent path.

```rust
use tagtree::{
    tag_index::TagIndex,
    tag_path::normalize_tags,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut index = TagIndex::<u64>::new();

    let item_42_tags = normalize_tags([" work/rust ", "favorite"])?;
    index.upsert(&42, &item_42_tags);

    let item_7_tags = normalize_tags(["work/rust/async"])?;
    index.upsert(&7, &item_7_tags);

    assert_eq!(index.tags(&42), vec!["favorite", "work/rust"]);
    assert_eq!(index.items("work")?, vec![42, 7]);

    // Calling upsert again replaces all previous assignments for the item.
    let new_tags = normalize_tags(["personal"])?;
    index.upsert(&42, &new_tags);
    assert_eq!(index.tags(&42), vec!["personal"]);

    Ok(())
}
```

`TagIndex<T>` stores values using set semantics, so an item is returned only
once even when it matches several paths in the same subtree. `T` must implement
`Eq + Hash + Clone`; `items`, `rename_path`, and `delete_path` additionally
require `Ord` so affected items can be returned in descending order.

## Paths and input cleanup

Paths use `/` as the separator. A normalized path must not start or end with
`/`, contain `//`, or contain a blank segment. The root path is `""`.

The helpers in `tag_path` are useful at input boundaries:

```rust
use tagtree::tag_path::normalize_tags;

let tags = normalize_tags([" math ", "math/algebra", "", "math"])?;
assert_eq!(tags, vec!["math", "math/algebra"]);
# Ok::<(), tagtree::tag_path::TagPathError>(())
```

`normalize_tag` handles one value and returns `None` for blank input.
`normalize_tags` trims values, skips blanks, removes duplicates, and validates
every non-blank path. Invalid input returns `TagPathError` before anything is
stored.

## Updating assignments

An item can be untagged by passing an empty tag list. `TagIndex` keeps such
items at the root and exposes them through `items("")`.

Renaming moves a complete path subtree. If the destination already exists, the
two subtrees are merged, including items and same-named descendants:

```rust
use tagtree::tag_index::TagIndex;

fn main() -> Result<(), tagtree::treebag::TreeError> {
    let mut index = TagIndex::<u64>::new();
    index.upsert(&1, &["work/rust".into()]);

    let changed = index.rename_path("work/rust", "work/languages")?;
    assert_eq!(changed[0].tags, vec!["work/languages"]);

    let removed = index.delete_path("work/languages")?;
    assert!(removed[0].tags.is_empty());
    assert_eq!(index.items("")?, vec![1]);

    Ok(())
}
```

`delete_path` removes the requested subtree but preserves any unrelated tags on
the affected items. Items left with no tags become untagged. The returned
`TagAssignment` values show each affected item and its resulting tags, which is
useful when updating a database or UI after a path mutation. Unknown paths
return `TreeError`; removing an unknown item with `remove` is a no-op.

## Using the lower-level `Tree`

Choose `Tree` when you want direct control over path membership rather than
`TagIndex`'s replace-on-upsert behavior:

```rust
use tagtree::treebag::Tree;

fn main() -> Result<(), tagtree::treebag::TreeError> {
    let mut tree = Tree::<u64>::new("tags");
    tree.add_item(&1, &["work/rust".into()]);
    tree.add_item(&2, &["work/rust/async".into()]);

    let all_under_work = tree.get_items("work")?;
    let only_at_rust = tree.get_bag("work/rust")?;
    assert_eq!(all_under_work.len(), 2);
    assert_eq!(only_at_rust.len(), 1);

    let view = tree.view();
    assert_eq!(view.path, "");
    assert_eq!(view.item_count, 2);

    Ok(())
}
```

`get_items` returns the set union for a node and all descendants. `get_bag`
returns only items attached directly to that node. `TreeView` is a read-only
hierarchical summary with child nodes and descendant item counts, suitable for
rendering in a CLI or UI.

## API reference and license

- [API documentation on docs.rs](https://docs.rs/tagtree)
- [Source repository](https://github.com/jiaowenjun/tagtree)

Licensed under the [MIT License](LICENSE).
