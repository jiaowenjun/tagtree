# Performance model

`tagtree` is an in-memory index. Its useful scale depends on both the number of
items and the shape of the taxonomy, so a single item-count limit would be
misleading.

Use these terms when estimating a workload:

- `N`: distinct items;
- `P`: live path nodes, including intermediate ancestors;
- `A`: direct item-to-path assignments;
- `K`: paths assigned to the item being inspected or changed;
- `S`: nodes and assignments in the queried or mutated subtree;
- `R`: distinct items returned by a query.

Hash-table operations below are expected-time rather than worst-case bounds.

| Operation | Expected cost | Notes |
| --- | --- | --- |
| `len`, `is_empty`, `contains_item` | `O(1)` | Uses the internal item index. |
| `tags_for` | `O(K * depth + K log K)` | Reconstructs and sorts only that item's paths. |
| `set_tags`, `remove_item` | Proportional to the item's old/new assignments and path depth | Does not scan unrelated paths. |
| `items_under` | `O(S + R log R)` | Traverses the selected subtree, deduplicates items, clones results, then sorts them. |
| `move_subtree` | Proportional to the moved/merged subtree and affected assignments | Also returns the resulting tags for every affected item. |
| `remove_subtree` | Proportional to affected items and their assignments | Preserves tags outside the removed subtree. |
| `summary` | Full-tree traversal plus descendant-item deduplication | Treat as a snapshot operation, not a per-request constant-time counter. |

Memory is `O(N + P + A)`: items are stored in path bags and assignments are
also represented in the reverse index. Empty leaf paths are detached after
their last assignment disappears, and their arena slots are reused. The arena
therefore follows peak live path shape rather than growing once for every path
ever observed.

See [BENCHMARKS.md](BENCHMARKS.md) for reproducible workloads. When reporting a
performance issue, include `N`, `P`, average and maximum tags per item, taxonomy
depth, update/query ratio, and whether the workload is shared, wide, or
churn-heavy.
