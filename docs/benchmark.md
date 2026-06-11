# Benchmark Methodology

SBE benchmarks are intended to prove whether local indexing reduces LLM context for a planned code change.

For a real framework benchmark, see [benchmark-fastify.md](benchmark-fastify.md). The Fastify run indexed 33 TypeScript declaration/type-test files, built 506 symbols and 2700 edges, reported 230 affected symbols for `FastifyInstance`, and reduced a focused type-query estimate by 50%.

For a Python framework benchmark, see [benchmark-fastapi.md](benchmark-fastapi.md). The FastAPI run indexed 1120 Python files, built 6524 symbols and 67320 edges, reported 4481 affected symbols for `FastAPI`, and reduced a broad routing/dependency query estimate by 29%.

Context Compiler v2.1 can be benchmarked separately with:

```bash
sbe context <symbol> --budget 4000 --json
```

The output includes selected symbols, selected files, estimated tokens, and context reduction percentage.

## What Is Measured

`sbe benchmark <path> --query "<change>"` reports:

- indexed files
- indexed symbols
- impacted files
- impacted symbols
- impacted layers
- query time
- approximate full-project tokens
- approximate SBE-focused tokens
- saved tokens
- reduction percentage

## Token Estimate

Current alpha builds use an approximate token estimate:

```text
tokens ~= source characters / 4
```

This is not model-tokenizer exact. It is enough to compare broad full-project context against focused SBE context. Exact tokenizer support is on the roadmap.

## Full-Project Context

Full context is estimated from all indexed source files after SBE ignores folders such as:

- `node_modules`
- `.git`
- `.sbe`
- `dist`
- `build`
- `target`
- `.next`
- coverage folders

## SBE-Focused Context

Focused context is estimated from:

1. matched symbols from the query
2. direct dependencies
3. transitive dependents
4. impacted source ranges

SBE merges overlapping symbol ranges per file before counting tokens. This prevents nested class/method ranges from being counted multiple times.

## Example

```powershell
sbe validate C:\var\www\html\codex-backend --query "jwt to passport"
```

Maintainers can run the same validation plus benchmark JSON with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\validate-benchmark.ps1 -ProjectPath C:\var\www\html\codex-backend -Query "jwt to passport"
```

On macOS/Linux:

```bash
scripts/validate-benchmark.sh /path/to/project "jwt to passport"
```

Observed local result:

```text
indexed       : 42 files, 88 symbols
impacted      : 24 files, 49 symbols
tokens        : full ~9469, sbe ~5319, saved ~4150 (44%)
query time    : 3 ms
```

## How To Pitch The Result

SBE gives an LLM a focused change packet instead of a blind full-repo scan. The useful claim is not that every query saves tokens. The claim is:

```text
When the planned change touches a focused subsystem, SBE can reduce context by selecting impacted files, symbols, and layers before the LLM is called.
```

Good benchmark reports should include both wins and misses. If SBE reports no savings, that means the change is broad or the current resolver is too imprecise. Those cases become accuracy work items.

## Git Hygiene

SBE writes generated index data under `.sbe/`, including binary index files and validation reports. Keep `.sbe/` out of source control for normal projects.
