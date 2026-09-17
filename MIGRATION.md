# Migration Guide

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
