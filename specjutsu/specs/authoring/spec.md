# authoring Specification

## Purpose
Create, check and archive spec changes from the command line, so a project needs no other spec tool to run a spec-driven workflow.

## Requirements

### Requirement: sj spec new scaffolds a change
`sj spec new <change>` SHALL create `<spec_root>/changes/<change>/` with `proposal.md`, `design.md`, `tasks.md` and an empty `specs/` directory, each from a template. `<change>` MUST be kebab-case. It SHALL refuse an existing change name, and print the created path and the next free ID for every prefix in use.

#### Scenario: SPEC-001 A new change is scaffolded
- **WHEN** `sj spec new add-oauth-login` runs and AUTH-001…AUTH-004 exist
- **THEN** the change folder holds the three templates and `specs/`, and the output names the folder and `next AUTH-005`

#### Scenario: SPEC-002 Bad or duplicate names are refused
- **WHEN** `sj spec new Add_OAuth` runs, or `sj spec new add-oauth-login` runs a second time
- **THEN** nothing is created and it exits 2 with the reason

### Requirement: sj spec list shows open changes
`sj spec list` SHALL print one line per open change: its name, checked and total tasks, and the number of scenarios it adds or modifies. `--json` SHALL print the same data.

#### Scenario: SPEC-003 List shows progress
- **WHEN** change `add-oauth-login` has 3 of 7 tasks checked and adds 4 scenarios
- **THEN** `sj spec list` prints `add-oauth-login  3/7 tasks  4 scenarios`

### Requirement: sj spec validate checks structure
`sj spec validate` SHALL report, with file and line: a `#### Scenario:` without an ID, a duplicate ID, a scenario with no `WHEN` or no `THEN` step, a requirement with no scenario or no `SHALL`/`MUST`, a delta file whose requirements are outside `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`, a REMOVED requirement without `**Reason**` and `**Migration**`, a MODIFIED or REMOVED requirement whose name matches no requirement in `specs/`, and a task line that names no scenario ID and is not marked `(no-scenario: <reason>)`. It SHALL exit 0 when clean and 1 otherwise. `--strict` SHALL also fail on warnings.

#### Scenario: SPEC-004 A scenario without THEN is reported
- **WHEN** a delta scenario AUTH-005 has a WHEN line and no THEN line
- **THEN** validate prints `AUTH-005 has no THEN step (<file>:<line>)` and exits 1

#### Scenario: SPEC-005 MODIFIED must name an existing requirement
- **WHEN** a delta has `## MODIFIED Requirements` › `### Requirement: Login flow` and `specs/auth/spec.md` has no requirement named `Login flow`
- **THEN** validate reports `MODIFIED requirement not found: Login flow`

#### Scenario: SPEC-006 A task without a scenario ID is reported
- **WHEN** `tasks.md` has `- [ ] 2.1 Wire up the router` with no ID and no `(no-scenario: …)` marker
- **THEN** validate reports that task with its line

### Requirement: sj spec check finds gaps between spec and tasks
`sj spec check <change>` SHALL report every scenario the change adds or modifies that no task names, every task naming an ID the change and `specs/` do not define, and every `[NEEDS CLARIFICATION: <question>]` marker in the change's files. It SHALL exit 0 when there are none and 1 otherwise.

#### Scenario: SPEC-007 An uncovered scenario and an open question are reported
- **WHEN** the change adds AUTH-005 and AUTH-006, tasks name only AUTH-005, and `proposal.md` contains `[NEEDS CLARIFICATION: token lifetime?]`
- **THEN** check prints `uncovered AUTH-006` and `clarify token lifetime? (proposal.md:<line>)` and exits 1

### Requirement: sj spec archive applies the delta and moves the change
`sj spec archive <change>` SHALL, in this order: refuse with exit 1 if validate fails or any task is unchecked, unless `--force`. Then apply the delta to `<spec_root>/specs/<capability>/spec.md`: ADDED requirements are appended, MODIFIED ones replace the requirement of the same name, REMOVED ones are deleted, and RENAMED ones change the name. A new capability's spec is created with the delta's `## Purpose`. Finally, move the change to `changes/archive/<YYYY-MM-DD>-<change>/`. The merged file SHALL be what `openspec archive` writes: LF line endings, one blank line between requirement blocks, and runs of blank lines outside fenced code collapsed to one. A spec already in that layout SHALL keep every byte outside the edited requirement blocks.

