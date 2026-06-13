# SBE npm CLI

This package installs the `sbe` command for Software Brain Engine.

It is a Node.js wrapper around the native Rust engine. The wrapper downloads the correct prebuilt binary from GitHub Releases on install or first run, caches it under `~/.sbe/bin/<version>/`, and forwards commands to the Rust executable.

SBE indexes TypeScript, TSX, and Python projects. Python support is provided by the built-in Python language plugin.

## Install

```bash
npm install -g sbe-cli
```

Or run without a global install:

```bash
npx sbe-cli scan .
```

## Commands

```bash
sbe scan .
sbe update .
sbe graph createUser --json
sbe impact saveUser
sbe context createUser --budget 4000 --json
sbe simulate modify createUser
sbe simulate delete createUser --max-depth 6 --record
sbe simulate replace createUser --json
sbe explain "jwt to passport" --json
```

`sbe explain <flow>` maps to the Rust CLI command `sbe analyze-change <flow>`.

`sbe update` incrementally refreshes changed files in an existing `.sbe` index. `sbe impact <symbol>` prints affected symbol count, affected file count, and traversal depth; use `--json` for the detailed report.

`sbe context <symbol>` compiles a deterministic, budgeted context pack for future AI tools without calling an AI model.

`sbe simulate <modify|delete|replace> <symbol>` predicts the blast radius before code is edited. It traverses dependencies and callers, scores risk, detects affected entry-point flows, recommends connected existing tests, and compiles a context pack. Add `--record` to save the JSON report under `.sbe/reports/`.

SBE analysis is deterministic and graph-based. It does not call AI, execute project code, or claim compiler-grade TypeScript or Python type resolution.

## Supported Platforms

- Linux x64
- macOS x64
- macOS arm64
- Windows x64

## Safety

- Does not compile Rust during npm install.
- Does not require Rust on the user's machine.
- Downloads only from GitHub Releases.
- Validates SHA256 checksums from `checksums.txt`.
- Supports `SBE_SKIP_DOWNLOAD=1` to skip postinstall download and retry on first run.
- Supports `SBE_CORE_PATH=/path/to/sbe` for local development.
