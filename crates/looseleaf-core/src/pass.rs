//! The synthesis pass mechanics, the deterministic steps inside
//! one pass that a machine can do without judgment.
//!
//! Three steps live here. Staleness computation reports which
//! concepts' `stale_after` has passed relative to a stamp the
//! caller supplies, as a list and never an edit, because acting on
//! staleness is a contract judgment. Promotion copies one claim
//! from a daily file into a named curated concept with its
//! identity, quote and sources unchanged, and refuses when the
//! curated layer already carries the identity with different
//! wording, which is the drift the checker flags. When a promotion
//! carries a supersedes link, the superseded claims section of the
//! target concept gains the entry in the exact line shape the
//! registry scanner reads, so the registry's date and reason have
//! their contract home. No step reads a clock. Every date comes
//! from an argument or from the daily file's own name.

use std::fs;
use std::path::Path;

use crate::bundle::{Bundle, Layer, read_bundle};
use crate::check::is_calendar_date;
use crate::claim::{Claim, scan_claims};
use crate::frontmatter::Value;
use crate::indexes::frontmatter_span;
use crate::registry::section_entries_in;
use crate::write::render_scalar;

fn invalid(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

fn not_found(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::NotFound, message)
}

/// One concept whose `stale_after` has passed at the given stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleEntry {
    /// The bundle-relative path of the concept.
    pub path: String,
    /// The concept's `stale_after` date.
    pub stale_after: String,
    /// The newest `at` among the concept's verified entries, when
    /// it carries any.
    pub verified_at: Option<String>,
}

/// The date part of a stamp or a date value, when its first ten
/// characters form a real calendar date.
fn date_part(value: &str) -> Option<&str> {
    let part = value.get(..10)?;
    if is_calendar_date(part) {
        Some(part)
    } else {
        None
    }
}

/// The `at` stamp of one verification record, when the value is a
/// mapping carrying one.
fn verified_at_of(value: &Value) -> Option<&str> {
    value.as_map()?.iter().find_map(
        |(key, entry)| {
            if key == "at" { entry.as_scalar() } else { None }
        },
    )
}

/// The newest verification stamp a concept carries, across a
/// `verified` value written as one mapping or as a list of them.
fn newest_verified(frontmatter: &crate::frontmatter::Frontmatter) -> Option<String> {
    let value = frontmatter.get("verified")?;
    let mut newest: Option<&str> = None;
    match value {
        Value::List(items) => {
            for item in items {
                if let Some(at) = verified_at_of(item) {
                    newest = Some(newest.map_or(at, |current| current.max(at)));
                }
            }
        }
        other => newest = verified_at_of(other),
    }
    newest.map(ToOwned::to_owned)
}

/// Report the concepts in the bundle whose `stale_after` date has
/// passed at the given stamp, in path order. A concept is stale
/// when its `stale_after` date is earlier than the stamp's date.
/// The stamp is the caller's, an RFC 3339 timestamp or a plain
/// date, never a clock reading.
///
/// # Errors
///
/// Returns an `InvalidInput` error when the stamp carries no real
/// calendar date.
pub fn staleness_report(root: &Path, at: &str) -> std::io::Result<Vec<StaleEntry>> {
    let Some(stamp) = date_part(at) else {
        return Err(invalid(format!(
            "the stamp {at} carries no real calendar date"
        )));
    };
    let bundle = read_bundle(root);
    let mut stale: Vec<StaleEntry> = Vec::new();
    for file in &bundle.files {
        if file.reserved {
            continue;
        }
        let Some(frontmatter) = &file.frontmatter else {
            continue;
        };
        let Some(raw) = frontmatter.get_scalar("stale_after") else {
            continue;
        };
        let Some(date) = date_part(raw.trim()) else {
            continue;
        };
        if date < stamp {
            stale.push(StaleEntry {
                path: file.rel_path.clone(),
                stale_after: date.to_owned(),
                verified_at: newest_verified(frontmatter),
            });
        }
    }
    Ok(stale)
}

/// How a promotion ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Promotion {
    /// The claim block and any supersession entry were written.
    Promoted,
    /// The target already carried the claim unchanged, so nothing
    /// was written.
    AlreadyPresent,
}

/// Find a claim by identity in one layer of the bundle, returning
/// the claim and the relative path of the file that carries it.
fn find_claim(bundle: &Bundle, layer: Layer, id: &str) -> Option<(Claim, String)> {
    for file in &bundle.files {
        if file.layer != layer || file.reserved {
            continue;
        }
        for claim in scan_claims(&file.body) {
            if claim.id.as_deref() == Some(id) {
                return Some((claim, file.rel_path.clone()));
            }
        }
    }
    None
}

