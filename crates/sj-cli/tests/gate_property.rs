//! GATE-063..067 from specjutsu/changes/property-scenarios: FOR ALL scenarios and property tests.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

const REFUND_BUG: &str = "pub fn refund(paid: u32, asked: u32) -> u32 { paid - asked - 1 }\n";
const REFUND_OK: &str = "pub fn refund(paid: u32, asked: u32) -> u32 { asked.min(paid) }\n";
const REFUND_TEST: &str =
    "#[test]\nfn pay_020_refund_of_50_on_100() { assert_eq!(demo::refund(100, 50), 50); }\n";

fn scenario(kind: &str, steps: &str) -> String {
    format!(
        "## {kind} Requirements\n\n### Requirement: Refunds\nThe system SHALL refund.\n\n\
         #### Scenario: PAY-020 A refund never exceeds the payment\n{steps}\n"
    )
}

const FOR_ALL: &str = "- **FOR ALL** payments and refund requests\n- **WHEN** a refund is made\n- **THEN** it never exceeds the payment\n";
const EXAMPLE: &str = "- **WHEN** a refund is made\n- **THEN** it never exceeds the payment\n";

fn sj(repo: &Repo) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg("gate").arg("--root").arg(repo.path());
    c
}

/// The gate fails PAY-020 as not-property with the proptest hint. A macro, so the audit counts
/// the assertion in each test that uses it.
macro_rules! assert_not_property {
    ($repo:expr) => {
        sj($repo)
            .args(["--base", "main"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains(
                "fail    not-property PAY-020 pay_020_refund_of_50_on_100 \
                 (a FOR ALL scenario needs a property test; write one with proptest)\n",
            ))
    };
}

#[test]
fn gate_063_for_all_scenario_with_only_example_tests_fails() {
    let repo = Repo::with_lib(REFUND_BUG);
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &scenario("ADDED", FOR_ALL),
    );
    repo.write("src/lib.rs", REFUND_OK);
    repo.write("tests/pay.rs", REFUND_TEST);
    repo.commit("feat(PAY-020): refunds");
    assert_not_property!(&repo);
}

// spec: GATE-063
#[test]
fn gate_063_modified_scenario_gaining_for_all_needs_a_property_test() {
    let repo = Repo::with_lib(REFUND_BUG);
    repo.write(
        "openspec/specs/pay/spec.md",
        &scenario("ADDED", EXAMPLE).replace("## ADDED Requirements\n\n", ""),
    );
    repo.commit("spec: refunds");
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &scenario("MODIFIED", FOR_ALL),
    );
    repo.write("src/lib.rs", REFUND_OK);
    repo.write("tests/pay.rs", REFUND_TEST);
    repo.commit("feat(PAY-020): refunds for all payments");
    assert_not_property!(&repo);
}

/// A stand-in for the proptest crate: the scanner only needs the `proptest!` block.
const PROPERTY_TEST: &str = "macro_rules! proptest { ($($t:tt)*) => { $($t)* }; }\n\n\
proptest! {\n    #[test]\n    fn pay_020_refund_never_exceeds_payment() {\n        \
for (paid, asked) in [(100, 50), (100, 150), (0, 1)] {\n            \
assert!(demo::refund(paid, asked) <= paid);\n        }\n    }\n}\n";

/// A FOR ALL PAY-020 change whose base refunds whatever is asked, with `head` as the fix.
fn mixed_for_all(head: &str) -> Repo {
    let repo = Repo::with_lib("pub fn refund(_paid: u32, asked: u32) -> u32 { asked }\n");
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &scenario("ADDED", FOR_ALL),
    );
    repo.write("src/lib.rs", head);
    repo.write("tests/pay.rs", REFUND_TEST);
    repo.write("tests/pay_property.rs", PROPERTY_TEST);
    repo.commit("feat(PAY-020): refunds");
    repo
}

