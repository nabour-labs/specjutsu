## Context

First real code in the repo. Must stay dependency-light and deterministic.

## Decisions

- **Line-based heading parse, not a full Markdown AST.** `#### Scenario:` headings
  are unambiguous; a regex keeps line numbers exact. `pulldown-cmark` can come
  later if fenced-code false positives show up (we already skip fenced blocks).
- **ID normalisation.** IDs are uppercase `PREFIX-NNN`. Test names match by
  lowercasing the ID and replacing `-` with `_`, bounded by non-alphanumerics, so
  `trace_001_x` matches TRACE-001 but `trace_0012` does not.
- **Rust test detection.** A test is a `fn name(` preceded (attributes, comments,
  blank lines allowed) by `#[test]`, `#[tokio::test]` or any `#[...test...]`
  attribute. Comment links (`// spec: ID`) attach to the next test fn.
- **Which files count as test files.** Any `*.rs` file under the repo, walked with
  `ignore` (respects .gitignore), skipping `target/` and `fixtures/` unless they
  are the root.
- **Exit codes.** 0 by default (stage 1 is report-only); `--strict` → 1 on gaps/orphans.

## Risks

- Text matching can't confirm a test actually runs; runner-report confirmation
  lands with `sj test`.
