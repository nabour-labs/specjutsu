//! `sj test`: choose the tests in scope, run them, and join results to scenario IDs.

pub mod suite;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Result};
use regex::Regex;
use serde::Serialize;
use sj_runners::cargo::cargo_test_detailed;
pub use sj_runners::cargo::Outcome;

use crate::config::Runner;
use crate::{spec, testscan, trace};

/// Most failure-output lines shown per failing test.
pub const EXCERPT_LINES: usize = 20;

/// What `sj test [SCOPE]` runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    All,
    /// Only the tests mapped to this scenario ID.
    Scenario(String),
    /// A configured package path (every suite in it), or otherwise one Cargo package by name.
    Package(String),
    /// One suite: `<package>:<runner>`.
    Suite {
        package: String,
        runner: String,
    },
}

impl Scope {
    /// A scope shaped like a scenario ID (`PREFIX-NNN`) is a scenario; anything else is a package.
    pub fn parse(arg: Option<&str>) -> Scope {
        static ID: OnceLock<Regex> = OnceLock::new();
        let id = ID.get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9]*-[0-9]+$").unwrap());
        match arg {
            None => Scope::All,
            Some(s) if id.is_match(s) => Scope::Scenario(s.to_string()),
            Some(s) => match s.rsplit_once(':') {
                Some((package, runner)) if !package.is_empty() && !runner.is_empty() => {
                    Scope::Suite {
                        package: package.to_string(),
                        runner: runner.to_string(),
                    }
                }
                _ => Scope::Package(s.to_string()),
            },
        }
    }
}

/// How to invoke the runner for a scope.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    pub cargo_args: Vec<String>,
    pub filters: Vec<String>,
    /// For a scenario scope: the mapped test names and their IDs. Results for other names are dropped.
    pub only: Option<BTreeMap<String, Vec<String>>>,
    /// Test function name → mapped scenario IDs, for every test in source.
    pub ids: BTreeMap<String, Vec<String>>,
    /// Names of the mapped tests that are property tests.
    pub property: BTreeSet<String>,
}

/// The first lines of a failure's output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Excerpt {
    pub lines: Vec<String>,
    /// Lines cut after the limit.
    pub omitted: usize,
}

pub fn excerpt(text: &str, limit: usize) -> Excerpt {
    let all: Vec<&str> = text.lines().collect();
    Excerpt {
        lines: all.iter().take(limit).map(|l| l.to_string()).collect(),
        omitted: all.len().saturating_sub(limit),
    }
}

/// One test that ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestCase {
    /// libtest path, e.g. `tests::foo` or `foo`.
    pub path: String,
    pub result: Outcome,
    pub ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<Excerpt>,
    /// The input a failing property test printed (LEDGER-013).
    #[serde(skip)]
    pub counterexample: Option<String>,
}

/// The outcome of `sj test`, independent of the runner.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TestRun {
    pub pass: bool,
    pub build_failed: bool,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub build_errors: Vec<String>,
    pub tests: Vec<TestCase>,
    /// For a scenario scope: the selected test names and IDs (used to record build failures).
    #[serde(skip)]
    pub selected: Vec<(String, Vec<String>)>,
    /// Each suite that ran, in order.
    pub suites: Vec<SuiteRun>,
    /// Selected tests that could not build while other suites ran (recorded as build-failed).
    #[serde(skip)]
    pub partial_build_failed: Vec<(String, Vec<String>)>,
    /// Combined stdout and stderr bytes of every runner process.
    #[serde(skip)]
    pub raw_bytes: u64,
}

/// One suite's part of a run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SuiteRun {
    /// `<package>:<runner>`.
    pub name: String,
    pub pass: bool,
    pub build_failed: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub build_errors: Vec<String>,
}

