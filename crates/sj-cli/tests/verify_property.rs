//! LOOP-038 and LOOP-039 from specjutsu/changes/property-scenarios: `sj verify` and
//! `sj task done` on FOR ALL scenarios.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use support::Repo;

const REFUND_BUG: &str = "pub fn refund(paid: u32, asked: u32) -> u32 { paid - asked - 1 }\n";
const REFUND_OK: &str = "pub fn refund(paid: u32, asked: u32) -> u32 { asked.min(paid) }\n";
const REFUND_TEST: &str =
    "#[test]\nfn pay_020_refund_of_50_on_100() { assert_eq!(demo::refund(100, 50), 50); }\n";
const FOR_ALL_DELTA: &str = "## ADDED Requirements\n\n### Requirement: Refunds\nThe system SHALL refund.\n\n\
#### Scenario: PAY-020 A refund never exceeds the payment\n\
- **FOR ALL** payments and refund requests\n- **WHEN** a refund is made\n- **THEN** it never exceeds the payment\n";

fn sj(repo: &Repo, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.args(args).arg("--root").arg(repo.path());
    c
}

/// `specjutsu.toml` in `mode`, with the cargo suite it then has to list.
fn set_mode(repo: &Repo, mode: &str) {
    repo.write(
        "specjutsu.toml",
        &format!(
            "schema = 1\nmode = \"{mode}\"\n\n[[package]]\npath = \".\"\n\n\
             [[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**/*.rs\"]\n"
        ),
    );
    repo.commit("chore: mode");
}

/// A stand-in for the proptest crate: the scanner only needs the `proptest!` block.
const PROPERTY_TEST: &str = "macro_rules! proptest { ($($t:tt)*) => { $($t)* }; }\n\n\
proptest! {\n    #[test]\n    fn pay_020_refund_never_exceeds_payment() {\n        \
for (paid, asked) in [(100, 50), (100, 150), (0, 1)] {\n            \
assert!(demo::refund(paid, asked) <= paid);\n        }\n    }\n}\n";

/// Change `refunds`: task 1.1 names FOR ALL scenario PAY-020, whose only test is an example
/// test the ledger shows failing, then passing.
fn refunds_proven_by_an_example_test(mode: Option<&str>) -> Repo {
    refunds_proven(mode, false)
}

/// As [`refunds_proven_by_an_example_test`], plus a property test when `property`.
fn refunds_proven(mode: Option<&str>, property: bool) -> Repo {
    let repo = Repo::with_lib(REFUND_BUG);
    if let Some(mode) = mode {
        set_mode(&repo, mode);
    }
    if property {
        repo.write("tests/pay_property.rs", PROPERTY_TEST);
    }
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write("openspec/changes/refunds/specs/pay/spec.md", FOR_ALL_DELTA);
    repo.write(
        "openspec/changes/refunds/tasks.md",
        "- [ ] 1.1 Refunds (PAY-020)\n",
    );
    repo.write("tests/pay.rs", REFUND_TEST);
    repo.commit("test(PAY-020): refunds");
    // The ledger records the example test failing, then passing on the fix.
    sj(&repo, &["test", "PAY-020"]).assert().code(1);
    repo.write("src/lib.rs", REFUND_OK);
    repo.commit("feat(PAY-020): refunds");
    sj(&repo, &["test", "PAY-020"]).assert().success();
    repo
}

const NOT_PROPERTY: &str = "not-property PAY-020 pay_020_refund_of_50_on_100 \
    (a FOR ALL scenario needs a property test; write one with proptest)";

#[test]
fn loop_038_verify_reports_a_for_all_scenario_with_only_example_tests() {
    let repo = refunds_proven_by_an_example_test(None);
    for args in [&["verify"][..], &["verify", "refunds"]] {
        let out = sj(&repo, args).output().unwrap();
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(
            stdout.lines().any(|l| l == NOT_PROPERTY),
            "{args:?}: {stdout}"
        );
        assert_eq!(out.status.code(), Some(1), "{args:?}: {stdout}");
    }
}

