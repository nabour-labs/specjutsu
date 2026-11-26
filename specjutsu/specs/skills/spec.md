# skills Specification

## Purpose
Provide one lean set of agent skills that covers spec-driven work from idea to merge, built on `sj` commands, replacing Superpowers, OpenSpec and Spec Kit skills and commands.

## Requirements

### Requirement: The skill set covers every replaced workflow
specjutsu SHALL ship the skills `sj-explore`, `sj-propose`, `sj-clarify`, `sj-principles`, `sj-apply`, `sj-debug`, `sj-review`, `sj-verify`, `sj-finish` and `sj-onboard`. `skills/COVERAGE.md` SHALL map every Superpowers skill (except `writing-skills` and `using-superpowers`), every OpenSpec workflow command, and every Spec Kit command to exactly one of them.

#### Scenario: SKILL-001 Coverage is complete
- **WHEN** the coverage check reads `skills/COVERAGE.md` and the lists of Superpowers skills, OpenSpec commands (explore, propose, apply, archive, sync, update, onboard) and Spec Kit commands (constitution, specify, clarify, plan, tasks, analyze, checklist, implement)
- **THEN** every listed item maps to one existing `sj-*` skill, and the check fails naming any item that is missing

### Requirement: Skills are small and well-formed
Each skill SHALL be a `SKILL.md` with `name` and `description` frontmatter, a description that says when to use it, and a body of at most 6,000 bytes. A skill folder MAY also hold reference files, `*.md` files next to `SKILL.md`, each at most 8,000 bytes. `SKILL.md` SHALL name every reference file in its folder, and every `*.md` file name that `SKILL.md` names SHALL exist in the folder. Skills and reference files SHALL NOT name `openspec`, `specify`, `/speckit` or `superpowers:` commands.

#### Scenario: SKILL-002 An oversized or foreign skill fails the check
- **WHEN** `sj-propose` grows to 7,000 bytes, or `sj-apply` tells the agent to run `openspec status`
- **THEN** the skill check fails naming the skill and the rule

#### Scenario: SKILL-010 A missing, unnamed or oversized reference fails the check
- **WHEN** `sj-debug/SKILL.md` names `root-cause-tracing.md` but the folder has no such file, the folder holds `notes.md` that `SKILL.md` never names, and the folder holds `defense-in-depth.md` of 9,000 bytes
- **THEN** the skill check reports `sj-debug: names missing reference root-cause-tracing.md`, `sj-debug: reference notes.md is not named in SKILL.md` and `sj-debug: reference defense-in-depth.md is 9000 bytes (limit 8000)`

### Requirement: Skills drive work through sj commands
`sj-apply` SHALL take each task from `sj next`, write the failing test first, and run `sj test <ID>` to record red, then green, before `sj task done`. `sj-debug` SHALL reproduce a bug as a failing test linked to a scenario before any fix. `sj-verify` SHALL run `sj verify` and treat any problem line as not done. `sj-finish` SHALL run `sj verify`, then `sj spec archive`, then open the PR. `sj-propose` SHALL end with `sj spec validate` and `sj spec check` clean. `sj-clarify` SHALL resolve each `[NEEDS CLARIFICATION]` marker that `sj spec check` reports.

#### Scenario: SKILL-003 Each skill names its required commands
- **WHEN** the skill check reads the skills
- **THEN** `sj-apply` names `sj next`, `sj test` and `sj task done`; `sj-debug` names `sj test`; `sj-verify` and `sj-finish` name `sj verify`; `sj-finish` names `sj spec archive`; `sj-propose` names `sj spec new`, `sj spec validate` and `sj spec check`; and `sj-clarify` names `sj spec check`

### Requirement: Principles are part of planning and review
`sj-propose` and `sj-review` SHALL read `<spec_root>/principles.md` when it exists, and record any deviation from a principle, with its reason, in the change's `design.md`. `sj-principles` SHALL create or amend that file.

#### Scenario: SKILL-004 Planning and review reference the principles file
- **WHEN** the skill check reads `sj-propose` and `sj-review`
- **THEN** both name `principles.md` and the design-deviation rule

### Requirement: sj-onboard walks a first change end to end
`sj-onboard` SHALL guide a new user through one complete change on a toy scenario in a scratch change named `sj-onboard-demo`: `sj spec new`, one scenario, `sj test <ID>` red, the smallest code to make it green, `sj test <ID>` green, `sj task done`, and `sj verify`. It SHALL explain each step in one or two sentences before running it. At the end it SHALL remove the scratch change folder, its test and its code, and SHALL NOT run `sj spec archive` on it, so `specs/` is unchanged. It SHALL refuse to start when a change named `sj-onboard-demo` already exists or the working tree has uncommitted changes, and say why.

