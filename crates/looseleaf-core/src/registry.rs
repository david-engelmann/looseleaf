//! Regenerating the claim registry, `claims.md` at the bundle root.
//!
//! The registry is a generated concept. Its format is owned by this
//! module outright. Rendering is deterministic. The same bundle state
//! with the same stamp produces byte-identical output, which is what
//! makes a bad regeneration visible in a diff and what lets the
//! checker treat a stale registry as a finding.
//!
//! A superseded claim is presented as withheld, with its replacement
//! named. The date and the reason come from the curated layer's
//! superseded claims section when the supersession has been promoted
//! there, because that section is where the synthesis contract puts
//! them. When it has not been promoted yet, the date falls back to the
//! daily file that carries the replacing claim, and the line carries
//! no reason rather than an invented one.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::PROFILE_NAME;
use crate::bundle::{Bundle, Layer};

/// One entry of a curated concept's superseded claims section, the
/// line shape the profile defines in section 3. `- ` then the old
/// identity, the old claim text, `Replaced by`, the replacing
/// identity, `on`, the date, and a one-line reason after a comma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupersededEntry {
    /// The identity of the superseded claim.
    pub old: String,
    /// The identity of the claim that replaced it.
    pub replacement: String,
    /// The date the supersession is recorded with.
    pub date: String,
    /// The one-line reason, when the entry carries one.
    pub reason: Option<String>,
}

/// Scan one curated body for the entries of its superseded claims
/// section. Only lines under a heading naming the superseded claims
/// section count, so a line elsewhere in the body that merely has the
/// same shape is not harvested as a record.
#[must_use]
pub fn section_entries_in(body: &str) -> Vec<SupersededEntry> {
    let mut entries = Vec::new();
    let mut in_section = false;
    for line in body.lines() {
        if line.starts_with('#') {
            in_section = line.to_lowercase().contains("superseded claims");
            continue;
        }
        if !in_section {
            continue;
        }
        let Some(rest) = line.strip_prefix("- ") else {
            continue;
        };
        let Some((old, tail)) = rest.split_once(' ') else {
            continue;
        };
        let Some((_, after)) = tail.split_once(" Replaced by ") else {
            continue;
        };
        let Some((replacement, rest)) = after.split_once(' ') else {
            continue;
        };
        let Some(dated) = rest.strip_prefix("on ") else {
            continue;
        };
        let (date, reason) = match dated.split_once(", ") {
            Some((date, reason)) => (
                date.trim().to_owned(),
                Some(reason.trim().trim_end_matches('.').to_owned()),
            ),
            None => (dated.trim().trim_end_matches('.').to_owned(), None),
        };
        entries.push(SupersededEntry {
            old: old.to_owned(),
            replacement: replacement.to_owned(),
            date,
            reason,
        });
    }
    entries
}

struct Supersession {
    date: Option<String>,
    reason: Option<String>,
}

/// The superseded claims sections of the whole curated layer, by old
/// identity. When two concepts record the same supersession the last
/// in path order wins, matching the scan's historical behaviour.
fn scan_superseded_sections(bundle: &Bundle) -> BTreeMap<String, Supersession> {
    let mut found: BTreeMap<String, Supersession> = BTreeMap::new();
    for file in &bundle.files {
        if file.layer != Layer::Curated || file.reserved {
            continue;
        }
        for entry in section_entries_in(&file.body) {
            found.insert(
                entry.old,
                Supersession {
                    date: Some(entry.date),
                    reason: entry.reason,
                },
            );
        }
    }
    found
}

