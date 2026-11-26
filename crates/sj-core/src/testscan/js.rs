//! vitest, Jest and Playwright test discovery from a small tokenizer.
//!
//! The tokenizer skips comments, strings, template literals (with `${…}` nesting) and regex
//! literals, so `it(` inside any of them is never a test. Calls are matched on the token list:
//! an identifier chain like `test.describe.serial` or `it.each(table)` followed by `(` and a
//! string-literal title.

use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::TestFn;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    /// A string literal; `plain` is false for a template literal with `${…}`.
    Str {
        text: String,
        plain: bool,
    },
    Punct(char),
    /// `=>`, kept apart from `>` so an arrow body counts as a call position.
    Arrow,
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    line: usize,
}

const TEST_ROOTS: [&str; 5] = ["it", "test", "fit", "xit", "xtest"];
const GROUP_ROOTS: [&str; 4] = ["describe", "fdescribe", "xdescribe", "suite"];
const MODIFIERS: [&str; 9] = [
    "only",
    "skip",
    "todo",
    "fixme",
    "concurrent",
    "fails",
    "serial",
    "parallel",
    "sequential",
];
/// Modifiers whose call returns the test function: the title is in the next call.
const CURRIED: [&str; 4] = ["each", "for", "skipIf", "runIf"];
const SKIPPING: [&str; 3] = ["skip", "todo", "fixme"];

/// Keywords after which `/` starts a regex literal.
const REGEX_AFTER: [&str; 12] = [
    "return", "typeof", "case", "do", "else", "in", "of", "new", "delete", "void", "throw", "await",
];

/// Find tests in one JS/TS file.
pub fn scan_js(file: &Path, content: &str) -> Vec<TestFn> {
    let toks = tokenize(content);
    let close = matching_parens(&toks);
    let lines: Vec<&str> = content.lines().collect();
    let fc_names = fast_check_names(&toks);

    struct Group {
        title: String,
        ids: Vec<String>,
        skipped: bool,
        focused: bool,
        end: usize,
    }
    let mut groups: Vec<Group> = Vec::new();
    let mut out = Vec::new();

    let mut i = 0;
    while i < toks.len() {
        while groups.last().is_some_and(|g| i > g.end) {
            groups.pop();
        }
        let Some(call) = match_call(&toks, &close, i) else {
            i += 1;
            continue;
        };
        let line = toks[i].line;
        let comment = spec_comment_ids(&lines, line);
        if call.group {
            groups.push(Group {
                title: call.title,
                ids: comment,
                skipped: call.skipped,
                focused: call.focused,
                end: call.end,
            });
        } else {
            let marker_group = groups.iter().position(|g| g.skipped || g.focused).map(|k| {
                groups[..=k]
                    .iter()
                    .map(|g| g.title.as_str())
                    .collect::<Vec<_>>()
                    .join(" > ")
            });
            let mut titles: Vec<&str> = groups.iter().map(|g| g.title.as_str()).collect();
            titles.push(&call.title);
            let mut comment_ids: Vec<String> = groups.iter().flat_map(|g| g.ids.clone()).collect();
            comment_ids.extend(comment);
            dedup(&mut comment_ids);
            let mut name_ids: Vec<String> = titles.iter().flat_map(|t| title_ids(t)).collect();
            dedup(&mut name_ids);
            out.push(TestFn {
                name: titles.join(" > "),
                file: file.to_path_buf(),
                line,
                comment_ids,
                name_ids,
                skipped: call.skipped || groups.iter().any(|g| g.skipped),
                end_line: toks[call.end].line,
                focused: call.focused || groups.iter().any(|g| g.focused),
                marker_group,
                property: calls_fast_check(&toks[call.body_start..call.end], &fc_names),
                ..TestFn::default()
            });
        }
        // Continue inside the call: describes hold tests, and a test body can hold nothing we
        // need, but scanning it is harmless.
        i = call.body_start;
    }
    out
}

struct Call {
    group: bool,
    title: String,
    skipped: bool,
    focused: bool,
    /// Index of the `)` that closes the call.
    end: usize,
    /// Index to resume scanning from.
    body_start: usize,
}

