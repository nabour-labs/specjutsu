//! The shrunk counterexample a property test printed when it failed: proptest, Hypothesis
//! and fast-check each name the minimal failing input in their failure text.

/// Longest counterexample kept, in characters; longer ones end in `…`.
const MAX_CHARS: usize = 200;

/// The counterexample in a property test's failure text, as one line of at most 200
/// characters plus `…`, or `None` when the text holds none.
pub fn counterexample(text: &str) -> Option<String> {
    let line = rest_of_line(text, "minimal failing input:")
        .and_then(one_line)
        .or_else(|| hypothesis(text).and_then(one_line))
        .or_else(|| rest_of_line(text, "Counterexample:").and_then(one_line))?;
    if line.chars().count() <= MAX_CHARS {
        return Some(line);
    }
    Some(line.chars().take(MAX_CHARS).chain(['…']).collect())
}

/// `raw` with whitespace runs turned into one space and a trailing comma dropped, or `None`
/// when nothing is left.
fn one_line(raw: &str) -> Option<String> {
    let line = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let line = line.strip_suffix(',').unwrap_or(&line).trim_end();
    (!line.is_empty()).then(|| line.to_string())
}

/// The text after `marker` up to the end of its line.
fn rest_of_line<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let start = text.find(marker)? + marker.len();
    text[start..].lines().next()
}

/// The arguments of Hypothesis's `Falsifying example: <name>(...)`, through the matching `)`.
/// Brackets inside '...' and "..." strings don't count; an unclosed example gives `None`.
fn hypothesis(text: &str) -> Option<&str> {
    const MARKER: &str = "Falsifying example:";
    let after = &text[text.find(MARKER)? + MARKER.len()..];
    let open = after.find('(')?;
    let name = after[..open].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        return None;
    }
    let args = &after[open + 1..];
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (i, c) in args.char_indices() {
        if let Some(q) = quote {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                _ if c == q => quote = None,
                _ => {}
            }
            continue;
        }
        match c {
            '\'' | '"' => quote = Some(c),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            ')' => return Some(&args[..i]),
            _ => {}
        }
    }
    None
}
