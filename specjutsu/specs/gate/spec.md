# gate Specification

## Purpose
Block a change unless every scenario it adds or modifies has a test that fails on the base code and passes at head, and every other behaviour change carries a scenario or an explicit, reasoned exemption.

## Requirements

### Requirement: Scenarios are classified against the base
The system SHALL compare the scenarios at head with those at the base ref by ID:
an ID absent on base is **added**; an ID present on both whose heading or step
lines differ (ignoring whitespace) is **modified**; an ID present only on base is
**removed**; otherwise **unchanged**.

#### Scenario: GATE-001 Classify added, modified, removed and unchanged scenarios
- **WHEN** base has AUTH-001, AUTH-002 and AUTH-003, and head has AUTH-001 unchanged, AUTH-002 with a reworded THEN line, no AUTH-003, and a new AUTH-004
- **THEN** the gate classifies AUTH-004 as added, AUTH-002 as modified, AUTH-003 as removed and AUTH-001 as unchanged

#### Scenario: GATE-002 Moving a scenario between spec files is not a change
- **WHEN** AUTH-001 moves from a change's delta spec into `specs/` with identical text
- **THEN** it is classified as unchanged

### Requirement: Implementing an existing scenario puts it in scope
A scenario that is unchanged in text but had no linked test on base and has one
at head SHALL be classified as **implemented** and treated like an added
scenario by every gate rule, so a change that implements a spec merged earlier
must prove it. The one exception is a test that passes on base, which is
`green:linked` rather than `green-on-base`. When an implemented scenario has several tests and
at least one fails on base, that failure proves the scenario, and its tests that pass on base
are accepted.

#### Scenario: GATE-023 Implementing a previously untested scenario is gated
- **WHEN** base already has scenario MATH-001 with no test, and the change adds a test for MATH-001 and edits `src/lib.rs`
- **THEN** MATH-001 is classified as implemented, the change is not reported as an unspecified change, and MATH-001 must pass red-on-base

### Requirement: Every added or modified scenario has a mapped test
The gate SHALL fail when an added or modified scenario has no linked test at head.

#### Scenario: GATE-003 Added scenario without a test fails the gate
- **WHEN** the change adds AUTH-004 and no test references AUTH-004
- **THEN** the gate fails with reason `untested AUTH-004`

#### Scenario: GATE-004 Unchanged scenarios without tests do not fail the gate
- **WHEN** AUTH-001 is unchanged, has no test, and is listed by `sj trace` as a gap
- **THEN** the gate does not fail because of AUTH-001 and lists it under warnings

### Requirement: Mapped tests pass at head
The gate SHALL run every test linked to an added or modified scenario at head and
SHALL fail when any of them fails or does not run.

#### Scenario: GATE-005 A failing mapped test fails the gate
- **WHEN** the test for AUTH-004 fails at head
- **THEN** the gate fails with reason `failing AUTH-004 <test name>`

#### Scenario: GATE-006 A mapped test that does not run fails the gate
- **WHEN** a test referencing AUTH-004 is found in source but the runner does not report it (e.g. ignored or not compiled)
- **THEN** the gate fails with reason `not-run AUTH-004 <test name>`

### Requirement: New tests fail on the base code (red-on-base)
For each added or modified scenario, the gate SHALL check out the base commit,
overlay only the change's added and modified test files, and run that
scenario's tests. Each test MUST fail there. A test that does not compile or is
not found on base counts as failed and is recorded as `red:compile`; an
assertion failure is recorded as `red:assert`.

#### Scenario: GATE-007 A new test that already passes on base fails the gate
- **WHEN** the test for AUTH-004 passes when run against the base code
- **THEN** the gate fails with reason `green-on-base AUTH-004 <test name>`

#### Scenario: GATE-008 Assertion failure on base is accepted as red
- **WHEN** the test for AUTH-004 compiles on base and fails an assertion, then passes on head
- **THEN** that scenario passes red-on-base, recorded as `red:assert`

#### Scenario: GATE-009 Compile failure on base is accepted as red, recorded distinctly
- **WHEN** the test for AUTH-004 calls a function that does not exist on base, so it fails to compile there
- **THEN** that scenario passes red-on-base, recorded as `red:compile`