/// A test or group call starting at token `i`, if there is one.
fn match_call(toks: &[Token], close: &[Option<usize>], i: usize) -> Option<Call> {
    let Tok::Ident(root) = &toks[i].tok else {
        return None;
    };
    let is_test_root = TEST_ROOTS.contains(&root.as_str());
    let is_group_root = GROUP_ROOTS.contains(&root.as_str());
    if !is_test_root && !is_group_root {
        return None;
    }
    // Not `obj.it(`, and not JSX text right after a tag's `>`.
    if i > 0 && matches!(toks[i - 1].tok, Tok::Punct('.') | Tok::Punct('>')) {
        return None;
    }

    let mut chain: Vec<&str> = Vec::new();
    let mut j = i + 1;
    while let (Some(Tok::Punct('.')), Some(Tok::Ident(m))) =
        (toks.get(j).map(|t| &t.tok), toks.get(j + 1).map(|t| &t.tok))
    {
        chain.push(m);
        j += 2;
    }
    let mut group = is_group_root;
    let mut rest = chain.as_slice();
    if root == "test" && rest.first() == Some(&"describe") {
        group = true;
        rest = &rest[1..];
    }
    let (mods, curried) = match rest.last() {
        Some(last) if CURRIED.contains(last) => (&rest[..rest.len() - 1], true),
        _ => (rest, false),
    };
    if !mods.iter().all(|m| MODIFIERS.contains(m)) {
        return None;
    }
    let skipped = root.starts_with('x') || mods.iter().any(|m| SKIPPING.contains(m));
    let focused = root.starts_with('f') || mods.contains(&"only");

    if toks.get(j).map(|t| &t.tok) != Some(&Tok::Punct('(')) {
        return None;
    }
    if curried {
        // `it.each(table)(title, fn)`: skip the table call.
        let table_end = close.get(j).copied().flatten()?;
        j = table_end + 1;
        if toks.get(j).map(|t| &t.tok) != Some(&Tok::Punct('(')) {
            return None;
        }
    }
    let end = close.get(j).copied().flatten()?;
    match toks.get(j + 1).map(|t| &t.tok) {
        Some(Tok::Str { text, plain: true }) => Some(Call {
            group,
            title: text.clone(),
            skipped,
            focused,
            end,
            body_start: j + 2,
        }),
        _ => None,
    }
}

/// fast-check calls that make a test a property test.
const FAST_CHECK_CALLS: [&str; 3] = ["assert", "property", "asyncProperty"];

/// Names bound to `fast-check` by `import fc from`, `import fc, {…} from`, `import * as fc from`,
/// `import fc = require(…)` or `const fc = require(…)`.
fn fast_check_names(toks: &[Token]) -> Vec<String> {
    let is_fc = |k: usize| matches!(toks.get(k).map(|t| &t.tok), Some(Tok::Str { text, plain: true }) if text == "fast-check");
    let ident =
        |k: usize, w: &str| matches!(toks.get(k).map(|t| &t.tok), Some(Tok::Ident(x)) if x == w);
    let name = |k: usize| match toks.get(k).map(|t| &t.tok) {
        Some(Tok::Ident(x)) => Some(x.clone()),
        _ => None,
    };
    let punct = |k: usize, c: char| toks.get(k).map(|t| &t.tok) == Some(&Tok::Punct(c));
    let requires_fc = |k: usize| ident(k, "require") && punct(k + 1, '(') && is_fc(k + 2);
    // The import clause from `k` ends in `from 'fast-check'`.
    let from_fc = |k: usize| {
        (k..toks.len())
            .take_while(|&j| !punct(j, ';') && !ident(j, "import"))
            .find(|&j| ident(j, "from"))
            .is_some_and(|j| is_fc(j + 1))
    };
    let mut out = Vec::new();
    for i in 0..toks.len() {
        let bound = if ident(i, "import") {
            if punct(i + 1, '*') && ident(i + 2, "as") && from_fc(i + 4) {
                name(i + 3)
            } else if (punct(i + 2, '=') && requires_fc(i + 3))
                || ((ident(i + 2, "from") || punct(i + 2, ',')) && from_fc(i + 2))
            {
                name(i + 1)
            } else {
                None
            }
        } else if ["const", "let", "var"].iter().any(|k| ident(i, k))
            && punct(i + 2, '=')
            && requires_fc(i + 3)
        {
            name(i + 1)
        } else {
            None
        };
        out.extend(bound);
    }
    out
}

