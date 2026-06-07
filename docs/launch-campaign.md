# SBE Launch Campaign

This is the practical campaign plan for getting Software Brain Engine in front of developers.

The positioning should stay technical:

```text
Stop sending entire codebases to AI. SBE turns repositories into semantic graphs for precise code context retrieval.
```

Do not lead with vague AI claims. Lead with the problem developers recognize: context waste, dependency guessing, and slow code understanding.

## Launch Goals

Primary goals:

- GitHub stars from developers who use AI coding tools.
- Feedback from TypeScript/Rust/backend engineers.
- Real projects to test SBE against.
- Issues for resolver accuracy, tokenizer support, `.sbeignore`, and watch mode.

What to ask for:

```text
Try it on a TypeScript repo and tell me where the graph is wrong.
```

## Launch Assets

Use these URLs:

```text
GitHub: https://github.com/sarathkumar1207/software-brain-engine
Website: https://sarathkumar1207.github.io/software-brain-engine/
Demo doc: https://github.com/sarathkumar1207/software-brain-engine/blob/main/docs/demo-hono.md
```

Core proof:

```text
Hono middleware demo:
Without SBE: 12-16 files, ~22k-30k tokens
With SBE: 5-7 files, ~5k-7k tokens
Estimated reduction: ~68-78%
```

Use the proof carefully:

```text
This is a realistic simulated demo, not an upstream Hono bug claim.
```

## 7-Day Plan

### Day 1: GitHub And Website

Checklist:

- Push latest README, website, logo, and demo docs.
- Add repository topics:
  - `rust`
  - `typescript`
  - `cli`
  - `developer-tools`
  - `ai-tools`
  - `llm`
  - `code-intelligence`
  - `semantic-graph`
  - `token-optimization`
- Pin the Hono demo issue/doc in README.
- Make sure Releases have downloadable artifacts.

Post only after the GitHub page looks complete.

### Day 2: Hacker News

Title:

```text
Show HN: SBE, a Rust CLI that builds semantic graphs for AI code context
```

Post:

```text
I built Software Brain Engine, an open-source Rust CLI for TypeScript/TSX repositories.

The problem: AI coding tools often read too much code before they understand a focused change. They search broad folders, open unrelated files, and burn context reconstructing dependencies.

SBE scans a repo locally, stores a .sbe index, extracts symbols/imports/references, and lets you query the semantic graph:

  sbe scan ./repo
  sbe graph createUser --json
  sbe impact createUser --json

The output is meant for both humans and coding agents: impacted files, dependencies, dependents, code ranges, and context/token estimates.

I also added a Hono middleware demo showing the workflow:
Without SBE: 12-16 files, ~22k-30k tokens
With SBE: 5-7 files, ~5k-7k tokens

This is production-alpha. It is syntax-based Tree-sitter analysis, not a TypeScript type checker. I am looking for feedback on graph accuracy, benchmark methodology, and whether this helps real AI coding workflows.

GitHub: https://github.com/sarathkumar1207/software-brain-engine
Website: https://sarathkumar1207.github.io/software-brain-engine/
```

First comment:

```text
Limitations:

- TypeScript/TSX only for now
- syntax-based, not type-aware yet
- token estimates are approximate
- small projects can show 0% savings because metadata overhead can be larger than the focused context

I want the benchmark to show misses honestly. The goal is not magic compression; it is graph-guided context selection before an LLM reads code.
```

### Day 3: daily.dev Article

Publish the article draft in [`docs/articles/daily-dev-sbe-launch.md`](articles/daily-dev-sbe-launch.md).

Title:

```text
Stop sending entire codebases to AI: building a semantic graph CLI in Rust
```

CTA:

```text
Try SBE on a TypeScript repo and share where the graph is wrong.
```

### Day 4: X / Twitter Thread

```text
1/ AI coding tools often waste context before they understand the actual change.

They search broad folders, open unrelated files, and reconstruct dependencies manually.

I built SBE to attack that problem.

2/ Software Brain Engine is an open-source Rust CLI that turns TypeScript repos into semantic graphs.

Run:
  sbe scan ./repo
  sbe graph createUser --json
  sbe impact createUser --json

3/ The goal is not to replace a type checker.

The goal is to retrieve the code that matters before an LLM reads source:
- symbols
- dependencies
- dependents
- files
- source ranges
- impact paths

4/ Demo shape:

Hono middleware lifecycle issue

Without SBE:
12-16 files, ~22k-30k tokens

With SBE:
5-7 files, ~5k-7k tokens

5/ Current limits:
- TypeScript/TSX only
- syntax-based analysis
- approximate token estimate
- production-alpha

Feedback wanted from backend engineers and AI coding tool users.

GitHub:
https://github.com/sarathkumar1207/software-brain-engine
```

