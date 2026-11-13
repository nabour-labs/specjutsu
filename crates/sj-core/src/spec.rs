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

/// A scenario's identity and normalised content, used to tell modified scenarios from unchanged ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    pub id: String,
    /// Heading name plus step lines, whitespace-collapsed, blank lines dropped.
    pub text: String,
}

/// Fingerprint every identified scenario in one spec file (fenced code skipped).
pub fn fingerprints(content: &str) -> Vec<Fingerprint> {
    let scenarios = parse_scenarios(Path::new(""), content);
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    for s in &scenarios {
        let Some(id) = &s.id else { continue };
        let mut parts = vec![collapse(&s.name)];
        for line in lines.iter().skip(s.line) {
            let t = line.trim_start();
            if t.starts_with('#') || t.starts_with("```") || t.starts_with("~~~") {
                break;
            }
            let c = collapse(line);
            if !c.is_empty() {
                parts.push(c);
            }
        }
        out.push(Fingerprint {
            id: id.clone(),
            text: parts.join("\n"),
        });
    }
    out
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether a repo-relative path (forward slashes) is a spec file under `spec_root` by the
/// discovery rule: `specs/**/spec.md` or `changes/<change>/specs/**/spec.md`, archive excluded.
pub fn is_spec_path(rel: &str, spec_root: &str) -> bool {
    let root = spec_root.trim_end_matches('/');
    let Some(rest) = rel.strip_prefix(root).and_then(|r| r.strip_prefix('/')) else {
        return false;
    };
    if !rest.ends_with("/spec.md") {
        return false;
    }
    if rest.starts_with("specs/") {
        return true;
    }
    let mut parts = rest.split('/');
    parts.next() == Some("changes")
        && parts.next().is_some_and(|c| c != "archive")
        && parts.next() == Some("specs")
}
