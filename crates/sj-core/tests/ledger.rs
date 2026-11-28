//! LEDGER scenarios covered by sj-core (specjutsu/changes/test-ledger).

mod support;

use std::path::Path;

use sj_core::git::Git;
use sj_core::ledger::{self, LedgerResult, Record, Recorded};
use sj_core::run::{run_tests, Scope, TestCase, TestRun};
use sj_runners::cargo::Outcome;
use support::{spec_md, Repo};

fn case(path: &str, ids: &[&str], result: Outcome) -> TestCase {
    TestCase {
        path: path.into(),
        result,
        ids: ids.iter().map(|s| s.to_string()).collect(),
        excerpt: None,
        counterexample: None,
    }
}

fn three_tests() -> TestRun {
    TestRun {
        tests: vec![
            case("tests::foo_001_adds", &["FOO-001"], Outcome::Passed),
            case("foo_002_subtracts", &["FOO-002"], Outcome::Failed),
            case("slow", &[], Outcome::Ignored),
        ],
        ..TestRun::default()
    }
}

fn rec(id: &str, tree: &str, result: LedgerResult) -> Record {
    Record {
        v: ledger::FORMAT,
        run: "r".into(),
        ts: 0,
        tree: tree.into(),
        test: format!("t_{id}"),
        ids: vec![id.into()],
        result,
        counterexample: None,
    }
}

#[test]
fn ledger_001_a_run_appends_one_record_per_test() {
    let recs = ledger::records_for(&three_tests(), "run-1", 42, "abc");
    assert_eq!(recs.len(), 3);
    assert!(recs
        .iter()
        .all(|r| r.run == "run-1" && r.ts == 42 && r.tree == "abc"));
    assert!(recs.iter().all(|r| r.v == ledger::FORMAT));
    let got: Vec<(&str, Vec<String>, LedgerResult)> = recs
        .iter()
        .map(|r| (r.test.as_str(), r.ids.clone(), r.result))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                "tests::foo_001_adds",
                vec!["FOO-001".to_string()],
                LedgerResult::Passed
            ),
            (
                "foo_002_subtracts",
                vec!["FOO-002".to_string()],
                LedgerResult::Failed
            ),
            ("slow", vec![], LedgerResult::Ignored),
        ]
    );

    let dir = tempfile::tempdir().unwrap();
    ledger::append(dir.path(), &recs).unwrap();
    assert!(dir.path().join(".specjutsu/ledger/runs.jsonl").is_file());
    let loaded = ledger::load(dir.path()).unwrap();
    assert_eq!(loaded.records, recs);
    assert_eq!(loaded.skipped, 0);
}

#[test]
fn ledger_002_records_accumulate_across_runs() {
    let dir = tempfile::tempdir().unwrap();
    let first = ledger::records_for(&three_tests(), "run-1", 1, "aaa");
    let second = ledger::records_for(&three_tests(), "run-2", 2, "bbb");
    ledger::append(dir.path(), &first).unwrap();
    ledger::append(dir.path(), &second).unwrap();
    let loaded = ledger::load(dir.path()).unwrap();
    assert_eq!(loaded.records.len(), 6);
    assert_eq!(&loaded.records[..3], &first[..]);
    assert_eq!(&loaded.records[3..], &second[..]);
}

#[test]
fn ledger_003_build_failure_records_build_failed_for_scoped_tests() {
    let repo = Repo::with_lib("");
    repo.write(
        "openspec/specs/auth/spec.md",
        &spec_md(&[("AUTH-003", "Reject")]),
    );
    repo.write(
        "tests/auth.rs",
        "#[test]\nfn auth_003_rejects() { nope(); }\n",
    );
    let run = run_tests(
        repo.path(),
        Path::new("openspec"),
        &Scope::Scenario("AUTH-003".into()),
    )
    .unwrap();
    assert!(run.build_failed);
    let recs = ledger::records_for(&run, "r", 1, "t");
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].test, "auth_003_rejects");
    assert_eq!(recs[0].ids, ["AUTH-003"]);
    assert_eq!(recs[0].result, LedgerResult::BuildFailed);

    // A whole-workspace build failure is not evidence for any one scenario.
    let ws = TestRun {
        build_failed: true,
        ..TestRun::default()
    };
    assert!(ledger::records_for(&ws, "r", 1, "t").is_empty());
}