#### Scenario: SPEC-008 Archive merges all four delta kinds
- **WHEN** a change adds requirement `Refresh`, modifies `Login`, removes `Legacy token` and renames `Logout` to `Sign out` in capability `auth`
- **THEN** `specs/auth/spec.md` has `Refresh` appended, `Login` replaced, `Legacy token` gone and `Sign out` in place of `Logout`, and the change folder is under `changes/archive/` with today's date

#### Scenario: SPEC-009 Archive refuses open tasks
- **WHEN** `sj spec archive add-oauth-login` runs with 1 unchecked task
- **THEN** nothing changes and it exits 1 naming the open task

#### Scenario: SPEC-010 A new capability gets its Purpose
- **WHEN** a change adds capability `billing` whose delta starts with `## Purpose` text
- **THEN** archive creates `specs/billing/spec.md` with that Purpose and the added requirements

#### Scenario: SPEC-011 An archived OpenSpec change is readable unchanged
- **WHEN** a change created by OpenSpec (with `.openspec.yaml`) is archived by `sj spec archive`
- **THEN** the result matches what `openspec archive` produces for the same input, ignoring the metadata file

### Requirement: sj spec check keeps inline code in open questions
`sj spec check <change>` SHALL print each `clarify` question with the text of its inline-code spans intact. A marker that itself starts inside an inline-code span SHALL still be ignored.

#### Scenario: SPEC-012 Clarify questions keep their inline code
- **WHEN** `design.md` contains a clarify marker whose question is "do `PreToolUse` hooks fire for `apply_patch` file edits?"
- **THEN** check prints `clarify do PreToolUse hooks fire for apply_patch file edits? (design.md:<line>)`, with the inline code text kept (backticks may be kept)

### Requirement: Templates can be overridden per project
`sj templates` SHALL list each template (`proposal.md`, `design.md`, `tasks.md`, `spec.md`) with the path it resolves to: `<spec_root>/templates/<name>` when that file exists, otherwise `built-in:<name>`. `sj spec new` and `sj instructions` SHALL use the same resolution. `--json` SHALL print an array of objects with `name`, `path` and `source` (`project` or `built-in`).

#### Scenario: SPEC-034 Templates list built-ins by default
- **WHEN** `<spec_root>/templates/` does not exist
- **THEN** `sj templates` prints four lines, each naming a template and `built-in:<name>`

#### Scenario: SPEC-035 A project template wins and is used by sj spec new
- **WHEN** `openspec/templates/proposal.md` exists with the heading `## Problem`
- **THEN** `sj templates` lists `proposal.md  openspec/templates/proposal.md`, and `sj spec new add-x` writes a `proposal.md` starting with `## Problem`

#### Scenario: SPEC-036 JSON templates name their source
- **WHEN** only `openspec/templates/tasks.md` is overridden and `sj templates --json` runs
- **THEN** the `tasks.md` entry has `source` = `project` and the other three have `source` = `built-in`

### Requirement: sj spec show prints one spec, change or scenario
`sj spec show <item>` SHALL resolve `<item>` in this order: a scenario ID, an open change name, an archived change (by its `changes/archive/` folder name, or by its bare change name when exactly one archived change has it), a capability name. An archived change SHALL print as an open one does, with `archived <YYYY-MM-DD>` in place of the task count; a bare name matching several archived changes SHALL exit 1 and list their folder names. For a capability it SHALL print `<spec_root>/specs/<capability>/spec.md` as it is on disk. For a change it SHALL print the proposal, each delta file under a header naming its capability, and the change's checked and total tasks. For a scenario it SHALL print the capability, the requirement name and its SHALL sentence, the scenario's steps, each mapped test as `<file>:<line> <name>`, and the latest ledger result with whether red-before-green is proven. `--json` SHALL print the same data as one JSON object with a `kind` field of `capability`, `change` or `scenario`. When `<item>` matches nothing, it SHALL exit 1 and list up to 3 known names or IDs whose edit distance is lowest. Scenarios in an open change's delta SHALL be found as well as those in `specs/`.

