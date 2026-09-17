# Migration Guide

## 0.3.1

Behavior fixes with no breaking API changes:

- `remove_subtree("")` now returns `Error::CannotRemoveRoot`. Previously it
  returned every item as affected while changing nothing: untagged items live
  at the root path, so removing the root subtree is ambiguous. Clear tags per
  item instead.
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
