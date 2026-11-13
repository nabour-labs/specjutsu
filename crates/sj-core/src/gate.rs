//! The CI gate: classification, rules, red-on-base, exemptions and history check.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;
use sj_runners::cargo::Run;

use crate::spec::Fingerprint;
use crate::trace::Report;

/// How a scenario changed between base and head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Change {
    Added,
    Modified,
    Removed,
    Unchanged,
    /// Unchanged text, but untested on base and tested at head (GATE-023).
    Implemented,
}

/// Classify scenario IDs. When an ID appears more than once on one side, the last occurrence wins
/// (a change's delta overrides the current spec).
pub fn classify(_base: &[Fingerprint], _head: &[Fingerprint]) -> BTreeMap<String, Change> {
    todo!("GATE-001, GATE-002")
}

/// Result of running one scenario test against the base code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Red {
    #[serde(rename = "red:assert")]
    Assert,
    #[serde(rename = "red:compile")]
    Compile,
    #[serde(rename = "green-on-base")]
    Green,
    #[serde(rename = "not-run-on-base")]
    NotRun,
    #[serde(rename = "unverifiable")]
    Unverifiable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Exemption {
    pub kind: String,
    pub reason: String,
}

/// Parse exemption lines (`<prefix> <kind> — <reason>`; `-` or `–` also accepted).
pub fn parse_exemptions(_text: &str, _prefix: &str) -> Vec<Exemption> {
    todo!("GATE-015, GATE-016")
}

/// Everything the gate rules need, gathered by [`run`].
#[derive(Debug, Clone)]
pub struct Input {
    pub changes: BTreeMap<String, Change>,
    pub trace: Report,
    /// Repo-relative paths, forward slashes.
    pub changed_files: Vec<String>,
    pub exemptions: Vec<Exemption>,
    /// The `--base` ref as given, e.g. `origin/main`.
    pub base_branch: String,
    pub head_run: Option<Run>,
    /// Base results keyed by (scenario ID, test fn name).
    pub base: BTreeMap<(String, String), Red>,
    pub allow_unverifiable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Proven {
    pub id: String,
    pub red: Red,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GateReport {
    pub pass: bool,
    pub summary: String,
    pub failures: Vec<String>,
    pub warnings: Vec<String>,
    pub exemptions: Vec<String>,
    pub proven: Vec<Proven>,
}

/// Apply the gate rules. Pure: no git, no processes.
pub fn evaluate(_input: &Input) -> GateReport {
    todo!("GATE-003..GATE-019")
}

/// A test file the cargo runner can overlay: a top-level `*.rs` file in a crate's `tests/` dir.
pub fn is_test_file(_rel: &str) -> bool {
    todo!("GATE-011")
}

/// A source file whose change needs a scenario or an exemption.
pub fn is_impl_file(_rel: &str) -> bool {
    todo!("GATE-013")
}

pub struct Options {
    pub root: PathBuf,
    /// Spec root relative to `root`.
    pub spec_root: PathBuf,
    pub base: String,
    pub allow_unverifiable: bool,
}

/// Gather inputs from git and the runner (including red-on-base), then evaluate.
pub fn run(_opts: &Options) -> Result<GateReport> {
    todo!("GATE-007..GATE-011, GATE-020")
}

/// Expand `TRACE-001..006`, `TRACE-001..TRACE-006` and comma-separated lists into IDs.
pub fn expand_ids(_list: &str) -> Vec<String> {
    todo!("GATE-021")
}

/// Check that every ID in a `feat(<IDs>)` subject after `since` has an earlier `test(<IDs>): red`.
pub fn history(_root: &Path, _since: &str) -> Result<GateReport> {
    todo!("GATE-021, GATE-022")
}
