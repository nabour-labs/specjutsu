//! `cargo test` runner: runs libtest and parses its stable text output.

use std::collections::BTreeMap;
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
    Ok(cargo_test_failures(dir, args, target_dir)?.0)
}

/// [`cargo_test`], also returning each failing test's captured output ([`parse_failures`]).
pub fn cargo_test_failures(
    dir: &Path,
    args: &[&str],
    target_dir: Option<&Path>,
) -> Result<(Run, BTreeMap<String, String>)> {
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
    let run = Run {
        success: out.status.success(),
        results: parse_libtest(&stdout),
    };
    Ok((run, parse_failures(&stdout)))
}

/// A `cargo test` run with build-failure detection and per-test failure output.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Detailed {
    pub run: Run,
    /// The tests did not compile; `run.results` is empty.
    pub build_failed: bool,
    /// Compiler `error` lines and their `-->` locations (capped), when the build failed.
    pub build_errors: Vec<String>,
    /// libtest path → captured failure output, for each failing test.
    pub failures: BTreeMap<String, String>,
    /// Combined stdout and stderr bytes of the cargo processes.
    #[serde(skip)]
    pub output_bytes: u64,
}

/// Most build-error lines kept from cargo's stderr.
pub const BUILD_ERROR_CAP: usize = 40;

/// Build with `cargo test --no-run <cargo_args>`, then run `cargo test <cargo_args> -- <filters>`.
/// A failed build is reported as such instead of as an empty run.
pub fn cargo_test_detailed(dir: &Path, cargo_args: &[&str], filters: &[&str]) -> Result<Detailed> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let build = Command::new(&cargo)
        .args(["test", "--no-run", "--color", "never"])
        .args(cargo_args)
        .current_dir(dir)
        .output()
        .with_context(|| format!("running cargo test --no-run in {}", dir.display()))?;
    if !build.status.success() {
        let stderr = String::from_utf8_lossy(&build.stderr);
        return Ok(Detailed {
            build_failed: true,
            build_errors: parse_build_errors(&stderr, BUILD_ERROR_CAP),
            output_bytes: output_bytes(&build),
            ..Detailed::default()
        });
    }
    let out = Command::new(&cargo)
        .args(["test", "--color", "never"])
        .args(cargo_args)
        .arg("--")
        .args(filters)
        .current_dir(dir)
        .output()
        .with_context(|| format!("running cargo test in {}", dir.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(Detailed {
        run: Run {
            success: out.status.success(),
            results: parse_libtest(&stdout),
        },
        build_failed: false,
        build_errors: Vec::new(),
        failures: parse_failures(&stdout),
        output_bytes: output_bytes(&build) + output_bytes(&out),
    })
}

fn output_bytes(out: &std::process::Output) -> u64 {
    (out.stdout.len() + out.stderr.len()) as u64
}

/// Parse libtest's `---- <path> stdout ----` sections into path → failure text.
pub fn parse_failures(output: &str) -> BTreeMap<String, String> {
    static HEADER: OnceLock<Regex> = OnceLock::new();
    let re = HEADER.get_or_init(|| Regex::new(r"^---- (\S+) std(?:out|err) ----$").unwrap());
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    let flush = |cur: &mut Option<(String, Vec<&str>)>, out: &mut BTreeMap<String, String>| {
        if let Some((path, lines)) = cur.take() {
            let text = lines.join("\n").trim_end().to_string();
            let entry = out.entry(path).or_default();
            if !entry.is_empty() && !text.is_empty() {
                entry.push('\n');
            }
            entry.push_str(&text);
        }
    };
    for line in output.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(c) = re.captures(line) {
            flush(&mut current, &mut out);
            current = Some((c[1].to_string(), Vec::new()));
        } else if matches!(line, "failures:" | "successes:") || line.starts_with("test result:") {
            flush(&mut current, &mut out);
        } else if let Some((_, lines)) = current.as_mut() {
            lines.push(line);
        }
    }
    flush(&mut current, &mut out);
    out
}

/// Keep cargo's `error` lines and the `-->` location line that follows each, up to `cap` lines.
pub fn parse_build_errors(stderr: &str, cap: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut after_error = false;
    for line in stderr.lines() {
        let line = line.trim_end();
        if line.starts_with("error") {
            out.push(line.to_string());
            after_error = true;
        } else if after_error && line.trim_start().starts_with("-->") {
            out.push(line.to_string());
            after_error = false;
        } else if !line.trim().is_empty() && !line.trim_start().starts_with('|') {
            after_error = false;
        }
        if out.len() >= cap {
            out.truncate(cap);
            break;
        }
    }
    out
}
