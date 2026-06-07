# Marketing And Launch Playbook

This document gives SBE a practical open-source launch plan and ready-to-post copy.

For the current campaign sequence, use [`docs/launch-campaign.md`](launch-campaign.md). For long-form article drafts, use [`docs/articles/medium-sbe-launch.md`](articles/medium-sbe-launch.md) and [`docs/articles/daily-dev-sbe-launch.md`](articles/daily-dev-sbe-launch.md).

## Positioning

Software Brain Engine is a local Rust CLI that helps AI coding tools avoid reading unnecessary code. It indexes TypeScript/TSX repositories, finds impacted symbols/files/layers for a planned change, and reports approximate token savings.

Short version:

```text
SBE is a local code-intelligence CLI that gives LLMs focused change context instead of the full repo.
```

One-line pitch:

```text
I built an open-source Rust CLI that scans TypeScript repos and shows what code an LLM actually needs for a change, with token-savings benchmarks.
```

Problem:

```text
AI coding tools often burn tokens exploring broad folders before they understand a targeted change.
```

Solution:

```text
SBE builds a local .sbe index, maps symbols/imports/references, classifies impacted layers, and returns a benchmarkable context packet.
```

Best proof:

```text
sbe benchmark C:\repo --query "jwt to passport"
```

Download page:

```text
https://github.com/sarathkumar1207/software-brain-engine/releases/latest
```

## Launch Checklist

Before posting widely:

1. GitHub README has install, benchmark, validation, status, and roadmap.
2. GitHub Releases has a downloadable Windows MSI and platform archives.
3. Website is live through GitHub Pages.
4. A public benchmark example is available with command output.
5. Issues are open for roadmap items such as watch mode, exact tokenizer, and resolver accuracy.
6. Repository topics are set:
   - rust
   - cli
   - typescript
   - ai
   - llm
   - code-intelligence
   - developer-tools
   - token-optimization

## Hacker News

HN usually responds better to honest technical framing than polished marketing.

Title:

```text
Show HN: SBE, a Rust CLI that indexes TypeScript repos for LLM context
```

Post text:

```text
I built Software Brain Engine, an open-source Rust CLI for TypeScript/TSX repositories.

The idea is to reduce wasted LLM context. Instead of sending broad folders or the whole repo, SBE scans locally, stores a binary .sbe index, extracts symbols/imports/references, and answers questions like:

  sbe benchmark ./repo --query "jwt to passport"

It reports matched symbols, impacted files, impacted layers such as Auth/Middleware/Controller/DTO/Database, query time, and approximate full-context vs focused-context tokens.

This is production-alpha, not a full TypeScript type checker. Current analysis is syntax-based through Tree-sitter. I am sharing it early because I want feedback on the benchmark methodology, resolver accuracy, and whether this is useful for AI coding workflows.

Repo: <GitHub URL>
Website: <Website URL>
```

First comment:

```text
Important limitation: the token estimator is currently approximate, based on source characters. It is meant to compare full-project context vs focused SBE context, not to be model-tokenizer exact yet.

Also, small projects can show 0% savings because metadata overhead is larger than the focused code slice. That is intentional: the benchmark should show misses honestly.
```

## daily.dev

daily.dev works well with a practical developer-tool angle.

Title:

```text
I built SBE: a local Rust CLI to reduce LLM code-context tokens
```

Post:

```text
AI coding tools are useful, but they often inspect too much code before understanding a focused change.

Software Brain Engine is an open-source Rust CLI that indexes TypeScript/TSX repos locally and answers change-impact questions:

  sbe benchmark ./repo --query "jwt to passport"

It returns:
- matched symbols
- impacted files
- impacted layers
- dependency/dependent context
- approximate full-repo vs focused-context tokens

The goal is not magic compression. The goal is benchmarkable context selection before an LLM call.

It is production-alpha: local binary index, CLI UX, validation reports, Windows MSI workflow, and tests. Type-aware resolution and exact tokenizer support are next.

GitHub: <GitHub URL>
```

## LinkedIn

Use a clearer business/developer productivity framing.

```text
I am building Software Brain Engine, an open-source Rust CLI for AI-assisted software development.

The problem: LLM coding workflows often waste tokens by reading too much of the codebase before understanding a specific change.

SBE indexes a TypeScript repository locally and answers questions like:

  "What changes if we migrate JWT auth to Passport?"

It returns impacted symbols, files, dependency paths, code layers, and an approximate token-savings benchmark.

The goal is simple: give AI tools the code that matters, not the entire repository.

Current alpha includes:
- TypeScript/TSX scanning
- local .sbe binary index
- impact and graph queries
- benchmark and validation reports
- Windows MSI release workflow

I am looking for feedback from developers using AI coding tools on real TypeScript backends.

GitHub: <GitHub URL>
```

## X / Twitter

Thread:

```text
1/ I am building SBE: an open-source Rust CLI that helps LLM coding tools avoid reading unnecessary code.

2/ Run:
   sbe benchmark ./repo --query "jwt to passport"

SBE scans TypeScript/TSX locally and reports impacted symbols, files, layers, dependencies, and token estimates.

3/ Why? AI coding tools often burn context exploring broad folders. SBE creates a local .sbe index so agents can ask for focused code context first.

4/ It is production-alpha: binary storage, CLI UX, validation reports, Windows MSI workflow, tests, and docs.

5/ It is honest about limits: syntax-based analysis for now, approximate token estimate, exact tokenizer and richer resolver coming next.

GitHub: <GitHub URL>
```

## Reddit / Dev Communities

Use only communities that allow project sharing. Keep the post technical and ask for feedback.

Title:

```text
Feedback wanted: Rust CLI for reducing LLM context in TypeScript repos
```

Post:

```text
I am working on an open-source tool called Software Brain Engine.

It scans a TypeScript/TSX repo locally, builds a .sbe index, and answers planned-change queries such as "jwt to passport". The output includes impacted files, symbols, layers, dependencies, and approximate full-repo vs focused-context tokens.

I am not trying to claim perfect static analysis. V1 is syntax-based with Tree-sitter. The goal is to validate whether a local impact index can reduce LLM context before sending code to an AI coding tool.

I would appreciate feedback on:
- benchmark methodology
- TypeScript resolver accuracy
- CLI UX
- whether this would help your AI coding workflow

Repo: <GitHub URL>
```

## Product Hunt / Launch Sites

Wait until the release artifacts and website are stable. Product Hunt-style launches need a polished demo and screenshots.

Suggested tagline:

```text
Local code intelligence for smaller, smarter LLM context.
```

Description:

```text
Software Brain Engine indexes TypeScript repositories locally and returns focused impact reports for planned code changes, helping AI coding tools reduce unnecessary token usage.
```

## GitHub README Badges

Add badges only after workflows are stable:

```md
[![Rust](https://github.com/<owner>/<repo>/actions/workflows/rust.yml/badge.svg)](https://github.com/<owner>/<repo>/actions/workflows/rust.yml)
[![Release](https://github.com/<owner>/<repo>/actions/workflows/release.yml/badge.svg)](https://github.com/<owner>/<repo>/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
```

## What Not To Claim

Avoid these claims until the project proves them:

- "SBE always reduces tokens"
- "SBE understands TypeScript types"
- "SBE replaces the TypeScript language server"
- "SBE is production-stable"
- "SBE gives exact token counts"

Use this instead:

```text
SBE is a production-alpha tool for benchmarkable, local-first context selection.
```
