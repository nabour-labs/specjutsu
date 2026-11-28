# ledger Specification

## Purpose
Keep a local, append-only record of every `sj test` run, so it is possible to show which scenarios had a failing test before a passing one, and on which code.

## Requirements

### Requirement: Every test run is recorded
Each `sj test` run SHALL append one record per test that ran to the ledger under `.specjutsu/ledger/` in the project root. Every record SHALL hold the run ID, a timestamp, the git tree hash of the working tree at the time of the run, the test path, the scenario IDs the test maps to, and the result: `passed`, `failed`, `ignored` or `build-failed`.

#### Scenario: LEDGER-001 A run appends one record per test
- **WHEN** `sj test` runs three tests
- **THEN** three records are appended to the ledger
- **AND** all three share one run ID, timestamp and tree hash
- **AND** each holds its own test path, scenario IDs and result

#### Scenario: LEDGER-002 Records accumulate across runs
- **WHEN** `sj test` is run twice
- **THEN** the ledger holds the records of both runs
- **AND** the records of the first run are unchanged

### Requirement: A build failure is recorded against the tests in scope
When the build fails, the run SHALL record a `build-failed` record for each test the run selected that can be found in source, so a compile failure counts as red.

#### Scenario: LEDGER-003 A compile failure records build-failed for the scoped tests
- **WHEN** `sj test AUTH-003` fails to compile and one test in source maps to AUTH-003
- **THEN** the ledger gains one `build-failed` record for that test with AUTH-003

### Requirement: The tree hash identifies the code that ran
The tree hash SHALL identify the working tree's contents, including uncommitted and untracked files that git does not ignore. Computing it SHALL NOT change the user's index, staged changes or working tree.

#### Scenario: LEDGER-004 Uncommitted edits change the tree hash
- **WHEN** `sj test` runs, a source file is edited without committing, and `sj test` runs again
- **THEN** the two runs record different tree hashes

#### Scenario: LEDGER-005 Recording leaves the index untouched
- **WHEN** `sj test` runs in a repository with some changes staged and some unstaged
- **THEN** after the run, `git status` shows exactly the same staged and unstaged changes as before

### Requirement: Recording can be turned off
`sj test --no-ledger` SHALL run the tests without writing to the ledger. Outside a git repository `sj test` SHALL still run the tests, and SHALL skip recording with a warning instead of failing.

#### Scenario: LEDGER-006 --no-ledger writes nothing
- **WHEN** `sj test --no-ledger` runs
- **THEN** the ledger is unchanged

#### Scenario: LEDGER-007 Outside git, tests run and recording is skipped
- **WHEN** `sj test` runs in a Cargo project that is not a git repository
- **THEN** the tests run and the exit code reflects their results
- **AND** a warning says the run was not recorded

### Requirement: The ledger reports red-before-green per scenario
`sj ledger` SHALL report, for each scenario ID in the ledger, the latest recorded result and whether red-before-green is proven. It is proven when a `failed` or `build-failed` record for the scenario comes before a `passed` record for it on a different tree hash. `sj ledger ID` SHALL report only that scenario.

#### Scenario: LEDGER-008 Red then green on different trees is proven
- **WHEN** the ledger holds a `failed` record for AUTH-003 on tree A and a later `passed` record for AUTH-003 on tree B
- **THEN** `sj ledger AUTH-003` reports AUTH-003 as red-before-green proven, latest result passed

#### Scenario: LEDGER-009 Green only, or green before red, is not proven
- **WHEN** the ledger holds only `passed` records for AUTH-004, and for AUTH-005 a `passed` record followed by a later `failed` one
- **THEN** `sj ledger` reports neither AUTH-004 nor AUTH-005 as proven
- **AND** reports AUTH-005's latest result as failed

#### Scenario: LEDGER-010 Red and green on the same tree is not proven
- **WHEN** the ledger holds a `failed` and a later `passed` record for AUTH-006, both on the same tree hash
- **THEN** `sj ledger` does not report AUTH-006 as proven

#### Scenario: LEDGER-011 sj ledger supports JSON
- **WHEN** `sj ledger --json` runs
- **THEN** stdout is one JSON object with an entry per scenario holding its latest result and proven flag

### Requirement: A damaged ledger does not block work
`sj ledger` and `sj test` SHALL skip ledger lines they cannot parse, warn with the count of skipped lines, and continue.

#### Scenario: LEDGER-012 An unparsable line is skipped with a warning
- **WHEN** the ledger holds one corrupt line among valid records
- **THEN** `sj ledger` reports from the valid records
- **AND** warns that one line was skipped

### Requirement: Red records of property tests keep their counterexample
When `sj test` records a `failed` result for a property test, the record SHALL hold an
optional `counterexample` with the input read from the test's failure output, following the
gate's rules for counterexamples. `sj ledger` SHALL show the counterexample of a scenario's
latest red record as `(counterexample: <input>)` after its status, and `sj ledger --json`
SHALL include it as `"counterexample"` in the scenario's entry. Records without the field
SHALL load as before, and the ledger format version SHALL NOT change.

#### Scenario: LEDGER-013 A failing property test's record holds its counterexample
- **WHEN** `sj test PAY-020` runs and the proptest test `pay_020_refund_never_exceeds_charge` fails with `minimal failing input: charge = 100, refund = 101`
- **THEN** its ledger record has result `failed` and `"counterexample": "charge = 100, refund = 101"`

#### Scenario: LEDGER-014 sj ledger shows the latest red counterexample
- **WHEN** the ledger holds a `failed` record for PAY-020 with counterexample `charge = 100, refund = 101` on tree A and a later `passed` record on tree B
- **THEN** `sj ledger PAY-020` reports PAY-020 as proven with `(counterexample: charge = 100, refund = 101)`, and `sj ledger --json` has `"counterexample": "charge = 100, refund = 101"` in its entry

#### Scenario: LEDGER-015 Records without a counterexample stay as they are
- **WHEN** the ledger holds a record written before this change and `sj test` records a failure of an example test
- **THEN** both records load, neither has a `counterexample` key, and `sj ledger` shows no counterexample for their scenarios
