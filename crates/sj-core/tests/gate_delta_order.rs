//! GATE-045 from openspec/changes/fix-gate-delta-order: an open change merged into the base
//! is not a change on branches made after it.

mod support;

use std::path::PathBuf;

use sj_core::gate::{run, Options};
use support::Repo;

const ADD_OK: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
const ADD_TEST: &str = "#[test]\nfn auth_001_valid_login() { assert_eq!(demo::add(2, 2), 4); }\n";

const SPEC: &str = "## Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
                    #### Scenario: AUTH-001 Valid login\n- **WHEN** credentials match\n- **THEN** a session starts\n";

const DELTA: &str = "## MODIFIED Requirements\n\n### Requirement: Login\nThe system SHALL log in.\n\n\
                     #### Scenario: AUTH-001 Valid login\n- **WHEN** credentials match\n- **THEN** a session starts and lasts 24 hours\n";

#[test]
fn gate_045_a_merged_open_change_delta_is_not_a_change_on_later_branches() {
    let repo = Repo::with_lib(ADD_OK);
    repo.write("openspec/specs/auth/spec.md", SPEC);
    repo.write("openspec/changes/rework/specs/auth/spec.md", DELTA);
    repo.write("tests/auth.rs", ADD_TEST);
    repo.commit("feat(AUTH-001): green");
    repo.branch("docs");
    repo.write("README.md", "# demo\n");
    repo.commit("docs: readme");

    let r = run(&Options {
        root: repo.path().to_path_buf(),
        spec_root: PathBuf::from("openspec"),
        base: "main".into(),
        allow_unverifiable: false,
    })
    .unwrap();
    assert_eq!(r.changes.get("AUTH-001"), None, "{:?}", r.changes);
    assert!(
        !r.failures.iter().any(|f| f.contains("AUTH-001")),
        "{:?}",
        r.failures
    );
    assert!(r.pass, "{:?}", r.failures);
}
