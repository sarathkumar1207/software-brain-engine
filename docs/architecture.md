# Architecture

Software Brain Engine is a modular Rust workspace. Each crate has one responsibility and communicates through stable data types from `sbe-common`.

## V1 Data Flow

```text
scanner -> parser -> storage
                 \-> symbols -> graph -> impact -> query -> cli
```

`indexer` orchestrates scanning, parsing, and persistence. The CLI calls the indexer for writes and the query layer for reads.

## Storage Boundary

The `.sbe/` directory is private to `sbe-storage`. Other crates pass typed values into storage and receive typed snapshots back. V1 uses JSON for inspectability and versioned metadata for future migrations.

## TypeScript Boundary

V1 uses Tree-sitter syntax parsing. It extracts declarations, imports, exports, and best-effort references. It does not perform type-aware name resolution, overload resolution, or project-wide TypeScript compiler analysis.

## Phase Notes

- Phase 0: workspace, docs, CI, and shared types.
- Phase 1: scanner and storage.
- Phase 2: parser and symbol registry.
- Phase 3: graph, impact, and query.
- Phase 4: CLI and end-to-end indexing.

