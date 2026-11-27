//! LEDGER-013, LEDGER-015 from specjutsu/changes/property-counterexamples: the counterexample
//! a failing property test printed, kept on its `sj test` ledger record.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use support::Repo;

const REFUND_BASE: &str = "pub fn refund(_charge: u32, asked: u32) -> u32 { asked }\n";

const PAY_SPEC: &str =
    "## ADDED Requirements\n\n### Requirement: Refunds\nThe system SHALL refund.\n\n\
#### Scenario: PAY-020 A refund never exceeds the charge\n\
- **FOR ALL** charges and refund requests\n- **WHEN** a refund is made\n\
- **THEN** it never exceeds the charge\n\n\
#### Scenario: FOO-001 An old scenario\n- **WHEN** called\n- **THEN** it works\n\n\
#### Scenario: FOO-002 A refund is logged\n- **WHEN** a refund is made\n- **THEN** it is logged\n";

/// A stand-in for the proptest crate: the scanner only needs the `proptest!` block.
const PROPTEST_MACRO: &str = "macro_rules! proptest { ($($t:tt)*) => { $($t)* }; }\n\n";

/// A property test that fails the way proptest reports a shrunk failure.
fn proptest_failure() -> String {
    format!(
        "{PROPTEST_MACRO}proptest! {{\n    #[test]\n    fn pay_020_refund_never_exceeds_charge() {{\n        \
         let (charge, refund) = (100, 101);\n        \
         if demo::refund(charge, refund) > charge {{\n            \
         panic!(\"Test failed: refund exceeds charge; minimal failing input: charge = {{charge}}, refund = {{refund}}\");\n        \
         }}\n    }}\n}}\n"
    )
}

/// An example test whose failure text happens to look like a counterexample.
const EXAMPLE_FAILURE: &str = "#[test]\nfn foo_002_logs_a_refund() {\n    \
panic!(\"not logged; minimal failing input: charge = 1, refund = 2\");\n}\n";

fn sj(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .arg(args[0])
        .arg("--root")
        .arg(root)
        .args(&args[1..])
        .output()
        .unwrap()
}

fn repo() -> Repo {
    let repo = Repo::with_lib(REFUND_BASE);
    repo.write("openspec/specs/pay/spec.md", PAY_SPEC);
    // One test binary, so cargo runs both tests even though one fails.
    repo.write(
        "tests/pay.rs",
        &format!("{}\n{EXAMPLE_FAILURE}", proptest_failure()),
    );
    repo
}

fn ledger_lines(repo: &Repo) -> Vec<serde_json::Value> {
    std::fs::read_to_string(repo.path().join(".specjutsu/ledger/runs.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn record_for<'a>(lines: &'a [serde_json::Value], id: &str) -> &'a serde_json::Value {
    lines
        .iter()
        .find(|r| r["ids"].as_array().unwrap().iter().any(|i| i == id))
        .unwrap_or_else(|| panic!("no record for {id}: {lines:#?}"))
}

#[test]
fn ledger_013_failing_property_test_record_holds_its_counterexample() {
    let repo = repo();
    let out = sj(repo.path(), &["test", "PAY-020"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");

    let lines = ledger_lines(&repo);
    let r = record_for(&lines, "PAY-020");
    assert!(
        r["test"]
            .as_str()
            .unwrap()
            .ends_with("pay_020_refund_never_exceeds_charge"),
        "{r:#}"
    );
    assert_eq!(r["result"], "failed", "{r:#}");
    assert_eq!(r["counterexample"], "charge = 100, refund = 101", "{r:#}");
}

#[test]
fn ledger_015_records_without_a_counterexample_stay_as_they_are() {
    let repo = repo();
    // A record written before this change: no counterexample field.
    let old = r#"{"v":1,"run":"1-1","ts":1,"tree":"0000","test":"foo_001_old","ids":["FOO-001"],"result":"failed"}"#;
    repo.write(".specjutsu/ledger/runs.jsonl", &format!("{old}\n"));

    let out = sj(repo.path(), &["test"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");

    let lines = ledger_lines(&repo);
    // Control: in the same run, the property test's failure does keep its counterexample.
    assert_eq!(
        record_for(&lines, "PAY-020")["counterexample"],
        "charge = 100, refund = 101"
    );
    let old_rec = record_for(&lines, "FOO-001");
    assert_eq!(old_rec["test"], "foo_001_old");
    assert!(old_rec.get("counterexample").is_none(), "{old_rec:#}");
    let example = record_for(&lines, "FOO-002");
    assert_eq!(example["result"], "failed", "{example:#}");
    assert!(example.get("counterexample").is_none(), "{example:#}");

    let out = sj(repo.path(), &["ledger", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let scenarios = v["scenarios"].as_array().unwrap();
    for id in ["FOO-001", "FOO-002"] {
        let s = scenarios
            .iter()
            .find(|s| s["id"] == id)
            .unwrap_or_else(|| panic!("{id} missing: {v:#}"));
        assert_eq!(s["latest"], "failed", "{v:#}");
        assert!(s.get("counterexample").is_none(), "{v:#}");
    }
    assert!(
        String::from_utf8_lossy(&out.stderr).is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
