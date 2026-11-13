# specjutsu — Project Kickoff & Design

Nov 13, 2026 · @Jayson

## Overview

**specjutsu is an open-source, single-binary CLI that makes AI coding agents prove their work: every spec scenario gets a test, every test fails first, and CI refuses anything unproven.** It plugs into Claude Code, Cursor, Antigravity, Codex and Copilot from day one, works in any language and any repo shape, and keeps token use low by doing deterministic work in the binary instead of the model.

**Tagline:** Master the art of the spec.

**The problem.** Today's AI-SDLC tools split the job in two. Spec tools (GitHub Spec Kit, OpenSpec) manage *what* to build as Markdown. Discipline tools (Superpowers) manage *how* the agent behaves. Neither checks that the code actually satisfies the spec: the link between a scenario and the test that proves it lives only in prompts. Teams that combine them get two competing workflows, more ceremony and more tokens.

**Goals**

- Spec-to-test traceability that a machine checks, in CI, for every change.
- Real TDD: evidence that each new test failed before the implementation existed.
- One tool for any language (polyglot runners), any repo shape (single repo, monorepo), new or existing projects.
- Native integration with five agent tools on day one.
- Lower token use than any current SDD workflow, measured and reported.

**Non-goals (v1)**

- Not an AI agent or model wrapper; specjutsu never calls an LLM itself.
- Not a replacement for your test framework, CI or issue tracker.
- Not a hosted service; everything runs locally and in your CI.

## Why specjutsu wins

**specjutsu is the only tool in this space that verifies the spec was built, not just written down.** Others hand that job to the model through prompts; specjutsu does it deterministically, in the binary and in CI.

