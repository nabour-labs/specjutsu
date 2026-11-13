//! TRACE scenarios (openspec/changes/trace-engine).

use std::path::{Path, PathBuf};

use sj_core::spec::{discover_spec_files, load_scenarios, parse_scenarios, Scenario};
use sj_core::testscan::{ids_in_name, scan_rust, scan_tree};
use sj_core::trace::build_report;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/trace-basic")
}

#[test]
fn trace_001_parse_scenario_ids_from_headings() {
    let src = "### Requirement: R\n\n#### Scenario: AUTH-003 Reject expired token\n- **WHEN** x\n";
    let got = parse_scenarios(Path::new("spec.md"), src);
    assert_eq!(
        got,
        vec![Scenario {
            id: Some("AUTH-003".into()),
            name: "Reject expired token".into(),
            file: PathBuf::from("spec.md"),
            line: 3,
        }]
    );
}

#[test]
fn trace_001_skips_headings_inside_fenced_code() {
    let src = "```md\n#### Scenario: AUTH-777 Fenced\n```\n#### Scenario: AUTH-001 Real\n";
    let got = parse_scenarios(Path::new("spec.md"), src);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id.as_deref(), Some("AUTH-001"));
    assert_eq!(got[0].line, 4);
}

#[test]
fn trace_002_headings_without_an_id_are_reported() {
    let src = "#### Scenario: Reject expired token\n";
    let got = parse_scenarios(Path::new("spec.md"), src);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, None);
    assert_eq!(got[0].name, "Reject expired token");
    assert_eq!(got[0].line, 1);
}

#[test]
fn trace_003_discover_current_and_change_specs_skip_archive() {
    let root = fixture().join("openspec");
    let mut files: Vec<String> = discover_spec_files(&root)
        .unwrap()
        .iter()
        .map(|p| {
            p.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    files.sort();
    assert_eq!(
        files,
        vec![
            "changes/add-billing/specs/billing/spec.md",
            "specs/auth/spec.md"
        ]
    );
}

#[test]
fn trace_004_match_ids_in_rust_test_names_and_spec_comments() {
    let path = fixture().join("src/lib.rs");
    let content = std::fs::read_to_string(&path).unwrap();
    let tests = scan_rust(&path, &content);
    let names: Vec<&str> = tests.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "auth_001_logs_in",
            "charges_saved_card",
            "auth_009_points_at_nothing",
            "unrelated_v_2_test"
        ]
    );
    assert_eq!(tests[0].line, 6);
    assert_eq!(tests[1].comment_ids, vec!["BILL-001".to_string()]);
    assert!(tests[0].comment_ids.is_empty());
}

#[test]
fn trace_004_ids_in_name_are_normalised() {
    assert_eq!(ids_in_name("test_AUTH_003_rejects"), vec!["AUTH-003"]);
    assert_eq!(ids_in_name("trace_001_parses"), vec!["TRACE-001"]);
    assert!(ids_in_name("trace_0012x").is_empty());
    assert!(ids_in_name("plain_name").is_empty());
}

#[test]
fn trace_005_report_mapped_gaps_and_orphans() {
    let root = fixture();
    let scenarios = load_scenarios(&root.join("openspec")).unwrap();
    let tests = scan_tree(&root).unwrap();
    let r = build_report(&scenarios, &tests);

    let mapped: Vec<&str> = r.mapped.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(mapped, vec!["AUTH-001", "BILL-001"]);
    assert_eq!(r.mapped[0].tests[0].name, "auth_001_logs_in");
    let gaps: Vec<_> = r.gaps.iter().map(|s| s.id.clone().unwrap()).collect();
    assert_eq!(gaps, vec!["AUTH-002"]);
    let orphans: Vec<&str> = r.orphans.iter().map(|o| o.id.as_str()).collect();
    assert_eq!(orphans, vec!["AUTH-009"]);
    assert_eq!(r.unidentified.len(), 1);
    assert!(!r.is_clean());
}
