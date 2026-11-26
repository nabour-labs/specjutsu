//! Test discovery: per-language scanners that find test functions and the scenario IDs they name.

mod js;
mod python;

pub use js::scan_js;
pub use python::scan_python;

use std::path::{Path, PathBuf};

use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use serde::Serialize;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use crate::config::{Config, Package, Runner};

/// A test found in source.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TestFn {
    /// Function name (Rust, pytest `Class::func`) or titles joined with ` > ` (JS/TS).
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    /// Explicit IDs: `spec:` comments above the test and `@pytest.mark.spec` markers.
    pub comment_ids: Vec<String>,
    /// Implicit IDs read from the name or titles; they only count as orphans for known prefixes.
    pub name_ids: Vec<String>,
    /// Marked skip/todo/fixme (or inside a skipped group).
    pub skipped: bool,
    /// A property test: proptest (Rust), Hypothesis (Python) or fast-check (JS/TS).
    pub property: bool,
    /// Owning package path, when found through the project config.
    pub package: Option<String>,
    /// Suite runner, when found through the project config.
    pub suite: Option<Runner>,
    /// Last line of the test (closing brace, closing paren, or last indented line), 1-based.
    #[serde(skip)]
    pub end_line: usize,
    /// Marked focused (`.only`, `fit`, `fdescribe`), directly or through its group (JS/TS).
    #[serde(skip)]
    pub focused: bool,
    /// The group whose marker skips or focuses this test, as its titles joined with ` > `.
    #[serde(skip)]
    pub marker_group: Option<String>,
}

/// Find test functions (`fn` items carrying a `#[...test...]` attribute) in one Rust file.
/// An `#[ignore]` attribute marks the test skipped. A test inside a `proptest!` block or
/// carrying a `#[proptest]` attribute is a property test.
pub fn scan_rust(file: &Path, content: &str) -> Vec<TestFn> {
    static ATTR: OnceLock<Regex> = OnceLock::new();
    static FN: OnceLock<Regex> = OnceLock::new();
    static SPEC: OnceLock<Regex> = OnceLock::new();
    static PROPTEST_BLOCK: OnceLock<Regex> = OnceLock::new();
    static PROPTEST_ATTR: OnceLock<Regex> = OnceLock::new();
    let attr = ATTR.get_or_init(|| Regex::new(r"^\s*#\[").unwrap());
    let test_attr = |l: &str| attr.is_match(l) && l.contains("test") && !l.contains("cfg(test)");
    let fn_re = FN.get_or_init(|| {
        Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")
            .unwrap()
    });
    let spec_re = SPEC.get_or_init(|| {
        Regex::new(r"^\s*//+\s*spec:\s*([A-Z][A-Z0-9]*-[0-9]+(?:\s*,\s*[A-Z][A-Z0-9]*-[0-9]+)*)")
            .unwrap()
    });

    let proptest_block =
        PROPTEST_BLOCK.get_or_init(|| Regex::new(r"^\s*(?:proptest::)?proptest!").unwrap());
    let proptest_attr = PROPTEST_ATTR
        .get_or_init(|| Regex::new(r"^\s*#\[(?:[A-Za-z_][A-Za-z0-9_]*::)*proptest\b").unwrap());

    let mut out = Vec::new();
    // 1-based last line of the `proptest!` block being scanned, or 0 outside one.
    let mut block_end = 0;
    // State gathered from the run of attribute/comment/blank lines above the current line.
    let mut pending_test = false;
    let mut pending_ids: Vec<String> = Vec::new();
    let mut pending_skip = false;
    let mut pending_property = false;
    for (i, line) in content.lines().enumerate() {
        if proptest_block.is_match(line) {
            block_end = rust_body_end(content, i);
            continue;
        }
        if let Some(c) = spec_re.captures(line) {
            pending_ids.extend(c[1].split(',').map(|s| s.trim().to_string()));
            continue;
        }
        let t = line.trim();
        // Before `test_attr`: an ignore reason may contain the word "test".
        if t.strip_prefix("#[ignore")
            .and_then(|r| r.chars().next())
            .is_some_and(|c| matches!(c, ']' | '=' | '(') || c.is_whitespace())
        {
            pending_skip = true;
            continue;
        }
        if proptest_attr.is_match(line) {
            pending_test = true;
            pending_property = true;
            continue;
        }
        if test_attr(line) {
            pending_test = true;
            continue;
        }
        if t.is_empty() || t.starts_with("//") || attr.is_match(line) {
            continue;
        }
        if let Some(c) = fn_re.captures(line) {
            if pending_test {
                out.push(TestFn {
                    name: c[1].to_string(),
                    file: file.to_path_buf(),
                    line: i + 1,
                    end_line: rust_body_end(content, i),
                    comment_ids: std::mem::take(&mut pending_ids),
                    name_ids: ids_in_name(&c[1]),
                    skipped: pending_skip,
                    property: pending_property || i < block_end,
                    ..TestFn::default()
                });
            }
        }
        pending_test = false;
        pending_skip = false;
        pending_property = false;
        pending_ids.clear();
    }
    out
}

