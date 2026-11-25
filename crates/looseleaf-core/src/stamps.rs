//! The synthesis pass's frontmatter stamps.
//!
//! The contract has the pass restamp `generated` on every concept
//! it creates or changes in meaning, and stamp `verified` with its
//! own actor for checks it truly ran. These are the tool's two
//! stamp commands. Both work through the frontmatter upsert, so
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
use crate::frontmatter::Value;
use crate::write::{invalid_input, not_found, upsert_frontmatter};

/// The actor the synthesis pass stamps under the profile.
pub const PASS_ACTOR: &str = "process:looseleaf-synthesis";

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
    if file.reserved {
        return Err(invalid_input(format!(
            "{path} is a reserved file, not a concept"
        )));
    }
    if file.is_registry {
        return Err(invalid_input(
            "claims.md is the claim registry, and its frontmatter is owned by the registry renderer"
                .to_owned(),
        ));
    }
    match file.layer {
        Layer::Daily => {
            return Err(invalid_input(format!(
                "{path} is in the daily layer, which the tool never edits"
            )));
        }
        Layer::Directives => {
            return Err(invalid_input(format!(
                "{path} is a directive concept, which the contract human-gates against change"
            )));
        }
        Layer::Root => {
            return Err(invalid_input(format!(
                "{path} is not inside a layer directory"
            )));
        }
        _ => {}
    }
    if file.read_error {
        return Err(invalid_input(format!("{path} could not be read as text")));
    }
    Ok((file.clone(), root.join(path)))
}

fn write_stamp(target: &Path, text: &str) -> std::io::Result<()> {
    crate::write::write_via_temp(target, text)
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
    let upsert =
        upsert_frontmatter(&text, "generated", &by_at_value(by, at)).map_err(invalid_input)?;
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
    let upsert = upsert_frontmatter(&text, "verified", &new_value).map_err(invalid_input)?;
    if !upsert.changed {
        return Ok(Stamp::AlreadyCurrent);
    }
    write_stamp(&target, &upsert.text)?;
    Ok(Stamp::Stamped)
}
