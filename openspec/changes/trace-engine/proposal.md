## Why

specjutsu's core promise is that a machine checks every spec scenario has a test.
Nothing in the repo does that yet. `sj trace` is the first piece, and the repo
must be able to trace itself before anything else ships (bootstrapping stage 1).

## What Changes

- Parse scenario IDs from `#### Scenario: <ID> <name>` headings under the spec root
  (`openspec/specs/**/spec.md` and `openspec/changes/*/specs/**/spec.md`, archive excluded).
- Scan Rust test sources for scenario IDs in test function names and in
  `// spec: <ID>` comments.
- New command `sj trace [--json] [--root <dir>]` that reports mapped scenarios,
  scenarios with no test (gaps) and tests that reference unknown IDs (orphans).
- Exit code is always 0 in v0.1 (report only); `--strict` exits 1 on gaps or orphans.

## Capabilities

### New Capabilities
- `trace`: scenario-ID extraction, test matching, and the trace report.

### Modified Capabilities
None.

## Impact

- New code in `crates/sj-core` (spec parser, matcher, report) and `crates/sj-cli` (`trace` command).
- Runner-report confirmation (JUnit XML etc.) is out of scope; it comes with `sj test`.
