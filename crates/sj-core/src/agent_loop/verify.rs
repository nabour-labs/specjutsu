//! `sj verify`: the local check before a PR. Everything the gate checks except red-on-base.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::Serialize;

use super::{
    classification, default_base, not_property, targets, why_unproven_against_base, Ledger,
};
use crate::authoring::{self, check};
use crate::gate;
use crate::git::Git;
use crate::ledger;
use crate::run::{self, Outcome, Scope};

/// One line of `sj verify` output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    /// `untested`, `not-property`, `failing`, `build-failed`, `unproven`, `orphan`, `baseline-added`,
    /// `uncovered`, `unknown`, `clarify` or `gate`.
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub problems: Vec<Problem>,
    /// Problems writing the ledger, and baseline entries a `config` exemption adopts;
    /// reported, never failing.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Combined stdout and stderr of every runner process verify started.
    #[serde(skip)]
    pub raw_bytes: u64,
}

pub struct Options {
    pub root: PathBuf,
    /// Spec root relative to `root`.
    pub spec_root: PathBuf,
    pub change: Option<String>,
    pub base: Option<String>,
}

fn problem(kind: &'static str, id: Option<&str>, message: String) -> Problem {
    Problem {
        kind,
        id: id.map(str::to_string),
        message,
    }
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn verify(opts: &Options) -> Result<VerifyReport> {
    let root = &opts.root;
    let spec_root = root.join(&opts.spec_root);
    if !Git::new(root).is_repo() {
        bail!("{} is not in a git repository", root.display());
    }
    let Some(base) = default_base(root, opts.base.as_deref()) else {
        bail!("no default branch found (tried origin/HEAD, origin/main, origin/master, main, master); pass --base <ref>");
    };
    let config = crate::config::load(root)?;
    let changes = match &opts.change {
        Some(name) => vec![authoring::find_change(&spec_root, name)?],
        None => authoring::open_changes(&spec_root)?,
    };
    let mut ids: Vec<String> = Vec::new();
    for c in &changes {
        for id in targets(c)? {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    let prep = gate::prepare(root, &opts.spec_root, &config, &base)?;
    let mut r = VerifyReport::default();
    let classified = classification(root, &prep);

    // Run each linked target and record the run.
    let mut passing = Vec::new();
    for id in &ids {
        if !prep.report.mapped.iter().any(|m| &m.id == id) {
            r.problems
                .push(problem("untested", Some(id), format!("untested {id}")));
            continue;
        }
        // LOOP-038: example tests can't prove a FOR ALL scenario.
        let np = not_property(&prep.report, classified, id);
        if !np.is_empty() {
            for f in np {
                r.problems.push(problem("not-property", Some(id), f));
            }
            continue;
        }
        let run = run::run_tests(root, &opts.spec_root, &Scope::Scenario(id.clone()))?;
        r.raw_bytes += run.raw_bytes;
        if let Err(e) = ledger::record(root, &run) {
            r.warnings.push(format!("run not recorded: {e:#}"));
        }
        if run.build_failed {
            r.problems.push(problem(
                "build-failed",
                Some(id),
                format!("build-failed {id}"),
            ));
            continue;
        }
        let failing: Vec<&run::TestCase> = run
            .tests
            .iter()
            .filter(|t| t.result == Outcome::Failed)
            .collect();
        for t in &failing {
            r.problems.push(problem(
                "failing",
                Some(id),
                format!("failing {}  {id}", t.path),
            ));
        }
        if failing.is_empty() && run.partial_build_failed.is_empty() {
            passing.push(id.clone());
        } else if !run.partial_build_failed.is_empty() {
            r.problems.push(problem(
                "build-failed",
                Some(id),
                format!("build-failed {id}"),
            ));
        }
    }

    // Evidence, read after the runs above were recorded.
    let ledger = Ledger::load(root)?;
    let keep = crate::spec::keep_ids(&spec_root)?;
    for id in &passing {
        if let Some(why) = why_unproven_against_base(&ledger, &keep, classified, id) {
            r.problems.push(problem(
                "unproven",
                Some(id),
                format!("unproven {id} ({why})"),
            ));
        }
    }

    // The gate's baseline rules (LOOP-032, LOOP-033).
    let rules = gate::baseline_rules(
        &prep.baseline,
        &prep.baseline_added,
        &prep.exemptions,
        &prep.report,
    );
    for f in rules.failures {
        r.problems.push(problem("baseline-added", None, f));
    }
    r.warnings.extend(rules.warnings);
    for o in &rules.orphans {
        r.problems.push(problem(
            "orphan",
            Some(&o.id),
            format!("orphan {} {}", o.id, o.test.name),
        ));
    }

    for c in &changes {
        let ck = check::check(&spec_root, &c.name)?;
        for id in &ck.uncovered {
            r.problems
                .push(problem("uncovered", Some(id), format!("uncovered {id}")));
        }
        for u in &ck.unknown {
            r.problems.push(problem(
                "unknown",
                Some(&u.id),
                format!(
                    "unknown {} ({}:{})",
                    u.id,
                    rel(&c.dir.join(&u.file), root),
                    u.line
                ),
            ));
        }
        for q in &ck.clarify {
            r.problems.push(problem(
                "clarify",
                None,
                format!(
                    "clarify {} ({}:{})",
                    q.question,
                    rel(&c.dir.join(&q.file), root),
                    q.line
                ),
            ));
        }
    }

    // The gate's rules on the whole branch.
    for f in gate::exemption_failures(&prep.exemptions, &base) {
        r.problems.push(problem("gate", None, f));
    }
    // Verify doesn't run base, so it can't see `green:linked`. An implemented scenario never
    // proven red may be one, and a linked scenario covers no source change (GATE-060).
    let linked: std::collections::BTreeSet<String> = prep
        .changes
        .iter()
        .filter(|(id, c)| {
            **c == gate::Change::Implemented && !ledger.status(id).is_some_and(|s| s.proven)
        })
        .map(|(id, _)| id.clone())
        .collect();
    if let Some(f) = gate::unspecified(
        &prep.changes,
        &linked,
        &prep.changed_files,
        &prep.exemptions,
        &prep.classifier,
        &prep.spec_root,
    ) {
        r.problems.push(problem("gate", None, f));
    }

    r.ok = r.problems.is_empty();
    Ok(r)
}
