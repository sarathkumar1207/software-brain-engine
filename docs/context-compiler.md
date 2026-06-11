# Context Compiler v2.1

Context Compiler v2.1 turns graph data into deterministic, minimal context packs for future AI integrations.

It does not call an LLM, create embeddings, implement MCP, or perform vector search.

## Pipeline

```text
repository -> graph -> context compiler -> context pack
```

Compilation is deterministic:

1. Locate the root symbol.
2. Expand dependencies.
3. Expand callers.
4. Rank symbols.
5. Apply token budget.
6. Build the context pack.

## Crate Structure

```text
crates/context/src/
  lib.rs
  compiler.rs
  budget.rs
  ranking.rs
  pack.rs
  builder.rs
```

## Data Model

The public output is `ContextPack`:

```rust
pub struct ContextPack {
    pub root_symbol: SymbolId,
    pub summary: String,
    pub symbols: Vec<ContextSymbol>,
    pub dependencies: Vec<DependencyPath>,
    pub callers: Vec<SymbolId>,
    pub code_ranges: Vec<CodeRange>,
    pub metrics: ContextMetrics,
}
```

Symbols include deterministic `importance_score` values in the range `0.0..=100.0`.

## Ranking

Ranking uses configurable weights:

- direct dependency weight
- caller count weight
- impact score
- graph depth
- module crossings

The root symbol is always scored at `100.0`.

## Budgeting

Use:

```bash
sbe context createUser --budget 4000
sbe context createUser --budget 8000
```

When the budget is exceeded, lower-ranked symbols are pruned first.

The compiler never removes:

- root symbol
- direct dependencies

## Metrics

`ContextMetrics` reports:

- selected symbols
- selected files
- estimated tokens
- context reduction percentage

The reduction percentage compares selected symbols against all indexed repository symbols.

## CLI

Human output:

```bash
sbe context createUser
```

JSON output:

```bash
sbe context createUser --json
```

Budgeted JSON output:

```bash
sbe context createUser --budget 4000 --json
```

## Future Compatibility

The context pack is designed to be consumed later by MCP servers, Codex, Claude Code, Cursor, and OpenAI Agents. Those integrations are intentionally not implemented here.