#### Scenario: SPEC-013 Show a scenario with its evidence
- **WHEN** AUTH-003 is in `specs/auth/spec.md`, test `auth_003_rejects_expired_token` at `tests/auth.rs:12` names it, and the ledger shows a failed then a passed run
- **THEN** `sj spec show AUTH-003` prints the requirement name, the WHEN and THEN lines, `tests/auth.rs:12 auth_003_rejects_expired_token` and `latest passed, red-before-green proven`, and exits 0

#### Scenario: SPEC-014 Show a change
- **WHEN** open change `add-oauth-login` has a proposal, a delta for `auth` and 3 of 7 tasks checked
- **THEN** `sj spec show add-oauth-login` prints the proposal text, a header naming `auth` followed by that delta, and `3/7 tasks`

#### Scenario: SPEC-015 Show a capability byte for byte
- **WHEN** `specs/auth/spec.md` exists and no change or scenario ID is named `auth`
- **THEN** `sj spec show auth` prints that file's contents unchanged

#### Scenario: SPEC-016 JSON names the kind
- **WHEN** `sj spec show AUTH-003 --json` runs
- **THEN** stdout is one JSON object with `"kind": "scenario"`, `"id": "AUTH-003"`, a `tests` array and a `latest` field

#### Scenario: SPEC-017 Unknown item suggests near matches
- **WHEN** capabilities `auth` and `audit` exist and `sj spec show aut` runs
- **THEN** it exits 1 and the output names `auth` and `audit` as suggestions

#### Scenario: SPEC-018 A scenario in an open change is found
- **WHEN** AUTH-009 exists only in `changes/add-oauth-login/specs/auth/spec.md` and has no test
- **THEN** `sj spec show AUTH-009` prints its steps, names the change `add-oauth-login`, and reports `untested`

#### Scenario: SPEC-019 Show an archived change by name
- **WHEN** `changes/archive/2026-11-18-audit-checks/` exists, no open change is named `audit-checks`, and `sj spec show audit-checks` runs
- **THEN** it prints that change's proposal and deltas with `archived 2026-11-18`, and `sj spec show 2026-11-18-audit-checks` prints the same

### Requirement: sj spec archive archives several changes at once
`sj spec archive` SHALL accept one or more change names, or `--all-done` for every open change whose tasks are all checked. It SHALL validate every named change and check every precondition of the single-change archive before writing anything; if any change fails, it SHALL archive none, name each failing change with its reason, and exit 1. Changes SHALL be applied in the order given (alphabetical for `--all-done`). Two changes in one run that MODIFY, REMOVE or RENAME the same requirement in the same capability SHALL be refused, naming both changes and the requirement. `--all-done` with no finished change SHALL print `nothing to archive` and exit 0.

#### Scenario: SPEC-030 Archive several finished changes
- **WHEN** `sj spec archive add-login add-refresh` runs and both changes validate with all tasks checked
- **THEN** both deltas are merged into `specs/`, both folders move to `changes/archive/` with today's date, and it exits 0

#### Scenario: SPEC-031 One failing change archives nothing
- **WHEN** `sj spec archive add-login add-refresh` runs and `add-refresh` has 1 unchecked task
- **THEN** no file under `specs/` or `changes/` changes, the output names `add-refresh` and its open task, and it exits 1

#### Scenario: SPEC-032 Two changes editing one requirement are refused
- **WHEN** `add-login` and `add-sso` both have `## MODIFIED Requirements` › `### Requirement: Login flow` in capability `auth` and are archived in one run
- **THEN** nothing changes, the output names `add-login`, `add-sso` and `auth: Login flow`, and it exits 1

#### Scenario: SPEC-033 All-done archives only finished changes
- **WHEN** `sj spec archive --all-done` runs with 3 open changes, of which `add-login` and `add-refresh` have every task checked and `add-sso` has 2 unchecked tasks
- **THEN** `add-login` and `add-refresh` are archived, `add-sso` stays open, and the output lists the two archived changes