#### Scenario: GATE-010 Only the change's test files are overlaid on base
- **WHEN** the change edits `src/auth.rs` and adds `tests/auth.rs`
- **THEN** the base run uses base's `src/auth.rs` and the head's `tests/auth.rs`, and the working tree of the head checkout is left untouched

#### Scenario: GATE-011 Tests inline in implementation files are reported as unverifiable
- **WHEN** the only test for AUTH-004 lives in a `#[cfg(test)]` module inside `src/auth.rs`
- **THEN** red-on-base for AUTH-004 is reported as `unverifiable` with a hint to move the test to `tests/`, and the gate fails unless run with `--allow-unverifiable`

### Requirement: Tests mapped to missing scenarios fail the gate
The gate SHALL fail when a test references a scenario ID that no scenario at head defines.

#### Scenario: GATE-012 Test referencing a removed scenario fails the gate
- **WHEN** AUTH-003 is removed and a test still references AUTH-003
- **THEN** the gate fails with reason `orphan AUTH-003 <test name>`

### Requirement: Behaviour changes need a scenario or an exemption
The gate SHALL fail when the change modifies implementation files but adds or modifies no scenario and declares no exemption. Every changed file is an implementation file unless it is a test file, a file under the spec root, documentation or media (`*.md`, `*.txt`, `*.rst`, images), repository metadata (`.github/**`, `.gitignore`, `.gitattributes`, `LICENSE*`), or a package manifest or lockfile. Changes touching only those SHALL pass without an exemption.

#### Scenario: GATE-013 Unspecified implementation change fails the gate
- **WHEN** the change edits `src/auth.rs` and adds or modifies no scenario and declares no exemption
- **THEN** the gate fails with reason `unspecified change` listing the changed source files

#### Scenario: GATE-014 Documentation-only change passes without an exemption
- **WHEN** the change edits only `README.md` and `docs/guide.md`
- **THEN** the gate passes

#### Scenario: GATE-027 A file type the gate does not recognise counts as implementation
- **WHEN** the change edits `web/app.ts` and `scripts/build.sh`, adds or modifies no scenario and declares no exemption
- **THEN** the gate fails with reason `unspecified change` listing `scripts/build.sh` and `web/app.ts`

#### Scenario: GATE-028 Manifest, lockfile and CI changes pass without an exemption
- **WHEN** the change edits only `Cargo.toml`, `Cargo.lock` and `.github/workflows/ci.yml`
- **THEN** the gate passes

### Requirement: Exemptions are explicit, reasoned and reported
A change SHALL declare an exemption with a line `Exempt: <kind> — <reason>` in
its change folder's `proposal.md`, or for changes without a change folder, a
commit trailer `Specjutsu-Exempt: <kind> — <reason>` on any commit in the range.
Kinds are `refactor`, `docs`, `config`, `chore` and `spike`. Every exemption
SHALL appear in the gate summary with its reason.

#### Scenario: GATE-015 Refactor exemption skips red-on-base but requires green
- **WHEN** a change declares `Exempt: refactor — extract token parser` and edits `src/auth.rs`
- **THEN** the gate skips red-on-base and the unspecified-change rule, runs the full test suite at head, and fails if any test fails

#### Scenario: GATE-016 An exemption without a reason is rejected
- **WHEN** a change declares `Exempt: chore` with no reason
- **THEN** the gate fails with reason `exemption needs a reason`

#### Scenario: GATE-017 Spike exemptions are blocked from main
- **WHEN** a change declares `Exempt: spike — try new parser` and the base ref is `main`
- **THEN** the gate fails with reason `spike cannot merge to main`
- **AND** with base ref `spike/parser` the gate passes the exemption

### Requirement: Gate output is compact, machine-readable and has stable exit codes
The gate SHALL print one summary line, then one line per failure, warning and
exemption. With `--json` it SHALL print the same data as JSON. It SHALL exit 0
when it passes, 1 when it fails and 2 on a usage or environment error.

#### Scenario: GATE-018 Passing gate prints one line and exits 0
- **WHEN** every added scenario is tested, red on base and green on head
- **THEN** the gate prints `gate: pass — 2 scenarios proven (1 red:assert, 1 red:compile)` and exits 0

#### Scenario: GATE-019 Failing gate lists reasons and exits 1
- **WHEN** one scenario is untested and one is green-on-base
- **THEN** the gate prints a summary line with `fail`, one line per reason, and exits 1, and `--json` prints the same reasons as JSON

