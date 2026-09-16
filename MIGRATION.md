# Migration Guide

## 0.3.0

`PathTree` is now an internal implementation detail of `TagTree`:

- `tagtree::PathTree` is no longer exported.
- The `tagtree::path_tree` module is no longer public.
- Use `TagTree` for item/tag assignment, queries, and subtree mutations.
- `TagNode` remains available at the crate root for consuming
  `TagTree::summary()` snapshots (`use tagtree::TagNode`).

The current Weimo consumers use `TagTree`, `TagNode`, and the root path
helpers, so no source changes are required when upgrading them to `0.3`.
