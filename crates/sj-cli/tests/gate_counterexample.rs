//! GATE-068..072 and GATE-074 from specjutsu/changes/property-counterexamples: the
//! counterexample a property test printed on base, in the gate's text and JSON reports, for
//! Cargo suites and for pytest and vitest suites run by a fake runner with a canned report.

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

#[test]
fn gate_074_pr_comment_shows_the_counterexample_in_the_evidence_cell() {
    let repo = pay_020_change(&proptest_failure());

    let out = Command::cargo_bin("sj")
        .unwrap()
        .arg("gate")
        .arg("--root")
        .arg(repo.path())
        .args(["--base", "main", "--format", "markdown"])
        .output()
        .unwrap();
    let body = String::from_utf8(out.stdout).unwrap();
    let row = body
        .lines()
        .find(|l| l.starts_with("| PAY-020 |"))
        .unwrap_or_else(|| panic!("no PAY-020 row: {body}"));
    let evidence = row.split(" | ").nth(1).unwrap();
    assert_eq!(
        evidence, "red:assert (counterexample: charge = 100, refund = 101)",
        "{body}"
    );
}

/// A stand-in test runner (Unix only): it writes `fail.xml` from its own folder as the JUnit
/// report pytest (`--junitxml=`) or vitest (`--outputFile.junit=`) was asked for and fails,
/// unless the package's `src` holds the fix (`min`), where it writes `pass.xml` and passes.
#[cfg(unix)]
const FAKE_RUNNER: &str = r#"#!/bin/sh
here=$(dirname "$0")
out=""
for a in "$@"; do
  case "$a" in
    --junitxml=*) out="${a#--junitxml=}" ;;
    --outputFile.junit=*) out="${a#--outputFile.junit=}" ;;
  esac
done
mkdir -p "$(dirname "$out")"
if grep -rq min src 2>/dev/null; then cp "$here/pass.xml" "$out"; exit 0; fi
cp "$here/fail.xml" "$out"
exit 1
"#;

/// A folder holding the fake runner and its two canned reports.
#[cfg(unix)]
fn fake_runner(fail_xml: &str, pass_xml: &str) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let run = dir.path().join("run.sh");
    std::fs::write(&run, FAKE_RUNNER).unwrap();
    std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(dir.path().join("fail.xml"), fail_xml).unwrap();
    std::fs::write(dir.path().join("pass.xml"), pass_xml).unwrap();
    dir
}

/// A repository whose package `svc` has one `runner` suite over `tests` run by `bin`'s fake
/// runner, on `main` with `base_src`, switched to `feature`.
#[cfg(unix)]
fn suite_repo(runner: &str, tests: &str, bin: &tempfile::TempDir, base_src: (&str, &str)) -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    repo.write(
        "specjutsu.toml",
        &format!(
            "schema = 1\n\n[[package]]\npath = \"svc\"\nsources = [\"svc/src/**\"]\n\n\
             [[package.suite]]\nrunner = \"{runner}\"\ntests = [\"{tests}\"]\n\
             command = '{}'\n",
            bin.path().join("run.sh").display()
        ),
    );
    repo.write(base_src.0, base_src.1);
    repo.commit("base");
    repo.branch("feature");
    repo.write("openspec/changes/refunds/proposal.md", "## Why\n\nTest.\n");
    repo
}

#[cfg(unix)]
#[test]
fn gate_069_gate_reports_a_pytest_hypothesis_falsifying_example() {
    let fail = "<testsuites><testsuite name=\"pytest\">\
        <testcase classname=\"tests.test_pay\" name=\"test_pay_020_refund_never_exceeds_charge\">\
        <failure message=\"AssertionError: assert 101 &lt;= 100\">\
        def test_pay_020_refund_never_exceeds_charge(charge, refund):\n\
        &gt;       assert refund_of(charge, refund) &lt;= charge\n\
        E       AssertionError: assert 101 &lt;= 100\n\
        Falsifying example: test_pay_020_refund_never_exceeds_charge(\n    \
        charge=100,\n    refund=101,\n)\n\
        </failure></testcase></testsuite></testsuites>";
    let pass = "<testsuites><testsuite name=\"pytest\">\
        <testcase classname=\"tests.test_pay\" name=\"test_pay_020_refund_never_exceeds_charge\"/>\
        </testsuite></testsuites>";
    let bin = fake_runner(fail, pass);
    let repo = suite_repo(
        "pytest",
        "svc/tests/**",
        &bin,
        (
            "svc/src/pay.py",
            "def refund_of(charge, asked):\n    return asked\n",
        ),
    );
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &format!("{SPEC_HEAD}{PAY_020}"),
    );
    repo.write(
        "svc/src/pay.py",
        "def refund_of(charge, asked):\n    return min(charge, asked)\n",
    );
    repo.write(
        "svc/tests/test_pay.py",
        "from hypothesis import given, strategies as st\nfrom src.pay import refund_of\n\n\n\
         @given(st.integers(), st.integers())\n\
         def test_pay_020_refund_never_exceeds_charge(charge, refund):\n    \
         assert refund_of(charge, refund) <= charge\n",
    );
    repo.commit("feat(PAY-020): refunds");

    let (_, json) = gate(&repo, true);
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        proven(&report, "PAY-020")["counterexample"],
        serde_json::json!({
            "test": "test_pay_020_refund_never_exceeds_charge",
            "input": "charge=100, refund=101"
        }),
        "{report:#}"
    );
}

