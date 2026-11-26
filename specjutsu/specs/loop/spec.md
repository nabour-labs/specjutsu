# loop Specification

## Purpose
Answer the agent's working-loop questions (where am I, what next, am I done) from files and test evidence, in a few lines, so the agent never has to read whole spec folders or raw test logs.

## Requirements

### Requirement: sj status summarises each open change
`sj status` SHALL print at most 10 lines per open change: its name, checked and total tasks, and its added or modified scenarios counted as proven (red-before-green in the ledger, latest green), green-unproven, red, and untested. It SHALL end with the next unchecked task. `sj status <change>` SHALL show only that change. `--json` SHALL print the same data.

#### Scenario: LOOP-001 Status fits in ten lines
- **WHEN** change `add-oauth-login` has 7 tasks (3 checked) and 4 scenarios: 2 proven, 1 red, 1 untested
- **THEN** `sj status add-oauth-login` prints at most 10 lines including `3/7 tasks`, `2 proven, 1 red, 1 untested` and `next 2.1 <task text>`

### Requirement: sj next gives the minimal context for one task
`sj next [<change>]` SHALL print, for the first unchecked task: the task line, the full text of each scenario it names, the paths of files changed since the merge-base, the path of `<spec_root>/principles.md` when it exists, and one `sj test <ID>` command per scenario. The output SHALL be at most 1,600 bytes plus the scenario texts.

#### Scenario: LOOP-002 Next shows one task and its scenarios
- **WHEN** the first unchecked task is `- [ ] 2.1 Refresh tokens (AUTH-005, AUTH-006)`
- **THEN** `sj next` prints that line, the WHEN/THEN steps of AUTH-005 and AUTH-006, and `sj test AUTH-005` and `sj test AUTH-006`, and no other task

#### Scenario: LOOP-003 Parallel tasks are listed only when independent
- **WHEN** open tasks 3.1 [P] (AUTH-007) and 3.2 [P] (AUTH-008) link to tests in different files, and 3.3 [P] (AUTH-009) links to a test in the same file as AUTH-007
- **THEN** `sj next --parallel` lists 3.1 and 3.2 as one batch and 3.3 separately

### Requirement: Task completion can require proof
`sj task done <change> <task-number>` SHALL check that task's box in `tasks.md` and change no other byte. In `strict` mode it SHALL refuse, exiting 1 and naming each unproven ID, unless the ledger shows red-before-green and a latest green result for every scenario the task names. In `guided` and `advisory` modes it SHALL check the box and print a warning for each unproven ID.

#### Scenario: LOOP-004 Strict mode refuses an unproven task
- **WHEN** mode is `strict` and task 2.1 names AUTH-005, which has only `passed` records in the ledger
- **THEN** `sj task done add-oauth-login 2.1` leaves `tasks.md` unchanged and prints `unproven AUTH-005 (no red run recorded)`, exiting 1

#### Scenario: LOOP-005 Guided mode warns but completes
- **WHEN** mode is `guided` with the same ledger
- **THEN** the box is checked and a warning names AUTH-005

### Requirement: sj verify is the local check before a PR
`sj verify [<change>]` SHALL run the tests linked to the change's added and modified scenarios, and report failing tests, orphans, `sj spec check` problems, unproven scenarios (per the ledger), and the gate's unspecified-change and exemption rules against the merge-base with the default branch. It SHALL NOT run red-on-base. It SHALL print `verify: ok` and exit 0 when there are no problems, and one line per problem with exit 1 otherwise.

#### Scenario: LOOP-006 Verify catches what the gate would catch
- **WHEN** the change adds AUTH-005 with a passing test and AUTH-006 with no test
- **THEN** `sj verify` prints `untested AUTH-006` and exits 1

#### Scenario: LOOP-007 Verify passes a finished change
- **WHEN** every added scenario has a passing test proven red-before-green and `sj spec check` is clean
- **THEN** `sj verify` prints `verify: ok` and exits 0

