# Stop Sending Entire Codebases To AI

AI coding tools are useful, but they often waste context before they understand the actual change.

You ask for something focused:

```text
Fix the middleware behavior around createUser.
```

The assistant starts by reading broad folders:

```text
controllers/
services/
dto/
middleware/
routes/
tests/
```

Some of that context is useful. A lot of it is not.

That is the problem I am trying to solve with **Software Brain Engine**, or **SBE**.

SBE is an open-source Rust CLI that turns TypeScript/TSX/Python repositories into semantic graphs. The goal is to retrieve the code that matters before an LLM reads source.

Graph Intelligence v2 also includes a Python language plugin. On a FastAPI checkout, SBE indexed 1120 Python files into 6524 symbols and 67320 edges, then reported 4481 affected symbols for `FastAPI`.

Context Compiler v2.1 adds the next step: `sbe context <symbol>` returns ranked symbols, dependency paths, callers, code ranges, deterministic summaries, token budgets, and context reduction metrics without calling an AI model.

## The Basic Idea

Instead of sending the full repository, ask the codebase for graph context:

```bash
sbe scan ./repo
sbe update ./repo
sbe graph createUser --json
sbe impact createUser --json
```

SBE returns:

- matching symbols
- dependencies
- dependents
- impacted files
- source ranges
- approximate token estimates

This is not magic compression. It is graph-guided context selection.

## Why This Matters

LLMs have limited context and limited architectural awareness. When a coding assistant reads unrelated files, three things happen:

1. Tokens are wasted.
2. The model has more irrelevant context to reason over.
3. Real dependencies can still be missed.

For focused changes, the better workflow is:

```text
query graph -> retrieve relevant ranges -> send focused context -> edit code
```

## Example Demo: Hono Middleware

I added a realistic demo based on `honojs/hono`, a TypeScript web framework.

The simulated issue:

```text
Middleware post-processing after await next() should still mutate an error response created by onError.
```

Normal assistant flow:

```text
Search middleware
Search next
Search onError
Open context, dispatch, router, helper, tests
Infer lifecycle manually
```

Estimated broad context:

```text
12-16 files
~22k-30k tokens
```

SBE flow:

```bash
sbe scan ./hono
sbe graph compose --json
sbe impact compose --json
```

Focused context:

```text
src/compose.ts:18-102
src/hono-base.ts:210-275
src/context.ts:120-175
src/types.ts:40-88
src/helper/factory/index.ts:12-44
src/compose.test.ts:1-90
src/hono-base.test.ts:300-370
```

Estimated focused context:

```text
5-7 files
~5k-7k tokens
```

That is roughly a 68-78% context reduction for this demo shape.

Important: this is a simulated reproducible-concept demo, not a claim that Hono has this bug. Before opening an upstream issue or PR, the test must be reproduced against a real Hono commit.

## What SBE Does Today

Current alpha scope:

- TypeScript/TSX/Python scanning
- Tree-sitter syntax parsing
- symbol extraction
- import/reference edges
- local `.sbe/index.bin`
- graph queries
- impact analysis
- benchmark and validation reports
- human output and JSON output

It skips common unwanted folders by default:

```text
node_modules
.git
.sbe
dist
build
target
.next
coverage
```

## What SBE Does Not Do Yet

SBE is production-alpha, not a finished static analysis platform.

Current limitations:

- TypeScript/TSX/Python source support
- syntax-based, not type-aware yet
- approximate token estimates
- no watch mode yet
- no exact tokenizer yet

I want the benchmark to show misses honestly. Small projects can show 0% savings if metadata overhead is larger than the focused code slice.

## Why Rust?

SBE is a CLI and local indexer. Rust is a good fit because it gives:

- fast filesystem scanning
- reliable native binaries
- good cross-platform release support
- clean modular crate boundaries

The project is organized as a Cargo workspace:

```text
scanner -> parser -> storage -> graph -> impact -> query -> cli
                         \-> update -> graph diff
```

## Feedback Wanted

I am looking for feedback from:

- backend engineers
- TypeScript maintainers
- AI coding assistant users
- Rust CLI developers
- people working on code intelligence tools

The most useful feedback:

```text
I tried SBE on this repo, and the graph was wrong here.
```

GitHub:

```text
https://github.com/sarathkumar1207/software-brain-engine
```

Website:

```text
https://sarathkumar1207.github.io/software-brain-engine/
```

Downloads:

```text
https://github.com/sarathkumar1207/software-brain-engine/releases/latest
```

Platform artifacts:

```text
Windows x64 : sbe-0.2.0-windows-x64.msi
Linux x64   : sbe-linux-x64.tar.gz
macOS ARM64 : sbe-macos-arm64.tar.gz
```
