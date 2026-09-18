# Roadmap

`tagtree` aims to be a focused, dependable in-memory index for hierarchical
tags. The crate should hide tree maintenance behind a small `TagTree`
interface, behave predictably under mixed updates, and publish performance
claims only when they are backed by reproducible measurements.

This roadmap describes direction, not a promise of dates. Each release is
gated by tests, documentation, SemVer review, and measurements relevant to the
change.

## Current main

The latest published release is 0.3.1. The current `main` branch is preparing
0.4.0 and supports:

- assigning a complete tag set to an item;
- querying an item or a complete subtree;
- moving, merging, and removing subtrees;
- presenting a read-only tree summary;
- constant-time expected item membership and count inspection;
- item-to-path lookup without a complete tree scan;
- pruning empty paths and reusing detached arena slots;
- deterministic owned results;
- Rust 1.88 and newer.

The public interface is exercised by worked examples, regression tests, and a
stateful reference-model test. Criterion benchmarks cover both shared
taxonomies and wide trees. Performance numbers are not yet a stable contract.

## 0.4: Contract and lifecycle hardening

Goal: remove ambiguous behavior before more callers depend on it.

Included in the 0.4.0 candidate:

- Keep public error enums `#[non_exhaustive]` and document downstream matching.
- Reject whitespace-padded segments while preserving spaces inside names.
- Prune empty paths and reuse detached arena nodes so long-running update
  workloads track live state instead of historical churn.
- Maintain an internal item-to-node reverse index and verify it against an
  independent state model.
- Return items in deterministic ascending order. Retain the `Ord` requirement
  rather than expose hash iteration order through the public interface.
- Keep common inspection operations (`len`, `is_empty`, and `contains_item`)
  on the existing `TagTree` interface rather than exposing storage internals.
- Add automated SemVer comparison against the latest published release.

Exit criteria: every behavior change has migration guidance; stateful tests
cover the new invariants; benchmarks include update-heavy and churn-heavy
workloads; no known mutation violates failure atomicity.

## 0.5: Scale and memory

Goal: make cost predictable for larger, long-lived indexes.

- Measure item count and path count independently instead of treating tree size
  as one number.
- Measure and tune the reverse index's memory overhead under realistic tag
  counts per item.
- Add bulk construction and update operations when they reduce repeated scans
  without enlarging the ordinary interface unnecessarily.
- Measure peak memory and detached-node churn in addition to elapsed time.
- Investigate borrowed or iterator-based query results where they provide a
  material benefit without exposing internal storage.

Exit criteria: documented complexity expectations, reproducible results at
representative sizes, and no optimization that weakens correctness tests.

## 0.6: Integration readiness

Goal: make adoption and upgrades routine for downstream applications.

- Evaluate optional serialization for public snapshots and mutation results;
  do not serialize internal arena layout.
- Add fuzzing for path parsing and mutation sequences, plus Miri runs for the
  supported toolchain where useful.
- Publish complete examples for rebuilding an index from application data and
  applying returned `ItemTags` updates transactionally.
- Document feature policy, MSRV policy, security reporting, and support scope.

Exit criteria: a new user can evaluate fit, integrate the crate, upgrade it,
and diagnose performance without reading the implementation.

## 1.0: Stable contract

`1.0` requires a settled path model, error model, ordering contract, complexity
documentation, and at least one release cycle of downstream use after the last
breaking interface change. It also requires clean SemVer checks, sustained
model/fuzz coverage, and published benchmark methodology.

## Non-goals

`tagtree` will not become a database, full-text search engine, authorization
system, UI tree widget, or async runtime abstraction. Applications own durable
storage and synchronization. Keeping those concerns outside the module lets
the in-memory tag interface stay small and useful.

Feedback and workload reports are welcome through
[GitHub issues](https://github.com/jiaowenjun/tagtree/issues). Reports are most
useful when they include item count, distinct path count, average tags per
item, update/query ratio, and a minimal example.