#[test]
fn ledger_004_uncommitted_edits_change_the_tree_hash() {
    let repo = Repo::with_lib("pub fn a() {}\n");
    let git = Git::new(repo.path());
    let h1 = git.worktree_tree_hash().unwrap();
    assert_eq!(h1.len(), 40, "{h1}");
    assert_eq!(h1, git.worktree_tree_hash().unwrap(), "hash must be stable");
    repo.write("src/lib.rs", "pub fn a() { let _ = 1; }\n");
    let h2 = git.worktree_tree_hash().unwrap();
    assert_ne!(h1, h2);
    repo.write("src/new.rs", "// untracked\n");
    let h3 = git.worktree_tree_hash().unwrap();
    assert_ne!(h2, h3, "untracked, non-ignored files count");
    repo.write("target/junk", "ignored");
    assert_eq!(
        h3,
        git.worktree_tree_hash().unwrap(),
        "ignored files do not count"
    );
}

#[test]
fn ledger_005_hashing_leaves_the_index_untouched() {
    let repo = Repo::with_lib("pub fn a() {}\n");
    repo.write("src/lib.rs", "pub fn a() { let _ = 1; }\n");
    repo.git(&["add", "src/lib.rs"]);
    repo.write("src/lib.rs", "pub fn a() { let _ = 2; }\n");
    repo.write("notes.txt", "untracked\n");
    let status = || repo.git(&["status", "--porcelain"]);
    let staged = || repo.git(&["diff", "--cached"]);
    let (s0, c0) = (status(), staged());
    Git::new(repo.path()).worktree_tree_hash().unwrap();
    assert_eq!(status(), s0);
    assert_eq!(staged(), c0);
}

#[test]
fn ledger_007_outside_git_recording_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!Git::new(dir.path()).is_repo());
    let got = ledger::record(dir.path(), &three_tests()).unwrap();
    assert_eq!(got, Recorded::NotGit);
    assert!(!dir.path().join(".specjutsu").exists());

    let repo = Repo::with_lib("");
    let got = ledger::record(repo.path(), &three_tests()).unwrap();
    assert_eq!(got, Recorded::Records(3));
    assert_eq!(ledger::load(repo.path()).unwrap().records.len(), 3);
}

#[test]
fn ledger_008_red_then_green_on_different_trees_is_proven() {
    let recs = [
        rec("AUTH-003", "A", LedgerResult::Failed),
        rec("AUTH-003", "B", LedgerResult::Passed),
    ];
    let r = ledger::report(&recs);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].id, "AUTH-003");
    assert_eq!(r[0].latest, LedgerResult::Passed);
    assert!(r[0].proven);

    let compile_red = [
        rec("AUTH-007", "A", LedgerResult::BuildFailed),
        rec("AUTH-007", "B", LedgerResult::Passed),
    ];
    assert!(
        ledger::report(&compile_red)[0].proven,
        "build-failed counts as red"
    );
}

#[test]
fn ledger_009_green_only_or_green_before_red_is_not_proven() {
    let recs = [
        rec("AUTH-004", "A", LedgerResult::Passed),
        rec("AUTH-004", "B", LedgerResult::Passed),
        rec("AUTH-005", "A", LedgerResult::Passed),
        rec("AUTH-005", "B", LedgerResult::Failed),
    ];
    let r = ledger::report(&recs);
    let get = |id: &str| r.iter().find(|s| s.id == id).unwrap();
    assert!(!get("AUTH-004").proven);
    assert!(!get("AUTH-005").proven);
    assert_eq!(get("AUTH-005").latest, LedgerResult::Failed);
}

#[test]
fn ledger_010_red_and_green_on_the_same_tree_is_not_proven() {
    let recs = [
        rec("AUTH-006", "A", LedgerResult::Failed),
        rec("AUTH-006", "A", LedgerResult::Passed),
    ];
    assert!(!ledger::report(&recs)[0].proven);
}

#[test]
fn ledger_012_unparsable_lines_are_skipped_and_counted() {
    let dir = tempfile::tempdir().unwrap();
    let good = ledger::records_for(&three_tests(), "r", 1, "t");
    ledger::append(dir.path(), &good[..1]).unwrap();
    let p = dir.path().join(".specjutsu/ledger/runs.jsonl");
    let mut text = std::fs::read_to_string(&p).unwrap();
    text.push_str("{not json\n");
    text.push_str("{\"v\":99,\"future\":true}\n");
    std::fs::write(&p, text).unwrap();
    ledger::append(dir.path(), &good[1..2]).unwrap();
    let loaded = ledger::load(dir.path()).unwrap();
    assert_eq!(loaded.records, good[..2].to_vec());
    assert_eq!(loaded.skipped, 2);
}

#[test]
fn ledger_012_missing_ledger_loads_empty() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = ledger::load(dir.path()).unwrap();
    assert!(loaded.records.is_empty());
    assert_eq!(loaded.skipped, 0);
}
