//! `cargo test` runner: runs libtest and parses its stable text output.

use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
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
    pub fn outcome(&self, name: &str) -> Option<Outcome> {
        let suffix = format!("::{name}");
        self.results
            .iter()
            .find(|r| r.path == name || r.path.ends_with(&suffix))
            .map(|r| r.outcome)
    }
}

/// Parse libtest result lines from captured output.
pub fn parse_libtest(output: &str) -> Vec<TestResult> {
    static LINE: OnceLock<Regex> = OnceLock::new();
    let re = LINE.get_or_init(|| {
        Regex::new(r"^test (\S+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)").unwrap()
    });
    output
        .lines()
        .filter_map(|l| re.captures(l.trim_end()))
        .map(|c| TestResult {
            path: c[1].to_string(),
            outcome: match &c[2] {
                "ok" => Outcome::Passed,
                "FAILED" => Outcome::Failed,
                _ => Outcome::Ignored,
            },
        })
        .collect()
}

/// Run `cargo test <args>` in `dir`, optionally with a separate target directory.
pub fn cargo_test(dir: &Path, args: &[&str], target_dir: Option<&Path>) -> Result<Run> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("test").args(args).current_dir(dir);
    if let Some(t) = target_dir {
        cmd.env("CARGO_TARGET_DIR", t);
    }
    let out = cmd
        .output()
        .with_context(|| format!("running cargo test in {}", dir.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(Run {
        success: out.status.success(),
        results: parse_libtest(&stdout),
    })
}
