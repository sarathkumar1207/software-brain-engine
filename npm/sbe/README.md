# SBE npm CLI

This package installs the `sbe` command for Software Brain Engine.

It is a Node.js wrapper around the native Rust engine. The wrapper downloads the correct prebuilt binary from GitHub Releases on install or first run, caches it under `~/.sbe/bin/<version>/`, and forwards commands to the Rust executable.

## Install

```bash
npm install -g sbe
```

Or run without a global install:

```bash
npx sbe scan .
```

## Commands

```bash
sbe scan .
sbe graph createUser --json
sbe impact saveUser
sbe explain "jwt to passport" --json
```

`sbe explain <flow>` maps to the Rust CLI command `sbe analyze-change <flow>`.

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
