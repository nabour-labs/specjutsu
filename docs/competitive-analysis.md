# Best in class: how specjutsu compares

specjutsu aims to be the best-in-class tool for spec-driven development (SDD). This page
sets out what that means, where specjutsu leads today, and where it doesn't yet, with the
change that closes each gap. It was last checked against each tool's own docs and releases
on 20 November 2026.

## What best in class means

A credible SDD tool in late 2026 needs all of the following:

1. Lightweight, change-centred deltas and living specs.
2. Works with the agents people already use, through skills.
3. A real path for existing codebases, not only greenfield.
4. A clarify and analyse step before code is written.
5. Workflows and templates a team can customise.
6. Specs that can live across repositories.
7. Proof that the code does what the spec says, from something other than an LLM's opinion.

Every tool below covers some of 1–6. None covers 7. That gap is why specjutsu exists.

## The market

| | OpenSpec 2.1 | Spec Kit 1.4 | Kiro | Superpowers | BMAD 6 | **specjutsu** |
| --- | --- | --- | --- | --- | --- | --- |
| Licence | MIT | MIT | Proprietary, paid | MIT | MIT | **MIT** |
| Spec format | Delta specs, WHEN/THEN | Spec, plan, tasks, Given/When/Then | EARS requirements, design, tasks | Plans, no durable spec | Persona briefs, PRD, architecture | **OpenSpec deltas, read unchanged** |
| How the result is checked | LLM verify skill; structure lint | LLM `converge` and `analyze` | Optional property tests, in the IDE | TDD by prompt | Adversarial LLM review | **Scenario IDs traced to tests; red-on-base; `FOR ALL` scenarios need a property test** |
| Test must fail before code | No | No | No | Asked, not enforced | No | **Yes: edits blocked, then proven in CI** |
| CI or PR gate | No | No | No | No | No | **Yes: GitHub Action, one PR comment** |
| Test-tampering checks | No | No | No | No | No | **Yes: `sj audit`** |
| Runtime | Node | Python and uv | IDE or CLI | None | Node | **None** |

Sources: the [OpenSpec releases](https://github.com/Fission-AI/OpenSpec/releases),
[Spec Kit](https://github.github.com/spec-kit/), [Kiro specs](https://kiro.dev/docs/specs/)
and [correctness](https://kiro.dev/docs/specs/correctness/),
[Superpowers](https://github.com/obra/superpowers) and
[BMAD Method](https://github.com/bmad-code-org/BMAD-METHOD). Tessl, once an SDD tool, now
describes itself as an agent enablement platform and is left out.

## Where specjutsu leads today

- **Deterministic proof.** Every scenario has an ID, every ID is traced to a test, and the
  gate runs each new test against the base code to show it failed first. No other tool
  checks the implementation without a model.
- **Enforced test-first.** The agent adapters block source edits until the task's
  scenarios have a red run. Superpowers asks for this in a prompt; specjutsu enforces it.
- **A gate in CI.** The GitHub Action gives the same answer every run and refuses the PR
  until the proof holds.
- **Proven bug fixes.** A `[keep]` scenario pins behaviour a fix must not change, and the
  gate checks its test passes before and after. Kiro and Spec Kit record unchanged
  behaviour too, but check it with a model. See
  [Bug fixes and kept behaviour](../README.md#bug-fixes-and-kept-behaviour).
- **Catches gaming.** `sj audit` flags deleted, skipped and focused tests, lost assertions
  and invented dependencies, with no specs needed.
- **No lock-in.** Existing `openspec/` folders work as they are, Spec Kit imports, and
  `sj migrate` replaces OpenSpec, Spec Kit or Superpowers in one step.
- **Low token cost.** Commands return a few lines instead of raw logs.

## Where specjutsu is behind, and the plan

Most OpenSpec parity work shipped by v0.5 (more agents, `show` and `view`, bulk archive,
completions, onboarding). Custom workflow schemas and multi-repo spec stores are planned
for v0.7. Beyond parity, these changes take ideas that other tools check with a model and
check them deterministically instead. Specs from existing tests ships as `sj spec infer`,
and property scenarios ship as `FOR ALL` scenarios that the gate refuses without a property
test. The gate also shows the shrunk counterexample the property test found on base.

| Gap | Who has it today | How they check it | specjutsu change |
| --- | --- | --- | --- |
| Specs for code that already exists (OpenSpec's most-requested open issue) | cc-sdd, GSD, BMAD | A model reads the code | Ships as `sj spec infer`: it drafts a `[keep]` scenario for every untraced test, and the filled-in change passes the gate as `green:keep` |
| Requirements that hold for all inputs | Kiro | Property tests, IDE-only, optional | Ships: a `FOR ALL` scenario needs a proptest, Hypothesis or fast-check test, and `sj gate` fails it as `not-property` without one. The gate and `sj ledger` show the shrunk counterexample the property test printed as evidence |
| A public, reproducible token benchmark | No one | — | Shipped in v0.6: [the benchmark](https://nabour-labs.github.io/specjutsu-bench/) |

## Not planned

- **Persona agents and unattended epic loops** (BMAD, GSD). They add ceremony, the most
  common complaint about SDD. specjutsu's skills drive one agent through `sj` commands.
- **An extension marketplace** (Spec Kit). Custom workflow schemas cover the need without
  a second package ecosystem.
- **An LLM verify step.** specjutsu never calls a model, so CI gives the same answer every
  time.

## What users complain about, and the answer

- **Specs drift from the code.** Scenarios are traced to tests, and `sj drift` reports
  changed code owned by scenarios the change didn't touch.
- **Agents ignore the spec.** The gate refuses the PR unless every scenario's test failed
  first and passes now.
- **Too much Markdown for small changes.** A change can be one scenario and one test, and
  docs-only or refactor changes take a one-line exemption.
- **Upgrades overwrite customisations.** `sj update` only rewrites the files it generated,
  and leaves edited ones alone.
