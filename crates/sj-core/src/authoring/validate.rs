//! `sj spec validate`: spec format, delta structure and task-to-scenario links.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use super::blocks::{self, Block};
use super::{find_change, open_changes, read, spec_files, tasks, Change};

/// One problem, at a file and 1-based line (0 when it concerns the whole file or folder).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub errors: Vec<Problem>,
    pub warnings: Vec<Problem>,
}

impl Report {
    fn err(&mut self, message: impl Into<String>, file: &Path, line: usize) {
        self.errors.push(Problem {
            message: message.into(),
            file: file.to_path_buf(),
            line,
        });
    }

    fn warn(&mut self, message: impl Into<String>, file: &Path, line: usize) {
        self.warnings.push(Problem {
            message: message.into(),
            file: file.to_path_buf(),
            line,
        });
    }
}

/// Where an ID was first defined: (file, line).
type Seen = BTreeMap<String, (PathBuf, usize)>;

/// Validate every `specs/` file and open change, or only `change`.
pub fn validate(spec_root: &Path, change: Option<&str>) -> Result<Report> {
    let scoped = change.map(|c| find_change(spec_root, c)).transpose()?;
    let mut r = Report::default();
    let specs_dir = spec_root.join("specs");

    // IDs in specs/, in file order; duplicates there are errors when specs/ is in scope.
    let mut seen: Seen = BTreeMap::new();
    for file in spec_files(&specs_dir)? {
        let text = read(&file)?;
        let lines: Vec<&str> = text.split('\n').collect();
        let mask = blocks::fence_mask(&lines);
        let all = 0..lines.len();
        for s in blocks::scenarios_in(&lines, &mask, all.clone()) {
            let Some(id) = &s.id else { continue };
            if let Some(first) = seen.get(id) {
                if scoped.is_none() {
                    r.err(dup(id, first, spec_root), &file, s.line);
                }
            } else {
                seen.insert(id.clone(), (file.clone(), s.line));
            }
        }
        if scoped.is_none() {
            check_keywords(&mut r, &file, &lines, &mask);
            let blocks = blocks::blocks_in(&lines, &mask, all);
            for b in &blocks {
                check_block(&mut r, &file, &lines, &mask, b, None);
            }
            check_purpose(&mut r, &file, &text);
        }
    }

    // ADDED IDs of every open change, in change order, so two changes can't claim one ID.
    let changes = open_changes(spec_root)?;
    for c in &changes {
        let in_scope = scoped.as_ref().map_or(true, |s| s.name == c.name);
        validate_change(&mut r, spec_root, c, &mut seen, in_scope)?;
    }
    Ok(r)
}

fn dup(id: &str, first: &(PathBuf, usize), spec_root: &Path) -> String {
    let rel = first.0.strip_prefix(spec_root).unwrap_or(&first.0);
    format!(
        "duplicate ID {id} (first at {}:{})",
        rel.to_string_lossy().replace('\\', "/"),
        first.1
    )
}

/// Known translations of the structural keywords, lowercased, with the English heading.
const TRANSLATED: &[(&str, &str)] = &[
    ("scénario", "#### Scenario:"),
    ("escenario", "#### Scenario:"),
    ("cenário", "#### Scenario:"),
    ("cenario", "#### Scenario:"),
    ("szenario", "#### Scenario:"),
    ("scenariusz", "#### Scenario:"),
    ("сценарий", "#### Scenario:"),
    ("senaryo", "#### Scenario:"),
    ("シナリオ", "#### Scenario:"),
    ("场景", "#### Scenario:"),
    ("場景", "#### Scenario:"),
    ("시나리오", "#### Scenario:"),
    ("exigence", "### Requirement:"),
    ("exigência", "### Requirement:"),
    ("requisito", "### Requirement:"),
    ("requerimiento", "### Requirement:"),
    ("anforderung", "### Requirement:"),
    ("wymaganie", "### Requirement:"),
    ("требование", "### Requirement:"),
    ("gereksinim", "### Requirement:"),
    ("vereiste", "### Requirement:"),
    ("要件", "### Requirement:"),
    ("需求", "### Requirement:"),
    ("요구사항", "### Requirement:"),
    ("objectif", "## Purpose"),
    ("propósito", "## Purpose"),
    ("zweck", "## Purpose"),
];

