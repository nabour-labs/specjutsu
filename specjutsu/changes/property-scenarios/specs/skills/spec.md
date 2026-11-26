## ADDED Requirements

### Requirement: The planning and apply skills describe FOR ALL scenarios
The `sj-propose` skill SHALL explain when to write a FOR ALL step and show its form, and the
`sj-apply` skill SHALL say that a FOR ALL scenario needs a property test and name the
library for each language: proptest for Rust, Hypothesis for Python and fast-check for
TypeScript and JavaScript.

#### Scenario: SKILL-022 The skills name FOR ALL and the property libraries
- **WHEN** the installed `sj-propose` and `sj-apply` skills are read
- **THEN** `sj-propose` shows a `- **FOR ALL**` step, and `sj-apply` names `proptest`, `Hypothesis` and `fast-check` and says a FOR ALL scenario fails the gate as `not-property` without one
