//! Join scenarios and tests into a trace report.

use std::path::PathBuf;

use serde::Serialize;

use crate::spec::Scenario;
use crate::testscan::TestFn;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestRef {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mapped {
    pub id: String,
    pub name: String,
    pub tests: Vec<TestRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Orphan {
    pub id: String,
    pub test: TestRef,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub mapped: Vec<Mapped>,
    pub gaps: Vec<Scenario>,
    pub orphans: Vec<Orphan>,
    pub unidentified: Vec<Scenario>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.gaps.is_empty() && self.orphans.is_empty()
    }
}

/// Link tests to scenarios. A name-embedded ID counts as an orphan only when its prefix is one the
/// specs use (avoids noise like `v_2`); `// spec:` comment IDs are explicit and always count.
pub fn build_report(_scenarios: &[Scenario], _tests: &[TestFn]) -> Report {
    todo!("TRACE-005")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::{spec, testscan};

    #[test]
    fn trace_005_report_mapped_gaps_and_orphans() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/trace-basic");
        let scenarios = spec::load_scenarios(&root.join("openspec")).unwrap();
        let tests = testscan::scan_tree(&root).unwrap();
        let r = build_report(&scenarios, &tests);

        let mapped: Vec<&str> = r.mapped.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(mapped, vec!["AUTH-001", "BILL-001"]);
        assert_eq!(r.mapped[0].tests[0].name, "auth_001_logs_in");
        let gaps: Vec<_> = r.gaps.iter().map(|s| s.id.clone().unwrap()).collect();
        assert_eq!(gaps, vec!["AUTH-002"]);
        let orphans: Vec<&str> = r.orphans.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(orphans, vec!["AUTH-009"]);
        assert_eq!(r.unidentified.len(), 1);
        assert!(!r.is_clean());
    }
}