#### Scenario: SKILL-005 Onboard names the full loop
- **WHEN** the skill check reads `skills/sj-onboard/SKILL.md`
- **THEN** it names `sj spec new`, `sj test`, `sj task done` and `sj verify` in that order, and the scratch change `sj-onboard-demo`

#### Scenario: SKILL-006 Onboard cleans up after itself
- **WHEN** the skill check reads `skills/sj-onboard/SKILL.md`
- **THEN** it tells the agent to delete `changes/sj-onboard-demo/` and the demo test and code at the end, and it does not tell the agent to run `sj spec archive`

#### Scenario: SKILL-007 Onboard refuses a dirty start
- **WHEN** the skill check reads `skills/sj-onboard/SKILL.md`
- **THEN** it tells the agent to stop when `git status` shows uncommitted changes or `changes/sj-onboard-demo/` exists, and it passes the size and naming rules of SKILL-002

### Requirement: The specjutsu skill sets up a project from nothing
specjutsu SHALL ship a setup skill, `skills/specjutsu/SKILL.md`, for an agent that may not have `sj` yet. It SHALL follow the size and naming rules of SKILL-002. It SHALL name, in this order: `sj --version` to check for `sj`, `install.sh` to install it, `sj adopt` for an existing project, `sj init --agents` to install the agent adapters, `sj doctor` and `sj status`. It SHALL name `cargo install specjutsu` for platforms the script does not cover. It SHALL NOT be embedded in the agent adapters: `sj init --agents` installs only the `sj-*` skills.

#### Scenario: SKILL-008 The setup skill names the setup steps in order
- **WHEN** the setup check reads `skills/specjutsu/SKILL.md`
- **THEN** it names `sj --version`, `install.sh`, `sj adopt`, `sj init --agents`, `sj doctor` and `sj status` in that order and names `cargo install specjutsu`, and the check fails naming the first step that is missing or out of order

### Requirement: The setup skill asks before installing and keeps existing config
The setup skill SHALL tell the agent to ask the user before installing `sj`, and to keep an existing `specjutsu.toml` rather than replace it. It SHALL NOT tell the agent to pass `--force`.

#### Scenario: SKILL-009 The setup skill never forces
- **WHEN** the setup check reads a setup skill that tells the agent to run `sj init --force`, or that never mentions asking the user or an existing `specjutsu.toml`
- **THEN** the check fails naming `specjutsu` and the rule

### Requirement: sj-apply reviews every task with a fresh subagent
`sj-apply` SHALL send each task, sequential or `[P]`, to an implementer subagent using `implementer-prompt.md`, then to a fresh task-reviewer subagent using `task-reviewer-prompt.md`. Findings SHALL go back to the implementer and then to a fresh re-reviewer using `re-review-prompt.md`, for at most 3 rounds before the agent asks the user. Only the main agent SHALL run `sj task done`, and only after the reviewer approves. When every task is done, `sj-apply` SHALL run `sj-review` over the whole change in a fresh subagent.

#### Scenario: SKILL-011 sj-apply names the per-task review loop
- **WHEN** the skill check reads an `sj-apply` that has the main agent review the diffs itself, names no `task-reviewer-prompt.md` or `re-review-prompt.md`, and gives no round limit
- **THEN** the check reports `sj-apply: does not name task-reviewer-prompt.md`, `sj-apply: does not name re-review-prompt.md`, `sj-apply: does not send each task to a fresh reviewer subagent` and `sj-apply: does not limit the fix loop to 3 rounds`, and the shipped `sj-apply` reports none of them

#### Scenario: SKILL-012 The task reviewer works from evidence, not the implementer's report
- **WHEN** the skill check reads a `task-reviewer-prompt.md` that passes on the implementer's summary and never names `sj verify`
- **THEN** the check reports `sj-apply: task-reviewer-prompt.md does not name sj verify` and `sj-apply: task-reviewer-prompt.md does not say to ignore the implementer's report`; the shipped prompt names the scenario text, `git diff`, `sj verify`, the question whether the test fails without the change, and the verdicts `APPROVED` and `CHANGES_REQUESTED`

#### Scenario: SKILL-013 The implementer reports a status and never closes the task
- **WHEN** the skill check reads an `implementer-prompt.md` that tells the implementer to run `sj task done`
- **THEN** the check reports `sj-apply: implementer-prompt.md runs sj task done`; the shipped prompt names `sj test` and the statuses `DONE`, `DONE_WITH_CONCERNS`, `BLOCKED` and `NEEDS_CONTEXT`