/// The ISO date of the daily file carrying each daily claim, computed
/// once for the whole bundle so rendering stays linear in the number
/// of claims.
fn daily_dates(bundle: &Bundle) -> BTreeMap<String, String> {
    let mut dates: BTreeMap<String, String> = BTreeMap::new();
    for file in &bundle.files {
        if file.layer != Layer::Daily || file.reserved {
            continue;
        }
        let name = file.rel_path.rsplit('/').next().unwrap_or(&file.rel_path);
        let date = name.strip_suffix(".md").unwrap_or(name).to_owned();
        for claim in &file.claims {
            if let Some(id) = &claim.id {
                dates.entry(id.clone()).or_insert_with(|| date.clone());
            }
        }
    }
    dates
}

/// Escape a stamp for the double-quoted scalar the generated
/// frontmatter carries it in, so a stamp containing a quote or a
/// newline can never break out of the scalar and corrupt the file
/// the tool itself writes.
pub(crate) fn escape_quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

/// Render the full text of `claims.md` for a bundle, stamped with the
/// given `at` timestamp for the `generated` entry.
#[must_use]
pub fn render_registry(bundle: &Bundle, at: &str) -> String {
    let mut ids: Vec<String> = Vec::new();
    let mut replacements: BTreeMap<String, String> = BTreeMap::new();
    for file in &bundle.files {
        if !matches!(file.layer, Layer::Daily | Layer::Curated) || file.reserved {
            continue;
        }
        for claim in &file.claims {
            if let Some(id) = &claim.id {
                if !ids.contains(id) {
                    ids.push(id.clone());
                }
                if let Some(target) = &claim.supersedes {
                    // The first claimant in bundle order names the
                    // replacement. A fork, two claims superseding one
                    // target, is a checker finding, and the rendering
                    // stays deterministic while it is resolved.
                    replacements
                        .entry(target.clone())
                        .or_insert_with(|| id.clone());
                }
            }
        }
    }
    ids.sort();
    let sections = scan_superseded_sections(bundle);
    let dates = daily_dates(bundle);
    let mut lines: Vec<String> = Vec::new();
    for id in &ids {
        if let Some(replacement) = replacements.get(id) {
            let section = sections.get(id);
            let date = section
                .and_then(|s| s.date.clone())
                .or_else(|| dates.get(replacement).cloned());
            let reason = section.and_then(|s| s.reason.clone());
            let line = match (&date, &reason) {
                (Some(date), Some(reason)) => format!(
                    "- {id} withheld, superseded by {replacement} on {date}. Reason, {reason}."
                ),
                (Some(date), None) => {
                    format!("- {id} withheld, superseded by {replacement} on {date}.")
                }
                (None, Some(reason)) => {
                    format!("- {id} withheld, superseded by {replacement}. Reason, {reason}.")
                }
                (None, None) => format!("- {id} withheld, superseded by {replacement}."),
            };
            lines.push(line);
        } else {
            lines.push(format!("- {id} current."));
        }
    }
    let actor = format!("process:{PROFILE_NAME}");
    let stamp = escape_quoted(at);
    let mut out = format!(
        "---\ntype: Claim Registry\ntitle: \"Claim registry\"\ndescription: \"Generated index of every claim chain in the bundle.\"\ngenerated: {{ by: \"{actor}\", at: \"{stamp}\" }}\n---\n\n# Claim registry\n\n"
    );
    out.push_str(&lines.join("\n"));
    out.push('\n');
    out
}

/// Regenerate `claims.md` at the bundle root when its current bytes
/// differ from the rendering. The write goes to a sibling temporary
/// file and is renamed over the target, so a failed write never
/// truncates the registry that was there.
///
/// # Errors
///
/// Returns an I/O error when the existing registry cannot be read or
/// the replacement cannot be written.
pub fn regenerate_registry(root: &Path, at: &str) -> std::io::Result<bool> {
    let bundle = crate::bundle::read_bundle(root);
    let rendered = render_registry(&bundle, at);
    let target = root.join("claims.md");
    let current = match fs::read_to_string(&target) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if current.as_deref() == Some(rendered.as_str()) {
        return Ok(false);
    }
    crate::write::write_via_temp(&target, &rendered)?;
    Ok(true)
}
