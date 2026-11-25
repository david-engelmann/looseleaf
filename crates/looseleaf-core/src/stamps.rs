//! The synthesis pass's frontmatter stamps.
//!
//! The contract has the pass restamp `generated` on every concept
//! it creates or changes in meaning, and stamp `verified` with its
//! own actor for checks it truly ran. These are the tool's two
//! stamp commands, and the module also houses the deprecation act,
//! which writes a retired concept's status, its replacement
//! pointer and its restamp in one composed write. All three work
//! through the frontmatter upsert, so
//! only the stamped key's lines change, and both validate their
//! arguments before any write, because the staleness report orders
//! verifications as strings and one garbage stamp would falsify it
//! permanently. What a stamp may certify stays with the contract.
//! A stateless command cannot know its caller or what else the
//! caller changed in the same pass, so the actor prohibitions are
//! recorded in DECISIONS D20 rather than enforced here.

use std::fs;
use std::path::{Path, PathBuf};

use crate::bundle::{ConceptFile, Layer, read_bundle};
use crate::check::is_calendar_date;
use crate::frontmatter::{Value, parse_document};
use crate::write::{invalid_input, not_found, upsert_frontmatter};

/// The actor the synthesis pass stamps under the profile.
pub const PASS_ACTOR: &str = "process:looseleaf-synthesis";

/// What a deprecation did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deprecation {
    /// The concept was deprecated by this call.
    Deprecated,
    /// The concept already stood exactly as this call would
    /// write it, and nothing was written.
    AlreadyCurrent,
}

/// Deprecate a concept in favour of its replacement, executing
/// a decision the contract's human gate has already made.
///
/// The act writes the three keys the profile names for it, the
/// status deprecated, the `superseded_by` pointer at the
/// replacement, and the generated restamp with the pass actor,
/// composed in memory and written once, so no reader sees a
/// half-deprecated concept. A concept carrying part of the act
/// already, the pointer without the status or the status
/// without the pointer, in the plain scalar forms the profile
/// writes, is completed, and a standing pointer in a
/// non-canonical form the checker accepts is canonicalized by
/// the completion. A pointer naming a different replacement is
/// refused, because repointing a deprecation is a new decision,
/// not an execution of one. A replacement whose own pointer
/// chain leads back to the target is refused, because the act
/// would close a supersession cycle no checker rule detects.
/// The concept's claims are untouched, the profile keeps the
/// concept and claim axes separate.
///
/// # Errors
///
/// Returns an error when the stamp is not a real date with an
/// optional real time, when either path names no file, a
/// reserved file, the registry, a daily or directive file, a
/// root file or a file that did not read cleanly, when the
/// replacement is the target itself, a pass report, or the
/// end of a cycle back to the target, when the replacement
/// path carries an anchor a pointer cannot name, when the
/// target carries no frontmatter, frontmatter that does not
/// parse, or a status or pointer that is not a plain scalar,
/// or when the composed document would no longer parse.
pub fn deprecate_concept(
    root: &Path,
    path: &str,
    superseded_by: &str,
    at: &str,
) -> std::io::Result<Deprecation> {
    checked_stamp(at)?;
    if superseded_by.contains('#') {
        return Err(invalid_input(format!(
            "the replacement path {superseded_by} carries an anchor, which a superseded_by pointer cannot name"
        )));
    }
    let bundle = read_bundle(root);
    let Some(file) = bundle.files.iter().find(|file| file.rel_path == path) else {
        return Err(not_found(format!("no file at {path} in the bundle")));
    };
    if let Some(refusal) = stampable_refusal(file, path) {
        return Err(refusal);
    }
    if file.fm_error.is_some() {
        return Err(invalid_input(format!(
            "{path} carries frontmatter that does not parse in the subset"
        )));
    }
    let Some(frontmatter) = file.frontmatter.as_ref() else {
        return Err(invalid_input(format!(
            "{path} carries no frontmatter to deprecate"
        )));
    };
    if let Some(status) = frontmatter.get("status")
        && !matches!(status, Value::Scalar(_))
    {
        return Err(invalid_input(format!(
            "{path} carries a status that is not a plain value"
        )));
    }
    if let Some(existing) = frontmatter.get("superseded_by") {
        let Value::Scalar(existing) = existing else {
            return Err(invalid_input(format!(
                "{path} carries a superseded_by that is not a plain path"
            )));
        };
        if normalized_pointer(existing).as_deref() != Some(superseded_by) {
            return Err(invalid_input(format!(
                "{path} already points at {existing} as its replacement, and repointing a deprecation is a new decision"
            )));
        }
    }
    if superseded_by == path {
        return Err(invalid_input(
            "a concept cannot be deprecated in favour of itself".to_owned(),
        ));
    }
    let Some(replacement) = bundle
        .files
        .iter()
        .find(|file| file.rel_path == superseded_by)
    else {
        return Err(not_found(format!(
            "no file at {superseded_by} in the bundle"
        )));
    };
    if let Some(refusal) = stampable_refusal(replacement, superseded_by) {
        return Err(refusal);
    }
    if replacement.read_error || replacement.frontmatter.is_none() {
        return Err(invalid_input(format!(
            "{superseded_by} does not carry readable frontmatter"
        )));
    }
    if replacement.layer == Layer::Synthesis && is_pass_report(superseded_by) {
        return Err(invalid_input(format!(
            "{superseded_by} is a synthesis pass report, not a successor concept"
        )));
    }
    if chain_leads_to(&bundle, replacement, path) {
        return Err(invalid_input(format!(
            "deprecating {path} in favour of {superseded_by} would close a supersession cycle"
        )));
    }
    let target = root.join(path);
    let text = fs::read_to_string(&target)?;
    let mut composed = upsert_frontmatter(&text, "status", &Value::Scalar("deprecated".to_owned()))
        .map_err(invalid_input)?;
    composed = upsert_frontmatter(
        &composed.text,
        "superseded_by",
        &Value::Scalar(superseded_by.to_owned()),
    )
    .map_err(invalid_input)?;
    composed = upsert_frontmatter(&composed.text, "generated", &by_at_value(PASS_ACTOR, at))
        .map_err(invalid_input)?;
    let composed = checked_upsert(composed, path, "deprecating")?;
    if composed.text == text {
        return Ok(Deprecation::AlreadyCurrent);
    }
    write_stamp(&target, &composed.text)?;
    Ok(Deprecation::Deprecated)
}

