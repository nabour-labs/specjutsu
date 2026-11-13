//! `cargo test` runner: runs libtest and parses its stable text output.

use std::path::Path;

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Passed,
    Failed,
    Ignored,
}

/// One libtest result line, e.g. `test tests::foo ... ok`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestResult {
    /// libtest path, e.g. `tests::foo` or `foo`.
    pub path: String,
    pub outcome: Outcome,
}

/// The result of one `cargo test` invocation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Run {
    /// Cargo exited 0.
    pub success: bool,
    pub results: Vec<TestResult>,
}

impl Run {
    /// Outcome of the test whose function name is `name` (matches `name` or `…::name`).
    pub fn outcome(&self, _name: &str) -> Option<Outcome> {
        todo!("GATE-005")
    }
}

/// Parse libtest result lines from captured output.
pub fn parse_libtest(_output: &str) -> Vec<TestResult> {
    todo!("GATE-005")
}

/// Run `cargo test <args>` in `dir`, optionally with a separate target directory.
pub fn cargo_test(_dir: &Path, _args: &[&str], _target_dir: Option<&Path>) -> Result<Run> {
    todo!("GATE-005")
}
