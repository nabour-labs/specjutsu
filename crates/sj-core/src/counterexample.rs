//! The shrunk counterexample a property test printed when it failed: proptest, Hypothesis
//! and fast-check each name the minimal failing input in their failure text.

/// Longest counterexample kept, in characters; longer ones end in `…`.
const MAX_CHARS: usize = 200;

/// The counterexample in a property test's failure text, as one line of at most 200
/// characters plus `…`, or `None` when the text holds none.
pub fn counterexample(text: &str) -> Option<String> {
    let raw = rest_of_line(text, "minimal failing input:")
        .or_else(|| hypothesis(text))
        .or_else(|| rest_of_line(text, "Counterexample:"))?;
    let line = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let line = line.strip_suffix(',').unwrap_or(&line).trim_end();
    if line.is_empty() {
        return None;
    }
    if line.chars().count() <= MAX_CHARS {
        return Some(line.to_string());
    }
    Some(line.chars().take(MAX_CHARS).chain(['…']).collect())
}

/// The text after `marker` up to the end of its line.
fn rest_of_line<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let start = text.find(marker)? + marker.len();
    text[start..].lines().next()
}

/// The arguments of Hypothesis's `Falsifying example: <name>(...)`, through the matching `)`.
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
    for (i, c) in args.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            ')' => return Some(&args[..i]),
            _ => {}
        }
    }
    None
}