#### Scenario: GATE-020 Missing base ref is an environment error
- **WHEN** `--base origin/main` does not resolve in the repository
- **THEN** the gate prints an error naming the ref and suggesting `fetch-depth: 0`, and exits 2

### Requirement: Red-before-green history can be checked retroactively
`sj gate --since <commit>` SHALL check, for every scenario ID named in a
`feat(<ID>)` commit subject after `<commit>`, that an earlier commit in the range
has subject `test(<ID>): red`. IDs may be listed as a range (`TRACE-001..006`) or
comma-separated.

#### Scenario: GATE-021 History with red before green passes
- **WHEN** the range has `test(TRACE-001..006): red` followed by `feat(TRACE-001..006): green`
- **THEN** the history check passes for TRACE-001 through TRACE-006

#### Scenario: GATE-022 Green without an earlier red fails
- **WHEN** the range has `feat(GATE-003): green` and no earlier `test(GATE-003): red`
- **THEN** the history check fails with reason `no red commit GATE-003`

### Requirement: Scenarios without an ID cannot enter a change
The gate SHALL fail for each scenario without an ID whose heading and step lines (whitespace ignored) match no scenario on base. Scenarios without an ID that match one on base SHALL NOT fail the gate; the gate SHALL report them as one warning with their count.

#### Scenario: GATE-024 A new scenario without an ID fails the gate
- **WHEN** the change adds `#### Scenario: Logout clears session` with no ID
- **THEN** the gate fails with reason `unidentified Logout clears session (<file>:<line>)`

#### Scenario: GATE-025 Existing scenarios without IDs only warn, once
- **WHEN** base and head both hold the same 3 scenarios without IDs, and the change adds a scenario with an ID that is tested, red on base and green on head
- **THEN** the gate passes and prints one warning `3 scenarios without IDs`
- **AND** a scenario without an ID whose THEN line the change rewords fails as `unidentified`

### Requirement: The gate refuses a repository it cannot test
`sj gate --base` SHALL exit 2 with an error naming the supported runners when no test suite is configured: `specjutsu.toml` lists no suite, or there is no `specjutsu.toml` and no root `Cargo.toml`. The error SHALL suggest `sj adopt`. `sj gate --since` runs no tests and is not affected.

#### Scenario: GATE-026 A repository with no supported runner is an environment error
- **WHEN** `sj gate --base main` runs in a git repository with `package.json`, no `Cargo.toml` and no `specjutsu.toml`
- **THEN** it prints an error saying no test suite is configured, naming the supported runners `cargo, jest, junit, playwright, pytest, vitest` and suggesting `sj adopt`, and exits 2

### Requirement: Exempted source changes still pass the full suite
When a change declares any exemption and modifies implementation files, the gate SHALL run the full test suite at head and SHALL fail for every failing test.

#### Scenario: GATE-029 A chore exemption does not let a failing test through
- **WHEN** a change declares `Exempt: chore — bump token lifetime`, edits `src/auth.rs`, and a test fails at head
- **THEN** the gate fails with reason `failing <test path>`

### Requirement: Red-on-base overlays each suite's test and support files
For non-Cargo suites, the gate SHALL overlay on the base worktree every changed file that matches a suite's `tests` or `support` globs. It SHALL run the suite's `setup` command in the base worktree before the first base run of that suite, and run only the target tests there. A setup failure SHALL be an environment error (exit 2), not red.

#### Scenario: GATE-030 A co-located test is overlaid without its source file
- **WHEN** the change adds `apps/api/src/auth.test.ts` for AUTH-004 and edits `apps/api/src/auth.ts`
- **THEN** the base run uses base's `auth.ts` and head's `auth.test.ts`, and AUTH-004 passes red-on-base if the test fails there

#### Scenario: GATE-031 Base dependencies are installed before the base run
- **WHEN** the vitest suite of `apps/api` declares `setup = "pnpm install --frozen-lockfile"`
- **THEN** the gate runs it once in the base worktree before running AUTH-004's test there

#### Scenario: GATE-032 A failing base setup is an environment error
- **WHEN** the base `setup` command exits non-zero
- **THEN** the gate prints the setup's last 20 lines and exits 2

