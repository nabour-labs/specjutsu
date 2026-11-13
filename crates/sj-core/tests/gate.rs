//! GATE scenarios (openspec/changes/gate-red-on-base).

mod support;

use std::collections::BTreeMap;
use std::path::PathBuf;

use sj_core::gate::{
    classify, evaluate, expand_ids, history, parse_exemptions, run, Change, Exemption, Input,
    Options, Red,
};
use sj_core::spec::{fingerprints, Scenario};
use sj_core::trace::{Mapped, Orphan, Report, TestRef};
use sj_runners::cargo::{Outcome, Run, TestResult};
use support::Repo;

// ---------- helpers ----------

fn input() -> Input {
    Input {
        changes: BTreeMap::new(),
        trace: Report::default(),
        changed_files: vec![],
        exemptions: vec![],
        base_branch: "main".into(),
        head_run: None,
        base: BTreeMap::new(),
        allow_unverifiable: false,
    }
}

fn tref(name: &str) -> TestRef {
    TestRef {
        name: name.into(),
        file: PathBuf::from("tests/auth.rs"),
        line: 1,
    }
}

fn mapped(id: &str, test: &str) -> Mapped {
    Mapped {
        id: id.into(),
        name: "x".into(),
        tests: vec![tref(test)],
    }
}

fn head(results: &[(&str, Outcome)]) -> Option<Run> {
    Some(Run {
        success: results.iter().all(|(_, o)| *o != Outcome::Failed),
        results: results
            .iter()
            .map(|(p, o)| TestResult {
                path: p.to_string(),
                outcome: *o,
            })
            .collect(),
    })
}

fn added_and_proven(id: &str, test: &str) -> Input {
    let mut i = input();
    i.changes.insert(id.into(), Change::Added);
    i.trace.mapped.push(mapped(id, test));
    i.head_run = head(&[(&format!("tests::{test}"), Outcome::Passed)]);
    i.base.insert((id.into(), test.into()), Red::Assert);
    i
}

fn has(list: &[String], needle: &str) -> bool {
    list.iter().any(|l| l.contains(needle))
}

fn opts(repo: &Repo) -> Options {
    Options {
        root: repo.path().to_path_buf(),
        spec_root: PathBuf::from("openspec"),
        base: "main".into(),
        allow_unverifiable: false,
    }
}

const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const ADD_BUG: &str = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
const ADD_TEST: &str = "#[test]\nfn math_001_adds() { assert_eq!(demo::add(2, 2), 4); }\n";

// ---------- classification ----------

#[test]
fn gate_001_classify_added_modified_removed_unchanged() {
    let base = fingerprints(
        "#### Scenario: AUTH-001 One\n- **WHEN** a\n\n#### Scenario: AUTH-002 Two\n- **WHEN** b\n- **THEN** c\n\n#### Scenario: AUTH-003 Three\n- **WHEN** d\n",
    );
    let head = fingerprints(
        "#### Scenario: AUTH-001 One\n- **WHEN**   a\n\n#### Scenario: AUTH-002 Two\n- **WHEN** b\n- **THEN** c, reworded\n\n#### Scenario: AUTH-004 Four\n- **WHEN** e\n",
    );
    let got = classify(&base, &head);
    let expect: BTreeMap<String, Change> = [
        ("AUTH-001", Change::Unchanged),
        ("AUTH-002", Change::Modified),
        ("AUTH-003", Change::Removed),
        ("AUTH-004", Change::Added),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    assert_eq!(got, expect);
}

#[test]
fn gate_001_fingerprint_stops_at_next_heading_and_skips_fences() {
    let f = fingerprints(
        "#### Scenario: AUTH-001 One\n- **WHEN** a\n### Requirement: Next\nBody\n```\n#### Scenario: AUTH-999 Fenced\n```\n",
    );
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].id, "AUTH-001");
    assert!(!f[0].text.contains("Body"));
}

#[test]
fn gate_002_moving_a_scenario_between_files_is_not_a_change() {
    let delta =
        "## ADDED Requirements\n\n### Requirement: R\n#### Scenario: AUTH-001 One\n- **WHEN** a\n";
    let current =
        "## Requirements\n\n### Requirement: R\n\n#### Scenario: AUTH-001 One\n- **WHEN** a\n";
    let got = classify(&fingerprints(delta), &fingerprints(current));
    assert_eq!(got["AUTH-001"], Change::Unchanged);
}

