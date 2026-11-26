//! The Markdown model shared by validate, check and archive: fences, `## ` sections,
//! requirement blocks, delta sections, scenarios and Purpose. The rules follow OpenSpec
//! 1.14.0's `requirement-blocks.js`, so both tools see the same structure.

use std::ops::Range;
use std::sync::OnceLock;

use regex::Regex;

/// Drop a leading BOM and turn CRLF and CR into LF.
pub fn normalize(content: &str) -> String {
    content
        .strip_prefix('\u{feff}')
        .unwrap_or(content)
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

/// `true` for every line inside a fenced code block, the fence lines included.
pub fn fence_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = vec![false; lines.len()];
    let mut active: Option<(char, usize)> = None;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim_start();
        let run = |c: char| t.chars().take_while(|&x| x == c).count();
        match active {
            None => {
                for c in ['`', '~'] {
                    let n = run(c);
                    if n >= 3 {
                        active = Some((c, n));
                        mask[i] = true;
                        break;
                    }
                }
            }
            Some((c, n)) => {
                mask[i] = true;
                let m = run(c);
                if m >= n && t[m * c.len_utf8()..].trim().is_empty() {
                    active = None;
                }
            }
        }
    }
    mask
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).unwrap())
}

/// The name in a `### Requirement:` header line, or `None` when the line is not one.
pub fn requirement_name(line: &str) -> Option<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"(?i)^###\s*Requirement:\s*(.+)\s*$")
        .captures(line)
        .map(|c| normalize_name(&c[1]))
}

/// Trim a requirement name and drop a closing ` ###` run.
pub fn normalize_name(name: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"[ \t]+#+[ \t]*$")
        .replace(name, "")
        .trim()
        .to_string()
}

/// A `## ` title, or `None`.
pub fn section_title(line: &str) -> Option<&str> {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^##\s+(.+)$")
        .captures(line)
        .map(|c| c.get(1).unwrap().as_str().trim())
}

/// A `## ` section: its title, the header's line index, and the body's line indexes.
#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub header: usize,
    pub body: Range<usize>,
}

pub fn sections(lines: &[&str], mask: &[bool]) -> Vec<Section> {
    let heads: Vec<(usize, String)> = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !mask[*i])
        .filter_map(|(i, l)| section_title(l).map(|t| (i, t.to_string())))
        .collect();
    heads
        .iter()
        .enumerate()
        .map(|(k, (i, title))| Section {
            title: title.clone(),
            header: *i,
            body: i + 1..heads.get(k + 1).map_or(lines.len(), |h| h.0),
        })
        .collect()
}

/// One requirement block: its header line index, its lines and its raw text (trailing
/// whitespace trimmed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    pub header: usize,
    pub lines: Range<usize>,
    pub raw: String,
}

/// Requirement blocks inside `range`: each runs to the next requirement header or `## ` line.
pub fn blocks_in(lines: &[&str], mask: &[bool], range: Range<usize>) -> Vec<Block> {
    let is_req = |i: usize| !mask[i] && requirement_name(lines[i]).is_some();
    let is_top = |i: usize| !mask[i] && section_title(lines[i]).is_some();
    let mut out = Vec::new();
    let mut i = range.start;
    while i < range.end {
        if !is_req(i) {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < range.end && !is_req(i) && !is_top(i) {
            i += 1;
        }
        out.push(Block {
            name: requirement_name(lines[start]).unwrap(),
            header: start,
            lines: start..i,
            raw: lines[start..i].join("\n").trim_end().to_string(),
        });
    }
    out
}

/// A `#### Scenario:` heading inside a block, with whether it has WHEN and THEN steps.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub id: Option<String>,
    pub name: String,
    /// The `[word]` between the ID and the name, if any (`keep` is the only valid one).
    pub marker: Option<String>,
    pub line: usize,
    pub has_when: bool,
    pub has_then: bool,
    /// The 1-based line of the first FOR ALL step that comes after a WHEN step.
    pub late_for_all: Option<usize>,
}

