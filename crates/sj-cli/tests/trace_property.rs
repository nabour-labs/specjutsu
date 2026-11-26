//! TRACE-022 and TRACE-023 from specjutsu/changes/property-scenarios: proptest tests are
//! property tests.

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
runner = "cargo"
tests = ["tests/**"]
"#;

const SPEC: &str = "## Requirements\n\n### Requirement: Pay\nThe system SHALL refund.\n\n\
     #### Scenario: PAY-020 Refund never exceeds charge\n- **WHEN** x\n- **THEN** y\n\n\
     #### Scenario: PAY-021 Plain refund\n- **WHEN** x\n- **THEN** y\n";

/// `sj trace --json` on a project with `tests/pay.rs`, returning `(id, name, property)` per linked test.
fn trace(test_file: &str) -> Vec<(String, String, serde_json::Value)> {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(r, "specjutsu/specs/pay/spec.md", SPEC);
    write(r, "tests/pay.rs", test_file);
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
fn trace_022_a_test_inside_a_proptest_block_is_a_property_test() {
    let got = trace(
        "use proptest::prelude::*;\n\n\
         proptest! {\n    \
             #[test]\n    \
             fn pay_020_refund_le_charge(charge in 0u64..10_000, refund in 0u64..10_000) {\n        \
                 let s = \"}\";\n        \
                 prop_assert!(refund.min(charge) <= charge, \"{}\", s);\n    \
             }\n\
         }\n\n\
         #[test]\nfn pay_021_plain_refund() {}\n",
    );
    assert_eq!(
        got,
        vec![
            (
                "PAY-020".into(),
                "pay_020_refund_le_charge".into(),
                true.into()
            ),
            (
                "PAY-021".into(),
                "pay_021_plain_refund".into(),
                false.into()
            ),
        ]
    );
}

#[test]
fn trace_023_a_proptest_attribute_marks_a_property_test() {
    let got = trace(
        "use test_strategy::proptest;\n\n\
         #[proptest]\n\
         fn pay_020_refund_le_charge(charge: u64, refund: u64) {\n    \
             assert!(refund.min(charge) <= charge);\n\
         }\n",
    );
    assert_eq!(
        got,
        vec![(
            "PAY-020".into(),
            "pay_020_refund_le_charge".into(),
            true.into()
        )]
    );
}
