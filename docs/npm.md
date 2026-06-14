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
sbe update .
sbe graph createUser --json
sbe impact saveUser --json
sbe context createUser --budget 4000 --json
sbe simulate modify createUser
sbe simulate delete createUser --max-depth 6 --record
```

`sbe update` passes through to the Rust incremental update engine. It scans file hashes, reparses changed files, refreshes the stored graph, and reports added, modified, removed, and affected symbols.

The same commands were benchmarked against a shallow Fastify checkout. See [`docs/benchmark-fastify.md`](benchmark-fastify.md) for the scan, impact, and token results.

Python support was benchmarked against FastAPI. See [`docs/benchmark-fastapi.md`](benchmark-fastapi.md) for the Python plugin scan, impact, token, and incremental update results.

Context Compiler v2.1 is available through the same npm wrapper because it is implemented in the native Rust binary:

```bash
sbe context createUser
sbe context createUser --budget 8000 --json
```

Change Simulator v2.2 is also implemented in the native binary. It predicts impact before an edit using only the stored dependency graph:

```bash
sbe simulate modify createUser
sbe simulate delete createUser --record
sbe simulate replace createUser --json
```

`--record` writes the machine-readable report to `.sbe/reports/simulation-<operation>-<symbol>-latest.json`.

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
2. Create the matching GitHub Release first, for example `v0.4.0`.
3. Confirm the release contains every npm download asset:

```text
sbe-core-linux-x64
sbe-core-macos-x64
sbe-core-macos-arm64
sbe-core-windows-x64.exe
checksums.txt
```

4. Only then publish the matching npm version, for example `sbe-cli@0.4.0`.

The npm package version and GitHub Release tag must match. `sbe-cli@0.4.0` downloads from `releases/download/v0.4.0/`.

Be careful with manual releases: the tag must be `v0.4.0`, not `vv0.3.0`. A double `v` release can show assets on GitHub but still break npm downloads.

5. Configure npm publishing in GitHub Actions using one of these supported modes:

```text
Recommended: npm Trusted Publishing
- npm package: sbe-cli
- Provider: GitHub Actions
- Repository: sarathkumar1207/software-brain-engine
- Workflow file: npm-publish.yml
- Environment: leave empty unless the workflow uses one
```

```text
Alternative: npm Automation token
- npm account settings -> Access Tokens -> Generate New Token
- Token type must be Automation, not a normal publish token
- GitHub repository secret name: NPM_TOKEN
```

Normal npm publish tokens can fail in CI with `EOTP` when the account has 2FA enabled. GitHub Actions cannot enter an interactive authenticator code, so use Trusted Publishing or an Automation token.

6. From `npm/sbe`, validate locally:

```bash
npm pack --dry-run
```

7. The GitHub workflow publishes with:

```bash
npm publish --access public --provenance
```

8. Verify after publish:

```bash
npx sbe-cli version
npx sbe-cli scan .
```

## Troubleshooting

### `refusing non-GitHub download URL`

GitHub Release assets redirect through GitHub-owned CDN hosts. The npm wrapper allows:

```text
github.com
objects.githubusercontent.com
release-assets.githubusercontent.com
```

If this error appears, upgrade to the latest `sbe-cli` package.

### `checksum validation failed: ENOENT`

This means the temporary native binary disappeared before checksum verification. Older npm wrapper versions could hit this on Windows while following GitHub release redirects. Upgrade to the next patch version and clear the partial cache:

```powershell
npm uninstall -g sbe-cli
Remove-Item -Recurse -Force "$env:USERPROFILE\.sbe\bin\0.3.0" -ErrorAction SilentlyContinue
npm install -g sbe-cli
sbe version
```

Use the version folder that failed in your error message.

### npm version already published

npm versions are immutable. If `sbe-cli@0.4.0` has a wrapper bug, publish `sbe-cli@0.4.0`; do not try to republish `0.2.2`.
