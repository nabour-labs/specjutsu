# specjutsu

**Master the art of the spec.**

**[Website](https://nabour-labs.github.io/specjutsu/)** · **[User manual](https://nabour-labs.github.io/specjutsu/manual.html)** · **[Benchmarks](https://nabour-labs.github.io/specjutsu-bench/)** · **[GitHub Marketplace](https://github.com/marketplace/actions/specjutsu-gate)**

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-specjutsu%20gate-2ea44f?logo=github)](https://github.com/marketplace/actions/specjutsu-gate)

**The power tool for spec-driven development with AI.** Think OpenSpec or Spec Kit on
steroids: the same durable Markdown specs, plus a deterministic engine that proves your
coding agent actually built them, at a fraction of the token cost.

**The quality gate for AI-generated code.** specjutsu stops your coding agent from
shipping work it can't prove: every spec scenario gets a test, every test must fail
before the code exists, and the pull request is refused until both hold. It's one
open-source binary and one GitHub Action, with no LLM in the loop: every check is
deterministic, so CI gives the same answer every time.

## Verification, not vibes

specjutsu does in the binary and in CI what other spec-driven tools hand to the model.

- **Proof, not promises.** Every spec scenario is traced to a test. A scenario with
  no test, or a test pointing at a scenario that no longer exists, blocks the merge.
- **Red first, for real.** CI runs your new tests against the old code. If they
  already pass there, they didn't drive the change, and the gate says so.
- **Fewer tokens.** If a program can decide it, the binary does it and hands your
  agent a few lines, not whole folders and raw test logs. `sj next` gives the agent
  one task's scenarios, files and test commands; a green test run prints one line;
  a red one prints only the failing tests and their scenarios. Every command takes
  `--json`. The [public token benchmark](https://nabour-labs.github.io/specjutsu-bench/) runs the same three tasks with
  specjutsu, Spec Kit, OpenSpec and Superpowers and counts every byte each puts into
  the agent's context.

## One Action, every pull request

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
touches, the evidence for each (`red:assert`, `red:compile`, `green:linked`,
`untested`, `green-on-base`, …) and how to fix what fails. The Action is listed on the GitHub
Marketplace as [specjutsu gate](https://github.com/marketplace/actions/specjutsu-gate). See [The GitHub Action](#the-github-action).

**No specs yet?** Start with `command: audit`. It needs no setup and catches the ways
an agent makes checks pass without doing the work: deleted, skipped or focused
tests, lost assertions, and dependencies that don't exist. See
[Works without specs](#works-without-specs-sj-audit).

Under the Action is `sj`, a CLI that covers the whole loop: authoring (`sj spec`),
the agent's working loop (`sj status`, `next`, `verify`), the proof layer
(`sj trace`, `sj test`, `sj gate`), adapters for Claude Code, Codex, Cursor, Cline,
Copilot, Gemini CLI, Windsurf, Kiro, Antigravity and `AGENTS.md` with ten skills, and `sj migrate`. Specs live in `specjutsu/`;
projects coming from OpenSpec keep their `openspec/` folder unchanged, and it replaces
the OpenSpec, Spec Kit and Superpowers stack with one binary.

## How specjutsu compares

OpenSpec, Spec Kit, Kiro, Superpowers and BMAD stop at writing the spec, or ask a model
whether the code matches it. specjutsu is the only spec-driven tool that proves the work,
and it checks it deterministically:

| | OpenSpec | Spec Kit | Kiro | Superpowers | **specjutsu** |
| --- | --- | --- | --- | --- | --- |
| Scenario → test trace checked | No | No | Partly (property tests, in the IDE) | No | **Yes, `sj trace`** |
| Requirement for all inputs backed by a property test | No | No | Optional, in the IDE | No | **Enforced, `FOR ALL` scenarios** |
| Test must fail before the code | No | No | No | By prompt | **Enforced, then proven in CI** |
| PR blocked until proof holds | No | No | No | No | **Yes, GitHub Action** |
| Test-tampering checks | No | No | No | No | **Yes, `sj audit`** |
| Bug fix proven not to break nearby behaviour | No | LLM | LLM | No | **Yes, `[keep]` scenarios** |
| How the result is checked | LLM | LLM | LLM + optional tests | Prompt | **Deterministic, no LLM** |
| Runtime needed | Node | Python + uv | IDE | None | **None: single binary** |

The [public token benchmark](https://nabour-labs.github.io/specjutsu-bench/) is out: see how many context bytes each workflow
spends on the same feature, bug fix and change. specjutsu is not ahead everywhere yet:
the rest of OpenSpec parity (custom workflow schemas, multi-repo spec stores) for
v0.7, and keeping the shrunk counterexample of a property test as evidence
(`property-counterexamples`) is next. Specs for code that already exists ship as
`sj spec infer`, which drafts `[keep]` scenarios from untraced tests, and
[`FOR ALL` scenarios](#property-scenarios) need a property test. See [How specjutsu compares](docs/competitive-analysis.md) for the full
comparison, its sources, and the change that closes each gap.

## Status

**Pre-alpha, v0.6.0. Beta is next**: it opens once specjutsu has fully replaced
OpenSpec and Superpowers in our own production monorepo. `sj trace`, `sj test` and
`sj gate` work with Rust, vitest, Jest, Playwright and pytest, and any runner that
writes JUnit XML. Adapters install into Claude Code, Codex, Cursor, Cline, Copilot,
Gemini CLI, Windsurf, Kiro, Antigravity and any agent that reads `AGENTS.md`. See the [user manual](https://nabour-labs.github.io/specjutsu/manual.html)
for install, your first change, CI setup, agent setup and the command reference, and
the [design doc](docs/specjutsu%20%E2%80%94%20Project%20Kickoff%20%26%20Design.md)
and [analysis](docs/analysis.md) for the thinking behind it.

## Install

macOS and Linux:

```bash
curl -fsSL https://nabour-labs.github.io/specjutsu/install.sh | sh
```

Windows (PowerShell):

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://nabour-labs.github.io/specjutsu/install.ps1 | iex"
```

Both download the latest release for your platform, check it against the release's
`SHA256SUMS` and install `sj` into `~/.local/bin` (Windows: `%LOCALAPPDATA%\Programs\sj`,
added to your user `PATH`). Set `SJ_VERSION=0.6.0` to pin a release or `SJ_INSTALL_DIR`
to choose the folder. Or use a package manager:

```bash
brew install nabour-labs/tap/specjutsu   # macOS and Linux
cargo binstall specjutsu                 # downloads the release binary
cargo install specjutsu                  # builds from source; needs Rust
```

The archives themselves (macOS arm64 and x64, Linux x64 and arm64, Windows x64 and
arm64) are on the [releases page](https://github.com/nabour-labs/specjutsu/releases). A
browser download on macOS is quarantined by Gatekeeper; clear it with
`xattr -d com.apple.quarantine sj`. The installers and Homebrew don't need this.

To update, run the installer again, `brew upgrade specjutsu`, or
`cargo install specjutsu --force`. To get a fix that is merged but not yet released,
build from `main`. Then run `sj update` in each project:

```bash
cargo install specjutsu --force
cargo install --git https://github.com/nabour-labs/specjutsu --branch main specjutsu --force
cargo install --list | grep -A1 specjutsu   # the installed build and its source
```

`sj --version` prints the commit after the version for a build from a git
checkout, such as `sj 0.6.0 (f62a48b)`, and only the version for a crates.io
install. Open agent sessions use the new binary on their next hook call.

### Let your agent install it

The `specjutsu` setup skill takes an agent from no `sj` to a checked setup: it
installs `sj` (asking first), runs `sj adopt` on an existing project or `sj init` on
a new one, installs the agent adapters with `sj init --agents auto`, and finishes
with `sj doctor` and `sj status`. On a project with tests it offers `sj spec infer` to
draft their scenarios. It never replaces an existing `specjutsu.toml`.
Add it with the [skills CLI](https://skills.sh), where it is also the `/specjutsu`
command in Claude Code:

```bash
npx skills add nabour-labs/specjutsu --skill specjutsu
```

Or paste this into any agent:

```text
Set up specjutsu in this project: read
https://raw.githubusercontent.com/nabour-labs/specjutsu/main/skills/specjutsu/SKILL.md
and follow it.
```

Once set up, the project has the `sj-*` skills, so the setup skill isn't needed again.

### Shell completions

`sj completions <shell>` prints a completion script for `bash`, `zsh`, `fish`,
`powershell` or `elvish`, generated from the binary's own commands and flags. It
writes no file; save the output where your shell looks, and regenerate it after
upgrading:

```bash
# bash
sj completions bash > ~/.local/share/bash-completion/completions/sj
# zsh: then add `fpath=(~/.zfunc $fpath)` before `compinit` in ~/.zshrc
mkdir -p ~/.zfunc && sj completions zsh > ~/.zfunc/_sj
# fish
sj completions fish > ~/.config/fish/completions/sj.fish
# elvish: then add `use sj` to ~/.config/elvish/rc.elv
sj completions elvish > ~/.config/elvish/lib/sj.elv
```

In PowerShell, add `sj completions powershell | Out-String | Invoke-Expression` to
your `$PROFILE`.

## Quickstart

```bash
sj trace            # map scenario IDs in specjutsu/ to tests; list gaps
sj trace --json     # machine-readable
sj trace --write-baseline  # record today's gaps and orphans; later runs list only new ones
sj gate --base origin/main   # CI: changed scenarios tested, red on base, green on head
sj test             # run the tests; print failures only; record the run
sj test GATE-004    # run only the tests mapped to one scenario (or pass a package name)
sj ledger           # per scenario: latest result, red-before-green proven or not
sj gain             # bytes sj printed vs the raw output it replaced, per command
sj init             # new project: write specjutsu.toml, create specjutsu/specs and changes
sj adopt --dry-run  # existing project: detect packages and runners, print the config
sj doctor           # check the config, runner executables and spec folders
sj ids              # list scenarios without IDs and the IDs they would get
sj ids --write      # assign them in place
sj ids --write staff  # only one capability (a name, or a path to a spec file or folder)
sj ids prefixes     # suggest a short ID prefix per capability; --write adds them to specjutsu.toml
sj spec new add-oauth-login  # scaffold a change; prints the next free ID per prefix
sj spec list        # open changes: tasks done, scenarios added or modified
sj spec show AUTH-003   # one scenario, change or capability; a scenario with its tests and evidence
sj spec validate    # spec format, delta structure, every task names its scenarios
sj spec check add-oauth-login    # uncovered scenarios, unknown IDs, open questions
sj spec archive add-oauth-login  # merge the deltas into specs/, move to changes/archive/
sj spec archive --all-done       # archive every change whose tasks are all done, together
sj spec infer tests/refunds.rs   # existing tests: draft a [keep] scenario per untraced test; --write makes a change
sj status           # each open change: tasks, scenario evidence, the next task
sj view             # one-screen dashboard: capabilities, open changes, evidence totals
sj next             # the next task: its scenarios, changed files, test commands
sj task done add-oauth-login 2.1  # check the box (strict mode needs red-before-green)
sj verify           # before a PR: what the gate checks, without red-on-base
sj init --agents claude  # install the Claude Code adapter: CLAUDE.md block, skills, hooks
sj init --agents auto    # install for every agent the repo already has files for
sj update           # regenerate installed adapters; hand-edited files are kept
sj update --dry-run # print what update would write, keep or remove; change nothing
sj completions zsh  # print a shell completion script (bash, zsh, fish, powershell, elvish)
sj import speckit   # preview converting Spec Kit's specs/NNN-*/ features; --write converts
sj migrate --from openspec --diff  # preview replacing OpenSpec, with each file's diff
sj migrate --undo   # put back what the last sj migrate --write changed
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
test in `--json` carries its `package` and `suite`, `skipped` for
`.skip`/`.todo`/`.fixme`, `@pytest.mark.skip` and Rust `#[ignore]`, and `property`
for a proptest, Hypothesis or fast-check test (see
[Property scenarios](#property-scenarios)). A package with source files but
no suite gets a `warn    no test suite: <path>` line. Titles built at runtime
(variables, `${}` templates) are not read. Use a `// spec:` comment for those.

### Trace baseline

A project that adopts specjutsu late may have thousands of scenarios without
tests. `sj trace --write-baseline` records today's gaps and orphans in
`<spec_root>/trace-baseline.txt`, one sorted entry per line, and replaces any
earlier baseline:

```
$ sj trace --write-baseline
baseline: 3 entries → ./specjutsu/trace-baseline.txt
$ cat specjutsu/trace-baseline.txt
gap AUTH-002
gap AUTH-005
orphan AUTH-009 tests/auth.rs auth_009_old
```

An orphan is keyed by its ID, test file and test name, so a new test that reuses a
baselined unknown ID is still new. Test file paths with spaces aren't supported: such
an orphan never matches its baseline line, so it always counts as new.

With a baseline, `sj trace` prints a line only for entries it doesn't list, and the
summary counts the known ones in parentheses:

```
$ sj trace --strict
trace: 2 mapped, 2 gaps (1 known), 1 orphan (1 known), 1 without ID
gap     AUTH-006 Expired  (specjutsu/specs/auth/spec.md:23)
stale gap AUTH-002 (remove from trace-baseline.txt)
no-id   Ships binaries  (specjutsu/specs/distribution/spec.md:11)
```

A **stale** entry is one that is no longer a gap or orphan, here because AUTH-002
gained a test. `--strict` exits 1 only for entries the baseline doesn't list and for
stale entries, so CI notices both new debt and lines left behind; known entries
alone pass.
`sj trace --write-baseline` drops stale entries. In `--json`, every gap and orphan
carries `"known": true` or `false`, and a `stale` array lists stale entries as their
baseline lines (`["gap AUTH-002"]`). See
[Adopting a baseline](#adopting-a-baseline) for the gate's side.

## How the gate works

`sj gate --base origin/main` fails a change when an added, modified or newly
implemented scenario has no test, a test fails, or a test points at a missing
scenario that the base branch's [baseline](#adopting-a-baseline) doesn't list.
Its core check is **red-on-base**: it checks out the base commit,
overlays only the change's test files, and requires each new test to fail
there. It does this for every suite in `specjutsu.toml`: Cargo `tests/` files, and
for other runners every changed file matching the suite's `tests` or `support`
globs, so a co-located `auth.test.ts` runs against the base `auth.ts`. A test that passes on the old code didn't drive the change. Refactors and
chores declare an exemption with a reason (`Exempt: refactor — <why>` in the
change's `proposal.md`). An exempted change that edits source still has to pass
the full test suite.

The gate fails closed. Any changed file counts as source, so it needs a scenario
or an exemption, unless it is a test, a spec, documentation or media (`*.md`,
`*.txt`, images), repository metadata (`.github/`, `.gitignore`, `LICENSE`), tool
configuration (`specjutsu.toml` and the root agent folders `.claude/`, `.agents/`,
`.agent/`, `.codex/`, `.cursor/`, `.gemini/`, `.kiro/`, `.windsurf/`, `.clinerules/`), or a
manifest or lockfile (`Cargo.toml`, `package.json`, `pyproject.toml`, …). A new
or edited scenario without an ID fails as `unidentified`. Existing scenarios
without IDs are counted in one warning. With no test suite configured (no
`specjutsu.toml` suites and no root `Cargo.toml`), the gate exits 2 and suggests
`sj adopt` instead of passing changes it can't test.

A base checkout has no installed dependencies, so a suite that needs them declares
`setup`. The gate runs it once in the base package directory before that suite's
base run, and a failing setup exits 2 with its last 20 lines. A suite whose base run
writes no report at all, because its runner couldn't start, also exits 2 rather than
counting as red. `sj adopt` writes `setup` from the nearest lockfile (`pnpm install
--frozen-lockfile`, `npm ci`, `yarn install --frozen-lockfile`, `bun install
--frozen-lockfile`, `uv sync --frozen`) and notes suites it found none for. A suite that can't run
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

A test can also fail to load on base because it imports a test helper the change
added or modified but the gate didn't overlay. That shows nothing about the old code, so the
gate fails it as `not-overlaid STAFF-073 lists admins (apps/api/src/testing/admin.ts;
add it to the suite's support globs)` instead. The gate checks the files the change
added or modified under a helper folder (`testing`, `test-utils`, `test-helpers`,
`testutils`, `fixtures`, `__fixtures__`, `mocks` or `__mocks__`) that weren't overlaid,
and looks for each one in the base failure message, by its path from the repository, the
package or the test's folder, with or without its extension, with `\` or `/` as the
separator. A helper base already has counts too, such as an edited `testing/index.ts`
whose new export the base run reports as `does not provide an export named 'makeAdmin'`. Only
non-Cargo suites report failure messages, so Cargo tests are never `not-overlaid`.
A new source module that base can't load stays `red:compile`. Declare helpers
outside these folders in `support` by hand.

A suite that wants no compile red at all sets `red_on_base = "assert"` (it takes no
`reason`, and works for Cargo suites too). There only an assertion failure on base
counts, and a test that fails to compile or load fails as `compile-red AUTH-004
rejects expired token (suite requires an assertion failure on base)`. A
`not-overlaid` failure is reported first, before the `[keep]` checks, and both come
before `compile-red`.

A scenario that already existed but gains its first test in the change is
**implemented**, and is gated like an added one, with one exception: when its tests
pass on base and at head, the behaviour predates the change and the test only links
to it. The gate passes it as `green:linked` instead of failing it as
`green-on-base`. `--json` lists it in `proven` with `"red": "green:linked"`, and
`--format markdown` lists it under **Linked existing behaviour**, and the pass line
counts it, as in `gate: pass — 1 scenario proven (1 green:linked)`. When one of its
tests fails on base, that failure proves it instead, and its other tests may pass there.
An implemented scenario whose tests all pass on base never counts as specifying a source
change, even when a test then fails at head, so a change that also edits source still
needs a scenario that is red on base, or an exemption, or it fails as
`unspecified change`.

When a package declares `sources`, only changed files matching them need a scenario,
and files elsewhere in that package don't. A top-level `non_source = ["infra/**"]`
adds globs to the built-in non-source list.

### Adopting a baseline

The gate reads `<spec_root>/trace-baseline.txt` (see [Trace
baseline](#trace-baseline)) as it is on the merge base, not as the pull request
leaves it:

- **Known gaps collapse into one warning**, `known gaps: 2981 (trace-baseline.txt)`,
  instead of a `gap <ID>` line each.
- **Known orphans don't fail the gate.**
- **Changed scenarios are still gated.** An added, modified or newly implemented
  scenario needs its red-on-base test (or, for an implemented one, a
  `green:linked` pass) whether the baseline lists it or not.
- **Stale entries only warn**, as `stale gap AUTH-002`, so adding a test to an old
  scenario doesn't fail a pull request because of the baseline. Run
  `sj trace --strict` in CI to make someone remove the line.
- **Entries a change adds fail** as `baseline-added <entry>`, so a pull request
  can't hide its own orphans. With a `config` exemption they are honoured and listed
  as `baseline-added <entry>` warnings, for the reviewer to see.

So the first baseline lands in its own commit, with a `config` exemption:

```bash
sj trace --write-baseline
git add specjutsu/trace-baseline.txt
git commit -m "chore: adopt trace baseline" -m "Specjutsu-Exempt: config — adopt trace baseline"
```

Its gate run passes and lists what it adopted:

```
gate: pass — no changed scenarios to prove
warn    baseline-added gap AUTH-002
warn    baseline-added gap AUTH-005
warn    baseline-added orphan AUTH-009 tests/auth.rs auth_009_old
warn    known gaps: 2 (trace-baseline.txt)
warn    1 scenario without ID
exempt  config — adopt trace baseline
```

Without the trailer, each added entry fails as `baseline-added gap AUTH-002`, and so
on. Regenerating the baseline later, to drop stale entries, needs no exemption: only
added entries do.

## The GitHub Action

The Action is published on the GitHub Marketplace as
[specjutsu gate](https://github.com/marketplace/actions/specjutsu-gate), where you can also find its latest version.

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
`specjutsu/` (or `openspec/`, when only that folder exists) and, when there is a root
`Cargo.toml`, one Cargo suite at the root.

Nobody should write it by hand. `sj init` writes it for a new project. `sj adopt`
reads an existing one's manifests (Cargo, pnpm, npm and yarn workspaces,
`pyproject.toml`) and config files, and never runs a tool. It detects `cargo`,
`vitest`, `jest`, `pytest` and `playwright`. For a pnpm monorepo with a uv
service, `sj adopt --dry-run` prints:

```toml
schema = 1
spec_root = "specjutsu"
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

For each non-Cargo suite, adopt also declares the package's test helper folders
(`testing`, `test-utils`, `test-helpers`, `testutils`, `fixtures`, `__fixtures__`,
`mocks`, `__mocks__`) that its `tests` globs don't cover, so the gate overlays them
on base. With `apps/api/src/testing/admin.ts` and `apps/api/src/__fixtures__/user.json`,
the `apps/api` vitest suite gets
`support = ["apps/api/src/__fixtures__/**", "apps/api/src/testing/**"]`. A suite
with no helper folder gets no `support` key. Helper folders under a `tests` folder are
left out, since the gate always overlays those. Adopt prints a note on stderr for each
glob it adds:

```
note    support: apps/api/src/testing/** (not source; overlaid on base; remove it if this folder holds product code)
```

Support files are not source: they need no scenario, and the gate runs the head copy
against the base code. Don't mark a folder of product code as support; remove the glob if
one is. A helper folder adopt can't read stops it with the error.

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
USERAUTH-001 Valid registration (specjutsu/specs/user-auth/spec.md:6)
USERAUTH-002 Password too short (specjutsu/specs/user-auth/spec.md:10)
wrote 2 IDs to 1 file
```

- **Only the headings change.** The ID goes before the scenario name, and every
  other byte, line endings included, stays as it was. Running it again changes
  nothing.
- **Prefixes come from the capability folder:** its letters, uppercased
  (`user-auth` → `USERAUTH`). Set your own in the project file, or let
  `sj ids prefixes --write` do it (see [Short prefixes](#short-prefixes)):

  ```toml
  [ids.prefixes]
  user-auth = "AUTH"
  ```

  If a capability that needs IDs ends up with the same prefix as another
  capability, nothing is written and the command exits 2 naming both. That holds
  when the other one is already fully numbered. Map one of them to fix it.
- **IDs are never reused.** Numbers continue after the highest one used for that
  prefix anywhere under the spec root, `changes/archive/` included, so keep the
  archive. Archived changes are read, never written.
- **A delta copy shares its spec's ID.** A scenario in an open change with the
  same requirement and scenario name as one in `specs/` gets the same ID, matching
  a requirement the same change renames by its old name. It works
  the other way too: a `specs/` scenario without an ID takes the ID its copy in an
  open change's MODIFIED requirement already has, matched by capability,
  requirement and scenario name, ignoring IDs and `[keep]`. A requirement the same
  change renames (`## RENAMED Requirements`) matches by its old name. The original
  gets a new number instead, with a `note:` line, when another scenario already
  uses that ID or when copies in two open changes disagree:

  ```
  note: copies disagree: STAFF-057 Role change is recorded (openspec/changes/c1/specs/staff/spec.md:6), STAFF-058 Role change is recorded (openspec/changes/c2/specs/staff/spec.md:6); Role change is recorded (openspec/specs/staff/spec.md:6) gets STAFF-100; give the delta copies STAFF-100
  ```

  A taken ID reads `note: <copy> is taken by <scenario>; … gets STAFF-100; give the
  delta copies STAFF-100`. Either way, update the delta copies by hand so they match.

  In `--json`, such an assignment carries a `note` object: `{"kind": "taken",
  "copy", "by"}` or `{"kind": "disagree", "copies"}`, each scenario with its `id`,
  `name`, `file` and `line`.
- **Targets limit the run to some capabilities.** `sj ids --write staff` lists and
  writes only the `staff` scenarios, in `specs/` and in open changes, and leaves
  every other file byte for byte. A target is a capability name or a path to a
  spec file or folder, resolved against `--root` or absolute, and selects the whole
  capability of every open spec file under it, so a MODIFIED copy and its original
  always get the same ID. The prefix check covers every capability, anywhere under
  the spec root, whose prefix a selected capability uses, so a collision on that
  prefix still stops the run, but one between two other capabilities (`billing` and
  `bill-ing` both giving `BILLING`) doesn't stop `sj ids staff` or `sj ids --write staff`.
  When listing a target, a collision that involves a capability being numbered prints
  as a `note: capabilities … both use prefix …` line instead of stopping. Numbering
  still covers the whole spec root, so numbers continue after the highest one used
  anywhere. A target that selects nothing exits 2, names it and lists the known
  capabilities. A capability named `prefixes` can only be targeted by path (`sj ids specjutsu/specs/prefixes`), since
  `sj ids prefixes` is the subcommand.

The gate treats a scenario that only gained an ID as unchanged, so the ID commit
can land on its own: its scenarios show as `gap` warnings, not failures. Adding a
test in the same change makes the scenario `implemented`: a test that fails on
base proves it, and one that passes on base is `green:linked`, which covers no
source change made alongside it. `sj gate --json` lists every scenario that isn't unchanged under
`changes`.

### Short prefixes

A folder name longer than six letters gives a long prefix. `sj ids` then ends its
listing with a hint per capability:

```
hint: distribution would use DISTRIBUTION; sj ids prefixes --write maps it to DIST
```

`sj ids prefixes` prints an `[ids.prefixes]` table with one line per capability
folder, sorted, and writes nothing:

```
$ sj ids prefixes
[ids.prefixes]
auth = "AUTH"
distribution = "DIST"
payment-methods = "PAYM"
transactions = "TRANSA"
transfers = "TRANSF"
```

Each prefix is chosen in this order:

1. A prefix already mapped in `[ids.prefixes]` is kept.
2. A capability whose scenarios already carry IDs keeps the prefix they use, so
   nothing is ever renamed.
3. Otherwise its letters, uppercased: six letters or fewer are kept whole
   (`gate` → `GATE`), and longer names are cut to four (`payment-methods` →
   `PAYM`).

Suggestions that clash with each other or with a kept prefix are lengthened together,
one letter at a time, until all differ (`transactions` → `TRANSA`, `transfers` →
`TRANSF`). When names run out of letters and still clash, as `bill` does with
`billing = "BILL"`, it exits 2 without printing the table or writing anything:
`` capabilities `bill` and `billing` both use prefix BILL ``. Map one of them in
`[ids.prefixes]` by hand. `sj migrate --from openspec` refuses the same way before
it changes anything.

`sj ids prefixes --write` adds the capabilities missing from `[ids.prefixes]` to
`specjutsu.toml` and prints `added 5 prefixes to specjutsu.toml`. It only appends
lines: existing keys and every other byte stay as they were, and the table is
created at the end of the file when there is none. Running it again changes
nothing. Without a `specjutsu.toml` it exits 2 and asks for `sj init` first. When
the prefixes are written in a layout it can't splice, such as an inline
`prefixes = { … }` under `[ids]`, it exits 2, changes nothing, and prints the lines
to add by hand. It has no `--json`, and `sj ids --write prefixes` or
`sj ids --json prefixes` exits 2, pointing at `sj ids prefixes --write`.

Run it before `sj ids --write`, so the new IDs come out short.

## Writing and archiving changes

`sj spec` creates, validates, checks and archives changes, including the *analyze*
and *clarify* passes. Existing `openspec/` folders work unchanged.

- **`sj spec new <change>`** creates `changes/<change>/` with `proposal.md`,
  `design.md`, `tasks.md` and an empty `specs/`. The name must be kebab-case. It
  prints the next free ID for every prefix in use. `--bug` scaffolds a bug fix
  instead (see [Bug fixes and kept behaviour](#bug-fixes-and-kept-behaviour)).
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
  prints the same data with a `kind` field; a scenario also has `"property"`,
  true when it has a [`FOR ALL` step](#property-scenarios).
- **`sj spec validate [<change>]`** reports each problem with its file and line:
  - A scenario without an ID, a duplicate ID, or a missing WHEN or THEN step.
  - A requirement with no scenario, or with no SHALL or MUST.
  - A delta requirement outside `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`.
  - A REMOVED requirement without `**Reason**` and `**Migration**`.
  - A MODIFIED, REMOVED or RENAMED name that the current spec doesn't have.
  - A task that names no scenario ID. Mark tasks that cover none with
    `(no-scenario: <reason>)`, for example docs.
  - A scenario marker other than `[keep]`.
  - A `FOR ALL` step after the WHEN step: `PAY-021 FOR ALL step must come before
    WHEN (<file>:<line>)`.

  A scenario in a MODIFIED requirement whose original in `specs/` has no ID gets
  a hint, since OpenSpec matches scenarios by full name and would read the copy as
  a dropped scenario plus a new one. With an ID it is a warning, `STAFF-057 Role change is recorded:
  specs/staff/spec.md has this scenario without an ID; run sj ids --write staff
  first`; without one, the missing-ID error ends with `; run sj ids --write staff
  first`. When both have IDs and they differ, the same tools see two scenarios, so
  validate warns `STAFF-057 Role change is recorded: specs/staff/spec.md has this
  scenario as STAFF-100; give the delta copy STAFF-100`.

  It exits 1 on errors. `--strict` also fails on warnings, such as a placeholder
  Purpose.
- **`sj spec check <change>`** lists scenarios no task names (`uncovered
  AUTH-006`), tasks naming IDs nobody defines, and every
  `[NEEDS CLARIFICATION: …]` marker outside code. For a bug change it also
  fails when no scenario reproduces the bug, and warns when none is `[keep]`.
- **`sj spec archive <change>`** refuses while validation fails or a task is
  open, unless you pass `--force`. Then it applies RENAMED, REMOVED, MODIFIED and
  ADDED requirements to `specs/` and moves the change to
  `changes/archive/YYYY-MM-DD-<change>/`. A new capability gets the delta's
  `## Purpose`.

  The merge follows `openspec archive` 1.14.0 rule for rule, and writes the same
  bytes. Like OpenSpec, it writes LF line endings and one blank line between
  requirements. A MODIFIED block that would drop a scenario is an error. Nothing
  is written unless every delta merges.

  It takes several names, `sj spec archive add-oauth-login fix-token-expiry`, or
  `--all-done` for every open change with at least one task and all of them done
  (it prints `nothing to archive` when there are none). The changes are archived
  in name order as one operation: if any one is refused, nothing is written and
  each problem is listed. Two changes whose deltas modify, remove or rename the
  same requirement are refused even with `--force`; archive one, then rebase the
  other's delta.
- **`sj spec infer <path>...`** drafts a `[keep]` scenario for every untraced test
  under the given paths. See [Specs from existing tests](#specs-from-existing-tests).

### Bug fixes and kept behaviour

A bug fix has to do two things: fix the broken case and leave the cases next to
it alone. specjutsu proves both. A scenario marked `[keep]` describes behaviour
the change must not alter:

```markdown
#### Scenario: PAY-012 Refunds under $100 auto-approve
- **WHEN** a $99 charge is refunded
- **THEN** the refund is approved without review

#### Scenario: PAY-013 [keep] Refunds over $100 need approval
- **WHEN** a $150 charge is refunded
- **THEN** the refund waits for approval
```

PAY-012 is the reproduction: like any scenario, its test must fail on base.
PAY-013's test must do the opposite: pass on base and at head. The gate records
it as `green:keep`, and fails with `keep-red-on-base` when the test fails on
base (it describes new behaviour, so drop the marker) or `keep-broken` when the
change breaks it. The PR comment lists kept scenarios under **Kept behaviour**.

`sj spec new fix-small-refunds --bug` writes a proposal with Current,
Expected and Unchanged Behaviour sections and a `Kind: bug` line, which makes
`sj spec check` ask for both kinds of scenario. The `sj-debug` skill writes
bug fixes this way.

### Property scenarios

Some behaviour must hold for every input, not just the values a test picks. A
`FOR ALL` step, before WHEN, says so:

```markdown
#### Scenario: PAY-020 Refund never exceeds the charge
- **FOR ALL** charges and refund requests
- **WHEN** a refund is issued
- **THEN** the refunded total is at most the charge
```

Such a scenario needs a property test. specjutsu finds them by reading the test
source; it never runs or imports the libraries itself:

- **Rust:** a test inside a `proptest! { ... }` block, or one with `#[proptest]`.
- **Python:** a test with `@given(...)` or `@hypothesis.given(...)` in a file that
  imports `hypothesis`.
- **TS/JS:** a test that calls `<name>.assert(`, `<name>.property(` or
  `<name>.asyncProperty(`, where `<name>` comes from `import fc from 'fast-check'`,
  `import * as fc from 'fast-check'`, `import fc, { … } from 'fast-check'`,
  `const fc = require('fast-check')` or TS `import fc = require('fast-check')`.

Not detected: aliases of named imports (`import { assert as check }`), re-exports,
`@fast-check/vitest`'s `test.prop`, a property callback passed by reference, and the
closure form `proptest!(|..| ..)` inside a plain `#[test]`.

`sj spec validate` reports `PAY-021 FOR ALL step must come before WHEN (<file>:<line>)`
when the step is out of place, and `sj spec show <ID> --json` reports `"property": true`.
Each test in `sj trace --json` carries `"property"`. A FOR ALL scenario whose linked tests
are all example tests is listed as `not-property PAY-020 pay_020_refund_of_50_on_100`, and
under `not_property` in `--json`; `--strict` exits 1 on it, and the baseline never
records it. A FOR ALL scenario with no test is a gap, as before.

The gate fails such a scenario, `[keep]` ones included, with
`not-property PAY-020 pay_020_refund_of_50_on_100 (a FOR ALL scenario needs a property
test; write one with proptest)`. The hint names proptest for Cargo suites, Hypothesis
for pytest and fast-check for vitest, Jest and Playwright; JUnit suites pick it by the
test file's extension (`.rs` proptest, `.py` Hypothesis, JS/TS fast-check). When a scenario has both
kinds, only its property tests count as evidence on base, and its example tests must
still pass at head. The pass line counts proven properties after the evidence kinds,
as in `gate: pass — 2 scenarios proven (2 red:assert; 1 property)`, and each `proven`
entry in `--json` has `"property": true` or `false`.

### Specs from existing tests

A project that adopts specjutsu late has tests that no scenario describes.
`sj spec infer <path>... [--capability <name>] [--write] [--change <name>]` turns
them into `[keep]` scenarios, one per test, so the behaviour they already prove is
written down and gated. It picks the untraced tests in suite files under the paths:
tests with no scenario ID in their name, title, `spec:` comment or
`@pytest.mark.spec` marker. Tests that are already linked, and tests marked skip,
todo, fixme or Rust `#[ignore]`, are left out and counted.

Without `--write` it is a dry run and writes nothing:

```
$ sj spec infer tests/refunds.rs --capability refunds
REFUNDS-001 [keep] Refund under 100 auto approves  (tests/refunds.rs:4)
REFUNDS-002 [keep] Refund over limit needs review  (tests/refunds.rs:10)
2 tests drafted, 1 linked left out
```

When there is nothing to draft it exits 2 with `no untraced tests under <paths>`.

- **Headings** come from the test name: a leading `test` word is dropped and `_` and
  `-` become spaces (`test_partial_refund` → `Partial refund`). A pytest class keeps
  each class name minus a leading `Test` word (`TestRefunds::test_partial_refund` →
  `Refunds partial refund`). JS and TS titles are kept as written.
- **The capability** is `--capability`, or else the nearest folder above the test
  file that isn't `tests`, `test`, `__tests__`, `spec`, `unit`, `integration`, `e2e`
  or `src` (any case), in kebab-case: `packages/CartItems/Tests/unit/cart.test.ts`
  gives `cart-items`. One run drafts for one capability; tests in folders that name
  different capabilities, or in no capability folder, exit 2 with a hint to pass
  `--capability`.
- **IDs** follow the [ID rules](#adding-ids-to-existing-scenarios): an
  `[ids.prefixes]` entry wins, numbering continues after the capability's current,
  open-change and archived IDs and after any number a scanned test already carries
  with that prefix in its name or `spec:` comment, and a colliding prefix or a
  capability with no letters exits 2. A test whose name already carries an ID with
  the capability's prefix (`billing_007_named_early`) keeps that ID; when another
  scanned test carries it too, or a current, open-change or archived scenario ever
  used it, the run exits 2 naming the test and the ID and asks for the test to be
  renamed. Drafts are listed in file order. A `--capability` that isn't kebab-case exits 2
  with ``capability `<name>` must be kebab-case`` before anything is written.

`--write` creates `<spec_root>/changes/infer-<capability>/` (or `--change <name>`)
with `proposal.md`, `tasks.md` and `specs/<capability>/spec.md`: one requirement
per test file and one `[keep]` scenario per test, whose WHEN and THEN are
`[NEEDS CLARIFICATION: … — see <file>:<line>]` markers. It also inserts
`// spec: <ID>` (Rust, JS, TS) or `# spec: <ID>` (Python) above each test's first
attribute, decorator or declaration line, and changes no other byte. Each comment
moves the tests below it down a line, so the markers and the lines `--write` prints
name the lines the tests are on afterwards. A requirement whose name is already
used by the capability's spec or an open change's delta for it is named after the
file's path instead, and the proposal lists the capability under Modified
Capabilities when its spec exists or an open change already adds it. It refuses a
change folder that already exists, and leaves nothing behind on an error.

```
$ sj spec infer tests/refunds.rs --capability refunds --write
REFUNDS-001 [keep] Refund under 100 auto approves  (tests/refunds.rs:5)
REFUNDS-002 [keep] Refund over limit needs review  (tests/refunds.rs:12)
2 tests drafted, 1 linked left out
created specjutsu/changes/infer-refunds
$ sj spec check infer-refunds    # a clarify line per marker until each is replaced
$ sj gate --base origin/main     # REFUNDS-001 and REFUNDS-002 pass as green:keep
```

Replace each marker with the WHEN and THEN the test proves, reading the test at the
line the marker names (the `sj-propose` skill does this). The tests only gained a
comment, so they pass on base and at head, and the change passes the gate as
`green:keep`.

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
a passing one, on different code. When a scenario's latest failing record holds
the counterexample its property test printed, `sj ledger` shows it after the
status, as `(counterexample: charge = 100, refund = 101)`, and `sj ledger --json`
gives it as the entry's `counterexample`. Pass `--no-ledger` to skip recording.

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
  task names is proven: `unproven AUTH-005 (no red run recorded)`. It also
  refuses a FOR ALL scenario whose tests are all example tests, even after
  red-before-green: `not-property <ID> <test>`, as `sj verify` prints it. In
  `guided` and `advisory` modes it checks the box and prints the same lines as
  warnings.
  Like `sj verify`, it asks for a red run only for added and modified scenarios
  (see below). It always judges against the default branch, never `--base`, and
  when it can't classify the scenarios it warns `cannot classify scenarios
  against the merge-base, so every one needs a red run and every FOR ALL one a
  property test: <why>`.
- **`sj view`** prints a one-screen project dashboard: each capability with
  its scenario count, each open change with its tasks and scenario evidence
  (the counts `sj status` uses), evidence totals over every scenario, the
  number of orphan tests and of archived changes. It fits in 40 lines, cutting
  long lists with `… +N more`. It reads files and the ledger only, and never
  runs tests.
- **`sj verify [<change>]`** is the local check before a PR. It runs the
  change's scenario tests (recording them), and reports untested and failing
  scenarios, unproven ones, FOR ALL scenarios with only example tests
  (`not-property <ID> <test>`, as in the gate; a red-then-green example test
  doesn't prove them), orphans, `sj spec check` problems, and the gate's
  unspecified-change, exemption and trace baseline rules against the merge-base:
  orphans the base's baseline lists aren't problems, and entries the branch adds
  without a `config` exemption are `baseline-added <entry>` problems. Red-on-base
  stays in CI. It prints `verify: ok` or one line per problem, and exits 1 when
  there are any.

  Both `sj verify` and `sj task done` classify each scenario against the
  merge-base the way the gate does, and need a red run only for **added** and
  **modified** scenarios. An **unchanged** scenario, such as one copied without
  edits into a MODIFIED requirement, and an **implemented** one (an existing
  scenario that gains its first test) need only a passing latest run: an
  implemented scenario may be `green:linked`, which only the gate's base run can
  tell. Their tests still run, and a failing one is still reported. Since verify
  can't see the base run, an implemented scenario whose ledger shows no
  red-before-green doesn't count as covering a source change there: a change that
  only adds such a test and edits `src/lib.rs` gets `unspecified change`. On the
  default branch itself (the merge-base is HEAD), with no merge-base, or when
  `sj task done` can't classify, every scenario needs a red run.

`sj next` and `sj verify` compare against `--base`, or else the first of
`origin/HEAD`, `origin/main`, `origin/master`, `main` and `master` that exists.

### Gain: bytes saved

`sj test`, `sj verify`, `sj next` and `sj status` each append one record per run
to `.specjutsu/ledger/gain.jsonl` (gitignored with the ledger): the bytes the
command printed and the bytes of the raw material it stands in for. For `sj test`
and `sj verify` that is the runners' combined stdout and stderr; for `sj next`,
every file in the change folder it reported on; for `sj status`, every file in
the open change folders it reported on (or the named one). Nothing is recorded
with `sj test --no-ledger` or outside a git repository, and recording never
changes a command's output or exit code.

`sj gain` totals the records, one line per command, then a total:

```
test   2   500  50000  99%
next   1  1000   4000  75%
total  3  1500  54000  97%  ~13125 tokens saved
```

The columns are runs, printed bytes, raw bytes and percent saved (rounded down).
Tokens are estimated as (raw − printed) / 4, and printed > raw shows 0% and ~0
tokens saved; specjutsu never calls a model. The numbers are bytes, a proxy, not
a benchmark. Unreadable lines are skipped with a warning on stderr, and with no
records it prints `no gain records yet`. `--json` reports the skipped lines as
`skipped`, and with no records prints empty `commands` and zero totals instead.

```bash
sj gain --json                          # {"commands": [...], "total": {...}, "skipped": 0}
sj gain --since 2026-11-22T09:00:00Z    # only records at or after a time: one session
sj gain --reset                         # delete the records: removed N gain records
```

## Skills

`skills/` holds ten agent skills that cover spec-driven work from idea to merge,
plus the setup skill `specjutsu` (see [Let your agent install it](#let-your-agent-install-it)).
Each one drives the work through `sj` commands instead of having the agent read
spec folders and logs:

| Skill | Use it to | Replaces |
| --- | --- | --- |
| `sj-explore` | brainstorm and think an idea through before specifying it: go wide, then narrow to two or three options | Superpowers brainstorming, OpenSpec explore |
| `sj-propose` | write the proposal, delta specs, design and tasks, or draft scenarios from existing tests with `sj spec infer` | Superpowers writing-plans; OpenSpec propose, update; Spec Kit specify, plan, tasks, analyze |
| `sj-clarify` | resolve `[NEEDS CLARIFICATION]` markers with the user | Spec Kit clarify |
| `sj-principles` | create or amend `<spec_root>/principles.md` | Spec Kit constitution |
| `sj-apply` | implement tasks test-first, each by an implementer subagent and checked by a fresh reviewer subagent, in parallel where `[P]` allows | Superpowers TDD, executing-plans, subagent and parallel-agent skills; OpenSpec apply; Spec Kit implement |
| `sj-debug` | find a bug's root cause in four phases, capture it as a failing scenario test, then fix it | Superpowers systematic-debugging |
| `sj-review` | review a diff against its scenarios and the principles | Superpowers code review; Spec Kit checklist |
| `sj-verify` | check `sj verify` before calling anything done | Superpowers verification-before-completion |
| `sj-finish` | verify, archive, and open the PR | Superpowers finishing-a-branch, git worktrees; OpenSpec archive, sync |
| `sj-onboard` | walk a newcomer through one toy change, then remove it | OpenSpec onboard |

`sj-onboard` runs the whole loop in a scratch change, `sj-onboard-demo`: `sj spec
new`, one scenario, a red `sj test`, the smallest code, a green `sj test`, `sj task
done` and `sj verify`, explaining each step first. It refuses to start on a dirty
working tree or when `sj-onboard-demo` already exists. At the end it deletes the
change, the demo test and the demo code without archiving, so `specs/` is unchanged.

`sj-apply` runs every task as a loop. An implementer subagent gets the task's `sj next`
output and does red, green and refactor. A fresh reviewer subagent, which never sees the
implementer's report, checks the scenario, the diff and `sj verify`, and answers
`APPROVED` or `CHANGES_REQUESTED`. Fixes go to a fresh re-reviewer, for at most 3 rounds
before the user decides. Only the main agent runs `sj task done`, and the change ends
with a whole-change `sj-review` in a fresh subagent.

A skill keeps its `SKILL.md` short and loads the detail from reference files next to it
only at the step that needs it:

| Skill | Reference files |
| --- | --- |
| `sj-apply` | `implementer-prompt.md`, `task-reviewer-prompt.md`, `re-review-prompt.md`, `writing-good-tests.md` |
| `sj-debug` | `root-cause-tracing.md`, `defense-in-depth.md`, `condition-based-waiting.md` |
| `sj-explore` | `brainstorming.md` |

`sj-apply`, `sj-debug` and `sj-verify` also list the rationalizations that lead agents to
skip a step, and the red flags that mean stop and go back.

[skills/COVERAGE.md](skills/COVERAGE.md) maps every replaced skill and command to
its skill and, for Superpowers, to the file that carries the technique. `cargo test`
checks the skills: frontmatter whose description says when to use the skill, a body of
at most 6,000 bytes, reference files of at most 8,000 bytes that `SKILL.md` names, no
commands from the tools they replace, the `sj` commands each workflow needs, the review
loop and discipline sections above, the brainstorming rules of `sj-explore` (a description
that names brainstorming, a go-wide step, and a reference with a YAGNI pass that presents the
design one section at a time), and complete coverage. Agent adapters install the
ten `sj-*` skills with their reference files; the setup skill is fetched on its own.

## The Claude Code adapter

```bash
sj init --agents claude          # install (writes specjutsu.toml too if it's missing)
sj update                        # regenerate; prints `adapters up to date` when nothing changed
sj update --dry-run              # preview: one `write`, `keep` or `remove` line per file; --json too
sj init --remove-agents claude   # remove everything it added
```

It writes three things and touches nothing else:

- **A block in CLAUDE.md**, between `<!-- specjutsu:begin -->` and
  `<!-- specjutsu:end -->`, appended after your own content. It is under 1,200
  bytes.
- **The ten skills**, under `.claude/skills/sj-*/`: each `SKILL.md` and its reference
  files. Each file ends with a generated-hash line. `sj update` reports a skill you edited by hand
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

`sj update --dry-run` prints the plan without changing anything: one line per
file, `write <path>`, `keep <path>` (a hand-edited file `--force` would replace)
or `remove <path>` (a generated `sj-*` skill this version no longer ships, which
`sj update` deletes unless you edited it). It prints `adapters up to date` when
there is nothing to do and `no adapters installed` when there is no adapter.
`--json` prints `{"files": [{"path": …, "action": …}]}` instead. It always
exits 0.

## Other agents

```bash
sj init --agents codex,cursor    # several at once; AGENTS.md is written once
sj init --agents auto            # the agents whose files the repo already has
sj init --remove-agents codex,cursor,cline
sj init --agents claude,codex --global   # the skills only, in your home folder, for every project
sj doctor                        # one line per installed agent: hook, instructions or hook missing
```

`sj init --agents` takes `claude`, `codex`, `cursor`, `cline`, `copilot`,
`gemini`, `windsurf`, `kiro`, `antigravity`, `agents-md` and `auto`. `auto` picks
Copilot when the repo has `.github/copilot-instructions.md` or `.github/hooks/`,
Gemini CLI for `.gemini/` or `GEMINI.md`, Windsurf for `.windsurf/`, Kiro for
`.kiro/`, and Antigravity for `.agents/rules/` or `.agent/rules/`. Every agent
but Claude Code also gets `agents-md`: a specjutsu block in `AGENTS.md`, with the same markers and rules as the CLAUDE.md block. It names the
`sj` commands of the loop rather than skills, since not every agent that reads
AGENTS.md can load skills. Amp, Roo Code and other AGENTS.md readers get it with
`--agents agents-md`.

| Agent | Instructions | Pre-edit hook | Level |
| --- | --- | --- | --- |
| Claude Code | `CLAUDE.md` block, ten skills | `PreToolUse` in `.claude/settings.json` | `hook` |
| Codex | `AGENTS.md` block; skills in `.agents/skills/` | `[[hooks.PreToolUse]]` for `apply_patch` edits, in `.codex/config.toml` | `hook` |
| Cursor | `AGENTS.md` block | `preToolUse` for `Write` and `Delete`, in `.cursor/hooks.json` | `hook` |
| Cline | `AGENTS.md` block | none: Cline's hooks are TypeScript plugins for its CLI only | `instructions` |
| Copilot (CLI and cloud agent) | `AGENTS.md` block | `PreToolUse` for `Write`, `Edit`, `create` and `edit`, in `.github/hooks/specjutsu.json` | `hook` |
| Gemini CLI | `AGENTS.md` block, added to `context.fileName`; skills in `.agents/skills/` | `BeforeTool` for `write_file` and `replace`, in `.gemini/settings.json` | `hook` |
| Windsurf | `AGENTS.md` block | `pre_write_code`, in `.windsurf/hooks.json` | `hook` |
| Kiro | `AGENTS.md` block | `PreToolUse` for the `write` category, in `.kiro/hooks/specjutsu.json` | `hook` |
| Antigravity | `AGENTS.md` block; skills in `.agents/skills/` | `PreToolUse` for `write_to_file`, `replace_file_content` and `multi_replace_file_content`, as a `specjutsu` hook in `.agents/hooks.json` | `hook` |
| Amp, Roo Code, others | `AGENTS.md` block | none | `instructions` |

Each hook makes the same decision as Claude's and answers in the agent's own
format. Codex sends a whole patch, so `sj hook codex pre-edit` checks every file
in it and blocks the patch if any one is a blocked source file, naming it. Cursor
treats output that isn't JSON as a deny, so `sj hook cursor pre-edit` always
prints `{"permission":"allow"}` or a deny. Gemini CLI and Antigravity read one
JSON decision, `{"decision":"allow"}` or `{"decision":"deny","reason":…}`, so their
hooks always print one. Copilot denies a call on any non-zero exit, so
`sj hook copilot pre-edit` always exits 0: it prints
`{"permissionDecision":"deny",…}` to block and nothing to allow, and checks every
file of an `apply_patch` patch like Codex. Windsurf and Kiro answer by exit code,
like `sj hook check-edit`: 0 to allow, 2 to block with the reason on stderr.
Input a hook can't read allows the edit with a warning.
Codex runs project hooks only in a trusted project, after you trust the hook once with `/hooks`; until then the CI gate is the backstop.

The hook entries are added after yours. `.cursor/hooks.json` is edited as text
like Claude's settings, as are `.gemini/settings.json`, `.windsurf/hooks.json`
and `.agents/hooks.json`. Copilot and Kiro get a file of their own,
`.github/hooks/specjutsu.json` and `.kiro/hooks/specjutsu.json`, deleted on removal.
The Copilot cloud agent reads hooks only from the default branch, which
`sj init --agents copilot` reminds you of.
Codex's hook goes in a `# specjutsu:begin` / `# specjutsu:end` block at the end of `.codex/config.toml`, so removing an
adapter gives every file back byte for byte.

Gemini CLI reads only `GEMINI.md` until `context.fileName` names other files.
When the setting is missing, the adapter writes `["GEMINI.md", "AGENTS.md"]`, so
your `GEMINI.md` still loads; a name or list you set gets `AGENTS.md` added.
Removal takes out only the list it wrote.

For an agent or editor with a hook system specjutsu doesn't support yet, call the
check directly. It exits 0 to allow and 2 to block, with the reason on stderr:

```bash
sj hook check-edit apps/api/src/auth.ts
```

### Shared skills

Codex, Gemini CLI and Antigravity load skills from `.agents/skills/`. Installing
any of them writes the ten `sj-*` skills there, beside Claude Code's
`.claude/skills/` copy, and `sj update` keeps both copies in step. Your own skills
in the folder stay. Removing the last of the three removes the `sj-*` skills
from it, and the folder too if nothing else is left.

### Global skills

`sj init --agents <list> --global` installs only the skills, in
`~/.claude/skills/` for Claude Code and `~/.agents/skills/` for the other
readers, so every project has them. It writes nothing in the repository: hooks
and instruction blocks stay per repository. `sj update --global` refreshes the
global copies and `sj init --remove-agents <list> --global` removes them.
`sj doctor` prints where the skills come from: `skills: project`,
`skills: global` or `skills: project, global`.

## Importing Spec Kit

```bash
sj import speckit           # print the plan: targets, counts and gaps; writes nothing
sj import speckit --write   # convert (exits 2 if a target exists; --force overwrites)
```

Each `specs/NNN-<name>/` feature becomes the specjutsu layout under the spec root:

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
sj migrate --from openspec --from superpowers --diff    # the same, with a unified diff per file
sj migrate --from openspec --from superpowers --write   # apply it (needs a clean git tree)
sj migrate --undo                                       # put back what that write changed
```

`sj migrate` replaces the old tool in one step, so two workflows don't compete
for the agent. `--from` is repeatable: `openspec`, `speckit` or `superpowers`.

- **openspec** keeps `openspec/` (specs, open changes and archive) as the spec
  root and gives every scenario an ID. Before it assigns them, it maps every
  capability without IDs or a mapping to the prefix `sj ids prefixes` suggests
  (see [Short prefixes](#short-prefixes)), in `[ids.prefixes]` of
  `specjutsu.toml`, so `distribution` gets `DIST-001` rather than
  `DISTRIBUTION-001`. When `specjutsu.toml` already exists, the preview shows
  this as `change specjutsu.toml (1 ID prefix)`. The `context`, `rules` and
  `operations` guidance in `openspec/config.yaml` move to
  `openspec/principles.md`, and the file is removed.

  It removes OpenSpec's skills and `/opsx` commands from every agent folder,
  such as `.claude/` and `.agents/`. A file is removed only when its metadata
  says OpenSpec generated it, so your own skills stay. Antigravity's
  `.agents/workflows/opsx-*.md` carry no such metadata, so one goes only when
  its frontmatter holds just a `description` and its body runs the `openspec`
  CLI; every other workflow stays.
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
- installs the Claude Code adapter, plus every adapter `sj init --agents auto`
  would install for the agent files the project has (`AGENTS.md`, `.codex/`,
  `.cursor/`, `.clinerules/`, `.github/hooks/`, `.gemini/`, `GEMINI.md`,
  `.windsurf/`, `.kiro/`, `.agents/rules/`)
- prints `sj doctor` and `sj spec validate`

It exits 0 when both are clean, and 1 with the problems listed otherwise. It
exits 2, changing nothing, when the tree is dirty.

The preview prints one line per file it would create, change or remove. The adapter's
line for `.claude/settings.json` says how many of your hook entries it keeps:

```
change .claude/settings.json (adapter) (merged: 1 existing hook kept)
```

`--diff` adds a unified diff (`--- a/<path>`, `+++ b/<path>`, `@@` hunks) for every
file it would create or change, a new file diffed against nothing. It is preview only
and can't be combined with `--write`:

```diff
--- a/.claude/settings.json
+++ b/.claude/settings.json
@@ -1,13 +1,14 @@
 {
-  "enabledPlugins": {
-    "superpowers@superpowers-marketplace": true
-  },
   "hooks": {
     "PreToolUse": [
       {
         "matcher": "Bash",
         "hooks": [{ "type": "command", "command": "npm run lint" }]
-      }
+      },
+      {"matcher": "Edit|Write|MultiEdit|NotebookEdit", "hooks": [{"type": "command", "command": "sj hook claude pre-edit"}]}
```

The preview works on a copy of the project without its gitignored files, so a step
that reads one, such as a gitignored `.claude/settings.json`, can preview differently
from the real write.

`--write` leaves the changes uncommitted and ends with the counts and the rollback
command:

```
git: 20 created, 2 changed, 0 removed
rollback: sj migrate --undo
```

Review them with `git diff`. The write records the commit it started from and every
file it created, changed or removed, gitignored files included, with each file's hash
and its content before the write, in
`sj-migrate-undo.json` in the git directory (`git rev-parse --git-dir`), so it is
never committed. `sj migrate --undo` then:

- restores every changed or removed file to its content before the write, byte for
  byte, line endings included, and deletes every file migrate created, along with any folder
  that leaves empty;
- touches no other file, so work you added since, such as a new `notes.txt`, stays;
- deletes the manifest, and leaves the result uncommitted:
  `migrate: undone (2 files restored, 20 deleted; left uncommitted)`.

It works after you commit the migration too. It refuses, exiting 2 and changing
nothing, when a recorded file was edited since, and names each one so you can
put it back by hand first. With no manifest it exits 2 with `nothing to undo`.
Only the last `--write` can be undone. Limits: a restored file doesn't get its
executable bit back; and git doesn't track empty folders, so a folder left empty by
the deleted files goes even if it existed, empty, before the migration.

### Adopting a repo that already has OpenSpec specs

When specjutsu runs beside OpenSpec instead of replacing it, three things keep the
two in step:

- **Give a capability IDs before modifying it.** Run `sj ids --write <capability>`
  and commit it on its own. Otherwise a MODIFIED copy with an ID doesn't match its
  original by name, `openspec validate --strict` reads it as a dropped scenario plus
  a new one, and `sj spec validate` says to run the same command.
- **Declare test helper folders as `support`.** `sj adopt` does this for the
  folders it knows; add any other helper path by hand, or the gate fails tests that
  import it as `not-overlaid`. Support files aren't source and are overlaid on base,
  so keep product code out of them.
- **Behaviour that already existed shows as `green:linked`.** A test added for an
  old scenario passes on base, so the gate links it instead of failing it, and
  `sj verify` and `sj task done` accept its passing run. It doesn't cover source
  edits in the same change.

## Roadmap

| Release | Delivers |
| --- | --- |
| v0.1 | `sj trace` on OpenSpec + `cargo test`; non-blocking CI report |
| v0.2 | `sj gate` with red-on-base |
| v0.3 | vitest, Jest, Playwright, pytest and JUnit suites; `specjutsu.toml`; `sj ids`, `sj spec`, `status`, `next`, `verify`; Claude Code adapter and skills; Spec Kit import; `sj migrate` |
| v0.4 | Codex, Cursor and Cline adapters and a shared `AGENTS.md` block; `sj spec show` and `sj view`; `sj instructions` and `sj templates` with live project context and artifact language |
| v0.5 | Copilot, Gemini CLI, Windsurf, Kiro and Antigravity adapters, shared `.agents/skills/` and `--global` skills; bulk archive, `sj update --dry-run`, shell completions and the `sj-onboard` skill; `specjutsu/` as the default spec root |
| v0.5.1 | Trace baseline for gradual adoption, `sj ids prefixes`, `sj migrate --diff` and `--undo`; one-line installers, Homebrew and `cargo binstall`; `sj gain` |
| v0.6 | Public token benchmark against the tools specjutsu replaces (`sj gain` is its local half: byte counts on your machine; the benchmark's agent runs and headline numbers live in a separate repo); `red_on_base = "assert"` and `green:linked` in the gate; `sj ids` for selected capabilities; brainstorming in `sj-explore` |
| Beta | Opens once specjutsu has fully replaced OpenSpec and Superpowers in our own production monorepo |
| v0.7 | Custom workflow schemas (OpenSpec `schema.yaml` read in place); spec stores and worksets for specs kept outside the code repo |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Adding a test runner or an agent adapter
is the intended first PR.

## License

[MIT](LICENSE)
