//! GATE-068, GATE-071, GATE-072 from specjutsu/changes/property-counterexamples: the
//! counterexample a property test printed on base, in the gate's text and JSON reports.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use support::Repo;

const REFUND_BASE: &str = "pub fn refund(_charge: u32, asked: u32) -> u32 { asked }\n";
const REFUND_OK: &str = "pub fn refund(charge: u32, asked: u32) -> u32 { asked.min(charge) }\n";

const SPEC_HEAD: &str =
    "## ADDED Requirements\n\n### Requirement: Refunds\nThe system SHALL refund.\n";

const PAY_020: &str = "\n#### Scenario: PAY-020 A refund never exceeds the charge\n\
- **FOR ALL** charges and refund requests\n- **WHEN** a refund is made\n\
- **THEN** it never exceeds the charge\n";

/// A stand-in for the proptest crate: the scanner only needs the `proptest!` block.
const PROPTEST_MACRO: &str = "macro_rules! proptest { ($($t:tt)*) => { $($t)* }; }\n\n";

/// A property test that fails on base the way proptest reports a shrunk failure.
fn proptest_failure() -> String {
    format!(
        "{PROPTEST_MACRO}proptest! {{\n    #[test]\n    fn pay_020_refund_never_exceeds_charge() {{\n        \
         let (charge, refund) = (100, 101);\n        \
         if demo::refund(charge, refund) > charge {{\n            \
         panic!(\"Test failed: refund exceeds charge; minimal failing input: charge = {{charge}}, refund = {{refund}}\");\n        \
         }}\n    }}\n}}\n"
    )
}

/// A property test whose base failure is a bare assertion with no counterexample line.
fn plain_assert_failure() -> String {
    format!(
        "{PROPTEST_MACRO}proptest! {{\n    #[test]\n    fn pay_020_refund_never_exceeds_charge() {{\n        \
         let charge = 100;\n        let refund = demo::refund(charge, 101);\n        \
         assert!(refund <= charge);\n    }}\n}}\n"
    )
}

/// A FOR ALL PAY-020 change on a base that refunds whatever is asked, fixed at head.
fn pay_020_change(property_test: &str) -> Repo {
    let repo = Repo::with_lib(REFUND_BASE);
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &format!("{SPEC_HEAD}{PAY_020}"),
    );
    repo.write("src/lib.rs", REFUND_OK);
    repo.write("tests/pay_property.rs", property_test);
    repo.commit("feat(PAY-020): refunds");
    repo
}

fn gate(repo: &Repo, json: bool) -> (Option<i32>, String) {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg("gate")
        .arg("--root")
        .arg(repo.path())
        .args(["--base", "main"]);
    if json {
        c.arg("--json");
    }
    let out = c.output().unwrap();
    (out.status.code(), String::from_utf8(out.stdout).unwrap())
}

fn proven<'a>(report: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report["proven"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap_or_else(|| panic!("{id} not proven: {report:#}"))
}

#[test]
fn gate_068_proptest_counterexample_is_shown_in_text_and_json() {
    let repo = pay_020_change(&proptest_failure());

    let (code, stdout) = gate(&repo, false);
    assert!(
        stdout.contains(
            "info    counterexample PAY-020 pay_020_refund_never_exceeds_charge: \
             charge = 100, refund = 101\n"
        ),
        "{stdout}"
    );
    assert_eq!(code, Some(0), "{stdout}");

    let (_, json) = gate(&repo, true);
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        proven(&report, "PAY-020")["counterexample"],
        serde_json::json!({
            "test": "pay_020_refund_never_exceeds_charge",
            "input": "charge = 100, refund = 101"
        }),
        "{report:#}"
    );
}

/// A control beside a scenario that must show no counterexample: FOR ALL PAY-023, whose
/// property test prints a readable one on base, so an absence isn't just a missing feature.
const PAY_023: &str = "\n#### Scenario: PAY-023 A partial refund never exceeds the charge\n\
- **FOR ALL** charges and partial refund requests\n- **WHEN** a partial refund is made\n\
- **THEN** it never exceeds the charge\n";