### Requirement: sj instructions tells an agent what to write next
`sj instructions <change> [<artifact>]` SHALL print, for one artifact of the change (`proposal`, `specs`, `design` or `tasks`): the resolved template path, the project context, the rules for that artifact, the next free ID for every prefix in use, the project language when one is set, and the command that checks the artifact. With no `<artifact>` it SHALL pick the first of `proposal`, `specs`, `design`, `tasks` that is missing or still identical to its template. `--json` SHALL print the same data with the keys `change`, `artifact`, `template`, `context`, `rules`, `next_ids`, `language` and `check`. It SHALL exit 2 for an unknown change or artifact.

#### Scenario: LOOP-012 Instructions pick the first unwritten artifact
- **WHEN** change `add-oauth-login` has a written `proposal.md`, an empty `specs/` and template-only `design.md` and `tasks.md`
- **THEN** `sj instructions add-oauth-login` prints instructions for `specs` and names `changes/add-oauth-login/specs/<capability>/spec.md` as the file to write

#### Scenario: LOOP-013 JSON instructions carry every field
- **WHEN** `sj instructions add-oauth-login design --json` runs
- **THEN** it prints one JSON object with `change` = `add-oauth-login`, `artifact` = `design`, and the keys `template`, `context`, `rules`, `next_ids`, `language` and `check`, with `language` null when none is set

#### Scenario: LOOP-014 Instructions name the next free IDs
- **WHEN** AUTH-001…AUTH-004 exist in `specs/` and the open change `add-sso` adds AUTH-005
- **THEN** `sj instructions add-oauth-login specs` lists `next AUTH-006`

#### Scenario: LOOP-015 Instructions include only the artifact's own rules
- **WHEN** `principles.md` has `### tasks` and `### design` sections under `## Rules`
- **THEN** `sj instructions add-oauth-login tasks` prints the `tasks` rules and not the `design` rules

#### Scenario: LOOP-016 Each artifact names its check
- **WHEN** `sj instructions add-oauth-login <artifact>` runs for `specs`, then `tasks`
- **THEN** the check is `sj spec validate add-oauth-login` for `specs` and `sj spec check add-oauth-login` for `tasks`

#### Scenario: LOOP-017 Unknown change or artifact is an error
- **WHEN** `sj instructions no-such-change` runs, or `sj instructions add-oauth-login readme` runs
- **THEN** it prints nothing to stdout and exits 2, naming the unknown change or listing the four artifacts

#### Scenario: LOOP-018 A fully written change points to the loop
- **WHEN** all four artifacts of `add-oauth-login` are written and `sj instructions add-oauth-login` runs with no artifact
- **THEN** it prints `all artifacts written; run sj next add-oauth-login` and exits 0

#### Scenario: LOOP-019 Instructions carry the project language
- **WHEN** `specjutsu.toml` sets `language = "fr"`
- **THEN** `sj instructions add-oauth-login proposal` says to write prose in `fr` and to keep `### Requirement:`, `#### Scenario:`, `WHEN`, `THEN`, `SHALL`, `MUST` and the delta section headers in English

### Requirement: sj view prints a project dashboard
`sj view` SHALL print, in at most 40 lines for a project with up to 20 capabilities and 10 open changes: one line per capability with its scenario count; one line per open change with checked and total tasks and its scenarios counted as proven, green-unproven, red and untested (the same counts `sj status` uses); project totals for those four counts across all scenarios; the number of orphan tests (tests naming an ID that does not exist); and the number of archived changes. Beyond those sizes it SHALL show the first entries of each list and a `… +N more` line. `--json` SHALL print the same data. It SHALL read files and the ledger only, run no tests and wait for no input.

#### Scenario: LOOP-008 Dashboard shows changes and totals
- **WHEN** the project has capabilities `auth` (6 scenarios) and `billing` (4), open change `add-oauth-login` at 3/7 tasks with 2 proven and 1 untested scenario, 1 orphan test and 5 archived changes
- **THEN** `sj view` prints `auth  6 scenarios`, `billing  4 scenarios`, a line with `add-oauth-login`, `3/7 tasks`, `2 proven` and `1 untested`, `1 orphan test` and `5 archived`, and exits 0

