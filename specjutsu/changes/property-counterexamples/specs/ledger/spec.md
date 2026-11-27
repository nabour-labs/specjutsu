## ADDED Requirements

### Requirement: Red records of property tests keep their counterexample
When `sj test` records a `failed` result for a property test, the record SHALL hold an
optional `counterexample` with the input read from the test's failure output, following the
gate's rules for counterexamples. `sj ledger` SHALL show the counterexample of a scenario's
latest red record as `(counterexample: <input>)` after its status, and `sj ledger --json`
SHALL include it as `"counterexample"` in the scenario's entry. Records without the field
SHALL load as before, and the ledger format version SHALL NOT change.

#### Scenario: LEDGER-013 A failing property test's record holds its counterexample
- **WHEN** `sj test PAY-020` runs and the proptest test `pay_020_refund_never_exceeds_charge` fails with `minimal failing input: charge = 100, refund = 101`
- **THEN** its ledger record has result `failed` and `"counterexample": "charge = 100, refund = 101"`

#### Scenario: LEDGER-014 sj ledger shows the latest red counterexample
- **WHEN** the ledger holds a `failed` record for PAY-020 with counterexample `charge = 100, refund = 101` on tree A and a later `passed` record on tree B
- **THEN** `sj ledger PAY-020` reports PAY-020 as proven with `(counterexample: charge = 100, refund = 101)`, and `sj ledger --json` has `"counterexample": "charge = 100, refund = 101"` in its entry

#### Scenario: LEDGER-015 Records without a counterexample stay as they are
- **WHEN** the ledger holds a record written before this change and `sj test` records a failure of an example test
- **THEN** both records load, neither has a `counterexample` key, and `sj ledger` shows no counterexample for their scenarios