// ---------- rules (pure) ----------

#[test]
fn gate_003_added_scenario_without_a_test_fails() {
    let mut i = input();
    i.changes.insert("AUTH-004".into(), Change::Added);
    let r = evaluate(&i);
    assert!(!r.pass);
    assert!(has(&r.failures, "untested AUTH-004"), "{:?}", r.failures);
}

#[test]
fn gate_004_unchanged_gap_warns_but_passes() {
    let mut i = input();
    i.changes.insert("AUTH-001".into(), Change::Unchanged);
    i.trace.gaps.push(Scenario {
        id: Some("AUTH-001".into()),
        name: "x".into(),
        file: PathBuf::from("spec.md"),
        line: 1,
    });
    let r = evaluate(&i);
    assert!(r.pass, "{:?}", r.failures);
    assert!(has(&r.warnings, "gap AUTH-001"));
}

#[test]
fn gate_005_failing_mapped_test_fails() {
    let mut i = added_and_proven("AUTH-004", "auth_004_rejects");
    i.head_run = head(&[("tests::auth_004_rejects", Outcome::Failed)]);
    let r = evaluate(&i);
    assert!(
        has(&r.failures, "failing AUTH-004 auth_004_rejects"),
        "{:?}",
        r.failures
    );
}

#[test]
fn gate_006_mapped_test_that_does_not_run_fails() {
    let mut i = added_and_proven("AUTH-004", "auth_004_rejects");
    i.head_run = head(&[("tests::other", Outcome::Passed)]);
    assert!(has(
        &evaluate(&i).failures,
        "not-run AUTH-004 auth_004_rejects"
    ));
    i.head_run = head(&[("tests::auth_004_rejects", Outcome::Ignored)]);
    assert!(has(
        &evaluate(&i).failures,
        "not-run AUTH-004 auth_004_rejects"
    ));
}

#[test]
fn gate_008_proven_scenario_passes() {
    let r = evaluate(&added_and_proven("AUTH-004", "auth_004_rejects"));
    assert!(r.pass, "{:?}", r.failures);
    assert_eq!(r.proven.len(), 1);
    assert_eq!(r.proven[0].red, Red::Assert);
}

#[test]
fn gate_012_test_referencing_a_removed_scenario_fails() {
    let mut i = input();
    i.changes.insert("AUTH-003".into(), Change::Removed);
    i.trace.orphans.push(Orphan {
        id: "AUTH-003".into(),
        test: tref("auth_003_old"),
    });
    assert!(has(&evaluate(&i).failures, "orphan AUTH-003 auth_003_old"));
}

#[test]
fn gate_013_unspecified_implementation_change_fails() {
    let mut i = input();
    i.changed_files = vec!["src/auth.rs".into(), "README.md".into()];
    let r = evaluate(&i);
    assert!(
        has(&r.failures, "unspecified change: src/auth.rs"),
        "{:?}",
        r.failures
    );
    assert!(!has(&r.failures, "README.md"));
}

#[test]
fn gate_014_documentation_only_change_passes() {
    let mut i = input();
    i.changed_files = vec![
        "README.md".into(),
        "docs/guide.md".into(),
        ".github/workflows/ci.yml".into(),
        "openspec/changes/x/proposal.md".into(),
    ];
    assert!(evaluate(&i).pass);
}

#[test]
fn gate_015_refactor_skips_red_on_base_but_requires_green() {
    let mut i = input();
    i.changed_files = vec!["src/auth.rs".into()];
    i.exemptions = vec![Exemption {
        kind: "refactor".into(),
        reason: "extract token parser".into(),
    }];
    i.head_run = head(&[("tests::a", Outcome::Passed)]);
    let r = evaluate(&i);
    assert!(r.pass, "{:?}", r.failures);
    assert!(has(&r.exemptions, "refactor — extract token parser"));

    i.head_run = head(&[("tests::a", Outcome::Passed), ("tests::b", Outcome::Failed)]);
    assert!(has(&evaluate(&i).failures, "failing tests::b"));
}

#[test]
fn gate_015_parse_exemption_lines() {
    let text =
        "## Why\nExempt: refactor — extract token parser\nExempt: docs - fix typo\nnot Exempt: x\n";
    assert_eq!(
        parse_exemptions(text, "Exempt:"),
        vec![
            Exemption {
                kind: "refactor".into(),
                reason: "extract token parser".into()
            },
            Exemption {
                kind: "docs".into(),
                reason: "fix typo".into()
            },
        ]
    );
    assert_eq!(
        parse_exemptions(
            "fix\n\nSpecjutsu-Exempt: chore — bump deps\n",
            "Specjutsu-Exempt:"
        ),
        vec![Exemption {
            kind: "chore".into(),
            reason: "bump deps".into()
        }]
    );
}

