# Changelog

All notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[SemVer](https://semver.org/).

## [Unreleased]

### Added
- `sj spec infer <path>... [--capability <name>] [--write] [--change <name>]` drafts one
  `[keep]` scenario for every untraced test (no scenario ID in its name, title, `spec:`
  comment or marker) in suite files under the paths. By default it is a dry run that
  prints `<ID> [keep] <heading>  (<file>:<line>)` per test and `N tests drafted, L linked
  and S skipped left out`, and exits 2 with `no untraced tests under <paths>` when there
  are none. Headings come from the test name; the capability is `--capability` or the
  nearest folder above the test that isn't a test or source folder, and IDs follow the
  `sj ids` rules. `--write` creates `changes/infer-<capability>/` (or `--change`) with
  WHEN/THEN as `[NEEDS CLARIFICATION: … — see <file>:<line>]` markers and inserts a
  `spec:` comment above each test, changing no other byte. Once the markers are
  replaced, the change passes `sj gate` as `green:keep`. The `sj-propose` skill has a
  "From existing tests" section for it.
- Property scenarios: a `- **FOR ALL** <inputs>` step, before WHEN, marks a scenario that
  must hold for every input. `sj spec validate` reports a FOR ALL step after WHEN as
  `<ID> FOR ALL step must come before WHEN (<file>:<line>)`, and `sj spec show <ID> --json`
  reports `"property": true` or `false`. The test scanners mark a test `property` by
  reading its source: Rust `proptest! { ... }` blocks and `#[proptest]`; Python
  `@given(...)` or `@hypothesis.given(...)` in a file that imports `hypothesis`; TS/JS
  `<name>.assert(`, `.property(` or `.asyncProperty(` where `<name>` is bound by a
  default, namespace or `require` import of `fast-check`. Aliases of named imports,
  re-exports, `@fast-check/vitest` `test.prop`, a callback passed by reference and the
  closure form `proptest!(|..| ..)` aren't detected. `sj trace --json` reports
  `"property"` on each test. A FOR ALL scenario whose tests are all example tests is
  listed by `sj trace` as `not-property <ID> <test>...` and under `not_property` in
  `--json`; `--strict` exits 1 on it and the baseline never records it. `sj gate` fails
  it, `[keep]` ones included, with `not-property <ID> <test> (a FOR ALL scenario needs a
  property test; write one with <library>)`, naming proptest, Hypothesis or fast-check
  for the suite. In a scenario with both kinds, only the property tests count as
  evidence on base, and the example tests must still pass at head. The pass line counts
  proven properties (`gate: pass — 2 scenarios proven (2 red:assert; 1 property)`), and
  each `proven` entry in `--json` has `"property": true` or `false`.
- Property counterexamples: when a property test fails its assertion on base, `sj gate`
  reads the shrunk input it printed: proptest `minimal failing input: …`, Hypothesis
  `Falsifying example: <test>(…)` (on one line or several; brackets inside quotes are
  skipped and `# …` comments explaining an argument are left out) and fast-check
  `Counterexample: …`. Whitespace is collapsed to one line and cut to 200 characters
  plus `…`. The text report prints `info    counterexample
  PAY-020 pay_020_refund_never_exceeds_charge: charge = 100, refund = 101`, `--json`
  adds `"counterexample": {"test": …, "input": …}` to the `proven` entry, and
  `--format markdown` shows `red:assert (counterexample: charge = 100, refund = 101)`
  in the evidence cell. When a scenario has several, the first of its tests in trace
  order is shown. A missing or unreadable counterexample never changes the gate's
  result. The gate's Cargo base run now keeps each failing test's failure text. `sj
  test` stores `counterexample` on the `failed` ledger record of a property test, `sj
  ledger` shows the latest red record's as `(counterexample: …)` after the status, and
  `sj ledger --json` gives it as the entry's `counterexample`. Older ledger lines still
  load, and the ledger format version is unchanged.

### Changed
- `sj trace` marks Rust tests with `#[ignore]` or `#[ignore = "..."]` as `skipped`, as it
  already did for JS and Python skip markers; it shows in `sj trace --json` and the badge.

## [0.6.0] - 2026-11-24

### Added
- `red_on_base = "assert"` suite policy: only an assertion failure on base counts as red,
  and a test that fails to compile or load there fails as
  `compile-red <ID> <test> (suite requires an assertion failure on base)`. It takes no
  `reason` and applies to Cargo suites too.
- `green:linked`: an implemented scenario (one that gains its first test) whose tests pass
  on base and at head passes the gate instead of failing as `green-on-base`, and the pass
  line counts it (`gate: pass — 1 scenario proven (1 green:linked)`). An implemented
  scenario whose tests all pass on base never counts as specifying a source change, even
  when a test fails at head, so a change that also edits source still needs a red-on-base
  scenario or an exemption. When one of its tests fails on base, that proves it instead.
  `--json` lists it in `proven` with `"red": "green:linked"`; `--format markdown` lists it
  under **Linked existing behaviour**.
- `sj ids [--write] [<target>...]` takes capability names or paths to spec files or
  folders (resolved against `--root`, or absolute) and lists and writes only those whole
  capabilities. The prefix check covers only the prefixes the selected capabilities use,
  so an unrelated collision doesn't stop the run; numbering still covers the whole spec
  root. A target that selects nothing exits 2 and lists the known capabilities. A
  capability named `prefixes` can only be targeted by path.
- `sj adopt` declares each package's test helper folders (`testing`, `test-utils`,
  `test-helpers`, `testutils`, `fixtures`, `__fixtures__`, `mocks`, `__mocks__`) as
  `support` globs on its non-Cargo suites, except folders under a `tests` folder, and
  prints `note    support: <glob> (not source; overlaid on base; remove it if this folder
  holds product code)` on stderr for each. Support files aren't source and are overlaid
  on base, so don't mark product code as support. A helper folder it can't read stops it
  with the error.
