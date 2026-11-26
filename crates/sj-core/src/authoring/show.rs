//! `sj spec show <item>`: one capability spec, one change or one scenario with its evidence.
//!
//! An item resolves as a scenario ID, then an open change, then an archived change (by folder
//! name, or by bare name when only one archived change has it), then a capability.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use serde::Serialize;

use super::{open_changes, read, spec_files, tasks};
use crate::agent_loop::{Evidence, Ledger};
use crate::config::Config;
use crate::spec;

/// A mapped test, as `<file>:<line> <name>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestLine {
    /// Forward-slash path relative to the repository root.
    pub file: String,
    pub line: usize,
    pub name: String,
}

/// One delta file of a change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Delta {
    pub capability: String,
    /// Forward-slash path relative to the change folder.
    pub file: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Shown {
    Capability {
        name: String,
        file: String,
        text: String,
    },
    Change {
        name: String,
        /// `YYYY-MM-DD` for an archived change.
        archived: Option<String>,
        proposal: Option<String>,
        deltas: Vec<Delta>,
        tasks_done: usize,
        tasks_total: usize,
    },
    Scenario {
        id: String,
        name: String,
        capability: String,
        /// The open change whose delta holds it, when it isn't in `specs/`.
        change: Option<String>,
        file: String,
        line: usize,
        /// A `[keep]` scenario: behaviour the change must not alter.
        keep: bool,
        /// A `- **FOR ALL**` step makes it a property scenario.
        property: bool,
        requirement: Option<String>,
        /// The requirement's first paragraph: its SHALL sentence.
        statement: Option<String>,
        steps: Vec<String>,
        tests: Vec<TestLine>,
        /// The latest ledger result, or `None` when no run names it.
        latest: Option<String>,
        evidence: Evidence,
    },
}

/// What `show` found.
pub enum Outcome {
    Found(Box<Shown>),
    /// Nothing matched; the nearest names, at most 3.
    Unknown(Vec<String>),
    /// A bare name that several archived changes share: their folder names.
    Ambiguous(Vec<String>),
}

fn is_id(s: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9]*-[0-9]+$").unwrap())
        .is_match(s)
}

fn rel(p: &Path, base: &Path) -> String {
    p.strip_prefix(base)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// The capability a spec file belongs to: the folders between `specs/` and `spec.md`.
fn capability_of(file: &Path) -> String {
    let parts: Vec<String> = file
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    match parts.iter().rposition(|p| p == "specs") {
        Some(i) if i + 2 <= parts.len() => parts[i + 1..parts.len() - 1].join("/"),
        _ => String::new(),
    }
}

/// `YYYY-MM-DD-name` as (date, name).
fn split_archived(folder: &str) -> Option<(&str, &str)> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"^(\d{4}-\d{2}-\d{2})-(.+)$").unwrap());
    let c = re.captures(folder)?;
    Some((c.get(1)?.as_str(), c.get(2)?.as_str()))
}

/// Archived change folder names, sorted.
pub fn archived(spec_root: &Path) -> Result<Vec<String>> {
    let dir = spec_root.join("changes").join("archive");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    out.sort();
    Ok(out)
}

/// (capability name, spec file) for every `spec.md` under `specs/`, sorted.
pub fn capabilities(spec_root: &Path) -> Result<Vec<(String, PathBuf)>> {
    Ok(spec_files(&spec_root.join("specs"))?
        .into_iter()
        .map(|f| (capability_of(&f), f))
        .collect())
}

