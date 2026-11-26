//! TRACE-024 from specjutsu/changes/property-scenarios: a Hypothesis test is a property test.

use std::path::Path;

use assert_cmd::Command;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const CONFIG: &str = r#"schema = 1
spec_root = "specjutsu"

[[package]]
path = "."
sources = ["src/**"]

[[package.suite]]
runner = "pytest"
tests = ["tests/**"]
"#;

const SPEC: &str = "## Requirements\n\n### Requirement: Pay\nThe system SHALL refund.\n\n\
     #### Scenario: PAY-020 Refund never exceeds charge\n- **WHEN** x\n- **THEN** y\n";

/// `sj trace --json` on a project with `tests/test_pay.py`, returning `(id, name, property)`
/// per linked test.
fn trace(test_file: &str) -> Vec<(String, String, serde_json::Value)> {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(r, "specjutsu/specs/pay/spec.md", SPEC);
    write(r, "tests/test_pay.py", test_file);
    let out = Command::cargo_bin("sj")
        .unwrap()
        .args(["trace", "--json", "--root"])
        .arg(r)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["mapped"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| {
            m["tests"].as_array().unwrap().iter().map(|t| {
                (
                    m["id"].as_str().unwrap().to_string(),
                    t["name"].as_str().unwrap().to_string(),
                    t["property"].clone(),
                )
            })
        })
        .collect()
}

#[test]
fn trace_024_a_hypothesis_given_test_is_a_property_test() {
    let got = trace(
        "from hypothesis import given, strategies as st\n\n\n\
         @given(st.integers(), st.integers())\n\
         def test_pay_020_refund_le_charge(charge, refund):\n    \
             assert min(refund, charge) <= charge\n",
    );
    assert_eq!(
        got,
        vec![(
            "PAY-020".into(),
            "test_pay_020_refund_le_charge".into(),
            true.into()
        )]
    );
}

// spec: TRACE-024
#[test]
fn a_local_given_decorator_without_the_hypothesis_import_is_not_a_property_test() {
    let got = trace(
        "def given(*strategies):\n    return lambda f: f\n\n\n\
         @given(1, 2)\n\
         def test_pay_020_refund_le_charge(charge, refund):\n    \
             assert min(refund, charge) <= charge\n",
    );
    assert_eq!(
        got,
        vec![(
            "PAY-020".into(),
            "test_pay_020_refund_le_charge".into(),
            false.into()
        )]
    );
}
