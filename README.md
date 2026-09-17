# tagtree

[![Crates.io](https://img.shields.io/crates/v/tagtree.svg)](https://crates.io/crates/tagtree)
[![Documentation](https://docs.rs/tagtree/badge.svg)](https://docs.rs/tagtree)
[![CI](https://github.com/jiaowenjun/tagtree/actions/workflows/ci.yml/badge.svg)](https://github.com/jiaowenjun/tagtree/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/tagtree.svg)](https://github.com/jiaowenjun/tagtree/blob/main/LICENSE)

`tagtree` is an in-memory collection for assigning items to hierarchical tag
paths. It fits bookmarks, documents, tasks, notes, or any records that may
belong to more than one branch, such as `work/rust/async` and `favorite`.

## The core idea

An item is assigned to one or more complete paths. Querying a parent includes
all descendants, but does not create extra assignments:

```text
tags
|-- work                 -> item 1
|   `-- rust             -> item 2
|       `-- async        -> item 3
|-- favorite             -> item 1
`-- (untagged)           -> item 4
```

`items_under("work")` returns items 1, 2, and 3. The item assigned to
`work/rust` is not separately assigned to `work`; `work` is its ancestor for
queries. An item can also appear in another branch, as item 1 does above.

## Installation

Add the published crate to your application:

```toml
[dependencies]
tagtree = "0.3"
```

The crate requires Rust 1.85 or newer.

## Quick start

Use [`TagTree`](https://docs.rs/tagtree/latest/tagtree/struct.TagTree.html) for
the normal item-centric workflow:

```rust
use tagtree::TagTree;

fn main() -> Result<(), tagtree::Error> {
    let mut tags = TagTree::<u64>::new();

    tags.set_tags(&42, [" work/rust ", "favorite"])?;
    tags.set_tags(&7, ["work/rust/async"])?;

    assert_eq!(tags.tags_for(&42), vec!["favorite", "work/rust"]);
    assert_eq!(tags.items_under("work")?, vec![42, 7]);

    // set_tags replaces all previous tags for this item.
    tags.set_tags(&42, ["personal"])?;
    assert_eq!(tags.tags_for(&42), vec!["personal"]);

    Ok(())
}
```

`set_tags` trims, skips blank values, removes duplicates, validates paths, and
stores an empty tag list as an untagged item. Items are set-like, so a query
returns each matching item once. `items_under` returns values in descending
order; `T` therefore needs `Eq + Hash + Clone + Ord` for queries and subtree
mutations.

## Updating tags

`move_subtree` moves every path and item below a path. If the destination
already exists, the two subtrees are merged. `remove_subtree` removes the
subtree while preserving unrelated tags on affected items:

```rust
use tagtree::TagTree;

fn main() -> Result<(), tagtree::Error> {
    let mut tags = TagTree::<u64>::new();
    tags.set_tags(&1, ["work/rust"])?;

    let changed = tags.move_subtree("work/rust", "work/languages")?;
    assert_eq!(changed[0].tags, vec!["work/languages"]);

    let removed = tags.remove_subtree("work/languages")?;
    assert!(removed[0].tags.is_empty());
    assert_eq!(tags.items_under("")?, vec![1]);

    Ok(())
}
```

The returned [`ItemTags`](https://docs.rs/tagtree/latest/tagtree/struct.ItemTags.html)
values contain each affected item and its resulting tags, which is useful for
updating a database or UI. Removing an unknown item with `remove_item` is a
no-op; unknown paths return `Error::PathNotFound`. A rejected mutation leaves
the tree unchanged. `remove_subtree("")` returns `Error::CannotRemoveRoot`
because untagged items live at the root path; `move_subtree` uses `""` as the
destination to strip the leading segment.

## Paths

Paths use `/` as the separator. A valid path must not start or end with `/`,
contain `//`, or contain a blank segment. The empty path (`""`) is the root.

The path helpers are available at the crate root and in the [`path`] module:

```rust
use tagtree::normalize_paths;

let paths = normalize_paths([" math ", "math/algebra", "", "math"])?;
assert_eq!(paths, vec!["math", "math/algebra"]);
# Ok::<(), tagtree::TagPathError>(())
```

`normalize_path` handles one value and returns `None` for blank input.
`validate_path` checks one path without changing it. `is_within` tests whether
one path is a descendant of another path, including equality; the empty path
is the root and contains every path.

## API reference and license

- [API documentation on docs.rs](https://docs.rs/tagtree)
- [Source repository](https://github.com/jiaowenjun/tagtree)

Licensed under the [MIT License](LICENSE).
