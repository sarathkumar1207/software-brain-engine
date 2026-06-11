# Architecture

Software Brain Engine is a modular Rust workspace. Each crate has one responsibility and communicates through stable data types from `sbe-common`.

## Data Flow

```text
scanner -> parser -> storage
                 \-> symbols -> graph -> impact -> query -> cli
                         ^          ^
                         |          |
                      update      context
```

`indexer` orchestrates scanning, parsing, persistence, and incremental updates. The CLI calls the indexer for writes and the query layer for reads.

## Storage Boundary

The `.sbe/` directory is private to `sbe-storage`. Other crates pass typed values into storage and receive typed snapshots back. The index is stored as versioned bincode with a JSON export path for debugging.

## Language Boundary

TypeScript and TSX use Tree-sitter syntax parsing. Python uses a local language plugin that extracts classes, functions, async functions, methods, module variables, imports, containment edges, and best-effort local references.

SBE does not perform type-aware name resolution, overload resolution, project-wide TypeScript compiler analysis, Python type checking, decorator evaluation, or runtime import execution.

## Parser Plugins

The parser crate exposes a `LanguagePlugin` trait. The existing parser facade dispatches by file extension:

- `ts` and `tsx`: Tree-sitter TypeScript plugin.
- `py`: Python plugin.

This keeps language support inside the existing parser/indexer pipeline. No database, language server, runtime execution, vector store, or LLM integration is introduced.

## Graph Intelligence v2

The graph crate upgrades SBE from a symbol-only graph into a typed dependency graph:

- `SymbolNode` stores the symbol id, name, kind, file id, line range, and stable content hash.
- `EdgeType` models calls, imports, containment, inheritance, and reverse relationships.
- `SemanticGraph` keeps forward edges for callees/dependencies and reverse edges for callers/dependents.
- `GraphTraversal` exposes O(1) adjacency lookup through `callers(symbol_id)` and `callees(symbol_id)`.
- `ImpactAnalysis` performs iterative BFS over reverse dependencies with cycle protection and optional max depth.
- `GraphDiff` compares old and new graph versions to classify added, modified, and removed symbols.
- `ContextPack` prepares root symbol, dependencies, callers, and file ranges for future retrieval integrations.

No database, vector store, or LLM integration sits in this layer. It remains an in-memory graph over the stored snapshot.

## Incremental Updates

`sbe update` reads the existing snapshot, scans current file hashes, parses only added or modified files, removes deleted-file symbols, preserves unaffected graph portions, refreshes import edges, and writes a new snapshot.

The update flow is:

```text
current file hashes -> changed files -> changed symbols -> graph diff -> affected symbols
```

## Phase Notes

- Phase 0: workspace, docs, CI, and shared types.
- Phase 1: scanner and storage.
- Phase 2: parser and symbol registry.
- Phase 3: graph, impact, and query.
- Phase 4: CLI and end-to-end indexing.
- Graph Intelligence v2: bidirectional graph traversal, symbol version diffing, incremental updates, impact summaries, and context preparation.
