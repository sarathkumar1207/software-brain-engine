# Stop Sending Entire Codebases to AI: Introducing Software Brain Engine

**Subtitle:** A local semantic graph engine that helps AI coding tools retrieve the right code, reduce token waste, and understand software dependencies faster.

Website: [https://sarathkumar1207.github.io/software-brain-engine/](https://sarathkumar1207.github.io/software-brain-engine/)  
GitHub: [https://github.com/sarathkumar1207/software-brain-engine](https://github.com/sarathkumar1207/software-brain-engine)
Downloads: [https://github.com/sarathkumar1207/software-brain-engine/releases/latest](https://github.com/sarathkumar1207/software-brain-engine/releases/latest)

## The Problem: AI Coding Tools Read Too Much Code

AI coding assistants are becoming part of everyday software development. They help write code, debug issues, explain unfamiliar systems, and speed up refactoring.

But there is one serious problem:

```text
Most AI coding workflows still treat the repository like a text dump.
```

When you ask an AI system to fix a bug or make a change, it often starts by searching broad folders, opening unrelated files, and trying to reconstruct architecture from raw source code.

That creates four problems:

1. **Token waste**: the model reads files that are not relevant.
2. **Slower debugging**: the assistant spends time discovering the codebase before solving the issue.
3. **Lower accuracy**: unrelated context can distract the model.
4. **Weak dependency awareness**: the assistant may miss downstream impact.

This is especially painful in backend systems where a single change can cross controllers, services, DTOs, middleware, database models, routes, and tests.

For example:

```text
"Change JWT authentication to Passport"
"Fix createUser validation"
"Refactor saveUser"
"Find what breaks if this middleware changes"
```

These are not simple text-search problems. They are dependency and impact-analysis problems.

That is why I am building **Software Brain Engine**, or **SBE**.

## What Is Software Brain Engine?

**Software Brain Engine** is an open-source Rust CLI that turns TypeScript, TSX, and Python repositories into a local semantic graph.

Instead of sending the entire repository to an LLM, SBE helps retrieve only the relevant code context.

In simple terms:

```text
SBE scans your codebase, builds a graph, and answers:
"What code matters for this change?"
```

Basic commands:

```bash
sbe scan ./repo
sbe update ./repo
sbe graph createUser --json
sbe impact createUser --json
sbe benchmark ./repo --query "jwt to passport"
```

SBE returns:

- matched symbols
- functions, classes, methods, interfaces, variables, and type aliases
- dependencies
- dependents
- impacted files
- source code ranges
- code layers
- approximate token savings
- JSON output for AI agents and tools

On a real Fastify checkout, Graph Intelligence v2 indexed 33 TypeScript declaration/type-test files into 506 symbols and 2700 edges. The `FastifyInstance` impact query reported 230 affected symbols across 24 files at depth 4, and the focused benchmark query reduced estimated context from ~84633 tokens to ~42335 tokens.

On a real FastAPI checkout, the Python plugin indexed 1120 Python files into 6524 symbols and 67320 edges. The `FastAPI` impact query reported 4481 affected symbols across 706 files at depth 6, and the focused routing/dependency benchmark reduced estimated context from ~978145 tokens to ~696741 tokens.

Context Compiler v2.1 is the next layer: it turns graph results into deterministic context packs with ranked symbols, dependency paths, callers, code ranges, summaries, token budgets, and reduction metrics. It prepares context for future AI tools without calling an AI model.

The goal is not to replace the developer.

The goal is to give AI systems better code context before they start reading source.

## The Core Idea: Graph First, Tokens Second

A normal AI coding workflow often looks like this:

```text
User asks for change
AI searches files
AI opens many files
AI guesses dependencies
AI edits code
```

SBE proposes a better workflow:

```text
User asks for change
SBE queries semantic graph
SBE returns impacted symbols and source ranges
AI reads focused context
AI edits code with better dependency awareness
```

This matters because context is one of the biggest bottlenecks in AI-assisted software engineering.

The future of AI coding is not just bigger context windows. It is better context retrieval.

## Why This Matters for Developers

Developers do not need another vague AI tool.

Developers need infrastructure that makes AI coding workflows more reliable.

SBE is designed for:

- AI code context retrieval
- semantic code search
- dependency graph analysis
- impact analysis
- token optimization
- codebase understanding
- AI-assisted debugging
- refactoring analysis
- TypeScript backend exploration
- local-first developer tooling

If you are working with large TypeScript backends, SBE is meant to help answer questions like:

```text
Which files are affected by this function?
What calls this service method?
Which controller depends on this middleware?
What code should an AI assistant read before changing this symbol?
How much context can we avoid sending?
```

## Demo: Hono Middleware Lifecycle Analysis

To explain the workflow, I added a realistic demo based on [`honojs/hono`](https://github.com/honojs/hono), a TypeScript web framework.

The simulated issue:

```text
Middleware post-processing after await next() should still mutate an error response created by onError.
```

Example:

```ts
app.onError((err, c) => c.text('failed', 500))

app.use(async (c, next) => {
  await next()
  c.header('x-request-processed', 'true')
})

app.get('/users/:id', () => {
  throw new Error('database unavailable')
})
```

This is the kind of issue that requires multi-file understanding:

- middleware composition
- request dispatch
- error handling
- context response mutation
- tests

Without SBE, a coding assistant might inspect:

```text
src/hono.ts
src/hono-base.ts
src/compose.ts
src/context.ts
src/request.ts
src/router.ts
src/types.ts
src/helper/factory/index.ts
src/compose.test.ts
src/hono-base.test.ts
docs/guides/middleware.md
```

Estimated broad context:

```text
12-16 files
~22k-30k tokens
```

With SBE:

```bash
sbe scan ./hono
sbe graph compose --json
sbe impact compose --json
```

SBE can produce a focused file/range packet:

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

Estimated reduction:

```text
~68-78%
```

Important note: this is a realistic simulated demo, not a claim that Hono currently has this bug. Before opening an upstream issue or PR, the behavior must be reproduced against a specific Hono commit with a failing test.

That honesty matters. SBE should prove where it helps, not exaggerate results.

## What SBE Does Today

SBE is currently a **production-alpha** developer tool.

Current capabilities:

- Rust CLI
- TypeScript, TSX, and Python scanning
- Tree-sitter syntax parsing
- symbol extraction
- import extraction
- best-effort reference edges
- local `.sbe/index.bin` binary storage
- debug JSON export
- graph queries
- impact analysis
- benchmark reports
- validation reports
- human-readable CLI output
- JSON output for agents and tools
- Windows MSI release workflow
- GitHub Actions CI

Example:

```bash
sbe scan ./repo
```

Output:

```text
SBE scan complete
  indexed files : 214
  graph         : 918 symbols, 301 imports, 1440 edges
  storage       : ./repo/.sbe
  elapsed       : 682 ms
```

## What SBE Skips

SBE is designed to index source code, not dependencies or generated output.

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

This matters because scanning `node_modules` would destroy the value of the tool. The purpose is to understand your project code, not installed packages.

A future `.sbeignore` file can make this configurable per project.

## What SBE Does Not Do Yet

SBE is not a magic AI brain.

Current limitations:

- TypeScript/TSX/Python source support
- syntax-based analysis, not full TypeScript type checking
- approximate token estimates
- no exact tokenizer yet
- no watch mode yet
- no editor extension yet
- no language-server replacement

The current token estimate is approximate:

```text
tokens ~= source characters / 4
```

That is enough for early full-context vs focused-context comparison, but exact tokenizer support is planned.

## Why Build This in Rust?

SBE is a local CLI and indexing engine.

Rust is a strong fit because it gives:

- fast filesystem scanning
- reliable native binaries
- strong modular architecture
- cross-platform release support
- good performance for graph/index workloads

The project is organized as a Cargo workspace:

```text
common   - shared types
scanner  - repository traversal
parser   - Tree-sitter TypeScript extraction
storage  - .sbe binary index
symbols  - symbol registry
graph    - typed dependency graph with forward/reverse indexes
impact   - reverse BFS traversal
update   - incremental graph refresh
query    - context and benchmark reports
indexer  - scan/parse/update/store pipeline
cli      - user-facing commands
```

This modular structure is important because SBE is meant to grow as infrastructure, not as a one-file script.

## Why Not Just Use grep or ripgrep?

`ripgrep` is excellent for text search.

SBE is trying to solve a different problem:

```text
graph-shaped code context
```

For AI coding workflows, the useful output is often not:

```text
Here are all files containing "createUser"
```

The useful output is:

```text
Here is createUser.
Here are its dependencies.
Here are its dependents.
Here are the exact code ranges.
Here are the impacted files.
Here is the minimal context packet.
```

That is the difference between search and code intelligence.

## The Marketing Claim I Am Willing to Make

I do not want to claim that SBE always reduces tokens.

It does not.

Small projects can show 0% savings because metadata overhead can be larger than the focused code slice.

The honest claim is:

```text
When a change maps to a focused dependency path, SBE can reduce unnecessary AI context by retrieving graph-relevant code before the model reads source.
```

That is the product.

Not hype.

Infrastructure.

## Who Should Try SBE?

SBE is useful for:

- backend engineers
- TypeScript teams
- AI coding assistant users
- maintainers of large repositories
- developer tooling engineers
- AI agent builders
- people working on code intelligence
- teams trying to reduce LLM token usage

If your AI assistant often opens too many files before solving the issue, SBE is built for that pain.

## Roadmap

Near-term roadmap:

- `.sbeignore`
- exact tokenizer support
- watch mode
- better TypeScript resolver
- graph diff and symbol versioning
- public benchmark corpus
- editor and agent integrations
- more language support

The long-term goal is simple:

```text
Make codebases queryable as semantic graphs for AI systems.
```

## Try It

Website:

[https://sarathkumar1207.github.io/software-brain-engine/](https://sarathkumar1207.github.io/software-brain-engine/)

GitHub:

[https://github.com/sarathkumar1207/software-brain-engine](https://github.com/sarathkumar1207/software-brain-engine)

Download:

[https://github.com/sarathkumar1207/software-brain-engine/releases/latest](https://github.com/sarathkumar1207/software-brain-engine/releases/latest)

Platform artifacts:

| Platform | Artifact |
| --- | --- |
| Windows x64 | `sbe-0.5.0-windows-x64.msi` |
| Linux x64 | `sbe-linux-x64.tar.gz` |
| macOS ARM64 | `sbe-macos-arm64.tar.gz` |

Install from source:

```bash
cargo install --path crates/cli --force
```

Run:

```bash
sbe scan ./your-typescript-project
sbe benchmark ./your-typescript-project --query "jwt to passport"
```

## Feedback Wanted

The most useful feedback is not:

```text
Looks cool.
```

The most useful feedback is:

```text
I ran SBE on this repo, and the graph was wrong here.
```

If you work on TypeScript backends, AI coding tools, code intelligence, LLM context retrieval, or developer infrastructure, I would love your feedback.

SBE is open source and MIT licensed.

Give it a try, open an issue, or tell me where the graph breaks.