- `sj spec validate` says to run `sj ids --write <capability>` first when a MODIFIED
  scenario's original in `specs/` has no ID: as a warning when the delta scenario has an
  ID, appended to the missing-ID error when it has none. When both have IDs and they
  differ, it warns, as in `STAFF-057 Role change is recorded: specs/staff/spec.md has this
  scenario as STAFF-100; give the delta copy STAFF-100`.

### Changed
- `sj-explore` covers brainstorming: its description names it, so "let's brainstorm"
  triggers it; it lists many ideas before narrowing to two or three options; and the new
  reference `sj-explore/brainstorming.md` holds idea prompts, a YAGNI pass, and presenting
  the design one section at a time. The skill check enforces all three, and the
  `brainstorming` row of `skills/COVERAGE.md` now has depth `brainstorming.md`.
- `sj verify` and `sj task done` classify scenarios against the merge-base like the gate
  and need a red run only for added and modified scenarios. Unchanged scenarios, such as
  ones copied into a MODIFIED requirement, and implemented ones need only a passing latest
  run. On the default branch itself, or when `sj task done` can't classify (it warns),
  every scenario needs a red run. `sj task done` always compares with the default branch.
  In `sj verify`, an implemented scenario with no red-before-green in the ledger doesn't
  cover a source change, so editing source alongside it is an `unspecified change`.
- `sj ids --write` gives a `specs/` scenario without an ID the ID its MODIFIED copy in an
  open change already has, matching a RENAMED requirement by its old name. When that ID is
  taken, or copies in two changes disagree, it assigns a new number and prints a `note:`
  line ending `; give the delta copies <ID>`; `--json` carries it as a `note` object.

### Fixed
- A test that failed to load on base because it imports a test helper the change added or
  modified but didn't overlay no longer passes as `red:compile`. The gate fails it as
  `not-overlaid <ID> <test> (<file>; add it to the suite's support globs)`, matching the
  helper's path in the base failure message with `\` or `/` as the separator (non-Cargo
  suites). A new source module missing on base is still `red:compile`.
