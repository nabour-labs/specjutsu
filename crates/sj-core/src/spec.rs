//! Scenario extraction from spec Markdown.

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
    /// The heading carries `[keep]`: behaviour the change must not alter (GATE-039).
    pub keep: bool,
    /// A step reads `- **FOR ALL** <inputs>`: the scenario is a property (SPEC-072).
    pub property: bool,
}

/// The marker allowed between a scenario's ID and its name.
pub const KEEP: &str = "keep";

/// Split a leading `[word] ` marker off a scenario name: `[keep] Refunds` → (`Some("keep")`,
/// `Refunds`). A name with no marker comes back whole.
pub fn split_marker(name: &str) -> (Option<&str>, &str) {
    static MARKER: OnceLock<Regex> = OnceLock::new();
    let re = MARKER.get_or_init(|| Regex::new(r"^\[([A-Za-z][A-Za-z-]*)\]\s+(.*)$").unwrap());
    match re.captures(name) {
        Some(c) => (Some(c.get(1).unwrap().as_str()), c.get(2).unwrap().as_str()),
        None => (None, name),
    }
}

/// The keyword of a scenario step line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    ForAll,
    When,
    Then,
}

/// The keyword a step line opens with: `- **WHEN** ...`, `* **then** ...`, `- **FOR ALL** ...`.
/// Bullets may be `-` or `*` and keywords any case. Other lines give `None`.
pub fn step_keyword(line: &str) -> Option<Step> {
    static STEP: OnceLock<Regex> = OnceLock::new();
    let re =
        STEP.get_or_init(|| Regex::new(r"(?i)^\s*[-*]\s+\*\*(WHEN|THEN|FOR\s+ALL)\*\*").unwrap());
    let c = re.captures(line)?;
    let k = &c[1];
    Some(if k.eq_ignore_ascii_case("WHEN") {
        Step::When
    } else if k.eq_ignore_ascii_case("THEN") {
        Step::Then
    } else {
        Step::ForAll
    })
}

/// A scenario heading with where it sits in the file's bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    pub scenario: Scenario,
    /// Byte offset of the heading's title (the ID when there is one, else the name).
    pub title_offset: usize,
    /// The `### Requirement:` the scenario sits under, reset by any `## ` heading.
    pub requirement: Option<String>,
}

/// Parse every scenario heading in one spec file. Lines inside fenced code are skipped.
pub fn parse_scenarios(file: &Path, content: &str) -> Vec<Scenario> {
    locate_scenarios(file, content)
        .into_iter()
        .map(|l| l.scenario)
        .collect()
}

/// Every scenario heading with its byte offset and requirement. Lines inside fenced code are
/// skipped. Line ends (`\n` or `\r\n`) are handled as [`str::lines`] does.
pub fn locate_scenarios(file: &Path, content: &str) -> Vec<Located> {
    static HEADING: OnceLock<Regex> = OnceLock::new();
    static REQUIREMENT: OnceLock<Regex> = OnceLock::new();
    static ID: OnceLock<Regex> = OnceLock::new();
    let heading = HEADING.get_or_init(|| Regex::new(r"^####\s+Scenario:\s*(.*?)\s*$").unwrap());
    let requirement =
        REQUIREMENT.get_or_init(|| Regex::new(r"^###\s+Requirement:\s*(.*?)\s*$").unwrap());
    let id_re = ID.get_or_init(|| Regex::new(r"^([A-Z][A-Z0-9]*-[0-9]+)(?:\s+(.*))?$").unwrap());

    let mut out: Vec<Located> = Vec::new();
    let mut fence: Option<&str> = None;
    let mut current_req: Option<String> = None;
    let mut offset = 0;
    // Inside a scenario's steps: after its heading, before the next heading.
    let mut in_steps = false;
    for (i, raw) in content.split_inclusive('\n').enumerate() {
        let line_start = offset;
        offset += raw.len();
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let line = line.strip_suffix('\r').unwrap_or(line);
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
        if trimmed.starts_with('#') {
            in_steps = false;
        } else if in_steps && step_keyword(line) == Some(Step::ForAll) {
            if let Some(last) = out.last_mut() {
                last.scenario.property = true;
            }
            continue;
        }
        if let Some(c) = requirement.captures(line) {
            current_req = Some(c[1].to_string());
            continue;
        }
        if line.starts_with("## ") || line == "##" {
            current_req = None;
            continue;
        }
        let Some(caps) = heading.captures(line) else {
            continue;
        };
        let title_match = caps.get(1).unwrap();
        let title = title_match.as_str().to_string();
        let (id, name) = match id_re.captures(&title) {
            Some(c) => (
                Some(c[1].to_string()),
                c.get(2).map_or(String::new(), |m| m.as_str().to_string()),
            ),
            None => (None, title),
        };
        let (keep, name) = match split_marker(&name) {
            (Some(KEEP), rest) => (true, rest.to_string()),
            _ => (false, name),
        };
        out.push(Located {
            scenario: Scenario {
                id,
                name,
                file: file.to_path_buf(),
                line: i + 1,
                keep,
                property: false,
            },
            title_offset: line_start + title_match.start(),
            requirement: current_req.clone(),
        });
        in_steps = true;
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

/// The IDs whose scenario carries `[keep]` (GATE-039). An open change's copy comes after
/// `specs/`, so its marker wins for an ID that appears in both.
pub fn keep_ids(spec_root: &Path) -> Result<std::collections::BTreeSet<String>> {
    let mut keep_by_id = std::collections::BTreeMap::new();
    for s in load_scenarios(spec_root)? {
        if let Some(id) = s.id {
            keep_by_id.insert(id, s.keep);
        }
    }
    Ok(keep_by_id
        .into_iter()
        .filter_map(|(id, k)| k.then_some(id))
        .collect())
}

/// IDs of FOR ALL scenarios, judged by each ID's last occurrence (a change's version of a
/// scenario wins over `specs/`, as for [`keep_ids`]).
pub fn property_ids(scenarios: &[Scenario]) -> std::collections::BTreeSet<String> {
    let mut by_id = std::collections::BTreeMap::new();
    for s in scenarios {
        if let Some(id) = &s.id {
            by_id.insert(id.clone(), s.property);
        }
    }
    by_id
        .into_iter()
        .filter_map(|(id, p)| p.then_some(id))
        .collect()
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

/// Every scenario in one spec file with its normalised text: heading name plus step lines,
/// whitespace-collapsed, blank lines dropped (fenced code skipped). The ID is not part of the text.
pub fn scenario_texts(file: &Path, content: &str) -> Vec<(Scenario, String)> {
    let lines: Vec<&str> = content.lines().collect();
    parse_scenarios(file, content)
        .into_iter()
        .map(|s| {
            // The marker is part of the text: adding or dropping `[keep]` modifies a scenario.
            let shown = if s.keep {
                format!("[{KEEP}] {}", s.name)
            } else {
                s.name.clone()
            };
            let mut parts = vec![collapse(&shown)];
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
            let text = parts.join("\n");
            (s, text)
        })
        .collect()
}

/// Fingerprint every identified scenario in one spec file (fenced code skipped).
pub fn fingerprints(content: &str) -> Vec<Fingerprint> {
    scenario_texts(Path::new(""), content)
        .into_iter()
        .filter_map(|(s, text)| s.id.map(|id| Fingerprint { id, text }))
        .collect()
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
