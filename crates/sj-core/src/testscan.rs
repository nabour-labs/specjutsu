//! Rust test discovery: test functions and their `// spec: ID` comments.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

/// A test function found in source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestFn {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    /// IDs named in `// spec: ID` comments directly above the test.
    pub comment_ids: Vec<String>,
}

/// Find test functions (`fn` items carrying a `#[...test...]` attribute) in one Rust file.
pub fn scan_rust(_file: &Path, _content: &str) -> Vec<TestFn> {
    todo!("TRACE-004")
}

/// Walk `root` for `*.rs` files (respecting .gitignore, skipping `target/` and `fixtures/`) and scan each.
pub fn scan_tree(_root: &Path) -> Result<Vec<TestFn>> {
    todo!("TRACE-004")
}

/// Candidate IDs embedded in a test name: an alphabetic segment followed by a numeric segment,
/// e.g. `test_auth_003_x` → `AUTH-003`.
pub fn ids_in_name(_name: &str) -> Vec<String> {
    todo!("TRACE-004")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_src() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/trace-basic/src/lib.rs")
    }

    #[test]
    fn trace_004_match_ids_in_rust_test_names_and_spec_comments() {
        let path = fixture_src();
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
}
