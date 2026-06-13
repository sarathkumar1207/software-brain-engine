# CodeRabbit Reviews

The repository uses CodeRabbit's free tier for automatic pull-request reviews into `main`.
Repository behavior is configured in `.coderabbit.yaml`.

## One-time GitHub setup

1. Sign in at <https://app.coderabbit.ai/> with the GitHub account that owns the repository.
2. Install the CodeRabbit GitHub App for `sarathkumar1207/software-brain-engine`.
3. Grant access only to this repository when GitHub offers repository selection.
4. Open or update a pull request whose base branch is `main`.
5. Confirm that the CodeRabbit check and walkthrough appear on the pull request.

CodeRabbit reviews new non-draft pull requests automatically and reruns an incremental review
after each push. To request a fresh review in a pull-request comment, use:

```text
@coderabbitai full review
```

CodeRabbit findings should be converted into GitHub issues when they cannot be fixed in the
current pull request. Record the issue link and resolution in `docs/review-backlog.md` so the
repository keeps a durable audit trail.

The existing Rust workflow remains authoritative for formatting, compilation, tests, and
Clippy. CodeRabbit supplements those checks; it does not replace them.
