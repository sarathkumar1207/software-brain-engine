# Contributing

Software Brain Engine is built phase-by-phase. Keep changes scoped to one subsystem when possible, and add tests for the crate whose behavior changes.

## Local checks

Run these before opening a pull request:

```powershell
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Design rules

- `storage` owns `.sbe/` disk layout.
- `common` contains data types only.
- `indexer` is the orchestration boundary.
- V1 TypeScript analysis is syntax-based; do not add a Node.js or TypeScript language-server dependency without an architecture update.
- Prefer readable module-level tests over broad end-to-end tests when changing one crate.

