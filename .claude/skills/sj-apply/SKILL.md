---
name: sj-apply
description: Use when implementing a change's tasks — "start implementing", "continue", "work through the tasks". Runs the test-first loop per task with sj next, sj test and sj task done, with an implementer and a fresh reviewer subagent for every task.
---

# Implement a change, test first, reviewed per task

The ledger records every `sj test` run. A scenario counts as proven only when a failing run
was recorded before a passing one on different code, and CI checks it again on the base
branch. The ledger proves the order. It can't prove the test is any good, or that the code
does only what the scenario says. That is what the reviewer is for.

You are the controller. You don't write the code; you hand out tasks, judge the reviews and
close tasks. Keep your context for that.

## The loop, one task at a time

### 1. Get the task

```bash
sj next <change>
git rev-parse HEAD          # BASE: the commit before this task
```

`sj next` prints the task, each scenario's full text, the files changed so far, the
principles file and the test commands. That output is the task brief. Don't paste the whole
change folder, earlier tasks' history or your own summary into it.

### 2. Dispatch the implementer

Start a new subagent with `implementer-prompt.md`, `writing-good-tests.md`, the `sj next`
output, and only the interfaces or decisions from earlier tasks it can't see in the code.
It writes the failing test, records red with `sj test <ID>`, makes it pass, refactors, and
commits `test(<ID>): red` and `feat(<ID>): green`. A `**FOR ALL**` scenario needs a
property test: `proptest` (Rust), `Hypothesis` (Python) or `fast-check` (TS/JS); without
one the gate fails it as `not-property`. It reports one status:

- `DONE`: go to review.
- `DONE_WITH_CONCERNS`: read the concerns. If one is about correctness or scope, settle it
  before review; otherwise pass it to the reviewer.
- `NEEDS_CONTEXT`: answer it fully, then send it back to the same implementer.
- `BLOCKED`: if the scenario is wrong, stop and fix the spec with `sj-propose`. If the task
  is too big, split it in tasks.md. Otherwise tell the user.

### 3. Send it to a fresh reviewer subagent

Every task gets a reviewer that has never seen the implementer's work or reasoning. Start a
new subagent with `task-reviewer-prompt.md` and give it:

- the scenario text from `sj next`,
- the task's commits: `BASE..HEAD` for a task run on its own; for a `[P]` task, only its
  own commits (`git log --format=%H --grep='(<ID>' BASE..HEAD`), since other tasks in the
  batch commit to the same branch,
- the principles file path.

Don't give it the implementer's report, and don't tell it what to ignore or how serious a
finding may be. It runs `git diff` and `sj verify <change>` itself and answers `APPROVED` or
`CHANGES_REQUESTED` with `file:line` findings.

### 4. The fix loop: at most 3 rounds

On `CHANGES_REQUESTED`, note `git rev-parse HEAD` (PREVIOUS_HEAD), then send the findings,
as written, to the implementer. When it reports back, start another fresh subagent with
`re-review-prompt.md`, the earlier findings and the new commits `PREVIOUS_HEAD..HEAD`. It
checks only those findings and the new diff.

The first review is round 1. If the third review still says `CHANGES_REQUESTED`, stop
after 3 rounds without approval. Tell the user which findings are open and what each
side says. Don't close the task, and don't start the next one.

If you think a finding is wrong, check it against the scenario and the code. Say why in the
next round; don't drop it quietly.

### 5. Close the task

Only after `APPROVED`, check the evidence yourself, then close the task:

```bash
sj verify <change>                      # no problem line for this task's scenarios
sj task done <change> <task-number>
```

The reviewer's `APPROVED` is a judgment, not evidence; `sj verify` is the evidence.

The implementer never runs it. In strict mode it refuses until each scenario is proven, and
names the missing evidence. Don't edit the checkbox by hand.

## Parallel tasks

```bash
sj next --parallel
```

It lists open `[P]` tasks in batches whose tests share no file. Dispatch one implementer per
task in a batch at once. Each task still gets its own fresh reviewer subagent and its own
fix loop, and you close each task yourself. Run one batch at a time.

## Tasks with no scenario

A `(no-scenario: docs)` task has nothing to prove red. Do it, or dispatch it, without the
test steps. Review it only if it changes behaviour or files other than its stated kind.

## Final review

When every task is done, start a fresh subagent with `sj-review` over the whole change
(`git diff <base>...HEAD`). Send its blocking and should-fix findings to an implementer
and a fresh re-reviewer, as in the fix loop; there is no task to close. Put any finding
you decide not to fix, with the reason, to the user. Then continue with `sj-verify`.

## Rationalizations

| Thought | Reality |
| --- | --- |
| "The task is small, skip the reviewer." | Small tasks hide the test that asserts nothing. Review it. |
| "I watched the implementer, I can review it." | You saw its reasoning; the reviewer must not. |
| "The ledger says proven, so it's right." | The ledger proves order, not that the test checks the THEN. |
| "The implementer says all tests pass." | A report is not evidence. The reviewer runs `sj verify`. |
| "One more round will fix it." | After 3 rounds the disagreement is the user's to settle. |

## Red flags

Stop and go back to the loop if you notice any of these:

- You're writing implementation code yourself instead of dispatching it.
- You're about to run `sj task done` without an `APPROVED` from a fresh reviewer.
- A reviewer prompt you wrote contains the implementer's summary or "don't flag".
- You're on round 4 of a fix loop.
- A test you didn't touch started failing. Stop and use `sj-debug`.
- The scenario turns out wrong or incomplete. Fix the spec first with `sj-propose`; don't
  implement behaviour no scenario describes.
- Code that needs no scenario (a refactor, a build script) needs an exemption line in the
  proposal. Ask the user before adding one.
<!-- specjutsu:generated 23280722ee6b3b33 -->
