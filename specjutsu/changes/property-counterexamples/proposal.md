## Why

A FOR ALL scenario passes the gate as `red:assert` when its property test fails on base, but
the report doesn't say which input broke base. A reviewer can't tell whether the property
found a real case or failed for an unrelated reason. proptest, Hypothesis and fast-check all
print the shrunk input that failed. specjutsu just drops it: the gate's Cargo base run keeps
no failure text, and neither the gate report nor the ledger has anywhere to put it.

This is the follow-up the `property-scenarios` change named. It shows the counterexample
and never makes the gate fail.

## What Changes

- The gate reads the shrunk counterexample from the base failure text of every property test
  that fails its assertion on base: proptest `minimal failing input: …`, Hypothesis
  `Falsifying example: <test>(…)` (spread over several lines or on one), and fast-check
  `Counterexample: …`. It is cut to one line of at most 200 characters.
- The gate's Cargo base run keeps each failing test's failure text, as non-Cargo suites
  already do.
- `sj gate` prints `info    counterexample <ID> <test>: <input>` for each one. `--json` adds
  `"counterexample": {"test": …, "input": …}` to the `proven` entry, and `--format markdown`
  shows it in the scenario's evidence cell.
- A property test with no counterexample the gate can read still counts as proof, the same
  as today. Nothing new can make the gate fail.
- `sj test` stores the counterexample on the ledger record of a property test that fails.
  `sj ledger` and `sj ledger --json` show the counterexample from a scenario's latest red
  record. Ledger lines written before this change still load.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gate`: counterexamples in the text, JSON and markdown reports.
- `ledger`: an optional `counterexample` on red records, shown by `sj ledger`.

## Impact

- `crates/sj-core/src/gate.rs`: keep Cargo base failure text, add a counterexample to
  `Proven` and `Row`, and add the info line.
- `crates/sj-core/src/ci.rs`: the evidence cell in markdown.
- A new counterexample parser module in `sj-core`. It has no new dependencies.
- `crates/sj-core/src/ledger.rs` and `run/`: an optional `counterexample` on `Record`
  (`#[serde(default)]`) and `ScenarioStatus`. The ledger format version stays the same.
- README (gate output, property scenarios, ledger) and CHANGELOG.
