## ADDED Requirements

### Requirement: Rust property tests are marked
The Rust test scanner SHALL mark a test `property` when the test function is inside a
`proptest!` block or carries a `#[proptest]` attribute. `sj trace --json` SHALL report
`"property": true` on such a test and `"property": false` on every other test.

#### Scenario: TRACE-022 A test inside a proptest block is a property test
- **WHEN** PAY-020 exists and a Rust test file has `proptest! {` followed by `#[test]` and `fn pay_020_refund_le_charge(charge in 0u64..10_000, refund in 0u64..10_000)`
- **THEN** `sj trace --json` links PAY-020 to `pay_020_refund_le_charge` with `"property": true`

#### Scenario: TRACE-023 A proptest attribute marks a property test
- **WHEN** PAY-020 exists and a Rust test file has `#[proptest]` directly above `fn pay_020_refund_le_charge(charge: u64, refund: u64)`
- **THEN** `sj trace --json` links PAY-020 to `pay_020_refund_le_charge` with `"property": true`

### Requirement: Python property tests are marked
The Python test scanner SHALL mark a test `property` when a `@given(...)` or
`@hypothesis.given(...)` decorator is among its decorators and the file imports
`hypothesis` (`import hypothesis` or `from hypothesis import ...`).

#### Scenario: TRACE-024 A Hypothesis test is a property test
- **WHEN** PAY-020 exists and a Python test file has `from hypothesis import given, strategies as st` and `@given(st.integers(), st.integers())` directly above `def test_pay_020_refund_le_charge(charge, refund):`
- **THEN** `sj trace --json` links PAY-020 to `test_pay_020_refund_le_charge` with `"property": true`

### Requirement: TypeScript and JavaScript property tests are marked
The JS/TS test scanner SHALL mark a test `property` when its body calls `<name>.assert(` or
`<name>.property(` or `<name>.asyncProperty(`, where `<name>` is bound by an import or
`require` of `fast-check` in the same file. A call through any other name SHALL NOT mark the
test.

#### Scenario: TRACE-025 A fast-check test is a property test
- **WHEN** PAY-020 exists and a test file has `import fc from 'fast-check'` and `it('PAY-020 refund never exceeds the charge', () => { fc.assert(fc.property(fc.nat(), fc.nat(), check)) })`
- **THEN** `sj trace --json` links PAY-020 to that test with `"property": true`

#### Scenario: TRACE-026 A property call through another name is not a property test
- **WHEN** PAY-020 exists, a test file imports nothing from `fast-check`, and its test `PAY-020 refund never exceeds the charge` calls `helpers.property(check)`
- **THEN** `sj trace --json` links PAY-020 to that test with `"property": false`

### Requirement: FOR ALL scenarios without a property test are listed
`sj trace` SHALL list every FOR ALL scenario whose linked tests are all non-property tests
as `not-property <ID> <test>...`, and `sj trace --json` SHALL list those IDs under
`not_property`. A FOR ALL scenario with no test at all SHALL be a gap, as before, and not
listed as `not-property`. `--strict` SHALL exit 1 when any are listed. Whether a scenario is FOR ALL SHALL
be read from an open change's copy of it when there is one, as the gate does.

#### Scenario: TRACE-027 A FOR ALL scenario with only example tests is listed
- **WHEN** FOR ALL scenario PAY-020 is linked only to `pay_020_refund_of_50_on_100` and that test is not a property test
- **THEN** `sj trace` prints `not-property PAY-020 pay_020_refund_of_50_on_100`, `sj trace --json` lists `PAY-020` under `not_property`, and `sj trace --strict` exits 1

#### Scenario: TRACE-028 A change that adds FOR ALL to a scenario is checked by its copy
- **WHEN** `specs/pay/spec.md` has PAY-020 without a FOR ALL step, an open change MODIFIES PAY-020 to add `- **FOR ALL** charges and refund requests`, and its only test `pay_020_refund_of_50_on_100` is not a property test
- **THEN** `sj trace` prints `not-property PAY-020 pay_020_refund_of_50_on_100`

#### Scenario: TRACE-029 A change that removes FOR ALL from a scenario is not listed
- **WHEN** `specs/pay/spec.md` has PAY-020 with a FOR ALL step, an open change MODIFIES PAY-020 to drop it, and its only test is not a property test
- **THEN** `sj trace` lists no `not-property` line and `sj trace --strict` exits 0
