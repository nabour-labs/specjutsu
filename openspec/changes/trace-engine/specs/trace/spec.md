## ADDED Requirements

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