- `sj ids --write` gives a delta copy without an ID, under a requirement renamed in the
  same change, the ID of its `specs/` original matched by the old name. It used to give
  the copy and the original different numbers.
- `sj ids --write` exits 2 naming both capabilities when a capability that needs IDs shares
  its prefix with a fully numbered one, as in `` capabilities `st-aff` and `staff` both use
  prefix STAFF ``. It used to continue the other capability's numbering.

## [0.5.1] - 2026-11-23

### Changed
- **Breaking (output only):** `sj migrate --write` ends with `rollback: sj migrate --undo`
  instead of `rollback: git restore . && git clean -fd`, which also deleted untracked
  files made after the migration. Scripts that parse the line need updating.

### Added
- `sj trace --write-baseline` records every current gap and orphan in
  `<spec_root>/trace-baseline.txt`, one sorted `gap <ID>` or
  `orphan <ID> <file> <test>` line each. With a baseline, `sj trace` lists only new
  entries and counts the rest (`3 gaps (2 known)`), reports entries that are no longer
  true as `stale <entry> (remove from trace-baseline.txt)`, and `--strict` fails only
  on new or stale entries. `--json` marks each gap and orphan `"known"` and lists a
  `stale` array.
- `sj gate` reads the baseline from the merge base: known gaps collapse into one
  `known gaps: N (trace-baseline.txt)` warning, known orphans no longer fail, and
  stale entries are `stale <entry>` warnings. Changed scenarios are still gated.
  Entries a change adds fail as `baseline-added <entry>` unless it declares a `config`
  exemption, such as the trailer `Specjutsu-Exempt: config — adopt trace baseline`.
- `sj verify` applies the same baseline rules: known orphans aren't problems, and
  `baseline-added <entry>` is a problem without a `config` exemption.
- `sj ids prefixes` prints a short, collision-free `[ids.prefixes]` table, keeping
  prefixes already mapped or in use (`distribution = "DIST"`). `--write` adds the
  missing lines to `specjutsu.toml` and changes no other byte. A clash it can't
  lengthen away exits 2 naming both capabilities and the prefix. `sj ids` ends with a
  `hint:` line when a derived prefix is longer than six letters.
- `sj migrate --from openspec` pins those short prefixes in `specjutsu.toml` before it
  assigns IDs, so `distribution` gets `DIST-001`.
- `sj migrate --diff` prints a unified diff of every file the preview would create or
  change. The `.claude/settings.json` preview line says how many existing hooks it
  keeps, as `(merged: 1 existing hook kept)`.
- `sj migrate --undo` puts back what the last `sj migrate --write` touched, from an
  undo manifest in the git directory, and keeps every other file. It works after the
  migration is committed and refuses, naming the files, when one was edited since.
- One-line installers: `curl -fsSL https://nabour-labs.github.io/specjutsu/install.sh | sh`
  on macOS and Linux, `irm https://nabour-labs.github.io/specjutsu/install.ps1 | iex` on
  Windows. Both verify the download against `SHA256SUMS`; `SJ_VERSION` pins a release.
- Homebrew: `brew install nabour-labs/tap/specjutsu`, updated by every release.
- `cargo binstall specjutsu` downloads the release binary instead of compiling.
- Releases build `aarch64-pc-windows-msvc`.
- `sj-apply` sends every task to an implementer subagent and then to a fresh reviewer
  subagent that works from the scenario, the diff and `sj verify`, not the implementer's
  report. Fixes go to a fresh re-reviewer for at most 3 rounds; only the main agent runs
  `sj task done`; a whole-change `sj-review` runs at the end.
- Skills ship reference files next to `SKILL.md`: `sj-apply`'s implementer, task-reviewer
  and re-review prompts and `writing-good-tests.md`; `sj-debug`'s
  `root-cause-tracing.md`, `defense-in-depth.md` and `condition-based-waiting.md`. The
  adapters install, update and remove them like `SKILL.md`.
- `sj-debug` works in four phases and stops after 3 failed fixes to question the design.
  `sj-verify` has a gate procedure and treats a subagent's report as a claim, not
  evidence. Both, and `sj-apply`, list rationalizations and red flags.