/// Work out what to run for `scope`. A scenario with no mapped test is an error.
pub fn resolve(root: &Path, spec_root: &Path, scope: &Scope) -> Result<Selection> {
    let scenarios = spec::load_scenarios(&root.join(spec_root))?;
    let tests = testscan::scan_tree(root)?;
    let report = trace::build_report(&scenarios, &tests);
    let mut ids: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut property = BTreeSet::new();
    for m in &report.mapped {
        for t in &m.tests {
            if t.property {
                property.insert(t.name.clone());
            }
            let v = ids.entry(t.name.clone()).or_default();
            if !v.contains(&m.id) {
                v.push(m.id.clone());
            }
        }
    }
    let mut sel = Selection {
        ids,
        property,
        ..Selection::default()
    };
    match scope {
        Scope::All | Scope::Suite { .. } => sel.cargo_args.push("--workspace".into()),
        Scope::Package(p) => sel.cargo_args.extend(["-p".to_string(), p.clone()]),
        Scope::Scenario(id) => {
            let Some(m) = report.mapped.iter().find(|m| &m.id == id) else {
                bail!("scenario {id} has no mapped test");
            };
            let names: BTreeSet<&str> = m.tests.iter().map(|t| t.name.as_str()).collect();
            sel.cargo_args.push("--workspace".into());
            sel.filters = names.iter().map(|n| n.to_string()).collect();
            sel.only = Some(
                names
                    .iter()
                    .map(|n| (n.to_string(), sel.ids[*n].clone()))
                    .collect(),
            );
        }
    }
    Ok(sel)
}