/// The 1-based line of the `}` closing the item whose header is on 0-based line `start`.
///
/// Lexes from the header to the end of the file, keeping string, raw string, char literal and
/// comment state across lines, so braces and quotes inside literals and comments are skipped.
/// Falls back to `start + 1`.
fn rust_body_end(content: &str, start: usize) -> usize {
    let from: usize = content
        .split_inclusive('\n')
        .take(start)
        .map(str::len)
        .sum();
    let b = &content.as_bytes()[from..];
    let ident = |c: u8| c == b'_' || c.is_ascii_alphanumeric() || c >= 0x80;
    let mut depth = 0usize;
    let mut opened = false;
    let mut j = 0;
    while j < b.len() {
        let c = b[j];
        match c {
            b'/' if b.get(j + 1) == Some(&b'/') => {
                while j < b.len() && b[j] != b'\n' {
                    j += 1;
                }
                continue;
            }
            b'/' if b.get(j + 1) == Some(&b'*') => {
                j = skip_block_comment(b, j);
                continue;
            }
            b'"' => {
                j = skip_string(b, j + 1);
                continue;
            }
            b'\'' => {
                j = skip_char_or_lifetime(b, j);
                continue;
            }
            b'{' => {
                depth += 1;
                opened = true;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                if opened && depth == 0 {
                    return start + 1 + b[..j].iter().filter(|&&x| x == b'\n').count();
                }
            }
            _ if ident(c) && (j == 0 || !ident(b[j - 1])) => {
                if let Some(end) = skip_raw_string(b, j) {
                    j = end;
                } else {
                    while j < b.len() && ident(b[j]) {
                        j += 1;
                    }
                }
                continue;
            }
            _ => {}
        }
        j += 1;
    }
    start + 1
}

/// Index just past the `"` closing a normal string whose body starts at `j`.
fn skip_string(b: &[u8], mut j: usize) -> usize {
    while j < b.len() {
        match b[j] {
            b'\\' => j += 2,
            b'"' => return j + 1,
            _ => j += 1,
        }
    }
    b.len()
}

/// When a raw string (`r"…"`, `r#"…"#`, `br…`, `cr…`) starts at `j`, the index just past it.
fn skip_raw_string(b: &[u8], j: usize) -> Option<usize> {
    let mut k = j;
    if matches!(b.get(k), Some(b'b' | b'c')) {
        k += 1;
    }
    if b.get(k) != Some(&b'r') {
        return None;
    }
    k += 1;
    let hashes = b[k..].iter().take_while(|&&x| x == b'#').count();
    k += hashes;
    if b.get(k) != Some(&b'"') {
        return None;
    }
    k += 1;
    while k < b.len() {
        if b[k] == b'"' && b[k + 1..].iter().take_while(|&&x| x == b'#').count() >= hashes {
            return Some(k + 1 + hashes);
        }
        k += 1;
    }
    Some(b.len())
}

/// Index past a char literal (`'{'`, `'\''`, `'\u{7f}'`) at `j`, or past the `'` of a lifetime.
fn skip_char_or_lifetime(b: &[u8], j: usize) -> usize {
    if b.get(j + 1) == Some(&b'\\') {
        // An escape: the closing quote follows within a few bytes (`\u{10FFFF}` is the longest).
        if let Some(n) = b.iter().skip(j + 3).take(10).position(|&x| x == b'\'') {
            return j + 3 + n + 1;
        }
        return j + 1;
    }
    let width = match b.get(j + 1) {
        Some(&x) if x < 0x80 => 1,
        Some(&x) if x >= 0xF0 => 4,
        Some(&x) if x >= 0xE0 => 3,
        Some(_) => 2,
        None => return j + 1,
    };
    if b.get(j + 1 + width) == Some(&b'\'') {
        j + 2 + width
    } else {
        j + 1
    }
}