// spec: LOOP-039
#[test]
fn loop_039_strict_task_done_refuses_a_for_all_scenario_with_only_example_tests() {
    let repo = refunds_proven_by_an_example_test(Some("strict"));
    let tasks = repo.path().join("openspec/changes/refunds/tasks.md");
    let before = std::fs::read(&tasks).unwrap();

    let out = sj(&repo, &["task", "done", "refunds", "1.1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.lines().any(|l| l == NOT_PROPERTY), "{stdout}");
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert_eq!(std::fs::read(&tasks).unwrap(), before);
}

// spec: LOOP-039
#[test]
fn loop_039_guided_task_done_checks_the_box_and_warns_of_a_for_all_scenario() {
    let repo = refunds_proven_by_an_example_test(Some("guided"));
    let tasks = repo.path().join("openspec/changes/refunds/tasks.md");

    let out = sj(&repo, &["task", "done", "refunds", "1.1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout
            .lines()
            .any(|l| l == format!("warn    {NOT_PROPERTY}")),
        "{stdout}"
    );
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert_eq!(
        std::fs::read_to_string(&tasks).unwrap(),
        "- [x] 1.1 Refunds (PAY-020)\n"
    );
}

const PAY_020: &str = "#### Scenario: PAY-020 A refund never exceeds the payment\n\
- **FOR ALL** payments and refund requests\n- **WHEN** a refund is made\n- **THEN** it never exceeds the payment\n";

fn refunds(pay_021_then: &str) -> String {
    format!(
        "### Requirement: Refunds\nThe system SHALL refund.\n\n{PAY_020}\n\
         #### Scenario: PAY-021 A refund is recorded\n- **WHEN** a refund is made\n- **THEN** {pay_021_then}\n"
    )
}

// spec: LOOP-040
#[test]
fn loop_040_verify_leaves_an_unchanged_for_all_scenario_alone() {
    let repo = Repo::with_lib(REFUND_OK);
    repo.write(
        "openspec/specs/pay/spec.md",
        &format!("## Requirements\n\n{}", refunds("it is logged")),
    );
    repo.write(
        "tests/pay.rs",
        &format!("{REFUND_TEST}\n#[test]\nfn pay_021_refund_is_recorded() {{}}\n"),
    );
    repo.commit("spec: refunds");
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &format!(
            "## MODIFIED Requirements\n\n{}",
            refunds("it is logged with its reason")
        ),
    );
    repo.write(
        "openspec/changes/refunds/tasks.md",
        "- [ ] 1.1 Refunds (PAY-020, PAY-021)\n",
    );
    repo.commit("spec(PAY-021): log the reason");

    let out = sj(&repo, &["verify"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("not-property"), "{stdout}");
    assert!(stdout.contains("unproven PAY-021"), "{stdout}");
}

// spec: LOOP-041
#[test]
fn loop_041_strict_task_done_accepts_a_for_all_scenario_with_a_property_test() {
    let repo = refunds_proven(Some("strict"), true);
    let tasks = repo.path().join("openspec/changes/refunds/tasks.md");

    let out = sj(&repo, &["task", "done", "refunds", "1.1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert!(!stdout.contains("not-property"), "{stdout}");
    assert_eq!(
        std::fs::read_to_string(&tasks).unwrap(),
        "- [x] 1.1 Refunds (PAY-020)\n"
    );
}

// spec: LOOP-042
#[test]
fn loop_042_strict_task_done_accepts_an_unchanged_for_all_scenario() {
    let repo = Repo::with_lib(REFUND_OK);
    set_mode(&repo, "strict");
    repo.write(
        "openspec/specs/pay/spec.md",
        &format!("## Requirements\n\n{}", refunds("it is logged")),
    );
    repo.write("tests/pay.rs", REFUND_TEST);
    repo.commit("spec: refunds");
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &format!(
            "## MODIFIED Requirements\n\n{}",
            refunds("it is logged with its reason")
        ),
    );
    repo.write(
        "openspec/changes/refunds/tasks.md",
        "- [ ] 1.1 Refunds (PAY-020)\n",
    );
    repo.commit("spec(PAY-021): log the reason");
    sj(&repo, &["test", "PAY-020"]).assert().success();
    let tasks = repo.path().join("openspec/changes/refunds/tasks.md");

    let out = sj(&repo, &["task", "done", "refunds", "1.1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert!(!stdout.contains("not-property"), "{stdout}");
    assert_eq!(
        std::fs::read_to_string(&tasks).unwrap(),
        "- [x] 1.1 Refunds (PAY-020)\n"
    );
}