- `skills/COVERAGE.md` has a Depth column naming the file that carries each Superpowers
  technique; the skill check fails when it's missing.
- `sj gain` totals the bytes `sj test`, `sj verify`, `sj next` and `sj status`
  printed against the raw output they stand in for, per command, with an estimate
  of tokens saved (bytes / 4). Each of those commands appends one record per run
  to `.specjutsu/ledger/gain.jsonl`; `--json`, `--since <time>` and `--reset`.
  The numbers are bytes, the local half of the v0.6 token benchmark.

## [0.5.0] - 2026-11-21

### Changed
- **Breaking:** the default spec root is now `specjutsu/`. `sj init` and `sj adopt`
  write `spec_root = "specjutsu"` and create `specjutsu/specs/` and
  `specjutsu/changes/`. A project with an `openspec/` folder and no `specjutsu/`
  folder keeps using `openspec/` with no edit; `sj adopt` and `sj migrate --from
  openspec` keep that folder too. To switch, `git mv openspec specjutsu` and set
  or remove `spec_root`.

### Fixed
- `sj audit` and `sj gate` no longer report a Rust test as `assertion-free-test` when
  its body holds a string that spans lines, such as a raw `r#"{ … }"#` JSON fixture or
  a `"…\` continued string. The test scanner now keeps string, raw string, char literal
  and block comment state across lines when it looks for the test's closing brace.
- `sj audit` and `sj gate` no longer report a pytest test as `assertion-free-test` when
  its body holds a triple-quoted string with lines at column 0, such as a TOML or INI
  fixture. In JS/TS test files, a `\`-continued string no longer shifts the line numbers
  of the tests after it, which made their assertions miss the audit.
- The Antigravity hook goes under a `specjutsu` hook name in `.agents/hooks.json`,
  the shape Antigravity documents; the earlier `hooks.PreToolUse` entry was never
  read. Reinstalling or removing takes out the old entry.
- Kiro's `.kiro/hooks/specjutsu.json` uses Kiro's hook file schema,
  `{"version": "v1", "hooks": [{"name", "trigger", "matcher", "action"}]}`.
- `sj adopt` creates `<spec_root>/specs/` and `<spec_root>/changes/`, so `sj doctor`
  passes right after adopting.
- `sj gate` reads base spec files in the same order as head, so an open change's
  copy of a scenario wins on both sides. A change merged without being archived
  no longer makes every later pull request fail `green-on-base` on its
  modified scenarios.
- `sj gate` reads base scenarios from the base commit's own spec root. Moving the
  spec folder no longer reports every scenario as added and green on base.

### Added
- The setup skill `skills/specjutsu/SKILL.md`, for agents that don't have `sj` yet:
  install with `install.sh` or `install.ps1`, `sj adopt` or `sj init`,
  `sj init --agents auto`, `sj doctor`.
  `npx skills add nabour-labs/specjutsu --skill specjutsu`.
- Gemini CLI adapter, `sj init --agents gemini`: a `BeforeTool` pre-edit hook
  for `write_file` and `replace` in `.gemini/settings.json`, and `AGENTS.md`
  added to `context.fileName` (a missing setting becomes
  `["GEMINI.md", "AGENTS.md"]`). `auto` picks it for `.gemini/` or `GEMINI.md`.
- Copilot adapter, `sj init --agents copilot`: a `PreToolUse` pre-edit hook in
  `.github/hooks/specjutsu.json` for the Copilot CLI and cloud agent. It always
  exits 0, printing a deny to block and nothing to allow, and checks each file
  of an `apply_patch` patch. `auto` picks it for `.github/copilot-instructions.md`
  or `.github/hooks/`.
- Windsurf and Kiro adapters, `sj init --agents windsurf,kiro`: a
  `pre_write_code` hook in `.windsurf/hooks.json` and a `PreToolUse` hook for
  Kiro's `write` category in `.kiro/hooks/specjutsu.json`. Both answer by exit
  code, 2 to block with the reason on stderr; unreadable input allows with a
  warning. `auto` picks them for `.windsurf/` and `.kiro/`.
- Antigravity adapter, `sj init --agents antigravity`: a `PreToolUse` pre-edit
  hook in `.agents/hooks.json` and the `AGENTS.md` block. `auto` picks it for
  `.agents/rules/` or `.agent/rules/`.
- Codex, Gemini CLI and Antigravity get the `sj-*` skills in `.agents/skills/`,
  beside `.claude/skills/`. `sj update` keeps both copies in step, and removing
  the last of the three removes them, keeping your own skills.
- `--global` on `sj init`, `sj update` and `sj init --remove-agents` installs,
  refreshes or removes only the skills, in `~/.claude/skills/` and
  `~/.agents/skills/`. `sj doctor` prints `skills: project`, `global` or
  `project, global`.
- `sj migrate --from openspec` removes OpenSpec's Antigravity workflows,
  `.agents/workflows/opsx-*.md`, when the frontmatter holds only a
  `description` and the body runs the `openspec` CLI. Other workflows stay.
- `sj migrate` installs every adapter `sj init --agents auto` would install for
  the agent files the project has, alongside the Claude adapter, and the
  preview lists their files.
- `sj spec archive` takes several change names, or `--all-done` for every
  change whose tasks are all done. They archive as one operation: if any one is
  refused, nothing is written. Two changes that edit the same requirement are
  refused, even with `--force`.
- `sj update --dry-run` prints what an update would write, keep or remove, and
  changes nothing. `--json` prints the same plan as JSON.
- `sj completions <shell>` prints a completion script for bash, zsh, fish,
  PowerShell or elvish.
- The `sj-onboard` skill walks a new user through one toy change end to end in a
  scratch change, `sj-onboard-demo`, then removes it. `skills/COVERAGE.md` maps
  OpenSpec's `onboard` to it.

### Changed
- `sj update` removes generated `sj-*` skills this version no longer ships,
  unless you edited them.
- `sj --version` prints the short git commit after the version when `sj` was
  built from a git checkout or with `cargo install --git`, such as
  `sj 0.4.0 (f62a48b)`. A crates.io install prints only the version, as before.

## [0.4.0] - 2026-11-20

### Added
- `sj spec show <item>` prints one scenario, change (open or archived) or
  capability. A scenario comes with its requirement, steps, mapped tests and
  latest ledger evidence. An unknown item suggests near names. `--json` prints
  the same data with a `kind` field.
- `sj view` prints a one-screen project dashboard: capabilities, open changes
  with task and evidence counts, evidence totals, orphan tests and archived
  changes. `--json` prints the same data. It never runs tests.
- Adapters for Codex, Cursor and Cline, and a shared `AGENTS.md` block.
  `sj init --agents` takes a comma-separated list (`claude`, `codex`, `cursor`,
  `cline`, `agents-md`) or `auto`, which installs for the agents the repository
  already has files for. Codex and Cursor get a pre-edit hook
  (`sj hook codex|cursor pre-edit`) with the same red-before-implementation rule
  as Claude Code. Cline gets the `AGENTS.md` instructions only. Removal gives
  every file back byte for byte.
- `sj hook check-edit <path>` makes the pre-edit decision for any other agent or
  editor: exit 0 allows, 2 blocks, with the reason on stderr.
- `sj doctor` prints each installed agent's enforcement level: `hook`,
  `instructions` or `hook missing`.
- `sj instructions <change> [<artifact>]` prints what to write next for a
  change's proposal, specs, design or tasks: the file and template, the project
  context, that artifact's rules, the next free IDs, the language and the check
  command. `--json` prints the same data.
- `sj templates` lists the templates and where each resolves from. A file in
  `<spec_root>/templates/` overrides the built-in, and `sj spec new` uses it.
- Project context and per-artifact rules are read live from `principles.md`,
  and from an unmigrated `openspec/config.yaml` (read-only) after it.
- A `language` key in `specjutsu.toml` (a BCP-47 tag) for specs written in
  another language, with a fallback to `config.yaml`'s. `sj spec validate` warns
  on a translated structural keyword such as `#### Scénario:`.