### Requirement: Scenario headings may carry a keep marker
The spec parser SHALL accept `[keep]` between a scenario's ID and its name, and MUST store
it on the scenario as `keep`. Any other bracketed word in that position SHALL be a validation
error naming the file and line.

#### Scenario: SPEC-038 A keep marker is parsed
- **WHEN** a delta spec contains `#### Scenario: PAY-012 [keep] Refunds under $100 auto-approve`
- **THEN** `sj spec show PAY-012 --json` reports `"keep": true` and the name `Refunds under $100 auto-approve`

#### Scenario: SPEC-039 An unknown marker is rejected
- **WHEN** a delta spec contains `#### Scenario: PAY-013 [maybe] Partial refunds`
- **THEN** `sj spec validate` exits 1 with `unknown scenario marker [maybe]` and the file and line

### Requirement: sj spec new can scaffold a bug fix
`sj spec new <name> --bug` SHALL write a proposal.md with `## Current Behaviour`,
`## Expected Behaviour` and `## Unchanged Behaviour` sections and a `Kind: bug` line. A change
is a bug change when its proposal.md has that line.

#### Scenario: SPEC-040 A bugfix change is scaffolded
- **WHEN** the user runs `sj spec new fix-refund-rounding --bug`
- **THEN** `openspec/changes/fix-refund-rounding/proposal.md` has the three behaviour sections and the line `Kind: bug`

### Requirement: Bugfix changes are checked for a reproduction and a keep scenario
For a bug change, `sj spec check` SHALL warn when it has no keep scenario and
SHALL report an error when it has no normal (non-keep) scenario to reproduce the bug.

#### Scenario: SPEC-041 A bugfix with no keep scenario is warned about
- **WHEN** a bug change has one normal scenario and no keep scenario
- **THEN** `sj spec check` prints `warn    bug change has no [keep] scenario; add one for behaviour that must not change` and exits 0

#### Scenario: SPEC-042 A bugfix with only keep scenarios is an error
- **WHEN** a bug change has keep scenarios only
- **THEN** `sj spec check` exits 1 with `fail    bug change has no reproduction scenario`

### Requirement: sj spec archive matches MODIFIED scenarios without the keep marker
When `sj spec archive` checks that a MODIFIED requirement keeps every scenario of the current
spec, it SHALL compare scenario headings with any `[keep]` marker removed, so a scenario
carried into a change as `[keep]` matches the same scenario without the marker. The merged
spec SHALL contain the MODIFIED block as the delta writes it, marker included.

#### Scenario: SPEC-043 A keep marker in a MODIFIED block matches the unmarked scenario
- **WHEN** `specs/auth/spec.md` has `#### Scenario: AUTH-002 Wrong password` under `Login`, and a change's `## MODIFIED Requirements` › `### Requirement: Login` has `#### Scenario: AUTH-002 [keep] Wrong password`
- **THEN** archive succeeds and `specs/auth/spec.md` has `#### Scenario: AUTH-002 [keep] Wrong password` under `Login`

#### Scenario: SPEC-044 [keep] A MODIFIED block that drops a scenario is still refused
- **WHEN** a change's `## MODIFIED Requirements` › `### Requirement: Login` has `AUTH-001 Valid login` but not `AUTH-002 Wrong password`
- **THEN** archive fails with `current spec contains scenario(s) not present in the modified block: "AUTH-002 Wrong password"`

### Requirement: sj spec validate points MODIFIED scenarios at sj ids
When a scenario in a MODIFIED requirement matches, by name without its ID and `[keep]` marker,
a scenario without an ID in the same requirement in `specs/`, `sj spec validate` SHALL say to
run `sj ids --write <capability>` first, so both specs use the same name. When the delta
scenario has an ID this SHALL be a warning; when it has none, the hint SHALL be appended to the
missing-ID error. When the delta scenario and an original with the same name both have IDs
and the IDs differ, `sj spec validate` SHALL warn that the names differ for tools that match
by full name, and say to give the delta the original's ID.