|  | [GitHub Spec Kit](https://github.com/github/spec-kit) | [OpenSpec](https://github.com/Fission-AI/OpenSpec) + TDD | [Superpowers](https://github.com/obra/superpowers) | Superpowers + OpenSpec + TDD | **specjutsu** |
| --- | --- | --- | --- | --- | --- |
| Durable spec | Strong | Strongest (delta specs) | None | Strong | Strong (OpenSpec-compatible deltas) |
| TDD enforced | No | No (DIY) | Yes, by prompt | Yes, loosely tied to spec | Yes, by hook + CI gate |
| Scenario → test trace checked | No | No | No | No | **Yes** |
| Proof the test failed first | No | No | No | No | **Yes (red-on-base)** |
| Existing projects | Via extension | Best | Weak | Good | `adopt` command, incremental |
| Monorepo-aware | Limited | Limited | No | Limited | Per-package specs and runners |
| Runtime needed | Python + uv | Node 20.19+ | None | Node | **None (single binary)** |
| Token cost | High ceremony | Low | Medium | High (two workflows) | **Lowest; measured by `gain`** |
| Reads other tools' specs | No | No | n/a | n/a | **Yes (OpenSpec, Spec Kit)** |

**The moat**

1. **Verification, not prompts.** Markdown workflows are easy to copy; a polyglot trace-and-gate engine with test-runner adapters is not.
2. **Red-on-base proof.** CI runs new tests against the base branch's code; a feature test that already passes there didn't drive the change. No competitor does this.
3. **Interoperability.** specjutsu reads OpenSpec and Spec Kit folders, so adopting it doesn't mean abandoning them. It is the layer that makes their specs trustworthy.
4. **Zero-runtime distribution.** One Rust binary works in a Java, PHP, Go or Python repo without installing Node or Python.
5. **Token economics.** Deterministic commands return a few lines instead of the model reading whole folders and raw test logs.

**Positioning line:** *Keep your spec tool. specjutsu proves the code matches it.*

## Design principles

Six rules decide every design question; when two conflict, the earlier one wins.

1. **Deterministic first.** Anything a program can decide (status, next task, trace, test results, gate) is done by the binary. The model only gets the few lines it needs.
2. **Proof over promises.** A claim that work is done must be backed by evidence specjutsu can check: a mapped test, a red run, a green run.
3. **Language-agnostic.** Test runners are adapters declared in config (`pytest`, `jest`/`vitest`, `go test`, `cargo test`, JUnit/Gradle/Maven, `dotnet test`, PHPUnit, RSpec). Adding a language is a config entry or a small adapter, never a fork.
4. **Repo-agnostic.** Works the same in a single repo or a monorepo (pnpm/npm/yarn workspaces, Cargo workspaces, Go modules, Nx, Turborepo, Bazel, Gradle multi-project). Specs and runners are scoped per package.
5. **Greenfield and brownfield.** `init` for new projects; `adopt` for existing ones, which detects languages, runners and workspaces and lets teams spec only what they change.
6. **Interoperate, don't replace.** Read OpenSpec and Spec Kit folders; emit standard SKILL.md and AGENTS.md; coexist with Superpowers and RTK.

## Architecture

**One Rust binary owns all logic; everything agent-specific is a thin, generated adapter.** Four parts:

- **Core CLI** (`specjutsu`, alias `sj`): config, spec parsing, trace engine, test-runner adapters, gate, evidence ledger.
- **Adapters**: per-agent files the CLI generates and keeps in sync (hooks, skills, rules, AGENTS.md). Regenerated with `sj update`, never hand-edited.
- **Skills**: small SKILL.md files loaded on demand (spec writing, TDD loop, review). The always-on bootstrap stays under \~300 tokens.
- **Evidence ledger**: `.specjutsu/ledger/` records each test run (scenario ID, test ID, result, git tree hash, timestamp) so the gate can prove red-before-green.

&#91;embedded content: architecture · agents, adapters, core, repo and CI\]

Agent tools only ever talk to the binary through generated adapters; CI calls the same binary, so local and CI results always agree.

**Command reference**

| Command | What it does | Who calls it |
| --- | --- | --- |
| `sj init` | Set up a new project: config, spec folder, adapters | Human |
| `sj adopt` | Bring an existing repo in: detect languages, runners, workspaces | Human |
| `sj spec new <change>` | Create a change folder with proposal and scenario stubs | Agent or human |
| `sj status` | Where each change stands, in a few lines | Agent |
| `sj next` | Minimal context for the current task: scenarios, files, test command | Agent |
| `sj test [scope]` | Run the right runner per package; failures-only output; writes ledger | Agent, CI |
| `sj trace` | Map scenario IDs to test IDs; list gaps | Agent, CI |
| `sj gate` | CI check: every scenario tested, red-on-base, all green | CI |
| `sj hook <agent> <event>` | Enforcement entrypoint the adapters call | Agent tool |
| `sj import openspec\|speckit` | Read existing spec folders | Human |
| `sj doctor` | Check setup; suggest companions (RTK, etc.) | Human |
| `sj gain` | Tokens saved, by command and session | Human |
| `sj update` | Regenerate adapters after an upgrade | Human |

All commands support `--json` for tools and a compact default for agents.

## Spec format & traceability

**Specs are plain Markdown with stable scenario IDs; tests reference those IDs, and `sj trace` joins the two.** The format is OpenSpec's own (same folders, same headings) plus one convention: a stable ID at the start of each scenario name. Any OpenSpec repo that follows it is already a specjutsu repo.

```
openspec/  or  specjutsu/        # spec root; set spec_root in specjutsu.toml
  specs/                         # current truth, per capability
    auth/spec.md
  changes/
    add-oauth-login/
      proposal.md                # why + scope
      specs/auth/spec.md         # ADDED / MODIFIED / REMOVED Requirements
      design.md
      tasks.md
specjutsu.toml                   # runners, packages, mode, spec_root
```

In a monorepo each package can hold its own `specjutsu/` folder; the root config lists packages and their runners.

**Scenario syntax** (Given/When/Then, one ID each):

```markdown
### Requirement: Expired tokens are rejected
The system SHALL reject tokens older than 24 hours.

#### Scenario: AUTH-003 Reject expired token
- **WHEN** the client calls GET /me with a token issued 25 hours ago
- **THEN** the response is 401 with code TOKEN_EXPIRED
```

**Linking a test to a scenario**, in whatever the language allows:

- A tag or annotation: `@spec("AUTH-003")` (Java/Kotlin/TS decorators), `#[spec("AUTH-003")]` (Rust), pytest marker `@pytest.mark.spec("AUTH-003")`.
- Or the ID in the test name: `test_AUTH_003_rejects_expired_token`, `it("AUTH-003 rejects expired token")`.
- Or a comment on the line above: `// spec: AUTH-003`. This fallback works in every language.

The trace engine reads test files with lightweight per-language matchers and confirms each match through the runner's report (JUnit XML, JSON or TAP), so a renamed or deleted test shows up as a gap.

**Interop.** specjutsu reads the `openspec/` layout natively, with no import step; `sj ids --write` adds IDs to any scenario that lacks one. `sj import speckit` reads `.specify/specs/NNN-*/spec.md` and its acceptance scenarios.

## TDD enforcement

**CI is always strict; local enforcement is adjustable, because agent tools differ in what their hooks can block.**

| Mode | Local behaviour | Default for |
| --- | --- | --- |
| `advisory` | Hooks warn; nothing blocked | Trying it out, spikes |
| `guided` | Hooks block edits to implementation files until a failing test exists for the active scenario, where the tool allows blocking; otherwise warn | **Default** |
| `strict` | Same as guided, plus: no task marked done without a green ledger entry | Teams that want hard rules |

**The CI gate (`sj gate`)** fails the PR when any of these is true:

1. A scenario added or modified in the change has no mapped test.
2. A mapped test fails.
3. **Red-on-base fails:** a new or changed feature test passes when run against the base branch's code (the test didn't drive the change).
4. A test is mapped to a scenario that no longer exists.

