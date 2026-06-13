# Security Policy

## Supported Versions

Security fixes are applied to the latest released version and the `main` branch. Older releases
may not receive patches.

## Reporting A Vulnerability

Do not report suspected vulnerabilities in public issues, discussions, or pull requests.

Use GitHub's private vulnerability reporting form:

<https://github.com/sarathkumar1207/software-brain-engine/security/advisories/new>

Include:

- the affected SBE version or commit
- operating system and installation method
- reproduction steps or a minimal proof of concept
- the expected security impact
- any suggested mitigation

You should receive an acknowledgement within 7 days. The maintainer will validate the report,
coordinate a fix and release when necessary, and credit the reporter unless anonymity is
requested. Please allow reasonable time for remediation before public disclosure.

## Security Scope

Security-sensitive areas include release binaries, npm installation and checksum verification,
path handling, generated `.sbe/` data, parsing untrusted repositories, and GitHub Actions
publishing credentials.