/// Run every suite in `scope` and join the results to scenario IDs.
pub fn run_tests(root: &Path, spec_root: &Path, scope: &Scope) -> Result<TestRun> {
    let config = crate::config::load(root)?;
    if config.packages.iter().all(|p| p.suites.is_empty()) {
        bail!(
            "no test suite in {} (no Cargo.toml, and specjutsu.toml lists no suites); run `sj adopt`",
            root.display()
        );
    }
    let has_non_cargo = config
        .packages
        .iter()
        .flat_map(|p| &p.suites)
        .any(|s| s.runner != Runner::Cargo);
    if !has_non_cargo {
        // Cargo only: exactly today's behaviour.
        let mut run = run_cargo(root, spec_root, scope)?;
        let name = config
            .packages
            .iter()
            .find(|p| !p.suites.is_empty())
            .map(|p| format!("{}:cargo", p.path))
            .unwrap_or_default();
        run.suites.push(SuiteRun {
            name,
            pass: run.pass,
            build_failed: run.build_failed,
            build_errors: run.build_errors.clone(),
        });
        return Ok(run);
    }

    let scenarios = spec::load_scenarios(&root.join(spec_root))?;
    let scan = testscan::scan_project(root, &config)?;
    let report = trace::build_report(&scenarios, &scan.tests);
    let mut ids: BTreeMap<(PathBuf, String), Vec<String>> = BTreeMap::new();
    for m in &report.mapped {
        for t in &m.tests {
            let v = ids.entry((t.file.clone(), t.name.clone())).or_default();
            if !v.contains(&m.id) {
                v.push(m.id.clone());
            }
        }
    }

    // (package index, suite index, what to run, cargo scope for a cargo suite)
    let mut plans: Vec<(usize, usize, suite::Pick, Option<Scope>)> = Vec::new();
    let suites_of = |pi: usize| (0..config.packages[pi].suites.len()).map(move |si| (pi, si));
    match scope {
        Scope::All => {
            for pi in 0..config.packages.len() {
                for (pi, si) in suites_of(pi) {
                    plans.push((pi, si, suite::Pick::All, Some(Scope::All)));
                }
            }
        }
        Scope::Package(p) => match config.packages.iter().position(|x| &x.path == p) {
            Some(pi) => {
                for (pi, si) in suites_of(pi) {
                    plans.push((pi, si, suite::Pick::All, Some(Scope::All)));
                }
            }
            None => {
                let cargo = config.packages.iter().enumerate().find_map(|(pi, x)| {
                    x.suites
                        .iter()
                        .position(|s| s.runner == Runner::Cargo)
                        .map(|si| (pi, si))
                });
                let Some((pi, si)) = cargo else {
                    bail!("no package `{p}` in specjutsu.toml");
                };
                plans.push((pi, si, suite::Pick::All, Some(scope.clone())));
            }
        },
        Scope::Suite { package, runner } => {
            let Some(pi) = config.packages.iter().position(|x| &x.path == package) else {
                bail!("no package `{package}` in specjutsu.toml");
            };
            let Some(si) = config.packages[pi]
                .suites
                .iter()
                .position(|s| s.runner.as_str() == runner)
            else {
                bail!("package `{package}` has no `{runner}` suite");
            };
            plans.push((pi, si, suite::Pick::All, Some(Scope::All)));
        }
        Scope::Scenario(id) => {
            let Some(m) = report.mapped.iter().find(|m| &m.id == id) else {
                bail!("scenario {id} has no mapped test");
            };
            let mut by_suite: BTreeMap<(usize, usize), Vec<testscan::TestFn>> = BTreeMap::new();
            for t in &m.tests {
                let (Some(pkg), Some(runner)) = (&t.package, t.suite) else {
                    continue;
                };
                let Some(pi) = config.packages.iter().position(|x| &x.path == pkg) else {
                    continue;
                };
                let Some(si) = config.packages[pi]
                    .suites
                    .iter()
                    .position(|s| s.runner == runner)
                else {
                    continue;
                };
                if let Some(full) = scan
                    .tests
                    .iter()
                    .find(|x| x.file == t.file && x.name == t.name)
                {
                    by_suite.entry((pi, si)).or_default().push(full.clone());
                }
            }
            for ((pi, si), tests) in by_suite {
                plans.push((pi, si, suite::Pick::Tests(tests), Some(scope.clone())));
            }
        }
    }

    let mut run = TestRun::default();
    for (pi, si, pick, cargo_scope) in plans {
        let pkg = &config.packages[pi];
        let s = &pkg.suites[si];
        let name = format!("{}:{}", pkg.path, s.runner.as_str());
        if s.runner == Runner::Cargo {
            let sub = run_cargo(root, spec_root, cargo_scope.as_ref().unwrap_or(&Scope::All))?;
            run.passed += sub.passed;
            run.failed += sub.failed;
            run.ignored += sub.ignored;
            run.raw_bytes += sub.raw_bytes;
            run.tests.extend(sub.tests);
            run.selected.extend(sub.selected.iter().cloned());
            if sub.build_failed {
                run.partial_build_failed.extend(sub.selected);
            }
            run.suites.push(SuiteRun {
                name,
                pass: sub.pass,
                build_failed: sub.build_failed,
                build_errors: sub.build_errors,
            });
            continue;
        }

        let traced: Vec<testscan::TestFn> = scan
            .tests
            .iter()
            .filter(|t| {
                t.package.as_deref() == Some(pkg.path.as_str()) && t.suite == Some(s.runner)
            })
            .cloned()
            .collect();
        let ids_of = |t: &testscan::TestFn| {
            ids.get(&(t.file.clone(), t.name.clone()))
                .cloned()
                .unwrap_or_default()
        };
        let scenario_scope = matches!(pick, suite::Pick::Tests(_));
        if let suite::Pick::Tests(picked) = &pick {
            for t in picked {
                run.selected.push((
                    format!(
                        "{}::{}",
                        t.file.to_string_lossy().replace('\\', "/"),
                        t.name
                    ),
                    ids_of(t),
                ));
            }
        }
        let res = suite::run_suite(root, pkg, s, &traced, &pick)?;
        run.raw_bytes += res.output_bytes;
        let mut sr = SuiteRun {
            name,
            ..SuiteRun::default()
        };
        if res.no_report {
            sr.build_failed = true;
            sr.build_errors = res.build_errors;
            if let suite::Pick::Tests(picked) = &pick {
                for t in picked {
                    run.partial_build_failed.push((
                        format!(
                            "{}::{}",
                            t.file.to_string_lossy().replace('\\', "/"),
                            t.name
                        ),
                        ids_of(t),
                    ));
                }
            }
            run.suites.push(sr);
            continue;
        }
        let mut failed = 0;
        for j in res.joined {
            let tids = j.test.map(|i| ids_of(&traced[i])).unwrap_or_default();
            let result = match j.outcome {
                suite::JoinedOutcome::Passed => Outcome::Passed,
                suite::JoinedOutcome::Failed => Outcome::Failed,
                suite::JoinedOutcome::Skipped => Outcome::Ignored,
                suite::JoinedOutcome::BuildFailed => {
                    let first = j
                        .message
                        .as_deref()
                        .and_then(|m| m.lines().next())
                        .unwrap_or("failed to load");
                    sr.build_errors.push(format!("{}: {first}", j.path));
                    if scenario_scope && j.test.is_some() {
                        run.partial_build_failed.push((j.path.clone(), tids));
                    }
                    continue;
                }
            };
            match result {
                Outcome::Passed => run.passed += 1,
                Outcome::Failed => {
                    run.failed += 1;
                    failed += 1;
                }
                Outcome::Ignored => run.ignored += 1,
            }
            run.tests.push(TestCase {
                path: j.path,
                result,
                ids: tids,
                excerpt: (result == Outcome::Failed)
                    .then(|| j.message.as_deref().map(|m| excerpt(m, EXCERPT_LINES)))
                    .flatten(),
                counterexample: (result == Outcome::Failed
                    && j.test.is_some_and(|i| traced[i].property))
                .then(|| {
                    j.message
                        .as_deref()
                        .and_then(crate::counterexample::counterexample)
                })
                .flatten(),
            });
        }
        sr.pass = failed == 0 && sr.build_errors.is_empty() && (scenario_scope || res.exit_ok);
        run.suites.push(sr);
    }

    run.build_failed = !run.suites.is_empty() && run.suites.iter().all(|s| s.build_failed);
    if run.build_failed {
        run.partial_build_failed.clear();
    }
    run.build_errors = if run.suites.len() == 1 {
        run.suites[0].build_errors.clone()
    } else {
        run.suites
            .iter()
            .filter(|s| s.build_failed)
            .flat_map(|s| {
                std::iter::once(format!("{}:", s.name))
                    .chain(s.build_errors.iter().map(|l| format!("  {l}")))
            })
            .collect()
    };
    run.pass = !run.suites.is_empty() && run.suites.iter().all(|s| s.pass);
    Ok(run)
}

