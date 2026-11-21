# specjutsu

**Master the art of the spec.**

**The quality gate for AI-generated code.** specjutsu stops your coding agent from
shipping work it can't prove: every spec scenario gets a test, every test must fail
before the code exists, and the pull request is refused until both hold. It's one
open-source binary and one GitHub Action, with no LLM in the loop: every check is
deterministic, so CI gives the same answer every time.

```yaml
# .github/workflows/specjutsu.yml
on: pull_request
jobs:
  gate:
    runs-on: ubuntu-latest
    permissions: { contents: read, pull-requests: write }
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - uses: nabour-labs/specjutsu@v0
```

Every PR gets one comment, updated on each push, listing the scenarios the change
touches, the evidence for each (`red:assert`, `red:compile`, `untested`,
`green-on-base`, …) and how to fix what fails. See [The GitHub Action](#the-github-action).

**No specs yet?** Start with `command: audit`. It needs no setup and catches the ways
an agent makes checks pass without doing the work: deleted, skipped or focused
tests, lost assertions, and dependencies that don't exist. See
[Works without specs](#works-without-specs-sj-audit).

Under the Action is `sj`, a CLI that covers the whole loop: authoring (`sj spec`),
the agent's working loop (`sj status`, `next`, `verify`), the proof layer
(`sj trace`, `sj test`, `sj gate`), a Claude Code adapter with nine skills, and
`sj migrate`. Its spec format is OpenSpec's, so existing `openspec/` folders work
unchanged, and it replaces the OpenSpec, Spec Kit and Superpowers stack.

**Best in class for spec-driven development.** specjutsu aims to be the best tool for
spec-driven development (SDD), and it is the only one that proves the work. OpenSpec, Spec
Kit, Kiro, Superpowers and BMAD stop at writing the spec, or ask a model whether the code
matches it. specjutsu checks it deterministically:

| | OpenSpec | Spec Kit | Kiro | Superpowers | **specjutsu** |
| --- | --- | --- | --- | --- | --- |
| Scenario → test trace checked | No | No | Partly (property tests, in the IDE) | No | **Yes, `sj trace`** |
| Test must fail before the code | No | No | No | By prompt | **Enforced, then proven in CI** |
| PR blocked until proof holds | No | No | No | No | **Yes, GitHub Action** |
| Test-tampering checks | No | No | No | No | **Yes, `sj audit`** |
| How the result is checked | LLM | LLM | LLM + optional tests | Prompt | **Deterministic, no LLM** |

specjutsu is not ahead everywhere yet: OpenSpec parity (more agents, custom workflow
schemas, multi-repo spec stores) is planned for v0.6 and v0.7, and bugfix specs,
specs inferred from existing tests and property scenarios go further. See
[How specjutsu compares](docs/competitive-analysis.md) for the full comparison, its
sources, and the change that closes each gap.

**Website:** [nabour-labs.github.io/specjutsu](https://nabour-labs.github.io/specjutsu/) ·
**[User manual](https://nabour-labs.github.io/specjutsu/manual.html)**: install, your first
change, CI setup, agent setup and the command reference.

**Status:** pre-alpha, v0.4.0. `sj trace`, `sj test` and `sj gate` work with
Rust, vitest, Jest, Playwright and pytest, and any runner that writes JUnit XML.
Adapters install into Claude Code, Codex, Cursor, Cline and any agent that reads
`AGENTS.md`. See the [design doc](docs/specjutsu%20%E2%80%94%20Project%20Kickoff%20%26%20Design.md)
and [analysis](docs/analysis.md).

## Install

Prebuilt binaries for macOS, Linux and Windows are on the
[releases page](https://github.com/nabour-labs/specjutsu/releases). Or with Cargo,
from [crates.io](https://crates.io/crates/specjutsu) or from the git tag:

```bash
cargo install specjutsu
cargo install --git https://github.com/nabour-labs/specjutsu --tag v0.4.0 specjutsu
```

To update, install again with `--force`. To get a fix that is merged but not yet
released, build from `main`. Then run `sj update` in each project:

```bash
cargo install specjutsu --force
cargo install --git https://github.com/nabour-labs/specjutsu --branch main specjutsu --force
cargo install --list | grep -A1 specjutsu   # the installed build and its source
```

`sj --version` shows only the version number, which a build from `main` shares
with the last release. Open agent sessions use the new binary on their next hook
call.

## Quickstart

```bash
sj trace            # map scenario IDs in openspec/ to tests; list gaps
sj trace --json     # machine-readable
sj gate --base origin/main   # CI: changed scenarios tested, red on base, green on head
sj test             # run the tests; print failures only; record the run
sj test GATE-004    # run only the tests mapped to one scenario (or pass a package name)
sj ledger           # per scenario: latest result, red-before-green proven or not
sj init             # new project: write specjutsu.toml, create openspec/specs and changes
sj adopt --dry-run  # existing project: detect packages and runners, print the config
sj doctor           # check the config, runner executables and spec folders
sj ids              # list scenarios without IDs and the IDs they would get
sj ids --write      # assign them in place
sj spec new add-oauth-login  # scaffold a change; prints the next free ID per prefix
sj spec list        # open changes: tasks done, scenarios added or modified
sj spec show AUTH-003   # one scenario, change or capability; a scenario with its tests and evidence
sj spec validate    # spec format, delta structure, every task names its scenarios
sj spec check add-oauth-login    # uncovered scenarios, unknown IDs, open questions
sj spec archive add-oauth-login  # merge the deltas into specs/, move to changes/archive/
sj status           # each open change: tasks, scenario evidence, the next task
sj view             # one-screen dashboard: capabilities, open changes, evidence totals
sj next             # the next task: its scenarios, changed files, test commands
sj task done add-oauth-login 2.1  # check the box (strict mode needs red-before-green)
sj verify           # before a PR: what the gate checks, without red-on-base
sj init --agents claude  # install the Claude Code adapter: CLAUDE.md block, skills, hooks
sj init --agents auto    # install for every agent the repo already has files for
sj update           # regenerate installed adapters; hand-edited files are kept
sj import speckit   # preview converting Spec Kit's specs/NNN-*/ features; --write converts
```

## How tracing works

Give every scenario a stable ID:

```markdown
#### Scenario: AUTH-003 Reject expired token
- **WHEN** the client calls GET /me with a token issued 25 hours ago
- **THEN** the response is 401 with code TOKEN_EXPIRED
```

Reference it from a test, by name or by comment:

```rust
#[test]
fn auth_003_rejects_expired_token() { /* ... */ }

// spec: AUTH-003
#[test]
fn rejects_expired() { /* ... */ }
```

In vitest, Jest and Playwright, put the ID in the test title, in an enclosing
`describe` title (it then covers every test inside), or in a `// spec:` comment:

```ts
it("AUTH-003 rejects expired token", () => { /* ... */ })

describe("AUTH-004 refresh", () => {
  it("issues a new token", () => { /* ... */ })
})

// spec: CHAT-002, CHAT-003
test("sends a message", async ({ page }) => { /* ... */ })
```

In pytest, use the function name, a marker, or a `# spec:` comment above the
function or its `Test*` class:

```python
def test_inv_002_removes_item(): ...

# spec: INV-003
class TestDiff:
    @pytest.mark.spec("INV-004")
    def test_added(self): ...
```

`sj trace` reports which scenarios are covered, which have no test, and which
tests point at scenarios that no longer exist. It finds test files through each
suite in `specjutsu.toml` (see [The project file](#the-project-file)), so each
test in `--json` carries its `package` and `suite`, and `skipped` for
`.skip`/`.todo`/`.fixme` and `@pytest.mark.skip`. A package with source files but
no suite gets a `warn    no test suite: <path>` line. Titles built at runtime
(variables, `${}` templates) are not read. Use a `// spec:` comment for those.

## How the gate works

`sj gate --base origin/main` fails a change when an added, modified or newly
implemented scenario has no test, a test fails, or a test points at a missing
scenario. Its core check is **red-on-base**: it checks out the base commit,
overlays only the change's test files, and requires each new test to fail
there. It does this for every suite in `specjutsu.toml`: Cargo `tests/` files, and
for other runners every changed file matching the suite's `tests` or `support`
globs, so a co-located `auth.test.ts` runs against the base `auth.ts`. A test that passes on the old code didn't drive the change. Refactors and
chores declare an exemption with a reason (`Exempt: refactor — <why>` in the
change's `proposal.md`). An exempted change that edits source still has to pass
the full test suite.

The gate fails closed. Any changed file counts as source, so it needs a scenario
or an exemption, unless it is a test, a spec, documentation or media (`*.md`,
`*.txt`, images), repository metadata (`.github/`, `.gitignore`, `LICENSE`), or a
manifest or lockfile (`Cargo.toml`, `package.json`, `pyproject.toml`, …). A new
or edited scenario without an ID fails as `unidentified`. Existing scenarios
without IDs are counted in one warning. With no test suite configured (no
`specjutsu.toml` suites and no root `Cargo.toml`), the gate exits 2 and suggests
`sj adopt` instead of passing changes it can't test.

A base checkout has no installed dependencies, so a suite that needs them declares
`setup`. The gate runs it once in the base package directory before that suite's
base run, and a failing setup exits 2 with its last 20 lines. A suite that can't run
on old code at all, such as end-to-end tests against a deployed API, says so:

```toml
[[package.suite]]
runner = "vitest"
tests = ["apps/api/src/**/*.test.ts"]
setup = "pnpm install --frozen-lockfile"
support = ["apps/api/test/fixtures/**"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]
red_on_base = "skip"
reason = "needs staging API"
```

A new test in a skipped suite fails as `unverifiable CHAT-002 sends a message
(needs staging API)` unless the gate runs with `--allow-unverifiable`. Its head run
is never skipped. A test file that fails to load on base, for example because it
imports a function that doesn't exist yet, counts as `red:compile`.

When a package declares `sources`, only changed files matching them need a scenario,
and files elsewhere in that package don't. A top-level `non_source = ["infra/**"]`
adds globs to the built-in non-source list.

## The GitHub Action

`uses: nabour-labs/specjutsu@v0` installs a pinned `sj` release (checked against the
release's `SHA256SUMS`), runs `sj gate` against the pull request's base, writes the
result to the job summary and keeps one PR comment up to date. It runs on Linux,
macOS and Windows runners. The checkout needs `fetch-depth: 0` so the base commit is
there, and the job needs the toolchains your suites use (Rust, Node, uv) to run
tests on both sides.

| Input | Default | What it does |
| --- | --- | --- |
| `command` | `gate` | `audit` runs `sj audit --registry` instead, for repositories without specs |
| `version` | the release the action is tagged with | `sj` version to install |
| `base` | `origin/<PR base branch>` | ref to compare against |
| `comment` | `true` | post and update the PR comment; `false` writes only the job summary |
| `fail-on` | `gate` | `never` reports without failing the step, to try specjutsu without blocking merges |
| `badge` | `false` | on pushes to the default branch, publish `badge.json` to the `sj-badge` branch |
| `github-token` | `github.token` | token for the download, the comment and the badge push |

The comment needs `pull-requests: write`. On pull requests from forks the default
token is read-only, so the action warns and leaves the result in the job summary.
An environment error (gate exit 2) always fails the step.

The same output works in any CI: `sj gate --format markdown` prints the comment
body (at most 60,000 bytes, failures first), and `sj gate --json` the data.

### Spec coverage badge

`sj badge` writes a [shields.io endpoint](https://shields.io/badges/endpoint-badge)
file: scenarios in `specs/` with at least one test that isn't skipped, out of all of
them, coloured green from 90%, yellow from 70% and red below. Turn on `badge: true`
in a workflow that runs on pushes to the default branch with `contents: write`, then
add to your README:

```markdown
![specs](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/<owner>/<repo>/sj-badge/badge.json)
```

The badge says *tested*, not *proven*: red-before-green is checked per change by the
gate, not for the whole project at once.

## Works without specs: `sj audit`

`sj audit --base origin/main` compares the change's tests with the base and needs
no `specjutsu.toml` and no spec folder. Without a config it finds tests with
default globs (`*.rs` with `#[test]`, `*.test.ts`/`*.spec.js` and friends,
`test_*.py`/`*_test.py`); with one it uses each suite's globs. It reports:

| Finding | What happened | Severity |
| --- | --- | --- |
| `removed-test` | a test was deleted (a test moved or renamed with the same body is not) | fail |
| `skipped-test` | a skip marker was added: `#[ignore]`, `.skip`, `.todo`, `xit`, `@pytest.mark.skip`, `pytest.skip(` … | fail |
| `focused-test` | a focus marker was added (`.only`, `fit`, `fdescribe`); reported once per `describe` | fail |
| `weakened-test` | a test has fewer assertions than on base, e.g. `(5 → 2 assertions)` | fail |
| `assertion-free-test` | a new test asserts nothing | fail |
| `rewritten-expectation` | an expected literal changed in a change that also edits source, e.g. `108 → 110` | warn |
| `unknown-package` | with `--registry`: an added dependency doesn't exist on crates.io, npm or PyPI | fail |
| `new-package` | with `--registry`: an added dependency was first published under 30 days ago | warn |

Assertions are counted per language: `assert*!`, `panic!` and `#[should_panic]`
in Rust; `expect(` and `assert(` in JS/TS; `assert`, `pytest.raises` and
`self.assert*` in Python. Add your own helpers in `specjutsu.toml`:

```toml
[audit]
assert_helpers = ["assert_snapshot", "expect_json"]
```

When a finding is intended, allow it in a commit trailer with a reason. The audit
then lists it as `allowed` instead of failing:

```text
Specjutsu-Allow: removed-test tests/test_export.py › test_legacy_export — legacy export removed in #212
```

`--registry` is the only part that uses the network. Without it the audit makes
no requests, and an unreachable registry only warns. `sj gate` runs the test
checks (not the registry ones) on every change and fails on their failures. The
output and exit codes match the gate's (`--json`, `--format markdown`; 0 pass,
1 fail, 2 error).

## Spec drift

The gate proves the scenarios a change touches. Drift is about the ones it
doesn't touch: an agent fixes a bug in `src/pricing.rs`, which also decides how
discounts work, and the discount scenarios now describe behaviour the code no
longer has. specjutsu finds that without reading the code: coverage says which
lines each scenario's tests run, and the diff says which of them changed.

```bash
sj coverage map              # run each scenario's tests under coverage; write .specjutsu/coverage-map.json
sj drift --base origin/main  # what the change did to code its scenarios own
```

`sj drift` reports two things:

- `code-drift AUTH-003 src/auth.rs:15` means lines AUTH-003's tests run changed,
  but AUTH-003's spec and the files holding its tests did not.
- `stale-test AUTH-002` means AUTH-002's spec text changed but none of its tests
  did. This one needs no map.

Lines that more than half of the mapped scenarios run (setup, `main`, shared
helpers) are dropped from the map as shared, so they don't flag everything. A map
built on an older commit is translated to the base through `git diff`, and lines
changed in between are dropped. Without a map, `sj drift` runs only `stale-test`
and says so. `sj gate` reports drift as warnings. With `[drift] fail = true`
they fail it, and in a change exempted as `refactor` code drift is only `info`.

```toml
[drift]
fail = true             # drift findings fail the gate (default false)
shared_threshold = 0.5  # share of scenarios above which a line is shared
```

Each suite that takes part declares a `coverage` command. `{lcov}` is the LCOV
file to write and `{tests}` the scenario's tests in the runner's own syntax
(test names for cargo, files for vitest and Jest, `file::name` for pytest):

```toml
[[package.suite]]
runner = "cargo"
coverage = "cargo llvm-cov --lcov --output-path {lcov} -- {tests}"

[[package.suite]]
runner = "vitest"
coverage = "npx vitest run --coverage.enabled --coverage.reporter=lcov --coverage.reportsDirectory=.sj-cov {tests} && mv .sj-cov/lcov.info {lcov}"

[[package.suite]]
runner = "jest"
coverage = "npx jest --coverage --coverageReporters=lcov --coverageDirectory=.sj-cov {tests} && mv .sj-cov/lcov.info {lcov}"

[[package.suite]]
runner = "pytest"
coverage = "uv run pytest --cov --cov-report=lcov:{lcov} {tests}"
```

The map runs each scenario's tests once, so build it on the default branch and
reuse it on pull requests:

```yaml
# On pushes to main
- run: sj coverage map
- uses: actions/cache/save@v4
  with:
    path: .specjutsu/coverage-map.json
    key: sj-coverage-${{ github.sha }}

# On pull requests, before the gate
- uses: actions/cache/restore@v4
  with:
    path: .specjutsu/coverage-map.json
    key: sj-coverage-${{ github.event.pull_request.base.sha }}
    restore-keys: sj-coverage-
```

A restored map from a commit that isn't an ancestor of the base is not used, and
`sj drift` says so.

## The project file

`specjutsu.toml` at the repository root says where the specs are and which
packages and test suites the project has. Without one, every command uses
`openspec/` and, when there is a root `Cargo.toml`, one Cargo suite at the root.

Nobody should write it by hand. `sj init` writes it for a new project. `sj adopt`
reads an existing one's manifests (Cargo, pnpm, npm and yarn workspaces,
`pyproject.toml`) and config files, and never runs a tool. It detects `cargo`,
`vitest`, `jest`, `pytest` and `playwright`. For a pnpm monorepo with a uv
service, `sj adopt --dry-run` prints:

```toml
schema = 1
spec_root = "openspec"
mode = "guided"

[[package]]
path = "apps/app"
sources = ["apps/app/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/app/**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]

[[package]]
path = "python/worker"
sources = ["python/worker/**"]

[[package.suite]]
runner = "pytest"
tests = ["python/worker/**/test_*.py", "python/worker/**/*_test.py"]
```

Paths and globs are relative to the repository root and use forward slashes on
every OS. A file belongs to the package with the longest matching path. A
package with no runner is written with no suites, and adopt reports it as
`no runner detected: <path>`. Both commands refuse to replace an existing file
without `--force`.

Every command reads the file first. An unknown key or runner exits 2 and names
the problem. `--spec-root` still overrides `spec_root`. `sj doctor` lists config
errors, packages without a runner, runners that are not installed, and missing
spec folders, one per line. It never installs anything: pytest is checked on
`PATH` or with `uv run --no-sync`, and Node runners in `node_modules/.bin`.

`sj trace`, `sj test` and `sj gate` read the packages and suites.

### Spec language

`language = "fr"` (any BCP-47 tag, such as `pt-BR`) says the specs are written in
that language. `sj instructions` then asks for prose in it, with the structural
keywords kept in English: `### Requirement:`, `#### Scenario:`, `WHEN`, `THEN`,
`SHALL`, `MUST`, `## Purpose` and the delta section headers. The parsers only see
those English words. `sj spec validate` warns at a heading that looks translated,
such as `#### Scénario:`. An invalid tag exits 2. When `specjutsu.toml` sets no
`language`, an unmigrated `openspec/config.yaml`'s `language` is used.

## Adding IDs to existing scenarios

Trace and gate only see scenarios with an ID. `sj ids` lists every scenario
without one and the ID it would get, and exits 1 when there are any.
`sj ids --write` inserts them:

```
$ sj ids --write
2 scenarios without IDs
USERAUTH-001 Valid registration (openspec/specs/user-auth/spec.md:6)
USERAUTH-002 Password too short (openspec/specs/user-auth/spec.md:10)
wrote 2 IDs to 1 file
```

- **Only the headings change.** The ID goes before the scenario name, and every
  other byte, line endings included, stays as it was. Running it again changes
  nothing.
- **Prefixes come from the capability folder:** its letters, uppercased
  (`user-auth` → `USERAUTH`). Set your own in the project file:

  ```toml
  [ids.prefixes]
  user-auth = "AUTH"
  ```

  If two capabilities that need IDs end up with the same prefix, nothing is
  written and the command exits 2 naming both. Map one of them to fix it.
- **IDs are never reused.** Numbers continue after the highest one used for that
  prefix anywhere under the spec root, `changes/archive/` included, so keep the
  archive. Archived changes are read, never written.
- **A delta copy shares its spec's ID.** A scenario in an open change with the
  same requirement and scenario name as one in `specs/` gets the same ID.

The gate treats a scenario that only gained an ID as unchanged, so the ID commit
can land on its own: its scenarios show as `gap` warnings, not failures. Adding a
test in the same change makes the scenario `implemented`, which still needs
red-on-base. `sj gate --json` lists every scenario that isn't unchanged under
`changes`.

## Writing and archiving changes

`sj spec` does what `openspec new`, `validate` and `archive` do, plus Spec Kit's
*analyze* and *clarify* passes. The format is OpenSpec's, so existing `openspec/`
folders work unchanged.

- **`sj spec new <change>`** creates `changes/<change>/` with `proposal.md`,
  `design.md`, `tasks.md` and an empty `specs/`. The name must be kebab-case. It
  prints the next free ID for every prefix in use.
- **`sj instructions <change> [<artifact>]`** tells an agent what to write
  next: the file and its template, the project context, the rules for that
  artifact, the next free ID per prefix, the language and the command that checks
  it. With no artifact it picks the first of `proposal`, `specs`, `design` and
  `tasks` that is missing or still the template. `--json` prints the same data.
- **`sj templates`** lists the four templates (`proposal.md`, `spec.md`,
  `design.md`, `tasks.md`) and where each comes from. A file in
  `<spec_root>/templates/` overrides the built-in, for `sj spec new` and
  `sj instructions` alike.
- **`sj spec list`** prints one line per open change:
  `add-oauth-login  3/7 tasks  4 scenarios`.
- **`sj spec show <item>`** prints one item, looked up in this order: a
  scenario ID, an open change, an archived change (by its folder name, or by
  its bare name when only one archived change has it), a capability. A
  scenario prints with its requirement, its steps, the tests mapped to it and
  its latest ledger result, and is found in open changes too. A change prints
  its proposal, each delta under a `--- <capability>` header and its task
  count, or `archived <date>`. A capability prints its `spec.md` byte for
  byte. An unknown item exits 1 and suggests up to three near names. `--json`
  prints the same data with a `kind` field.
- **`sj spec validate [<change>]`** reports each problem with its file and line:
  - A scenario without an ID, a duplicate ID, or a missing WHEN or THEN step.
  - A requirement with no scenario, or with no SHALL or MUST.
  - A delta requirement outside `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`.
  - A REMOVED requirement without `**Reason**` and `**Migration**`.
  - A MODIFIED, REMOVED or RENAMED name that the current spec doesn't have.
  - A task that names no scenario ID. Mark tasks that cover none with
    `(no-scenario: <reason>)`, for example docs.

  It exits 1 on errors. `--strict` also fails on warnings, such as a placeholder
  Purpose.
- **`sj spec check <change>`** lists scenarios no task names (`uncovered
  AUTH-006`), tasks naming IDs nobody defines, and every
  `[NEEDS CLARIFICATION: …]` marker outside code.
- **`sj spec archive <change>`** refuses while validation fails or a task is
  open, unless you pass `--force`. Then it applies RENAMED, REMOVED, MODIFIED and
  ADDED requirements to `specs/` and moves the change to
  `changes/archive/YYYY-MM-DD-<change>/`. A new capability gets the delta's
  `## Purpose`.

  The merge follows `openspec archive` 1.14.0 rule for rule, and writes the same
  bytes. Like OpenSpec, it writes LF line endings and one blank line between
  requirements. A MODIFIED block that would drop a scenario is an error. Nothing
  is written unless every delta merges.

### Project context and rules

`principles.md` in the spec root is read on every run. Its text before
`## Rules` is the project context, and each `### <artifact>` section under
`## Rules` holds that artifact's rules: `### tasks` rules show only in
`sj instructions <change> tasks`. A project that still has OpenSpec's
`openspec/config.yaml` keeps its `context` and `rules`: they are read, never
written, and listed after `principles.md`'s, each under its file name.
`sj migrate --from openspec` is still the way to move them over.

## Test runs and the evidence ledger

`sj test` runs every suite in `specjutsu.toml` (or `cargo test` when there is
none) and prints one summary line when everything passes. When something fails, it lists each failing test with its scenario IDs
and the first 20 lines of its output. A compile error is reported as
`build failed` with the compiler's error lines, never as an empty run. Exit
codes match the gate: 0 pass, 1 fail, 2 error. Pass `--json` for tools.

Scopes work across runners:

```bash
sj test                    # every suite
sj test AUTH-003           # only the tests linked to AUTH-003, in every suite that has one
sj test apps/api           # every suite in a package
sj test apps/app:playwright  # one suite
```

Each suite runs in its package directory, with a JUnit report written to a
temporary file:

| Runner | Runs as |
| --- | --- |
| `vitest`, `jest`, `playwright` | `pnpm exec`, `yarn` or `npx --no-install`, from the nearest lockfile |
| `pytest` | `uv run pytest` when there is a `uv.lock`, otherwise `python3 -m pytest` |
| `junit` | your `command`, which writes the declared `report` |

`command` replaces the launcher of a built-in runner (sj still adds its reporter
and selection arguments), and `env` adds environment variables. Jest needs the
`jest-junit` package in the project. A suite that exits without a report is a
build failure, shown with the last lines of its output, and so is a test file
that fails to load. `sj test` never installs anything. With more than one suite,
the summary names the suites that failed.

Each run appends one line per test to `.specjutsu/ledger/runs.jsonl`
(gitignored, local only). Each line holds the run ID, the time, the git tree hash
of the working tree, the test, its scenario IDs and the result. `sj ledger`
reads it back. A scenario is **proven** when a failing run was recorded before
a passing one, on different code. Pass `--no-ledger` to skip recording.

## The agent loop

These commands answer the agent's questions (where am I, what next, am I done)
in a few lines, so it never reads whole spec folders or raw test logs. Each
takes `--json`, except `sj task done`.

- **`sj status [<change>]`** prints at most five lines per open change. The
  first line gives tasks done and the change's added or modified scenarios by
  evidence. One line lists the IDs in each state that isn't proven. The last
  line names the next task:

  ```
  add-oauth-login  3/7 tasks  4 scenarios: 2 proven, 1 red, 1 untested
    red AUTH-005
    untested AUTH-006
    next 2.1 Refresh tokens (AUTH-005, AUTH-006)
  ```

  **proven** means red-before-green in the ledger with a passing latest run.
  **green-unproven** passed but was never seen failing. **red** means the latest
  run failed. **untested** means nothing is recorded.
- **`sj next [<change>]`** prints the first open task, the full WHEN/THEN text
  of each scenario it names, the files changed since the merge-base,
  `<spec_root>/principles.md` when there is one, and a `sj test <ID>` command
  per scenario. Apart from the scenario texts it stays under 1,600 bytes.
  `--parallel` lists the open tasks marked `[P]` in batches whose tests share no
  file.
- **`sj task done <change> <task>`** checks that task's box and changes no
  other byte. In `strict` mode it refuses, exiting 1, unless every scenario the
  task names is proven: `unproven AUTH-005 (no red run recorded)`. In `guided`
  and `advisory` modes it checks the box and prints the same lines as warnings.
- **`sj view`** prints a one-screen project dashboard: each capability with
  its scenario count, each open change with its tasks and scenario evidence
  (the counts `sj status` uses), evidence totals over every scenario, the
  number of orphan tests and of archived changes. It fits in 40 lines, cutting
  long lists with `… +N more`. It reads files and the ledger only, and never
  runs tests.
- **`sj verify [<change>]`** is the local check before a PR. It runs the
  change's scenario tests (recording them), and reports untested and failing
  scenarios, unproven ones, orphans, `sj spec check` problems, and the gate's
  unspecified-change and exemption rules against the merge-base. Red-on-base
  stays in CI. It prints `verify: ok` or one line per problem, and exits 1 when
  there are any.

`sj next` and `sj verify` compare against `--base`, or else the first of
`origin/HEAD`, `origin/main`, `origin/master`, `main` and `master` that exists.

## Skills

`skills/` holds nine agent skills that cover spec-driven work from idea to merge.
Each one drives the work through `sj` commands instead of having the agent read
spec folders and logs:

| Skill | Use it to | Replaces |
| --- | --- | --- |
| `sj-explore` | think an idea through before specifying it | Superpowers brainstorming, OpenSpec explore |
| `sj-propose` | write the proposal, delta specs, design and tasks | Superpowers writing-plans; OpenSpec propose, update; Spec Kit specify, plan, tasks, analyze |
| `sj-clarify` | resolve `[NEEDS CLARIFICATION]` markers with the user | Spec Kit clarify |
| `sj-principles` | create or amend `<spec_root>/principles.md` | Spec Kit constitution |
| `sj-apply` | implement tasks test-first, in parallel where `[P]` allows | Superpowers TDD, executing-plans, subagent and parallel-agent skills; OpenSpec apply; Spec Kit implement |
| `sj-debug` | reproduce a bug as a failing scenario test, then fix it | Superpowers systematic-debugging |
| `sj-review` | review a diff against its scenarios and the principles | Superpowers code review; Spec Kit checklist |
| `sj-verify` | check `sj verify` before calling anything done | Superpowers verification-before-completion |
| `sj-finish` | verify, archive, and open the PR | Superpowers finishing-a-branch, git worktrees; OpenSpec archive, sync |

[skills/COVERAGE.md](skills/COVERAGE.md) maps every replaced skill and command to
its skill. `cargo test` checks the skills: frontmatter whose description says when
to use the skill, a body of at most 6,000 bytes, no commands from the tools they
replace, the `sj` commands each workflow needs, and complete coverage. Agent
adapters install them.

## The Claude Code adapter

```bash
sj init --agents claude          # install (writes specjutsu.toml too if it's missing)
sj update                        # regenerate; prints `adapters up to date` when nothing changed
sj init --remove-agents claude   # remove everything it added
```

It writes three things and touches nothing else:

- **A block in CLAUDE.md**, between `<!-- specjutsu:begin -->` and
  `<!-- specjutsu:end -->`, appended after your own content. It is under 1,200
  bytes.
- **The nine skills**, under `.claude/skills/sj-*/SKILL.md`. Each ends with a
  generated-hash line. `sj update` reports a skill you edited by hand
  (`modified … (kept; sj update --force replaces it)`), keeps it, and exits 1.
- **Two hooks** in `.claude/settings.json`, added after your existing ones. The
  file is edited as text, so your indentation, key order, permissions and other
  hooks stay as they are. Removing the adapter gives back the file byte for byte.

The generated files hold no timestamps, versions or paths, so prompt caches stay
valid. The hooks call `sj` from `PATH`:

- **`sj hook claude session-start`** prints the `sj status` lines and the active
  task, in at most 1,200 bytes.
- **`sj hook claude pre-edit`** runs before `Edit`, `Write`, `MultiEdit` and
  `NotebookEdit`. It always allows tests, specs, docs and other non-source
  files. For a source file, in `guided` and `strict` modes, it blocks the edit
  until each scenario of the active task has a red run in the ledger:
  `write a failing test for AUTH-005 first (sj test AUTH-005)`. In `advisory`
  mode it only warns. If the hook itself fails, it warns and allows the edit,
  so it never blocks work. CI's gate is the backstop for edits it can't see,
  such as shell redirects.

  Every pre-edit hook, and `sj hook check-edit`, judges a file by the git work
  tree that holds it. An edit in a linked worktree, under `.claude/worktrees/`
  or beside the main checkout, uses that worktree's `specjutsu.toml`, active
  task and ledger, even though Claude Code points the hook at the main checkout.
  Agents working side by side in their own worktrees don't block each other.

## Other agents

```bash
sj init --agents codex,cursor    # several at once; AGENTS.md is written once
sj init --agents auto            # the agents whose files the repo already has
sj init --remove-agents codex,cursor,cline
sj doctor                        # one line per installed agent: hook, instructions or hook missing
```

`sj init --agents` takes `claude`, `codex`, `cursor`, `cline`, `agents-md` and
`auto`. Codex, Cursor and Cline also get `agents-md`: a specjutsu block in
`AGENTS.md`, with the same markers and rules as the CLAUDE.md block. It names the
`sj` commands of the loop rather than skills, since not every agent that reads
AGENTS.md can load skills. Amp, Roo Code and other AGENTS.md readers get it with
`--agents agents-md`.

| Agent | Instructions | Pre-edit hook | Level |
| --- | --- | --- | --- |
| Claude Code | `CLAUDE.md` block, nine skills | `PreToolUse` in `.claude/settings.json` | `hook` |
| Codex | `AGENTS.md` block | `[[hooks.PreToolUse]]` for `apply_patch` edits, in `.codex/config.toml` | `hook` |
| Cursor | `AGENTS.md` block | `preToolUse` for `Write` and `Delete`, in `.cursor/hooks.json` | `hook` |
| Cline | `AGENTS.md` block | none: Cline's hooks are TypeScript plugins for its CLI only | `instructions` |
| Amp, Roo Code, others | `AGENTS.md` block | none | `instructions` |

Each hook makes the same decision as Claude's and answers in the agent's own
format. Codex sends a whole patch, so `sj hook codex pre-edit` checks every file
in it and blocks the patch if any one is a blocked source file, naming it. Cursor
treats output that isn't JSON as a deny, so `sj hook cursor pre-edit` always
prints `{"permission":"allow"}` or a deny. Input a hook can't read allows the edit
with a warning. Codex runs project hooks only in a trusted project, after you
trust the hook once with `/hooks`; until then the CI gate is the backstop.

The hook entries are added after yours. `.cursor/hooks.json` is edited as text
like Claude's settings, and Codex's hook goes in a `# specjutsu:begin` /
`# specjutsu:end` block at the end of `.codex/config.toml`, so removing an
adapter gives every file back byte for byte.

For an agent or editor with a hook system specjutsu doesn't support yet, call the
check directly. It exits 0 to allow and 2 to block, with the reason on stderr:

```bash
sj hook check-edit apps/api/src/auth.ts
```

## Importing Spec Kit

```bash
sj import speckit           # print the plan: targets, counts and gaps; writes nothing
sj import speckit --write   # convert (exits 2 if a target exists; --force overwrites)
```

Each `specs/NNN-<name>/` feature becomes OpenSpec layout under the spec root:

- **A finished feature** (no unchecked task) becomes `specs/<name>/spec.md` plus
  an archived change, `changes/archive/<Created date>-<name>/`.
- **A feature with open tasks** becomes an open change, `changes/<name>/`, whose
  spec is an `## ADDED Requirements` delta.

Inside the spec:

- **Each user story becomes a requirement.** Each numbered **Given / When / Then**
  item becomes a scenario with a new ID, `US<story>.<item>` plus its When clause
  as the name, and its clauses kept verbatim as GIVEN, WHEN and THEN steps. An
  item without a When uses its Given as the WHEN step.
- **Each `FR-NNN` line becomes a requirement too.** Spec Kit doesn't link
  requirements to scenarios, so an FR is a gap until a scenario names it. The
  import lists `gap FR-NNN (no scenario)`, and the spec gets a
  `[NEEDS CLARIFICATION]` marker that `sj spec check` reports.

Nothing is lost along the way:

- `plan.md` becomes `design.md`.
- Research, data model, quickstart, contracts, checklists and tasks are copied
  byte for byte. Spec Kit tasks keep their `T001` form.
- Clarifications, edge cases, key entities, success criteria, assumptions and the
  spec's header go into `proposal.md`.
- `.specify/memory/constitution.md` becomes `principles.md`.

The import never modifies or deletes anything under `specs/` or `.specify/`. To
remove them once the import is in, use `sj migrate --from speckit`.

## Migrating from OpenSpec, Spec Kit or Superpowers

```bash
sj migrate --from openspec --from superpowers           # print every planned change; writes nothing
sj migrate --from openspec --from superpowers --write   # apply it (needs a clean git tree)
```

`sj migrate` replaces the old tool in one step, so two workflows don't compete
for the agent. `--from` is repeatable: `openspec`, `speckit` or `superpowers`.

- **openspec** keeps `openspec/` (specs, open changes and archive) as the spec
  root and gives every scenario an ID. The `context`, `rules` and `operations`
  guidance in `openspec/config.yaml` move to `openspec/principles.md`, and the
  file is removed.

  It removes OpenSpec's skills and `/opsx` commands from every agent folder,
  such as `.claude/` and `.agents/`. A file is removed only when its metadata
  says OpenSpec generated it, so your own skills stay.
- **speckit** runs `sj import speckit --write` first. Only once that succeeds
  does it remove `.specify/`, the imported `specs/NNN-*/` folders and the agent
  files Spec Kit's manifests list. An import target that already exists stops
  the run before anything changes.
- **superpowers** removes the plugin, its marketplace and any permission rule
  naming `superpowers:` or `docs/superpowers/` from `.claude/settings.json`.
  Every other key, hook and rule stays byte for byte. Files under
  `docs/superpowers/` are kept and listed for you to review.

Every run then:

- writes `specjutsu.toml` with `sj adopt` if it's missing
- installs the Claude Code adapter
- prints `sj doctor` and `sj spec validate`

It exits 0 when both are clean, and 1 with the problems listed otherwise. It
exits 2, changing nothing, when the tree is dirty. The changes are left
uncommitted. Review them with `git diff`, and undo them with
`git restore . && git clean -fd`.

## Roadmap

| Release | Delivers |
| --- | --- |
| v0.1 | `sj trace` on OpenSpec + `cargo test`; non-blocking CI report |
| v0.2 | `sj gate` with red-on-base |
| v0.3 | vitest, Jest, Playwright, pytest and JUnit suites; `specjutsu.toml`; `sj ids`, `sj spec`, `status`, `next`, `verify`; Claude Code adapter and skills; Spec Kit import; `sj migrate` |
| v0.5 | More agent adapters, token benchmark, public launch |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Adding a test runner or an agent adapter
is the intended first PR.

## License

[MIT](LICENSE)
