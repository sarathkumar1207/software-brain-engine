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
[![npm version](https://img.shields.io/npm/v/sbe-cli.svg)](https://www.npmjs.com/package/sbe-cli)
[![npm downloads](https://img.shields.io/npm/dm/sbe-cli.svg)](https://www.npmjs.com/package/sbe-cli)
[![Version](https://img.shields.io/badge/version-0.3.0-blue.svg)](crates/cli/Cargo.toml)
[![Rust Stable](https://img.shields.io/badge/rust-stable-orange.svg)](rust-toolchain.toml)
[![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](docs/install.md)
[![Language Scope](https://img.shields.io/badge/scope-TypeScript%20%7C%20TSX%20%7C%20Python-3178c6.svg)](docs/architecture.md)
[![Status](https://img.shields.io/badge/status-production--alpha-yellow.svg)](docs/governance.md)

Software Brain Engine (`sbe`) is a local code-intelligence CLI for TypeScript, TSX, and Python projects. It builds a semantic index of your repository, then returns focused impact reports for planned code changes so LLMs do not need to read the whole codebase.

The goal is simple: install once, run `sbe`, and give developers or AI agents the smallest useful context for a change.

The logo represents SBE's core model: a central semantic graph node connected to the exact code symbols and dependency paths that matter for a change.

## Pitch

AI coding tools are powerful, but they often waste context by reading too much code before they understand the change. SBE acts like a local "brain index" for your repository:

- scan the repo once
- store a local binary index under `.sbe/`
- ask a change question such as `jwt to passport`
- get impacted symbols, files, layers, dependencies, and token estimates
- update changed files incrementally without rebuilding the whole graph
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

Graph Intelligence v2 was also validated against a real Fastify checkout:

```text
indexed       : 33 files, 506 symbols
graph         : 207 imports, 2700 edges
impact        : 230 affected symbols, 24 affected files, depth 4
tokens        : full ~84633, sbe ~42335, saved ~42298 (50%)
query time    : 184 ms
```

See [docs/benchmark-fastify.md](docs/benchmark-fastify.md).

Python plugin validation on a real FastAPI checkout:

```text
indexed       : 1120 Python files, 6524 symbols
graph         : 3582 imports, 67320 edges
impact        : FastAPI -> 4481 affected symbols, 706 affected files, depth 6
tokens        : full ~978145, sbe ~696741, saved ~281404 (29%)
update        : 1 changed file, 3 modified symbols, 10815 ms
```

See [docs/benchmark-fastapi.md](docs/benchmark-fastapi.md).

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

SBE is a production-alpha CLI. It is usable for local TypeScript/TSX/Python validation and benchmarking, but it is not yet a full type-aware TypeScript compiler integration or Python static analyzer.

Current scope:

- syntax-based TypeScript/TSX parsing through Tree-sitter
- Python language plugin for classes, functions, async functions, methods, imports, variables, and local references
- binary `.sbe/index.bin` storage
- debug JSON export
- Graph Intelligence v2 typed symbol and relationship graph
- bidirectional caller/callee lookup
- incremental index updates with `sbe update`
- symbol version diffing for added, modified, and removed symbols
- Context Compiler v2.1 for deterministic, budgeted context packs
- Change Simulator v2.2 for modify, delete, and replace predictions
- deterministic risk scoring, affected-flow detection, and recommended test selection
- impact analysis and layer classification
- benchmark and validation reports
- Windows MSI release workflow
- Linux/macOS release archives

Not yet:

- full TypeScript type resolution
- full Python type resolution, decorator evaluation, or runtime import execution
- watch mode
- exact model-tokenizer counting
- editor extension
- large public benchmark suite

## Install

Download the release artifact for your platform:

[GitHub Releases](https://github.com/sarathkumar1207/software-brain-engine/releases/latest)

Windows:

```text
sbe-0.3.0-windows-x64.msi
```

Linux/macOS:

```text
sbe-linux-x64.tar.gz
sbe-macos-arm64.tar.gz
```

Download locations:

| Platform | Artifact | Location |
| --- | --- | --- |
| Windows x64 | `sbe-0.3.0-windows-x64.msi` | [GitHub Releases](https://github.com/sarathkumar1207/software-brain-engine/releases/latest) |
| Linux x64 | `sbe-linux-x64.tar.gz` | [GitHub Releases](https://github.com/sarathkumar1207/software-brain-engine/releases/latest) |
| macOS ARM64 | `sbe-macos-arm64.tar.gz` | [GitHub Releases](https://github.com/sarathkumar1207/software-brain-engine/releases/latest) |

After install:

```powershell
sbe version
sbe --help
```

Recommended npm install:

```bash
npm install -g sbe-cli
sbe version
sbe scan .
```

The npm package downloads the native Rust binary from GitHub Releases on install or first run. It does not require Rust.

Native release downloads:

- [Windows MSI](https://github.com/sarathkumar1207/software-brain-engine/releases/latest)
- [Linux/macOS archives](https://github.com/sarathkumar1207/software-brain-engine/releases/latest)

Developer source install:

```powershell
cargo install --path crates/cli --force
```

Build folders such as `target/`, `dist/`, and `artifacts/` are generated locally. They are not part of the source repo or public release.

## Quick Start

Index a TypeScript, TSX, or Python project:

```powershell
sbe scan C:\path\to\typescript-project
```

Incrementally refresh changed files after a scan:

```powershell
sbe update C:\path\to\typescript-project
```

Check index health:

```powershell
sbe doctor C:\path\to\typescript-project
```

Analyze a planned change:

```powershell
sbe analyze-change "jwt to passport" C:\path\to\typescript-project
```

Compile an AI-ready context pack without calling an AI model:

```powershell
sbe context createUser C:\path\to\typescript-project
sbe context createUser C:\path\to\typescript-project --budget 4000
sbe context createUser C:\path\to\typescript-project --budget 8000 --json
```

Predict impact before changing code:

```powershell
sbe simulate modify createUser C:\path\to\project
sbe simulate delete createUser C:\path\to\project --max-depth 6
sbe simulate replace createUser C:\path\to\project --record --json
```

The simulator answers four practical questions before an edit: what symbols and files are affected, how risky the operation is, which entry-point flows participate, and which connected existing tests should run. `--record` stores the JSON result under `.sbe/reports/` for regression and release comparisons.

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
| `sbe update <path>` | Incrementally update changed files in the existing index. |
| `sbe inspect <symbol> <path>` | Return context packets for a symbol. |
| `sbe graph <symbol> <path>` | Show dependencies and dependents. |
| `sbe impact <symbol> <path>` | Show transitive impact. |
| `sbe context <symbol> <path>` | Compile a ranked, budgeted context pack. |
| `sbe simulate <modify\|delete\|replace> <symbol> <path>` | Predict blast radius, risk, flows, tests, and context before editing. |
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

### Is SBE available through npm?

Yes. The npm package is `sbe-cli`, and it installs the `sbe` command.

```bash
npm install -g sbe-cli
sbe scan .
```

The npm package does not compile Rust locally. It downloads the matching native binary from GitHub Releases and verifies it with `checksums.txt`.

## How SBE Optimizes Tokens

SBE does not claim magic compression. It reduces context by selecting the code slice that appears relevant to a change.

Benchmark flow:

1. Count approximate tokens for all indexed TypeScript/TSX/Python source.
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
- `parser`: TypeScript extraction plus Python language plugin
- `symbols`: in-memory symbol indexes
- `graph`: typed dependency graph with forward/reverse indexes, diffs, impact traversal, and context packs
- `impact`: reverse dependency analysis reports
- `context`: deterministic context compiler, ranking, budget pruning, dependency paths, and code ranges
- `simulator`: bounded graph traversal, risk scoring, flow detection, test selection, and context assembly
- `query`: context, benchmark, and change-analysis reports
- `indexer`: full scan and incremental update pipeline
- `cli`: user-facing command line

See [docs/architecture.md](docs/architecture.md), [docs/context-compiler.md](docs/context-compiler.md), and [docs/change-simulator.md](docs/change-simulator.md).
Review hardening notes are tracked in [docs/review-issues.md](docs/review-issues.md).
CodeRabbit setup and unresolved review findings are tracked in
[docs/coderabbit.md](docs/coderabbit.md) and [docs/review-backlog.md](docs/review-backlog.md).

## Release

GitHub Actions builds release artifacts.

Push to a tracked branch:

```text
build and test
build installer artifacts
```

Push a version tag:

```powershell
git tag v0.3.0
git push origin v0.3.0
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