/// Warn on a heading that starts with a translated structural keyword (CONF-017).
fn check_keywords(r: &mut Report, file: &Path, lines: &[&str], mask: &[bool]) {
    for (i, line) in lines.iter().enumerate() {
        if mask[i] {
            continue;
        }
        let Some(rest) = line.trim_start_matches('#').strip_prefix(' ') else {
            continue;
        };
        let hashes = line.len() - line.trim_start_matches('#').len();
        if !(2..=4).contains(&hashes) {
            continue;
        }
        let word: String = rest
            .trim_start()
            .split(|c: char| c == ':' || c.is_whitespace())
            .next()
            .unwrap_or("")
            .to_lowercase();
        if let Some((_, english)) = TRANSLATED.iter().find(|(w, _)| *w == word) {
            let shown = line.split(':').next().unwrap_or(line).trim_end();
            let colon = if english.ends_with(':') { ":" } else { "" };
            r.warn(
                format!(
                    "translated keyword `{shown}{colon}`: structural keywords stay English, use `{english}`"
                ),
                file,
                i + 1,
            );
        }
    }
}

/// The scenarios of the `specs/` copy of a MODIFIED requirement: those without an ID, with
/// the command that gives them one (SPEC-045/046), and the IDs of the others (SPEC-047).
struct IdFix {
    names: BTreeSet<String>,
    /// Scenario name → ID, for the scenarios that have one.
    ids: BTreeMap<String, String>,
    target: String,
    capability: String,
}

impl IdFix {
    fn hint(&self) -> String {
        format!("run sj ids --write {} first", self.capability)
    }
}

/// The `specs/` scenarios without an ID in the requirement a MODIFIED block `name` replaces.
/// `None` when the delta sits in no capability folder: the first folder, as `sj ids` reads it.
fn unnamed_originals(
    spec_root: &Path,
    target: &Path,
    rel: &Path,
    renames: &[blocks::Rename],
    name: &str,
) -> Result<Option<IdFix>> {
    let mut parts = rel.components();
    let capability = match (parts.next(), parts.next()) {
        (Some(c), Some(_)) => c.as_os_str().to_string_lossy().into_owned(),
        _ => return Ok(None),
    };
    let shown = target.strip_prefix(spec_root).unwrap_or(target);
    let mut fix = IdFix {
        names: BTreeSet::new(),
        ids: BTreeMap::new(),
        target: shown.to_string_lossy().replace('\\', "/"),
        capability,
    };
    if !target.is_file() {
        return Ok(Some(fix));
    }
    let original = blocks::name_before(renames, name);
    let text = read(target)?;
    let lines: Vec<&str> = text.split('\n').collect();
    let mask = blocks::fence_mask(&lines);
    for b in blocks::blocks_in(&lines, &mask, 0..lines.len()) {
        if b.name == original {
            for s in blocks::scenarios_in(&lines, &mask, b.lines.clone()) {
                match s.id {
                    None => {
                        fix.names.insert(s.name);
                    }
                    Some(id) => {
                        // The first of two same-named scenarios; other checks flag the pair.
                        fix.ids.entry(s.name).or_insert(id);
                    }
                }
            }
        }
    }
    Ok(Some(fix))
}

