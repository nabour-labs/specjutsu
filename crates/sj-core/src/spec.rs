//! Scenario extraction from OpenSpec-format Markdown.

use std::path::{Path, PathBuf};

use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
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
pub fn parse_scenarios(file: &Path, content: &str) -> Vec<Scenario> {
    static HEADING: OnceLock<Regex> = OnceLock::new();
    static ID: OnceLock<Regex> = OnceLock::new();
    let heading = HEADING.get_or_init(|| Regex::new(r"^####\s+Scenario:\s*(.*?)\s*$").unwrap());
    let id_re = ID.get_or_init(|| Regex::new(r"^([A-Z][A-Z0-9]*-[0-9]+)(?:\s+(.*))?$").unwrap());

    let mut out = Vec::new();
    let mut fence: Option<&str> = None;
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let marker = if trimmed.starts_with("```") {
            Some("```")
        } else if trimmed.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };
        if let Some(m) = marker {
            match fence {
                None => fence = Some(m),
                Some(open) if open == m => fence = None,
                Some(_) => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let Some(caps) = heading.captures(line) else {
            continue;
        };
        let title = caps[1].to_string();
        let (id, name) = match id_re.captures(&title) {
            Some(c) => (
                Some(c[1].to_string()),
                c.get(2).map_or(String::new(), |m| m.as_str().to_string()),
            ),
            None => (None, title),
        };
        out.push(Scenario {
            id,
            name,
            file: file.to_path_buf(),
            line: i + 1,
        });
    }
    out
}

/// Find `specs/**/spec.md` and `changes/*/specs/**/spec.md` under `spec_root`, excluding `changes/archive/`.
pub fn discover_spec_files(spec_root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_spec_md(&spec_root.join("specs"), &mut out)?;
    let changes = spec_root.join("changes");
    if changes.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(&changes)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.file_name() != "archive")
            .map(|e| e.path())
            .collect();
        entries.sort();
        for change in entries {
            collect_spec_md(&change.join("specs"), &mut out)?;
        }
    }
    Ok(out)
}

fn collect_spec_md(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_spec_md(&path, out)?;
        } else if path.file_name().is_some_and(|n| n == "spec.md") {
            out.push(path);
        }
    }
    Ok(())
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
        let src =
            "### Requirement: R\n\n#### Scenario: AUTH-003 Reject expired token\n- **WHEN** x\n";
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
            vec![
                "changes/add-billing/specs/billing/spec.md",
                "specs/auth/spec.md"
            ]
        );
    }
}