/// Run the Cargo suite at `root` (a Cargo project) and join results to scenario IDs.
fn run_cargo(root: &Path, spec_root: &Path, scope: &Scope) -> Result<TestRun> {
    if !root.join("Cargo.toml").is_file() {
        bail!(
            "no Cargo.toml in {}; sj test supports Cargo projects only for now",
            root.display()
        );
    }
    let sel = resolve(root, spec_root, scope)?;
    let args: Vec<&str> = sel.cargo_args.iter().map(String::as_str).collect();
    let filters: Vec<&str> = sel.filters.iter().map(String::as_str).collect();
    let d = cargo_test_detailed(root, &args, &filters)?;
    let selected: Vec<(String, Vec<String>)> = sel
        .only
        .as_ref()
        .map(|o| o.iter().map(|(n, i)| (n.clone(), i.clone())).collect())
        .unwrap_or_default();
    if d.build_failed {
        return Ok(TestRun {
            build_failed: true,
            build_errors: d.build_errors,
            selected,
            raw_bytes: d.output_bytes,
            ..TestRun::default()
        });
    }
    let mut run = TestRun {
        selected,
        raw_bytes: d.output_bytes,
        ..TestRun::default()
    };
    for r in &d.run.results {
        let name = r.path.rsplit("::").next().unwrap_or(&r.path);
        if let Some(only) = &sel.only {
            if !only.contains_key(name) {
                continue;
            }
        }
        match r.outcome {
            Outcome::Passed => run.passed += 1,
            Outcome::Failed => run.failed += 1,
            Outcome::Ignored => run.ignored += 1,
        }
        let failure = (r.outcome == Outcome::Failed)
            .then(|| d.failures.get(&r.path))
            .flatten();
        run.tests.push(TestCase {
            path: r.path.clone(),
            result: r.outcome,
            ids: sel.ids.get(name).cloned().unwrap_or_default(),
            excerpt: failure.map(|t| excerpt(t, EXCERPT_LINES)),
            counterexample: failure
                .filter(|_| sel.property.contains(name))
                .and_then(|t| crate::counterexample::counterexample(t)),
        });
    }
    // With a post-filter, a dropped unrelated failure must not fail the run.
    run.pass = run.failed == 0 && (sel.only.is_some() || d.run.success);
    Ok(run)
}
