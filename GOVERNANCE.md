# Governance

Software Brain Engine currently uses a maintainer-led governance model.

## Roles

- **Users** run SBE and provide feedback.
- **Contributors** submit issues, documentation, tests, or code through pull requests.
- **Maintainers** review changes, manage releases, handle security reports, and make final project
  decisions.

The current maintainer and code owner is `@sarathkumar1207`.

## Decision Making

Routine decisions are made through public issues and pull requests. Decisions prioritize
correctness, compatibility, maintainability, measurable performance, and the documented project
scope. Significant changes to storage, CLI contracts, supported languages, or release behavior
require an architecture or governance update in the same pull request.

When consensus is not reached, the maintainer records the decision and rationale publicly. This
model may be expanded when sustained contributors need shared ownership.

## Releases And Changes

SBE follows semantic versioning and Conventional Commits. All normal changes use pull requests,
required CI, and maintainer review. Release automation and branch-protection expectations are
documented in [`docs/governance.md`](docs/governance.md).

## Conduct And Security

Participation is governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Vulnerabilities must be
reported privately according to [SECURITY.md](SECURITY.md).
