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

use crate::bundle::{Bundle, Layer};

struct Supersession {
    replacement: String,
    date: Option<String>,
    reason: Option<String>,
}

/// Scan a curated body for superseded claims section entries of the
/// form the contract defines, one line per superseded claim carrying
/// the old identity, the replacing identity, the date and the reason.
fn scan_superseded_sections(bundle: &Bundle) -> BTreeMap<String, Supersession> {
    let mut found: BTreeMap<String, Supersession> = BTreeMap::new();
    for file in &bundle.files {
        if file.layer != Layer::Curated || file.reserved {
            continue;
        }
        for line in file.body.lines() {
            let Some(rest) = line.strip_prefix("- ") else {
                continue;
            };
            let Some((old, tail)) = rest.split_once(' ') else {
                continue;
            };
            let Some((_, after)) = tail.split_once(" Replaced by ") else {
                continue;
            };
            let Some((replacement, after)) = after.split_once(' ') else {
                continue;
            };
            let Some(reason_part) = after.strip_prefix("on ") else {
                continue;
            };
            let (date, reason) = match reason_part.split_once(", ") {
                Some((date, reason)) => (
                    Some(date.trim().to_owned()),
                    Some(reason.trim().trim_end_matches('.').to_owned()),
                ),
                None => (
                    Some(reason_part.trim().trim_end_matches('.').to_owned()),
                    None,
                ),
            };
            found.insert(
                old.to_owned(),
                Supersession {
                    replacement: replacement.to_owned(),
                    date,
                    reason,
                },
            );
        }
    }
    found
}

/// The ISO date of the daily file that carries a claim, when the
/// claim lives in the daily layer.
fn daily_date_of(bundle: &Bundle, claim_id: &str) -> Option<String> {
    for file in &bundle.files {
        if file.layer != Layer::Daily || file.reserved {
            continue;
        }
        if file
            .claims
            .iter()
            .any(|claim| claim.id.as_deref() == Some(claim_id))
        {
            let name = file.rel_path.rsplit('/').next().unwrap_or(&file.rel_path);
            return Some(name.strip_suffix(".md").unwrap_or(name).to_owned());
        }
    }
    None
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
                    replacements.insert(target.clone(), id.clone());
                }
            }
        }
    }
    ids.sort();
    let sections = scan_superseded_sections(bundle);
    let mut lines: Vec<String> = Vec::new();
    for id in &ids {
        if let Some(replacement) = replacements.get(id) {
            let section = sections.get(id);
            let date = section
                .and_then(|s| s.date.clone())
                .or_else(|| daily_date_of(bundle, replacement));
            let reason = section.and_then(|s| s.reason.clone());
            let mut line = format!("- {id} withheld, superseded by {replacement}");
            if let Some(date) = &date {
                line.push_str(&format!(" on {date}"));
            }
            line.push('.');
            if let Some(reason) = &reason {
                line.push_str(&format!(" Reason, {reason}."));
            }
            lines.push(line);
        } else {
            lines.push(format!("- {id} current."));
        }
    }
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("type: Claim Registry\n");
    out.push_str("title: \"Claim registry\"\n");
    out.push_str("description: \"Generated index of every claim chain in the bundle.\"\n");
    out.push_str(&format!(
        "generated: {{ by: \"process:looseleaf\", at: \"{at}\" }}\n"
    ));
    out.push_str("---\n\n# Claim registry\n\n");
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
    let temp = root.join("claims.md.tmp-looseleaf");
    fs::write(&temp, rendered)?;
    fs::rename(&temp, &target)?;
    Ok(true)
}
