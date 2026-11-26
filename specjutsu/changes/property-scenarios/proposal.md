## Why

Some behaviour has to hold for every input, not just the two or three examples a test picks:
a refund never exceeds the charge, a parser round-trips whatever it prints, IDs never
collide. Today a spec can only say this in prose, and the gate accepts any test that names
the scenario, so an example test with one hand-picked value "proves" a claim about all
inputs. Kiro is the one tool that offers property tests, and it checks them only in its IDE,
and only optionally. specjutsu can check this deterministically: the scenario says it holds
for all inputs, and the gate refuses it unless a property test backs it.

This is the first of two changes. It makes `FOR ALL` enforceable. A follow-up,
`property-counterexamples`, keeps the shrunk counterexample from a red run as evidence in
the gate report and the ledger.

## What Changes

- A scenario step `- **FOR ALL** <inputs>` marks a scenario as a property. The spec parser
  stores it as `property`, and `sj spec show --json` reports `"property": true`.
- `sj spec validate` reports a FOR ALL step that comes after the WHEN step.
- The test scanners mark a test `property` when it uses a property-testing library:
  proptest in Rust (`proptest!` blocks and `#[proptest]`), Hypothesis in Python (`@given`
  with `hypothesis` imported) and fast-check in TS/JS (`fc.assert`/`fc.property` through a
  name imported from `fast-check`). `sj trace --json` shows `"property": true` on the test.
- `sj trace` lists a FOR ALL scenario whose tests are all example tests as `not-property`.
- `sj gate` fails a FOR ALL scenario that has no property test with
  `not-property <ID> <test>`, and counts proven properties on the pass line and in `--json`.
- `[keep]` and FOR ALL combine: a kept property needs a property test that passes on base
  and at head.
- The `sj-propose` and `sj-apply` skills explain FOR ALL and the supported libraries.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `authoring`: scenarios may carry a FOR ALL step, and validate checks where it goes.
- `trace`: tests are marked property or not, and FOR ALL scenarios without a property test
  are listed.
- `gate`: FOR ALL scenarios need a property test.
- `skills`: `sj-propose` and `sj-apply` describe FOR ALL.

## Impact

- `crates/sj-core/src/spec.rs` (scenario steps), `testscan/` (Rust, Python and JS
  scanners), `trace.rs`, `gate.rs`, the `sj spec show` and validate code in `authoring/`.
- JSON output gains `property` fields on scenarios, tests and proven entries; existing
  fields don't change.
- No new dependencies. Detection is a static scan of the test source; specjutsu never runs
  or imports the property libraries itself.
- Docs: README (scenario format, trace and gate sections), the site, CHANGELOG, and
  `docs/competitive-analysis.md`.