#### Scenario: SPEC-045 An ID the original lacks is a warning with the fix
- **WHEN** a delta's `## MODIFIED Requirements` › `### Requirement: Role changes are audited` has `#### Scenario: STAFF-057 Role change is recorded`, and `specs/staff/spec.md` has that requirement with `#### Scenario: Role change is recorded`
- **THEN** validate warns `STAFF-057 Role change is recorded: specs/staff/spec.md has this scenario without an ID; run sj ids --write staff first (<file>:<line>)`, exits 0, and exits 1 with `--strict`

#### Scenario: SPEC-046 A missing ID in a MODIFIED copy names the same fix
- **WHEN** the same delta scenario is `#### Scenario: Role change is recorded`, without an ID
- **THEN** validate's missing-ID error for it ends with `; run sj ids --write staff first`, and it exits 1

#### Scenario: SPEC-047 Different IDs on a copy and its original are a warning
- **WHEN** a delta's MODIFIED copy is `#### Scenario: STAFF-057 Role change is recorded` and `specs/staff/spec.md` has `#### Scenario: STAFF-100 Role change is recorded` in the same requirement
- **THEN** validate warns `STAFF-057 Role change is recorded: specs/staff/spec.md has this scenario as STAFF-100; give the delta copy STAFF-100 (<file>:<line>)`, exits 0, and exits 1 with `--strict`

### Requirement: sj spec infer is a dry run unless asked to write
`sj spec infer <path>...` SHALL select the untraced tests in suite files under the given paths:
tests with no scenario ID in their name, titles, `spec:` comment or `@pytest.mark.spec` marker.
It SHALL print one line per selected test, `<ID> <heading>  (<file>:<line>)`, then a count line,
and SHALL write nothing. Paths resolve against `--root`, or are absolute.

#### Scenario: SPEC-048 A dry run lists the drafts and writes nothing
- **WHEN** `tests/refunds.rs` has untraced tests `refund_under_100_auto_approves` and `refund_over_limit_needs_review`, and `sj spec infer tests/refunds.rs --capability refunds` runs
- **THEN** it prints `REFUNDS-001 [keep] Refund under 100 auto approves  (tests/refunds.rs:<line>)`, a line for REFUNDS-002, and `2 tests drafted`, and `git status` shows no change

#### Scenario: SPEC-049 A target with no untraced test is refused
- **WHEN** every test under `tests/billing/` already carries a scenario ID, and `sj spec infer tests/billing` runs
- **THEN** it exits 2 with `no untraced tests under tests/billing` and writes nothing

### Requirement: sj spec infer --write drafts a change of keep scenarios
With `--write`, `sj spec infer` SHALL create `<spec_root>/changes/<change>/`, where `<change>` is
`--change` or `infer-<capability>`, holding `proposal.md`, `tasks.md` and
`specs/<capability>/spec.md`. The delta SHALL have one `## ADDED Requirements` requirement per
test file, named after the file's stem in words and stating that the system SHALL keep the
behaviour that file tests, and one `[keep]` scenario per selected test. When that name is empty,
or matches another requirement in the delta, in `specs/<capability>/spec.md`, or in an open
change's `specs/<capability>/spec.md`, the requirement
SHALL be named after the file's root-relative path in words, extensions included, instead; if that name is taken too,
the run SHALL stop with exit 2 before writing. When `specs/<capability>/spec.md` exists or an open
change already adds that capability, the proposal SHALL list it under Modified Capabilities,
otherwise under New Capabilities.
On any error while writing, no part of the change folder SHALL be left behind. `tasks.md` SHALL have one task per
requirement naming its scenario IDs. It SHALL refuse with exit 2 and write nothing when the change
folder already exists.

#### Scenario: SPEC-050 Writing creates a change that validates
- **WHEN** `sj spec infer tests/refunds.rs --capability refunds --write` runs on the two tests above
- **THEN** `changes/infer-refunds/specs/refunds/spec.md` has one requirement with scenarios `REFUNDS-001 [keep]` and `REFUNDS-002 [keep]`, `tasks.md` has `(REFUNDS-001..002)`, and `sj spec validate infer-refunds` exits 0