/// A requirement needs a scenario and a SHALL or MUST; each scenario needs an ID, WHEN and THEN.
/// `fix`, for a MODIFIED block, describes the `specs/` copy's scenarios: which lack an ID, and the
/// IDs of the rest.
fn check_block(
    r: &mut Report,
    file: &Path,
    lines: &[&str],
    mask: &[bool],
    b: &Block,
    fix: Option<&IdFix>,
) {
    let scenarios = blocks::scenarios_in(lines, mask, b.lines.clone());
    let first = scenarios.first().map_or(b.lines.end, |s| s.line - 1);
    let text = lines[b.header..first].join("\n");
    if !text.contains("SHALL") && !text.contains("MUST") {
        r.err(
            format!("requirement {} has no SHALL or MUST", b.name),
            file,
            b.header + 1,
        );
    }
    if scenarios.is_empty() {
        r.err(
            format!("requirement {} has no scenario", b.name),
            file,
            b.header + 1,
        );
    }
    for s in &scenarios {
        let label = s.id.clone().unwrap_or_else(|| s.name.clone());
        let unnamed = fix.filter(|f| f.names.contains(&s.name));
        match (&s.id, unnamed) {
            (None, None) => r.err(format!("{} has no ID", s.name), file, s.line),
            (None, Some(f)) => r.err(format!("{} has no ID; {}", s.name, f.hint()), file, s.line),
            (Some(id), Some(f)) => r.warn(
                format!(
                    "{id} {}: {} has this scenario without an ID; {}",
                    s.name,
                    f.target,
                    f.hint()
                ),
                file,
                s.line,
            ),
            (Some(id), None) => {
                // SPEC-047: tools that match by full name see two scenarios. When another
                // scenario in the block already has the original's ID, taking it would make two.
                let other = fix.and_then(|f| Some((f, f.ids.get(&s.name)?)));
                let in_block = |o: &String| scenarios.iter().any(|t| t.id.as_ref() == Some(o));
                if let Some((f, original)) = other.filter(|(_, o)| *o != id && !in_block(o)) {
                    r.warn(
                        format!(
                            "{id} {}: {} has this scenario as {original}; give the delta copy {original}",
                            s.name, f.target
                        ),
                        file,
                        s.line,
                    );
                }
            }
        }
        if let Some(m) = s.marker.as_deref().filter(|m| *m != crate::spec::KEEP) {
            r.err(format!("unknown scenario marker [{m}]"), file, s.line);
        }
        if !s.has_when {
            r.err(format!("{label} has no WHEN step"), file, s.line);
        }
        if !s.has_then {
            r.err(format!("{label} has no THEN step"), file, s.line);
        }
        if let Some(line) = s.late_for_all {
            r.err(
                format!("{label} FOR ALL step must come before WHEN"),
                file,
                line,
            );
        }
    }
}

fn check_purpose(r: &mut Report, file: &Path, text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();
    let mask = blocks::fence_mask(&lines);
    let Some(s) = blocks::sections(&lines, &mask)
        .into_iter()
        .find(|s| s.title.eq_ignore_ascii_case("Purpose"))
    else {
        r.warn("no Purpose section", file, 0);
        return;
    };
    let body = s.body.clone().find(|&i| !lines[i].trim().is_empty());
    match body {
        None => r.warn("Purpose is empty", file, s.header + 1),
        Some(i) if lines[i].trim_start().starts_with("TBD") => {
            r.warn("Purpose is a placeholder", file, i + 1)
        }
        Some(_) => {}
    }
}

/// Requirement names in a target spec file, after the delta's renames.
fn target_names(target: &Path, renames: &[blocks::Rename]) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    if target.is_file() {
        let text = read(target)?;
        let lines: Vec<&str> = text.split('\n').collect();
        let mask = blocks::fence_mask(&lines);
        for b in blocks::blocks_in(&lines, &mask, 0..lines.len()) {
            names.insert(b.name);
        }
    }
    Ok(names
        .into_iter()
        .map(|n| {
            renames
                .iter()
                .find(|r| r.from == n)
                .map_or(n, |r| r.to.clone())
        })
        .collect())
}

