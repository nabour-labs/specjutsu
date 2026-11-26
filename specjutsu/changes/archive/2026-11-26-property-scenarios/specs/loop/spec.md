## ADDED Requirements

### Requirement: sj verify reports FOR ALL scenarios without a property test
`sj verify [<change>]` SHALL report every added, modified or implemented FOR ALL scenario
whose mapped tests are all non-property tests as `not-property <ID> <test>`, by the same rule
as the gate, and exit 1. A red-then-green ledger record from an example test SHALL NOT make
such a scenario pass verify.

#### Scenario: LOOP-038 Verify reports a FOR ALL scenario with only example tests
- **WHEN** a change adds FOR ALL scenario PAY-020, its only test `pay_020_refund_of_50_on_100` is not a property test, and the ledger shows it failed and then passed
- **THEN** `sj verify` prints `not-property PAY-020 pay_020_refund_of_50_on_100` and exits 1

### Requirement: sj task done refuses a FOR ALL scenario without a property test
In `strict` mode, `sj task done <change> <task-number>` SHALL refuse, exiting 1 and leaving
`tasks.md` unchanged, when the task names an added, modified or implemented FOR ALL scenario
whose mapped tests are all non-property tests, and print `not-property <ID> <test>` for it as
`sj verify` does, even when the ledger shows red-before-green. In `guided` and `advisory`
modes it SHALL check the box and print the same line as a warning.

#### Scenario: LOOP-039 Strict task done refuses a FOR ALL scenario with only example tests
- **WHEN** mode is `strict`, task 1.1 names FOR ALL scenario PAY-020, its only test `pay_020_refund_of_50_on_100` is not a property test, and the ledger shows it failed and then passed
- **THEN** `sj task done refunds 1.1` prints `not-property PAY-020 pay_020_refund_of_50_on_100`, exits 1 and leaves `tasks.md` unchanged