### Requirement: Discipline skills carry their techniques
`sj-apply`, `sj-debug` and `sj-verify` SHALL each have a `## Rationalizations` section and a `## Red flags` section. `sj-apply` SHALL name `writing-good-tests.md`. `sj-debug` SHALL name `root-cause-tracing.md`, `defense-in-depth.md` and `condition-based-waiting.md`, and SHALL tell the agent to question the design after 3 failed fixes. `sj-verify` SHALL tell the agent that a subagent's report is not evidence.

#### Scenario: SKILL-014 A discipline skill without its sections fails the check
- **WHEN** the skill check reads an `sj-debug` with no `## Red flags` heading and an `sj-verify` with no `## Rationalizations` heading
- **THEN** the check reports `sj-debug: has no ## Red flags section` and `sj-verify: has no ## Rationalizations section`

#### Scenario: SKILL-015 sj-debug names its techniques and the three-fix rule
- **WHEN** the skill check reads an `sj-debug` that names neither `condition-based-waiting.md` nor a limit of 3 failed fixes
- **THEN** the check reports `sj-debug: does not name condition-based-waiting.md` and `sj-debug: does not question the design after 3 failed fixes`

#### Scenario: SKILL-016 sj-verify does not trust a subagent's report
- **WHEN** the skill check reads an `sj-verify` that never says a subagent's report is not evidence
- **THEN** the check reports `sj-verify: does not say a subagent's report is not evidence`

### Requirement: Coverage names where each technique lives
Each Superpowers row in `skills/COVERAGE.md` SHALL have a `Depth` column naming a file in its skill's folder, `SKILL.md` or a reference file, that covers the item. Rows for OpenSpec and Spec Kit MAY leave it empty.

#### Scenario: SKILL-017 A depth file that doesn't exist fails the check
- **WHEN** the Superpowers row `systematic-debugging` maps to `sj-debug` with depth `root-cause.md`, which `sj-debug/` doesn't have, and the row `verification-before-completion` has an empty depth
- **THEN** the check reports `coverage: Superpowers systematic-debugging depth sj-debug/root-cause.md does not exist` and `coverage: Superpowers verification-before-completion has no depth`

### Requirement: sj-explore carries brainstorming
The `sj-explore` description SHALL name brainstorming. Its body SHALL tell the agent to go wide before narrowing, listing many ideas without judging them before choosing the two or three approaches it lays out, and SHALL name `brainstorming.md`. `sj-explore/brainstorming.md` SHALL name a `YAGNI` pass and SHALL tell the agent to present the design one section at a time. The `brainstorming` row of `skills/COVERAGE.md` SHALL have depth `brainstorming.md`.

#### Scenario: SKILL-018 Brainstorming triggers sj-explore
- **WHEN** the skill check reads an `sj-explore` whose description never mentions brainstorming
- **THEN** the check reports `sj-explore: description does not name brainstorming`, and the shipped `sj-explore` reports nothing

#### Scenario: SKILL-019 sj-explore goes wide before narrowing
- **WHEN** the skill check reads an `sj-explore` that names neither `go wide` nor `brainstorming.md`
- **THEN** the check reports `sj-explore: does not go wide before narrowing` and `sj-explore: does not name brainstorming.md`

#### Scenario: SKILL-020 The brainstorming reference carries the technique
- **WHEN** the skill check reads a `sj-explore/brainstorming.md` that names no YAGNI pass and never says to present the design one section at a time
- **THEN** the check reports `sj-explore: brainstorming.md does not name YAGNI` and `sj-explore: brainstorming.md does not present the design one section at a time`; the shipped coverage row `brainstorming` has depth `brainstorming.md` and the shipped skills check clean

### Requirement: sj-propose covers specs drafted from existing tests
The `sj-propose` skill SHALL have a section on drafting specs from existing tests that tells the
agent to run `sj spec infer` on one area at a time, read each test at the file and line its
marker names, and replace both markers with a concrete WHEN and THEN before running
`sj spec check`.

#### Scenario: SKILL-021 sj-propose names the infer workflow
- **WHEN** the installed `sj-propose` skill is read
- **THEN** it has a `From existing tests` section that names `sj spec infer`, `--write`, replacing the `[NEEDS CLARIFICATION` markers from the test, and `sj spec check`

### Requirement: The planning and apply skills describe FOR ALL scenarios
The `sj-propose` skill SHALL explain when to write a FOR ALL step and show its form, and the
`sj-apply` skill SHALL say that a FOR ALL scenario needs a property test and name the
library for each language: proptest for Rust, Hypothesis for Python and fast-check for
TypeScript and JavaScript.

#### Scenario: SKILL-022 The skills name FOR ALL and the property libraries
- **WHEN** the installed `sj-propose` and `sj-apply` skills are read
- **THEN** `sj-propose` shows a `- **FOR ALL**` step, and `sj-apply` names `proptest`, `Hypothesis` and `fast-check` and says a FOR ALL scenario fails the gate as `not-property` without one
