//! `sj task done`: check one task's box, with proof in strict mode.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

use super::{
    classification, default_base, not_property, task_number, why_unproven_against_base, Ledger,
};
use crate::authoring::{self, blocks};
use crate::config::Mode;
use crate::gate;

/// An ID the task names that the ledger does not prove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unproven {
    pub id: String,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    AlreadyDone {
        text: String,
    },
    /// The box was checked; `warnings` are the unproven IDs (never strict mode).
    Done {
        text: String,
        warnings: Vec<Unproven>,
        /// `not-property` lines for FOR ALL scenarios with only example tests (LOOP-039).
        not_property: Vec<String>,
        /// Why the merge-base classification couldn't be read, when it couldn't.
        notes: Vec<String>,
    },
    /// Strict mode: nothing was written.
    Refused {
        unproven: Vec<Unproven>,
        not_property: Vec<String>,
        notes: Vec<String>,
    },
}

/// Byte offset of each line start in `raw`, treating `\r\n`, `\n` and a lone `\r` as line ends
/// (the same lines `authoring::read` sees after normalising).
fn line_starts(raw: &[u8]) -> Vec<usize> {
    let mut out = vec![0];
    let mut i = 0;
    while i < raw.len() {
        match raw[i] {
            b'\r' if raw.get(i + 1) == Some(&b'\n') => {
                out.push(i + 2);
                i += 2;
                continue;
            }
            b'\r' | b'\n' => out.push(i + 1),
            _ => {}
        }
        i += 1;
    }
    out
}

/// The classification and the gate's trace report, as `classify` returns them.
type Classified = (
    Option<BTreeMap<String, gate::Change>>,
    Option<crate::trace::Report>,
);

/// Each scenario's change against the merge-base with the default branch (never `--base`), as
/// the gate sees it, `None` without one or when the merge-base is HEAD (LOOP-036); and the
/// gate's trace report, `None` without a default branch.
fn classify(root: &Path, spec_root: &Path) -> Result<Classified> {
    let Some(base) = default_base(root, None) else {
        return Ok((None, None));
    };
    let config = crate::config::load(root)?;
    let rel = spec_root.strip_prefix(root).unwrap_or(spec_root);
    let prep = gate::prepare(root, rel, &config, &base)?;
    Ok((classification(root, &prep).cloned(), Some(prep.report)))
}

pub fn done(
    spec_root: &Path,
    root: &Path,
    change: &str,
    number: &str,
    mode: Mode,
) -> Result<Outcome> {
    let c = authoring::find_change(spec_root, change)?;
    let file = c.dir.join("tasks.md");
    let tasks = authoring::tasks(&file)?;
    let Some(task) = tasks.iter().find(|t| task_number(&t.text) == number) else {
        bail!("no task {number} in {}", file.display());
    };
    if task.done {
        return Ok(Outcome::AlreadyDone {
            text: task.text.clone(),
        });
    }

    let ledger = Ledger::load(root)?;
    let keep = crate::spec::keep_ids(spec_root)?;
    let ids = blocks::task_ids(&task.text);
    // The gate's classification, worked out only when the ledger alone leaves an ID unproven
    // or the task names a FOR ALL scenario (LOOP-039), whose tests are then scanned.
    // Unreadable, it falls back to the stricter ledger-only rule.
    let mut notes = Vec::new();
    let scenarios = crate::spec::load_scenarios(spec_root)?;
    let property = crate::spec::property_ids(&scenarios);
    let for_all = ids.iter().any(|id| property.contains(id));
    let (changes, report) = if for_all
        || ids
            .iter()
            .any(|id| why_unproven_against_base(&ledger, &keep, None, id).is_some())
    {
        classify(root, spec_root).unwrap_or_else(|e| {
            notes.push(format!(
                "cannot classify scenarios against the merge-base, so every one needs a red run \
                 and every FOR ALL one a property test: {e:#}"
            ));
            (None, None)
        })
    } else {
        (None, None)
    };
    let mut unproven = Vec::new();
    for id in &ids {
        if unproven.iter().any(|u: &Unproven| &u.id == id) {
            continue;
        }
        if let Some(why) = why_unproven_against_base(&ledger, &keep, changes.as_ref(), id) {
            unproven.push(Unproven {
                id: id.clone(),
                why,
            });
        }
    }
    let mut not_property_lines = Vec::new();
    if for_all {
        // The gate's report when classified; otherwise the tests are scanned here.
        let report = match report {
            Some(r) => r,
            None => {
                let config = crate::config::load(root)?;
                let scan = crate::testscan::scan_project(root, &config)?;
                crate::trace::build_report(&scenarios, &scan.tests)
            }
        };
        let mut seen = Vec::new();
        for id in &ids {
            if !seen.contains(&id) {
                seen.push(id);
                not_property_lines.extend(not_property(&report, changes.as_ref(), id));
            }
        }
    }
    if mode == Mode::Strict && !(unproven.is_empty() && not_property_lines.is_empty()) {
        return Ok(Outcome::Refused {
            unproven,
            not_property: not_property_lines,
            notes,
        });
    }

    let mut raw =
        std::fs::read(&file).with_context(|| format!("cannot read {}", file.display()))?;
    let starts = line_starts(&raw);
    let start = starts[task.line - 1];
    let end = starts.get(task.line).copied().unwrap_or(raw.len());
    let Some(at) = raw[start..end].windows(3).position(|w| w == b"[ ]") else {
        bail!("cannot find the box of task {number} in {}", file.display());
    };
    raw[start + at + 1] = b'x';
    std::fs::write(&file, &raw).with_context(|| format!("cannot write {}", file.display()))?;
    Ok(Outcome::Done {
        text: task.text.clone(),
        warnings: unproven,
        not_property: not_property_lines,
        notes,
    })
}
