---
name: sj-propose
description: Use when a direction is agreed and needs to become a change — a proposal, delta specs with scenario IDs, a design and tasks — or when an open change's plan must be revised. Replaces writing plans, specify, plan, tasks and analyze steps.
---

# Turn a direction into a change

Everything lands in one change folder, and it is checked by `sj` before you hand it back.

## 1. Scaffold

```bash
sj spec new <change-name>      # kebab-case, e.g. add-oauth-login
```

It prints `next <PREFIX>-<NNN>` for every prefix in use. Use those numbers for new scenarios
and never reuse an ID. To revise an existing open change, skip this step and edit its files.

## 2. proposal.md

- **Why:** the problem, in one or two paragraphs.
- **What Changes:** one bullet per change. Mark breaking ones **BREAKING**.
- **Capabilities:** the new and modified capability folders.
- **Impact:** code, APIs and dependencies touched.
- **Exemptions:** a change with no behaviour to test (pure refactor, docs, config) says so on
  its own line: `Exempt: refactor — <reason>`. Kinds are refactor, docs, config, chore and spike.

## 3. Delta specs

One file per capability: `changes/<change>/specs/<capability>/spec.md`, using only these sections:

- `## ADDED Requirements`
- `## MODIFIED Requirements`: copy the whole current requirement, then edit it. Keep every
  scenario name it had.
- `## REMOVED Requirements`: each with `**Reason**` and `**Migration**`.
- `## RENAMED Requirements`: `FROM:` and `TO:` lines.

Each requirement is `### Requirement: <name>`, then a sentence using SHALL or MUST, then at
least one scenario:

```markdown
#### Scenario: AUTH-005 Refresh an expiring token
- **WHEN** the client calls POST /refresh with a token that expires in 5 minutes
- **THEN** the response is 200 with a new token valid for 24 hours
```

A good scenario is one observable behaviour that one test can prove, with concrete values,
not "works correctly". Write questions you can't answer as `[NEEDS CLARIFICATION: …]`.

When a behaviour must hold for every input, not just one example, put a FOR ALL step before
WHEN. Its tests must then include a property test:

```markdown
#### Scenario: PAY-020 Refund never exceeds the charge
- **FOR ALL** charges and refund requests
- **WHEN** a refund is issued
- **THEN** the refunded total is at most the charge
```

## 4. design.md

Context, Goals / Non-Goals, Decisions (each with the alternative you rejected), and
Risks / Trade-offs. Keep it to what a reviewer needs to judge the approach.

**Principles:** if `<spec_root>/principles.md` exists, check the design against each
principle. Record every deviation under `## Principles` in design.md: which principle, what
the design does instead, and why. No deviation means no section.

## 5. tasks.md

Numbered groups of checkbox tasks, each small enough for one red–green cycle:

```markdown
## 2. Tokens
- [ ] 2.1 Refresh endpoint (AUTH-005, AUTH-006)
- [ ] 2.2 [P] Revocation list (AUTH-007)
- [ ] 2.3 README section (no-scenario: docs)
```

- Every task names the scenario IDs it proves (ranges like `AUTH-005..007` work), or says
  `(no-scenario: <reason>)`.
- Every added or modified scenario is named by some task.
- Mark `[P]` on tasks that can run in parallel with their neighbours.
  `sj next --parallel` confirms that their tests don't share files.
- Order tasks so each one builds on finished work.

Delegate: once the scenarios are written, drafting tasks.md from them is mechanical and fine
for a cheaper model or subagent.

## 6. Check it

```bash
sj spec validate <change>      # format, IDs, WHEN/THEN, SHALL, delta structure, task IDs
sj spec check <change>         # uncovered scenarios, unknown IDs, open questions
```

Fix everything they report. Both must be clean before you hand the change back, except
`clarify` lines, which go to `sj-clarify` with the user. Then show the user the proposal and
the scenario list (`sj status <change>`), not the whole folder.

## From existing tests

When a project has tests but no scenarios for them, draft the scenarios from the tests
instead of writing them from scratch.

```bash
sj spec infer <paths> [--capability <name>]          # dry run: the drafts and counts
sj spec infer <paths> [--capability <name>] --write  # writes changes/infer-<capability>/
```

- Run it without `--write` first. Narrow the paths to one area a reviewer can judge.
- `--write` creates `changes/infer-<capability>/` (or `--change <name>`) and adds a
  `spec:` comment above each test.
- For each scenario, open the test at the file and line its marker names. Replace both
  `[NEEDS CLARIFICATION: …]` markers with a concrete WHEN and THEN that the test proves,
  using the test's own values. Rename the heading or merge requirements if the drafted
  names are weak.
- Run `sj spec check <change>` until no `clarify` lines remain, then go on as for any
  change. These markers are answered from the test, not with sj-clarify. The scenarios
  are `[keep]`, so their tests must pass on base and head: don't change the test bodies.
<!-- specjutsu:generated 83d8156bd847752d -->
