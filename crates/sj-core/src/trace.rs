//! Join scenarios and tests into a trace report.

use std::collections::{BTreeMap, BTreeSet};
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
pub fn build_report(scenarios: &[Scenario], tests: &[TestFn]) -> Report {
    let mut report = Report::default();
    // First occurrence of each ID wins (a change's delta may repeat a current-spec ID).
    let mut order: Vec<&Scenario> = Vec::new();
    let mut known: BTreeSet<&str> = BTreeSet::new();
    for s in scenarios {
        match &s.id {
            None => report.unidentified.push(s.clone()),
            Some(id) => {
                if known.insert(id.as_str()) {
                    order.push(s);
                }
            }
        }
    }
    let prefixes: BTreeSet<&str> = known
        .iter()
        .filter_map(|id| id.rsplit_once('-').map(|(p, _)| p))
        .collect();

    let mut links: BTreeMap<&str, Vec<TestRef>> = BTreeMap::new();
    for t in tests {
        let tref = TestRef {
            name: t.name.clone(),
            file: t.file.clone(),
            line: t.line,
        };
        let mut ids: Vec<(String, bool)> =
            t.comment_ids.iter().map(|i| (i.clone(), true)).collect();
        ids.extend(ids_in_name_deduped(&t.name, &t.comment_ids));
        for (id, explicit) in ids {
            if let Some(k) = known.get(id.as_str()) {
                links.entry(k).or_default().push(tref.clone());
            } else {
                let prefix = id.rsplit_once('-').map_or("", |(p, _)| p);
                if explicit || prefixes.contains(prefix) {
                    report.orphans.push(Orphan {
                        id,
                        test: tref.clone(),
                    });
                }
            }
        }
    }

    for s in order {
        let id = s.id.as_deref().unwrap_or_default();
        match links.remove(id) {
            Some(tests) => report.mapped.push(Mapped {
                id: id.to_string(),
                name: s.name.clone(),
                tests,
            }),
            None => report.gaps.push(s.clone()),
        }
    }
    report
}

fn ids_in_name_deduped(name: &str, already: &[String]) -> Vec<(String, bool)> {
    crate::testscan::ids_in_name(name)
        .into_iter()
        .filter(|id| !already.contains(id))
        .map(|id| (id, false))
        .collect()
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
