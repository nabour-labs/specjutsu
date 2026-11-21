//! The CI gate: classification, rules, red-on-base, exemptions and history check.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;
use sj_runners::cargo::{cargo_test, Outcome, Run};

use crate::config::{Config, RedOnBase, Runner};
use crate::git::Git;
use crate::run::suite::{self, JoinedOutcome, Pick};
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
    /// A `[keep]` scenario whose tests pass on base and at head (GATE-039). Only ever a
    /// [`Proven`] kind; base runs report a keep test's pass as [`Red::Green`].
    #[serde(rename = "green:keep")]
    Keep,
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
#[derive(Debug, Clone, Default)]
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
    /// Spec root relative to the repository root, forward slashes.
    pub spec_root: String,
    /// Scenarios without an ID whose text is new at head: `<name> (<file>:<line>)`.
    pub unidentified: Vec<String>,
    /// Scenarios without an ID that already existed on base.
    pub unidentified_known: usize,
    /// Which changed files are implementation files.
    pub classifier: Classifier,
    /// Head results of non-Cargo tests, keyed by (repository-relative file, test name).
    pub head_other: BTreeMap<(String, String), Outcome>,
    /// Failing non-Cargo results at head, as `<file>::<name>` (for the full-suite rules).
    pub head_failing: Vec<String>,
    /// Non-Cargo suites that failed to build at head: `<package>:<runner>`.
    pub head_build_failed: Vec<String>,
    /// Why a target test is unverifiable on base (`red_on_base = "skip"`), keyed like `base`.
    pub reasons: BTreeMap<(String, String), String>,
    /// IDs of `[keep]` scenarios at head: their tests must pass on base, not fail (GATE-039).
    pub keep: BTreeSet<String>,
}

/// `<ID> <name>`, or just the name when it already starts with the ID (JS and Python names
/// keep their ID, TRACE-009).
fn label(id: &str, name: &str) -> String {
    if name.starts_with(id) {
        name.to_string()
    } else {
        format!("{id} {name}")
    }
}

fn test_key(t: &trace::TestRef) -> (String, String) {
    (t.file.to_string_lossy().replace('\\', "/"), t.name.clone())
}

fn is_cargo(t: &trace::TestRef) -> bool {
    matches!(t.suite, None | Some(Runner::Cargo))
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
    /// Findings that never fail the gate, e.g. code drift in a refactor (DRIFT-011).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub info: Vec<String>,
    pub exemptions: Vec<String>,
    pub proven: Vec<Proven>,
    /// Every scenario whose classification is not `unchanged`, with its kind.
    pub changes: BTreeMap<String, Change>,
    /// One row per added, modified or implemented scenario, for `--format markdown`.
    #[serde(skip)]
    pub rows: Vec<Row>,
}

/// A changed scenario's evidence: `untested`, the first failure kind of its tests (e.g.
/// `green-on-base`), or its red-on-base kind when it is proven.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub evidence: String,
    pub tests: Vec<String>,
    pub failed: bool,
    /// A `[keep]` scenario (GATE-043).
    pub keep: bool,
}

/// Failures of the exemption rules: unknown kinds, missing reasons, and a spike merging to main.
pub fn exemption_failures(exemptions: &[Exemption], base: &str) -> Vec<String> {
    let mut out = Vec::new();
    for e in exemptions {
        if !KINDS.contains(&e.kind.as_str()) {
            out.push(format!("unknown exemption `{}`", e.kind));
        }
        if e.reason.is_empty() {
            out.push(format!("exemption needs a reason ({})", e.kind));
        }
        if e.kind == "spike" && is_main(base) {
            out.push("spike cannot merge to main".to_string());
        }
    }
    out
}

/// `unspecified change: <files>` when source files changed with no changed scenario and no
/// exemption.
pub fn unspecified(
    changes: &BTreeMap<String, Change>,
    changed_files: &[String],
    exemptions: &[Exemption],
    classifier: &Classifier,
    spec_root: &str,
) -> Option<String> {
    let targets = changes
        .values()
        .any(|c| matches!(c, Change::Added | Change::Modified | Change::Implemented));
    if targets || !exemptions.is_empty() {
        return None;
    }
    let impl_files: Vec<&str> = changed_files
        .iter()
        .filter(|f| classifier.is_impl_under(f, spec_root))
        .map(String::as_str)
        .collect();
    (!impl_files.is_empty()).then(|| format!("unspecified change: {}", impl_files.join(", ")))
}

