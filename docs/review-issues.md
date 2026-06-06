# Review Issue Ledger

This ledger records the 15 review issues from the alpha hardening pass and the regression coverage added with the fix. GitHub issues can mirror these titles when `gh` or the GitHub UI is available.

| # | Issue | Fix | Test Coverage |
| --- | --- | --- | --- |
| 1 | `is_exported()` false positives from byte prefix | Export detection now checks Tree-sitter `export_statement` parents. | `export_detection_uses_ast_not_prefix_text` |
| 2 | `symbol_id()` collisions with multiplier `10_000` | Symbol IDs now hash `(file_id, index)` with BLAKE3. | `symbol_ids_do_not_overlap_at_legacy_multiplier_boundary` |
| 3 | Local reference edges create parent/child false edges | Reference edge generation skips containment pairs. | `local_reference_edges_do_not_duplicate_contains_edges` |
| 4 | Import edge resolver reads stale absolute file paths | Resolver reads `snapshot.root + relative_path`. | `import_edges_read_from_snapshot_root_relative_path` |
| 5 | `slice_by_range()` byte counting is not Unicode-safe | Range slicing now works by source lines. | `unicode_source_ranges_still_slice_reference_bodies` |
| 6 | CLI `validate` writes report twice | CLI computes report path first and writes once. | `validate_auth_fixture_end_to_end` |
| 7 | Auth service layer classified as `Service` | Layer precedence now keeps `src/auth/auth.service.ts` as `Auth`. | `classifies_auth_change_layers` |
| 8 | Missing `Variable` and `TypeAlias` symbol kinds | Added shared symbol variants and parser extraction. | `extracts_variables_and_type_aliases` |
| 9 | Sequential parser bottleneck | Indexer parses files in parallel with one parser per worker. | `indexes_sample_project_end_to_end` |
| 10 | `Store::open()` creates `.sbe` for read-only commands | Storage has explicit `open_or_create` and `open_existing`. | `open_existing_does_not_create_storage`, `doctor_does_not_create_storage_for_unindexed_repo` |
| 11 | `import_names()` fragile text parsing | Import extraction prefers AST identifiers and keeps safer text fallback. | `import_name_parser_preserves_identifiers_containing_import` |
| 12 | No `.sbe/` gitignore guidance | README, install, and benchmark docs explain `.sbe/` as generated local data. | Documentation coverage |
| 13 | Token estimation reads stale absolute file paths | Query engine reads from snapshot root and relative path. | `token_estimate_uses_snapshot_root_not_absolute_file_path` |
| 14 | `TypeScriptParser::new()` unnecessarily fallible | Constructor is infallible and `Default` is implemented. | `default_parser_parses_files` |
| 15 | No auth fixture end-to-end test | Added CLI integration test over auth fixture. | `validate_auth_fixture_end_to_end` |
