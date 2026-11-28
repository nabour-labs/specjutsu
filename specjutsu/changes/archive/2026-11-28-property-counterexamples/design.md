## Context

`property-scenarios` made a FOR ALL scenario need a property test, and only its property
tests count as evidence on base. The failing input those tests find on base is in their
output, but nothing keeps it:

- `run_on_base` in `gate.rs` reads a Cargo target's result lines only. `Run` from
  `sj-runners/cargo.rs` drops failure text, even though `parse_failures` already reads libtest's
  `---- <path> stdout ----` sections for `sj test` excerpts.
- Non-Cargo base runs already keep the failure message in `GateInput.base_messages`, which the
  `not-overlaid` check (GATE-056/062) reads for `red:compile` tests only.
- `Proven`, `Row`, ledger `Record` and `ScenarioStatus` have no field for it.

## Goals / Non-Goals

**Goals:**

- Show the shrunk counterexample of a property test that is red on base in `sj gate` text,
  `--json` and `--format markdown`.
- Keep it on `sj test` ledger records and show it in `sj ledger`.
- The same answer on every run, read from text alone, with no model.

**Non-Goals:**

- Failing the gate when no counterexample can be read. The user chose show-only.
- Seeds and replay paths (fast-check `seed`/`path`, proptest `cc` lines). That could be a later
  change.
- Writing counterexamples back as example tests or regression files.
- Libraries other than proptest, Hypothesis and fast-check.

## Decisions

- **Parse the failure text, not library files.** One function,
  `counterexample(text) -> Option<String>`, tries three patterns in order: proptest
  `minimal failing input:` to the end of its line; Hypothesis `Falsifying example: <name>(`
  through the matching `)`, with the arguments joined by `, ` and the trailing comma
  dropped; fast-check `Counterexample:` to the end of its line. It strips `\r`, turns runs of
  whitespace into one space, and cuts the result to 200 characters, adding `…` (counted by
  `char`, never splitting UTF-8). *Rejected:* reading `proptest-regressions/*.txt` or the
  `.hypothesis/` database. They exist for one library each, live on the base checkout that
  is thrown away, and Hypothesis's database is binary.
- **Read it only from property tests that are `red:assert` on base.** An example test or a
  compile failure can contain the same words by accident (GATE-072), and only property tests
  are evidence for FOR ALL scenarios. *Rejected:* parsing every failing test. It is noisy and
  shows values that weren't shrunk.
- **Cargo base runs keep failure text in the existing `base_messages` map.** `run_on_base`
  calls `parse_failures` on the target's output and stores the text for tests that are
  `Red::Assert`. The `not-overlaid` check reads only `Red::Compile` entries, so its behaviour
  doesn't change. *Rejected:* a second map just for Cargo, which would duplicate the
  plumbing.
- **The text report uses an `info` line.** `info` already holds findings that never fail the
  gate (DRIFT-011), so a counterexample can't be mistaken for a failure. The pass line stays
  as it is. *Rejected:* indented lines under the pass line, a new line kind that CI log
  parsers would have to learn.
- **JSON is an object, `{"test", "input"}`, left out when there is none.** A scenario can have
  several property tests, so the reader needs to know which one found it. This adds a key
  and breaks no consumer.
- **The ledger stores a string on the record and shows the latest red one.** `sj test` already
  has `TestCase.excerpt` text and the traced test's `property` flag. `Record` gains
  `#[serde(default, skip_serializing_if = "Option::is_none")] counterexample`, so old lines
  load and new example-test lines are unchanged. `v` stays the same. *Rejected:* the
  counterexample of the red record that proved the scenario. The latest one is what a
  developer debugging now wants, and it is simpler to compute.

## Risks / Trade-offs

- [A library changes its wording] → the counterexample is missing, never wrong, and the gate
  result doesn't change (GATE-071). Each pattern is one line in one function.
- [Hypothesis output for very large arguments] → cut to 200 characters (GATE-073). The
  FOR ALL scenario GATE-075 proves the parser can't panic or return more than one line on
  any text.
- [Cargo base output is larger to keep] → only the failure sections of target tests that
  failed are kept, which `sj test` already does.
