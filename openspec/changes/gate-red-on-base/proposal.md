## Why

`sj trace` shows whether a scenario has a test; it can't show the test drove the
change, and nothing blocks a merge. `sj gate` is the CI check that turns trace
into enforcement and adds the moat: **red-on-base**, proof that each new test
fails on the code before the change and passes after it (bootstrapping stage 2).

## What Changes

- New command `sj gate --base <ref> [--head <ref>] [--json]` for CI.
- Classify scenarios in the change against the base: added, modified, removed, unchanged.
- Fail the gate when an added/modified scenario has no test, a mapped test fails,
  a new/changed test already passes on base, a test is mapped to a missing
  scenario, or implementation changed with no scenario and no exemption.
- Red-on-base: run the tests for added/modified scenarios on a base checkout with
  only the change's test files overlaid; they must fail, then pass on head.
- Exemptions (`refactor`, `docs`, `config`, `chore`, `spike`), each with a reason,
  shown in the summary.
- History check `sj gate --since <commit>`: every ID with a `feat(ID)` commit has an
  earlier `test(ID): red` commit.
- First runner: `cargo test`, for separate test files (`tests/*.rs`).

## Capabilities

### New Capabilities
- `gate`: change classification, the gate rules, red-on-base, exemptions, history check.

### Modified Capabilities
None (`trace` is reused unchanged).

## Impact

- New code in `sj-core` (diffing, gate, exemptions), `sj-runners` (cargo test
  runner with per-test results), `sj-cli` (`gate` command).
- Needs `git` on PATH and a base ref fetched (`actions/checkout` with `fetch-depth: 0`).
- Out of scope: runners other than cargo, the local ledger, monorepo package scoping.
