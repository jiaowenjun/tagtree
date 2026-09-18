# Testing

The test suite is organized around the public seams that callers use. Private
tree storage has focused unit tests, but correctness is ultimately checked
through `TagTree` and the public path helpers.

## Layers

| Layer | Location | Purpose |
| --- | --- | --- |
| Worked examples | `src/lib.rs`, `README.md` | Keep the first-use workflow executable. |
| Public contracts | `tests/tag_tree.rs`, `tests/path.rs` | Lock named behavior, errors, ordering, and regressions. |
| Reference model | `tests/state_model.rs` | Compare mixed operation sequences with an independent map/set model. |
| Internal algorithms | `src/path_tree/*` | Exercise traversal, merge, and snapshot mechanics. |
| Performance | `benches/tag_tree.rs` | Detect cost changes under stable workloads. |

The reference model intentionally knows nothing about arena IDs, hash-table
layout, or detached nodes. It checks item assignments, subtree results,
mutation return values, summaries, failure atomicity, item inspection, empty
path pruning, and the invariant that only truly untagged items are assigned to
the root.

## Commands

Run the release test suite:

```bash
cargo test --locked
```

Increase the number of generated state-machine cases while investigating a
change:

```bash
PROPTEST_CASES=2000 cargo test --locked --test state_model
```

Compile every benchmark without spending time measuring it:

```bash
cargo bench --no-run --locked
```

The supported minimum Rust version is checked separately from current stable:

```bash
cargo +1.88.0 test --locked
cargo +stable test --locked
```

## Adding coverage

- Add a small public regression test for a reported bug before changing code.
- Extend the reference model when a public invariant changes across operation
  sequences.
- Keep implementation-specific cases beside the private algorithm they test.
- Add a benchmark only when it represents a distinct user workload or answers
  a concrete design question.
- Never turn current performance into an undocumented compatibility promise.