#### Scenario: SPEC-051 An existing change folder is refused
- **WHEN** `changes/infer-refunds/` exists and the same `--write` runs again
- **THEN** it exits 2 with `change infer-refunds already exists`, and no spec or test file changes

#### Scenario: SPEC-065 Test files with the same stem get distinct requirements
- **WHEN** `tests/a/refunds.rs` and `tests/b/refunds.rs` each hold an untraced test and `sj spec infer tests/a tests/b --capability dup --write` runs
- **THEN** the delta has requirements `Tests a refunds rs` and `Tests b refunds rs`, and `sj spec archive infer-dup --force` succeeds; likewise `tests/cart.test.ts` and `tests/cart.spec.ts` in one folder get `Tests cart test ts` and `Tests cart spec ts`

#### Scenario: SPEC-066 Drafts for an existing capability modify it without clashing
- **WHEN** `specs/refunds/spec.md` has `### Requirement: Refunds` with REFUNDS-001, `tests/refunds.rs` holds one untraced test, and `sj spec infer tests/refunds.rs --capability refunds --write` runs
- **THEN** the draft is REFUNDS-002 under `### Requirement: Tests refunds rs`, the proposal lists `refunds` under Modified Capabilities, and `sj spec archive infer-refunds --force` succeeds

#### Scenario: SPEC-069 A second run on the same capability does not clash with an open change
- **WHEN** `sj spec infer tests/one --capability shop --change one --write` has added `### Requirement: Cart` for `tests/one/cart.rs`, and `sj spec infer tests/two --capability shop --change two --write` runs for `tests/two/cart.rs`
- **THEN** change `two` names its requirement `Tests two cart rs` and lists `shop` under Modified Capabilities, and `sj spec archive one --force` followed by `sj spec archive two --force` both succeed

### Requirement: Inferred headings come from the test name
The scenario name SHALL be the test's name made into words: a leading `test` word dropped (`test_`,
or `test` followed by an upper-case letter, never `test` inside a longer word),
`_` and `-` turned into spaces, the first letter upper-cased. When nothing is left after the
`test` word is dropped, the heading SHALL be `Test`. A pytest `Class::func` (or
`Outer::Inner::func`) SHALL become each class name, in order, with a leading `Test` word dropped
by the same rule, followed by the function's words. JS and TS titles SHALL be joined with a space,
keeping their own text.

#### Scenario: SPEC-052 Names from Rust, pytest and JS become headings
- **WHEN** the selected tests are Rust `refund_under_100_auto_approves`, pytest `TestRefunds::test_partial_refund`, JS `Refunds > auto-approves under $100`, Rust `testimony_is_recorded`, pytest `TestimonyRecords::test_saved`, pytest `TestOuter::TestInner::test_x`, and Rust `test_`
- **THEN** their headings are `Refund under 100 auto approves`, `Refunds partial refund`, `Refunds auto-approves under $100`, `Testimony is recorded`, `TestimonyRecords saved`, `Outer Inner x` and `Test`

### Requirement: Inferred scenario bodies ask for the behaviour
Each drafted scenario SHALL have a WHEN and a THEN step whose text is
`[NEEDS CLARIFICATION: <what the test sets up | what the test checks> — see <file>:<line>]`, so
`sj spec check` reports both until they are replaced. With `--write`, `<line>` SHALL be the test's
line after the spec comments are inserted, and the lines `--write` prints SHALL use the same lines.

#### Scenario: SPEC-053 Check reports the drafted markers
- **WHEN** `sj spec check infer-refunds` runs on the change written above
- **THEN** it prints two `clarify` lines per scenario, each naming `tests/refunds.rs:<line>`, and exits 1

#### Scenario: SPEC-067 Markers point at the test after the comments shift it
- **WHEN** `tests/refunds.rs` declares `fn refund_under_100_auto_approves` on line 2 and `fn refund_over_limit_needs_review` on line 5, each preceded by `#[test]`, and `--write` runs
- **THEN** the markers for REFUNDS-001 and REFUNDS-002 name `tests/refunds.rs:3` and `tests/refunds.rs:7`, the lines where those `fn` declarations now are

