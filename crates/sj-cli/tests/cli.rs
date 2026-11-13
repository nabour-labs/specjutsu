use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/trace-basic")
}

#[test]
fn trace_006_compact_report_exits_zero() {
    Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--root"])
        .arg(fixture())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "trace: 2 mapped, 1 gap, 1 orphan, 1 without ID",
        ))
        .stdout(predicate::str::contains("gap     AUTH-002 Reject a wrong password"))
        .stdout(predicate::str::contains("orphan  AUTH-009 auth_009_points_at_nothing"));
}

#[test]
fn trace_006_json_output() {
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--json", "--root"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["gaps"][0]["id"], "AUTH-002");
    assert_eq!(v["orphans"][0]["id"], "AUTH-009");
    assert_eq!(v["mapped"].as_array().unwrap().len(), 2);
}

#[test]
fn trace_006_strict_exits_one_on_gaps() {
    Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--strict", "--root"])
        .arg(fixture())
        .assert()
        .code(1);
}
