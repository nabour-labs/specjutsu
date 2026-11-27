//! PR-facing output: the gate as a markdown comment (CI-001..003).

use crate::gate::{GateReport, Row};

/// First line of every gate comment; the action finds the comment to update by it.
pub const GATE_MARKER: &str = "<!-- specjutsu:gate -->";

/// GitHub's comment limit is 65,536 characters; stay well under it.
pub const MAX_BYTES: usize = 60_000;

/// Room kept for the truncation note.
const FOOTER_ROOM: usize = 200;

/// The one-line fix hint for a row's evidence, or empty when it needs none.
pub fn hint(row: &Row) -> String {
    match row.evidence.as_str() {
        "untested" => format!("add a test that names {}", row.id),
        "green-on-base" => {
            "the test passes without your change; make it assert the new behaviour".to_string()
        }
        "failing" => "make the test pass at head".to_string(),
        "not-run" => {
            "the runner did not report this test; check it compiles and is not ignored".to_string()
        }
        "not-run-on-base" => {
            "the test did not run on base; run `sj gate` locally to see why".to_string()
        }
        "unverifiable" => {
            "move the test where it can run on base, or pass --allow-unverifiable".to_string()
        }
        "keep-red-on-base" => {
            "the test fails on base, so this is new behaviour; drop [keep] from the scenario"
                .to_string()
        }
        "keep-broken" => "the change broke kept behaviour; make the test pass at head".to_string(),
        _ => String::new(),
    }
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn table_row(row: &Row) -> String {
    let tests = if row.tests.is_empty() {
        "—".to_string()
    } else {
        row.tests
            .iter()
            .map(|t| format!("`{}`", cell(t)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let evidence = match &row.counterexample {
        Some(input) => format!("{} (counterexample: {})", row.evidence, input),
        None => row.evidence.clone(),
    };
    format!(
        "| {} | {} | {} | {} |",
        row.id,
        cell(&evidence),
        tests,
        cell(&hint(row))
    )
}

/// Failures that belong to no scenario row (unspecified changes, orphans, exemptions, …).
fn other_failures(report: &GateReport) -> Vec<String> {
    report
        .failures
        .iter()
        .filter(|f| {
            !report
                .rows
                .iter()
                .any(|r| f.split_whitespace().nth(1) == Some(&r.id))
        })
        .cloned()
        .collect()
}

/// Render the gate result as a PR comment body of at most [`MAX_BYTES`] bytes. `base` is the
/// ref the gate compared against, used in the truncation note.
pub fn render_markdown(report: &GateReport, base: &str) -> String {
    let icon = if report.pass { "✅" } else { "❌" };
    let mut out = format!("{GATE_MARKER}\n### {icon} {}\n", report.summary);

    // Items in order of importance; each is one line. Section headers are emitted lazily.
    enum Item {
        Row(String),
        Kept(String),
        Linked(String),
        Other(String),
        Warn(String),
        Exempt(String),
    }
    // GATE-043: passing keep scenarios get their own section; failing ones stay in the table
    // with their fix hint.
    let (kept, rows): (Vec<&Row>, Vec<&Row>) =
        report.rows.iter().partition(|r| r.keep && !r.failed);
    // GATE-059: so do passing linked scenarios.
    let (linked, mut rows): (Vec<&Row>, Vec<&Row>) =
        rows.into_iter().partition(|r| r.linked && !r.failed);
    rows.sort_by_key(|r| !r.failed); // failures first, order otherwise kept
    let line = |r: &Row| {
        let tests: Vec<String> = r.tests.iter().map(|t| format!("`{}`", cell(t))).collect();
        format!("- {} {} ({})", r.id, r.evidence, tests.join(", "))
    };
    let mut items: Vec<Item> = rows.iter().map(|r| Item::Row(table_row(r))).collect();
    items.extend(kept.iter().map(|r| Item::Kept(line(r))));
    items.extend(linked.iter().map(|r| Item::Linked(line(r))));
    items.extend(other_failures(report).into_iter().map(Item::Other));
    items.extend(report.warnings.iter().cloned().map(Item::Warn));
    items.extend(report.exemptions.iter().cloned().map(Item::Exempt));

    let budget = MAX_BYTES - FOOTER_ROOM;
    let mut section = "";
    let mut omitted = 0usize;
    for item in &items {
        let (name, header, line) = match item {
            Item::Row(l) => (
                "rows",
                "\n| Scenario | Evidence | Test | Fix |\n| --- | --- | --- | --- |\n",
                l.clone(),
            ),
            Item::Kept(l) => ("kept", "\n**Kept behaviour**\n\n", l.clone()),
            Item::Linked(l) => ("linked", "\n**Linked existing behaviour**\n\n", l.clone()),
            Item::Other(l) => (
                "other",
                "\n**Other problems**\n\n",
                format!("- {}", cell(l)),
            ),
            Item::Warn(l) => ("warn", "\n**Warnings**\n\n", format!("- {}", cell(l))),
            Item::Exempt(l) => ("exempt", "\n**Exemptions**\n\n", format!("- {}", cell(l))),
        };
        let head = if section == name { "" } else { header };
        if omitted > 0 || out.len() + head.len() + line.len() + 1 > budget {
            omitted += 1;
            continue;
        }
        out.push_str(head);
        section = name;
        out.push_str(&line);
        out.push('\n');
    }
    if omitted > 0 {
        let what = if omitted == 1 { "row" } else { "rows" };
        out.push_str(&format!(
            "\n_{omitted} more {what} not shown; run `sj gate --base {base}` locally for the full list._\n"
        ));
    }
    out
}

/// A shields.io endpoint badge (CI-004..006).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub schema_version: u8,
    pub label: String,
    pub message: String,
    pub color: String,
}

/// Count the scenarios under `<spec_root>/specs/` and those with at least one mapped test that
/// is not skipped. Scenarios in change folders are not counted.
pub fn badge(
    spec_root: &std::path::Path,
    scenarios: &[crate::spec::Scenario],
    report: &crate::trace::Report,
) -> Badge {
    let specs = spec_root.join("specs");
    let ids: Vec<&str> = scenarios
        .iter()
        .filter(|s| s.file.starts_with(&specs))
        .filter_map(|s| s.id.as_deref())
        .collect();
    let total = ids.len();
    let tested = ids
        .iter()
        .filter(|id| {
            report
                .mapped
                .iter()
                .any(|m| m.id == **id && m.tests.iter().any(|t| !t.skipped))
        })
        .count();
    let (message, color) = match (tested * 100).checked_div(total) {
        None => ("no specs".to_string(), "lightgrey"),
        Some(percent) => {
            let color = match percent {
                90.. => "brightgreen",
                70.. => "yellow",
                _ => "red",
            };
            (format!("{tested}/{total} tested"), color)
        }
    };
    Badge {
        schema_version: 1,
        label: "specs".to_string(),
        message,
        color: color.to_string(),
    }
}

/// First line of every audit comment.
pub const AUDIT_MARKER: &str = "<!-- specjutsu:audit -->";

/// The one-line fix hint for an audit finding's kind.
pub fn audit_hint(kind: &str) -> &'static str {
    match kind {
        "removed-test" => {
            "restore the test, or allow the removal with a `Specjutsu-Allow:` trailer and a reason"
        }
        "skipped-test" => {
            "remove the skip marker, or allow it with a `Specjutsu-Allow:` trailer and a reason"
        }
        "focused-test" => "remove .only; it stops every other test in the file from running",
        "weakened-test" => "restore the assertions the test lost, or allow it with a reason",
        "assertion-free-test" => "assert the behaviour the test exercises",
        "rewritten-expectation" => {
            "check the new expected value comes from the spec, not from the code's output"
        }
        "unknown-package" => {
            "no such package on the registry; check the name before installing anything"
        }
        "new-package" => "first published recently; check it is the package you meant",
        "allow without reason" => {
            "add a reason after a dash: `Specjutsu-Allow: <kind> <target> — <reason>`"
        }
        _ => "",
    }
}

/// Render an audit result as a PR comment body of at most [`MAX_BYTES`] bytes.
pub fn render_audit_markdown(report: &crate::audit::AuditReport, base: &str) -> String {
    use crate::audit::Severity;
    let icon = if report.pass { "✅" } else { "❌" };
    let mut out = format!("{AUDIT_MARKER}\n### {icon} {}\n", report.summary);
    if report.findings.is_empty() {
        return out;
    }
    out.push_str("\n| | Finding | Test or package | Fix |\n| --- | --- | --- | --- |\n");
    let mut findings: Vec<_> = report.findings.iter().collect();
    findings.sort_by_key(|f| match f.severity {
        Severity::Fail => 0,
        Severity::Warn => 1,
        Severity::Allowed => 2,
    });
    let budget = MAX_BYTES - FOOTER_ROOM;
    let mut omitted = 0usize;
    for f in findings {
        let sev = match f.severity {
            Severity::Fail => "fail",
            Severity::Warn => "warn",
            Severity::Allowed => "allowed",
        };
        let mut what = f.target.clone();
        if !f.detail.is_empty() {
            what.push_str(if f.detail.starts_with(':') { "" } else { " " });
            what.push_str(&f.detail);
        }
        if f.severity == Severity::Allowed {
            what.push_str(&format!(" ({})", f.reason));
        }
        let line = format!(
            "| {sev} | {} | {} | {} |\n",
            cell(&f.kind),
            cell(&what),
            cell(audit_hint(&f.kind))
        );
        if omitted > 0 || out.len() + line.len() > budget {
            omitted += 1;
            continue;
        }
        out.push_str(&line);
    }
    if omitted > 0 {
        out.push_str(&format!(
            "\n_{omitted} more findings not shown; run `sj audit --base {base}` locally for the full list._\n"
        ));
    }
    out
}