### Requirement: sj spec infer links each test with a spec comment
With `--write`, `sj spec infer` SHALL insert `// spec: <ID>` (Rust, JS, TS) or `# spec: <ID>`
(Python) on its own line directly above the test's first attribute, decorator or declaration
line, with that line's indentation. No other byte of the test file SHALL change.

#### Scenario: SPEC-054 Tests gain a spec comment and nothing else
- **WHEN** `--write` drafts REFUNDS-001 for a Rust test preceded by `#[test]` and REFUNDS-003 for a pytest test preceded by `@pytest.mark.slow`
- **THEN** `// spec: REFUNDS-001` sits above `#[test]`, `# spec: REFUNDS-003` sits above the decorator at the same indentation, the diff of each file is that one added line per test, and `sj trace` maps both IDs

### Requirement: Linked and skipped tests are left out
`sj spec infer` SHALL leave out tests that already carry a scenario ID and tests marked skip,
todo or fixme. The count line SHALL be `<N> test(s) drafted`, followed by
`, <L> linked and <S> skipped left out` with either part omitted when its count is 0.

#### Scenario: SPEC-055 A second run drafts nothing new
- **WHEN** `sj spec infer tests/refunds.rs` runs after the `--write` above
- **THEN** it exits 2 with `no untraced tests under tests/refunds.rs`

#### Scenario: SPEC-056 Skipped tests are counted, not drafted
- **WHEN** the target holds three untraced tests, one marked `#[ignore]` and one inside `describe.skip`, and one test linked to an existing scenario by a `spec:` comment
- **THEN** one scenario is drafted and the count line reads `1 test drafted, 1 linked and 2 skipped left out`

### Requirement: Inferred IDs follow the ID rules
The capability SHALL be `--capability`, or else the name of the nearest folder above the test file
that is not `tests`, `test`, `__tests__`, `spec`, `unit`, `integration`, `e2e` or `src` (compared
case-insensitively), in kebab-case. All selected tests SHALL name the same capability: tests whose
folders name different capabilities, or a test with no such folder above it, stop the run with
exit 2 and a hint to pass `--capability`. Its prefix and numbering SHALL follow the `ids` rules: a
configured `[ids.prefixes]` entry wins, numbers continue after the capability's current,
open-change and archived IDs, a prefix that collides with another capability's stops the run with
exit 2, and a capability with no letters to make a prefix stops the run with exit 2 as `sj ids`
does. Numbering SHALL also continue after the highest number with that prefix in any scanned
test's name or `spec:` comment, so a drafted ID never matches an ID a test already carries. A
selected test whose name already carries an ID with the capability's prefix SHALL be drafted under
that ID when no other scanned test carries it and no current, open-change or archived scenario has
ever used it; otherwise the run SHALL stop with exit 2 before writing, naming the test and the ID
and asking for the test to be renamed. Drafts SHALL be listed in file order. A
`--capability` value that is not kebab-case SHALL stop the run with exit 2 before anything is
written.

#### Scenario: SPEC-057 An existing capability keeps its prefix and numbering
- **WHEN** `specjutsu.toml` maps `billing = "BILL"`, `specs/billing/spec.md` ends at BILL-014, an archived change used BILL-015, and `sj spec infer tests/invoices.rs --capability billing` runs
- **THEN** the first draft is `BILL-016`

#### Scenario: SPEC-058 The capability is taken from the folder
- **WHEN** `sj spec infer packages/checkout/tests/cart.test.ts` runs without `--capability`
- **THEN** the drafts use capability `checkout` and prefix `CHECKOUT`

#### Scenario: SPEC-061 Test folders are skipped and the name is kebab-cased
- **WHEN** `sj spec infer packages/CartItems/Tests/unit/cart.test.ts` runs without `--capability`
- **THEN** the capability is `cart-items` and the first draft is `CARTITEMS-001`

#### Scenario: SPEC-059 A colliding prefix stops the run
- **WHEN** `specjutsu.toml` maps `payments = "PAY"`, `specs/payments/spec.md` has PAY-001, and `sj spec infer tests/pay --capability pay` runs, whose derived prefix is also `PAY`
- **THEN** it exits 2 naming both capabilities and the prefix, and writes nothing

