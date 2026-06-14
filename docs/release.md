# Release Policy

SBE releases should look like a normal developer-tool release.

## Public Downloads

Publish only user-facing artifacts:

- npm: `sbe-cli`, which installs the `sbe` command and downloads native release assets.
- Windows: one clickable `.msi` installer.
- Linux: one compressed archive containing the `sbe` binary.
- macOS: one compressed archive containing the `sbe` binary.

Download page:

[GitHub Releases](https://github.com/sarathkumar1207/software-brain-engine/releases/latest)

| Platform | Artifact |
| --- | --- |
| Windows x64 | `sbe-0.4.1-windows-x64.msi` |
| Linux x64 | `sbe-linux-x64.tar.gz` |
| macOS x64 | `sbe-macos-x64.tar.gz` |
| macOS ARM64 | `sbe-macos-arm64.tar.gz` |

Native assets for the npm wrapper:

| Platform | Artifact |
| --- | --- |
| Linux x64 | `sbe-core-linux-x64` |
| macOS x64 | `sbe-core-macos-x64` |
| macOS ARM64 | `sbe-core-macos-arm64` |
| Windows x64 | `sbe-core-windows-x64.exe` |
| Checksums | `checksums.txt` |

Do not publish:

- `target/`
- loose build folders
- WiX source files
- `.wixobj` or `.wixpdb`
- debug `.sbe/` indexes

## Why Installer Source Exists

Files under `packaging/` are source code for generating the installer, the same way Rust files are source code for generating `sbe.exe`. End users should never download those files. They download only the generated `.msi` from the release page.

## Automatic Version Flow

SBE uses a PR-based semantic version flow.

1. Contributors merge normal PRs into `main`.
2. Commit messages follow Conventional Commits.
3. The `Version PR` workflow calculates the next version:
   - `fix:` -> patch
   - `feat:` -> minor
   - `type!:` or `BREAKING CHANGE:` -> major
4. The workflow opens `chore(release): vX.Y.Z`.
5. A maintainer reviews and merges the release PR.
6. The `Tag Release` workflow creates tag `vX.Y.Z`.
7. The `Release` workflow builds the MSI and native archives from the tag.
8. After the GitHub Release contains all `sbe-core-*` assets and `checksums.txt`, the `Publish npm` workflow publishes `sbe-cli@X.Y.Z`.

Do not publish npm before the matching GitHub Release assets exist. The npm wrapper downloads from:

```text
https://github.com/sarathkumar1207/software-brain-engine/releases/download/vX.Y.Z/
```

If the matching release assets are missing, `npm install -g sbe-cli` can install the wrapper but the first `sbe` run will fail with a 404.

For manual Release workflow runs, enter the version as `0.3.0`. The workflow also normalizes `v0.4.1`, but the final GitHub Release tag must be exactly `v0.4.1`, not `vv0.3.0`.

The version bump script updates crate versions, installer docs, release fallback metadata, and README version badges:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bump-version.ps1 -Version 0.3.0
```

## Required Repository Settings

Enable branch protection for `main`:

- require pull requests before merging
- require at least one approval
- require CODEOWNERS review
- require CI status checks
- restrict direct pushes to `main`
- keep branches up to date before merge

Source files cannot fully enforce these settings. They must be enabled in GitHub repository settings.
