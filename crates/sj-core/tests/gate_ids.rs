//! GATE scenarios from specjutsu/changes/scenario-ids: assigning an ID is not a change.

mod support;

use std::path::PathBuf;

use sj_core::gate::{run, Change, Options};
use support::Repo;

fn opts(repo: &Repo) -> Options {
    Options {
        root: repo.path().to_path_buf(),
        spec_root: PathBuf::from("openspec"),
        base: "main".into(),
        allow_unverifiable: false,
    }
}

const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const ADD_TEST: &str = "#[test]\nfn auth_001_valid_login() { assert_eq!(demo::add(2, 2), 4); }\n";

/// A spec file of `n` scenarios without IDs, named `Scenario <i>`.
fn many(n: usize) -> String {
    let mut s =
        String::from("## Requirements\n\n### Requirement: Auth\nThe system SHALL auth.\n\n");
    for i in 1..=n {
        s.push_str(&format!(
            "#### Scenario: Scenario {i}\n- **WHEN** case {i}\n- **THEN** it works\n\n"
        ));
    }
    s
}

/// `many(n)` with each heading given `AUTH-<i>`.
fn many_with_ids(n: usize) -> String {
    let mut s = many(n);
    for i in (1..=n).rev() {
        s = s.replace(
            &format!("#### Scenario: Scenario {i}\n"),
            &format!("#### Scenario: AUTH-{i:03} Scenario {i}\n"),
        );
    }
    s
}

#[test]
fn gate_036_an_id_only_commit_passes_the_gate() {
    let repo = Repo::with_lib(ADD_OK);
    repo.write("openspec/specs/auth/spec.md", &many(300));
    repo.commit("docs(spec): auth");
    repo.branch("ids");
    repo.write("openspec/specs/auth/spec.md", &many_with_ids(300));
    repo.commit("chore(spec): assign scenario IDs");
    let r = run(&opts(&repo)).unwrap();
    assert!(r.pass, "{:?}", r.failures);
    assert!(r.proven.is_empty(), "{:?}", r.proven);
    assert!(r.changes.is_empty(), "{:?}", r.changes);
    let gaps = r
        .warnings
        .iter()
        .filter(|w| w.starts_with("gap AUTH-"))
        .count();
    assert_eq!(gaps, 300, "{:?}", &r.warnings[..5.min(r.warnings.len())]);
    assert!(
        !r.warnings.iter().any(|w| w.contains("without ID")),
        "{:?}",
        r.warnings
    );
}

#[test]
fn gate_036_rewording_while_adding_an_id_is_still_added() {
    let repo = Repo::with_lib(ADD_OK);
    repo.write("openspec/specs/auth/spec.md", &many(2));
    repo.commit("docs(spec): auth");
    repo.branch("ids");
    repo.write(
        "openspec/specs/auth/spec.md",
        &many_with_ids(2).replace("case 2", "case two"),
    );
    repo.commit("chore(spec): ids and a reword");
    let r = run(&opts(&repo)).unwrap();
    assert!(!r.pass);
    assert_eq!(r.changes.get("AUTH-002"), Some(&Change::Added));
    assert_eq!(r.changes.get("AUTH-001"), None);
    assert!(
        r.failures.iter().any(|f| f.contains("AUTH-002")),
        "{:?}",
        r.failures
    );
    assert!(
        !r.failures.iter().any(|f| f.contains("AUTH-001")),
        "{:?}",
        r.failures
    );
}

#[test]
fn gate_037_adding_an_id_and_a_test_at_once_is_still_gated() {
    let base = "## Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
                #### Scenario: Valid login\n- **WHEN** credentials match\n- **THEN** a session starts\n";
    // The test passes on base too, so it links AUTH-001 to existing behaviour; it can't be
    // what drove the edit to `src/auth.rs`.
    let repo = Repo::with_lib(ADD_OK);
    repo.write("openspec/specs/auth/spec.md", base);
    repo.write("src/auth.rs", "pub fn login() -> bool { true }\n");
    repo.commit("docs(spec): auth");
    repo.branch("ids");
    repo.write(
        "openspec/specs/auth/spec.md",
        &base.replace("Scenario: Valid", "Scenario: AUTH-001 Valid"),
    );
    repo.write("tests/auth.rs", ADD_TEST);
    repo.write("src/auth.rs", "pub fn login() -> bool { false }\n");
    repo.commit("test(AUTH-001): red");
    let r = run(&opts(&repo)).unwrap();
    assert_eq!(r.changes.get("AUTH-001"), Some(&Change::Implemented));
    assert_eq!(
        serde_json::to_value(&r.proven).unwrap(),
        serde_json::json!([{ "id": "AUTH-001", "red": "green:linked", "property": false }]),
        "{:?}",
        r.failures
    );
    assert!(!r.pass);
    assert_eq!(r.failures, vec!["unspecified change: src/auth.rs"]);
}