### Day 5: Reddit / Dev Communities

Use only communities that allow project feedback posts.

Title:

```text
Feedback wanted: Rust CLI that builds semantic graphs for AI code context
```

Post:

```text
I am building Software Brain Engine, an open-source Rust CLI for TypeScript/TSX repositories.

The problem I am trying to solve:
AI coding tools often read too much code before understanding a focused change. They load broad folders and unrelated files, then manually infer dependencies.

SBE scans a repo locally and builds a semantic graph:
- functions/classes/methods/interfaces
- imports and best-effort references
- file and symbol relationships
- source ranges
- impact paths

Example:

  sbe scan ./repo
  sbe graph createUser --json
  sbe impact createUser --json

I added a Hono middleware demo to show the intended workflow. It is a realistic simulated case, not an upstream bug claim.

The rough result:
- normal workflow: 12-16 files, ~22k-30k tokens
- SBE workflow: 5-7 files, ~5k-7k tokens

Current limitations:
- TypeScript/TSX only
- syntax-based, not type-aware yet
- token estimate is approximate

I am looking for feedback on:
- whether this would help your AI coding workflow
- graph accuracy
- benchmark methodology
- what output an AI agent should receive

GitHub:
https://github.com/sarathkumar1207/software-brain-engine
```

### Day 6: LinkedIn

```text
I am building Software Brain Engine, an open-source Rust CLI for AI-assisted software development.

The problem: AI coding tools often read too much code before understanding the actual change.

SBE scans a TypeScript repository locally and builds a semantic graph. Instead of sending broad folders to an LLM, you can ask:

  sbe graph createUser --json
  sbe impact createUser --json

It returns relevant symbols, files, dependencies, dependents, and source ranges.

The goal is simple:
give AI systems the code that matters, not the entire repository.

Current alpha:
- Rust CLI
- TypeScript/TSX first
- local .sbe index
- impact analysis
- graph queries
- benchmark/demo docs

Looking for feedback from backend engineers, TypeScript teams, and developers using AI coding assistants.

GitHub:
https://github.com/sarathkumar1207/software-brain-engine
```

### Day 7: Follow-up Post

Do not repeat the launch post. Share what feedback you got.

```text
One useful question from the SBE launch:

"Does it skip node_modules?"

Yes. SBE skips node_modules, .git, .sbe, dist, build, target, .next, and coverage by default.

Next improvement I am considering: .sbeignore, so projects can add custom ignore rules like generated folders or vendor code.

Would you prefer SBE to read .gitignore too, or keep a separate .sbeignore?
```

## Reply Templates

### "Does it use TypeScript types?"

```text
Not yet. V1 is syntax-based through Tree-sitter. That keeps it fast and simple, but it means the graph is best-effort. Type-aware resolution is a future milestone.
```

### "Does it scan node_modules?"

```text
No. SBE skips node_modules and common generated/build folders by default: .git, .sbe, dist, build, target, .next, coverage, etc.
```

### "Why not just use ripgrep?"

```text
ripgrep is great for text search. SBE is trying to provide graph-shaped context: symbols, dependencies, dependents, source ranges, and impact paths that an AI agent can consume directly.
```

### "Is the token estimate exact?"

```text
No. Current token estimates are approximate. The benchmark is meant to compare full-project context vs focused graph context. Exact tokenizer support is on the roadmap.
```

### "Can I use this with Java/Python/Go?"

```text
Not yet. V1 focuses on TypeScript/TSX. The workspace is modular so more parsers can be added later.
```

## Success Metrics

Track:

- GitHub stars
- issues opened
- comments asking technical questions
- people testing on real repos
- forks
- release downloads
- website visits if analytics are added

Good early result:

```text
10 useful technical comments > 100 generic likes
```

## What To Avoid

Avoid:

- "AI brain" hype without technical proof
- claiming exact token savings
- claiming confirmed upstream bugs without reproduction
- adding npm badges before npm publication
- pretending SBE replaces language servers

Use:

```text
SBE is graph infrastructure for focused AI code context.
```
