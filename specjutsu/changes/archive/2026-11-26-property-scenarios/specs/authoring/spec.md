## ADDED Requirements

### Requirement: A FOR ALL step marks a property scenario
The spec parser SHALL accept a `- **FOR ALL** <inputs>` step in a scenario and MUST store
the scenario as a property. `sj spec show <ID>` SHALL print the FOR ALL step with the other
steps, and `sj spec show <ID> --json` SHALL report `"property": true` for a property
scenario and `"property": false` for any other.

#### Scenario: SPEC-072 A FOR ALL step is parsed
- **WHEN** a delta spec has `#### Scenario: PAY-020 Refund never exceeds the charge` with the steps `- **FOR ALL** charges and refund requests`, `- **WHEN** a refund is issued` and `- **THEN** the refunded total is at most the charge`
- **THEN** `sj spec show PAY-020 --json` reports `"property": true` and lists `FOR ALL charges and refund requests` as the first step

### Requirement: The FOR ALL step comes before WHEN
`sj spec validate` SHALL report a scenario whose FOR ALL step comes after its first WHEN
step, with the file and line, and exit 1. The FOR ALL step names the inputs the WHEN and THEN steps
range over, so it MUST come first.

#### Scenario: SPEC-073 A FOR ALL step after WHEN is reported
- **WHEN** scenario PAY-021 has `- **WHEN** a refund is issued`, then `- **FOR ALL** charges`, then a THEN step
- **THEN** `sj spec validate` prints `PAY-021 FOR ALL step must come before WHEN (<file>:<line>)` and exits 1
