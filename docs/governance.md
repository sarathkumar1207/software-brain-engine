# Repository Governance

This document defines the intended open-source controls for SBE.

## Main Branch Rules

Enable branch protection for `main` in GitHub:

1. Open `Settings`.
2. Open `Branches`.
3. Add a branch protection rule for `main`.
4. Enable `Require a pull request before merging`.
5. Enable `Require approvals` and set at least `1`.
6. Enable `Require review from Code Owners`.
7. Enable `Require status checks to pass before merging`.
8. Select these checks:
   - Rust
   - Release build checks when available
9. Enable `Require branches to be up to date before merging`.
10. Enable `Restrict who can push to matching branches`.
11. Disable direct maintainer bypass unless there is an emergency.

GitHub branch protection is not fully enforceable from source files. The repository includes `CODEOWNERS`, PR templates, issue templates, and CI workflows, but the maintainer must enable branch protection in GitHub settings.

## Merge Policy

Use pull requests for all changes.

Recommended merge strategy:

- Squash merge normal PRs.
- Keep release PRs as normal merge or squash merge with `chore(release): vX.Y.Z`.
- Do not merge PRs with failing CI.
- Do not merge PRs that change benchmark behavior without test coverage.

## Version Policy

SBE uses semantic versioning:

- Patch: bug fixes and documentation that does not change behavior.
- Minor: new commands, new output fields, new parser capability, installer improvements.
- Major: storage format breaking changes, CLI contract breaking changes, incompatible JSON output changes.

Version bumping is automated by the `Version PR` workflow:

1. Normal PR is merged.
2. Workflow reads Conventional Commits since the last `vX.Y.Z` tag.
3. Workflow opens a release PR.
4. Maintainer merges release PR.
5. Tag workflow creates `vX.Y.Z`.
6. Release workflow builds installers and archives.

## Security And Credentials

- Do not store GitHub tokens inside SBE.
- Do not store user project data outside `.sbe/`.
- Do not include `.sbe/` indexes in normal commits.
- Do not publish generated build folders as release assets.

## Issue Discipline

Issues should be concrete:

- bug reports include command output and expected behavior
- feature requests include the workflow they improve
- benchmark issues include query, project shape, and token output

For review findings, create one issue per finding so tests and fixes can be tracked independently.
