# Roadmap

`tagtree` is a focused in-memory index for hierarchical tags. Its intended
feature set is deliberately small: assign tags, query items and subtrees,
move or remove subtrees, and inspect a read-only summary.

Version 0.4 establishes that scope. After 0.4, development is maintenance
oriented: make the existing behavior more dependable, faster, easier to
diagnose, and harder to regress. This roadmap does not plan feature growth or
promise release dates.

## Maintenance principles

Priorities are evaluated in this order:

1. Correctness and failure atomicity come before performance.
2. Existing public behavior and SemVer compatibility come before convenience.
3. Performance changes require reproducible measurements and must preserve
   correctness invariants.
4. New implementation machinery must remove more complexity than it adds.
5. Documentation, tests, and benchmarks are part of the maintained contract.

The public API should remain small and stable. Future releases may clarify
errors, fix incorrect behavior, or optimize internals, but should not add new
product capabilities.

## Current main: 0.4.0 candidate

The latest published release is 0.3.1. The current `main` branch is preparing
0.4.0 with:

- a non-exhaustive public error type and preserved error sources;
- stricter rejection of visually ambiguous paths;
- deterministic ascending result order;
- an internal item-to-node reverse index;
- pruning and reuse of detached arena nodes;
- constant-time expected item count and membership inspection;
- a stateful reference-model test and regression coverage for known bugs;
- Criterion workloads for construction, queries, updates, churn, snapshots,
  and subtree mutations;
- CI coverage for Rust 1.88, stable Rust, rustdoc, benchmark compilation, and
  SemVer checks.

The 0.4 exit criteria are synchronized public documentation, migration notes
for every observable behavior change, no known violation of mutation failure
atomicity, and passing release checks on the minimum and stable toolchains.

## After 0.4: maintenance-only evolution

### P0: Correctness and bug prevention

- Treat every reported bug as a regression-test opportunity.
- Keep node relationships, path lookup, direct assignments, reverse-index
  assignments, and summary counts consistent after every mutation.
- Verify that rejected operations never leave paths, assignments, or recycled
  arena slots in a partially changed state.
- Expand state-model operation sequences and adversarial cases when they expose
  a real gap in existing coverage.
- Prefer small, auditable fixes over broad internal redesigns.

### P1: Error behavior and compatibility

- Keep errors deterministic, inspectable through the standard error chain, and
  specific enough to diagnose invalid input or rejected mutations.
- Preserve state on every error path and test that property explicitly.
- Add or refine error cases only when existing behavior is ambiguous; avoid an
  ever-growing error taxonomy.
- Review public API and behavior changes with SemVer tooling and document every
  caller-visible difference in `MIGRATION.md`.
- Raise the MSRV only deliberately, with a documented benefit and CI coverage.

### P2: Performance and memory predictability

- Maintain benchmarks for shared, wide, deep, and churn-heavy taxonomies.
- Track scaling against item count, live path count, assignment count, subtree
  size, and tags per item instead of relying on one headline number.
- Measure latency, allocations, peak memory, and arena reuse where the tooling
  is reliable enough to reproduce results.
- Optimize only after a benchmark identifies a meaningful bottleneck; require
  the state model and regression suite to remain unchanged or stronger.
- Keep the ordinary dependency tree empty unless a dependency provides a
  measured maintenance or performance benefit that outweighs its cost.

### P3: Test and benchmark quality

- Keep worked examples, public contract tests, internal algorithm tests, and
  the independent reference model aligned with the same semantics.
- Increase property-test cases in scheduled or release validation while
  keeping ordinary local runs fast.
- Add targeted fuzzing or interpreter-based checks only when they protect an
  identified risk in path parsing or mutation logic.
- Keep benchmark fixtures deterministic and document hardware, toolchain, and
  commands with any published comparison.
- Compile benchmarks in shared CI; use stable, dedicated hardware before
  introducing performance thresholds.

### P4: Release and documentation reliability

- Keep README examples, rustdoc, complexity notes, tests, and migration
  guidance synchronized with shipped behavior.
- Continue minimum-toolchain and stable-toolchain validation, rustdoc warning
  checks, SemVer comparison, package inspection, and publish dry-runs.
- Publish only when the GitHub tag, workflow result, and crates.io version can
  be independently verified.
- Keep maintenance policies concise enough that contributors can execute them
  consistently.

## 1.0 readiness

Version 1.0 is a stability milestone, not a feature milestone. It requires:

- a settled path, ordering, mutation, and error contract;
- no known correctness defects in supported operations;
- sustained regression and state-model coverage across maintenance releases;
- documented and reproducible performance characteristics;
- demonstrated upgrade and SemVer discipline; and
- evidence from real downstream use that the existing scope is sufficient.

No feature is waiting for 1.0. If those conditions are met, the current small
interface can become the stable interface.

## Permanently out of scope

The roadmap does not include serialization support, persistence, storage
backends, import/export formats, async APIs, concurrent mutation, full-text
search, authorization, query languages, UI components, or framework adapters.
It also does not plan bulk, borrowed, iterator, or convenience variants of the
existing operations merely to expand the API surface.

Applications own durable storage, serialization, synchronization, and domain
policy. Keeping those responsibilities outside `tagtree` is how the crate
remains understandable and dependable.

Feedback is welcome through
[GitHub issues](https://github.com/jiaowenjun/tagtree/issues). Reports are most
useful when they include a minimal reproduction or benchmark, the Rust and
crate versions, item and path counts, tags per item, taxonomy shape, and the
expected versus observed result.
