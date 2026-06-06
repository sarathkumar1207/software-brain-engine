# Live Demo: Hono Middleware Lifecycle Analysis

This demo shows how Software Brain Engine can guide an AI assistant through a focused multi-file debugging task.

The issue below is a realistic simulated bug for demonstration. It is not a claim that the bug currently exists in Hono. Before opening an upstream issue or pull request, reproduce it against a specific Hono commit with a failing test.

Machine-readable demo data is available in [`examples/hono-middleware-demo.json`](../examples/hono-middleware-demo.json).

## Selected Repository

- Repository: [`honojs/hono`](https://github.com/honojs/hono)
- Project type: TypeScript web framework
- Subsystem: middleware composition, dispatch, context response handling

Hono is a good SBE demo target because middleware behavior crosses multiple files: composition, request dispatch, context mutation, factory helpers, and tests.

## Simulated Issue

Middleware post-processing after `await next()` should still apply headers when a downstream handler throws and `onError` creates the final response.

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

Expected:

```text
HTTP 500
x-request-processed: true
```

Why this is useful for a demo:

- It is not a single-line bug.
- It requires middleware lifecycle understanding.
- It crosses handler dispatch, error handling, context response mutation, and tests.
- It is exactly the kind of issue where an AI assistant can waste context reading broad framework files.

## Without SBE

A normal AI/code assistant usually starts with broad search:

```text
middleware
next
onError
Context
Response
dispatch
```

Likely files opened:

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

Estimated context:

```text
12-16 files
~22k-30k tokens
```

Main weakness:

```text
The assistant still has to manually infer:
compose -> next -> handler throws -> onError -> Context.res -> post-next middleware
```

## With SBE

Commands:

```bash
sbe scan ./hono
sbe graph compose --json
sbe impact compose --json
```

Example graph output:

```json
{
  "symbol": "compose",
  "file": "src/compose.ts",
  "range": "18-102",
  "dependencies": ["MiddlewareHandler", "Context", "Next", "dispatch"],
  "dependents": ["Hono.dispatch", "Hono.request", "createMiddleware"]
}
```

Example impact output:

```json
{
  "origin": "compose",
  "affected_symbols": [
    "compose",
    "dispatch",
    "Hono.dispatch",
    "Hono.request",
    "createMiddleware",
    "Context.res",
    "onError"
  ],
  "affected_files": [
    "src/compose.ts",
    "src/hono-base.ts",
    "src/context.ts",
    "src/helper/factory/index.ts",
    "src/types.ts",
    "src/compose.test.ts",
    "src/hono-base.test.ts"
  ]
}
```

Focused code-range packet:

```text
src/compose.ts:18-102
src/hono-base.ts:210-275
src/context.ts:120-175
src/types.ts:40-88
src/helper/factory/index.ts:12-44
src/compose.test.ts:1-90
src/hono-base.test.ts:300-370
```

Estimated context:

```text
5-7 files
~5k-7k tokens
```

## Efficiency Comparison

| Metric | Without SBE | With SBE |
| --- | ---: | ---: |
| Files read | 12-16 | 5-7 |
| Context size | ~22k-30k tokens | ~5k-7k tokens |
| Dependency awareness | Manual inference | Graph-guided |
| Accuracy | Medium | Higher |
| Debug time | Slower | Faster |

Estimated token reduction:

```text
~68-78%
```

Conservative claim:

```text
SBE can reduce context by roughly 70% on this kind of focused middleware lifecycle issue.
```

Do not claim:

```text
SBE always reduces tokens.
```

## Candidate Regression Test

This is the kind of failing test that must be validated against the actual upstream repository before opening a Hono PR:

```ts
it('runs middleware post-processing after onError response', async () => {
  const app = new Hono()

  app.onError((err, c) => c.text('failed', 500))

  app.use(async (c, next) => {
    await next()
    c.header('x-request-processed', 'true')
  })

  app.get('/users/:id', () => {
    throw new Error('database unavailable')
  })

  const res = await app.request('/users/1')

  expect(res.status).toBe(500)
  expect(res.headers.get('x-request-processed')).toBe('true')
})
```

Expected lifecycle:

```text
middleware before
  handler throws
  onError creates response
middleware after
  mutates final response
return response
```

## Upstream Contribution Path

Before raising an upstream issue or PR:

1. Clone `honojs/hono`.
2. Check out a specific commit.
3. Add the candidate regression test.
4. Run Hono's test suite.
5. Confirm whether the test fails.
6. If it fails, use SBE to capture the graph and focused ranges.
7. Open a Hono issue with the failing test and lifecycle explanation.
8. Open a PR only after the fix is verified locally.

This protects SBE credibility. The public claim should be:

```text
This is a realistic SBE debugging demo using Hono's middleware architecture.
```

Not:

```text
SBE found a confirmed bug in Hono.
```

## Video Script

### Hook

AI coding tools often read too much code before understanding a focused bug. In framework code, that means thousands of wasted tokens across router, request, context, middleware, and tests.

### Show Bug

A middleware calls `await next()` and then adds a header. The downstream handler throws, `onError` returns a 500 response, and we want the middleware post-processing to still mutate the final response.

### Normal Flow

Without SBE, the assistant searches broad terms and opens 12-16 files. It still has to manually infer the control flow.

### SBE Flow

Run:

```bash
sbe graph compose --json
sbe impact compose --json
```

SBE returns the dependency path and focused code ranges.

### Result

The AI receives 5-7 focused files instead of 12-16 broad files. Estimated context drops from ~22k-30k tokens to ~5k-7k tokens for this demo shape.

### Closing

SBE does not replace the developer or the type checker. It gives AI systems graph-aware code context before they read source.