/// Render a verbatim quote as a double-quoted scalar, escaping
/// backslashes and double quotes. Quote sub-items always carry
/// the quoted form, in the daily files the product converts and
/// in the curated layer alike, because a quote is an excerpt
/// whose exact wording is the evidence.
fn render_quote(text: &str) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Render one claim block in the grammar the scanner reads, with
/// the identity, quote and sources carried unchanged.
fn render_claim_block(claim: &Claim) -> Vec<String> {
    let mut lines = vec![format!(
        "- [{}|{}] {}",
        claim.kind, claim.confidence, claim.text
    )];
    if let Some(id) = &claim.id {
        lines.push(format!("  - claim: {}", render_scalar(id)));
    }
    if let Some(quote) = &claim.quote {
        lines.push(format!("  - quote: {}", render_quote(quote)));
    }
    if !claim.sources.is_empty() {
        let joined = claim
            .sources
            .iter()
            .map(|source| render_scalar(source))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("  - sources: {joined}"));
    }
    if let Some(supersedes) = &claim.supersedes {
        lines.push(format!("  - supersedes: {}", render_scalar(supersedes)));
    }
    lines
}

/// Insert lines into a body after the line at the given zero-based
/// index, keeping the document's own line ending and exactly one
/// trailing newline.
fn insert_after(body: &str, at: usize, new_lines: &[String]) -> String {
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = body.lines().collect();
    let mut out: Vec<String> = lines[..=at].iter().map(ToString::to_string).collect();
    out.extend(new_lines.iter().cloned());
    out.extend(lines[at + 1..].iter().map(ToString::to_string));
    let mut text = out.join(eol);
    text.push_str(eol);
    text
}

/// The line index a new claim block is inserted after, the last
/// line of the body's last claim block, or the body's last line
/// when the body carries no claim blocks.
fn block_insert_point(body: &str) -> usize {
    let claims = scan_claims(body);
    let lines: Vec<&str> = body.lines().collect();
    let Some(last) = claims.last() else {
        return lines.len().saturating_sub(1);
    };
    let mut at = last.line - 1;
    while at + 1 < lines.len() && lines[at + 1].starts_with("  - ") {
        at += 1;
    }
    at
}

/// Append the superseded claims section entry for one supersession
/// to a body, creating the section at the end of the body when it
/// has none. Returns the new body, or the body unchanged when the
/// section already records this supersession for this replacement.
fn append_section_entry(
    body: &str,
    old: &str,
    old_text: &str,
    replacement: &str,
    date: &str,
    reason: &str,
) -> Result<String, std::io::Error> {
    for entry in section_entries_in(body) {
        if entry.old == old {
            if entry.replacement == replacement {
                return Ok(body.to_owned());
            }
            return Err(invalid(format!(
                "the superseded claims section already records {old} replaced by {}",
                entry.replacement
            )));
        }
    }
    let reason = reason.trim().trim_end_matches('.');
    let line = format!("- {old} {old_text} Replaced by {replacement} on {date}, {reason}.");
    let lines: Vec<&str> = body.lines().collect();
    let section = lines.iter().position(|line| {
        line.starts_with('#') && line.to_lowercase().contains("superseded claims")
    });
    if let Some(heading) = section {
        let end = lines[heading + 1..]
            .iter()
            .position(|line| line.starts_with('#'))
            .map_or(lines.len(), |at| heading + 1 + at);
        let last_content = (heading + 1..end)
            .rev()
            .find(|&at| !lines[at].trim().is_empty())
            .unwrap_or(heading);
        return Ok(insert_after(body, last_content, &[line]));
    }
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let mut text = body.trim_end_matches(['\r', '\n']).to_owned();
    text.push_str(eol);
    text.push_str(eol);
    text.push_str("## Superseded claims");
    text.push_str(eol);
    text.push_str(eol);
    text.push_str(&line);
    text.push_str(eol);
    Ok(text)
}

