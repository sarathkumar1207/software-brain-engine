# Review Backlog

This ledger tracks unresolved findings from reviews of `main`. Create matching GitHub issues
after repository authentication is available, then add the issue URLs here.

| ID | Priority | Finding | Location | Required fix | GitHub issue | Status |
| --- | --- | --- | --- | --- | --- | --- |
| SBE-REV-001 | High | Import resolution treats package imports and unresolved relative imports as matching every project symbol with the same name, which can create false graph edges and inflated impact reports. | `crates/indexer/src/lib.rs`, `resolve_import_edges` | Only link named imports after resolving the module to a project file; model external or unresolved imports separately and add regression tests. | [#14](https://github.com/sarathkumar1207/software-brain-engine/issues/14) | Raised |
| SBE-REV-002 | Medium | `doctor` checks current files against the index but never reports indexed files that were deleted, so an index can be stale while the command reports no stale path. | `crates/indexer/src/lib.rs`, `stale_files` | Compare in both directions and test deletion after indexing. | [#15](https://github.com/sarathkumar1207/software-brain-engine/issues/15) | Raised |
| SBE-REV-003 | Medium | WalkDir errors are removed with `filter_map(Result::ok)`, so permission and traversal failures disappear instead of being included in scan warnings. | `crates/scanner/src/lib.rs`, `scan_with_report` | Preserve traversal errors in `ScanReport.warnings` and add an accessible platform-safe regression test. | [#16](https://github.com/sarathkumar1207/software-brain-engine/issues/16) | Raised |

## Baseline verification

Reviewed commit: `3e4e7718aae24112c46f794513e40ba0bc3d076b` (`origin/main`)

The following checks passed on June 13, 2026:

```text
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
