//! TRACE-025 and TRACE-026 from specjutsu/changes/property-scenarios: fast-check tests are
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
runner = "vitest"
tests = ["tests/**/*.test.ts"]
"#;

const SPEC: &str = "## Requirements\n\n### Requirement: Pay\nThe system SHALL refund.\n\n\
     #### Scenario: PAY-020 Refund never exceeds charge\n- **WHEN** x\n- **THEN** y\n";

/// `sj trace --json` on a project with `tests/pay.test.ts`, returning `(id, name, property)`
/// per linked test.
fn trace(test_file: &str) -> Vec<(String, String, serde_json::Value)> {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    write(r, "specjutsu.toml", CONFIG);
    write(r, "specjutsu/specs/pay/spec.md", SPEC);
    write(r, "tests/pay.test.ts", test_file);
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
fn trace_025_a_fast_check_test_is_a_property_test() {
    let got = trace(
        "import fc from 'fast-check'\n\
         import { it } from 'vitest'\n\n\
         const check = (a: number, b: number) => Math.min(a, b) <= b\n\n\
         it('PAY-020 refund never exceeds the charge', () => { fc.assert(fc.property(fc.nat(), fc.nat(), check)) })\n",
    );
    assert_eq!(
        got,
        vec![(
            "PAY-020".into(),
            "PAY-020 refund never exceeds the charge".into(),
            true.into()
        )]
    );
}

// spec: TRACE-025
#[test]
fn a_default_import_beside_named_imports_binds_fast_check() {
    let got = trace(
        "import fc, { nat } from 'fast-check'\n\
         import { it } from 'vitest'\n\n\
         const check = (a: number, b: number) => Math.min(a, b) <= b\n\n\
         it('PAY-020 refund never exceeds the charge', () => { fc.assert(fc.property(nat(), nat(), check)) })\n",
    );
    assert_eq!(
        got,
        vec![(
            "PAY-020".into(),
            "PAY-020 refund never exceeds the charge".into(),
            true.into()
        )]
    );
}

// spec: TRACE-025
#[test]
fn a_typescript_import_require_binds_fast_check() {
    let got = trace(
        "import fc = require('fast-check')\n\
         import { it } from 'vitest'\n\n\
         const check = (a: number, b: number) => Math.min(a, b) <= b\n\n\
         it('PAY-020 refund never exceeds the charge', () => { fc.assert(fc.property(fc.nat(), fc.nat(), check)) })\n",
    );
    assert_eq!(
        got,
        vec![(
            "PAY-020".into(),
            "PAY-020 refund never exceeds the charge".into(),
            true.into()
        )]
    );
}

#[test]
fn trace_026_a_property_call_through_another_name_is_not_a_property_test() {
    let got = trace(
        "import { it } from 'vitest'\n\
         import * as helpers from './helpers'\n\n\
         const check = (a: number, b: number) => Math.min(a, b) <= b\n\n\
         it('PAY-020 refund never exceeds the charge', () => { helpers.property(check) })\n",
    );
    assert_eq!(
        got,
        vec![(
            "PAY-020".into(),
            "PAY-020 refund never exceeds the charge".into(),
            false.into()
        )]
    );
}