- A GitHub Action, `uses: nabour-labs/specjutsu@v0`. It installs a pinned `sj`
  verified against `SHA256SUMS`, runs the gate on pull requests, keeps one PR
  comment up to date and writes the job summary. `fail-on: never` reports without
  blocking, and `badge: true` publishes a spec coverage badge to the `sj-badge`
  branch.
- `sj gate --format markdown` prints the gate result as a PR comment body: one row
  per changed scenario with its evidence and a fix hint, failures first, at most
  60,000 bytes. `--format json` is the same as `--json`.
- `sj badge [--out <path>]` writes a shields.io endpoint badge with the share of
  tested scenarios in `specs/`.
- `sj audit --base <ref>` checks a change's tests with no specjutsu setup
  needed. It fails on removed, newly skipped or focused, weakened and
  assertion-free tests, and warns on expected values rewritten next to a source
  change. `--registry` also looks up added dependencies on crates.io, npm and PyPI
  (`unknown-package`, `new-package`). `Specjutsu-Allow:` trailers with a reason
  allow a finding, and `[audit] assert_helpers` adds assertion helpers.
- The action's `command: audit` input runs `sj audit` instead of the gate.
- `sj coverage map` runs each scenario's tests under its suite's new `coverage`
  command and records the source lines they run in `.specjutsu/coverage-map.json`.
  Lines most scenarios run are dropped as shared.