#[test]
fn gate_066_mixed_for_all_scenario_is_proven_by_its_property_test() {
    let repo = mixed_for_all(REFUND_OK);
    let out = sj(&repo).args(["--base", "main"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("gate: pass — 1 scenario proven (1 red:assert; 1 property)\n"),
        "{stdout}"
    );
    assert!(
        !stdout
            .lines()
            .any(|l| l.contains("green-on-base") && l.contains("pay_020_refund_of_50_on_100")),
        "{stdout}"
    );
    assert_eq!(out.status.code(), Some(0), "{stdout}");
}

#[test]
fn gate_067_example_test_of_a_for_all_scenario_must_pass_at_head() {
    let repo =
        mixed_for_all("pub fn refund(paid: u32, asked: u32) -> u32 { asked.min(paid) / 2 }\n");
    sj(&repo)
        .args(["--base", "main"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "failing PAY-020 pay_020_refund_of_50_on_100\n",
        ));
}

const PAY_021: &str = "\n#### Scenario: PAY-021 A refund over the payment refunds the payment\n\
- **WHEN** 150 is asked back on a payment of 100\n- **THEN** the refund is 100\n";
const PAY_021_TEST: &str =
    "#[test]\nfn pay_021_refund_of_150_on_100() { assert_eq!(demo::refund(100, 150), 100); }\n";

#[test]
fn gate_064_property_scenario_red_on_base_is_counted_as_property() {
    let repo = Repo::with_lib("pub fn refund(_paid: u32, asked: u32) -> u32 { asked }\n");
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &(scenario("ADDED", FOR_ALL) + PAY_021),
    );
    repo.write("src/lib.rs", REFUND_OK);
    repo.write("tests/pay_property.rs", PROPERTY_TEST);
    repo.write("tests/pay.rs", PAY_021_TEST);
    repo.commit("feat(PAY-020, PAY-021): refunds");

    sj(&repo)
        .args(["--base", "main"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "gate: pass — 2 scenarios proven (2 red:assert; 1 property)\n",
        ));

    let out = sj(&repo)
        .args(["--base", "main", "--json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let property = |id: &str| {
        report["proven"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap_or_else(|| panic!("{id} not proven: {report:#}"))["property"]
            .clone()
    };
    assert_eq!(property("PAY-020"), true, "{report:#}");
    assert_eq!(property("PAY-021"), false, "{report:#}");
}

const PAY_022: &str = "## ADDED Requirements\n\n### Requirement: Refunds\nThe system SHALL refund.\n\n\
#### Scenario: PAY-022 [keep] Refund never exceeds the charge\n\
- **FOR ALL** charges and refund requests\n- **WHEN** a refund is made\n- **THEN** it never exceeds the charge\n";

#[test]
fn gate_065_kept_for_all_scenario_is_proven_by_its_property_test() {
    let repo = Repo::with_lib(REFUND_OK);
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write("openspec/changes/refunds/specs/pay/spec.md", PAY_022);
    repo.write(
        "tests/pay_property.rs",
        &PROPERTY_TEST.replace(
            "pay_020_refund_never_exceeds_payment",
            "pay_022_refund_never_exceeds_charge",
        ),
    );
    repo.commit("test(PAY-022): keep refunds within the charge");

    sj(&repo)
        .args(["--base", "main"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "gate: pass — 1 scenario proven (1 green:keep; 1 property)\n",
        ));

    let out = sj(&repo)
        .args(["--base", "main", "--json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let pay_022 = report["proven"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "PAY-022")
        .unwrap_or_else(|| panic!("PAY-022 not proven: {report:#}"));
    assert_eq!(pay_022["red"], "green:keep", "{report:#}");
    assert_eq!(pay_022["property"], true, "{report:#}");
}

// spec: GATE-065
#[test]
fn gate_065_kept_for_all_scenario_with_only_an_example_test_fails() {
    let repo = Repo::with_lib(REFUND_OK);
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write("openspec/changes/refunds/specs/pay/spec.md", PAY_022);
    repo.write(
        "tests/pay.rs",
        "#[test]\nfn pay_022_refund_of_50_on_100() { assert_eq!(demo::refund(100, 50), 50); }\n",
    );
    repo.commit("test(PAY-022): keep refunds within the charge");
    sj(&repo)
        .args(["--base", "main"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "fail    not-property PAY-022 pay_022_refund_of_50_on_100 \
             (a FOR ALL scenario needs a property test; write one with proptest)\n",
        ));
}
