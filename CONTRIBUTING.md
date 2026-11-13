# Contributing to specjutsu

Thanks for helping. specjutsu is built spec-first and test-first, and we hold
contributions to the same rule the tool enforces.

## Workflow

1. **Spec it.** Behaviour changes start as an OpenSpec change in
   `openspec/changes/<change>/`. Every scenario starts with a stable ID:
   `#### Scenario: TRACE-007 Short name`. One prefix per capability (TRACE,
   GATE, LEDGER, RUN, ADAPT). Never reuse an ID.
2. **Red.** Write the failing test, with the ID in its name
   (`fn trace_007_short_name()`), and commit it: `test(TRACE-007): red`.
3. **Green.** Implement, then commit: `feat(TRACE-007): green`.
4. Run `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings` and
   `cargo test --workspace` before opening a PR.

Refactors, docs, config and chores don't need scenarios; say which in the PR.

## Good first PRs

- A test-runner adapter (`runner-adapter` label).
- An agent adapter (`agent-adapter` label).

## Conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md).
