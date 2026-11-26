//! The agent's working loop: `sj status`, `sj next`, `sj task done` and `sj verify`.

pub mod instructions;
pub mod next;
pub mod status;
pub mod task;
pub mod verify;
pub mod view;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::authoring::{self, blocks, Change};
use crate::git::Git;
use crate::ledger::{self, LedgerResult, Record, ScenarioStatus};

/// Scenario IDs in the ADDED and MODIFIED sections of a change's delta files, in file order,
/// each once.
pub fn targets(change: &Change) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for file in authoring::spec_files(&change.dir.join("specs"))? {
        let text = authoring::read(&file)?;
        let lines: Vec<&str> = text.split('\n').collect();
        let mask = blocks::fence_mask(&lines);
        let d = blocks::parse_delta(&text);
        for b in d.added.iter().chain(&d.modified) {
            for s in blocks::scenarios_in(&lines, &mask, b.lines.clone()) {
                if let Some(id) = s.id {
                    if !out.contains(&id) {
                        out.push(id);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Where a scenario stands in the ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Evidence {
    /// Red before green on a different tree, latest passed.
    Proven,
    /// Latest passed, never proven.
    GreenUnproven,
    /// Latest failed or failed to build.
    Red,
    /// No record, or latest ignored.
    Untested,
}

impl Evidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Evidence::Proven => "proven",
            Evidence::GreenUnproven => "green-unproven",
            Evidence::Red => "red",
            Evidence::Untested => "untested",
        }
    }
}

/// The ledger, read once, answering per-scenario questions.
pub struct Ledger {
    records: Vec<Record>,
    status: BTreeMap<String, ScenarioStatus>,
}

impl Ledger {
    pub fn load(root: &Path) -> Result<Ledger> {
        Ok(Ledger::from_records(ledger::load(root)?.records))
    }

    pub fn from_records(records: Vec<Record>) -> Ledger {
        let status = ledger::report(&records)
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();
        Ledger { records, status }
    }

    /// The latest result for `id` and whether red-before-green is proven, or `None` when no
    /// run names it.
    pub fn status(&self, id: &str) -> Option<&ScenarioStatus> {
        self.status.get(id)
    }

    pub fn evidence(&self, id: &str) -> Evidence {
        match self.status.get(id) {
            None => Evidence::Untested,
            Some(s) if s.latest == LedgerResult::Passed && s.proven => Evidence::Proven,
            Some(s) if s.latest == LedgerResult::Passed => Evidence::GreenUnproven,
            Some(s) if s.latest.is_red() => Evidence::Red,
            Some(_) => Evidence::Untested,
        }
    }

    /// Why `id` is not proven, or `None` when it is.
    pub fn why_unproven(&self, id: &str) -> Option<String> {
        let s = match self.status.get(id) {
            None => return Some("no runs recorded".to_string()),
            Some(s) => s,
        };
        if s.latest != LedgerResult::Passed {
            return Some(format!("latest {}", s.latest.as_str()));
        }
        if s.proven {
            return None;
        }
        let any_red = self
            .records
            .iter()
            .any(|r| r.result.is_red() && r.ids.iter().any(|i| i == id));
        Some(if any_red {
            "red only on the passing tree".to_string()
        } else {
            "no red run recorded".to_string()
        })
    }
}

/// The classification to judge red runs by, or `None` when the merge-base is HEAD itself: a
/// change committed on the default branch has nothing to compare against, so every scenario
/// needs a red run (LOOP-036).
pub fn classification<'a>(
    root: &Path,
    prep: &'a crate::gate::Prepared,
) -> Option<&'a BTreeMap<String, crate::gate::Change>> {
    let head = Git::new(root).resolve("HEAD").ok()?;
    (prep.mb != head).then_some(&prep.changes)
}

/// `not-property` lines for `id` when it is a FOR ALL scenario whose tests are all example
/// tests and is in the gate's scope: added, modified or implemented in `changes`, or any
/// scenario without a classification (GATE-063). Shared by `sj verify` (LOOP-038) and
/// `sj task done` (LOOP-039).
pub(crate) fn not_property(
    report: &crate::trace::Report,
    changes: Option<&BTreeMap<String, crate::gate::Change>>,
    id: &str,
) -> Vec<String> {
    use crate::gate::Change as C;
    let in_scope = changes.map_or(true, |c| {
        matches!(c.get(id), Some(C::Added | C::Modified | C::Implemented))
    });
    if !in_scope || !report.not_property.iter().any(|p| p == id) {
        return Vec::new();
    }
    crate::gate::not_property_failures(id, report.tests_for(id))
}

/// Why `id` is not proven, judged the way the gate classifies it against the merge-base:
/// `changes` is that classification, `None` without a merge-base. Only added and modified
/// scenarios need red before green; an unchanged or implemented one needs only a green latest
/// run (LOOP-034, LOOP-035), and a `[keep]` one needs nothing (LOOP-020). Shared by `sj verify`
/// and `sj task done`.
pub fn why_unproven_against_base(
    ledger: &Ledger,
    keep: &BTreeSet<String>,
    changes: Option<&BTreeMap<String, crate::gate::Change>>,
    id: &str,
) -> Option<String> {
    use crate::gate::Change as Scenario;
    if keep.contains(id) {
        return None;
    }
    let why = ledger.why_unproven(id)?;
    let needs_red = !matches!(
        changes.and_then(|c| c.get(id)),
        Some(Scenario::Unchanged | Scenario::Implemented)
    );
    let green = ledger
        .status(id)
        .is_some_and(|s| s.latest == LedgerResult::Passed);
    (needs_red || !green).then_some(why)
}

/// The base to compare against: `flag`, else the first of `origin/HEAD`, `origin/main`,
/// `origin/master`, `main` and `master` that resolves. `None` outside a git repository or when
/// nothing resolves.
pub fn default_base(root: &Path, flag: Option<&str>) -> Option<String> {
    let git = Git::new(root);
    if !git.is_repo() {
        return None;
    }
    if let Some(f) = flag {
        return Some(f.to_string());
    }
    if let Ok(r) = git.run(&["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"]) {
        let r = r.trim();
        if let Some(short) = r.strip_prefix("refs/remotes/") {
            if git.resolve(short).is_ok() {
                return Some(short.to_string());
            }
        }
    }
    ["origin/main", "origin/master", "main", "master"]
        .into_iter()
        .find(|b| git.resolve(b).is_ok())
        .map(str::to_string)
}

/// The text of a task: its number is the first word.
pub fn task_number(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or("")
}
