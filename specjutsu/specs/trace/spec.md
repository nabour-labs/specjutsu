# trace Specification

## Purpose
Map every spec scenario to the tests that cover it, by stable scenario ID, and report scenarios that have no test (gaps) and tests that name an unknown ID (orphans).

## Requirements

### Requirement: Scenario IDs are parsed from spec headings
The system SHALL extract a scenario ID from every `#### Scenario:` heading whose
name begins with an ID of the form `PREFIX-NNN` (uppercase letters, a hyphen,
one or more digits), recording the spec file and line.

#### Scenario: TRACE-001 Parse scenario IDs from headings
- **WHEN** a spec file contains `#### Scenario: AUTH-003 Reject expired token`
- **THEN** the parser returns scenario `AUTH-003` named `Reject expired token` with its file path and line number

#### Scenario: TRACE-002 Headings without an ID are reported, not dropped
- **WHEN** a spec file contains `#### Scenario: Reject expired token` with no ID
- **THEN** the parser returns it as an unidentified scenario with its file and line

### Requirement: Spec files are discovered under the spec root
The system SHALL read `specs/**/spec.md` and `changes/*/specs/**/spec.md` under
the spec root and SHALL ignore `changes/archive/`.

#### Scenario: TRACE-003 Discover current and change specs, skip archive
- **WHEN** the spec root holds `specs/a/spec.md`, `changes/x/specs/b/spec.md` and `changes/archive/old/specs/c/spec.md`
- **THEN** scenarios from the first two files are returned and none from the archive

### Requirement: Tests are matched to scenario IDs
The system SHALL match a Rust test to a scenario when the test function name
contains the ID with `-` replaced by `_`, compared case-insensitively, or when a
`// spec: <ID>` comment appears within the lines directly above the test.

#### Scenario: TRACE-004 Match IDs in Rust test names and spec comments
- **WHEN** a test file contains `fn auth_003_rejects_expired()` and another test preceded by `// spec: AUTH-004`
- **THEN** the matcher returns links AUTH-003 → `auth_003_rejects_expired` and AUTH-004 → that test, each with file and line

### Requirement: The trace report lists gaps and orphans
The system SHALL report every scenario with its linked tests, every scenario with
no test, and every test linked to an ID that no scenario defines.

#### Scenario: TRACE-005 Report mapped, gaps and orphans
- **WHEN** scenarios AUTH-001 and AUTH-002 exist and tests reference AUTH-001 and AUTH-009
- **THEN** the report shows AUTH-001 as mapped, AUTH-002 as a gap and AUTH-009 as an orphan

#### Scenario: TRACE-006 sj trace prints a compact report and supports JSON
- **WHEN** `sj trace` runs in a repo with one gap
- **THEN** it prints a summary line with counts and one line per gap, exiting 0, and `--json` prints the same data as JSON
- **AND** with `--strict` it exits 1

### Requirement: Test files are found through each suite's globs
The system SHALL find test files for each configured suite by matching its `tests` globs under the package path, respecting `.gitignore`. When a suite declares no globs, the runner's defaults SHALL apply: vitest and Jest `**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}`, Playwright `**/*.spec.ts` under its configured test directory, pytest `**/test_*.py` and `**/*_test.py`, and cargo the current Rust rule.

#### Scenario: TRACE-007 Co-located and tests-folder files are both found
- **WHEN** package `apps/api` has a vitest suite with default globs and contains `src/auth.test.ts` and `tests/unit/models.test.ts`
- **THEN** both files are scanned and their tests belong to package `apps/api`, suite `vitest`

#### Scenario: TRACE-008 Files outside every suite are not scanned
- **WHEN** `scripts/seed.ts` contains a call `test("AUTH-003 x")` and matches no suite's globs
- **THEN** it produces no test and no orphan

### Requirement: TypeScript and JavaScript tests link by title, describe or comment
In vitest, Jest and Playwright files the system SHALL treat each `it(…)`, `test(…)` and their `.only`/`.skip`/`.each` forms with a string-literal title as a test. A test SHALL link to every scenario ID that appears in its own title, in the title of any enclosing `describe(…)`, or in a `// spec: <IDs>` comment on the lines directly above the test or its `describe`. A test's name is its titles joined with ` > `.

#### Scenario: TRACE-009 ID in an it() title
- **WHEN** `auth.test.ts` contains `it("AUTH-003 rejects expired token", …)`
- **THEN** the trace links AUTH-003 to the test `AUTH-003 rejects expired token` with its file and line

#### Scenario: TRACE-010 ID in a describe() title applies to every test inside
- **WHEN** `describe("AUTH-004 refresh", () => { it("issues a new token"); it("revokes the old one") })`
- **THEN** both tests link to AUTH-004, named `AUTH-004 refresh > issues a new token` and `AUTH-004 refresh > revokes the old one`