/// A vitest package `svc` with fast-check tests for PAY-020 (`src/pay.test.ts`) and PAY-022
/// (`src/capped.test.ts`) and an example test for PAY-021, with `fail` as the base report.
#[cfg(unix)]
fn vitest_repo(fail: &str) -> (Repo, tempfile::TempDir) {
    let pass = "<testsuites><testsuite name=\"vitest\">\
        <testcase classname=\"src/pay.test.ts\" name=\"PAY-020 refund never exceeds charge\"/>\
        <testcase classname=\"src/capped.test.ts\" name=\"PAY-022 capped refund never exceeds charge\"/>\
        <testcase classname=\"src/refund.test.ts\" name=\"PAY-021 refund of 150 on 100\"/>\
        </testsuite></testsuites>";
    let bin = fake_runner(fail, pass);
    let repo = suite_repo(
        "vitest",
        "svc/src/**/*.test.ts",
        &bin,
        ("svc/src/pay.ts", "export const refund = (c, a) => a\n"),
    );
    repo.write(
        "openspec/changes/refunds/specs/pay/spec.md",
        &format!("{SPEC_HEAD}{PAY_020}{PAY_021_022}"),
    );
    repo.write(
        "svc/src/pay.ts",
        "export const refund = (c, a) => Math.min(c, a)\n\
         export const capped = (c, a) => Math.min(c, a)\n",
    );
    let test = |id: &str, title: &str, f: &str| {
        format!(
            "import fc from 'fast-check'\nimport {{ {f} }} from './pay'\n\n\
             it(\"{id} {title}\", () => {{\n  \
             fc.assert(fc.property(fc.nat(), fc.nat(), (c, a) => {f}(c, a) <= c))\n}})\n"
        )
    };
    repo.write(
        "svc/src/pay.test.ts",
        &test("PAY-020", "refund never exceeds charge", "refund"),
    );
    repo.write(
        "svc/src/capped.test.ts",
        &test("PAY-022", "capped refund never exceeds charge", "capped"),
    );
    repo.write(
        "svc/src/refund.test.ts",
        "import { refund } from './pay'\n\n\
         it(\"PAY-021 refund of 150 on 100\", () => { expect(refund(100, 150)).toBe(100) })\n",
    );
    repo.commit("feat(PAY-020, PAY-021, PAY-022): refunds");
    (repo, bin)
}

#[cfg(unix)]
#[test]
fn gate_070_gate_reports_a_vitest_fast_check_counterexample() {
    let fail = "<testsuites><testsuite name=\"vitest\">\
        <testcase classname=\"src/pay.test.ts\" name=\"PAY-020 refund never exceeds charge\">\
        <failure message=\"Property failed after 4 tests\">\
        Error: Property failed after 4 tests\n\
        { seed: 42, path: \"3:0\", endOnFailure: true }\n\
        Counterexample: [100,101]\nShrunk 2 time(s)\n\
        </failure></testcase>\
        <testcase classname=\"src/capped.test.ts\" name=\"PAY-022 capped refund never exceeds charge\">\
        <failure message=\"Property failed\"/></testcase>\
        <testcase classname=\"src/refund.test.ts\" name=\"PAY-021 refund of 150 on 100\">\
        <failure message=\"expected 150 to be 100\"/></testcase>\
        </testsuite></testsuites>";
    let (repo, _bin) = vitest_repo(fail);

    let (_, json) = gate(&repo, true);
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        proven(&report, "PAY-020")["counterexample"],
        serde_json::json!({
            "test": "PAY-020 refund never exceeds charge",
            "input": "[100,101]"
        }),
        "{report:#}"
    );
}

#[cfg(unix)]
#[test]
fn gate_072_a_property_test_failing_to_load_on_base_has_no_counterexample() {
    // PAY-022's file fails to load on base with text that reads like a counterexample; PAY-020
    // fails its assertion with one, as a control.
    let fail = "<testsuites><testsuite name=\"vitest\">\
        <testcase classname=\"src/pay.test.ts\" name=\"PAY-020 refund never exceeds charge\">\
        <failure message=\"Property failed after 4 tests\">Counterexample: [100,101]</failure>\
        </testcase>\
        <testcase classname=\"src/capped.test.ts\" name=\"src/capped.test.ts\">\
        <failure message=\"SyntaxError: The requested module does not provide an export named capped. Counterexample: [1]\"/>\
        </testcase>\
        <testcase classname=\"src/refund.test.ts\" name=\"PAY-021 refund of 150 on 100\">\
        <failure message=\"expected 150 to be 100\"/></testcase>\
        </testsuite></testsuites>";
    let (repo, _bin) = vitest_repo(fail);

    let (_, json) = gate(&repo, true);
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        proven(&report, "PAY-020")["counterexample"]["input"],
        "[100,101]",
        "{report:#}"
    );
    let pay_022 = proven(&report, "PAY-022");
    assert_eq!(pay_022["red"], "red:compile", "{report:#}");
    assert!(pay_022.get("counterexample").is_none(), "{report:#}");
}
