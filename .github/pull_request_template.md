## Summary

Describe what changed and why.

## Related Issue

Closes #

## Scope

- [ ] CLI/user-facing behavior
- [ ] Parser/scanner/indexer behavior
- [ ] Storage format
- [ ] Benchmark/validation output
- [ ] Documentation or website
- [ ] CI/release workflow

## Checks

- [ ] `cargo fmt --all --check`
- [ ] `cargo check --workspace --all-targets --all-features --locked`
- [ ] `cargo test --workspace --all-targets --all-features --locked`
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- [ ] User-facing documentation is updated, or no documentation change is required.
- [ ] A regression test is included for bug fixes.

## Compatibility

- [ ] No storage format change
- [ ] No breaking CLI or JSON output change
- [ ] No new security-sensitive behavior

Explain any unchecked compatibility item below.

## Notes

Call out migration concerns, storage compatibility, benchmark changes, or known limitations.