**How red-on-base works.** In CI, `sj gate` checks out the base commit, overlays only the PR's new and changed test files, and runs just those tests. They must fail. It then runs them on the PR head, where they must pass. Cost is bounded because only the changed tests run twice.

**Escape hatches** (recorded, shown in the PR summary):

- `refactor`: no new behaviour; existing tests must stay green, and red-on-base is skipped.
- `docs`, `config`, `chore`: no scenario required.
- `spike`: allowed on branches matching `spike/*`; blocked from merging to main.
- Every exemption needs a one-line reason in the change folder.

## Token efficiency

**The rule: if a program can do it, the binary does it and returns a few lines.** Target for v1.0: at least 50% fewer tokens per completed change than the same task run with Spec Kit or OpenSpec + Superpowers, measured on a public benchmark repo.

- **`sj next` instead of reading folders.** Returns only the active scenarios, relevant file paths and the exact test command, typically under 400 tokens.
- **Failures-only test output.** `sj test` parses runner reports and prints failing tests with the assertion and a short trace. A green run prints one line.
- **Compact status.** `sj status` answers "where am I" in under 10 lines.
- **Lean skills.** The bootstrap loaded every session stays under \~300 tokens; phase skills (spec, TDD loop, review) load only when triggered.
- **Cache-friendly files.** Generated AGENTS.md, CLAUDE.md and rules contain no timestamps or volatile content, so the agent's prompt cache can reuse them.
- **Model hints.** Skills mark mechanical subtasks (scaffold tests, renames, boilerplate) as fine for a cheaper model or subagent.
- **Measure it.** `sj gain` estimates tokens saved per command versus the raw equivalent (e.g. full test log vs failures-only).

