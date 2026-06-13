# Change Simulator v2.2

Change Simulator predicts the impact of a code change before source files are edited. It uses only SBE's stored semantic graph, reverse graph, impact traversal, and Context Compiler.

## Purpose

Use simulation before deleting, replacing, or modifying a shared symbol to answer:

- Which symbols and files are in the blast radius?
- How risky is the planned operation?
- Which application entry points or named flows depend on it?
- Which existing tests are connected to the affected graph?
- What focused source ranges are needed to review the change?

## Commands

```bash
sbe simulate modify createUser
sbe simulate delete createUser --max-depth 6
sbe simulate replace createUser --json
sbe simulate delete createUser --record
```

`--max-depth` bounds both forward and reverse iterative traversals. The default is `6`. `--record` writes `.sbe/reports/simulation-<operation>-<symbol>-latest.json`.

## Report

`SimulationReport` contains the operation, target symbol, 0-100 risk score, risk level and breakdown, reached traversal depth, affected symbol/file IDs, affected flow IDs, recommended test IDs, and a deterministic `ContextPack`.

Risk combines direct caller count, direct dependency count, cross-file graph edges, reached graph depth, critical flow participation, and operation severity. Delete receives a larger operation contribution than replace; modify has no operation surcharge. Every factor is configurable through `RiskConfig`.

Flow detection treats connected workflow-named symbols and exported graph entry points as affected flows. Test selection uses connected symbols whose names or file paths follow common `test`, `tests`, `spec`, `.test.`, `.spec.`, and `__tests__` conventions. These are deterministic graph recommendations, not executed test results.

## Complexity

Traversal is iterative BFS with cycle protection. Each bounded forward or reverse traversal is `O(V + E)` over the visited graph portion. Adjacency lookup uses the graph's forward and reverse hash indexes.

The benchmark target builds a deterministic 10,000-symbol graph and checks average simulation time:

```bash
cargo bench -p sbe-simulator --bench change_simulator
```

## Language Accuracy

Simulation quality depends on graph accuracy. SBE currently provides syntax-based TypeScript/TSX analysis and a syntax-based Python plugin. It does not yet provide full TypeScript compiler resolution, Python type checking, decorator execution, overload resolution, or runtime import evaluation. The simulator does not overstate those boundaries.

Useful search keywords: change impact analysis, dependency graph, blast radius, code change simulation, reverse dependencies, risk scoring, affected flows, recommended tests, TypeScript code intelligence, Python code intelligence, and deterministic context retrieval.
