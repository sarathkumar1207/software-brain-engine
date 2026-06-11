# FastAPI Python Plugin Benchmark

This benchmark validates SBE's Python language plugin against a real, widely used Python framework repository:

```text
Repository: https://github.com/fastapi/fastapi
Checkout: shallow clone on 2026-06-11
SBE scope: Python source files
```

The Python plugin is intentionally local and lightweight. It extracts Python classes, functions, async functions, methods, module variables, imports, containment edges, and best-effort local reference edges. It does not run Python, import packages, evaluate decorators, or perform type checking.

## Scan

Command:

```powershell
cargo run -p sbe-cli -- scan C:\tmp\sbe-fastapi-benchmark
```

Output:

```text
SBE scan complete
  indexed files : 1120
  graph         : 6524 symbols, 3582 imports, 67320 edges
  read          : 3911239 bytes
  index size    : 2938794 bytes
  elapsed       : 17058 ms
  storage       : C:\tmp\sbe-fastapi-benchmark\.sbe
  skipped dirs  : 2
```

## Impact Queries

Commands:

```powershell
cargo run -p sbe-cli -- impact FastAPI C:\tmp\sbe-fastapi-benchmark
cargo run -p sbe-cli -- impact APIRouter C:\tmp\sbe-fastapi-benchmark
cargo run -p sbe-cli -- impact Depends C:\tmp\sbe-fastapi-benchmark
```

Outputs:

```text
FastAPI
Affected Symbols: 4481
Affected Files: 706
Depth: 6

APIRouter
Affected Symbols: 4481
Affected Files: 706
Depth: 9

Depends
Affected Symbols: 4483
Affected Files: 707
Depth: 7
```

These queries cover FastAPI's central application object, routing abstraction, dependency injection helper, and their broad test/documentation surface.

## Token Benchmark

Command:

```powershell
cargo run -p sbe-cli -- benchmark C:\tmp\sbe-fastapi-benchmark --query "FastAPI APIRouter Depends dependency injection route"
```

Output:

```text
SBE benchmark
  query         : FastAPI APIRouter Depends dependency injection route
  query time    : 10108 ms
  indexed       : 1120 files, 6524 symbols
  impacted      : 732 files, 5015 symbols
  tokens        : full ~978145, sbe ~696741, saved ~281404 (29%)
  layers        :
    Middleware: 10 files, 26 symbols
    Controller: 2 files, 17 symbols
    Dto: 77 files, 873 symbols
    Database: 64 files, 422 symbols
    Route: 17 files, 249 symbols
    Config: 27 files, 71 symbols
    Test: 273 files, 2121 symbols
    Unknown: 262 files, 1068 symbols
```

## Incremental Update

After appending a benchmark comment to `fastapi/applications.py`, `sbe update` reparsed the changed file and preserved the rest of the graph:

```text
SBE update complete
  changed files : 1
  symbols       : 0 added, 3 modified, 0 removed
  affected      : 3 symbols
  affected files: 1
  graph         : 67320 edges
  elapsed       : 10815 ms
  storage       : C:\tmp\sbe-fastapi-benchmark\.sbe
```

The update path avoided a full reparse, but import-edge refresh remains a visible cost on large Python repositories. That is a concrete optimization target for the next graph/indexer pass.

## Result

The benchmark validates that SBE can index and query a large Python framework repository with the new Python plugin:

- 1120 Python files scanned.
- 6524 symbols extracted.
- 67320 graph edges built.
- Central framework impact queries returned thousands of affected symbols with bounded BFS depth.
- Focused context estimation reduced a broad FastAPI dependency/routing query by 29%.
- Incremental update detected one changed file and three modified symbols.