**Companions, not dependencies.** `sj doctor` detects and recommends [RTK](https://github.com/rtk-ai/rtk) for git and shell output compression, plus output-brevity skills. When RTK is present, specjutsu leaves general shell compression to it and handles only test and spec output.

## Agent adapters

**`sj init --agents claude,cursor,antigravity,codex,copilot` writes each tool's native files; every hook calls back into `sj hook`, so the logic lives in one place.** Each adapter is also published to that tool's plugin marketplace where one exists.

| Agent | Instructions | Skills | Enforcement hook | Distribution |
| --- | --- | --- | --- | --- |
| Claude Code | CLAUDE.md snippet | `.claude/skills/` | SessionStart (bootstrap) + PreToolUse (block impl edits before red) | Claude plugin marketplace |
| Cursor | `.cursor/rules/` | Plugin skills | `hooks.json` preToolUse | Cursor plugin marketplace (`/add-plugin`) |
| Antigravity | AGENTS.md | Plugin skills | Session-start hook | `agy plugin install` |
| Codex (CLI + app) | AGENTS.md | `.agents/skills/` | To verify; fallback = AGENTS.md rule + CI gate | Codex plugin marketplace |
| GitHub Copilot (CLI + VS Code) | `.github/copilot-instructions.md` | Plugin skills | PreToolUse; CLI can only deny with a suggestion | Copilot plugin marketplace |

Where a tool cannot block an edit, the adapter falls back to a warning plus the CI gate, so no tool can merge unproven work. An adapter conformance test suite (one fixture repo, one scripted session per agent) runs in specjutsu's own CI before each release.

## Repository, stack and releases

**A Cargo workspace in one GitHub repo, released with cargo-dist to every channel developers use.**

```
specjutsu/
  crates/
    sj-cli/          # clap commands, output formatting
    sj-core/         # config, spec parser, trace engine, gate, ledger
    sj-runners/      # test-runner adapters (pytest, jest/vitest, go, cargo, junit, dotnet, phpunit, rspec)
    sj-adapters/     # generators for the 5 agent tools
  skills/            # SKILL.md sources (bootstrap, spec, tdd-loop, review)
  adapters/          # templates per agent
  fixtures/          # sample repos: polyglot, monorepo, brownfield
  docs/              # user docs (also built into the site)
  site/              # marketing website (GitHub Pages)
  .github/workflows/ # ci.yml, release.yml, pages.yml
```

**Stack:** Rust (stable), `clap` (CLI), `serde` + `toml` (config), `pulldown-cmark` (Markdown specs), `quick-xml` (JUnit reports), `ignore` (fast file walking), `insta` (snapshot tests), `assert_cmd` (CLI tests).

**Distribution:**

- GitHub Releases: prebuilt binaries for macOS (arm64, x64), Linux (x64, arm64, musl), Windows (x64).
- Installers: `curl | sh`, PowerShell, Homebrew tap, `cargo install specjutsu`.
- Thin wrappers so people find it in their ecosystem: `npx specjutsu`, `pipx install specjutsu`.
- Agent plugin bundles published per marketplace on each release.

**Versioning:** SemVer; config and spec formats carry a `schema` version so `sj update` can migrate them.

## Open-source setup

**MIT license, a public roadmap, and a contribution path where adding a test runner or an agent adapter is the easy first PR.**

- **License:** MIT, matching Spec Kit, OpenSpec, Superpowers and RTK, so teams can adopt it without legal review friction. (I'm not a lawyer; Apache-2.0 is the alternative if you want an explicit patent grant.)
- **Repo files:** README (pitch, 60-second quickstart, demo GIF), LICENSE, CONTRIBUTING.md, CODE\_OF\_CONDUCT.md (Contributor Covenant), SECURITY.md, GOVERNANCE.md, CHANGELOG.md, issue and PR templates.
- **Dogfooding:** specjutsu is built with OpenSpec until v0.1, then increasingly with itself (see Bootstrapping). Its own `specjutsu/` folder and `sj gate` in CI are the best demo.
- **CI (GitHub Actions):** fmt, clippy, tests on macOS/Linux/Windows, fixture-repo end-to-end runs, adapter conformance suite, `sj gate` on itself.
- **Supply chain:** signed release artifacts and checksums, SBOM per release, Dependabot, `cargo audit` and `cargo deny` in CI, OpenSSF Scorecard badge.
- **Community:** GitHub Discussions (Q&A, ideas, show-and-tell), labels `good first issue` and `runner-adapter` / `agent-adapter`, public project board for the roadmap.
- **Governance:** BDFL (you) to start; add maintainers per area (core, runners, adapters) once regular contributors appear.

## Marketing website (GitHub Pages)

**A static site in `site/`, built by a Pages workflow on every push to main, served at `specjutsu.dev` (or `<user>.github.io/specjutsu` until the domain is bought).** Use Astro Starlight or plain HTML: static, fast, no server, docs and marketing in one place.

**Pages**

| Page | Purpose |
| --- | --- |
| Home | Pitch, 3 pillars, install command, demo GIF |
| Why specjutsu | The comparison table and moat, written for decision-makers |
| Quickstart | Install, `sj init` or `sj adopt`, first red-green change in 5 minutes |
| Docs | Commands, spec format, runners, adapters, CI setup |
| Agents | One page per agent tool: install and what you get |
| Benchmarks | Token and time results on the public benchmark repo |
| Blog | Launch post, release notes, case studies |

**Home page copy (draft)**

- **Hero:** *Master the art of the spec.* AI agents that prove their work. Every scenario tested, every test red first, every merge verified.
- **Install:** `curl -fsSL https://specjutsu.dev/install.sh | sh`
- **Pillar 1 — Proof, not promises.** specjutsu traces every spec scenario to a test and blocks merges that can't prove it.
- **Pillar 2 — Red first, for real.** CI runs new tests against the old code. If they already pass, they didn't drive the change.
- **Pillar 3 — Fewer tokens.** Deterministic commands hand your agent a few lines instead of whole folders and raw logs.
- **Works with:** Claude Code · Cursor · Antigravity · Codex · GitHub Copilot. Any language, any repo, new or existing.
- **Bring your specs:** Already on OpenSpec or Spec Kit? `sj import` and keep going.
- **CTA:** Star on GitHub · Read the quickstart.

**"Why specjutsu" page message:** lead with the one gap every other tool leaves open (nobody checks the spec was built), then the comparison table from this doc, then red-on-base explained with a 20-second animation, then benchmark numbers once they exist. Keep the tone factual: name competitors respectfully and link to them.

## Bootstrapping: OpenSpec first, then specjutsu on itself

**Build specjutsu with OpenSpec now, in a format specjutsu will read unchanged, and hand over one job at a time as each release lands.** Because specjutsu's spec format is OpenSpec's format plus scenario IDs, the switch is a config change, not a migration.

| Stage | Starts when | Specs written with | Verified by | CI |
| --- | --- | --- | --- | --- |
| 0. OpenSpec only | Day 1 | OpenSpec | TDD discipline (optionally the Superpowers TDD skill) | fmt, clippy, `cargo test` |
| 1. specjutsu reads | v0.1 released | OpenSpec | `sj trace`, report only | adds `sj trace` report, non-blocking |
| 2. specjutsu gates | v0.2 released | OpenSpec | `sj gate`, including red-on-base | `sj gate` blocks merges |
| 3. specjutsu only | v0.3 released | specjutsu (`sj spec new` + its own adapters) | `sj gate` | OpenSpec removed |

**Stage 0 setup:**

```bash
npm install -g @fission-ai/openspec@latest
cd specjutsu && openspec init --tools claude
```

**Five conventions from day one** keep every stage-0 change valid for later:

1. **Plain OpenSpec, nothing custom.** Changes live in `openspec/changes/<change>/specs/<capability>/spec.md`, with `### Requirement:` and `#### Scenario:` blocks using WHEN/THEN, as in the [OpenSpec schema docs](https://openspec.dev/docs/schemas/spec-driven).
2. **A stable ID at the start of every scenario name:** `#### Scenario: TRACE-001 Parse scenario IDs from headings`. Use one prefix per capability (TRACE, GATE, LEDGER, RUN, ADAPT) and never reuse an ID.
3. **The ID in the test name:** `fn trace_001_parses_scenario_ids()`. No library needed, and it's the first pattern `sj trace` will match.
4. **Commit the failing test before the code:** `test(TRACE-001): red`, then `feat(TRACE-001): green`. That history lets the v0.2 gate check the earlier work retroactively with `sj gate --since <first commit>`.
5. **One owner per job.** Until stage 3, OpenSpec owns authoring (propose, apply, archive) and specjutsu only reads and checks. Don't install specjutsu's agent adapters on its own repo before stage 3, so two workflows never compete for the agent.

**Never let the code under test grade itself.** CI checks the repo with the last released `sj` binary, pinned by version in the workflow, not the binary built from the PR. A separate job runs the freshly built binary against the fixture repos. Compilers bootstrap the same way (Rust builds itself with the previous release), and it stops a buggy PR from approving itself.

**What this changes in the roadmap:** reading the `openspec/` layout moves into v0.1, and `cargo test` becomes the first runner, because specjutsu is written in Rust and must trace itself first.

## Roadmap

**Ship the verification core first (trace, then gate), widen to all languages and agents, then launch publicly at v0.5.** Week counts are targets from project start, not commitments.

&#91;embedded content: roadmap · 5 releases with exit gates\]

Each phase ends only when its exit gate is met; the public launch waits for the token benchmark so the website's claims are backed by numbers.

## Risks and open questions

**The biggest risk is adapter churn: five agent tools change their plugin and hook formats often.** Mitigation: adapters are thin, generated, and covered by a conformance suite that runs on every release.

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Agent tools change plugin/hook formats | Adapters break | Thin generated adapters; conformance suite; CI gate as backstop |
| Red-on-base is slow in big repos | Developers disable it | Run only new/changed tests; cache builds; allow per-package opt-out with reason |
| Trace matching misses tests in some languages | False gaps, lost trust | Report-based confirmation (JUnit/JSON/TAP); comment-tag fallback works everywhere |
| Ceremony scares off small fixes | Low adoption | Escape hatches; `sj spec new --quick` for one-scenario changes |
| Competitors add verification | Moat narrows | Ship red-on-base first; interoperate so specjutsu is the layer they plug into |
| Name or trademark conflict | Rebrand cost | Check GitHub org, domain and trademarks before launch |

**Open questions**

- [ ] Is the `specjutsu` GitHub org and `specjutsu.dev` domain free?
- [ ] Does Codex support a pre-edit hook today, or only AGENTS.md + skills?
- [ ] Apache-2.0 or MIT?
- [ ] Astro Starlight or plain HTML for the site?
- [ ] Which public repo becomes the token benchmark?

## First steps this week

**Claim the name, stand up the repo, and get `sj trace` working on one language end to end.**

- [ ] Register the `specjutsu` GitHub org and repo; reserve `specjutsu` on crates.io, npm and PyPI with placeholder packages.
- [ ] Buy `specjutsu.dev` (or confirm the github.io fallback).
- [ ] Create the Cargo workspace skeleton (`sj-cli`, `sj-core`, `sj-runners`, `sj-adapters`), MIT license, README, CONTRIBUTING, CI with fmt/clippy/tests.
- [ ] Run `openspec init --tools claude` and write the first change, `openspec/changes/trace-engine/`, with scenarios TRACE-001 onward (parse scenario IDs, match `cargo test` names, report gaps).
- [ ] Implement `sj trace` reading `openspec/` and matching `cargo test` names, test-first, and run it on specjutsu itself as a non-blocking CI report.
- [ ] Generate the Claude Code adapter first (bootstrap skill + PreToolUse hook calling `sj hook`).
- [ ] Publish a placeholder GitHub Pages site with the hero, install command and "coming soon" form.