pub fn show(root: &Path, spec_root: &Path, config: &Config, item: &str) -> Result<Outcome> {
    if is_id(item) {
        if let Some(s) = scenario(root, spec_root, config, item)? {
            return Ok(Outcome::Found(Box::new(s)));
        }
    }
    if let Some(c) = open_changes(spec_root)?
        .into_iter()
        .find(|c| c.name == item)
    {
        return Ok(Outcome::Found(Box::new(change(&c.dir, &c.name, None)?)));
    }
    let archive = archived(spec_root)?;
    let archive_dir = spec_root.join("changes").join("archive");
    let by_folder = archive.iter().find(|f| f.as_str() == item);
    let by_name: Vec<&String> = archive
        .iter()
        .filter(|f| split_archived(f).is_some_and(|(_, n)| n == item))
        .collect();
    let folder = match (by_folder, by_name.as_slice()) {
        (Some(f), _) => Some(f),
        (None, [one]) => Some(*one),
        (None, []) => None,
        (None, many) => {
            return Ok(Outcome::Ambiguous(
                many.iter().map(|s| s.to_string()).collect(),
            ))
        }
    };
    if let Some(folder) = folder {
        let (date, name) = match split_archived(folder) {
            Some((d, n)) => (Some(d.to_string()), n),
            None => (None, folder.as_str()),
        };
        return Ok(Outcome::Found(Box::new(change(
            &archive_dir.join(folder),
            name,
            date,
        )?)));
    }
    for (name, file) in capabilities(spec_root)? {
        if name == item {
            let text = std::fs::read_to_string(&file)?;
            return Ok(Outcome::Found(Box::new(Shown::Capability {
                name,
                file: rel(&file, root),
                text,
            })));
        }
    }
    Ok(Outcome::Unknown(suggest(spec_root, item)?))
}

fn change(dir: &Path, name: &str, archived: Option<String>) -> Result<Shown> {
    let proposal = dir.join("proposal.md");
    let proposal = if proposal.is_file() {
        Some(std::fs::read_to_string(&proposal)?)
    } else {
        None
    };
    let mut deltas = Vec::new();
    for file in spec_files(&dir.join("specs"))? {
        deltas.push(Delta {
            capability: capability_of(&file),
            file: rel(&file, dir),
            text: std::fs::read_to_string(&file)?,
        });
    }
    let t = tasks(&dir.join("tasks.md"))?;
    Ok(Shown::Change {
        name: name.to_string(),
        archived,
        proposal,
        deltas,
        tasks_done: t.iter().filter(|t| t.done).count(),
        tasks_total: t.len(),
    })
}

fn scenario(root: &Path, spec_root: &Path, config: &Config, id: &str) -> Result<Option<Shown>> {
    let changes_dir = spec_root.join("changes");
    for file in spec::discover_spec_files(spec_root)? {
        let text = read(&file)?;
        let Some(loc) = spec::locate_scenarios(&file, &text)
            .into_iter()
            .find(|l| l.scenario.id.as_deref() == Some(id))
        else {
            continue;
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let line = loc.scenario.line;
        let steps = lines
            .iter()
            .skip(line)
            .take_while(|l| !l.trim_start().starts_with('#'))
            .map(|l| l.trim_end())
            .filter(|l| !l.trim().is_empty())
            .map(str::to_string)
            .collect();
        let statement = loc
            .requirement
            .as_deref()
            .and_then(|r| requirement_statement(&lines, r));
        let change = file
            .strip_prefix(&changes_dir)
            .ok()
            .and_then(|p| p.components().next())
            .map(|c| c.as_os_str().to_string_lossy().into_owned());

        let scenarios = spec::load_scenarios(spec_root)?;
        let scan = crate::testscan::scan_project(root, config)?;
        let report = crate::trace::build_report(&scenarios, &scan.tests);
        let tests = report
            .mapped
            .iter()
            .filter(|m| m.id == id)
            .flat_map(|m| &m.tests)
            .map(|t| TestLine {
                file: rel(&t.file, root),
                line: t.line,
                name: t.name.clone(),
            })
            .collect();
        let ledger = Ledger::load(root)?;
        return Ok(Some(Shown::Scenario {
            id: id.to_string(),
            name: loc.scenario.name.clone(),
            capability: capability_of(&file),
            change,
            file: rel(&file, root),
            line,
            keep: loc.scenario.keep,
            property: loc.scenario.property,
            requirement: loc.requirement.clone(),
            statement,
            steps,
            tests,
            latest: ledger.status(id).map(|s| s.latest.as_str().to_string()),
            evidence: ledger.evidence(id),
        }));
    }
    Ok(None)
}

/// The first paragraph under `### Requirement: <name>`.
fn requirement_statement(lines: &[&str], name: &str) -> Option<String> {
    let header = lines.iter().position(|l| {
        super::blocks::requirement_name(l).is_some_and(|n| n.trim() == name.trim())
    })?;
    let para: Vec<&str> = lines[header + 1..]
        .iter()
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|l| l.trim())
        .collect();
    (!para.is_empty()).then(|| para.join(" "))
}

