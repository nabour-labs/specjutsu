## ADDED Requirements

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
must prove it.

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
The gate SHALL fail when the change modifies implementation files (source files
that are not test files, spec files or documentation) but adds or modifies no
scenario and declares no exemption. Changes touching only documentation,
specs, CI or other non-source files SHALL pass without an exemption.

#### Scenario: GATE-013 Unspecified implementation change fails the gate
- **WHEN** the change edits `src/auth.rs` and adds or modifies no scenario and declares no exemption
- **THEN** the gate fails with reason `unspecified change` listing the changed source files

#### Scenario: GATE-014 Documentation-only change passes without an exemption
- **WHEN** the change edits only `README.md` and `docs/guide.md`
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
