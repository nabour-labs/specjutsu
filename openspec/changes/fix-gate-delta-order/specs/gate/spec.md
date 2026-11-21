## ADDED Requirements

### Requirement: A change's delta wins on base and head alike
When a scenario ID appears in more than one spec file on one side of the comparison, the text from an open change's delta spec SHALL win over the text in `specs/`, on base and on head alike, so that the same spec tree on both sides never classifies a scenario as modified.

#### Scenario: GATE-045 A merged open change's delta is not a change on later branches
- **WHEN** base holds AUTH-001 in `specs/auth/spec.md` and an open change `rework` whose delta spec modifies AUTH-001's THEN line, and a branch from base changes only `README.md`
- **THEN** the gate classifies AUTH-001 as unchanged, reports no failure for it, and passes