#[test]
fn gate_016_exemption_without_a_reason_is_rejected() {
    let ex = parse_exemptions("Exempt: chore\n", "Exempt:");
    assert_eq!(
        ex,
        vec![Exemption {
            kind: "chore".into(),
            reason: String::new()
        }]
    );
    let mut i = input();
    i.exemptions = ex;
    assert!(has(&evaluate(&i).failures, "exemption needs a reason"));
}

#[test]
fn gate_017_spike_exemptions_are_blocked_from_main() {
    let mut i = input();
    i.changed_files = vec!["src/auth.rs".into()];
    i.exemptions = vec![Exemption {
        kind: "spike".into(),
        reason: "try new parser".into(),
    }];
    for base in ["main", "origin/main"] {
        i.base_branch = base.into();
        assert!(
            has(&evaluate(&i).failures, "spike cannot merge to main"),
            "{base}"
        );
    }
    i.base_branch = "spike/parser".into();
    let r = evaluate(&i);
    assert!(r.pass, "{:?}", r.failures);
}

#[test]
fn gate_018_pass_summary_counts_red_kinds() {
    let mut i = added_and_proven("MATH-001", "math_001_adds");
    i.changes.insert("MATH-002".into(), Change::Added);
    i.trace
        .mapped
        .push(mapped("MATH-002", "math_002_multiplies"));
    i.head_run = head(&[
        ("math_001_adds", Outcome::Passed),
        ("math_002_multiplies", Outcome::Passed),
    ]);
    i.base.insert(
        ("MATH-002".into(), "math_002_multiplies".into()),
        Red::Compile,
    );
    let r = evaluate(&i);
    assert_eq!(
        r.summary,
        "gate: pass — 2 scenarios proven (1 red:assert, 1 red:compile)"
    );
}

#[test]
fn gate_019_fail_summary_counts_problems() {
    let mut i = input();
    i.changes.insert("AUTH-004".into(), Change::Added);
    i.changes.insert("AUTH-005".into(), Change::Added);
    let r = evaluate(&i);
    assert_eq!(r.summary, "gate: fail — 2 problems");
}

// ---------- red-on-base (real repos) ----------

#[test]
fn gate_007_new_test_that_passes_on_base_fails() {
    let repo = Repo::with_lib(ADD_OK);
    repo.branch("feature");
    repo.add_change("add-math", &[("MATH-001", "Add numbers")]);
    repo.write("tests/math.rs", ADD_TEST);
    repo.commit("test(MATH-001): red");
    let r = run(&opts(&repo)).unwrap();
    assert!(!r.pass);
    assert!(
        has(&r.failures, "green-on-base MATH-001 math_001_adds"),
        "{:?}",
        r.failures
    );
}

#[test]
fn gate_008_assertion_failure_on_base_is_red() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.branch("feature");
    repo.add_change("fix-add", &[("MATH-001", "Add numbers")]);
    repo.write("tests/math.rs", ADD_TEST);
    repo.write("src/lib.rs", ADD_OK);
    repo.commit("feat(MATH-001): green");
    let r = run(&opts(&repo)).unwrap();
    assert!(r.pass, "{:?}", r.failures);
    assert_eq!(r.proven[0].red, Red::Assert);
}

#[test]
fn gate_009_compile_failure_on_base_is_red_compile() {
    let repo = Repo::with_lib("pub fn other() {}\n");
    repo.branch("feature");
    repo.add_change("add-math", &[("MATH-001", "Add numbers")]);
    repo.write("tests/math.rs", ADD_TEST);
    repo.write("src/lib.rs", ADD_OK);
    repo.commit("feat(MATH-001): green");
    let r = run(&opts(&repo)).unwrap();
    assert!(r.pass, "{:?}", r.failures);
    assert_eq!(r.proven[0].red, Red::Compile);
}

