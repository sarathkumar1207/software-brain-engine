# Software Brain Engine

Software Brain Engine (`sbe`) is a Rust CLI that builds a local semantic index for TypeScript and TSX repositories.

V1 focuses on practical syntax-based analysis: file scanning, hashing, symbol extraction, import/reference edges, impact analysis, and machine-readable context packets for future AI integrations.

## Status

This repository is the first open-source V1 implementation. The TypeScript analysis is intentionally syntax-based and does not run the TypeScript type checker yet.

## Install

End users should install SBE from release artifacts, not from `target/`.

- Windows: download and run `sbe-<version>-windows-x64.msi`.
- macOS/Linux: download the platform archive from the release page, then place `sbe` on `PATH`.
- Developers can still use `cargo install --path crates/cli --force`.

Build folders such as `target/`, `dist/`, and `artifacts/` are generated locally and are not part of the source repo or public release.

## Commands

```powershell
cargo run -p sbe-cli -- init
cargo run -p sbe-cli -- scan path\to\repo
cargo run -p sbe-cli -- inspect MySymbol --json
cargo run -p sbe-cli -- graph MySymbol
cargo run -p sbe-cli -- impact MySymbol
cargo run -p sbe-cli -- analyze-change "jwt to passport" path\to\repo
cargo run -p sbe-cli -- benchmark path\to\repo --query "jwt to passport"
cargo run -p sbe-cli -- validate path\to\repo --query "jwt to passport"
cargo run -p sbe-cli -- doctor path\to\repo
cargo run -p sbe-cli -- export-json path\to\repo
```

The binary name is `sbe` when installed:

```powershell
cargo install --path crates/cli
sbe scan .
```

## Workspace

- `common`: shared public data types.
- `scanner`: repository traversal and file hashing.
- `storage`: all `.sbe/` persistence.
- `parser`: Tree-sitter TypeScript extraction.
- `symbols`: in-memory symbol indexes.
- `graph`: directed dependency graph.
- `impact`: reverse dependency analysis.
- `query`: context packet compiler.
- `indexer`: end-to-end indexing pipeline.
- `cli`: user-facing command line.

## Development

```powershell
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

See [docs/architecture.md](docs/architecture.md) for the module contracts and V1 design boundaries.

## AI Impact Analysis

Use `analyze-change` to ask SBE for a focused change report before sending context to an LLM:

```powershell
sbe scan C:\path\to\repo
sbe analyze-change "jwt to passport" C:\path\to\repo --json
```

The report includes matched symbols, affected symbols, impacted files, inferred layers such as auth/middleware/controller/service/DTO/database, impact percentage, and an approximate token comparison between full-repo context and SBE-focused context.

## Production Alpha Commands

SBE writes a binary index to `.sbe/index.bin`. Use `export-json` when you need a readable debug snapshot:

```powershell
sbe export-json C:\path\to\repo
```

Use `doctor` to check whether the index exists and whether files are stale:

```powershell
sbe doctor C:\path\to\repo
```

Use `benchmark` to compare full-project token context against focused SBE context:

```powershell
sbe benchmark C:\path\to\repo --query "jwt to passport"
```

Use `validate` for repeatable real-project testing. It scans the project, runs a benchmark, and writes `.sbe/reports/validation-latest.json`:

```powershell
sbe validate C:\path\to\repo --query "jwt to passport"
```

Install and packaging notes are in [docs/install.md](docs/install.md).
