## MODIFIED Requirements

### Requirement: Scenarios are classified against the base
The system SHALL compare the scenarios at head with those at the base ref by ID:
an ID absent on base is **added**; an ID present on both whose heading or step
lines differ (ignoring whitespace) is **modified**; an ID present only on base is
**removed**; otherwise **unchanged**. When an ID appears in more than one spec file on one
side, the text from an open change's delta spec SHALL win over the text in `specs/`, on base
and on head alike, so the same tree on both sides never classifies a scenario as modified.

#### Scenario: GATE-001 Classify added, modified, removed and unchanged scenarios
- **WHEN** base has AUTH-001, AUTH-002 and AUTH-003, and head has AUTH-001 unchanged, AUTH-002 with a reworded THEN line, no AUTH-003, and a new AUTH-004
- **THEN** the gate classifies AUTH-004 as added, AUTH-002 as modified, AUTH-003 as removed and AUTH-001 as unchanged

#### Scenario: GATE-002 Moving a scenario between spec files is not a change
- **WHEN** AUTH-001 moves from a change's delta spec into `specs/` with identical text
- **THEN** it is classified as unchanged

#### Scenario: GATE-045 A merged open change's delta is not a change on later branches
- **WHEN** base holds AUTH-001 in `specs/auth/spec.md` and an open change `rework` whose delta spec modifies AUTH-001's THEN line, and a branch from base changes only `README.md`
- **THEN** the gate classifies AUTH-001 as unchanged, reports no failure for it, and passes
