<p align="center">
  <img src="assets/sbe-logo.svg" alt="Software Brain Engine logo" width="128" height="128">
</p>

<h1 align="center">Software Brain Engine</h1>

<p align="center">
  Local semantic graph infrastructure for precise AI code context retrieval.
</p>

[![Rust](https://github.com/sarathkumar1207/software-brain-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/sarathkumar1207/software-brain-engine/actions/workflows/rust.yml)
[![Release](https://github.com/sarathkumar1207/software-brain-engine/actions/workflows/release.yml/badge.svg)](https://github.com/sarathkumar1207/software-brain-engine/actions/workflows/release.yml)
[![Website](https://github.com/sarathkumar1207/software-brain-engine/actions/workflows/pages.yml/badge.svg)](https://github.com/sarathkumar1207/software-brain-engine/actions/workflows/pages.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](crates/cli/Cargo.toml)
[![Rust Stable](https://img.shields.io/badge/rust-stable-orange.svg)](rust-toolchain.toml)
[![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](docs/install.md)
[![Language Scope](https://img.shields.io/badge/scope-TypeScript%20%7C%20TSX-3178c6.svg)](docs/architecture.md)
[![Status](https://img.shields.io/badge/status-production--alpha-yellow.svg)](docs/governance.md)

Software Brain Engine (`sbe`) is a local code-intelligence CLI for TypeScript and TSX projects. It builds a semantic index of your repository, then returns focused impact reports for planned code changes so LLMs do not need to read the whole codebase.

The goal is simple: install once, run `sbe`, and give developers or AI agents the smallest useful context for a change.

The logo represents SBE's core model: a central semantic graph node connected to the exact code symbols and dependency paths that matter for a change.

## Pitch

AI coding tools are powerful, but they often waste context by reading too much code before they understand the change. SBE acts like a local "brain index" for your repository:

- scan the repo once
- store a local binary index under `.sbe/`
- ask a change question such as `jwt to passport`
- get impacted symbols, files, layers, dependencies, and token estimates
- send the focused context to an LLM instead of the full codebase

SBE is useful when the change is specific enough to map to code layers: auth migrations, API changes, DTO updates, controller/service refactors, middleware rewrites, database model changes, and similar engineering work.

## Why SBE

Large codebases waste tokens when an LLM has to inspect broad folders before it can understand a focused change. SBE indexes the project locally and answers questions like:

```powershell
sbe benchmark C:\path\to\repo --query "jwt to passport"
```

Instead of sending the full repository, SBE reports:

- matched symbols
- affected symbols
- impacted files
- impacted layers such as auth, middleware, controller, service, DTO, and database
- approximate full-context tokens vs SBE-focused tokens
- scan/query timing

Example real local validation:

```text
indexed       : 42 files, 88 symbols
impacted      : 24 files, 49 symbols
tokens        : full ~9469, sbe ~5319, saved ~4150 (44%)
query time    : 3 ms
```

This is not a promise that every query saves tokens. Small projects or broad changes may show no savings. That honesty is the point: SBE gives benchmark evidence, not marketing-only claims.

## Who Should Use It

- Developers using AI coding assistants on TypeScript backends.
- Teams that want local-first code context before sending data to an LLM.
- Open-source maintainers who want repeatable impact analysis.
- Agent builders who need structured context packets instead of raw repository dumps.
- Engineers who want to benchmark token savings before pitching an AI workflow.

## Live Demo: Hono Middleware Analysis

SBE includes a realistic demo based on [`honojs/hono`](https://github.com/honojs/hono), focused on a middleware lifecycle issue that requires multi-file understanding.

Demo question:

```text
When middleware calls await next(), does post-processing still mutate an error response created by onError?
```

SBE flow:

```bash
sbe scan ./hono
sbe graph compose --json
sbe impact compose --json
```

Demo estimate:

| Metric | Without SBE | With SBE |
| --- | ---: | ---: |
| Files read | 12-16 | 5-7 |
| Context size | ~22k-30k tokens | ~5k-7k tokens |
| Dependency awareness | Manual inference | Graph-guided |

This is a simulated, reproducible-concept demo. It is not an upstream Hono bug claim until verified against a specific Hono commit with a failing test.

See [docs/demo-hono.md](docs/demo-hono.md).

## Status

SBE is a production-alpha CLI. It is usable for local TypeScript/TSX validation and benchmarking, but it is not yet a full type-aware TypeScript compiler integration.

Current scope:

- syntax-based TypeScript/TSX parsing through Tree-sitter
- binary `.sbe/index.bin` storage
- debug JSON export
- impact analysis and layer classification
- benchmark and validation reports
- Windows MSI release workflow
- Linux/macOS release archives

Not yet:

- full TypeScript type resolution
- watch mode
- exact model-tokenizer counting
- editor extension
- large public benchmark suite

## Install

Download the release artifact for your platform.

Windows:

```text
sbe-0.2.0-windows-x64.msi
```

Linux/macOS:

```text
sbe-linux-x64.tar.gz
sbe-macos-arm64.tar.gz
```

After install:

```powershell
sbe version
sbe --help
```

Developer install:

```powershell
cargo install --path crates/cli --force
```

NPM package:

```text
Not published yet. Use GitHub Releases or cargo install for now.
```

Build folders such as `target/`, `dist/`, and `artifacts/` are generated locally. They are not part of the source repo or public release.

## Quick Start

Index a project:

```powershell
sbe scan C:\path\to\typescript-project
```

Check index health:

```powershell
sbe doctor C:\path\to\typescript-project
```

Analyze a planned change:

```powershell
sbe analyze-change "jwt to passport" C:\path\to\typescript-project
```

Benchmark token optimization:

```powershell
sbe benchmark C:\path\to\typescript-project --query "jwt to passport"
```

Run repeatable validation:

```powershell
sbe validate C:\path\to\typescript-project --query "jwt to passport"
```

Run the maintainer benchmark script:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\validate-benchmark.ps1 -ProjectPath C:\path\to\typescript-project -Query "jwt to passport"
```

On macOS/Linux:

```bash
scripts/validate-benchmark.sh /path/to/typescript-project "jwt to passport"
```

Export the binary index for debugging:

```powershell
sbe export-json C:\path\to\typescript-project
```

## Commands

| Command | Purpose |
| --- | --- |
| `sbe init <path>` | Create `.sbe/` metadata. |
| `sbe scan <path>` | Build or refresh the local index. |
| `sbe inspect <symbol> <path>` | Return context packets for a symbol. |
| `sbe graph <symbol> <path>` | Show dependencies and dependents. |
| `sbe impact <symbol> <path>` | Show transitive impact. |
| `sbe analyze-change <query> <path>` | Explain affected layers/files/symbols for a planned change. |
| `sbe benchmark <path> --query <query>` | Compare full-project tokens vs focused SBE context. |
| `sbe validate <path> --query <query>` | Scan, benchmark, and write `.sbe/reports/validation-latest.json`. |
| `sbe doctor <path>` | Check index health and stale files. |
| `sbe export-json <path>` | Export `.sbe/index.bin` to readable JSON. |
| `sbe version` | Print version and storage metadata. |

`.sbe/` is local runtime index data, similar to a build cache. It is ignored by this repository and should be ignored in projects that use SBE.

## FAQ

### Does SBE scan `node_modules`?

No. SBE skips dependency, VCS, build, and generated folders by default, including:

```text
node_modules
.git
.sbe
dist
build
target
.next
coverage
```

The goal is to index source code, not installed packages or generated output.

### Is SBE published on npm?

Not yet. The current distribution path is GitHub Releases for binaries/installers and `cargo install` for developers.

An npm package can be added later as a convenience wrapper around the native binary. Until that exists, the README intentionally does not show an npm badge.

## How SBE Optimizes Tokens

SBE does not claim magic compression. It reduces context by selecting the code slice that appears relevant to a change.

Benchmark flow:

1. Count approximate tokens for all indexed TypeScript/TSX source.
2. Match the query to symbols and files.
3. Traverse dependencies and dependents.
4. Merge overlapping symbol ranges so nested symbols are not double-counted.
5. Estimate focused context tokens from actual source characters.
6. Report saved tokens and reduction percentage.

See [docs/benchmark.md](docs/benchmark.md) for the benchmark methodology and how to interpret results.

## Architecture

SBE is a Rust workspace:

- `common`: shared public data types
- `scanner`: repository traversal and file hashing
- `storage`: binary `.sbe/` persistence
- `parser`: Tree-sitter TypeScript extraction
- `symbols`: in-memory symbol indexes
- `graph`: directed dependency graph
- `impact`: reverse dependency analysis
- `query`: context, benchmark, and change-analysis reports
- `indexer`: end-to-end indexing pipeline
- `cli`: user-facing command line

See [docs/architecture.md](docs/architecture.md).
Review hardening notes are tracked in [docs/review-issues.md](docs/review-issues.md).

## Release

GitHub Actions builds release artifacts.

Push to a tracked branch:

```text
build and test
build installer artifacts
```

Push a version tag:

```powershell
git tag v0.2.0
git push origin v0.2.0
```

Then GitHub publishes release downloads.

See [docs/install.md](docs/install.md) and [docs/release.md](docs/release.md).
See [docs/marketing.md](docs/marketing.md) for launch messaging and community posting templates.
See [docs/launch-campaign.md](docs/launch-campaign.md) for a concrete 7-day launch plan and ready-to-post copy.
See [docs/governance.md](docs/governance.md) for branch protection, PR review, and version automation rules.

## Website

The static website lives in [website/](website/). GitHub Pages deployment is configured in `.github/workflows/pages.yml` and runs on pushes to `main` that change the website.

Open locally:

```text
website/index.html
```

## Development

```powershell
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --release -p sbe-cli
```

Contributions should use pull requests and Conventional Commits. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Roadmap

- `sbe watch` for automatic incremental indexing
- exact tokenizer support
- richer TypeScript import/call resolution
- public benchmark corpus
- editor integration
- signed installers
- package-manager distribution

## License

MIT
