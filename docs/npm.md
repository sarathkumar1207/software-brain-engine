# npm CLI Distribution

The npm package is a thin Node.js wrapper around the native Rust SBE engine.

Package path:

```text
npm/sbe
```

Package name:

```text
sbe-cli
```

The unscoped `sbe` name is already used on npm. The package is published as `sbe-cli`, but it installs a binary named `sbe`.

## User Experience

Global install:

```bash
npm install -g sbe-cli
sbe scan .
```

One-off run:

```bash
npx sbe-cli scan .
```

The user does not need Rust installed.

## Runtime Flow

On postinstall or first run:

1. Detect OS and CPU architecture.
2. Resolve the matching release asset.
3. Check cache under `~/.sbe/bin/<version>/`.
4. Download the native binary if missing.
5. Download `checksums.txt`.
6. Verify SHA256.
7. Execute the Rust binary with the provided arguments.

## Supported Assets

The GitHub Release must include:

| Platform | npm target | Release asset |
| --- | --- | --- |
| Linux x64 | `linux-x64` | `sbe-core-linux-x64` |
| macOS x64 | `macos-x64` | `sbe-core-macos-x64` |
| macOS arm64 | `macos-arm64` | `sbe-core-macos-arm64` |
| Windows x64 | `windows-x64` | `sbe-core-windows-x64.exe` |

The release must also include:

```text
checksums.txt
```

## Command Mapping

Most commands pass directly to the Rust binary:

```bash
sbe scan .
sbe graph createUser --json
sbe impact saveUser --json
```

The npm wrapper adds one user-friendly alias:

```bash
sbe explain "jwt to passport"
```

This maps to:

```bash
sbe analyze-change "jwt to passport"
```

## Safety Rules

- Never compile Rust during npm install.
- Never require users to install Rust.
- Download only from GitHub Releases or GitHub's release object CDN.
- Validate SHA256 from `checksums.txt`.
- Cache under `~/.sbe/bin/<version>/`.
- Allow `SBE_SKIP_DOWNLOAD=1` for CI/package tests.
- Allow `SBE_CORE_PATH=/path/to/sbe` for local development.

## Publish Checklist

1. Merge the npm package changes.
2. Create a version release so `sbe-core-*` assets and `checksums.txt` exist.
3. From `npm/sbe`, run:

```bash
npm pack --dry-run
npm publish --access public
```

4. Verify:

```bash
npx sbe-cli version
npx sbe-cli scan .
```