#### Scenario: LOOP-009 Long lists are cut to fit
- **WHEN** the project has 30 capabilities
- **THEN** `sj view` prints 20 capability lines followed by `… +10 more` and at most 40 lines in all

#### Scenario: LOOP-010 JSON dashboard
- **WHEN** `sj view --json` runs on the project in LOOP-008
- **THEN** stdout is one JSON object with `capabilities`, `changes`, `totals`, `orphans` (1) and `archived` (5)

#### Scenario: LOOP-011 View never runs tests
- **WHEN** `sj view` runs with stdin closed and a runner executable that would fail if started
- **THEN** it exits 0 and the ledger file is unchanged

### Requirement: A keep scenario needs no red run
`sj verify` and `sj task done` SHALL NOT report a `[keep]` scenario as unproven, since it must pass on base and so never has a red run. Its tests SHALL still run in `sj verify`, and a failing one SHALL still be reported.

#### Scenario: LOOP-020 Verify and task done accept a passing keep scenario
- **WHEN** a change adds AUTH-005 `[keep]` and AUTH-006, both with passing tests and no red run recorded
- **THEN** neither `sj verify` nor `sj task done` prints an unproven line for AUTH-005

#### Scenario: LOOP-021 [keep] A passing scenario without a red run is still unproven
- **WHEN** a change adds AUTH-006, not `[keep]`, with a passing test and no red run recorded
- **THEN** `sj verify` and `sj task done` print `unproven AUTH-006 (no red run recorded)`

### Requirement: Agent-loop commands record their gain
Each run of `sj test`, `sj verify`, `sj next` and `sj status` that prints its report SHALL append
one record to `.specjutsu/ledger/gain.jsonl` holding the command name, a timestamp, the bytes the
command printed to stdout, and the raw-equivalent bytes. A run that exits with an error before
printing its report records nothing. For `sj test` and `sj verify` the raw-equivalent is the combined
stdout and stderr bytes of the runner processes. For `sj next` it is the total size of every file in
the change folder of the task it reports, and 0 when it reports no open task. For `sj status` it is
the total size of every file in every change folder it reports on: the named one, or every open one.
Recording SHALL NOT change the command's output or exit code.

#### Scenario: LOOP-022 sj test records printed and raw bytes
- **WHEN** `sj test` runs a suite whose runner writes 40,000 bytes of output and `sj test` prints 300 bytes
- **THEN** `gain.jsonl` gains one record with command `test`, printed 300 and raw 40,000

#### Scenario: LOOP-023 sj verify records the runner output as raw
- **WHEN** `sj verify` runs and the runners it starts write 25,000 bytes while `sj verify` prints 450 bytes
- **THEN** `gain.jsonl` gains one record with command `verify`, printed 450 and raw 25,000

#### Scenario: LOOP-024 sj next records the change folder as raw
- **WHEN** `sj next add-oauth-login` runs on a change folder whose files total 12,000 bytes and it prints 900 bytes
- **THEN** `gain.jsonl` gains one record with command `next`, printed 900 and raw 12,000

#### Scenario: LOOP-025 sj status records every open change as raw
- **WHEN** `sj status` runs with two open change folders of 8,000 and 5,000 bytes and prints 400 bytes
- **THEN** `gain.jsonl` gains one record with command `status`, printed 400 and raw 13,000

#### Scenario: LOOP-026 No record when recording is off
- **WHEN** `sj test --no-ledger` runs, or any of the four commands runs outside a git repository
- **THEN** no gain record is written and the command's output and exit code are the same as with recording on

### Requirement: sj gain reports the bytes and tokens saved
`sj gain` SHALL print one line per recorded command with its run count, printed bytes, raw bytes and
the share saved as a whole percent (rounded down), then a `total` line with the same columns and the
estimated tokens saved, computed as (raw − printed) / 4 rounded down; when printed exceeds raw, the
share and the tokens saved are 0. `sj gain --json` SHALL print the same totals as one JSON object,
with the number of unreadable records skipped as `skipped`. `sj gain --since <RFC 3339 timestamp>` SHALL count only records at or
after that time. `sj gain --reset` SHALL delete the records and print how many it removed.