/// Apply the gate rules. Pure: no git, no processes.
pub fn evaluate(input: &Input) -> GateReport {
    let mut r = GateReport {
        changes: input
            .changes
            .iter()
            .filter(|(_, c)| **c != Change::Unchanged)
            .map(|(id, c)| (id.clone(), *c))
            .collect(),
        ..GateReport::default()
    };

    // Exemptions.
    let mut skip_red = false;
    for e in &input.exemptions {
        r.exemptions.push(format!("{} — {}", e.kind, e.reason));
        if e.kind == "refactor" || e.kind == "spike" {
            skip_red = true;
        }
    }
    r.failures
        .extend(exemption_failures(&input.exemptions, &input.base_branch));
    let refactor = input.exemptions.iter().any(|e| e.kind == "refactor");
    let touches_impl = input
        .changed_files
        .iter()
        .any(|f| input.classifier.is_impl_under(f, &input.spec_root));
    // GATE-029: any exemption that touches source still needs a green suite.
    let full_suite = refactor || (!input.exemptions.is_empty() && touches_impl);

    // Head suite must build.
    if let Some(run) = &input.head_run {
        if !run.success && run.results.is_empty() {
            r.failures
                .push("test suite failed to build at head".to_string());
        }
        if full_suite {
            for t in &run.results {
                if t.outcome == Outcome::Failed {
                    r.failures.push(format!("failing {}", t.path));
                }
            }
        }
    }
    for s in &input.head_build_failed {
        r.failures
            .push(format!("test suite failed to build at head ({s})"));
    }
    if full_suite {
        for path in &input.head_failing {
            r.failures.push(format!("failing {path}"));
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
        let keep = input.keep.contains(*id);
        let Some(m) = input.trace.mapped.iter().find(|m| &&m.id == id) else {
            r.failures.push(format!("untested {id}"));
            r.rows.push(Row {
                id: id.to_string(),
                evidence: "untested".to_string(),
                tests: Vec::new(),
                failed: true,
                keep,
            });
            continue;
        };
        let mut ok = true;
        let mut weakest: Option<Red> = None;
        let failures_before = r.failures.len();
        for t in &m.tests {
            let head = if is_cargo(t) {
                input.head_run.as_ref().and_then(|run| run.outcome(&t.name))
            } else {
                input.head_other.get(&test_key(t)).copied()
            };
            let what = label(id, &t.name);
            match head {
                Some(Outcome::Passed) => {}
                Some(Outcome::Failed) => {
                    ok = false;
                    let kind = if keep { "keep-broken" } else { "failing" };
                    r.failures.push(format!("{kind} {what}"));
                }
                _ => {
                    ok = false;
                    r.failures.push(format!("not-run {what}"));
                }
            }
            if skip_red {
                continue;
            }
            let key = (id.to_string(), t.name.clone());
            match input.base.get(&key) {
                // GATE-039/040: a keep test must pass on base; failing there means the
                // scenario describes new behaviour.
                Some(Red::Green) if keep => {}
                Some(Red::Assert | Red::Compile) if keep => {
                    ok = false;
                    r.failures.push(format!(
                        "keep-red-on-base {what} (it fails on base, so it describes new behaviour; drop [keep] to prove it red-on-base)"
                    ));
                }
                Some(red @ (Red::Assert | Red::Compile)) => {
                    if weakest != Some(Red::Compile) {
                        weakest = Some(*red);
                    }
                }
                Some(Red::Green) => {
                    ok = false;
                    r.failures.push(format!("green-on-base {what}"));
                }
                Some(Red::Unverifiable) => {
                    let msg = match input.reasons.get(&key) {
                        Some(reason) => format!("unverifiable {what} ({reason})"),
                        None => format!(
                            "unverifiable {what} (inline test; move it to tests/ so it can run on base)"
                        ),
                    };
                    if input.allow_unverifiable {
                        r.warnings.push(msg);
                    } else {
                        ok = false;
                        r.failures.push(msg);
                    }
                }
                Some(Red::NotRun | Red::Keep) | None => {
                    ok = false;
                    r.failures.push(format!("not-run-on-base {what}"));
                }
            }
        }
        if ok && keep && !skip_red {
            weakest = Some(Red::Keep);
        }
        if ok {
            if let Some(red) = weakest {
                r.proven.push(Proven {
                    id: id.to_string(),
                    red,
                });
            }
        }
        // The first failure's kind is its first word (`green-on-base AUTH-005 …`).
        let first_failure = r.failures[failures_before..]
            .first()
            .and_then(|f| f.split_whitespace().next())
            .map(str::to_string);
        let evidence = match (&first_failure, weakest) {
            (Some(kind), _) => kind.clone(),
            (None, Some(Red::Keep)) => "green:keep".to_string(),
            (None, Some(Red::Compile)) => "red:compile".to_string(),
            (None, Some(_)) => "red:assert".to_string(),
            (None, None) => "green (exempt)".to_string(),
        };
        r.rows.push(Row {
            id: id.to_string(),
            evidence,
            tests: m.tests.iter().map(|t| t.name.clone()).collect(),
            failed: !ok,
            keep,
        });
    }

    // Unchanged gaps only warn.
    for g in &input.trace.gaps {
        if let Some(id) = &g.id {
            if !targets.contains(&id) {
                r.warnings.push(format!("gap {id} {}", g.name));
            }
        }
    }

    for u in &input.unidentified {
        r.failures.push(format!("unidentified {u}"));
    }
    if input.unidentified_known > 0 {
        r.warnings.push(if input.unidentified_known == 1 {
            "1 scenario without ID".to_string()
        } else {
            format!("{} scenarios without IDs", input.unidentified_known)
        });
    }

    for o in &input.trace.orphans {
        r.failures.push(format!("orphan {} {}", o.id, o.test.name));
    }

    r.failures.extend(unspecified(
        &input.changes,
        &input.changed_files,
        &input.exemptions,
        &input.classifier,
        &input.spec_root,
    ));

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
    for (k, label) in [
        (Red::Assert, "red:assert"),
        (Red::Compile, "red:compile"),
        (Red::Keep, "green:keep"),
    ] {
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

/// Extensions of documentation and media files.
const DOC_EXTS: [&str; 11] = [
    "md", "mdx", "txt", "rst", "png", "jpg", "jpeg", "gif", "svg", "webp", "ico",
];

/// Package manifests and lockfiles, by file name.
const MANIFESTS: [&str; 12] = [
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "yarn.lock",
    "pyproject.toml",
    "uv.lock",
    "go.mod",
    "go.sum",
    "Gemfile.lock",
];

/// A changed file that needs a scenario or an exemption. Fail-closed: everything is
/// implementation unless it is a test, a spec, documentation or media, repository metadata,
/// or a manifest or lockfile.
pub fn is_impl_file(rel: &str, spec_root: &str) -> bool {
    let root = spec_root.trim_end_matches('/');
    if rel.strip_prefix(root).is_some_and(|r| r.starts_with('/')) {
        return false;
    }
    if rel
        .split('/')
        .any(|p| matches!(p, "tests" | "fixtures" | "target"))
    {
        return false;
    }
    if rel.starts_with(".github/") {
        return false;
    }
    let name = rel.rsplit('/').next().unwrap_or(rel);
    if matches!(name, ".gitignore" | ".gitattributes")
        || name.starts_with("LICENSE")
        || MANIFESTS.contains(&name)
    {
        return false;
    }
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    !ext.is_some_and(|e| DOC_EXTS.contains(&e.as_str()))
}

/// Decides which changed files need a scenario: the built-in rule, refined by the project
/// file's `non_source` list, its suites' test and support globs, and its packages' `sources`.
#[derive(Debug, Clone, Default)]
pub struct Classifier {
    spec_root: String,
    /// `non_source`, plus the `tests` and `support` globs of non-Cargo suites. A Cargo suite's
    /// `tests` covers every `.rs` file, since Rust tests can sit inline; the built-in
    /// `tests/` rule handles Cargo instead.
    never: Option<globset::GlobSet>,
    /// (package path, its `sources`) for packages that declare sources.
    packages: Vec<(String, globset::GlobSet)>,
    /// Every package path, to find a file's owner.
    owners: Vec<String>,
}

fn globset(globs: &[String]) -> Result<globset::GlobSet> {
    let mut b = globset::GlobSetBuilder::new();
    for g in globs {
        b.add(
            globset::GlobBuilder::new(g)
                .literal_separator(true)
                .build()
                .with_context(|| format!("specjutsu.toml: bad glob `{g}`"))?,
        );
    }
    Ok(b.build()?)
}

impl Classifier {
    pub fn new(config: &crate::config::Config, spec_root: &str) -> Result<Classifier> {
        let mut never = config.non_source.clone();
        for p in &config.packages {
            for s in &p.suites {
                if s.runner != crate::config::Runner::Cargo {
                    never.extend(s.tests.iter().cloned());
                }
                never.extend(s.support.iter().cloned());
            }
        }
        let mut packages = Vec::new();
        for p in config.packages.iter().filter(|p| !p.sources.is_empty()) {
            packages.push((p.path.clone(), globset(&p.sources)?));
        }
        Ok(Classifier {
            spec_root: spec_root.to_string(),
            never: Some(globset(&never)?),
            packages,
            owners: config.packages.iter().map(|p| p.path.clone()).collect(),
        })
    }

    /// The built-in rule alone, for a repository without a project file (whose default
    /// package `sources` are a guess, not a declaration).
    pub fn builtin(spec_root: &str) -> Classifier {
        Classifier {
            spec_root: spec_root.to_string(),
            ..Classifier::default()
        }
    }

    /// Whether `rel` (repository-relative, forward slashes) needs a scenario or an exemption.
    pub fn is_impl(&self, rel: &str) -> bool {
        self.is_impl_under(rel, &self.spec_root)
    }

    fn is_impl_under(&self, rel: &str, spec_root: &str) -> bool {
        if !is_impl_file(rel, spec_root) || self.never.as_ref().is_some_and(|g| g.is_match(rel)) {
            return false;
        }
        let owns =
            |pkg: &str| pkg == "." || rel.strip_prefix(pkg).is_some_and(|r| r.starts_with('/'));
        let owner = self.owners.iter().filter(|p| owns(p)).max_by_key(|p| {
            if *p == "." {
                0
            } else {
                p.len() + 1
            }
        });
        match owner.and_then(|o| self.packages.iter().find(|(p, _)| p == o)) {
            Some((_, sources)) => sources.is_match(rel),
            None => true,
        }
    }
}

/// What the gate and `sj verify` both work from: the merge-base, the scenario classification,
/// the trace, changed files and exemptions.
pub struct Prepared {
    pub mb: String,
    /// Spec root relative to the repository root, forward slashes.
    pub spec_root: String,
    pub classifier: Classifier,
    pub changes: BTreeMap<String, Change>,
    pub unidentified: Vec<String>,
    pub unidentified_known: usize,
    pub scan: testscan::Scan,
    pub report: Report,
    pub changed_files: Vec<String>,
    pub exemptions: Vec<Exemption>,
}

/// Gather the gate's inputs against the merge-base of `base` and `HEAD`, without running tests.
pub fn prepare(root: &Path, spec_root_rel: &Path, config: &Config, base: &str) -> Result<Prepared> {
    let git = Git::new(root);
    let base_sha = git.resolve(base)?;
    let head_sha = git.resolve("HEAD")?;
    let mb = git.merge_base(&base_sha, &head_sha)?;
    let spec_root = spec_root_rel.to_string_lossy().replace('\\', "/");
    // Package sources are a declaration only when the project file exists (GATE-035).
    let classifier = if root.join(crate::config::FILE).is_file() {
        Classifier::new(config, &spec_root)?
    } else {
        Classifier::builtin(&spec_root)
    };

    // Classification: base from git objects, head from the working tree.
    let mut base_fps = Vec::new();
    let mut base_texts: BTreeSet<String> = BTreeSet::new();
    // Texts of base scenarios without an ID: gaining one is not a change (GATE-036).
    let mut base_unidentified: BTreeSet<String> = BTreeSet::new();
    let mut base_paths = git.ls_tree(&mb)?;
    let base_root = base_spec_root(&git, &mb, &spec_root, &base_paths);
    // Read base in head's order (`spec::discover_spec_files`: `specs/`, then each change), so
    // `classify`'s last-occurrence rule lets a change's delta win on both sides (GATE-045).
    // Git lists `changes/` before `specs/`.
    let changes_dir = format!("{}/changes/", base_root.trim_end_matches('/'));
    base_paths.sort_by_key(|p| p.starts_with(&changes_dir));
    for path in base_paths {
        if spec::is_spec_path(&path, &base_root) {
            let content = git.show(&mb, &path)?;
            base_fps.extend(spec::fingerprints(&content));
            for (s, t) in spec::scenario_texts(Path::new(&path), &content) {
                if s.id.is_none() {
                    base_unidentified.insert(t.clone());
                }
                base_texts.insert(t);
            }
        }
    }
    let head_spec_root = root.join(spec_root_rel);
    let mut head_fps = Vec::new();
    // Scenarios without an ID: new text fails, text already on base only counts (GATE-024/025).
    let mut unidentified = Vec::new();
    let mut unidentified_known = 0;
    for file in spec::discover_spec_files(&head_spec_root)? {
        let content = std::fs::read_to_string(&file)?;
        head_fps.extend(spec::fingerprints(&content));
        for (s, text) in spec::scenario_texts(&file, &content) {
            if s.id.is_some() {
                continue;
            }
            if base_texts.contains(&text) {
                unidentified_known += 1;
            } else {
                let rel = file
                    .strip_prefix(root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");
                unidentified.push(format!("{} ({rel}:{})", s.name, s.line));
            }
        }
    }
    let mut changes = classify(&base_fps, &head_fps);
    // GATE-036: an added ID whose text (last occurrence, as `classify`) existed without an ID.
    let head_text: BTreeMap<&str, &str> = head_fps
        .iter()
        .map(|f| (f.id.as_str(), f.text.as_str()))
        .collect();
    for (id, c) in changes.iter_mut() {
        if *c == Change::Added
            && head_text
                .get(id.as_str())
                .is_some_and(|t| base_unidentified.contains(*t))
        {
            *c = Change::Unchanged;
        }
    }

    let scenarios = spec::load_scenarios(&head_spec_root)?;
    let scan = testscan::scan_project(root, config)?;
    let report = trace::build_report(&scenarios, &scan.tests);

    // GATE-023: unchanged scenarios that gain their first test are in scope.
    let base_mapped = base_mapped_ids(&git, root, config, &mb, &base_root)?;
    for m in &report.mapped {
        if let Some(c) = changes.get_mut(&m.id) {
            if *c == Change::Unchanged && !base_mapped.contains(&m.id) {
                *c = Change::Implemented;
            }
        }
    }
    let mut changed_files = git.changed_files(&mb)?;
    // GATE-044: files leaving the base spec root are spec files, not source.
    if base_root != spec_root {
        let old = format!("{}/", base_root.trim_end_matches('/'));
        changed_files.retain(|f| !f.starts_with(&old));
    }

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
    Ok(Prepared {
        mb,
        spec_root,
        classifier,
        changes,
        unidentified,
        unidentified_known,
        scan,
        report,
        changed_files,
        exemptions,
    })
}

pub struct Options {
    pub root: PathBuf,
    /// Spec root relative to `root`.
    pub spec_root: PathBuf,
    pub base: String,
    pub allow_unverifiable: bool,
}

/// Gather inputs from git and the runners (including red-on-base), then evaluate.
pub fn run(opts: &Options) -> Result<GateReport> {
    let root = &opts.root;
    let config = crate::config::load(root)?;
    // GATE-026: never pass a repository the gate cannot test.
    if config.packages.iter().all(|p| p.suites.is_empty()) {
        anyhow::bail!(
            "no test suite configured in {} (supported runners: {}); \
             run `sj adopt` to write {}",
            root.display(),
            Runner::ALL.map(Runner::as_str).join(", "),
            crate::config::FILE
        );
    }
    let has_cargo = config
        .packages
        .iter()
        .any(|p| p.suites.iter().any(|s| s.runner == Runner::Cargo));
    let git = Git::new(root);
    let Prepared {
        mb,
        spec_root,
        classifier,
        changes,
        unidentified,
        unidentified_known,
        scan,
        report,
        changed_files,
        exemptions,
    } = prepare(root, &opts.spec_root, &config, &opts.base)?;

    let targets: Vec<&String> = changes
        .iter()
        .filter(|(_, c)| matches!(c, Change::Added | Change::Modified | Change::Implemented))
        .map(|(id, _)| id)
        .collect();
    // GATE-039: keep scenarios at head. A change's delta comes after `specs/`, so its
    // marker wins for an ID that appears in both.
    let mut keep_by_id: BTreeMap<String, bool> = BTreeMap::new();
    for s in spec::load_scenarios(&root.join(&opts.spec_root))? {
        if let Some(id) = s.id {
            keep_by_id.insert(id, s.keep);
        }
    }
    let keep: BTreeSet<String> = keep_by_id
        .into_iter()
        .filter_map(|(id, k)| k.then_some(id))
        .collect();
    let refactor = exemptions.iter().any(|e| e.kind == "refactor");
    let skip_red = refactor || exemptions.iter().any(|e| e.kind == "spike");
    let touches_impl = changed_files.iter().any(|f| classifier.is_impl(f));
    let full_suite = refactor || (!exemptions.is_empty() && touches_impl);
    let need_head = !targets.is_empty() || full_suite;

    // Target tests, by the suite that runs them: (package index, suite index) → (ID, test).
    let mut by_suite: BTreeMap<(usize, usize), Vec<(String, testscan::TestFn)>> = BTreeMap::new();
    let mut cargo_jobs: Vec<(String, String, String)> = Vec::new();
    let rel = |p: &Path| -> String {
        p.strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/")
    };
    for m in report.mapped.iter().filter(|m| targets.contains(&&m.id)) {
        for t in &m.tests {
            if is_cargo(t) {
                cargo_jobs.push((m.id.clone(), t.name.clone(), rel(&t.file)));
                continue;
            }
            let Some(at) = suite_of(&config, t.package.as_deref(), t.suite) else {
                continue;
            };
            if let Some(full) = scan
                .tests
                .iter()
                .find(|x| x.file == t.file && x.name == t.name)
            {
                by_suite
                    .entry(at)
                    .or_default()
                    .push((m.id.clone(), full.clone()));
            }
        }
    }
    let traced_in = |(pi, si): (usize, usize)| -> Vec<testscan::TestFn> {
        let (pkg, s) = (&config.packages[pi], &config.packages[pi].suites[si]);
        scan.tests
            .iter()
            .filter(|t| {
                t.package.as_deref() == Some(pkg.path.as_str()) && t.suite == Some(s.runner)
            })
            .cloned()
            .collect()
    };

    // Head runs.
    let head_run = if need_head && has_cargo {
        Some(cargo_test(root, &["--workspace", "--no-fail-fast"], None)?)
    } else {
        None
    };
    let mut head_other = BTreeMap::new();
    let mut head_failing = Vec::new();
    let mut head_build_failed = Vec::new();
    if need_head {
        for (pi, pkg) in config.packages.iter().enumerate() {
            for (si, s) in pkg.suites.iter().enumerate() {
                if s.runner == Runner::Cargo {
                    continue;
                }
                let pick = if full_suite {
                    Pick::All
                } else if let Some(jobs) = by_suite.get(&(pi, si)) {
                    Pick::Tests(jobs.iter().map(|(_, t)| t.clone()).collect())
                } else {
                    continue;
                };
                let traced = traced_in((pi, si));
                let res = suite::run_suite(root, pkg, s, &traced, &pick)?;
                if res.no_report {
                    head_build_failed.push(format!("{}:{}", pkg.path, s.runner.as_str()));
                    continue;
                }
                for j in res.joined {
                    let outcome = match j.outcome {
                        JoinedOutcome::Passed => Outcome::Passed,
                        JoinedOutcome::Failed | JoinedOutcome::BuildFailed => Outcome::Failed,
                        JoinedOutcome::Skipped => Outcome::Ignored,
                    };
                    if outcome == Outcome::Failed {
                        head_failing.push(j.path.clone());
                    }
                    if let Some(i) = j.test {
                        let t = &traced[i];
                        head_other.insert(
                            (t.file.to_string_lossy().replace('\\', "/"), t.name.clone()),
                            outcome,
                        );
                    }
                }
            }
        }
    }

    // Red-on-base.
    let mut base = BTreeMap::new();
    let mut reasons = BTreeMap::new();
    if !skip_red && !targets.is_empty() {
        for (id, name, file) in &cargo_jobs {
            if !is_test_file(file) {
                base.insert((id.clone(), name.clone()), Red::Unverifiable);
            }
        }
        let mut suite_jobs = Vec::new();
        for (&(pi, si), jobs) in &by_suite {
            let s = &config.packages[pi].suites[si];
            if s.red_on_base == RedOnBase::Skip {
                for (id, t) in jobs {
                    let key = (id.clone(), t.name.clone());
                    base.insert(key.clone(), Red::Unverifiable);
                    reasons.insert(key, s.reason.clone().unwrap_or_default());
                }
            } else {
                suite_jobs.push(SuiteJob {
                    at: (pi, si),
                    traced: traced_in((pi, si)),
                    jobs: jobs.clone(),
                });
            }
        }
        let overlay_globs = {
            let mut g = Vec::new();
            for p in &config.packages {
                for s in &p.suites {
                    if s.runner != Runner::Cargo {
                        g.extend(s.tests.iter().cloned());
                    }
                    g.extend(s.support.iter().cloned());
                }
            }
            globset(&g)?
        };
        let overlay: Vec<&String> = changed_files
            .iter()
            .filter(|f| {
                (is_test_support(f) || overlay_globs.is_match(f.as_str())) && root.join(f).is_file()
            })
            .collect();
        let verifiable: Vec<&(String, String, String)> = cargo_jobs
            .iter()
            .filter(|(_, _, f)| is_test_file(f))
            .collect();
        if !verifiable.is_empty() || !suite_jobs.is_empty() {
            let results =
                run_on_base(&git, root, &config, &mb, &overlay, &verifiable, &suite_jobs)?;
            base.extend(results);
        }
    }

    let mut gate = evaluate(&Input {
        changes,
        trace: report,
        changed_files,
        exemptions,
        base_branch: opts.base.clone(),
        head_run,
        base,
        allow_unverifiable: opts.allow_unverifiable,
        spec_root,
        unidentified,
        unidentified_known,
        classifier,
        head_other,
        head_failing,
        head_build_failed,
        reasons,
        keep,
    });
    // AUDIT-013: the audit's test checks (never the registry ones) over the same range.
    let audit = crate::audit::run(&crate::audit::Options {
        root: root.to_path_buf(),
        base: opts.base.clone(),
        registry: false,
    })?;
    merge_audit(&mut gate, &audit);
    // DRIFT-010/011: spec drift, failing only with `[drift] fail = true`.
    let drift = crate::drift::run(&crate::drift::Options {
        root: root.to_path_buf(),
        spec_root: opts.spec_root.clone(),
        base: opts.base.clone(),
    })?;
    merge_drift(&mut gate, &drift);
    Ok(gate)
}

/// Add drift findings to a gate report by severity.
pub fn merge_drift(gate: &mut GateReport, drift: &crate::drift::DriftReport) {
    use crate::drift::Severity;
    let before = gate.failures.len();
    for f in &drift.findings {
        match f.severity {
            Severity::Fail => gate.failures.push(f.reason()),
            Severity::Warn => gate.warnings.push(f.text()),
            Severity::Info => gate.info.push(f.text()),
        }
    }
    if gate.failures.len() > before {
        gate.pass = false;
        gate.summary = format!("gate: fail — {}", plural(gate.failures.len(), "problem"));
    }
}

/// Add audit findings to a gate report: failures fail the gate, the rest are listed.
pub fn merge_audit(gate: &mut GateReport, audit: &crate::audit::AuditReport) {
    use crate::audit::Severity;
    let before = gate.failures.len();
    for f in &audit.findings {
        match f.severity {
            Severity::Fail => gate.failures.push(f.line()),
            Severity::Warn => gate.warnings.push(f.line()),
            Severity::Allowed => gate.exemptions.push(format!("allowed {}", f.line())),
        }
    }
    if gate.failures.len() > before {
        gate.pass = false;
        gate.summary = format!("gate: fail — {}", plural(gate.failures.len(), "problem"));
    }
}

/// The (package, suite) indexes of a traced test's suite.
fn suite_of(
    config: &Config,
    package: Option<&str>,
    runner: Option<Runner>,
) -> Option<(usize, usize)> {
    let (package, runner) = (package?, runner?);
    let pi = config.packages.iter().position(|p| p.path == package)?;
    let si = config.packages[pi]
        .suites
        .iter()
        .position(|s| s.runner == runner)?;
    Some((pi, si))
}

/// Target tests of one non-Cargo suite to run on base.
struct SuiteJob {
    at: (usize, usize),
    /// Every traced test of the suite (what its report is joined to).
    traced: Vec<testscan::TestFn>,
    /// (scenario ID, target test).
    jobs: Vec<(String, testscan::TestFn)>,
}

/// Scenario IDs that already had a linked test at `rev`, read from git objects.
/// The spec root base scenarios are read from (GATE-044): the head's when it holds specs at
/// base, otherwise the base commit's own setting or default, so moving the folder changes nothing.
fn base_spec_root(git: &Git, rev: &str, head: &str, paths: &[String]) -> String {
    if paths.iter().any(|p| spec::is_spec_path(p, head)) {
        return head.to_string();
    }
    let set = git
        .show(rev, crate::config::FILE)
        .ok()
        .and_then(|text| toml::from_str::<toml::Table>(&text).ok())
        .and_then(|t| t.get("spec_root")?.as_str().map(str::to_string));
    if let Some(root) = set {
        return root;
    }
    let has = |dir: &str| paths.iter().any(|p| p.starts_with(&format!("{dir}/")));
    let (default, legacy) = (
        crate::config::DEFAULT_SPEC_ROOT,
        crate::config::LEGACY_SPEC_ROOT,
    );
    if !has(default) && has(legacy) {
        legacy.to_string()
    } else {
        default.to_string()
    }
}

fn base_mapped_ids(
    git: &Git,
    root: &Path,
    config: &Config,
    rev: &str,
    spec_root: &str,
) -> Result<BTreeSet<String>> {
    let mut scenarios = Vec::new();
    let paths = git.ls_tree(rev)?;
    for path in &paths {
        if spec::is_spec_path(path, spec_root) {
            scenarios.extend(spec::parse_scenarios(
                Path::new(path),
                &git.show(rev, path)?,
            ));
        }
    }
    let tests = testscan::scan_listed(root, config, &paths, |p| git.show(rev, p))?;
    Ok(trace::build_report(&scenarios, &tests)
        .mapped
        .into_iter()
        .map(|m| m.id)
        .collect())
}

/// Lines of output kept from a failing base `setup`.
const SETUP_ERROR_LINES: usize = 20;

/// Check out `rev` in a temporary worktree, overlay the change's test files, and run each Cargo
/// test target and each non-Cargo suite that holds a scenario test.
fn run_on_base(
    git: &Git,
    root: &Path,
    config: &Config,
    rev: &str,
    overlay: &[&String],
    jobs: &[&(String, String, String)],
    suite_jobs: &[SuiteJob],
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

        // Non-Cargo suites: setup once per (package, command), then the target tests.
        let mut set_up: BTreeSet<(String, String)> = BTreeSet::new();
        for job in suite_jobs {
            let (pi, si) = job.at;
            let pkg = &config.packages[pi];
            let s = &pkg.suites[si];
            if let Some(cmd) = &s.setup {
                if set_up.insert((pkg.path.clone(), cmd.clone())) {
                    run_setup(&wt, pkg, s, cmd)?;
                }
            }
            let picked: Vec<testscan::TestFn> = job.jobs.iter().map(|(_, t)| t.clone()).collect();
            let res = suite::run_suite(&wt, pkg, s, &job.traced, &Pick::Tests(picked))?;
            let failed_early = res.no_report || !res.exit_ok;
            for (id, t) in &job.jobs {
                let joined = res.joined.iter().find(|j| {
                    j.test.is_some_and(|i| {
                        job.traced[i].file == t.file && job.traced[i].name == t.name
                    })
                });
                let red = match joined.map(|j| j.outcome) {
                    Some(JoinedOutcome::Passed) => Red::Green,
                    Some(JoinedOutcome::Failed) => Red::Assert,
                    Some(JoinedOutcome::BuildFailed) => Red::Compile,
                    Some(JoinedOutcome::Skipped) => Red::NotRun,
                    None if failed_early => Red::Compile,
                    None => Red::NotRun,
                };
                out.insert((id.clone(), t.name.clone()), red);
            }
        }
        Ok(out)
    })();
    let cleanup = git.worktree_remove(&wt);
    let out = result?;
    cleanup?;
    Ok(out)
}

/// Run a suite's `setup` in the base worktree's package directory. A failure is an environment
/// error carrying the command's last lines (GATE-032).
fn run_setup(
    wt: &Path,
    pkg: &crate::config::Package,
    s: &crate::config::Suite,
    cmd: &str,
) -> Result<()> {
    let dir = suite::package_dir(wt, pkg);
    let out = suite::shell(cmd)
        .current_dir(&dir)
        .envs(&s.env)
        .stdin(std::process::Stdio::null())
        .output()
        .with_context(|| {
            format!(
                "cannot run setup `{cmd}` for {}:{}",
                pkg.path,
                s.runner.as_str()
            )
        })?;
    if out.status.success() {
        return Ok(());
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let lines: Vec<&str> = text.lines().collect();
    let tail = lines[lines.len().saturating_sub(SETUP_ERROR_LINES)..].join("\n");
    anyhow::bail!(
        "setup `{cmd}` failed on base for {}:{} ({})\n{tail}",
        pkg.path,
        s.runner.as_str(),
        out.status
    )
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