/// Whether `body` calls `<name>.assert(`, `.property(` or `.asyncProperty(` on a fast-check name.
fn calls_fast_check(body: &[Token], names: &[String]) -> bool {
    body.windows(4).enumerate().any(|(k, w)| {
        let after_dot = k > 0 && body[k - 1].tok == Tok::Punct('.');
        matches!(&w[0].tok, Tok::Ident(n) if names.contains(n))
            && !after_dot
            && w[1].tok == Tok::Punct('.')
            && matches!(&w[2].tok, Tok::Ident(m) if FAST_CHECK_CALLS.contains(&m.as_str()))
            && w[3].tok == Tok::Punct('(')
    })
}

/// For each `(`, the index of its matching `)`.
fn matching_parens(toks: &[Token]) -> Vec<Option<usize>> {
    let mut out = vec![None; toks.len()];
    let mut stack = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        match t.tok {
            Tok::Punct('(') => stack.push(i),
            Tok::Punct(')') => {
                if let Some(o) = stack.pop() {
                    out[o] = Some(i);
                }
            }
            _ => {}
        }
    }
    out
}

/// IDs in `// spec:` lines directly above 1-based `line`, skipping blank lines, other comments
/// and decorators.
fn spec_comment_ids(lines: &[&str], line: usize) -> Vec<String> {
    static SPEC: OnceLock<Regex> = OnceLock::new();
    let re = SPEC.get_or_init(|| {
        Regex::new(r"^\s*//+\s*spec:\s*([A-Z][A-Z0-9]*-[0-9]+(?:\s*,\s*[A-Z][A-Z0-9]*-[0-9]+)*)")
            .unwrap()
    });
    let mut found: Vec<Vec<String>> = Vec::new();
    let mut k = line.saturating_sub(1);
    while k > 0 {
        let l = lines.get(k - 1).copied().unwrap_or("");
        let t = l.trim();
        if let Some(c) = re.captures(l) {
            found.push(c[1].split(',').map(|s| s.trim().to_string()).collect());
        } else if !(t.is_empty()
            || t.starts_with("//")
            || t.starts_with("/*")
            || t.starts_with('*')
            || t.starts_with('@'))
        {
            break;
        }
        k -= 1;
    }
    found.into_iter().rev().flatten().collect()
}