fn pay_023_test() -> String {
    proptest_failure().replace(
        "pay_020_refund_never_exceeds_charge",
        "pay_023_partial_refund_never_exceeds_charge",
    )
}

const PAY_023_INFO: &str =
    "info    counterexample PAY-023 pay_023_partial_refund_never_exceeds_charge: \
charge = 100, refund = 101\n";

#[test]
fn gate_071_property_test_with_no_readable_counterexample_still_proves_its_scenario() {
    let repo = Repo::with_lib(REFUND_BASE);
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &format!("{SPEC_HEAD}{PAY_020}{PAY_023}"),
    );
    repo.write("src/lib.rs", REFUND_OK);
    repo.write("tests/pay_property.rs", &plain_assert_failure());
    repo.write("tests/pay_partial.rs", &pay_023_test());
    repo.commit("feat(PAY-020, PAY-023): refunds");

    let (code, stdout) = gate(&repo, false);
    assert_eq!(code, Some(0), "{stdout}");
    assert!(stdout.contains(PAY_023_INFO), "{stdout}");
    assert!(
        !stdout.contains("info    counterexample PAY-020"),
        "{stdout}"
    );

    let (_, json) = gate(&repo, true);
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    let pay_020 = proven(&report, "PAY-020");
    assert_eq!(pay_020["red"], "red:assert", "{report:#}");
    assert!(pay_020.get("counterexample").is_none(), "{report:#}");
}

const PAY_021_022: &str = "\n#### Scenario: PAY-021 A refund over the charge refunds the charge\n\
- **WHEN** 150 is asked back on a charge of 100\n- **THEN** the refund is 100\n\
\n#### Scenario: PAY-022 A capped refund never exceeds the cap\n\
- **FOR ALL** charges and refund requests\n- **WHEN** a capped refund is made\n\
- **THEN** it never exceeds the charge\n";

/// An example test whose base failure text happens to read like a fast-check counterexample.
const PAY_021_TEST: &str = "#[test]\nfn pay_021_refund_of_150_on_100() {\n    \
assert_eq!(demo::refund(100, 150), 100, \"Counterexample: [7]\");\n}\n";

/// A property test that calls a function base doesn't have, so it fails to compile there.
fn pay_022_test() -> String {
    format!(
        "{PROPTEST_MACRO}proptest! {{\n    #[test]\n    fn pay_022_capped_refund_never_exceeds_charge() {{\n        \
         assert!(demo::refund_capped(100, 101) <= 100); // minimal failing input: charge = 100\n    }}\n}}\n"
    )
}

#[test]
fn gate_072_only_a_property_test_failing_its_assertion_on_base_has_a_counterexample() {
    let repo = Repo::with_lib(REFUND_BASE);
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &format!("{SPEC_HEAD}{PAY_021_022}{PAY_023}"),
    );
    repo.write(
        "src/lib.rs",
        &format!(
            "{REFUND_OK}pub fn refund_capped(charge: u32, asked: u32) -> u32 {{ asked.min(charge) }}\n"
        ),
    );
    repo.write("tests/pay.rs", PAY_021_TEST);
    repo.write("tests/pay_capped.rs", &pay_022_test());
    repo.write("tests/pay_partial.rs", &pay_023_test());
    repo.commit("feat(PAY-021, PAY-022, PAY-023): refunds");

    let (code, stdout) = gate(&repo, false);
    assert!(stdout.contains(PAY_023_INFO), "{stdout}");
    for id in ["PAY-021", "PAY-022"] {
        assert!(
            !stdout.contains(&format!("info    counterexample {id}")),
            "{stdout}"
        );
    }
    assert_eq!(code, Some(0), "{stdout}");

    let (_, json) = gate(&repo, true);
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        proven(&report, "PAY-021")["red"],
        "red:assert",
        "{report:#}"
    );
    assert_eq!(
        proven(&report, "PAY-022")["red"],
        "red:compile",
        "{report:#}"
    );
    for id in ["PAY-021", "PAY-022"] {
        assert!(
            proven(&report, id).get("counterexample").is_none(),
            "{report:#}"
        );
    }
}
