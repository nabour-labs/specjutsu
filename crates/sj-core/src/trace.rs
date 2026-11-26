//! Join scenarios and tests into a trace report.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{bail, Result};
use serde::Serialize;

use crate::config::Runner;
use crate::spec::Scenario;
use crate::testscan::TestFn;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TestRef {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    pub package: Option<String>,
    pub suite: Option<Runner>,
    pub skipped: bool,
    /// A property test (see [`TestFn::property`]).
    pub property: bool,
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
    /// IDs of FOR ALL scenarios whose linked tests are all example tests (none a property test).
    pub not_property: Vec<String>,
    /// Setup problems worth a look, e.g. `no test suite: packages/ui`.
    pub warnings: Vec<String>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.gaps.is_empty() && self.orphans.is_empty()
    }

    /// The tests linked to scenario `id` (empty when it has none).
    pub fn tests_for(&self, id: &str) -> &[TestRef] {
        self.mapped
            .iter()
            .find(|m| m.id == id)
            .map_or(&[], |m| m.tests.as_slice())
    }

    /// Each [`Report::not_property`] ID with its linked (all example) tests.
    pub fn not_property_tests(&self) -> impl Iterator<Item = (&str, &[TestRef])> {
        self.not_property
            .iter()
            .map(|id| (id.as_str(), self.tests_for(id)))
    }
}

/// File name of the accepted-debt list, kept in the spec root.
pub const BASELINE_FILE: &str = "trace-baseline.txt";

/// One accepted gap or orphan in the baseline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BaselineEntry {
    Gap(String),
    /// ID, test file relative to the repo root with forward slashes, test name.
    Orphan(String, String, String),
}

/// The gaps and orphans a repository has accepted, sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Baseline {
    pub entries: BTreeSet<BaselineEntry>,
}

impl Baseline {
    pub fn from_report(report: &Report) -> Self {
        let gaps = report.gaps.iter().filter_map(gap_entry);
        let orphans = report.orphans.iter().map(orphan_entry);
        Baseline {
            entries: gaps.chain(orphans).collect(),
        }
    }

    /// Read the format [`Baseline::render`] writes. Blank lines are skipped; an orphan's test
    /// name is the rest of its line, since JS/TS test names contain spaces. Errors name the line;
    /// the caller adds where the text came from.
    pub fn parse(text: &str) -> Result<Self> {
        let mut entries = BTreeSet::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry = match line.splitn(4, ' ').collect::<Vec<_>>()[..] {
                ["gap", id] => BaselineEntry::Gap(id.to_string()),
                ["orphan", id, file, test] => {
                    BaselineEntry::Orphan(id.to_string(), file.to_string(), test.to_string())
                }
                _ => bail!("line {}: unrecognised baseline entry `{line}`", i + 1),
            };
            entries.insert(entry);
        }
        Ok(Baseline { entries })
    }

    /// A gap is known when the entry [`Baseline::from_report`] would write for it is listed:
    /// a set comparison, so order and duplicates in the file don't matter.
    pub fn knows_gap(&self, gap: &Scenario) -> bool {
        gap_entry(gap).is_some_and(|e| self.entries.contains(&e))
    }

    /// An orphan is known when the entry [`Baseline::from_report`] would write for it is listed.
    pub fn knows_orphan(&self, orphan: &Orphan) -> bool {
        self.entries.contains(&orphan_entry(orphan))
    }

    /// Entries the report no longer has as a gap or orphan, sorted.
    pub fn stale(&self, report: &Report) -> Vec<&BaselineEntry> {
        let current = Baseline::from_report(report).entries;
        self.entries
            .iter()
            .filter(|e| !current.contains(e))
            .collect()
    }

    /// One entry per line, with a trailing newline.
    pub fn render(&self) -> String {
        self.entries.iter().map(|e| format!("{e}\n")).collect()
    }
}

/// An entry as its baseline line, without the newline.
impl std::fmt::Display for BaselineEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaselineEntry::Gap(id) => write!(f, "gap {id}"),
            BaselineEntry::Orphan(id, file, test) => write!(f, "orphan {id} {file} {test}"),
        }
    }
}

fn gap_entry(gap: &Scenario) -> Option<BaselineEntry> {
    gap.id.clone().map(BaselineEntry::Gap)
}

/// The test file is written with forward slashes so the baseline reads the same on every OS.
fn orphan_entry(orphan: &Orphan) -> BaselineEntry {
    let file = orphan
        .test
        .file
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    BaselineEntry::Orphan(orphan.id.clone(), file, orphan.test.name.clone())
}

/// The ID prefixes the scenarios use.
fn id_prefixes(scenarios: &[Scenario]) -> BTreeSet<&str> {
    scenarios
        .iter()
        .filter_map(|s| s.id.as_deref())
        .map(prefix_of)
        .collect()
}

fn prefix_of(id: &str) -> &str {
    id.rsplit_once('-').map_or("", |(p, _)| p)
}

/// The tests [`build_report`] links to no ID, known or orphan: no `spec:` comment or marker
/// IDs, and no name-embedded ID with a prefix the specs use.
pub fn untraced<'a>(scenarios: &[Scenario], tests: &'a [TestFn]) -> Vec<&'a TestFn> {
    let prefixes = id_prefixes(scenarios);
    tests
        .iter()
        .filter(|t| {
            t.comment_ids.is_empty()
                && !t.name_ids.iter().any(|id| prefixes.contains(prefix_of(id)))
        })
        .collect()
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
    let prefixes = id_prefixes(scenarios);
    // Unlike the first-wins order above, FOR ALL is read from a change's copy (TRACE-028/029).
    let property = crate::spec::property_ids(scenarios);

    let mut links: BTreeMap<&str, Vec<TestRef>> = BTreeMap::new();
    for t in tests {
        let tref = TestRef {
            name: t.name.clone(),
            file: t.file.clone(),
            line: t.line,
            package: t.package.clone(),
            suite: t.suite,
            skipped: t.skipped,
            property: t.property,
        };
        let mut ids: Vec<(String, bool)> =
            t.comment_ids.iter().map(|i| (i.clone(), true)).collect();
        ids.extend(
            t.name_ids
                .iter()
                .filter(|id| !t.comment_ids.contains(id))
                .map(|id| (id.clone(), false)),
        );
        for (id, explicit) in ids {
            if let Some(k) = known.get(id.as_str()) {
                links.entry(k).or_default().push(tref.clone());
            } else if explicit || prefixes.contains(prefix_of(&id)) {
                report.orphans.push(Orphan {
                    id,
                    test: tref.clone(),
                });
            }
        }
    }

    for s in order {
        let id = s.id.as_deref().unwrap_or_default();
        match links.remove(id) {
            Some(tests) => {
                if property.contains(id) && !tests.iter().any(|t| t.property) {
                    report.not_property.push(id.to_string());
                }
                report.mapped.push(Mapped {
                    id: id.to_string(),
                    name: s.name.clone(),
                    tests,
                })
            }
            None => report.gaps.push(s.clone()),
        }
    }
    report
}