fn title_ids(title: &str) -> Vec<String> {
    static ID: OnceLock<Regex> = OnceLock::new();
    let re = ID.get_or_init(|| Regex::new(r"\b[A-Z][A-Z0-9]*-[0-9]+\b").unwrap());
    re.find_iter(title)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn dedup(v: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    v.retain(|x| seen.insert(x.clone()));
}

fn tokenize(src: &str) -> Vec<Token> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks: Vec<Token> = Vec::new();
    let mut line = 1;
    let mut i = 0;
    // Brace depth at each open `${`, to know when a `}` resumes the template.
    let mut brace = 0usize;
    let mut tmpl: Vec<usize> = Vec::new();

    while i < chars.len() {
        let c = chars[i];
        match c {
            '\n' => {
                line += 1;
                i += 1;
            }
            c if c.is_whitespace() => i += 1,
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i += 2;
            }
            '/' if regex_allowed(toks.last()) => {
                i += 1;
                let mut class = false;
                while i < chars.len() && chars[i] != '\n' {
                    match chars[i] {
                        '\\' => i += 1,
                        '[' => class = true,
                        ']' => class = false,
                        '/' if !class => break,
                        _ => {}
                    }
                    i += 1;
                }
                i += 1;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                // A regex is an operand; record it so a following `/` is division.
                toks.push(Token {
                    tok: Tok::Str {
                        text: String::new(),
                        plain: false,
                    },
                    line,
                });
            }
            '\'' | '"' => {
                let start_line = line;
                let (text, next) = quoted(&chars, i + 1, c, &mut line);
                i = next;
                toks.push(Token {
                    tok: Tok::Str { text, plain: true },
                    line: start_line,
                });
            }
            '`' => {
                let start_line = line;
                let (text, plain, next, opened) = template(&chars, i + 1, &mut line);
                i = next;
                if opened {
                    tmpl.push(brace);
                    brace += 1;
                }
                toks.push(Token {
                    tok: Tok::Str { text, plain },
                    line: start_line,
                });
            }
            '}' if tmpl.last() == Some(&(brace.wrapping_sub(1))) && brace > 0 => {
                // End of a `${…}`: continue the template literal.
                tmpl.pop();
                brace -= 1;
                let (_, _, next, opened) = template(&chars, i + 1, &mut line);
                i = next;
                if opened {
                    tmpl.push(brace);
                    brace += 1;
                }
            }
            '=' if chars.get(i + 1) == Some(&'>') => {
                toks.push(Token {
                    tok: Tok::Arrow,
                    line,
                });
                i += 2;
            }
            c if c == '_' || c == '$' || c.is_alphabetic() => {
                let start = i;
                while i < chars.len()
                    && (chars[i] == '_' || chars[i] == '$' || chars[i].is_alphanumeric())
                {
                    i += 1;
                }
                toks.push(Token {
                    tok: Tok::Ident(chars[start..i].iter().collect()),
                    line,
                });
            }
            c if c.is_ascii_digit() => {
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '.') {
                    i += 1;
                }
                toks.push(Token {
                    tok: Tok::Ident(String::from("0")),
                    line,
                });
            }
            _ => {
                match c {
                    '{' => brace += 1,
                    '}' => brace = brace.saturating_sub(1),
                    _ => {}
                }
                toks.push(Token {
                    tok: Tok::Punct(c),
                    line,
                });
                i += 1;
            }
        }
    }
    toks
}

/// A `/` starts a regex when the previous token cannot end an operand.
fn regex_allowed(prev: Option<&Token>) -> bool {
    match prev.map(|t| &t.tok) {
        None | Some(Tok::Arrow) => true,
        Some(Tok::Punct(p)) => !matches!(p, ')' | ']' | '}'),
        Some(Tok::Ident(w)) => REGEX_AFTER.contains(&w.as_str()),
        Some(Tok::Str { .. }) => false,
    }
}

/// A single- or double-quoted string from `i` (after the quote). Stops at a newline, which a JS
/// string cannot contain, so a stray apostrophe in JSX text spoils at most one line.
/// A `\` before a newline continues the string; that newline is counted in `line`.
fn quoted(chars: &[char], mut i: usize, q: char, line: &mut usize) -> (String, usize) {
    let mut text = String::new();
    while i < chars.len() && chars[i] != q && chars[i] != '\n' {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 1;
            if chars[i] == '\n' {
                *line += 1;
            }
        }
        text.push(chars[i]);
        i += 1;
    }
    if chars.get(i) == Some(&q) {
        i += 1;
    }
    (text, i)
}

/// Template literal text from `i` up to the closing backtick or a `${`. Returns the text,
/// whether it had no `${`, the next index, and whether it stopped at a `${`.
fn template(chars: &[char], mut i: usize, line: &mut usize) -> (String, bool, usize, bool) {
    let mut text = String::new();
    while i < chars.len() {
        match chars[i] {
            '`' => return (text, true, i + 1, false),
            '$' if chars.get(i + 1) == Some(&'{') => return (text, false, i + 2, true),
            '\\' => {
                if let Some(&n) = chars.get(i + 1) {
                    text.push(n);
                }
                i += 2;
                continue;
            }
            '\n' => *line += 1,
            _ => {}
        }
        text.push(chars[i]);
        i += 1;
    }
    (text, true, i, false)
}