#### Scenario: SPEC-062 Tests in different capability folders need --capability
- **WHEN** `sj spec infer packages/cart packages/orders` runs without `--capability`, and both folders hold untraced tests
- **THEN** it exits 2 with `the tests' folders name capabilities cart, orders; pass --capability to pick one`, and writes nothing

#### Scenario: SPEC-063 A test with no capability folder needs --capability
- **WHEN** `sj spec infer tests/top.rs` runs without `--capability`
- **THEN** it exits 2 with `tests/top.rs is in no capability folder; pass --capability`, and writes nothing

#### Scenario: SPEC-068 Drafted IDs skip numbers tests already carry
- **WHEN** no `billing` spec exists, `tests/billing.rs` has `fn billing_007_named_early()` and an untraced `fn counts_invoices()`, and `sj spec infer tests/billing.rs --capability billing` runs
- **THEN** `billing_007_named_early` is drafted as `BILLING-007` and `counts_invoices` as `BILLING-008`, in that order

#### Scenario: SPEC-071 A name ID that is shared or was used before is refused
- **WHEN** `tests/billing.rs` has `fn billing_007_inside()` and `tests/other.rs` has `fn billing_007_outside()`, and `sj spec infer tests/billing.rs --capability billing` runs; or an archived change once added BILLING-015 and `tests/billing.rs` has `fn billing_015_revived()`
- **THEN** it exits 2 naming the test and the ID (`BILLING-007` or `BILLING-015`) and asking for the test to be renamed, and writes nothing

#### Scenario: SPEC-070 A capability that is not kebab-case is refused
- **WHEN** `sj spec infer tests/x.rs --capability ../../oops --write` runs, or `--capability "Billing Stuff"`
- **THEN** it exits 2 with ``capability `<value>` must be kebab-case``, and writes nothing

#### Scenario: SPEC-064 A capability with no letters is refused
- **WHEN** `sj spec infer tests/x.rs --capability 2024` runs
- **THEN** it exits 2 with ``capability `2024` has no letters to make a prefix; map it in [ids.prefixes]``, and writes nothing

### Requirement: An inferred change passes the gate as keep
Once its markers are replaced, a change written by `sj spec infer` SHALL pass `sj gate` with every
scenario's evidence `green:keep`, because each test passes on base and at head and only gained a
comment.

#### Scenario: SPEC-060 The filled-in change passes the gate
- **WHEN** the markers in `infer-refunds` are replaced with WHEN/THEN text and `sj gate --base main` runs on the branch
- **THEN** the gate passes with REFUNDS-001 and REFUNDS-002 as `green:keep`

### Requirement: A FOR ALL step marks a property scenario
The spec parser SHALL accept a `- **FOR ALL** <inputs>` step in a scenario and MUST store
the scenario as a property. `sj spec show <ID>` SHALL print the FOR ALL step with the other
steps, and `sj spec show <ID> --json` SHALL report `"property": true` for a property
scenario and `"property": false` for any other.

#### Scenario: SPEC-072 A FOR ALL step is parsed
- **WHEN** a delta spec has `#### Scenario: PAY-020 Refund never exceeds the charge` with the steps `- **FOR ALL** charges and refund requests`, `- **WHEN** a refund is issued` and `- **THEN** the refunded total is at most the charge`
- **THEN** `sj spec show PAY-020 --json` reports `"property": true` and lists `FOR ALL charges and refund requests` as the first step

### Requirement: The FOR ALL step comes before WHEN
`sj spec validate` SHALL report a scenario whose FOR ALL step comes after its first WHEN
step, with the file and line, and exit 1. The FOR ALL step names the inputs the WHEN and THEN steps
range over, so it MUST come first.

#### Scenario: SPEC-073 A FOR ALL step after WHEN is reported
- **WHEN** scenario PAY-021 has `- **WHEN** a refund is issued`, then `- **FOR ALL** charges`, then a THEN step
- **THEN** `sj spec validate` prints `PAY-021 FOR ALL step must come before WHEN (<file>:<line>)` and exits 1