- `sj drift --base <ref>` reports `code-drift` (code a scenario's tests run changed
  while its spec and tests did not) and `stale-test` (a scenario's spec changed
  while its tests did not). A map from an older commit is translated to the base.

### Changed
- `sj gate` also runs the audit's test checks and fails on their failures, for
  example a test that gained `#[ignore]` or a new test that asserts nothing.
- `sj gate` reports spec drift as warnings, as failures with `[drift] fail = true`,
  and as `info` lines in a `refactor` change.

### Fixed
- The pre-edit hooks and `sj hook check-edit` judge a file by the git work tree
  that holds it. An edit in a linked worktree was judged by the main checkout's
  active task (blocking on another change's scenarios), and one in a worktree
  outside the main checkout was never checked.

## [0.3.1] - 2026-11-18

### Added
- Releases publish `specjutsu`, `sj-core`, `sj-runners` and `sj-adapters` to
  crates.io, so `cargo install specjutsu` works. CI checks that every crate builds
  from its own package.

### Changed
- `sj-adapters` embeds a copy of `skills/` and `adapters/claude/` under
  `crates/sj-adapters/assets/` so it builds outside the repository.
  `scripts/sync-adapter-assets.sh` refreshes it, and a test fails when it is stale.

## [0.3.0] - 2026-11-18

### Changed
- The `cargo` suite written by `sj init`, `sj adopt` and the built-in default
  uses `tests = ["**/*.rs"]`, so `sj trace` keeps finding inline `#[cfg(test)]`
  tests. A `specjutsu.toml` that lists no packages now traces no tests.
- **Breaking:** `sj gate` fails closed. Every changed file is source unless it is a
  test, spec, documentation, media, repository metadata or a manifest/lockfile, so
  editing e.g. a shell script or a `.ts` file now needs a scenario or an exemption.
- `sj gate` fails a new or edited scenario without an ID (`unidentified`), and
  warns once with the count of existing ones.
- `sj gate --base` exits 2 when no test suite is configured (no suites in
  `specjutsu.toml` and no root `Cargo.toml`) instead of passing, and suggests
  `sj adopt`.
- With `specjutsu.toml`, a package's `sources` decide which of its files need a
  scenario. Files in the package outside them no longer do.
- `sj gate` runs the full suite at head for every exemption kind when source
  files change, not only for `refactor`.

### Added
- `sj migrate --from openspec|speckit|superpowers [--write]` replaces those tools
  in a project. It is a dry run by default, and the write needs a clean git tree.
  - **OpenSpec:** specs and history stay, scenarios get IDs, and `config.yaml`
    moves to `principles.md`. Generated skills and `/opsx` commands are removed
    by their metadata.
  - **Spec Kit:** the features are imported, then `.specify/`, `specs/` and the
    manifest-listed agent files are removed.
  - **Superpowers:** its plugin, marketplace and permission rules leave
    `.claude/settings.json`, keeping project hooks.

  Every run ends by adopting the project, installing the Claude Code adapter, and
  running `sj doctor` and `sj spec validate`.
- `sj import speckit [--write] [--force] [--json]` converts Spec Kit features into
  the OpenSpec layout. A finished feature becomes a current spec plus an archived
  change, and an open one becomes an open change. User stories and FR lines become
  requirements, and acceptance items become scenarios with new IDs and verbatim
  GIVEN/WHEN/THEN steps. FRs without a scenario are listed as gaps and marked for
  clarification. The other files are copied byte for byte, the remaining spec
  sections go to `proposal.md`, and the constitution becomes `principles.md`. It
  is a dry run by default and never touches Spec Kit's own files.
- Claude Code adapter: `sj init --agents claude` writes a CLAUDE.md block, the
  `sj-*` skills under `.claude/skills/` and `SessionStart` / `PreToolUse` hooks in
  `.claude/settings.json`, keeping existing content, order and formatting.
  `sj update [--force]` regenerates it idempotently and keeps hand-edited skills.
  `sj init --remove-agents claude` restores the files byte for byte.
- `sj hook claude session-start` prints the status bootstrap (at most 1,200
  bytes). `sj hook claude pre-edit` blocks source edits in guided and strict modes
  until the active task's scenarios have a red ledger record, always allows tests,
  specs and docs, and never blocks on its own errors.
- Nine agent skills in `skills/` (`sj-explore`, `sj-propose`, `sj-clarify`,
  `sj-principles`, `sj-apply`, `sj-debug`, `sj-review`, `sj-verify`, `sj-finish`)
  covering every Superpowers skill, OpenSpec workflow command and Spec Kit command,
  mapped in `skills/COVERAGE.md`. `sj_adapters::skills::check` keeps them
  well-formed: `name` and a `Use when` description, bodies of at most 6,000 bytes,
  no foreign commands, each workflow's required `sj` commands, the principles rule
  for planning and review, and complete coverage.
- `sj status [<change>]`: per open change, at most five lines with tasks done,
  added and modified scenarios counted as proven, green-unproven, red and
  untested from the ledger, and the next task.
- `sj next [<change>] [--parallel] [--base <ref>]`: the first open task, the
  full text of its scenarios, files changed since the merge-base, the
  principles file and one `sj test <ID>` per scenario, within 1,600 bytes besides
  the scenario texts. `--parallel` batches open `[P]` tasks whose tests share no
  file.
- `sj task done <change> <task>` checks one box in `tasks.md` and changes no other
  byte. Strict mode refuses (exit 1) unless the ledger proves red-before-green for
  each of the task's scenarios. Guided and advisory modes warn.
- `sj verify [<change>] [--base <ref>]`: runs and records the change's scenario
  tests, then reports untested, failing and unproven scenarios, orphans,
  `sj spec check` problems, and the gate's unspecified-change and exemption rules
  against the merge-base. It skips red-on-base.
- `sj spec new | list | validate | check | archive`: scaffold a change (kebab-case
  name, next free ID per prefix), list open changes with task progress, validate
  spec format, delta structure and task-to-scenario links (`--strict` fails on
  warnings), report uncovered scenarios, unknown task IDs and
  `[NEEDS CLARIFICATION: …]` markers, and archive a change. Archive refuses
  validation errors and open tasks unless `--force`. It merges deltas the way
  `openspec archive` 1.14.0 does and writes the same bytes, and writes nothing
  when any delta fails to merge.
- `sj ids [--write] [--json]` lists scenarios without IDs with the ID each would
  get (exit 1 when there are any) and, with `--write`, inserts them in place,
  changing no other byte. Prefixes come from the capability folder or from
  `[ids.prefixes]` in `specjutsu.toml`. Numbers continue after the highest ever
  used, `changes/archive/` included. A delta copy of a `specs/` scenario gets the
  same ID. Two capabilities with the same prefix exit 2 and write nothing.
- `sj gate` treats a scenario that only gained an ID as unchanged, so an ID-only
  commit passes. `--json` gains `changes`: every scenario that isn't unchanged,
  with its kind.
- `sj gate` proves red-on-base for vitest, Jest, Playwright, pytest and `junit`
  suites. It overlays changed files matching each suite's `tests` and `support`
  globs on base, runs the suite's `setup` there first, and records a test file that
  fails to load as `red:compile`. Suites can set `red_on_base = "skip"` with a
  `reason`, reported as `unverifiable`. The head run covers every suite. A top-level
  `non_source` list extends the built-in non-source files.
- `sj test` runs vitest, Jest, Playwright, pytest and generic `junit` suites from
  `specjutsu.toml`, reading results from JUnit XML. Scopes: a scenario ID (its
  tests in every suite), a package path, or `<package>:<runner>`. Suites run in
  their package directory through the project's package manager (`pnpm exec`,
  `yarn`, `npx --no-install`) or `uv run`, and accept `command` and `env`. A suite
  without a report, or a test file that fails to load, is a build failure. The
  summary names failed suites, and `--json` gains `suites`.