#### Scenario: TRACE-011 Spec comment above a Playwright test
- **WHEN** an e2e file has `// spec: CHAT-002, CHAT-003` on the line above `test("sends a message", …)`
- **THEN** that test links to CHAT-002 and CHAT-003

#### Scenario: TRACE-012 Skipped tests are found and marked
- **WHEN** a file contains `it.skip("AUTH-005 pending case", …)`
- **THEN** the trace links AUTH-005 to that test and marks it `skipped`

### Requirement: Python tests link by name, marker or comment
In pytest files the system SHALL treat functions named `test*` at module level or inside `Test*` classes as tests. A test SHALL link to a scenario ID that appears in its name (`-` written as `_`, any case), in a `@pytest.mark.spec("<ID>", …)` decorator, or in a `# spec: <IDs>` comment on the lines directly above the function or its class.

#### Scenario: TRACE-013 ID in a pytest function name
- **WHEN** `tests/test_inventory.py` defines `def test_inv_002_removes_item():`
- **THEN** the trace links INV-002 to `test_inv_002_removes_item`

#### Scenario: TRACE-014 pytest marker and class-level comment
- **WHEN** `# spec: INV-003` sits above `class TestDiff:`, whose method `test_added` also has `@pytest.mark.spec("INV-004")`
- **THEN** `TestDiff::test_added` links to INV-003 and INV-004

### Requirement: Trace reports which package and suite each test belongs to
Every linked test in the trace report SHALL carry its package path and suite runner. `sj trace` SHALL warn once per package that has files matching its `sources` globs but no suite.

#### Scenario: TRACE-015 Package and suite appear in JSON
- **WHEN** `sj trace --json` runs in the polyglot fixture
- **THEN** each test entry has `package` and `suite` fields, e.g. `apps/api` and `vitest`

#### Scenario: TRACE-016 A package without a suite warns
- **WHEN** package `packages/ui` has `src/button.tsx` and no suite
- **THEN** `sj trace` prints `warn    no test suite: packages/ui`

### Requirement: A trace baseline records known gaps and orphans
`sj trace --write-baseline` SHALL write every current gap and orphan to `<spec_root>/trace-baseline.txt`, one entry per line, sorted. A gap is written as `gap <ID>`. An orphan is written as `orphan <ID> <test file> <test name>`. It SHALL replace any existing baseline and exit 0.

#### Scenario: TRACE-017 Writing the baseline lists every gap and orphan
- **WHEN** AUTH-005 and AUTH-002 are gaps and `tests/auth.rs` has `auth_009_old` naming the unknown ID AUTH-009, and `sj trace --write-baseline` runs
- **THEN** `specjutsu/trace-baseline.txt` holds exactly the lines `gap AUTH-002`, `gap AUTH-005` and `orphan AUTH-009 tests/auth.rs auth_009_old`, in that order, and the command exits 0

### Requirement: Baselined entries are counted, not listed
When `<spec_root>/trace-baseline.txt` exists, `sj trace` SHALL print one line only for each gap or orphan the baseline doesn't list. Its summary line SHALL give the total count with the number known from the baseline in parentheses. `--json` SHALL mark each gap and orphan with `"known": true` or `false`. `--strict` SHALL exit 1 only when there is an entry the baseline doesn't list, or a stale baseline entry.

#### Scenario: TRACE-018 Only new gaps get a line
- **WHEN** the baseline lists `gap AUTH-002` and `gap AUTH-005`, AUTH-006 is also a gap, and `sj trace --strict` runs
- **THEN** it prints one gap line, for AUTH-006, and a summary containing `3 gaps (2 known)`, and exits 1
- **AND** with `--json` AUTH-006 has `"known": false` and AUTH-002 has `"known": true`

#### Scenario: TRACE-019 Known entries alone pass --strict
- **WHEN** the baseline lists `gap AUTH-002` and `orphan AUTH-009 tests/auth.rs auth_009_old`, those are the only gap and orphan, and `sj trace --strict` runs
- **THEN** it prints no gap or orphan line and exits 0

### Requirement: Stale baseline entries are reported
A baseline entry that is no longer a gap or orphan SHALL be reported as `stale <entry> (remove from trace-baseline.txt)`, and `sj trace --strict` SHALL exit 1 when there is one. `--json` SHALL list them in a `stale` array, as their baseline lines. `sj trace --write-baseline` drops stale entries.

#### Scenario: TRACE-020 A gap that gained a test is stale
- **WHEN** the baseline lists `gap AUTH-002`, a test now names AUTH-002, and `sj trace --strict` runs
- **THEN** it prints `stale gap AUTH-002 (remove from trace-baseline.txt)` and exits 1
- **AND** with `--json` the `stale` array is exactly `["gap AUTH-002"]`
- **AND** after `sj trace --write-baseline` the baseline no longer lists AUTH-002