/// Index just past the `*/` closing the (possibly nested) block comment opening at `j`.
fn skip_block_comment(b: &[u8], mut j: usize) -> usize {
    let mut depth = 0usize;
    while j < b.len() {
        if b[j] == b'/' && b.get(j + 1) == Some(&b'*') {
            depth += 1;
            j += 2;
        } else if b[j] == b'*' && b.get(j + 1) == Some(&b'/') {
            depth -= 1;
            j += 2;
            if depth == 0 {
                return j;
            }
        } else {
            j += 1;
        }
    }
    b.len()
}

/// Walk `root` for `*.rs` files (respecting .gitignore, skipping `target/` and `fixtures/`) and scan each.
pub fn scan_tree(root: &Path) -> Result<Vec<TestFn>> {
    let walker = ignore::WalkBuilder::new(root)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(|e| {
            e.depth() == 0
                || !(e.file_type().is_some_and(|t| t.is_dir())
                    && matches!(e.file_name().to_str(), Some("target" | "fixtures")))
        })
        .build();
    let mut out = Vec::new();
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type().is_some_and(|t| t.is_file())
            && path.extension().is_some_and(|x| x == "rs")
        {
            let content = std::fs::read_to_string(path)?;
            out.extend(scan_rust(path, &content));
        }
    }
    Ok(out)
}

/// Tests found through the project config, and the packages that have source files but no suite.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scan {
    pub tests: Vec<TestFn>,
    pub unsuited: Vec<String>,
}

/// Directories never searched for tests.
const SKIP_DIRS: [&str; 6] = [
    "target",
    "node_modules",
    ".venv",
    "venv",
    "fixtures",
    ".git",
];

/// One suite's compiled globs, with the literal prefix length of each for tie-breaking.
struct SuiteGlobs {
    runner: Runner,
    set: GlobSet,
    prefixes: Vec<usize>,
}

/// Walk `root` once and scan every file that a suite of its owning package matches.
///
/// A file belongs to the package with the longest matching path. Within that package, the suite
/// whose matching glob has the longest literal prefix wins; config order breaks ties.
pub fn scan_project(root: &Path, config: &Config) -> Result<Scan> {
    let placer = Placer::new(root, config)?;
    let mut scan = Scan::default();
    let mut has_source = vec![false; config.packages.len()];
    let walker = ignore::WalkBuilder::new(root)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(|e| {
            e.depth() == 0
                || !(e.file_type().is_some_and(|t| t.is_dir())
                    && e.file_name()
                        .to_str()
                        .is_some_and(|n| SKIP_DIRS.contains(&n)))
        })
        .build();
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        match placer.place(&rel) {
            Place::None => {}
            Place::Source(idx) => has_source[idx] = true,
            Place::Test(..) => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    scan.tests.extend(placer.scan(&rel, &content));
                }
            }
        }
    }
    scan.unsuited = config
        .packages
        .iter()
        .zip(has_source)
        .filter(|(_, has)| *has)
        .map(|(p, _)| p.path.clone())
        .collect();
    Ok(scan)
}

/// Scan listed repository-relative files (for example from a git tree) as [`scan_project`]
/// would, reading each matched file with `read`. Skipped directories are skipped here too.
pub fn scan_listed(
    root: &Path,
    config: &Config,
    paths: &[String],
    mut read: impl FnMut(&str) -> Result<String>,
) -> Result<Vec<TestFn>> {
    let placer = Placer::new(root, config)?;
    let mut tests = Vec::new();
    for rel in paths {
        if rel.split('/').rev().skip(1).any(|d| SKIP_DIRS.contains(&d)) {
            continue;
        }
        if let Place::Test(..) = placer.place(rel) {
            tests.extend(placer.scan(rel, &read(rel)?));
        }
    }
    Ok(tests)
}

enum Place {
    None,
    /// A file of package `idx` with no suite; counts towards `unsuited` when it is a source.
    Source(usize),
    Test(usize, Runner, fn(&Path, &str) -> Vec<TestFn>),
}

/// Compiled suite and source globs: which suite, if any, scans a file.
struct Placer<'a> {
    config: &'a Config,
    suites: Vec<Vec<SuiteGlobs>>,
    sources: Vec<GlobSet>,
}