/// Promote one claim from the daily layer into a named curated
/// concept. The `target` is the curated concept's bundle-relative
/// path. When the claim carries a supersedes link, `reason` is
/// required and the target's superseded claims section gains the
/// entry, dated by the daily file the claim lives in. The daily
/// file is never touched, because the daily layer is append-only.
///
/// # Errors
///
/// Returns a `NotFound` error when the claim, the claim it
/// supersedes, or the target concept cannot be found, and an
/// `InvalidInput` error when the target is not a curated concept,
/// when the curated layer already carries the identity with
/// different wording, when the claim carries a reverse link, when
/// a superseding promotion is given no reason, or when the
/// section already records the supersession for another
/// replacement. Returns an I/O error when the target cannot be
/// read or written.
pub fn promote_claim(
    root: &Path,
    claim_id: &str,
    target: &str,
    reason: Option<&str>,
) -> std::io::Result<Promotion> {
    let bundle = read_bundle(root);
    let Some((claim, daily_rel)) = find_claim(&bundle, Layer::Daily, claim_id) else {
        return Err(not_found(format!(
            "no daily claim carries the identity {claim_id}"
        )));
    };
    if claim.superseded_by.is_some() {
        return Err(invalid(format!(
            "the daily claim {claim_id} carries a superseded-by reverse link, which the daily layer never holds"
        )));
    }
    let Some(target_file) = bundle.files.iter().find(|file| file.rel_path == target) else {
        return Err(not_found(format!(
            "the bundle carries no concept at {target}"
        )));
    };
    if target_file.layer != Layer::Curated || target_file.reserved {
        return Err(invalid(format!(
            "{target} is not a curated concept, and promotion lands only in the curated layer"
        )));
    }
    if let Some(existing) = scan_claims(&target_file.body)
        .into_iter()
        .find(|carried| carried.id.as_deref() == Some(claim_id))
    {
        let mut drift: Vec<&str> = Vec::new();
        if existing.text != claim.text {
            drift.push("text");
        }
        if existing.quote != claim.quote {
            drift.push("quote");
        }
        if existing.sources != claim.sources {
            drift.push("sources");
        }
        if existing.supersedes != claim.supersedes {
            drift.push("supersedes");
        }
        if !drift.is_empty() {
            return Err(invalid(format!(
                "the curated layer already carries {claim_id} with different {}, which is drift, and promotion refuses to overwrite it",
                drift.join(", ")
            )));
        }
        // The claim itself is carried unchanged. A supersession
        // entry the section is missing is still owed, so an
        // interrupted earlier promotion completes here rather than
        // reporting itself done.
        if let Some(old) = &claim.supersedes {
            let Some(reason) = reason else {
                return Err(invalid(format!(
                    "the claim {claim_id} supersedes {old}, and a superseding promotion needs its one-line reason"
                )));
            };
            let completed =
                complete_section_entry(&bundle, &target_file.body, &claim, &daily_rel, reason)?;
            if completed != target_file.body {
                write_body(root, target, &completed)?;
                return Ok(Promotion::Promoted);
            }
        }
        return Ok(Promotion::AlreadyPresent);
    }
    let mut body = append_claim_block(&target_file.body, &claim);
    if claim.supersedes.is_some() {
        let Some(reason) = reason else {
            return Err(invalid(format!(
                "the claim {claim_id} supersedes {}, and a superseding promotion needs its one-line reason",
                claim.supersedes.as_deref().unwrap_or_default()
            )));
        };
        body = complete_section_entry(&bundle, &body, &claim, &daily_rel, reason)?;
    }
    write_body(root, target, &body)?;
    Ok(Promotion::Promoted)
}

/// Give a body the superseded claims section entry for the
/// supersession a claim carries, dated by the daily file the
/// claim lives in. The superseded claim's own text is read from
/// wherever in the bundle it lives, daily first, then curated.
fn complete_section_entry(
    bundle: &Bundle,
    body: &str,
    claim: &Claim,
    daily_rel: &str,
    reason: &str,
) -> std::io::Result<String> {
    let Some(old) = &claim.supersedes else {
        return Ok(body.to_owned());
    };
    let Some(new_id) = &claim.id else {
        return Ok(body.to_owned());
    };
    let old_claim =
        find_claim(bundle, Layer::Daily, old).or_else(|| find_claim(bundle, Layer::Curated, old));
    let Some((old_claim, _)) = old_claim else {
        return Err(not_found(format!(
            "the superseded claim {old} is not in the bundle, so its section entry cannot be written"
        )));
    };
    let date = daily_rel
        .rsplit('/')
        .next()
        .unwrap_or(daily_rel)
        .strip_suffix(".md")
        .unwrap_or(daily_rel)
        .to_owned();
    append_section_entry(body, old, &old_claim.text, new_id, &date, reason)
}

/// Write a concept's new body under its existing frontmatter,
/// which is carried byte for byte, through a sibling temporary
/// file and a rename.
fn write_body(root: &Path, target: &str, body: &str) -> std::io::Result<()> {
    let target_path = root.join(target);
    let text = fs::read_to_string(&target_path)?;
    let Some((span, _)) = frontmatter_span(&text) else {
        return Err(invalid(format!(
            "the target {target} carries no frontmatter to preserve"
        )));
    };
    let mut out = String::from(span);
    out.push_str(body);
    let temp = target_path.with_extension("md.tmp-looseleaf");
    fs::write(&temp, out)?;
    fs::rename(&temp, &target_path)?;
    Ok(())
}

/// Append one claim block to a body after its last claim block,
/// or at the end of the body when it carries none.
fn append_claim_block(body: &str, claim: &Claim) -> String {
    let block = render_claim_block(claim);
    if scan_claims(body).is_empty() {
        let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
        let mut text = body.trim_end_matches(['\r', '\n']).to_owned();
        if !text.is_empty() {
            text.push_str(eol);
            text.push_str(eol);
        }
        text.push_str(&block.join(eol));
        text.push_str(eol);
        return text;
    }
    insert_after(body, block_insert_point(body), &block)
}