#### Scenario: LOOP-027 Totals per command and overall
- **WHEN** the records are two `test` runs (300/40,000 and 200/10,000 bytes printed/raw) and one `next` run (1,000/4,000) and the user runs `sj gain`
- **THEN** it prints `test 2 500 50000 99%`, then `next 1 1000 4000 75%`, then `total 3 1500 54000 97% ~13125 tokens saved`, in columns, and exits 0

#### Scenario: LOOP-028 JSON totals for a harness
- **WHEN** the user runs `sj gain --json` over the records in LOOP-027
- **THEN** stdout is one JSON object with a `commands` array holding `test` and `next`, each with `runs`, `printed`, `raw` and `saved_pct`, a `total` object with the same fields plus `tokens_saved` 13125, and `skipped` 0

#### Scenario: LOOP-029 Count one session with --since
- **WHEN** one `test` record is at 2026-11-22T10:00:00Z and another at 2026-11-22T12:00:00Z, and the user runs `sj gain --since 2026-11-22T11:00:00Z`
- **THEN** the `test` line shows 1 run and only the later record's bytes

#### Scenario: LOOP-030 Reset clears the records
- **WHEN** `gain.jsonl` holds 7 records and the user runs `sj gain --reset`
- **THEN** it prints `removed 7 gain records`, and a following `sj gain` prints `no gain records yet` and exits 0

#### Scenario: LOOP-031 A damaged record does not block the report
- **WHEN** `gain.jsonl` holds 3 valid records and 1 line that is not valid JSON and the user runs `sj gain`
- **THEN** it reports the 3 valid records, warns on stderr `skipped 1 unreadable gain record`, and exits 0

### Requirement: sj verify honours the trace baseline as the gate does
`sj verify` SHALL apply the same baseline rules as the gate: orphans listed in the merge base's `<spec_root>/trace-baseline.txt` are not problems, and a baseline entry present at head but not on the merge base is a problem `baseline-added <entry>` unless the change declares a `config` exemption.

#### Scenario: LOOP-032 A baselined orphan is not a verify problem
- **WHEN** the merge base's baseline lists `orphan AUTH-009 tests/auth.rs auth_009_old`, that test still exists, and `sj verify` runs
- **THEN** no problem line mentions `orphan AUTH-009`

#### Scenario: LOOP-033 A baseline entry added without an exemption is a verify problem
- **WHEN** the change adds `orphan AUTH-010 tests/auth.rs auth_010_x` to the baseline with no exemption, and `sj verify` runs
- **THEN** it prints the problem line `baseline-added orphan AUTH-010 tests/auth.rs auth_010_x` and exits 1

### Requirement: Only added and modified scenarios need a red run
`sj verify` and `sj task done` SHALL judge each scenario against a merge-base, classifying it
the same way the gate does. For `sj verify` that is the merge-base with `--base` when given,
otherwise with the default branch. `sj task done` always uses the default branch.

They SHALL require a red run only for **added** and **modified** scenarios. The following need
only a green latest run, and SHALL NOT be reported as unproven when they have one:

- an **unchanged** scenario, such as one copied without edits into a MODIFIED requirement;
- an **implemented** scenario (GATE-023). Its behaviour may predate the change, and the gate
  checks it on base.

The tests of these scenarios SHALL still run in `sj verify`, and a failing one SHALL still be
reported.

Every scenario in the change SHALL need a red run, as before, when there is no merge-base or
when the merge-base is HEAD itself (HEAD is already on the default branch). The same applies
when `sj task done` can't read the classification, and it then prints a warning. In `sj verify`,
an error while classifying is an environment error, as it is for the gate rules verify runs.

In its unspecified-change check, `sj verify` SHALL NOT count an implemented scenario as covering
a source change unless the ledger shows it red before green, since without a base run it can't
tell whether the gate will record it `green:linked` (GATE-060).