/// Whether the file at a bundle path is a synthesis pass
/// report, judged by the report naming pattern on its stem.
fn is_pass_report(rel_path: &str) -> bool {
    let name = rel_path.rsplit('/').next().unwrap_or(rel_path);
    let stem = name
        .get(..name.len().saturating_sub(3))
        .filter(|_| name.to_lowercase().ends_with(".md"))
        .unwrap_or(name);
    crate::report::is_report_stem(stem)
}

/// Whether the pointer chain starting at `from` reaches
/// `target`, following each concept's `superseded_by` through
/// the bundle. The walk is bounded by the bundle's file count,
/// so a chain that already cycles without reaching the target
/// still ends.
fn chain_leads_to(bundle: &crate::bundle::Bundle, from: &ConceptFile, target: &str) -> bool {
    let mut chain_at = from;
    for _ in 0..bundle.files.len() {
        let Some(Value::Scalar(next)) = chain_at
            .frontmatter
            .as_ref()
            .and_then(|frontmatter| frontmatter.get("superseded_by"))
        else {
            return false;
        };
        let Some(normalized) = normalized_pointer(next) else {
            return false;
        };
        if normalized == target {
            return true;
        }
        let Some(next_file) = bundle.files.iter().find(|file| file.rel_path == normalized) else {
            return false;
        };
        chain_at = next_file;
    }
    false
}

/// Normalize a standing `superseded_by` pointer the way the
/// checker's resolver does, dropping the anchor, empty and
/// dot components and resolving parent components that stay
/// inside the bundle, so a pointer the checker accepts as
/// naming a file compares equal to that file's canonical
/// path. A pointer that climbs out of the bundle normalizes
/// to nothing.
fn normalized_pointer(pointer: &str) -> Option<String> {
    let path_part = pointer.split('#').next().unwrap_or("");
    if path_part.is_empty() || path_part.starts_with('/') {
        return None;
    }
    let mut parts: Vec<&str> = Vec::new();
    for component in path_part.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("/"))
}

