## Context

A scenario is a list of WHEN/THEN/AND steps under a `#### Scenario: <ID> <name>` heading,
and the gate accepts any mapped test that is red on base and green at head. Nothing tells a
claim about one example apart from a claim about every input. The test scanners
(`testscan/mod.rs` for Rust, `js.rs`, `python.rs`) already find each test's start and end
line and mark it `skipped` from what surrounds it, so a property flag belongs there too.

## Goals / Non-Goals

**Goals:**

- A spec author can say a scenario holds for all inputs, in the scenario itself.
- The gate refuses such a scenario unless a property test backs it, with the same answer
  on every run.
- One supported library per language: proptest, Hypothesis and fast-check.

**Non-Goals:**

- Capturing the shrunk counterexample. That is the follow-up change
  `property-counterexamples` (gate report and ledger).
- Writing counterexamples back as pinned regression cases.
- Other libraries (quickcheck, jqwik, `@fast-check/vitest`'s `test.prop`). Each would be one
  more detection rule later, and none of our projects uses one.
- Checking that the property test's generators cover the inputs the FOR ALL step names.
  That needs a model, and specjutsu never calls one.

## Decisions

- **A `- **FOR ALL**` step, not a `[property]` heading marker.** The step says what the
  inputs are, and it reads as part of the scenario. Rejected: a `[property]` marker like
  `[keep]`, which would have to stay in step with the body, and accepting both, which gives
  two ways to write one thing. Adding or removing the step changes the scenario text, so the
  gate already classifies that as a modified scenario.
- **Static detection in the scanners.** A test is a property test when its source uses the
  library: `proptest!` or `#[proptest]`, `@given` with `hypothesis` imported, or a call
  through a name imported from `fast-check`. Rejected: detecting from runtime output, which
  the libraries print only on failure, so a `[keep]` test or a head run could never be
  checked; and an author's marker comment, which anyone can add to an example test.
- **JS/TS must bind the called name to a fast-check import; Python must import hypothesis.**
  In JS/TS, a helper called `property()` would otherwise count. In Python, the file must
  import `hypothesis` (`import hypothesis` or `from hypothesis import ...`) and the test must
  have a `@given(` or `@hypothesis.given(` decorator; the decorator is not bound to the
  import, so a local `given` in a file that also imports hypothesis still counts, which we
  accept. Rust needs no import check, since `proptest!` and `#[proptest]` only come from
  proptest crates.
- **A mixed scenario passes on its property tests.** When a FOR ALL scenario has a property
  test and example tests, the example tests run as usual, but only property tests count as
  evidence. Rejected: requiring every mapped test to be a property test, which would stop
  people from adding a pinned example next to the property.
- **`not-property` is a gate failure, and a trace listing.** It fails like `untested`; trace
  lists it so the agent sees it before CI. Rejected: a warning, which turns FOR ALL into a
  comment; and a per-suite setting, which is config for one value.
- **The pass line adds `; N property` inside the parentheses.** The evidence kinds still sum
  to the total, and the property count reads as a second axis. `--json` adds `property` to
  each `proven` entry and changes no existing field.

## Risks / Trade-offs

- [A property test that ignores its generated input passes as a property test] → `sj audit`
  already flags tests that lost their assertions; generator quality stays a review concern.
- [Aliased or re-exported imports (`import { assert as check } from 'fast-check'`, a local
  `test-utils` module re-exporting `fc`) are missed] → the hint names the library;
  re-exports can be a follow-up if anyone hits them.
- [A test whose callback is a function reference (`it('...', body)`) is not scanned through the
  reference] → only the tokens inside the test call count as its body; the hint names the inline
  form. Resolving references can follow if anyone hits it.
- [Authors put FOR ALL after WHEN] → `sj spec validate` rejects it with the file and line, so
  the step always reads as the scope of the WHEN and THEN that follow.