#### Scenario: GATE-033 A missing symbol on base is red:compile
- **WHEN** AUTH-004's vitest test imports `refreshToken`, which does not exist on base, so the suite fails before writing a report entry for the test
- **THEN** AUTH-004 is recorded `red:compile`

### Requirement: Each suite declares its red-on-base policy
A suite SHALL accept `red_on_base = "run"` (default), `red_on_base = "assert"`, or `red_on_base = "skip"` with a `reason`. In an `assert` suite, only an assertion failure on base SHALL count as red, and a target test that fails to compile or load on base SHALL fail the gate with `compile-red <ID> <test name> (suite requires an assertion failure on base)`. Target tests in a skipped suite SHALL be reported `unverifiable (<reason>)` and fail the gate unless it runs with `--allow-unverifiable`. Head runs are never skipped.

#### Scenario: GATE-034 An e2e suite that skips red-on-base is unverifiable
- **WHEN** the Playwright suite declares `red_on_base = "skip"`, `reason = "needs staging API"`, and CHAT-002's only test is in it
- **THEN** the gate fails with `unverifiable CHAT-002 sends a message (needs staging API)`
- **AND** with `--allow-unverifiable` it passes with that line as a warning, provided the test passes at head

#### Scenario: GATE-061 An assert suite rejects a compile red
- **WHEN** the vitest suite of `apps/api` declares `red_on_base = "assert"`, AUTH-004's test fails to load on base, and passes at head
- **THEN** the gate fails with reason `compile-red AUTH-004 <test name> (suite requires an assertion failure on base)`

### Requirement: Implementation files come from package source globs
When `specjutsu.toml` declares a package with non-empty `sources`, the unspecified-change rule SHALL treat a changed file owned by that package as an implementation file only when it matches those globs. A file owned by no package, or by a package with empty `sources`, SHALL be classified by the built-in rule. A top-level `non_source` list of globs SHALL extend the built-in non-source list, and files matching a suite's `tests` or `support` globs SHALL never be implementation files.

#### Scenario: GATE-035 Only files matching a package's sources need a scenario
- **WHEN** package `backend` has `sources = ["backend/src/**/*.py"]` and the change edits `backend/src/diff.py` and `backend/notebooks/explore.py` with no scenario or exemption
- **THEN** the gate fails with `unspecified change: backend/src/diff.py`

#### Scenario: GATE-038 Files on the project's non_source list pass without an exemption
- **WHEN** `specjutsu.toml` sets `non_source = ["infra/**"]` and the change edits only `infra/main.tf`
- **THEN** the gate passes

### Requirement: Assigning an ID to an existing scenario is not a change
A scenario with an ID at head whose ID is absent on base SHALL be classified **unchanged**, not added, when its text without the ID matches a scenario without an ID on base. It then follows the unchanged-scenario rules, including **implemented** (GATE-023) when it gains its first test in the same change.

#### Scenario: GATE-036 An ID-only commit passes the gate
- **WHEN** base holds 300 scenarios without IDs and the change only runs `sj ids --write`, giving each an ID
- **THEN** all 300 are classified unchanged, they appear as warnings (gaps), and the gate passes

#### Scenario: GATE-037 Adding an ID and a test at once is still gated
- **WHEN** the change gives the existing scenario `Valid login` the ID AUTH-001, adds a test for AUTH-001 that passes on base, and edits `src/auth.rs`
- **THEN** AUTH-001 is classified implemented and recorded `green:linked`, and the gate fails with reason `unspecified change` listing `src/auth.rs`

### Requirement: Keep scenarios must pass on base and at head
A scenario whose heading carries `[keep]` after its ID SHALL be treated as behaviour the
change must not alter. The gate MUST require a mapped test for it, MUST run that test on the
base code with the change's test files overlaid, and MUST record it as `green:keep` when it
passes on base and at head.

#### Scenario: GATE-039 A keep test that passes on base and head is accepted
- **WHEN** a change adds `#### Scenario: PAY-012 [keep] Refunds under $100 auto-approve` and its test passes on base and at head
- **THEN** the gate passes PAY-012 with evidence `green:keep`

#### Scenario: GATE-040 A keep test that fails on base fails the gate
- **WHEN** the test for keep scenario PAY-012 fails when run against the base code
- **THEN** the gate fails with reason `keep-red-on-base PAY-012 <test name>` and a hint that the scenario describes new behaviour, not kept behaviour

