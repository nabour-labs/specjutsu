//! Rust test discovery: test functions and their `// spec: ID` comments.

use std::path::{Path, PathBuf};

use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
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
pub fn scan_rust(file: &Path, content: &str) -> Vec<TestFn> {
    static ATTR: OnceLock<Regex> = OnceLock::new();
    static FN: OnceLock<Regex> = OnceLock::new();
    static SPEC: OnceLock<Regex> = OnceLock::new();
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

    let mut out = Vec::new();
    // State gathered from the run of attribute/comment/blank lines above the current line.
    let mut pending_test = false;
    let mut pending_ids: Vec<String> = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if let Some(c) = spec_re.captures(line) {
            pending_ids.extend(c[1].split(',').map(|s| s.trim().to_string()));
            continue;
        }
        if test_attr(line) {
            pending_test = true;
            continue;
        }
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || attr.is_match(line) {
            continue;
        }
        if let Some(c) = fn_re.captures(line) {
            if pending_test {
                out.push(TestFn {
                    name: c[1].to_string(),
                    file: file.to_path_buf(),
                    line: i + 1,
                    comment_ids: std::mem::take(&mut pending_ids),
                });
            }
        }
        pending_test = false;
        pending_ids.clear();
    }
    out
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
