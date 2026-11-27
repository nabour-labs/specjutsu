//! The evidence ledger: an append-only JSON Lines record of every `sj test` run.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sj_runners::cargo::Outcome;

use crate::git::Git;
use crate::run::TestRun;

/// Record format version. Lines with any other `v` are skipped on load.
pub const FORMAT: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LedgerResult {
    Passed,
    Failed,
    Ignored,
    BuildFailed,
}

impl LedgerResult {
    pub fn is_red(self) -> bool {
        matches!(self, LedgerResult::Failed | LedgerResult::BuildFailed)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LedgerResult::Passed => "passed",
            LedgerResult::Failed => "failed",
            LedgerResult::Ignored => "ignored",
            LedgerResult::BuildFailed => "build-failed",
        }
    }
}

impl From<Outcome> for LedgerResult {
    fn from(o: Outcome) -> Self {
        match o {
            Outcome::Passed => LedgerResult::Passed,
            Outcome::Failed => LedgerResult::Failed,
            Outcome::Ignored => LedgerResult::Ignored,
        }
    }
}

/// One test's result in one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub v: u32,
    pub run: String,
    /// Unix time in milliseconds.
    pub ts: u64,
    /// Git tree hash of the working tree when the run started.
    pub tree: String,
    pub test: String,
    pub ids: Vec<String>,
    pub result: LedgerResult,
    /// The input a failing property test printed; absent on older and example-test records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counterexample: Option<String>,
}

/// `<root>/.specjutsu/ledger/runs.jsonl`.
pub fn path(root: &Path) -> PathBuf {
    root.join(".specjutsu").join("ledger").join("runs.jsonl")
}

/// Records for a finished run. A build failure is recorded only against the tests a scenario
/// scope selected; a package or workspace build failure is not evidence for any one scenario.
/// When only some suites failed to build, their selected tests are recorded alongside the rest.
pub fn records_for(run: &TestRun, run_id: &str, ts: u64, tree: &str) -> Vec<Record> {
    let make = |test: &str, ids: &[String], result, counterexample: Option<&String>| Record {
        v: FORMAT,
        run: run_id.to_string(),
        ts,
        tree: tree.to_string(),
        test: test.to_string(),
        ids: ids.to_vec(),
        result,
        counterexample: counterexample.cloned(),
    };
    if run.build_failed {
        run.selected
            .iter()
            .map(|(name, ids)| make(name, ids, LedgerResult::BuildFailed, None))
            .collect()
    } else {
        run.tests
            .iter()
            .map(|t| make(&t.path, &t.ids, t.result.into(), t.counterexample.as_ref()))
            .chain(
                run.partial_build_failed
                    .iter()
                    .map(|(name, ids)| make(name, ids, LedgerResult::BuildFailed, None)),
            )
            .collect()
    }
}

/// Append records with a single write, so concurrent runs interleave by whole runs.
pub fn append(root: &Path, records: &[Record]) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }
    let mut buf = String::new();
    for r in records {
        buf.push_str(&serde_json::to_string(r)?);
        buf.push('\n');
    }
    append_lines(&path(root), &buf)
}

/// Append `buf` (whole JSON lines) to the file at `p` with a single write, creating its folder.
/// Shared by the evidence ledger and the gain records.
pub(crate) fn append_lines(p: &Path, buf: &str) -> Result<()> {
    std::fs::create_dir_all(p.parent().unwrap())
        .with_context(|| format!("creating {}", p.parent().unwrap().display()))?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(p)
        .with_context(|| format!("opening {}", p.display()))?;
    f.write_all(buf.as_bytes())
        .with_context(|| format!("writing {}", p.display()))
}

/// Records in file order, plus the number of lines that could not be read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Loaded {
    pub records: Vec<Record>,
    pub skipped: usize,
}

pub fn load(root: &Path) -> Result<Loaded> {
    let (records, skipped) = load_lines(&path(root), |r: &Record| r.v == FORMAT)?;
    Ok(Loaded { records, skipped })
}

/// Read a JSON Lines file: the records that parse and pass `keep`, in file order, and the
/// number of non-blank lines that don't. A missing file is empty.
pub(crate) fn load_lines<T: serde::de::DeserializeOwned>(
    p: &Path,
    keep: impl Fn(&T) -> bool,
) -> Result<(Vec<T>, usize)> {
    let text = match std::fs::read_to_string(p) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), 0)),
        Err(e) => return Err(e).with_context(|| format!("reading {}", p.display())),
    };
    let mut records = Vec::new();
    let mut skipped = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match serde_json::from_str::<T>(line) {
            Ok(r) if keep(&r) => records.push(r),
            _ => skipped += 1,
        }
    }
    Ok((records, skipped))
}

/// What `record` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recorded {
    Records(usize),
    /// The root is not in a git repository, so there is no tree hash; nothing was written.
    NotGit,
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Hash the working tree and append the run's records.
pub fn record(root: &Path, run: &TestRun) -> Result<Recorded> {
    let git = Git::new(root);
    if !git.is_repo() {
        return Ok(Recorded::NotGit);
    }
    let tree = git.worktree_tree_hash()?;
    let ts = now_ms();
    let recs = records_for(run, &format!("{ts}-{}", std::process::id()), ts, &tree);
    append(root, &recs)?;
    Ok(Recorded::Records(recs.len()))
}

/// Where one scenario stands in the ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScenarioStatus {
    pub id: String,
    pub latest: LedgerResult,
    /// A red record came before a passed record on a different tree.
    pub proven: bool,
    /// The counterexample of the scenario's latest red record, if it has one.
    /// It is always the latest red record's value, so a later red record without one clears it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterexample: Option<String>,
}

/// Per-scenario status, sorted by ID. History order is file order.
pub fn report(records: &[Record]) -> Vec<ScenarioStatus> {
    struct Acc<'a> {
        latest: LedgerResult,
        red_trees: BTreeSet<&'a str>,
        proven: bool,
        counterexample: Option<&'a str>,
    }
    let mut by_id: BTreeMap<&str, Acc> = BTreeMap::new();
    for r in records {
        for id in &r.ids {
            let a = by_id.entry(id.as_str()).or_insert(Acc {
                latest: r.result,
                red_trees: BTreeSet::new(),
                proven: false,
                counterexample: None,
            });
            a.latest = r.result;
            if r.result.is_red() {
                a.red_trees.insert(r.tree.as_str());
                a.counterexample = r.counterexample.as_deref();
            } else if r.result == LedgerResult::Passed && a.red_trees.iter().any(|t| *t != r.tree) {
                a.proven = true;
            }
        }
    }
    by_id
        .into_iter()
        .map(|(id, a)| ScenarioStatus {
            id: id.to_string(),
            latest: a.latest,
            proven: a.proven,
            counterexample: a.counterexample.map(str::to_string),
        })
        .collect()
}
