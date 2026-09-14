# tagtree

`tagtree` provides reusable data structures for hierarchical tag paths and an
in-memory index that maps tagged items to those paths.

The crate is maintained in its own repository so it can be tested, versioned,
and published to [crates.io](https://crates.io) separately from the Weimo
applications that currently consume it.

## Features

- Normalize and validate slash-separated tag paths.
- Store items under one or more hierarchical paths with `treebag::Tree`.
- Query descendants through a read-only `treebag::TreeView`.
- Maintain item assignments and rename or delete tag subtrees with
  `tag_index::TagIndex`.

## Development

Run checks from this directory:

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

The package can be checked for registry readiness without publishing:

```bash
cargo publish --dry-run
```
