//! The CI gate: classification, rules, red-on-base, exemptions and history check.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;
use sj_runners::cargo::{cargo_test, Outcome, Run};

use crate::git::Git;
use crate::spec::{self, Fingerprint};
use crate::testscan;
use crate::trace::{self, Report};

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
pub fn classify(base: &[Fingerprint], head: &[Fingerprint]) -> BTreeMap<String, Change> {
    let last = |fps: &[Fingerprint]| -> BTreeMap<String, String> {
        fps.iter().map(|f| (f.id.clone(), f.text.clone())).collect()
    };
    let (base, head) = (last(base), last(head));
    let mut out = BTreeMap::new();
    for (id, text) in &head {
        let change = match base.get(id) {
            None => Change::Added,
            Some(b) if b != text => Change::Modified,
            Some(_) => Change::Unchanged,
        };
        out.insert(id.clone(), change);
    }
    for id in base.keys() {
        if !head.contains_key(id) {
            out.insert(id.clone(), Change::Removed);
        }
    }
    out
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
pub fn parse_exemptions(text: &str, prefix: &str) -> Vec<Exemption> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix(prefix))
        .filter_map(|rest| {
            let rest = rest.trim();
            // Kinds are single words, so the first dash separates kind from reason.
            let (kind, reason) = match rest.find(['—', '–', '-']) {
                Some(i) => {
                    let sep = rest[i..].chars().next().unwrap();
                    (&rest[..i], &rest[i + sep.len_utf8()..])
                }
                None => (rest, ""),
            };
            let kind = kind.trim();
            (!kind.is_empty()).then(|| Exemption {
                kind: kind.to_string(),
                reason: reason.trim().to_string(),
            })
        })
        .collect()
}

const KINDS: [&str; 5] = ["refactor", "docs", "config", "chore", "spike"];