/// Scenario headings in `range`. A scenario's steps run to the next heading.
pub fn scenarios_in(lines: &[&str], mask: &[bool], range: Range<usize>) -> Vec<Scenario> {
    static HEAD: OnceLock<Regex> = OnceLock::new();
    static ID: OnceLock<Regex> = OnceLock::new();
    let head = re(&HEAD, r"^####\s+Scenario:\s*(.*?)\s*$");
    let id_re = re(&ID, r"^([A-Z][A-Z0-9]*-[0-9]+)(?:\s+(.*))?$");
    let mut out: Vec<Scenario> = Vec::new();
    let mut current: Option<Scenario> = None;
    for i in range {
        if mask[i] {
            continue;
        }
        let line = lines[i];
        if line.starts_with('#') {
            if let Some(s) = current.take() {
                out.push(s);
            }
            if let Some(c) = head.captures(line) {
                let title = c[1].to_string();
                let (id, name) = match id_re.captures(&title) {
                    Some(m) => (
                        Some(m[1].to_string()),
                        m.get(2).map_or(String::new(), |x| x.as_str().to_string()),
                    ),
                    None => (None, title),
                };
                let (marker, name) = match crate::spec::split_marker(&name) {
                    (Some(m), rest) => (Some(m.to_string()), rest.to_string()),
                    (None, _) => (None, name),
                };
                current = Some(Scenario {
                    id,
                    name,
                    marker,
                    line: i + 1,
                    has_when: false,
                    has_then: false,
                    late_for_all: None,
                });
            }
            continue;
        }
        if let Some(s) = current.as_mut() {
            match crate::spec::step_keyword(line) {
                Some(crate::spec::Step::When) => s.has_when = true,
                Some(crate::spec::Step::Then) => s.has_then = true,
                Some(crate::spec::Step::ForAll) if s.has_when => {
                    s.late_for_all.get_or_insert(i + 1);
                }
                _ => {}
            }
        }
    }
    out.extend(current);
    out
}

/// The four delta kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Added,
    Modified,
    Removed,
    Renamed,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Added, Kind::Modified, Kind::Removed, Kind::Renamed];

    pub fn word(self) -> &'static str {
        match self {
            Kind::Added => "ADDED",
            Kind::Modified => "MODIFIED",
            Kind::Removed => "REMOVED",
            Kind::Renamed => "RENAMED",
        }
    }

    fn of(title: &str) -> Option<Kind> {
        Kind::ALL
            .into_iter()
            .find(|k| title.eq_ignore_ascii_case(&format!("{} Requirements", k.word())))
    }
}

