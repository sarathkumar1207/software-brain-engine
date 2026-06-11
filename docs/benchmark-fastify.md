# Fastify Graph Intelligence v2 Benchmark

This benchmark validates SBE against a real Node.js framework repository:

```text
Repository: https://github.com/fastify/fastify
Checkout: shallow clone on 2026-06-11
SBE scope: TypeScript declaration files and TypeScript type tests
```

Fastify is primarily JavaScript, so this benchmark intentionally measures SBE's current TypeScript/TSX surface: `.d.ts`, `.ts`, and type-test files. That makes it a good realism check because SBE should report its useful boundary instead of pretending to understand JavaScript runtime files that it does not yet parse.

## Scan

Command:

```powershell
cargo run -p sbe-cli -- scan C:\tmp\sbe-fastify-benchmark
```

Output:

```text
SBE scan complete
  indexed files : 33
  graph         : 506 symbols, 207 imports, 2700 edges
  read          : 338475 bytes
  index size    : 168637 bytes
  elapsed       : 827 ms
  storage       : C:\tmp\sbe-fastify-benchmark\.sbe
  skipped dirs  : 4
```

## Impact Query

Command:

```powershell
cargo run -p sbe-cli -- impact FastifyInstance C:\tmp\sbe-fastify-benchmark
```

Output after the Graph Intelligence v2 CLI aggregation fix:

```text
Affected Symbols: 230
Affected Files: 24
Depth: 4
```

This query covers the central `FastifyInstance` type and its downstream type-test and declaration dependents. The benchmark exposed a real SBE CLI issue: multiple symbols with the same name caused duplicate human summaries. The fix aggregates non-JSON impact output while preserving detailed JSON output.

## Token Benchmark

Command:

```powershell
cargo run -p sbe-cli -- benchmark C:\tmp\sbe-fastify-benchmark --query "FastifyInstance register overload type provider"
```

Output:

```text
SBE benchmark
  query         : FastifyInstance register overload type provider
  query time    : 184 ms
  indexed       : 33 files, 506 symbols
  impacted      : 30 files, 506 symbols
  tokens        : full ~84633, sbe ~42335, saved ~42298 (50%)
  layers        :
    Dto: 5 files, 46 symbols
    Route: 2 files, 27 symbols
    Test: 9 files, 153 symbols
    Unknown: 14 files, 223 symbols
```

## Result

The benchmark validates that SBE can scan and query a real framework type surface quickly, produce a non-trivial reverse dependency impact result, and reduce broad TypeScript context by roughly half for a focused type-system query.

It also shows a current limitation clearly: JavaScript runtime implementation files in Fastify are outside SBE's current parser scope.
