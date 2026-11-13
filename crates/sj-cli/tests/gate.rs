//! GATE CLI scenarios (openspec/changes/gate-red-on-base).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

const ADD_BUG: &str = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const MUL: &str = "pub fn mul(a: i32, b: i32) -> i32 { a * b }\n";
const ADD_TEST: &str = "#[test]\nfn math_001_adds() { assert_eq!(demo::add(2, 2), 4); }\n";
const MUL_TEST: &str = "#[test]\nfn math_002_multiplies() { assert_eq!(demo::mul(2, 3), 6); }\n";

fn sj(repo: &Repo) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg("gate").arg("--root").arg(repo.path());
    c
}

#[test]
fn gate_018_passing_gate_prints_one_line_and_exits_zero() {
    let repo = Repo::with_lib(ADD_BUG);
    repo.branch("feature");
    repo.add_change("math", &[("MATH-001", "Add"), ("MATH-002", "Multiply")]);
    repo.write("src/lib.rs", &format!("{ADD_OK}{MUL}"));
    repo.write("tests/add.rs", ADD_TEST);
    repo.write("tests/mul.rs", MUL_TEST);
    repo.commit("feat(MATH-001, MATH-002): green");
    sj(&repo)
        .args(["--base", "main"])
        .assert()
        .success()
        .stdout("gate: pass — 2 scenarios proven (1 red:assert, 1 red:compile)\n");
}

#[test]
fn gate_019_failing_gate_lists_reasons_and_exits_one() {
    let repo = Repo::with_lib(&format!("{ADD_OK}{MUL}"));
    repo.branch("feature");
    repo.add_change("math", &[("MATH-001", "Add"), ("MATH-002", "Multiply")]);
    repo.write("tests/mul.rs", MUL_TEST);
    repo.commit("test(MATH-002): red");
    sj(&repo)
        .args(["--base", "main"])
        .assert()
        .code(1)
        .stdout(predicate::str::starts_with("gate: fail — 2 problems\n"))
        .stdout(predicate::str::contains("fail    untested MATH-001"))
        .stdout(predicate::str::contains(
            "fail    green-on-base MATH-002 math_002_multiplies",
        ));

    let out = sj(&repo)
        .args(["--base", "main", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["pass"], false);
    assert_eq!(v["failures"].as_array().unwrap().len(), 2);
}

#[test]
fn gate_020_missing_base_ref_exits_two() {
    let repo = Repo::with_lib(ADD_OK);
    sj(&repo)
        .args(["--base", "origin/main"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("origin/main"))
        .stderr(predicate::str::contains("fetch-depth: 0"));
}

#[test]
fn gate_021_since_runs_the_history_check() {
    let repo = Repo::with_lib(ADD_OK);
    let since = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.write("a.txt", "1");
    repo.commit("test(GATE-001): red");
    repo.write("a.txt", "2");
    repo.commit("feat(GATE-001): green");
    sj(&repo)
        .args(["--since", &since])
        .assert()
        .success()
        .stdout("gate: pass — history: 1 scenario red before green\n");
}
