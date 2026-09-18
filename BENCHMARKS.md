# Benchmarks

The Criterion suite measures public `TagTree` workflows. It is designed to
answer design questions and catch large regressions, not to manufacture a
single headline number.

## Workloads

| Group | Workload | What it exposes |
| --- | --- | --- |
| `construction` | Build 1,000 and 10,000 item indexes | Ingestion cost for shared and wide taxonomies. |
| `queries` | Root, branch, and item lookups | Traversal, deduplication, cloning, and ordering cost. |
| `updates` | Replace one existing item's tags | Cost of locating an item as path count grows. |
| `churn` | Reclassify one item through 1,000 and 10,000 unique paths | Empty-path pruning, arena reuse, and long-running update cost. |
| `snapshots` | Build `TagTreeSummary` | Tree traversal and descendant deduplication cost. |
| `subtree_mutations` | Move/merge, move to root, and remove a branch | Bulk mutation, root normalization, and returned-assignment cost. |

The shared taxonomy has many items attached to a bounded hierarchy. The wide
taxonomy gives each item a unique leaf, separating path-count scaling from
item-count scaling. The churn fixture ends with one live assignment after many
replacements, so historical paths cannot silently dominate future operations.

## Running

Run the complete suite in release mode:

```bash
cargo bench --bench tag_tree --locked
```

Save a local baseline before changing an implementation:

```bash
cargo bench --bench tag_tree --locked -- --save-baseline before
```

Compare the same machine and toolchain after the change:

```bash
cargo bench --bench tag_tree --locked -- --baseline before
```

Criterion writes reports below `target/criterion`. Do not compare results from
different machines, power modes, Rust versions, or background loads as if they
were equivalent.

Throughput uses the logical work performed by each fixture: indexed items for
construction and snapshots, returned or affected items for subtree operations,
one item for point queries and point updates, and completed reclassifications
for churn. Latency remains the primary comparison; throughput exists to make
scaling within one workload easier to read.

## Interpreting results

- Compare both shared and wide trees; an optimization may trade one shape for
  the other.
- Review allocations or memory separately when changing arena or index layout.
- Treat changes inside Criterion's noise threshold as inconclusive.
- Include the command, commit, Rust version, CPU, and affected workload in a
  performance pull request.
- Shared CI compiles benchmarks but does not enforce wall-clock thresholds;
  dedicated, stable hardware is required before adding performance gates.
