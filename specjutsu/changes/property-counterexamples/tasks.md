# Tasks

## 1. Parser

- [x] 1.1 `counterexample(text)` for proptest, Hypothesis and fast-check; whitespace, cut at 200 characters, never panics (GATE-069, GATE-070, GATE-073, GATE-075)

## 2. Gate

- [ ] 2.1 Cargo base runs keep failure text; `Proven.counterexample` in `--json` and the `info    counterexample` text line, only for property tests that are `red:assert` (GATE-068, GATE-071, GATE-072)
- [ ] 2.2 Evidence cell in `--format markdown` (GATE-074)

## 3. Ledger

- [ ] 3.1 `counterexample` on red records of property tests; old records load (LEDGER-013, LEDGER-015)
- [ ] 3.2 `sj ledger` and `--json` show the latest red counterexample (LEDGER-014)

## 4. Docs

- [ ] 4.1 README (gate output, property scenarios, ledger), CHANGELOG, competitive analysis marks `property-counterexamples` shipped (no-scenario: docs)