/// A REMOVED entry: a header (with its block) or a bullet naming one.
#[derive(Debug, Clone)]
pub struct Removed {
    pub name: String,
    pub line: usize,
    /// The header form's block. `None` for the bullet form.
    pub block: Option<Block>,
    /// The REMOVED section body this entry sits in, for the bullet form's markers.
    pub section: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Rename {
    pub from: String,
    pub to: String,
    pub line: usize,
}

/// The name requirement `name` had before `renamed`: the `from` of the rename to it, or
/// `name` itself when nothing renames to it.
pub fn name_before<'a>(renamed: &'a [Rename], name: &'a str) -> &'a str {
    renamed
        .iter()
        .find(|r| r.to == name)
        .map_or(name, |r| r.from.as_str())
}

/// A parsed delta file. Lines are kept so callers can scan blocks and sections.
#[derive(Debug, Clone, Default)]
pub struct Delta {
    pub added: Vec<Block>,
    pub modified: Vec<Block>,
    pub removed: Vec<Removed>,
    pub renamed: Vec<Rename>,
    /// FROM or TO lines that never formed a pair: (side, name, 1-based line).
    pub unpaired: Vec<(&'static str, String, usize)>,
    /// Requirement headers outside every delta section: (name, 1-based line).
    pub orphans: Vec<(String, usize)>,
}

impl Delta {
    pub fn has_operations(&self) -> bool {
        !(self.added.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.renamed.is_empty())
    }
}

/// Parse a delta file (already [`normalize`]d).
pub fn parse_delta(text: &str) -> Delta {
    static BULLET: OnceLock<Regex> = OnceLock::new();
    static FROM: OnceLock<Regex> = OnceLock::new();
    static TO: OnceLock<Regex> = OnceLock::new();
    let bullet = re(
        &BULLET,
        r"(?i)^\s*[-*+]\s*`?###\s*Requirement:\s*(.+?)`?\s*$",
    );
    let from_re = re(
        &FROM,
        r"(?i)^\s*[-*+]?\s*FROM:\s*`?###\s*Requirement:\s*(.+?)`?\s*$",
    );
    let to_re = re(
        &TO,
        r"(?i)^\s*[-*+]?\s*TO:\s*`?###\s*Requirement:\s*(.+?)`?\s*$",
    );

    let lines: Vec<&str> = text.split('\n').collect();
    let mask = fence_mask(&lines);
    let mut d = Delta::default();
    let mut in_delta = vec![false; lines.len()];
    for s in sections(&lines, &mask) {
        let Some(kind) = Kind::of(&s.title) else {
            continue;
        };
        for i in s.body.clone() {
            in_delta[i] = true;
        }
        match kind {
            Kind::Added => d.added.extend(blocks_in(&lines, &mask, s.body.clone())),
            Kind::Modified => d.modified.extend(blocks_in(&lines, &mask, s.body.clone())),
            Kind::Removed => {
                let blocks = blocks_in(&lines, &mask, s.body.clone());
                for i in s.body.clone() {
                    if mask[i] {
                        continue;
                    }
                    if let Some(name) = requirement_name(lines[i]) {
                        d.removed.push(Removed {
                            name,
                            line: i + 1,
                            block: blocks.iter().find(|b| b.header == i).cloned(),
                            section: s.body.clone(),
                        });
                    } else if let Some(c) = bullet.captures(lines[i]) {
                        d.removed.push(Removed {
                            name: normalize_name(&c[1]),
                            line: i + 1,
                            block: None,
                            section: s.body.clone(),
                        });
                    }
                }
            }
            Kind::Renamed => {
                let mut pending: Option<(String, usize)> = None;
                for i in s.body.clone() {
                    if mask[i] {
                        continue;
                    }
                    if let Some(c) = from_re.captures(lines[i]) {
                        if let Some((name, line)) = pending.take() {
                            d.unpaired.push(("FROM", name, line));
                        }
                        pending = Some((normalize_name(&c[1]), i + 1));
                    } else if let Some(c) = to_re.captures(lines[i]) {
                        let to = normalize_name(&c[1]);
                        match pending.take() {
                            Some((from, line)) => d.renamed.push(Rename { from, to, line }),
                            None => d.unpaired.push(("TO", to, i + 1)),
                        }
                    }
                }
                if let Some((name, line)) = pending {
                    d.unpaired.push(("FROM", name, line));
                }
            }
        }
    }
    for (i, line) in lines.iter().enumerate() {
        if !mask[i] && !in_delta[i] {
            if let Some(name) = requirement_name(line) {
                d.orphans.push((name, i + 1));
            }
        }
    }
    d.unpaired.sort_by_key(|u| u.2);
    d
}

/// The trimmed body of the first `## Purpose` section, when it has text.
pub fn purpose(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mask = fence_mask(&lines);
    let s = sections(&lines, &mask)
        .into_iter()
        .find(|s| s.title.eq_ignore_ascii_case("Purpose"))?;
    let body = lines[s.body].join("\n").trim().to_string();
    (!body.is_empty()).then_some(body)
}

/// Names of the `####` scenario headers with a body in a block's raw text, without a
/// `Scenario:` prefix (OpenSpec's MODIFIED guard compares these).
pub fn scenario_names(raw: &str) -> Vec<String> {
    static H4: OnceLock<Regex> = OnceLock::new();
    static END: OnceLock<Regex> = OnceLock::new();
    let h4 = re(&H4, r"^####\s+(.*)$");
    let end = re(&END, r"^#{1,4}\s");
    let lines: Vec<&str> = raw.split('\n').collect();
    let mask = fence_mask(&lines);
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if mask[i] {
            continue;
        }
        let Some(c) = h4.captures(line) else { continue };
        let mut has_body = false;
        for (j, l) in lines.iter().enumerate().skip(i + 1) {
            if !mask[j] && end.is_match(l) {
                break;
            }
            if !l.trim().is_empty() {
                has_body = true;
                break;
            }
        }
        if !has_body {
            continue;
        }
        let t = c[1].trim();
        let name = t
            .strip_prefix("Scenario:")
            .or_else(|| t.strip_prefix("scenario:"))
            .unwrap_or(t)
            .trim();
        out.push(name.to_string());
    }
    out
}

/// Scenario IDs named in a task's text, ranges like `IDS-001..007` expanded.
pub fn task_ids(text: &str) -> Vec<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let r = re(
        &R,
        r"[A-Z][A-Z0-9]*-[0-9]+(?:\.\.(?:[A-Z][A-Z0-9]*-)?[0-9]+)?",
    );
    r.find_iter(text)
        .flat_map(|m| crate::gate::expand_ids(m.as_str()))
        .collect()
}
