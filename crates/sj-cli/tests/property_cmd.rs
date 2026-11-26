//! Property scenarios (specjutsu/changes/property-scenarios): `- **FOR ALL**` steps.

use std::path::Path;

use assert_cmd::Command;

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

const CONFIG: &str = "schema = 1\n\n[[package]]\npath = \".\"\nsources = []\n\n\
                      [[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**/*.rs\"]\n";

const PAY_DELTA: &str = "## ADDED Requirements\n\n### Requirement: Refund limit\n\
The system SHALL never refund more than the charge.\n\n\
#### Scenario: PAY-020 Refund never exceeds the charge\n\
- **FOR ALL** charges and refund requests\n\
- **WHEN** a refund is issued\n\
- **THEN** the refunded total is at most the charge\n\n\
#### Scenario: PAY-021 Refund is logged\n\
```md\n- **FOR ALL** refunds\n```\n\
- **WHEN** a refund is issued\n\
- **THEN** the refund is logged\n\n\
#### Scenario: PAY-022 Partial refund is capped\n\
* **For  all** partial refunds\n\
- **WHEN** a partial refund is issued\n\
- **THEN** it is capped\n";

// spec: SPEC-072
#[test]
fn spec_072_a_for_all_step_is_parsed() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(r, "specjutsu.toml", CONFIG);
    put(
        r,
        "openspec/changes/refund-limit/specs/payments/spec.md",
        PAY_DELTA,
    );
    let show = |id: &str| -> serde_json::Value {
        let out = Command::cargo_bin("sj")
            .unwrap()
            .args(["spec", "show", id, "--json"])
            .arg("--root")
            .arg(r)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    };
    let v = show("PAY-020");
    assert_eq!(v["property"], true, "{v}");
    assert_eq!(
        v["steps"][0], "- **FOR ALL** charges and refund requests",
        "{v}"
    );
    // FOR ALL inside fenced code does not count, and does not leak from PAY-020.
    assert_eq!(show("PAY-021")["property"], false);
    // Any bullet, case and spacing the step grammar accepts.
    assert_eq!(show("PAY-022")["property"], true);
}

// spec: SPEC-073
#[test]
fn spec_073_a_for_all_step_after_when_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    put(r, "specjutsu.toml", CONFIG);
    put(
        r,
        "openspec/changes/refund-log/specs/payments/spec.md",
        "## ADDED Requirements\n\n### Requirement: Refund log\n\
         The system SHALL log every refund.\n\n\
         #### Scenario: PAY-021 Refund is logged\n\
         - **WHEN** a refund is issued\n\
         - **FOR ALL** charges\n\
         - **THEN** the refund is logged\n",
    );
    put(
        r,
        "openspec/changes/refund-log/tasks.md",
        "# Tasks\n\n- [x] 1.1 Log refunds (PAY-021)\n",
    );
    Command::cargo_bin("sj")
        .unwrap()
        .args(["spec", "validate", "refund-log"])
        .arg("--root")
        .arg(r)
        .assert()
        .code(1)
        .stdout(
            "validate: 1 error, 0 warnings\n\
             fail    PAY-021 FOR ALL step must come before WHEN \
             (openspec/changes/refund-log/specs/payments/spec.md:8)\n",
        );
}