impl<'a> Placer<'a> {
    fn new(root: &Path, config: &'a Config) -> Result<Placer<'a>> {
        let mut suites: Vec<Vec<SuiteGlobs>> = Vec::with_capacity(config.packages.len());
        let mut sources: Vec<GlobSet> = Vec::with_capacity(config.packages.len());
        for p in &config.packages {
            let mut v = Vec::new();
            for s in &p.suites {
                let globs = if s.tests.is_empty() {
                    default_globs(root, p, s.runner)
                } else {
                    s.tests.clone()
                };
                let (set, prefixes) = compile(&globs)?;
                v.push(SuiteGlobs {
                    runner: s.runner,
                    set,
                    prefixes,
                });
            }
            suites.push(v);
            sources.push(compile(&p.sources)?.0);
        }
        Ok(Placer {
            config,
            suites,
            sources,
        })
    }

    fn place(&self, rel: &str) -> Place {
        let Some(pkg) = self.config.package_for(rel) else {
            return Place::None;
        };
        let idx = self
            .config
            .packages
            .iter()
            .position(|p| std::ptr::eq(p, pkg))
            .unwrap_or_default();
        if self.suites[idx].is_empty() {
            return if self.sources[idx].is_match(rel) {
                Place::Source(idx)
            } else {
                Place::None
            };
        }
        let Some(runner) = pick_suite(&self.suites[idx], rel) else {
            return Place::None;
        };
        let scanner: fn(&Path, &str) -> Vec<TestFn> = match runner {
            Runner::Cargo if rel.ends_with(".rs") => scan_rust,
            Runner::Cargo => return Place::None,
            Runner::Vitest | Runner::Jest | Runner::Playwright => scan_js,
            Runner::Pytest if rel.ends_with(".py") => scan_python,
            Runner::Pytest => return Place::None,
            // A generic JUnit suite can hold any language: pick by extension.
            Runner::Junit => match rel.rsplit('.').next() {
                Some("rs") => scan_rust,
                Some("py") => scan_python,
                Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs") => scan_js,
                _ => return Place::None,
            },
        };
        Place::Test(idx, runner, scanner)
    }

    fn scan(&self, rel: &str, content: &str) -> Vec<TestFn> {
        let Place::Test(idx, runner, scanner) = self.place(rel) else {
            return Vec::new();
        };
        let pkg = &self.config.packages[idx];
        scanner(Path::new(rel), content)
            .into_iter()
            .map(|mut t| {
                t.package = Some(pkg.path.clone());
                t.suite = Some(runner);
                t
            })
            .collect()
    }
}

fn pick_suite(suites: &[SuiteGlobs], rel: &str) -> Option<Runner> {
    let mut best: Option<(usize, Runner)> = None;
    for s in suites {
        let longest = s.set.matches(rel).into_iter().map(|i| s.prefixes[i]).max();
        if let Some(len) = longest {
            if best.map_or(true, |(b, _)| len > b) {
                best = Some((len, s.runner));
            }
        }
    }
    best.map(|(_, r)| r)
}

/// A runner's default test globs under a package, relative to the repository root.
fn default_globs(root: &Path, pkg: &Package, runner: Runner) -> Vec<String> {
    let under = |g: &str| {
        if pkg.path == "." {
            g.to_string()
        } else {
            format!("{}/{g}", pkg.path)
        }
    };
    match runner {
        Runner::Cargo => vec![under("**/*.rs")],
        // No sensible default: a junit suite lists its test files.
        Runner::Junit => Vec::new(),
        Runner::Vitest | Runner::Jest => vec![under("**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}")],
        Runner::Pytest => vec![under("**/test_*.py"), under("**/*_test.py")],
        Runner::Playwright => {
            let test_dir = crate::adopt::config_file(root, &pkg.path, "playwright.config")
                .and_then(|f| std::fs::read_to_string(f).ok())
                .and_then(|t| crate::adopt::playwright_test_dir(&t))
                .unwrap_or_else(|| "e2e".into());
            vec![under(&format!("{test_dir}/**/*.spec.ts"))]
        }
    }
}

fn compile(globs: &[String]) -> Result<(GlobSet, Vec<usize>)> {
    let mut b = GlobSetBuilder::new();
    let mut prefixes = Vec::with_capacity(globs.len());
    for g in globs {
        b.add(GlobBuilder::new(g).literal_separator(true).build()?);
        prefixes.push(g.find(['*', '?', '[', '{']).unwrap_or(g.len()));
    }
    Ok((b.build()?, prefixes))
}

/// Candidate IDs embedded in a test name: an alphabetic segment followed by a numeric segment,
/// e.g. `test_auth_003_x` → `AUTH-003`.
pub fn ids_in_name(name: &str) -> Vec<String> {
    let segs: Vec<&str> = name.split('_').collect();
    segs.windows(2)
        .filter(|w| {
            !w[0].is_empty()
                && w[0].chars().all(|c| c.is_ascii_alphabetic())
                && !w[1].is_empty()
                && w[1].chars().all(|c| c.is_ascii_digit())
        })
        .map(|w| format!("{}-{}", w[0].to_ascii_uppercase(), w[1]))
        .collect()
}