### Requirement: Ignored Rust tests are marked skipped
The Rust test scanner SHALL mark a test `skipped` when an `#[ignore]` or `#[ignore = "<reason>"]`
attribute appears among the attributes directly above it, as the JS and Python scanners do for
their skip markers.

#### Scenario: TRACE-021 An ignored Rust test is marked skipped
- **WHEN** AUTH-006 exists and a test file has, on separate lines, `#[test]`, `#[ignore = "slow integration test"]` and `fn auth_006_bulk_import()`
- **THEN** `sj trace --json` links AUTH-006 to `auth_006_bulk_import` with `"skipped": true`

### Requirement: Rust property tests are marked
The Rust test scanner SHALL mark a test `property` when the test function is inside a
`proptest!` block or carries a `#[proptest]` attribute. `sj trace --json` SHALL report
`"property": true` on such a test and `"property": false` on every other test.

#### Scenario: TRACE-022 A test inside a proptest block is a property test
- **WHEN** PAY-020 exists and a Rust test file has `proptest! {` followed by `#[test]` and `fn pay_020_refund_le_charge(charge in 0u64..10_000, refund in 0u64..10_000)`
- **THEN** `sj trace --json` links PAY-020 to `pay_020_refund_le_charge` with `"property": true`

#### Scenario: TRACE-023 A proptest attribute marks a property test
- **WHEN** PAY-020 exists and a Rust test file has `#[proptest]` directly above `fn pay_020_refund_le_charge(charge: u64, refund: u64)`
- **THEN** `sj trace --json` links PAY-020 to `pay_020_refund_le_charge` with `"property": true`

### Requirement: Python property tests are marked
The Python test scanner SHALL mark a test `property` when a `@given(...)` or
`@hypothesis.given(...)` decorator is among its decorators and the file imports
`hypothesis` (`import hypothesis` or `from hypothesis import ...`).

#### Scenario: TRACE-024 A Hypothesis test is a property test
- **WHEN** PAY-020 exists and a Python test file has `from hypothesis import given, strategies as st` and `@given(st.integers(), st.integers())` directly above `def test_pay_020_refund_le_charge(charge, refund):`
- **THEN** `sj trace --json` links PAY-020 to `test_pay_020_refund_le_charge` with `"property": true`

### Requirement: TypeScript and JavaScript property tests are marked
The JS/TS test scanner SHALL mark a test `property` when its body calls `<name>.assert(` or
`<name>.property(` or `<name>.asyncProperty(`, where `<name>` is bound by an import or
`require` of `fast-check` in the same file. A call through any other name SHALL NOT mark the
test.

#### Scenario: TRACE-025 A fast-check test is a property test
- **WHEN** PAY-020 exists and a test file has `import fc from 'fast-check'` and `it('PAY-020 refund never exceeds the charge', () => { fc.assert(fc.property(fc.nat(), fc.nat(), check)) })`
- **THEN** `sj trace --json` links PAY-020 to that test with `"property": true`

#### Scenario: TRACE-026 A property call through another name is not a property test
- **WHEN** PAY-020 exists, a test file imports nothing from `fast-check`, and its test `PAY-020 refund never exceeds the charge` calls `helpers.property(check)`
- **THEN** `sj trace --json` links PAY-020 to that test with `"property": false`

### Requirement: FOR ALL scenarios without a property test are listed
`sj trace` SHALL list every FOR ALL scenario whose linked tests are all non-property tests
as `not-property <ID> <test>...`, and `sj trace --json` SHALL list those IDs under
`not_property`. A FOR ALL scenario with no test at all SHALL be a gap, as before, and not
listed as `not-property`. `--strict` SHALL exit 1 when any are listed. Whether a scenario is FOR ALL SHALL
be read from an open change's copy of it when there is one, as the gate does.

#### Scenario: TRACE-027 A FOR ALL scenario with only example tests is listed
- **WHEN** FOR ALL scenario PAY-020 is linked only to `pay_020_refund_of_50_on_100` and that test is not a property test
- **THEN** `sj trace` prints `not-property PAY-020 pay_020_refund_of_50_on_100`, `sj trace --json` lists `PAY-020` under `not_property`, and `sj trace --strict` exits 1

#### Scenario: TRACE-028 A change that adds FOR ALL to a scenario is checked by its copy
- **WHEN** `specs/pay/spec.md` has PAY-020 without a FOR ALL step, an open change MODIFIES PAY-020 to add `- **FOR ALL** charges and refund requests`, and its only test `pay_020_refund_of_50_on_100` is not a property test
- **THEN** `sj trace` prints `not-property PAY-020 pay_020_refund_of_50_on_100`

#### Scenario: TRACE-029 A change that removes FOR ALL from a scenario is not listed
- **WHEN** `specs/pay/spec.md` has PAY-020 with a FOR ALL step, an open change MODIFIES PAY-020 to drop it, and its only test is not a property test
- **THEN** `sj trace` lists no `not-property` line and `sj trace --strict` exits 0
