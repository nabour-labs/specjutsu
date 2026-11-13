# specjutsu — Design Analysis

Nov 13, 2026 · review of `specjutsu — Project Kickoff & Design.md`

## Verdict

The design is coherent and the wedge is real: no tool in the space checks that a
spec scenario is backed by a test, and none proves the test failed first. The
risk is scope. The doc commits to 8 test runners, 5 agent adapters, 2 importers,
a token benchmark and a website before v0.5. The plan below keeps the moat
(trace + red-on-base) on the critical path and treats everything else as
breadth that can follow.

## Strengths

- **Clear, defensible wedge.** "Keep your spec tool, specjutsu proves the code
  matches it" is a positioning that complements OpenSpec / Spec Kit instead of
  fighting them.
- **Deterministic-first is the right architecture.** It makes the tool testable,
  CI-reproducible and cheap in tokens at the same time.
- **Format reuse.** OpenSpec format + one ID convention means zero migration for
  OpenSpec users and lets the project bootstrap on itself cleanly.
- **Self-hosting rule.** Gating the repo with the last *released* binary avoids
  a PR approving itself. Good call.

## Gaps and risks to resolve early

| # | Issue | Why it matters | Recommendation |
|---|---|---|---|
| 1 | Red-on-base needs to build the base commit with the PR's tests overlaid | New tests often reference symbols that don't exist on base, so they fail to **compile**, not fail an assertion. Is a compile error "red"? | Define it: compile failure of the overlaid test counts as red, but record it distinctly (`red:compile` vs `red:assert`). Spec this as GATE scenarios before coding. |
| 2 | Overlay of "test files" | In Rust, Go, and many JS setups, unit tests live inside implementation files (`#[cfg(test)] mod tests`). Overlaying "test files only" isn't possible. | v0.2: support separate test files only (`tests/`, `*_test.go`, `*.test.ts`); document the limitation; inline tests rely on ledger evidence. |
| 3 | Trace matching precision | `AUTH-003` in a name vs `test_AUTH_003_…` vs `trace_001_…` (lowercase, no prefix-casing). The convention in the bootstrapping section (`trace_001_parses…`) differs from the general one. | Normalize: an ID `PREFIX-NNN` matches case-insensitively with `-` or `_` as separator. Captured in TRACE-002. |
| 4 | Ledger trust | A local ledger file is forgeable; CI can't trust it. | CI recomputes red/green itself (red-on-base); local ledger is advisory evidence for `strict` mode only. |
| 5 | "Scenario added or modified in the change" | Requires diffing spec deltas against base — needs a precise definition for MODIFIED/REMOVED blocks. | Treat scenario IDs as the unit: added = ID not on base; modified = same ID, different body hash. |
| 6 | Scope of v1 runners | 8 runners × report formats is a lot. | Lead with `cargo test` (self-hosting) then JUnit XML as a generic path — it covers pytest, Jest, Gradle, Maven, PHPUnit, dotnet via loggers. |
| 7 | Token benchmark | Claims ≥50% savings with no harness yet. | Keep "measured by `gain`" off the site until the benchmark exists (doc already says this — hold to it). |
| 8 | Open questions | org/domain/license/codex-hook. | Org decided: `nabour-labs/specjutsu`. License: MIT (as recommended). Others stay open. |

## Critical path

1. **v0.1 — trace.** Parse `openspec/` specs → scenario IDs; scan Rust test
   names → IDs; report mapped / unmapped / orphan. Run on itself in CI,
   non-blocking.
2. **v0.2 — gate.** Spec deltas vs base, red-on-base for separate test files,
   gate exit codes.
3. **v0.3 — agent loop.** `status`, `next`, `test` with failures-only output,
   Claude Code adapter. Switch the repo from OpenSpec authoring to specjutsu.
4. **v0.4–0.5 — breadth.** More runners, other 4 adapters, importers,
   benchmark, site, public launch.

## What was started today

- Repo `nabour-labs/specjutsu` (public), MIT.
- Cargo workspace: `sj-cli`, `sj-core`, `sj-runners`, `sj-adapters`.
- OpenSpec initialised; first change `openspec/changes/trace-engine/` with
  TRACE-001…TRACE-005.
- `sj trace` implemented test-first against those scenarios.
- CI: fmt, clippy, tests on macOS/Linux/Windows, `sj trace` report on itself.