- `sj trace` finds vitest, Jest, Playwright and pytest tests through each suite's
  globs in `specjutsu.toml`, linked by title, `describe` title, `// spec:` /
  `# spec:` comment, pytest function name or `@pytest.mark.spec`. Test entries in
  `--json` gain `package`, `suite` and `skipped`, the report gains `warnings`, and a
  package with sources but no suite prints `warn    no test suite: <path>`.
- `specjutsu.toml`: the project file, with `schema`, `spec_root`, `mode` and
  packages with test suites. Every command reads it, rejects unknown keys and
  runners (exit 2), and `--spec-root` overrides it. Without one, behaviour is
  unchanged.
- `sj init [--force]`: write `specjutsu.toml` and create the spec folders.
- `sj adopt [--dry-run] [--force]`: detect Cargo, pnpm, npm, yarn and
  `pyproject.toml` packages and the `cargo`, `vitest`, `jest`, `pytest` and
  `playwright` runners, and write them to `specjutsu.toml`.
- `sj doctor`: report config errors, packages without a runner, missing runner
  executables and missing spec folders; exit 1 when it finds any.
- `sj test [SCOPE]`: run `cargo test` for the workspace, one package or one
  scenario's mapped tests; failures-only output with scenario IDs and excerpts;
  build failures reported as such; `--json`; exit codes 0/1/2.
- Evidence ledger: every `sj test` run is appended to
  `.specjutsu/ledger/runs.jsonl` with the working-tree hash (`--no-ledger` to skip).
- `sj ledger [ID]`: per-scenario latest result and red-before-green proof from
  recorded runs; `--json`.

## [0.2.0] - 2026-11-13

### Added
- Cargo workspace skeleton (`sj-cli`, `sj-core`, `sj-runners`, `sj-adapters`).
- `sj trace`: map OpenSpec scenario IDs to Rust tests and report gaps.
- `sj gate --base <ref>`: CI gate with red-on-base (new tests must fail on the base
  code and pass on head), exemptions, `--json`, exit codes 0/1/2.
- `sj gate --since <commit>`: check every `feat(ID)` commit has an earlier `test(ID): red`.
- `cargo test` runner with libtest result parsing.

[Unreleased]: https://github.com/nabour-labs/specjutsu/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/nabour-labs/specjutsu/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/nabour-labs/specjutsu/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/nabour-labs/specjutsu/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/nabour-labs/specjutsu/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/nabour-labs/specjutsu/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/nabour-labs/specjutsu/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/nabour-labs/specjutsu/releases/tag/v0.2.0
