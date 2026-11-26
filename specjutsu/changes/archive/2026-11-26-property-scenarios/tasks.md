# Tasks

## 1. Spec format
- [x] 1.1 Parse the FOR ALL step and report it in sj spec show (SPEC-072)
- [x] 1.2 Validate that FOR ALL comes before WHEN (SPEC-073)

## 2. Property test detection
- [x] 2.1 [P] Rust: proptest blocks and the proptest attribute (TRACE-022, TRACE-023)
- [x] 2.2 [P] Python: Hypothesis given with the import (TRACE-024)
- [x] 2.3 [P] TS/JS: fast-check calls through an imported name (TRACE-025, TRACE-026)
- [x] 2.4 sj trace lists FOR ALL scenarios without a property test (TRACE-027)

## 3. Gate
- [x] 3.1 Fail FOR ALL scenarios with only example tests (GATE-063)
- [x] 3.2 Count properties on the pass line and in JSON (GATE-064)
- [x] 3.3 Kept FOR ALL scenarios need a property test (GATE-065)
- [x] 3.4 Mixed FOR ALL scenarios: property tests are the evidence, example tests must pass (GATE-066, GATE-067)

## 4. Skills and docs
- [x] 4.1 sj-propose and sj-apply describe FOR ALL (SKILL-022)
- [x] 4.2 README, site, CHANGELOG and competitive analysis (no-scenario: docs)

## 5. Review fixes
- [x] 5.1 sj trace reads FOR ALL from the change's copy, and the gate reuses trace's list (TRACE-028, TRACE-029)
- [x] 5.2 sj verify reports not-property (LOOP-038)
- [x] 5.3 Skill wording on mixed scenarios, design and README corrections (no-scenario: docs)
- [x] 5.4 sj task done refuses a FOR ALL scenario without a property test (LOOP-039)
