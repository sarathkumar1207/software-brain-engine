# Contributing

Software Brain Engine is a modular Rust CLI. Contributions are welcome, but changes must stay reviewable because the project touches parsing, storage, release artifacts, and AI-context benchmark claims.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md). Usage questions
belong in [GitHub Discussions](https://github.com/sarathkumar1207/software-brain-engine/discussions),
and vulnerabilities must be reported privately according to [SECURITY.md](SECURITY.md).

## Before You Start

- Search existing issues and pull requests before creating new work.
- Open an issue before large features, storage changes, or breaking CLI changes.
- Keep pull requests focused and avoid unrelated refactors.
- Do not include generated `.sbe/`, `target/`, packaged binaries, secrets, or proprietary fixtures.
- Use test fixtures that are minimal and safe to publish.

## Contribution Model

All code changes should go through pull requests.

Maintainer policy:

- Do not push directly to `main`.
- Use focused branches.
- Open a pull request for each meaningful change.
- Require CI to pass before merge.
- Require maintainer review before merge.
- Prefer squash merge for normal feature/fix PRs.
- Use release/version PRs for version bumps.

Repository branch protection must be enabled in GitHub settings. The config files in this repo support the workflow, but GitHub branch protection itself is controlled by the repository settings UI.

## Conventional Commits

Use Conventional Commits so version automation can infer the next release number.

Examples:

```text
fix(parser): avoid false export detection
feat(cli): add watch command
docs(readme): clarify benchmark methodology
test(indexer): cover auth fixture end to end
chore(release): v0.3.0
```

Version rules:

- `fix:` creates a patch bump, for example `0.2.0` -> `0.2.1`.
- `feat:` creates a minor bump, for example `0.2.0` -> `0.3.0`.
- `type!:` or `BREAKING CHANGE:` creates a major bump.

## Release Automation

The release process is PR-based:

1. A normal PR is merged into `main`.
2. The `Version PR` workflow calculates the next semantic version from commit messages.
3. The workflow opens a `chore(release): vX.Y.Z` PR.
4. A maintainer reviews and merges the version PR.
5. The `Tag Release` workflow creates tag `vX.Y.Z`.
6. The `Release` workflow builds downloadable artifacts from the tag.

The release workflow publishes user-facing artifacts only:

- Windows MSI
- Linux archive
- macOS archive

Do not publish `target/`, `.sbe/`, WiX intermediate files, or loose build folders.

## Local Checks

Run these before opening a pull request:

```powershell
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --all-features --no-deps --locked
npm --prefix npm/sbe test
```

For release-sensitive changes, also run:

```powershell
cargo build --release -p sbe-cli
```

For benchmark changes, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\validate-benchmark.ps1 -ProjectPath .\tests\fixtures\auth-ts -Query "jwt to passport"
```

## Design Rules

- `storage` owns `.sbe/` disk layout.
- `common` contains stable shared data types.
- `indexer` is the scan/parse/store orchestration boundary.
- `parser` uses Tree-sitter for V1 syntax-based TypeScript/TSX extraction.
- Do not add a TypeScript compiler, Node.js runtime dependency, or language-server dependency without an architecture update.
- Do not write `.sbe/` files directly from crates outside `storage`.
- Do not claim exact token counts until exact tokenizer support is implemented.

## Testing Expectations

Use focused tests for the behavior being changed.

Add tests when changing:

- parser extraction
- scanner ignore rules
- storage format
- graph or impact traversal
- benchmark token estimates
- CLI output shape
- release/version workflows

For bug fixes, add a regression test that fails before the fix and passes after it.

## Pull Request Review Standard

A PR can be merged when:

- scope is clear
- tests cover changed behavior
- CI passes
- docs are updated when user-facing behavior changes
- storage version impact is understood
- benchmark claims remain honest
- commits contain no secrets or generated local indexes
- public APIs and JSON output remain compatible or document the break

Large changes should be split by subsystem where possible.

## Review And Ownership

Maintainers may request changes for correctness, security, compatibility, test coverage, or scope.
Review is not guaranteed on a specific timeline. The governance and decision process is documented
in [GOVERNANCE.md](GOVERNANCE.md).
