//! TRACE-027..029 from specjutsu/changes/property-scenarios: `sj trace` lists FOR ALL scenarios whose
//! tests are all example tests, judged by an open change's copy of each scenario.

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

// PAY-020: FOR ALL, example test only. PAY-022: FOR ALL, property test. PAY-023: FOR ALL, no test.
const SPEC: &str = "## Requirements\n\n### Requirement: Pay\nThe system SHALL refund.\n\n\
     #### Scenario: PAY-020 Refund never exceeds charge\n\
     - **FOR ALL** charges and refunds\n- **WHEN** x\n- **THEN** y\n\n\
     #### Scenario: PAY-022 Refund is never negative\n\
     - **FOR ALL** refunds\n- **WHEN** x\n- **THEN** y\n\n\
     #### Scenario: PAY-023 Charge is never negative\n\
     - **FOR ALL** charges\n- **WHEN** x\n- **THEN** y\n";

// No source line starts with `#[test]` or `fn`, so the scanner doesn't read the fixture as tests.
const TESTS: &str = "use proptest::prelude::*;\n\n#[test]\nfn pay_020_refund_of_50_on_100() {}\n\n\
     proptest! {\n    #[test]\n    fn pay_022_refund_ge_zero(refund in 0u64..10_000) {\n        \
     prop_assert!(refund >= 0);\n    }\n}\n";

fn sj(root: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("sj")
        .unwrap()
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn trace_027_a_for_all_scenario_with_only_example_tests_is_listed() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(r, "specjutsu/specs/pay/spec.md", SPEC);
    write(r, "tests/pay.rs", TESTS);

    let text = String::from_utf8(sj(r, &["trace"]).stdout).unwrap();
    let listed: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with("not-property"))
        .collect();
    assert_eq!(
        listed,
        vec!["not-property PAY-020 pay_020_refund_of_50_on_100"]
    );
    assert!(
        text.lines().any(|l| l.starts_with("gap     PAY-023 ")),
        "{text}"
    );

    let json: serde_json::Value =
        serde_json::from_slice(&sj(r, &["trace", "--json"]).stdout).unwrap();
    assert_eq!(json["not_property"], serde_json::json!(["PAY-020"]));

    // Only the not-property entry can fail --strict: accept the PAY-023 gap in the baseline.
    write(r, "specjutsu/trace-baseline.txt", "gap PAY-023\n");
    let json: serde_json::Value =
        serde_json::from_slice(&sj(r, &["trace", "--json"]).stdout).unwrap();
    assert_eq!(json["not_property"], serde_json::json!(["PAY-020"]));
    assert_eq!(sj(r, &["trace", "--strict"]).status.code(), Some(1));
}

const PAY_HEAD: &str = "## Requirements\n\n### Requirement: Pay\nThe system SHALL refund.\n\n";
const PAY_020_EXAMPLE: &str = "#### Scenario: PAY-020 Refund never exceeds charge\n\
     - **WHEN** x\n- **THEN** y\n";
const PAY_020_FOR_ALL: &str = "#### Scenario: PAY-020 Refund never exceeds charge\n\
     - **FOR ALL** charges and refund requests\n- **WHEN** x\n- **THEN** y\n";
const MODIFIED_HEAD: &str =
    "## MODIFIED Requirements\n\n### Requirement: Pay\nThe system SHALL refund.\n\n";
const EXAMPLE_TEST: &str = "#[test]\nfn pay_020_refund_of_50_on_100() {}\n";

fn not_property_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| l.starts_with("not-property"))
        .collect()
}

#[test]
fn trace_028_a_change_that_adds_for_all_to_a_scenario_is_checked_by_its_copy() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(
        r,
        "specjutsu/specs/pay/spec.md",
        &format!("{PAY_HEAD}{PAY_020_EXAMPLE}"),
    );
    write(
        r,
        "specjutsu/changes/refunds/specs/pay/spec.md",
        &format!("{MODIFIED_HEAD}{PAY_020_FOR_ALL}"),
    );
    write(r, "tests/pay.rs", EXAMPLE_TEST);

    let text = String::from_utf8(sj(r, &["trace"]).stdout).unwrap();
    assert_eq!(
        not_property_lines(&text),
        vec!["not-property PAY-020 pay_020_refund_of_50_on_100"]
    );
}

#[test]
fn trace_029_a_change_that_removes_for_all_from_a_scenario_is_not_listed() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(
        r,
        "specjutsu/specs/pay/spec.md",
        &format!("{PAY_HEAD}{PAY_020_FOR_ALL}"),
    );
    write(
        r,
        "specjutsu/changes/refunds/specs/pay/spec.md",
        &format!("{MODIFIED_HEAD}{PAY_020_EXAMPLE}"),
    );
    write(r, "tests/pay.rs", EXAMPLE_TEST);

    let text = String::from_utf8(sj(r, &["trace"]).stdout).unwrap();
    assert_eq!(not_property_lines(&text), Vec::<&str>::new(), "{text}");
    let strict = sj(r, &["trace", "--strict"]);
    assert_eq!(
        strict.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&strict.stdout)
    );
}