fn is_main(base: &str) -> bool {
    let branch = ["refs/remotes/origin/", "refs/heads/", "origin/"]
        .iter()
        .find_map(|p| base.strip_prefix(p))
        .unwrap_or(base);
    matches!(branch, "main" | "master")
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
pub fn evaluate(input: &Input) -> GateReport {
    let mut r = GateReport::default();

    // Exemptions.
    let mut skip_red = false;
    for e in &input.exemptions {
        r.exemptions.push(format!("{} — {}", e.kind, e.reason));
        if !KINDS.contains(&e.kind.as_str()) {
            r.failures.push(format!("unknown exemption `{}`", e.kind));
        }
        if e.reason.is_empty() {
            r.failures
                .push(format!("exemption needs a reason ({})", e.kind));
        }
        if e.kind == "spike" && is_main(&input.base_branch) {
            r.failures.push("spike cannot merge to main".to_string());
        }
        if e.kind == "refactor" || e.kind == "spike" {
            skip_red = true;
        }
    }
    let refactor = input.exemptions.iter().any(|e| e.kind == "refactor");

    // Head suite must build.
    if let Some(run) = &input.head_run {
        if !run.success && run.results.is_empty() {
            r.failures
                .push("test suite failed to build at head".to_string());
        }
        if refactor {
            for t in &run.results {
                if t.outcome == Outcome::Failed {
                    r.failures.push(format!("failing {}", t.path));
                }
            }
        }
    }

    // Added / modified scenarios.
    let targets: Vec<&String> = input
        .changes
        .iter()
        .filter(|(_, c)| matches!(c, Change::Added | Change::Modified | Change::Implemented))
        .map(|(id, _)| id)
        .collect();
    for id in &targets {
        let Some(m) = input.trace.mapped.iter().find(|m| &&m.id == id) else {
            r.failures.push(format!("untested {id}"));
            continue;
        };
        let mut ok = true;
        let mut weakest: Option<Red> = None;
        for t in &m.tests {
            let head = input.head_run.as_ref().and_then(|run| run.outcome(&t.name));
            match head {
                Some(Outcome::Passed) => {}
                Some(Outcome::Failed) => {
                    ok = false;
                    r.failures.push(format!("failing {id} {}", t.name));
                }
                _ => {
                    ok = false;
                    r.failures.push(format!("not-run {id} {}", t.name));
                }
            }
            if skip_red {
                continue;
            }
            match input.base.get(&(id.to_string(), t.name.clone())) {
                Some(red @ (Red::Assert | Red::Compile)) => {
                    if weakest != Some(Red::Compile) {
                        weakest = Some(*red);
                    }
                }
                Some(Red::Green) => {
                    ok = false;
                    r.failures.push(format!("green-on-base {id} {}", t.name));
                }
                Some(Red::Unverifiable) => {
                    let msg = format!(
                        "unverifiable {id} {} (inline test; move it to tests/ so it can run on base)",
                        t.name
                    );
                    if input.allow_unverifiable {
                        r.warnings.push(msg);
                    } else {
                        ok = false;
                        r.failures.push(msg);
                    }
                }
                Some(Red::NotRun) | None => {
                    ok = false;
                    r.failures.push(format!("not-run-on-base {id} {}", t.name));
                }
            }
        }
        if ok {
            if let Some(red) = weakest {
                r.proven.push(Proven {
                    id: id.to_string(),
                    red,
                });
            }
        }
    }

    // Unchanged gaps only warn.
    for g in &input.trace.gaps {
        if let Some(id) = &g.id {
            if !targets.contains(&id) {
                r.warnings.push(format!("gap {id} {}", g.name));
            }
        }
    }

    for o in &input.trace.orphans {
        r.failures.push(format!("orphan {} {}", o.id, o.test.name));
    }

    if targets.is_empty() && input.exemptions.is_empty() {
        let impl_files: Vec<&str> = input
            .changed_files
            .iter()
            .filter(|f| is_impl_file(f))
            .map(String::as_str)
            .collect();
        if !impl_files.is_empty() {
            r.failures
                .push(format!("unspecified change: {}", impl_files.join(", ")));
        }
    }

    r.pass = r.failures.is_empty();
    r.summary = if r.pass {
        pass_summary(&r.proven)
    } else {
        format!("gate: fail — {}", plural(r.failures.len(), "problem"))
    };
    r
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn pass_summary(proven: &[Proven]) -> String {
    if proven.is_empty() {
        return "gate: pass — no changed scenarios to prove".to_string();
    }
    let n = |k: Red| proven.iter().filter(|p| p.red == k).count();
    let mut kinds = Vec::new();
    for (k, label) in [(Red::Assert, "red:assert"), (Red::Compile, "red:compile")] {
        if n(k) > 0 {
            kinds.push(format!("{} {label}", n(k)));
        }
    }
    format!(
        "gate: pass — {} proven ({})",
        plural(proven.len(), "scenario"),
        kinds.join(", ")
    )
}

/// A test file the cargo runner can overlay: a top-level `*.rs` file in a crate's `tests/` dir.
pub fn is_test_file(rel: &str) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    parts.len() >= 2 && parts[parts.len() - 2] == "tests" && rel.ends_with(".rs")
}

/// Any file under a `tests/` directory (test targets plus their helpers and data); these are
/// what gets overlaid on base.
fn is_test_support(rel: &str) -> bool {
    rel.split('/').rev().skip(1).any(|p| p == "tests")
}

/// A source file whose change needs a scenario or an exemption.
pub fn is_impl_file(rel: &str) -> bool {
    rel.ends_with(".rs")
        && !rel
            .split('/')
            .any(|p| matches!(p, "tests" | "fixtures" | "target"))
}

pub struct Options {
    pub root: PathBuf,
    /// Spec root relative to `root`.
    pub spec_root: PathBuf,
    pub base: String,
    pub allow_unverifiable: bool,
}

/// Gather inputs from git and the runner (including red-on-base), then evaluate.
pub fn run(opts: &Options) -> Result<GateReport> {
    let root = &opts.root;
    let git = Git::new(root);
    let base_sha = git.resolve(&opts.base)?;
    let head_sha = git.resolve("HEAD")?;
    let mb = git.merge_base(&base_sha, &head_sha)?;
    let spec_root = opts.spec_root.to_string_lossy().replace('\\', "/");

    // Classification: base from git objects, head from the working tree.
    let mut base_fps = Vec::new();
    for path in git.ls_tree(&mb)? {
        if spec::is_spec_path(&path, &spec_root) {
            base_fps.extend(spec::fingerprints(&git.show(&mb, &path)?));
        }
    }
    let head_spec_root = root.join(&opts.spec_root);
    let mut head_fps = Vec::new();
    for file in spec::discover_spec_files(&head_spec_root)? {
        head_fps.extend(spec::fingerprints(&std::fs::read_to_string(&file)?));
    }
    let mut changes = classify(&base_fps, &head_fps);

    let scenarios = spec::load_scenarios(&head_spec_root)?;
    let tests = testscan::scan_tree(root)?;
    let report = trace::build_report(&scenarios, &tests);

    // GATE-023: unchanged scenarios that gain their first test are in scope.
    let base_mapped = base_mapped_ids(&git, &mb, &spec_root)?;
    for m in &report.mapped {
        if let Some(c) = changes.get_mut(&m.id) {
            if *c == Change::Unchanged && !base_mapped.contains(&m.id) {
                *c = Change::Implemented;
            }
        }
    }
    let changed_files = git.changed_files(&mb)?;

    // Exemptions: proposals of touched change folders, then commit trailers.
    let mut exemptions = Vec::new();
    let touched: BTreeSet<&str> = changed_files
        .iter()
        .filter_map(|f| f.strip_prefix(&format!("{spec_root}/changes/")))
        .filter_map(|rest| rest.split_once('/').map(|(c, _)| c))
        .filter(|c| *c != "archive")
        .collect();
    for change in touched {
        let proposal = head_spec_root
            .join("changes")
            .join(change)
            .join("proposal.md");
        if let Ok(text) = std::fs::read_to_string(proposal) {
            exemptions.extend(parse_exemptions(&text, "Exempt:"));
        }
    }
    for c in git.log(&format!("{mb}..{head_sha}"))? {
        exemptions.extend(parse_exemptions(&c.body, "Specjutsu-Exempt:"));
    }

    let targets: Vec<&String> = changes
        .iter()
        .filter(|(_, c)| matches!(c, Change::Added | Change::Modified | Change::Implemented))
        .map(|(id, _)| id)
        .collect();
    let refactor = exemptions.iter().any(|e| e.kind == "refactor");
    let skip_red = refactor || exemptions.iter().any(|e| e.kind == "spike");

    let head_run = if !targets.is_empty() || refactor {
        Some(cargo_test(root, &["--workspace", "--no-fail-fast"], None)?)
    } else {
        None
    };

    let mut base = BTreeMap::new();
    if !skip_red && !targets.is_empty() {
        let rel = |p: &Path| -> String {
            p.strip_prefix(root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/")
        };
        // (scenario, test name, test file) for every target test.
        let mut jobs: Vec<(String, String, String)> = Vec::new();
        for m in report.mapped.iter().filter(|m| targets.contains(&&m.id)) {
            for t in &m.tests {
                jobs.push((m.id.clone(), t.name.clone(), rel(&t.file)));
            }
        }
        for (id, name, file) in &jobs {
            if !is_test_file(file) {
                base.insert((id.clone(), name.clone()), Red::Unverifiable);
            }
        }
        let overlay: Vec<&String> = changed_files
            .iter()
            .filter(|f| is_test_support(f) && root.join(f).is_file())
            .collect();
        let verifiable: Vec<&(String, String, String)> =
            jobs.iter().filter(|(_, _, f)| is_test_file(f)).collect();
        if !verifiable.is_empty() {
            let results = run_on_base(&git, root, &mb, &overlay, &verifiable)?;
            base.extend(results);
        }
    }

    Ok(evaluate(&Input {
        changes,
        trace: report,
        changed_files,
        exemptions,
        base_branch: opts.base.clone(),
        head_run,
        base,
        allow_unverifiable: opts.allow_unverifiable,
    }))
}

/// Scenario IDs that already had a linked test at `rev`, read from git objects.
fn base_mapped_ids(git: &Git, rev: &str, spec_root: &str) -> Result<BTreeSet<String>> {
    let mut scenarios = Vec::new();
    let mut tests = Vec::new();
    for path in git.ls_tree(rev)? {
        if spec::is_spec_path(&path, spec_root) {
            scenarios.extend(spec::parse_scenarios(
                Path::new(&path),
                &git.show(rev, &path)?,
            ));
        } else if path.ends_with(".rs")
            && !path.split('/').any(|p| matches!(p, "target" | "fixtures"))
        {
            tests.extend(testscan::scan_rust(
                Path::new(&path),
                &git.show(rev, &path)?,
            ));
        }
    }
    Ok(trace::build_report(&scenarios, &tests)
        .mapped
        .into_iter()
        .map(|m| m.id)
        .collect())
}

/// Check out `rev` in a temporary worktree, overlay the change's test files, and run each test
/// target that holds a scenario test.
fn run_on_base(
    git: &Git,
    root: &Path,
    rev: &str,
    overlay: &[&String],
    jobs: &[&(String, String, String)],
) -> Result<BTreeMap<(String, String), Red>> {
    let tmp = tempfile::tempdir().context("creating temp dir for base checkout")?;
    let wt = tmp.path().join("base");
    git.worktree_add(&wt, rev)?;
    let result = (|| -> Result<BTreeMap<(String, String), Red>> {
        for f in overlay {
            let dest = wt.join(f);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(root.join(f), &dest)
                .with_context(|| format!("overlaying {f} on base"))?;
        }
        let target_dir = root.join("target").join("sj-base");
        // One cargo invocation per test target: (crate dir, target name).
        let mut runs: BTreeMap<(String, String), Run> = BTreeMap::new();
        let mut out = BTreeMap::new();
        for (id, name, file) in jobs {
            let (crate_dir, stem) = split_test_target(file);
            let key = (crate_dir.clone(), stem.clone());
            if !runs.contains_key(&key) {
                let dir = wt.join(&crate_dir);
                let run = if dir.join("Cargo.toml").is_file() {
                    cargo_test(
                        &dir,
                        &["--no-fail-fast", "--test", &stem],
                        Some(&target_dir),
                    )?
                } else {
                    Run::default()
                };
                runs.insert(key.clone(), run);
            }
            let run = &runs[&key];
            let red = match run.outcome(name) {
                Some(Outcome::Passed) => Red::Green,
                Some(Outcome::Failed) => Red::Assert,
                Some(Outcome::Ignored) => Red::NotRun,
                None if !run.success => Red::Compile,
                None => Red::NotRun,
            };
            out.insert((id.clone(), name.clone()), red);
        }
        Ok(out)
    })();
    let cleanup = git.worktree_remove(&wt);
    let out = result?;
    cleanup?;
    Ok(out)
}

/// `crates/sj-core/tests/gate.rs` → (`crates/sj-core`, `gate`); `tests/math.rs` → (``, `math`).
fn split_test_target(file: &str) -> (String, String) {
    let (dir, name) = file.rsplit_once('/').unwrap_or(("", file));
    let crate_dir = dir
        .strip_suffix("tests")
        .unwrap_or(dir)
        .trim_end_matches('/');
    (
        crate_dir.to_string(),
        name.trim_end_matches(".rs").to_string(),
    )
}

/// Expand `TRACE-001..006`, `TRACE-001..TRACE-006` and comma-separated lists into IDs.
pub fn expand_ids(list: &str) -> Vec<String> {
    static RANGE: OnceLock<Regex> = OnceLock::new();
    let re = RANGE.get_or_init(|| {
        Regex::new(r"^([A-Z][A-Z0-9]*)-([0-9]+)(?:\.\.(?:[A-Z][A-Z0-9]*-)?([0-9]+))?$").unwrap()
    });
    let mut out = Vec::new();
    for item in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let Some(c) = re.captures(item) else { continue };
        let (prefix, start) = (&c[1], &c[2]);
        let width = start.len();
        let a: u64 = start.parse().unwrap_or(0);
        let b: u64 = c.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(a);
        for n in a..=b.max(a) {
            out.push(format!("{prefix}-{n:0width$}"));
        }
    }
    out
}

/// Check that every ID in a `feat(<IDs>)` subject after `since` has an earlier `test(<IDs>): red`.
pub fn history(root: &Path, since: &str) -> Result<GateReport> {
    static SUBJECT: OnceLock<Regex> = OnceLock::new();
    let re = SUBJECT.get_or_init(|| Regex::new(r"^(test|feat)\(([^)]*)\)(:\s*red\b)?").unwrap());
    let git = Git::new(root);
    let since = git.resolve(since)?;
    let mut red: BTreeSet<String> = BTreeSet::new();
    let mut proven: BTreeSet<String> = BTreeSet::new();
    let mut r = GateReport::default();
    for c in git.log(&format!("{since}..HEAD"))? {
        let Some(m) = re.captures(&c.subject) else {
            continue;
        };
        let ids = expand_ids(&m[2]);
        if &m[1] == "test" && m.get(3).is_some() {
            red.extend(ids);
        } else if &m[1] == "feat" {
            for id in ids {
                if red.contains(&id) {
                    proven.insert(id);
                } else if !r.failures.contains(&format!("no red commit {id}")) {
                    r.failures.push(format!("no red commit {id}"));
                }
            }
        }
    }
    r.pass = r.failures.is_empty();
    r.summary = if r.pass {
        format!(
            "gate: pass — history: {} red before green",
            plural(proven.len(), "scenario")
        )
    } else {
        format!("gate: fail — {}", plural(r.failures.len(), "problem"))
    };
    Ok(r)
}