#### Scenario: LOOP-034 Verify and task done accept a linked existing scenario
- **WHEN** mode is `strict`, the merge-base has scenario STAFF-063 with no test, and task 4.1 of the change names STAFF-063 and adds its first test, which has only `passed` records in the ledger
- **THEN** `sj verify` prints no unproven line for STAFF-063, and `sj task done <change> 4.1` checks the box and exits 0

#### Scenario: LOOP-035 An unchanged scenario in a MODIFIED requirement needs no red run
- **WHEN** a change's `## MODIFIED Requirements` copies AUTH-002 with the same text as on the merge-base, where AUTH-002 already has a passing test, and the ledger has only `passed` records for it
- **THEN** `sj verify` prints no unproven line for AUTH-002, and still reports `failing … AUTH-002` if its test fails

#### Scenario: LOOP-036 [keep] An added scenario committed on the default branch still needs a red run
- **WHEN** mode is `strict`, a change that adds MATH-001 is committed on `main` itself, and MATH-001's test has only `passed` records in the ledger
- **THEN** `sj verify` prints `unproven MATH-001 (no red run recorded)`, and `sj task done` refuses the task naming MATH-001 and exits 1

#### Scenario: LOOP-037 Verify does not let a never-red implemented scenario cover source
- **WHEN** the merge-base has scenario STAFF-063 with no test, the change adds its first test, which has only `passed` records in the ledger, edits `src/lib.rs`, and names no other scenario
- **THEN** `sj verify` reports `unspecified change` listing `src/lib.rs` and exits 1

### Requirement: sj verify reports FOR ALL scenarios without a property test
`sj verify [<change>]` SHALL report every added, modified or implemented FOR ALL scenario
whose mapped tests are all non-property tests as `not-property <ID> <test>`, by the same rule
as the gate, and exit 1. A red-then-green ledger record from an example test SHALL NOT make
such a scenario pass verify.

#### Scenario: LOOP-038 Verify reports a FOR ALL scenario with only example tests
- **WHEN** a change adds FOR ALL scenario PAY-020, its only test `pay_020_refund_of_50_on_100` is not a property test, and the ledger shows it failed and then passed
- **THEN** `sj verify` prints `not-property PAY-020 pay_020_refund_of_50_on_100` and exits 1

#### Scenario: LOOP-040 [keep] Verify leaves an unchanged FOR ALL scenario alone
- **WHEN** a change's MODIFIED requirement carries FOR ALL scenario PAY-020 unchanged, with only an example test, next to a modified scenario PAY-021
- **THEN** `sj verify` prints no `not-property` line

### Requirement: sj task done refuses a FOR ALL scenario without a property test
In `strict` mode, `sj task done <change> <task-number>` SHALL refuse, exiting 1 and leaving
`tasks.md` unchanged, when the task names an added, modified or implemented FOR ALL scenario
whose mapped tests are all non-property tests, and print `not-property <ID> <test>` for it as
`sj verify` does, even when the ledger shows red-before-green. In `guided` and `advisory`
modes it SHALL check the box and print the same line as a warning.

#### Scenario: LOOP-039 Strict task done refuses a FOR ALL scenario with only example tests
- **WHEN** mode is `strict`, task 1.1 names FOR ALL scenario PAY-020, its only test `pay_020_refund_of_50_on_100` is not a property test, and the ledger shows it failed and then passed
- **THEN** `sj task done refunds 1.1` prints `not-property PAY-020 pay_020_refund_of_50_on_100`, exits 1 and leaves `tasks.md` unchanged

#### Scenario: LOOP-041 [keep] Strict task done accepts a FOR ALL scenario with a property test
- **WHEN** mode is `strict`, task 1.1 names FOR ALL scenario PAY-020, which has a property test and an example test, and the ledger shows red-before-green
- **THEN** `sj task done refunds 1.1` checks the box, prints no `not-property` line and exits 0

#### Scenario: LOOP-042 [keep] Strict task done accepts an unchanged FOR ALL scenario
- **WHEN** mode is `strict`, and the task names FOR ALL scenario PAY-020, carried unchanged in a MODIFIED requirement with only an example test
- **THEN** `sj task done` checks the box, prints no `not-property` line and exits 0