/// The refusal a file draws when an act may not touch it, or
/// nothing when the file is touchable. The stamp target ladder
/// and the deprecation replacement ladder share this one rule
/// set, so the two can never drift apart on what they refuse.
fn stampable_refusal(file: &ConceptFile, path: &str) -> Option<std::io::Error> {
    if file.reserved {
        return Some(invalid_input(format!(
            "{path} is a reserved file, not a concept"
        )));
    }
    if file.is_registry {
        return Some(invalid_input(
            "claims.md is the claim registry, and its frontmatter is owned by the registry renderer"
                .to_owned(),
        ));
    }
    match file.layer {
        Layer::Daily => {
            return Some(invalid_input(format!(
                "{path} is in the daily layer, which the tool never edits"
            )));
        }
        Layer::Directives => {
            return Some(invalid_input(format!(
                "{path} is a directive concept, which the contract human-gates against change"
            )));
        }
        Layer::Root => {
            return Some(invalid_input(format!(
                "{path} is not inside a layer directory"
            )));
        }
        _ => {}
    }
    if file.read_error {
        return Some(invalid_input(format!("{path} could not be read as text")));
    }
    None
}

/// What a stamp command did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stamp {
    /// The stamp was written.
    Stamped,
    /// The stamp already stood as given, and nothing was written.
    AlreadyCurrent,
}

/// The `{ by, at }` mapping both stamps write, and the report
/// writer stamps on a report it creates.
pub(crate) fn by_at_value(by: &str, at: &str) -> Value {
    Value::Map(vec![
        ("by".to_owned(), Value::Scalar(by.to_owned())),
        ("at".to_owned(), Value::Scalar(at.to_owned())),
    ])
}

fn checked_actor(by: &str) -> std::io::Result<()> {
    if by.trim().is_empty() || by.contains(['\n', '\r']) {
        return Err(invalid_input(
            "the actor must be one non-empty line".to_owned(),
        ));
    }
    Ok(())
}

/// Split a stamp into its date and its optional six digit time,
/// validating the whole string. A stamp is a real calendar date,
/// optionally followed by `T`, a real time of day, and an optional
/// trailing `Z`. Nothing past the date is ignored, because the
/// staleness report orders stamps as strings and a garbage tail
/// would falsify that order within the day. The report writer
/// shares this rule and additionally requires the time.
pub(crate) fn stamp_parts(at: &str) -> Option<(&str, Option<String>)> {
    let date = at.get(..10)?;
    if !is_calendar_date(date) {
        return None;
    }
    let rest = at.get(10..)?;
    if rest.is_empty() {
        return Some((date, None));
    }
    let time = rest.strip_prefix('T')?;
    let clock = time.strip_suffix('Z').unwrap_or(time);
    if clock.len() != 8 {
        return None;
    }
    let digit = |index: usize| clock.as_bytes().get(index).is_some_and(u8::is_ascii_digit);
    if clock.as_bytes().get(2) != Some(&b':')
        || clock.as_bytes().get(5) != Some(&b':')
        || ![0, 1, 3, 4, 6, 7].iter().all(|&index| digit(index))
    {
        return None;
    }
    let hours: u32 = clock.get(..2)?.parse().ok()?;
    let minutes: u32 = clock.get(3..5)?.parse().ok()?;
    let seconds: u32 = clock.get(6..8)?.parse().ok()?;
    if hours > 23 || minutes > 59 || seconds > 59 {
        return None;
    }
    Some((date, Some(format!("{hours:02}{minutes:02}{seconds:02}"))))
}

fn checked_stamp(at: &str) -> std::io::Result<()> {
    if stamp_parts(at).is_none() {
        return Err(invalid_input(format!(
            "the stamp {at} is not a real calendar date with an optional real time"
        )));
    }
    Ok(())
}

/// Resolve a stamp target through the bundle walker and refuse
/// every file a stamp may not touch, so a caller-supplied path can
/// never name a file outside the bundle or outside the rules.
fn stamp_target(root: &Path, path: &str) -> std::io::Result<(ConceptFile, PathBuf)> {
    let bundle = read_bundle(root);
    let Some(file) = bundle.files.iter().find(|file| file.rel_path == path) else {
        return Err(not_found(format!("no file at {path} in the bundle")));
    };
    if let Some(refusal) = stampable_refusal(file, path) {
        return Err(refusal);
    }
    Ok((file.clone(), root.join(path)))
}

fn write_stamp(target: &Path, text: &str) -> std::io::Result<()> {
    crate::write::write_via_temp(target, text)
}

/// The upserted document, refused when it no longer parses.
/// Wrapping a record one level deeper, as the verified append
/// does, can carry a nested value past the depth the subset
/// parser reads, and the serializer renders it anyway. A stamp
/// never writes a document its own reader cannot take back.
fn checked_upsert(
    upserted: crate::write::Upsert,
    path: &str,
    act: &str,
) -> std::io::Result<crate::write::Upsert> {
    if parse_document(&upserted.text).parse_error.is_some() {
        return Err(invalid_input(format!(
            "{act} {path} would leave a document the subset cannot parse, so nothing is written"
        )));
    }
    Ok(upserted)
}