#### Scenario: GATE-041 A keep test that fails at head fails the gate
- **WHEN** the test for keep scenario PAY-012 passes on base and fails at head
- **THEN** the gate fails with reason `keep-broken PAY-012 <test name>`

#### Scenario: GATE-042 A keep scenario without a test fails the gate
- **WHEN** keep scenario PAY-012 has no mapped test
- **THEN** the gate fails with reason `untested PAY-012`, the same as any other scenario

#### Scenario: GATE-043 Keep evidence appears in the JSON output and PR comment
- **WHEN** `sj gate --json` runs on a change with one keep scenario and one normal scenario
- **THEN** `proven` lists the keep scenario with `"red": "green:keep"`, and `sj gate --format markdown` lists it under a `Kept behaviour` heading, apart from the scenario table

### Requirement: Moving the spec root is not a spec change
The gate SHALL read base scenarios from the base commit's own spec root: the `spec_root` its `specjutsu.toml` sets, otherwise the default for its tree. When the head spec root holds spec files at base, the gate SHALL keep using it for both sides.

#### Scenario: GATE-044 Moving openspec/ to specjutsu/ changes no scenario
- **WHEN** base has `openspec/specs/math/spec.md` with MATH-001 and its passing test, and the change only runs `git mv openspec specjutsu`
- **THEN** `sj gate --base main` prints `gate: pass — no changed scenarios to prove` and exits 0

### Requirement: Base and head read an open change's scenarios the same way
When `specs/` and an open change both hold a scenario, `sj gate` SHALL take its text from the open change on base and at head alike, reading `specs/` first and then the open changes by name.

#### Scenario: GATE-045 A merged, unarchived change is not modified again
- **WHEN** base holds AUTH-001 in `specs/` and an open change that MODIFIES its text, with the code and test merged, and the pull request leaves AUTH-001 alone
- **THEN** AUTH-001 is unchanged and the gate passes, while a pull request that adds an open change modifying AUTH-001 still classifies it modified

### Requirement: Project and agent configuration is not source
The gate's built-in rule SHALL treat `specjutsu.toml` at the repository root, and every file
under a root `.claude/`, `.agents/`, `.agent/`, `.codex/`, `.cursor/`, `.gemini/`, `.kiro/`,
`.windsurf/` or `.clinerules/` folder, as non-source, the same as `.github/`.

#### Scenario: GATE-048 Agent and project config changes need no scenario
- **WHEN** a change edits only `specjutsu.toml`, `.claude/settings.json`, `.agents/hooks.json` and `.cursor/hooks.json`
- **THEN** the gate passes with `gate: pass — no changed scenarios to prove`

#### Scenario: GATE-049 [keep] A nested config-named folder is still source
- **WHEN** a change edits `src/.claude/tool.rs` with no scenario or exemption
- **THEN** the gate fails with `unspecified change: src/.claude/tool.rs`

### Requirement: A base suite that writes no report is an environment error
When a non-Cargo suite's base run writes no parsable report, the gate SHALL treat it as an
environment error, not red: it SHALL name the package and runner, print the runner's last 20
output lines, suggest declaring `setup` or `red_on_base = "skip"`, and exit 2.

#### Scenario: GATE-046 A base run without installed dependencies exits 2
- **WHEN** the vitest suite of `apps/api` declares no `setup`, AUTH-004's test passes on base's
  `auth.ts`, and the base run fails with `Cannot find module 'vitest'` and writes no report
- **THEN** the gate prints `apps/api:vitest`, `Cannot find module 'vitest'` and a hint to declare `setup` on stderr
- **AND** exits 2 instead of recording AUTH-004 `red:compile`

#### Scenario: GATE-047 [keep] A base report with a failed test file is still red:compile
- **WHEN** the vitest suite of `apps/api` declares no `setup`, its base run writes a report whose only entry is a failed `src/auth.test.ts` file, and AUTH-004's test passes at head
- **THEN** AUTH-004 is recorded `red:compile` and the gate passes

### Requirement: The gate honours the base branch's trace baseline
The gate SHALL read `<spec_root>/trace-baseline.txt` as it is on the merge base. Unchanged gaps it lists SHALL NOT get a warning line each. They are counted in a single warning, `known gaps: <n> (trace-baseline.txt)`. Orphans it lists SHALL NOT fail the gate. Added, modified and implemented scenarios SHALL be gated as before, whether the baseline lists them or not. A stale baseline entry SHALL be listed as a warning `stale <entry>` and SHALL NOT fail the gate.

