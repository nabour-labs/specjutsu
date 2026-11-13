## Context

`sj trace` (v0.1) gives scenarios and test links at one commit. The gate needs the
same data at two commits, real test results, and a base run with the change's
tests overlaid. Only the `cargo test` runner exists in this change.

## Decisions

- **Compare against the merge-base.** `base = git merge-base <base> <head>`, so
  commits landed on main after the branch point don't show up as the change's work.
- **Reading base specs without a checkout.** Base spec files are read with
  `git ls-tree` + `git show <base>:<path>`; head is the working tree. The same
  parser as `sj trace` runs on both.
- **Modified = normalised text differs.** A scenario's identity is its ID; its
  content is the heading name plus its step lines, whitespace-collapsed. Moving a
  scenario between files (delta → `specs/` on archive) is therefore unchanged.
- **Base checkout via a temporary worktree.** `git worktree add --detach <tmp> <base>`,
  copy head's versions of the change's added/modified test files over it, run,
  then `git worktree remove --force`. The head checkout is never modified.
  Base builds use a separate `CARGO_TARGET_DIR` (`target/sj-base`) so CI caches it.
- **What is a test file (cargo runner).** Files under a crate's `tests/`
  directory. Tests in `#[cfg(test)]` modules inside `src/` can't be overlaid on
  their own, so red-on-base reports them `unverifiable` (GATE-011) and fails by
  default. Consequence for this repo: new tests go in `crates/*/tests/`; the
  TRACE unit tests move there in this change.
- **Running tests and reading results.** `cargo test --workspace --no-fail-fast`
  and parse libtest's stable text lines (`test <path> ... ok|FAILED|ignored`).
  `--format json` is nightly-only, so it is not used. A libtest path matches a
  traced test when its last `::` segment equals the fn name and the binary
  belongs to the crate that holds the file. On base, a non-zero exit with no
  line for the test means it didn't compile → `red:compile`.
- **Only scenario tests run twice.** Head runs the full suite (cheap and needed
  for refactors); base runs only the overlaid tests (`cargo test --test <file>`).
- **Implementation files.** `*.rs` outside `tests/`, `fixtures/` and `target/`.
  Everything else (Markdown, YAML, TOML, CI) is non-source for GATE-013/014. This
  list moves to `specjutsu.toml` with the runner config in a later change.
- **Exemptions.** `Exempt: <kind> — <reason>` (an ASCII `-` also accepted) in a
  change's `proposal.md`, or a `Specjutsu-Exempt:` commit trailer when there is no
  change folder. Parsed from head; trailers from `git log <base>..<head>`.
- **History check** parses commit subjects with
  `^(test|feat)\(([A-Z]+-\d+(?:\.\.\d+)?(?:,\s*[A-Z]+-\d+(?:\.\.\d+)?)*)\)`, expanding
  `TRACE-001..006` to TRACE-001…TRACE-006 (zero-padding kept).

## Risks / Trade-offs

- **Compile-red is weaker evidence than assert-red.** A test that doesn't compile
  on base proves the API is new, not that the behaviour was absent. Recorded
  distinctly so reviewers see it; a future `--require-assert-red` can tighten it.
- **Base build time.** A full base compile per PR. Mitigated by caching
  `target/sj-base` and running only the overlaid test binaries.
- **Text parsing of libtest output** could break if libtest changes format; it
  has been stable for years, and runner-report parsing (JUnit) arrives with more runners.

## Open Questions

- Should docs-only changes need an explicit `Exempt: docs`? This change says no
  (less ceremony); the kickoff doc implies yes.
- Should `unverifiable` fail by default or warn? This change says fail, to keep
  the moat honest; `--allow-unverifiable` exists for migration.
