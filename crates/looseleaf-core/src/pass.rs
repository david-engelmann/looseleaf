//! The synthesis pass mechanics, the deterministic steps inside
//! one pass that a machine can do without judgment.
//!
//! Four steps live here. Staleness computation reports which
//! concepts' `stale_after` has passed relative to a stamp the
//! caller supplies, as a list and never an edit, because acting on
//! staleness is a contract judgment. The re-verification queue
//! validates and orders the concepts a pass created or changed in
//! meaning, the set the contract hands on for re-verification,
//! likewise as a list and never an edit. Promotion copies one claim
//! from a daily file into a named curated concept with its
//! identity, quote and sources unchanged, and refuses when the
//! curated layer already carries the identity anywhere, identically
//! in another concept or with different wording in any concept,
//! which is the drift and duplication the checker flags. When a
//! promotion carries a supersedes link, the superseded claims
//! section entry goes to the concept that holds the superseded
//! claim, or to the target when the superseded claim was never
//! promoted, and the superseded claim's curated copy gains the
//! `superseded-by` reverse link the profile assigns to this
//! moment. No step reads a clock. Every date comes from an
//! argument or from the daily file's own name.

use std::fs;
use std::path::Path;

use crate::bundle::{Bundle, Layer, read_bundle};
use crate::check::is_calendar_date;
use crate::claim::{Claim, scan_claims};
use crate::frontmatter::{Frontmatter, Value};
use crate::indexes::frontmatter_span;
use crate::registry::{escape_quoted, section_entries_in};
use crate::write::{invalid_input as invalid, line_ending, not_found, render_scalar};

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
fn newest_verified(frontmatter: &Frontmatter) -> Option<String> {
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
/// date, never a clock reading. Only files inside the bundle's
/// layers are concepts, so a root-level file is never reported.
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
        if file.reserved || file.layer == Layer::Root {
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

/// Assemble the re-verification queue a pass hands on, the
/// concepts the pass created or changed in meaning in the same
/// pass, supplied by the caller in pass order. Each path is
/// resolved through the bundle walker and refused when it names
/// no concept the queue can carry, and duplicates fall out by
/// resolved path. The queue is a list, never an edit, and its
/// membership is the contract's set only. Verification currency
/// auditing stays the trust auditor's work, never recomputed
/// here.
///
/// # Errors
///
/// Returns an error when the stamp carries no real calendar
/// date, or when a supplied path names no file in the bundle, a
/// reserved file, the claim registry, or a daily file.
pub fn reverification_queue(
    root: &Path,
    at: &str,
    changed: &[String],
) -> std::io::Result<Vec<String>> {
    if date_part(at).is_none() {
        return Err(invalid(format!(
            "the stamp {at} carries no real calendar date"
        )));
    }
    let bundle = read_bundle(root);
    let mut queue: Vec<String> = Vec::new();
    for path in changed {
        if path.contains(['\n', '\r']) {
            return Err(invalid(
                "a queue path carries a line break, and the queue is printed as lines".to_owned(),
            ));
        }
        let Some(file) = bundle.files.iter().find(|file| &file.rel_path == path) else {
            return Err(not_found(format!("no file at {path} in the bundle")));
        };
        if file.is_registry {
            return Err(invalid(
                "claims.md is the claim registry, which is regenerated, not verified".to_owned(),
            ));
        }
        if !file.is_managed_concept() {
            return Err(invalid(format!(
                "{path} is not a concept the queue can carry"
            )));
        }
        if file.layer == Layer::Daily {
            return Err(invalid(format!(
                "{path} is in the daily layer, which a pass never changes"
            )));
        }
        if !queue.contains(path) {
            queue.push(path.clone());
        }
    }
    Ok(queue)
}

/// How a promotion ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Promotion {
    /// The claim block and any supersession upkeep were written.
    Promoted,
    /// The target already carried the claim unchanged and the
    /// supersession upkeep was already complete, so nothing was
    /// written.
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

/// Render a verbatim quote as a double-quoted scalar. Quote
/// sub-items always carry the quoted form, in the daily files the
/// product converts and in the curated layer alike, because a
/// quote is an excerpt whose exact wording is the evidence. The
/// escaping is the registry's own, so every control character the
/// subset's parser resolves is escaped the same way everywhere.
fn render_quote(text: &str) -> String {
    format!("\"{}\"", escape_quoted(text))
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
/// index. Existing lines are carried as raw slices of the input,
/// endings included, so nothing outside the inserted lines is
/// rewritten. The inserted lines use the document's own line
/// ending, and a document that did not end with a newline gains
/// one.
fn insert_after(body: &str, at: usize, new_lines: &[String]) -> String {
    let eol = line_ending(body);
    let segments: Vec<&str> = body.split_inclusive('\n').collect();
    let mut out = String::new();
    if segments.is_empty() {
        for line in new_lines {
            out.push_str(line);
            out.push_str(eol);
        }
        return out;
    }
    let cut = (at + 1).min(segments.len());
    out.push_str(&segments[..cut].concat());
    if !out.ends_with('\n') {
        out.push_str(eol);
    }
    for line in new_lines {
        out.push_str(line);
        out.push_str(eol);
    }
    out.push_str(&segments[cut..].concat());
    if !out.ends_with('\n') {
        out.push_str(eol);
    }
    out
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

/// Whether a line is the heading of a superseded claims section.
fn is_superseded_heading(line: &str) -> bool {
    line.starts_with('#') && line.to_lowercase().contains("superseded claims")
}

/// Append one claim block to a body after its last claim block.
/// A body with no claim blocks but a superseded claims section
/// takes the block before that section, never inside it. A body
/// with neither takes the block at its end.
fn append_claim_block(body: &str, claim: &Claim) -> String {
    let block = render_claim_block(claim);
    if !scan_claims(body).is_empty() {
        return insert_after(body, block_insert_point(body), &block);
    }
    let eol = line_ending(body);
    if let Some(heading) = body.lines().position(is_superseded_heading) {
        let segments: Vec<&str> = body.split_inclusive('\n').collect();
        let mut out = String::new();
        out.push_str(&segments[..heading].concat());
        let keep = out.trim_end_matches(['\r', '\n']).len();
        out.truncate(keep);
        if !out.is_empty() {
            out.push_str(eol);
            out.push_str(eol);
        }
        out.push_str(&block.join(eol));
        out.push_str(eol);
        out.push_str(eol);
        out.push_str(&segments[heading..].concat());
        if !out.ends_with('\n') {
            out.push_str(eol);
        }
        return out;
    }
    let mut text = body.trim_end_matches(['\r', '\n']).to_owned();
    if !text.is_empty() {
        text.push_str(eol);
        text.push_str(eol);
    }
    text.push_str(&block.join(eol));
    text.push_str(eol);
    text
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
) -> std::io::Result<String> {
    // The scanner reads the entry by splitting on the first
    // occurrence of the words Replaced by, so a superseded text
    // carrying those words itself cannot be recorded in this
    // shape. Refuse rather than write an entry nothing can read
    // back.
    if old_text.contains(" Replaced by ") {
        return Err(invalid(format!(
            "the superseded claim {old} carries the words Replaced by in its own text, which the superseded claims section entry cannot represent"
        )));
    }
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
    let section = lines.iter().position(|line| is_superseded_heading(line));
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
    let eol = line_ending(body);
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

/// Give a body the `superseded-by` reverse link on its copy of the
/// superseded claim, naming the replacement. Returns the body
/// unchanged when the link is already there, and refuses when the
/// copy already names a different replacement.
fn with_reverse_link(body: &str, old: &str, new_id: &str) -> std::io::Result<String> {
    let claims = scan_claims(body);
    let Some(claim) = claims
        .iter()
        .find(|carried| carried.id.as_deref() == Some(old))
    else {
        return Ok(body.to_owned());
    };
    match &claim.superseded_by {
        Some(existing) if existing == new_id => Ok(body.to_owned()),
        Some(existing) => Err(invalid(format!(
            "the curated copy of {old} already names {existing} as its replacement, so the reverse link is not this promotion's to write"
        ))),
        None => {
            let lines: Vec<&str> = body.lines().collect();
            let mut at = claim.line - 1;
            while at + 1 < lines.len() && lines[at + 1].starts_with("  - ") {
                at += 1;
            }
            let link = format!("  - superseded-by: {}", render_scalar(new_id));
            Ok(insert_after(body, at, &[link]))
        }
    }
}

/// The reason a superseding promotion carries, checked as the
/// one line the superseded claims section records. A reason that
/// is missing, blank, or spans lines would corrupt or forge
/// section entries, so it is refused before anything is written.
fn checked_reason<'a>(claim: &Claim, reason: Option<&'a str>) -> std::io::Result<&'a str> {
    let old = claim.supersedes.as_deref().unwrap_or_default();
    let Some(reason) = reason else {
        return Err(invalid(format!(
            "the claim {} supersedes {old}, and a superseding promotion needs its one-line reason",
            claim.id.as_deref().unwrap_or_default()
        )));
    };
    let trimmed = reason.trim();
    if trimmed.is_empty() || trimmed.contains(['\n', '\r']) {
        return Err(invalid(
            "a superseding promotion's reason is one non-empty line".to_owned(),
        ));
    }
    Ok(trimmed)
}

/// Promote one claim from the daily layer into a named curated
/// concept. The `target` is the curated concept's bundle-relative
/// path. When the claim carries a supersedes link, `reason` is
/// required. The superseded claims section entry goes to the
/// concept that holds the superseded claim, or to the target when
/// the superseded claim was never promoted, dated by the daily
/// file the claim lives in, and the superseded claim's curated
/// copy gains its `superseded-by` reverse link. The daily file is
/// never touched, because the daily layer is append-only. Every
/// refusal is decided before any file is written.
///
/// # Errors
///
/// Returns a `NotFound` error when the claim, the claim it
/// supersedes, or the target concept cannot be found, and an
/// `InvalidInput` error when the target is not a curated concept,
/// when the curated layer already carries the identity in another
/// concept or with different wording, when the claim carries a
/// reverse link, when a superseding promotion is given no usable
/// one-line reason, when the superseded claim's text cannot be
/// represented in a section entry, when its curated copy names a
/// different replacement, or when a section already records the
/// supersession for another replacement. Returns an I/O error
/// when a file cannot be read or written.
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
    // The checker's duplication and drift rules span the whole
    // curated layer, so the refusal does too. The identity carried
    // by another concept refuses outright, and the identity carried
    // by the target is compared for drift below.
    let mut carried: Option<Claim> = None;
    for file in &bundle.files {
        if file.layer != Layer::Curated || file.reserved {
            continue;
        }
        for other in scan_claims(&file.body) {
            if other.id.as_deref() == Some(claim_id) {
                if file.rel_path != target {
                    return Err(invalid(format!(
                        "the curated layer already carries {claim_id} in {}, and promotion into {target} would duplicate it",
                        file.rel_path
                    )));
                }
                carried = Some(other);
            }
        }
    }
    if let Some(existing) = &carried {
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
    }

    let mut target_body: Option<String> = None;
    if carried.is_none() {
        target_body = Some(append_claim_block(&target_file.body, &claim));
    }
    let upkeep = plan_supersession(
        &bundle,
        &claim,
        &daily_rel,
        target,
        target_file,
        target_body,
        reason,
    )?;

    let mut wrote = false;
    if let Some(body) = &upkeep.target_body
        && body != &target_file.body
    {
        write_body(root, target, body)?;
        wrote = true;
    }
    if let (Some(body), Some(rel)) = (&upkeep.holder_body, &upkeep.holder_rel) {
        let holder_file = bundle
            .files
            .iter()
            .find(|file| file.rel_path == *rel)
            .map(|file| &file.body);
        if holder_file != Some(body) {
            write_body(root, rel, body)?;
            wrote = true;
        }
    }
    if wrote {
        Ok(Promotion::Promoted)
    } else {
        Ok(Promotion::AlreadyPresent)
    }
}

/// The new bodies a promotion's supersession upkeep produces, the
/// target's and the superseded claim's holder's, either of which
/// may be absent when that file needs no change.
struct Upkeep {
    target_body: Option<String>,
    holder_body: Option<String>,
    holder_rel: Option<String>,
}

/// Plan the supersession upkeep for one promotion, the reverse
/// link and the superseded claims section entry. The concept
/// holding the superseded claim's curated copy is the entry's home
/// and the reverse link's home. When the old claim was never
/// promoted, the entry goes to the target, which is where the
/// registry has always read it for a daily-only claim. All of the
/// upkeep's refusals are decided here, before any file is written.
fn plan_supersession(
    bundle: &Bundle,
    claim: &Claim,
    daily_rel: &str,
    target: &str,
    target_file: &crate::bundle::ConceptFile,
    target_body: Option<String>,
    reason: Option<&str>,
) -> std::io::Result<Upkeep> {
    let mut upkeep = Upkeep {
        target_body,
        holder_body: None,
        holder_rel: None,
    };
    let Some(old) = &claim.supersedes else {
        return Ok(upkeep);
    };
    let claim_id = claim.id.as_deref().unwrap_or_default();
    let reason = checked_reason(claim, reason)?;
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
    let holder = bundle.files.iter().find(|file| {
        file.layer == Layer::Curated
            && !file.reserved
            && scan_claims(&file.body)
                .iter()
                .any(|carried| carried.id.as_deref() == Some(old.as_str()))
    });
    match holder {
        Some(holder_file) if holder_file.rel_path != target => {
            let mut body = with_reverse_link(&holder_file.body, old, claim_id)?;
            body = append_section_entry(&body, old, &old_claim.text, claim_id, &date, reason)?;
            upkeep.holder_body = Some(body);
            upkeep.holder_rel = Some(holder_file.rel_path.clone());
        }
        Some(_) => {
            let base = upkeep
                .target_body
                .take()
                .unwrap_or_else(|| target_file.body.clone());
            let mut body = with_reverse_link(&base, old, claim_id)?;
            body = append_section_entry(&body, old, &old_claim.text, claim_id, &date, reason)?;
            upkeep.target_body = Some(body);
        }
        None => {
            let base = upkeep
                .target_body
                .take()
                .unwrap_or_else(|| target_file.body.clone());
            upkeep.target_body = Some(append_section_entry(
                &base,
                old,
                &old_claim.text,
                claim_id,
                &date,
                reason,
            )?);
        }
    }
    Ok(upkeep)
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