#[test]
fn gate_010_only_test_files_are_overlaid_and_head_is_untouched() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.branch("feature");
    repo.add_change("fix-add", &[("MATH-001", "Add numbers")]);
    repo.write("tests/math.rs", ADD_TEST);
    repo.write("src/lib.rs", ADD_OK);
    repo.commit("feat(MATH-001): green");
    let r = run(&opts(&repo)).unwrap();
    // Red on base proves base's src/lib.rs (the bug) was used, not head's.
    assert_eq!(r.proven[0].red, Red::Assert);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    assert_eq!(repo.git(&["worktree", "list"]).lines().count(), 1);
    assert_eq!(
        std::fs::read_to_string(repo.path().join("src/lib.rs")).unwrap(),
        ADD_OK
    );
}

#[test]
fn gate_011_inline_tests_are_unverifiable() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.branch("feature");
    repo.add_change("fix-add", &[("MATH-001", "Add numbers")]);
    repo.write(
        "src/lib.rs",
        &format!("{ADD_OK}\n#[cfg(test)]\nmod t {{\n    #[test]\n    fn math_001_adds() {{ assert_eq!(super::add(2, 2), 4); }}\n}}\n"),
    );
    repo.commit("feat(MATH-001): green");
    let r = run(&opts(&repo)).unwrap();
    assert!(!r.pass);
    assert!(
        has(&r.failures, "unverifiable MATH-001 math_001_adds"),
        "{:?}",
        r.failures
    );
    assert!(has(&r.failures, "tests/"));

    let mut o = opts(&repo);
    o.allow_unverifiable = true;
    let r = run(&o).unwrap();
    assert!(r.pass, "{:?}", r.failures);
    assert!(has(&r.warnings, "unverifiable MATH-001"));
}

#[test]
fn gate_020_missing_base_ref_is_an_error() {
    let repo = Repo::with_lib(ADD_OK);
    let mut o = opts(&repo);
    o.base = "origin/main".into();
    let err = format!("{:#}", run(&o).unwrap_err());
    assert!(err.contains("origin/main"), "{err}");
    assert!(err.contains("fetch-depth: 0"), "{err}");
}

#[test]
fn gate_023_implementing_a_previously_untested_scenario_is_gated() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.write(
        "openspec/specs/math/spec.md",
        &support::spec_md(&[("MATH-001", "Add numbers")]),
    );
    repo.commit("docs(spec): MATH-001");
    repo.branch("feature");
    repo.write("tests/math.rs", ADD_TEST);
    repo.write("src/lib.rs", ADD_OK);
    repo.commit("feat(MATH-001): green");
    let r = run(&opts(&repo)).unwrap();
    assert!(r.pass, "{:?}", r.failures);
    assert_eq!(r.proven.len(), 1);
    assert_eq!(r.proven[0].id, "MATH-001");
    assert_eq!(r.proven[0].red, Red::Assert);
}

#[test]
fn gate_023_implemented_is_treated_like_added() {
    let mut i = input();
    i.changes.insert("AUTH-004".into(), Change::Implemented);
    assert!(has(&evaluate(&i).failures, "untested AUTH-004"));
}

// ---------- history ----------

#[test]
fn gate_021_history_with_red_before_green_passes() {
    let repo = Repo::with_lib(ADD_OK);
    let since = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.write("a.txt", "1");
    repo.commit("test(TRACE-001..006): red");
    repo.write("a.txt", "2");
    repo.commit("feat(TRACE-001..006): green");
    let r = history(repo.path(), &since).unwrap();
    assert!(r.pass, "{:?}", r.failures);
    assert_eq!(
        r.summary,
        "gate: pass — history: 6 scenarios red before green"
    );
}

#[test]
fn gate_021_expand_id_lists() {
    assert_eq!(
        expand_ids("TRACE-001..003"),
        vec!["TRACE-001", "TRACE-002", "TRACE-003"]
    );
    assert_eq!(
        expand_ids("TRACE-009..TRACE-010"),
        vec!["TRACE-009", "TRACE-010"]
    );
    assert_eq!(
        expand_ids("GATE-001, GATE-004"),
        vec!["GATE-001", "GATE-004"]
    );
}

#[test]
fn gate_022_green_without_an_earlier_red_fails() {
    let repo = Repo::with_lib(ADD_OK);
    let since = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.write("a.txt", "1");
    repo.commit("feat(GATE-003): green");
    repo.write("a.txt", "2");
    repo.commit("test(GATE-003): red");
    let r = history(repo.path(), &since).unwrap();
    assert!(!r.pass);
    assert!(
        has(&r.failures, "no red commit GATE-003"),
        "{:?}",
        r.failures
    );
}
