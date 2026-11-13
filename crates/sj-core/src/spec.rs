//! Scenario extraction from OpenSpec-format Markdown.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

/// One `#### Scenario:` heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Scenario {
    /// `PREFIX-NNN`, or `None` when the heading has no ID.
    pub id: Option<String>,
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
}

/// Parse every scenario heading in one spec file. Lines inside fenced code are skipped.
pub fn parse_scenarios(_file: &Path, _content: &str) -> Vec<Scenario> {
    todo!("TRACE-001, TRACE-002")
}

/// Find `specs/**/spec.md` and `changes/*/specs/**/spec.md` under `spec_root`, excluding `changes/archive/`.
pub fn discover_spec_files(_spec_root: &Path) -> Result<Vec<PathBuf>> {
    todo!("TRACE-003")
}

/// Discover and parse every spec file under `spec_root`.
pub fn load_scenarios(spec_root: &Path) -> Result<Vec<Scenario>> {
    let mut out = Vec::new();
    for file in discover_spec_files(spec_root)? {
        let content = std::fs::read_to_string(&file)?;
        out.extend(parse_scenarios(&file, &content));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/trace-basic/openspec")
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
        let mut files: Vec<String> = discover_spec_files(&fixture())
            .unwrap()
            .iter()
            .map(|p| {
                p.strip_prefix(fixture())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        files.sort();
        assert_eq!(
            files,
            vec!["changes/add-billing/specs/billing/spec.md", "specs/auth/spec.md"]
        );
    }
}
