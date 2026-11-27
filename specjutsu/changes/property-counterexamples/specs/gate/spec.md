## ADDED Requirements

### Requirement: The gate shows the counterexample a property test found on base
For each property test that fails its assertion on base, the gate SHALL read the shrunk
counterexample from that test's base failure text: the text after `minimal failing input:`
(proptest), the arguments of `Falsifying example: <test>(…)` joined on one line (Hypothesis),
or the text after `Counterexample:` (fast-check). Runs of whitespace SHALL become one space,
and an input longer than 200 characters SHALL be cut to 200 and end with `…`. The text report
SHALL print `info    counterexample <ID> <test>: <input>`, `--json` SHALL add
`"counterexample": {"test": "<test>", "input": "<input>"}` to the scenario's `proven` entry, and
`--format markdown` SHALL append `(counterexample: <input>)` to its evidence. When a scenario
has more than one, the first of its tests in trace order SHALL be shown. A counterexample is
evidence only: a missing or unreadable one MUST NOT change the gate's result.

#### Scenario: GATE-068 A proptest counterexample is shown in text and JSON
- **WHEN** a change adds FOR ALL scenario PAY-020, and its Cargo property test `pay_020_refund_never_exceeds_charge` fails on base with `Test failed: refund exceeds charge; minimal failing input: charge = 100, refund = 101` and passes at head
- **THEN** `sj gate` passes and prints `info    counterexample PAY-020 pay_020_refund_never_exceeds_charge: charge = 100, refund = 101`, and `--json` lists PAY-020 in `proven` with `"counterexample": {"test": "pay_020_refund_never_exceeds_charge", "input": "charge = 100, refund = 101"}`

#### Scenario: GATE-069 A Hypothesis falsifying example over several lines becomes one line
- **WHEN** the pytest property test `test_pay_020_refund_never_exceeds_charge` for FOR ALL scenario PAY-020 fails on base with `Falsifying example: test_pay_020_refund_never_exceeds_charge(` followed by the lines `    charge=100,`, `    refund=101,` and `)`
- **THEN** the gate reports the counterexample input `charge=100, refund=101`

#### Scenario: GATE-070 A fast-check counterexample is shown
- **WHEN** the vitest property test `PAY-020 refund never exceeds charge` fails on base with `Property failed after 4 tests`, `{ seed: 42, path: "3:0", endOnFailure: true }`, `Counterexample: [100,101]` and `Shrunk 2 time(s)`
- **THEN** the gate reports the counterexample input `[100,101]`

#### Scenario: GATE-071 A property test with no readable counterexample still proves its scenario
- **WHEN** the property test for FOR ALL scenario PAY-020 fails its assertion on base with `assertion failed: refund <= charge` and no counterexample line, and passes at head
- **THEN** the gate passes PAY-020 as `red:assert` with no `info    counterexample` line and no `counterexample` key in its `proven` entry

#### Scenario: GATE-072 Only a property test that fails its assertion on base has a counterexample
- **WHEN** a change adds scenario PAY-021 whose example test fails on base with text containing `Counterexample: [7]`, and FOR ALL scenario PAY-022 whose property test fails to compile on base
- **THEN** neither PAY-021 nor PAY-022 has a counterexample in the text or JSON report

#### Scenario: GATE-073 A long counterexample is cut to 200 characters
- **WHEN** a property test's base failure text holds `minimal failing input: items = [` followed by 300 characters and `]`
- **THEN** the reported input is the first 200 characters of `items = [...]` followed by `…`

#### Scenario: GATE-074 The PR comment shows the counterexample in the evidence cell
- **WHEN** `sj gate --format markdown` runs on the change from GATE-068
- **THEN** PAY-020's row has the evidence `red:assert (counterexample: charge = 100, refund = 101)`

#### Scenario: GATE-075 Reading a counterexample never fails and returns one short line
- **FOR ALL** failure texts of up to 10,000 characters
- **WHEN** the gate reads a counterexample from the text
- **THEN** it returns either nothing or one line with no line break and at most 200 characters plus `…`, without panicking
