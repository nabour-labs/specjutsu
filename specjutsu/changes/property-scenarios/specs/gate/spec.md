## ADDED Requirements

### Requirement: FOR ALL scenarios need a property test
For every added, modified or implemented scenario with a FOR ALL step, the gate MUST require
at least one mapped property test. When every mapped test is a non-property test, the gate
SHALL fail with `not-property <ID> <test>` and a hint naming the property library for that
test's suite (proptest for Cargo, Hypothesis for pytest, fast-check for vitest, Jest and
Playwright). Only the property tests SHALL count as evidence for the scenario: red on base
and green at head are checked on them. The pass line SHALL count proven property
scenarios after the evidence kinds, and `--json` SHALL mark each proven entry with
`"property": true` or `false`.

#### Scenario: GATE-063 A FOR ALL scenario with only example tests fails the gate
- **WHEN** a change adds FOR ALL scenario PAY-020 and its only test, `pay_020_refund_of_50_on_100` in a Cargo suite, is not a property test
- **THEN** the gate fails with reason `not-property PAY-020 pay_020_refund_of_50_on_100` and a hint that names proptest

#### Scenario: GATE-064 A FOR ALL scenario with a property test that is red on base passes
- **WHEN** a change adds FOR ALL scenario PAY-020 and normal scenario PAY-021, each test fails its assertion on base and passes at head, and PAY-020's test is a property test
- **THEN** the gate prints `gate: pass — 2 scenarios proven (2 red:assert; 1 property)`, and `--json` lists PAY-020 in `proven` with `"property": true` and PAY-021 with `"property": false`

#### Scenario: GATE-065 A kept FOR ALL scenario needs a property test
- **WHEN** a change adds `#### Scenario: PAY-022 [keep] Refund never exceeds the charge` with a FOR ALL step, and its only test is a property test that passes on base and at head
- **THEN** the gate passes PAY-022 with evidence `green:keep` and `"property": true`
