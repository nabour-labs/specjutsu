## 1. Groundwork

- [x] 1.1 Move TRACE unit tests from `src/` modules to `crates/sj-core/tests/` so they are overlay-able (refactor, no scenario)
- [x] 1.2 Git helpers: resolve ref + merge-base, list changed files, read a file at a commit, temp worktree add/remove (GATE-020)

## 2. Classification

- [x] 2.1 Load base scenarios from git objects; classify added/modified/removed/unchanged (GATE-001, GATE-002)
- [x] 2.2 Trace base tests from git objects; classify newly tested scenarios as implemented (GATE-023)

## 3. Cargo runner (sj-runners)

- [x] 3.1 Run `cargo test`, parse libtest lines into per-test results, detect not-run tests (GATE-005, GATE-006)
- [x] 3.2 Distinguish compile failure from assertion failure (GATE-008, GATE-009)

## 4. Gate rules (sj-core)

- [x] 4.1 Untested added/modified scenarios fail; unchanged gaps warn (GATE-003, GATE-004)
- [x] 4.2 Red-on-base with test-file overlay in a temp worktree (GATE-007, GATE-010)
- [x] 4.3 Inline tests reported unverifiable; `--allow-unverifiable` (GATE-011)
- [x] 4.4 Orphans fail (GATE-012)
- [x] 4.5 Unspecified implementation changes fail; docs-only passes (GATE-013, GATE-014)
- [x] 4.6 Exemptions: parse, require reason, refactor rules, spike blocked from main (GATE-015, GATE-016, GATE-017)

## 5. CLI and history

- [x] 5.1 `sj gate` compact output, `--json`, exit codes 0/1/2 (GATE-018, GATE-019, GATE-020)
- [x] 5.2 `sj gate --since <commit>` red-before-green history check (GATE-021, GATE-022)

## 6. Self-hosting

- [x] 6.1 Fixture git repos built at test time (`crates/sj-core/tests/support`), covering each rule
- [x] 6.2 Add a non-blocking `sj gate` job to CI (blocking after v0.2 release)