/// Up to 3 known names or IDs nearest to `item`, at an edit distance of at most 3.
fn suggest(spec_root: &Path, item: &str) -> Result<Vec<String>> {
    let mut known: Vec<String> = capabilities(spec_root)?
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    known.extend(open_changes(spec_root)?.into_iter().map(|c| c.name));
    for f in archived(spec_root)? {
        if let Some((_, n)) = split_archived(&f) {
            known.push(n.to_string());
        }
        known.push(f);
    }
    known.extend(
        spec::load_scenarios(spec_root)?
            .into_iter()
            .filter_map(|s| s.id),
    );
    known.sort();
    known.dedup();
    let mut scored: Vec<(usize, String)> = known
        .into_iter()
        .map(|k| (levenshtein(item, &k), k))
        .filter(|(d, _)| *d <= 3)
        .collect();
    scored.sort();
    Ok(scored.into_iter().take(3).map(|(_, k)| k).collect())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The text form of a found item. A capability is its file, byte for byte.
pub fn render(s: &Shown) -> String {
    match s {
        Shown::Capability { text, .. } => text.clone(),
        Shown::Change {
            name,
            archived,
            proposal,
            deltas,
            tasks_done,
            tasks_total,
        } => {
            let state = match archived {
                Some(date) => format!("archived {date}"),
                None => format!("{tasks_done}/{tasks_total} tasks"),
            };
            let mut out = format!("change {name}  {state}\n");
            if let Some(p) = proposal {
                out.push_str("\n--- proposal.md\n\n");
                out.push_str(p.trim_end());
                out.push('\n');
            }
            for d in deltas {
                out.push_str(&format!("\n--- {} ({})\n\n", d.capability, d.file));
                out.push_str(d.text.trim_end());
                out.push('\n');
            }
            out
        }
        Shown::Scenario {
            id,
            name,
            capability,
            change,
            file,
            line,
            keep,
            requirement,
            statement,
            steps,
            tests,
            latest,
            evidence,
            ..
        } => {
            let marker = if *keep { "[keep] " } else { "" };
            let mut out = format!("{id} {marker}{name}\n");
            out.push_str(&format!("capability   {capability} ({file}:{line})\n"));
            if let Some(c) = change {
                out.push_str(&format!("change       {c}\n"));
            }
            if let Some(r) = requirement {
                out.push_str(&format!("requirement  {r}\n"));
            }
            if let Some(st) = statement {
                out.push_str(&format!("  {st}\n"));
            }
            out.push('\n');
            for step in steps {
                out.push_str(step);
                out.push('\n');
            }
            out.push('\n');
            if tests.is_empty() {
                out.push_str("tests        none\n");
            } else {
                out.push_str("tests\n");
                for t in tests {
                    out.push_str(&format!("  {}:{} {}\n", t.file, t.line, t.name));
                }
            }
            let ev = match (latest, evidence) {
                (_, Evidence::Proven) => "latest passed, red-before-green proven".to_string(),
                (Some(l), Evidence::GreenUnproven) => {
                    format!("latest {l}, red-before-green not proven")
                }
                (Some(l), Evidence::Red) => format!("latest {l}"),
                _ => "untested".to_string(),
            };
            out.push_str(&format!("evidence     {ev}\n"));
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        assert_eq!(levenshtein("aut", "auth"), 1);
        assert_eq!(levenshtein("aut", "audit"), 2);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn capability_names() {
        assert_eq!(capability_of(Path::new("o/specs/auth/spec.md")), "auth");
        assert_eq!(
            capability_of(Path::new("o/changes/x/specs/a/b/spec.md")),
            "a/b"
        );
    }
}