#### Scenario: GATE-050 Baselined gaps collapse into one warning
- **WHEN** the baseline on base lists 2981 gaps, the change touches none of those scenarios, and the gate runs
- **THEN** the warnings hold `known gaps: 2981 (trace-baseline.txt)` and no `gap <ID>` line for any of them

#### Scenario: GATE-051 A baselined orphan does not fail the gate
- **WHEN** `tests/auth.rs` has `auth_009_old` naming the unknown ID AUTH-009, and the baseline on base lists `orphan AUTH-009 tests/auth.rs auth_009_old`
- **THEN** the gate does not fail with `orphan AUTH-009 auth_009_old`

#### Scenario: GATE-052 [keep] A baselined scenario that the change modifies is still gated
- **WHEN** the baseline on base lists `gap AUTH-002`, and the change modifies AUTH-002 without adding a test for it
- **THEN** the gate fails with reason `untested AUTH-002`

#### Scenario: GATE-055 A stale baseline entry only warns
- **WHEN** the baseline on base lists `gap AUTH-002`, and the change adds a test for AUTH-002 without editing the baseline
- **THEN** the gate does not fail because of the baseline and lists `stale gap AUTH-002` under warnings

### Requirement: Baseline entries added by a change need a config exemption
A baseline entry present at head but not on the merge base SHALL NOT be honoured unless the change declares a `config` exemption. Without one, the gate SHALL fail with `baseline-added <entry>` for each such entry. With one, the entries are honoured and each is listed as a warning `baseline-added <entry>`.

#### Scenario: GATE-053 Adding an orphan to the baseline fails without an exemption
- **WHEN** the change adds a test `auth_010_x` naming the unknown ID AUTH-010, and adds `orphan AUTH-010 tests/auth.rs auth_010_x` to the baseline, with no exemption
- **THEN** the gate fails with reasons `orphan AUTH-010 auth_010_x` and `baseline-added orphan AUTH-010 tests/auth.rs auth_010_x`

#### Scenario: GATE-054 Adopting a baseline passes with a config exemption
- **WHEN** base has no baseline and three orphans, and the change only adds `trace-baseline.txt` listing them, with the commit trailer `Specjutsu-Exempt: config — adopt trace baseline`
- **THEN** the gate passes and lists three `baseline-added orphan …` warnings