/// Restamp one concept's `generated` with the given actor and
/// stamp, through the frontmatter upsert, so only the `generated`
/// lines change.
///
/// # Errors
///
/// Returns an error when the actor or stamp fails validation, the
/// path names no stampable concept in the bundle, the file cannot
/// be read, or its frontmatter does not parse in the subset.
pub fn stamp_generated(root: &Path, path: &str, by: &str, at: &str) -> std::io::Result<Stamp> {
    checked_actor(by)?;
    checked_stamp(at)?;
    let (_, target) = stamp_target(root, path)?;
    let text = fs::read_to_string(&target)?;
    let upsert = checked_upsert(
        upsert_frontmatter(&text, "generated", &by_at_value(by, at)).map_err(invalid_input)?,
        path,
        "stamping",
    )?;
    if !upsert.changed {
        return Ok(Stamp::AlreadyCurrent);
    }
    write_stamp(&target, &upsert.text)?;
    Ok(Stamp::Stamped)
}

/// The `(by, at)` pair of a verification record, when the value is
/// a mapping carrying both as scalars. Other entries are ignored,
/// the same rule the checker's record shape applies, so a record
/// the checker accepts is a record this stamp can extend.
fn record_parts(value: &Value) -> Option<(&str, &str)> {
    let Value::Map(entries) = value else {
        return None;
    };
    let mut by = None;
    let mut at = None;
    for (key, item) in entries {
        match (key.as_str(), item) {
            ("by", Value::Scalar(text)) => by = Some(text.as_str()),
            ("at", Value::Scalar(text)) => at = Some(text.as_str()),
            ("by" | "at", _) => return None,
            _ => {}
        }
    }
    match (by, at) {
        (Some(by), Some(at)) => Some((by, at)),
        _ => None,
    }
}

/// Record one verification on a concept. An absent `verified` is
/// written as one mapping, a mapping becomes a list of two with
/// the existing record first, and a list gains the record at its
/// end. A record with the same actor and stamp already stands, and
/// the command is then a no-op. An existing `verified` in any
/// other shape is refused, never converted and never appended to.
///
/// # Errors
///
/// Returns an error when the actor or stamp fails validation, the
/// path names no stampable concept in the bundle, the concept
/// carries no frontmatter, its existing `verified` is malformed,
/// or its frontmatter does not parse in the subset.
pub fn stamp_verified(root: &Path, path: &str, by: &str, at: &str) -> std::io::Result<Stamp> {
    checked_actor(by)?;
    checked_stamp(at)?;
    let (file, target) = stamp_target(root, path)?;
    let Some(frontmatter) = &file.frontmatter else {
        return Err(invalid_input(format!(
            "{path} carries no frontmatter to stamp"
        )));
    };
    let malformed = || {
        invalid_input(format!(
            "the verified entry on {path} is not a mapping or a list of mappings with scalar by and at"
        ))
    };
    let record = by_at_value(by, at);
    let new_value = match frontmatter.get("verified") {
        None => record,
        Some(existing @ Value::Map(_)) => {
            let Some((old_by, old_at)) = record_parts(existing) else {
                return Err(malformed());
            };
            if old_by == by && old_at == at {
                return Ok(Stamp::AlreadyCurrent);
            }
            Value::List(vec![existing.clone(), record])
        }
        Some(Value::List(items)) => {
            if items.is_empty() {
                return Err(malformed());
            }
            // Every item is validated before the duplicate check,
            // so a malformed tail is refused rather than hidden
            // behind an early no-op.
            let mut records = Vec::with_capacity(items.len() + 1);
            let mut already = false;
            for item in items {
                let Some((old_by, old_at)) = record_parts(item) else {
                    return Err(malformed());
                };
                if old_by == by && old_at == at {
                    already = true;
                }
                records.push(item.clone());
            }
            if already {
                return Ok(Stamp::AlreadyCurrent);
            }
            records.push(record);
            Value::List(records)
        }
        Some(_) => return Err(malformed()),
    };
    let text = fs::read_to_string(&target)?;
    let upsert = checked_upsert(
        upsert_frontmatter(&text, "verified", &new_value).map_err(invalid_input)?,
        path,
        "stamping",
    )?;
    if !upsert.changed {
        return Ok(Stamp::AlreadyCurrent);
    }
    write_stamp(&target, &upsert.text)?;
    Ok(Stamp::Stamped)
}
