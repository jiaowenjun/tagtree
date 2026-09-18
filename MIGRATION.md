# Migration Guide

## 0.4.0

- The minimum supported Rust version is now 1.88.
- `Error` is now `#[non_exhaustive]`. Downstream matches must include a wildcard
  arm, allowing future error variants to be added without repeating the 0.3.1
  compatibility mistake.
- Paths with whitespace padding inside a segment, such as `"work /rust"` or
  `"work/ rust"`, are now rejected instead of creating visually ambiguous
  parallel branches. Leading and trailing whitespace around the complete input
  is still trimmed by `normalize_path` and `set_tags`.
- Empty paths are pruned after their last item or descendant is removed.
  Querying such a path now returns `Error::PathNotFound` instead of `Ok([])`;
  detached arena storage is reused by later paths.
- Moving a subtree to the root now assigns an affected item to the untagged
  root only when the item has no other tags. Previously an item with another
  tag could remain directly assigned to both that tag and the root, causing
  `untagged_item_count` to include a tagged item.
- `TagTree` adds `len`, `is_empty`, and `contains_item`. `TagTreeSummary` now
  implements `Clone`, `PartialEq`, and `Eq`.
- `items_under`, `move_subtree`, and `remove_subtree` now return items in
  ascending order. Version 0.3 used descending order, which embedded an
  application-specific preference in the general-purpose interface. The `Ord`
  bound remains so results stay deterministic.
- Item-to-path lookup now uses an internal reverse index. This does not change
  the public result shape, but avoids scanning the complete tree for
  `tags_for`, `set_tags`, and `remove_item`.

## 0.3.1

Bug fixes. Two of them are visible to downstream code, so this release is not
source-compatible for every consumer:

- `Error` gained the `CannotRemoveRoot` variant. `Error` can be matched
  exhaustively, so downstream `match` expressions without a wildcard arm stop
  compiling until a new arm is added. New `Error` variants will ship in minor
  releases, not patch releases.
- `remove_subtree("")` now returns `Err(Error::CannotRemoveRoot)`. Previously it
  returned `Ok` with every item listed as affected while changing nothing:
  untagged items live at the root path, so removing the root subtree is
  ambiguous. Clear tags per item instead.
- `move_subtree` no longer creates the destination path when the move is
  rejected (`CannotMoveRoot`, `CannotMoveIntoDescendant`). A rejected call now
  leaves the tree unchanged; previously the destination chain became queryable
  via `items_under`.
- `is_within` now treats the empty path as the root that contains every path,
  matching `items_under("")`. Previously `is_within("a", "")` returned `false`.

## 0.3.0

`PathTree` is now an internal implementation detail of `TagTree`:

- `tagtree::PathTree` is no longer exported.
- The `tagtree::path_tree` module is no longer public.
- Use `TagTree` for item/tag assignment, queries, and subtree mutations.
- `TagNode` remains available at the crate root for consuming
  `TagTree::summary()` snapshots (`use tagtree::TagNode`).

The current Weimo consumers use `TagTree`, `TagNode`, and the root path
helpers, so no source changes are required when upgrading them to `0.3`.