### Requirement: A test helper missing on base is not red
When a target test fails to load on base, the gate SHALL check every file the change added
or modified that lies in a helper folder (a path segment named `testing`, `test-utils`, `test-helpers`,
`testutils`, `fixtures`, `__fixtures__`, `mocks` or `__mocks__`) and that was not overlaid. If
the base failure message for that test names such a file, by its path from the repository, the
package or the test's folder, with or without its extension, the gate MUST fail with
`not-overlaid <ID> <test name> (<file>; add it to the suite's support globs)` instead of
recording `red:compile`. Paths in the message SHALL match with `\` or `/` as the separator.

#### Scenario: GATE-056 A helper the change added but did not overlay fails the gate
- **WHEN** the change adds `apps/api/src/testing/admin.ts` and `apps/api/src/users.test.ts`, whose test for STAFF-073 imports `./testing/admin`, no `support` glob matches `admin.ts`, and the base run reports `users.test.ts` as a failed file with the message `Failed to load url ./testing/admin`
- **THEN** the gate fails with reason `not-overlaid STAFF-073 <test name> (apps/api/src/testing/admin.ts; add it to the suite's support globs)` and does not record STAFF-073 as `red:compile`

#### Scenario: GATE-057 [keep] An overlaid helper lets the test fail on an assertion
- **WHEN** the same change runs with `support = ["apps/api/src/testing/**"]` on the vitest suite, and STAFF-073's test loads on base and fails an assertion
- **THEN** STAFF-073 is recorded `red:assert`

#### Scenario: GATE-062 A helper the change modified but did not overlay fails the gate
- **WHEN** base already has `apps/api/src/testing/index.ts`, the change adds an export `makeAdmin` to it and a test for STAFF-074 in `apps/api/src/staff.test.ts` that imports it, no `support` glob matches the file, and the base run reports `staff.test.ts` as a failed file with the message `The requested module './testing/index.ts' does not provide an export named 'makeAdmin'`
- **THEN** the gate fails with reason `not-overlaid STAFF-074 <test name> (apps/api/src/testing/index.ts; add it to the suite's support globs)`

#### Scenario: GATE-058 [keep] A new source module missing on base is still red:compile
- **WHEN** the change adds `apps/api/src/usage.ts` and a test for USAGE-001 that imports it, and the base run fails to load the test because `./usage` does not exist
- **THEN** USAGE-001 is recorded `red:compile`

### Requirement: An implemented scenario that passes on base is linked
An **implemented** scenario (GATE-023) whose tests all pass on base and at head SHALL pass the
gate, recorded as `green:linked`: the behaviour existed before the change, and the change only
links a test to it. A `green:linked` scenario SHALL NOT count as specifying a source change.
`--json` SHALL list it in `proven` with `"red": "green:linked"`, and `--format markdown` SHALL
list it under a `Linked existing behaviour` heading, apart from the scenario table.

#### Scenario: GATE-059 A test linked to existing behaviour passes the gate
- **WHEN** base has scenario STAFF-063 with no test, and the change adds a test for STAFF-063 that passes on base and at head, and changes no source file
- **THEN** the gate passes and `--json` lists STAFF-063 in `proven` with `"red": "green:linked"`

#### Scenario: GATE-060 A linked scenario does not cover a source change
- **WHEN** the same change also edits `apps/api/src/users.ts`, STAFF-063 is its only implemented, added or modified scenario, and it declares no exemption
- **THEN** the gate fails with reason `unspecified change` listing `apps/api/src/users.ts`

### Requirement: FOR ALL scenarios need a property test
For every added, modified or implemented scenario with a FOR ALL step, the gate MUST require
at least one mapped property test. When every mapped test is a non-property test, the gate
SHALL fail with `not-property <ID> <test>` and a hint naming the property library for that
test's suite (proptest for Cargo, Hypothesis for pytest, fast-check for vitest, Jest and
Playwright). Only the property tests SHALL count as evidence for the scenario: red on base
and green at head are checked on them. The pass line SHALL count proven property
scenarios after the evidence kinds, and `--json` SHALL mark each proven entry with
`"property": true` or `false`.

#### Scenario: GATE-063 A FOR ALL scenario with only example tests fails the gate
- **WHEN** a change adds FOR ALL scenario PAY-020 and its only test, `pay_020_refund_of_50_on_100` in a Cargo suite, is not a property test
- **THEN** the gate fails with reason `not-property PAY-020 pay_020_refund_of_50_on_100` and a hint that names proptest

#### Scenario: GATE-064 A FOR ALL scenario with a property test that is red on base passes
- **WHEN** a change adds FOR ALL scenario PAY-020 and normal scenario PAY-021, each test fails its assertion on base and passes at head, and PAY-020's test is a property test
- **THEN** the gate prints `gate: pass — 2 scenarios proven (2 red:assert; 1 property)`, and `--json` lists PAY-020 in `proven` with `"property": true` and PAY-021 with `"property": false`

#### Scenario: GATE-065 A kept FOR ALL scenario needs a property test
- **WHEN** a change adds `#### Scenario: PAY-022 [keep] Refund never exceeds the charge` with a FOR ALL step, and its only test is a property test that passes on base and at head
- **THEN** the gate passes PAY-022 with evidence `green:keep` and `"property": true`

#### Scenario: GATE-066 A mixed FOR ALL scenario is proven by its property test
- **WHEN** a change adds FOR ALL scenario PAY-020 with a property test that fails its assertion on base and passes at head, and an example test `pay_020_refund_of_50_on_100` that passes on base and at head
- **THEN** the gate passes PAY-020 with evidence `red:assert`, and no `green-on-base` reason names `pay_020_refund_of_50_on_100`

#### Scenario: GATE-067 [keep] An example test of a FOR ALL scenario must still pass at head
- **WHEN** a change adds FOR ALL scenario PAY-020 with a property test that is red on base and green at head, and an example test `pay_020_refund_of_50_on_100` that fails at head
- **THEN** the gate fails with reason `failing PAY-020 pay_020_refund_of_50_on_100`