fn validate_change(
    r: &mut Report,
    spec_root: &Path,
    c: &Change,
    seen: &mut Seen,
    in_scope: bool,
) -> Result<()> {
    let mut sink = Report::default();
    let out = if in_scope { &mut *r } else { &mut sink };
    let specs_dir = c.dir.join("specs");
    // IDs defined by this change's MODIFIED blocks: one copy per change.
    let mut modified_ids: Seen = BTreeMap::new();
    for file in spec_files(&specs_dir)? {
        let text = read(&file)?;
        let lines: Vec<&str> = text.split('\n').collect();
        let mask = blocks::fence_mask(&lines);
        let d = blocks::parse_delta(&text);
        check_keywords(out, &file, &lines, &mask);
        let rel = file.strip_prefix(&specs_dir).unwrap_or(&file);
        let target = spec_root.join("specs").join(rel);
        let before = target_names(&target, &[])?;
        let after = target_names(&target, &d.renamed)?;

        if !d.has_operations() {
            out.err(
                "no ADDED, MODIFIED, REMOVED or RENAMED requirements",
                &file,
                0,
            );
        }
        for (name, line) in &d.orphans {
            out.err(
                format!(
                    "requirement {name} is outside ADDED/MODIFIED/REMOVED/RENAMED Requirements"
                ),
                &file,
                *line,
            );
        }
        for b in &d.added {
            check_block(out, &file, &lines, &mask, b, None);
            for s in blocks::scenarios_in(&lines, &mask, b.lines.clone()) {
                let Some(id) = s.id else { continue };
                match seen.get(&id) {
                    Some(first) => out.err(dup(&id, first, spec_root), &file, s.line),
                    None => {
                        seen.insert(id, (file.clone(), s.line));
                    }
                }
            }
        }
        for b in &d.modified {
            let fix = unnamed_originals(spec_root, &target, rel, &d.renamed, &b.name)?;
            check_block(out, &file, &lines, &mask, b, fix.as_ref());
            if !after.contains(&b.name) {
                out.err(
                    format!("MODIFIED requirement not found: {}", b.name),
                    &file,
                    b.header + 1,
                );
            }
            for s in blocks::scenarios_in(&lines, &mask, b.lines.clone()) {
                let Some(id) = s.id else { continue };
                match modified_ids.get(&id) {
                    Some(first) => out.err(dup(&id, first, spec_root), &file, s.line),
                    None => {
                        modified_ids.insert(id, (file.clone(), s.line));
                    }
                }
            }
        }
        for rm in &d.removed {
            if !before.contains(&rm.name) {
                out.err(
                    format!("REMOVED requirement not found: {}", rm.name),
                    &file,
                    rm.line,
                );
            }
            let range: Range<usize> = match &rm.block {
                Some(b) => b.lines.clone(),
                None => rm.section.clone(),
            };
            let body = lines[range].join("\n");
            if !body.contains("**Reason**") || !body.contains("**Migration**") {
                out.err(
                    format!(
                        "REMOVED requirement {} needs **Reason** and **Migration**",
                        rm.name
                    ),
                    &file,
                    rm.line,
                );
            }
        }
        for rn in &d.renamed {
            if !before.contains(&rn.from) {
                out.err(
                    format!("RENAMED requirement not found: {}", rn.from),
                    &file,
                    rn.line,
                );
            }
        }
        for (side, name, line) in &d.unpaired {
            out.err(
                format!("RENAMED {side} without its pair: {name}"),
                &file,
                *line,
            );
        }
    }

    let tasks_file = c.dir.join("tasks.md");
    if !tasks_file.is_file() {
        out.warn("change has no tasks.md", &c.dir, 0);
    }
    for t in tasks(&tasks_file)? {
        if blocks::task_ids(&t.text).is_empty() && !t.text.contains("(no-scenario:") {
            out.err(
                format!("task names no scenario: {}", t.text),
                &tasks_file,
                t.line,
            );
        }
    }
    Ok(())
}
