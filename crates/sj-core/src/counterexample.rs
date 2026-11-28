//! The shrunk counterexample a property test printed when it failed: proptest, Hypothesis
//! and fast-check each name the minimal failing input in their failure text.

/// Longest counterexample kept, in characters; longer ones end in `…`.
const MAX_CHARS: usize = 200;

/// The counterexample in a property test's failure text, as one line of at most 200
/// characters plus `…`, or `None` when the text holds none.
pub fn counterexample(text: &str) -> Option<String> {
    let line = rest_of_line(text, "minimal failing input:")
        .and_then(one_line)
        .or_else(|| hypothesis(text).as_deref().and_then(one_line))
        .or_else(|| rest_of_line(text, "Counterexample:").and_then(one_line))?;
    if line.chars().count() <= MAX_CHARS {
        return Some(line);
    }
    let cut: String = line.chars().take(MAX_CHARS).collect();
    Some(format!("{}…", cut.trim_end()))
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

/// The arguments of Hypothesis's `Falsifying example: <name>(...)`, through the matching `)`,
/// without the `# ...` comments Hypothesis adds to explain them. Brackets inside '...' and
/// "..." strings or comments don't count; an unclosed example gives `None`.
fn hypothesis(text: &str) -> Option<String> {
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
    let mut out = String::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut comment = false;
    for c in args.chars() {
        if comment {
            if c == '\n' {
                comment = false;
                out.push(c);
            }
            continue;
        }
        if let Some(q) = quote {
            out.push(c);
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                _ if c == q => quote = None,
                _ => {}
            }
            continue;
        }
        match c {
            '#' => {
                comment = true;
                continue;
            }
            '\'' | '"' => quote = Some(c),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            ')' => return Some(out),
            _ => {}
        }
        out.push(c);
    }
    None
}
