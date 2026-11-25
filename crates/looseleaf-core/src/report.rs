//! The Synthesis report concept writer.
//!
//! Every synthesis run writes one report concept listing what the
//! pass did, and the contract treats that report as the unit the
//! run is audited against. This writer owns the concept's shape.
//! It creates the run's report with the six sections the contract
//! names, appends one entry at a time under the named section, and
//! refuses everything that would blur the record, a foreign
//! concept at the report path, a hand-edited report missing a
//! section, a symbolic linked synthesis directory. What an entry
//! says stays the host's judgment under the contract. The writer
//! enforces the line shape and the file shape, nothing more.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::bundle::{Layer, read_bundle};
use crate::check::is_calendar_date;
use crate::frontmatter::{Value, parse_document};
use crate::indexes::regenerate_layer;
use crate::stamps::{PASS_ACTOR, stamp_generated, stamp_parts};
use crate::write::{bare_segment, invalid_input, line_ending, write_via_temp};

/// The six section headings of a pass report, in contract order.
pub const REPORT_HEADINGS: [&str; 6] = [
    "Promotions",
    "Supersessions",
    "Regenerated",
    "Flags",
    "Blocked",
    "Re-verification queue",
];

/// One section of a pass report, named to the writer by token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportSection {
    /// Claims promoted in the run.
    Promotions,
    /// Claims superseded in the run.
    Supersessions,
    /// Indexes and the registry regenerated in the run.
    Regenerated,
    /// Shape and health findings the run raised.
    Flags,
    /// Promotions the run blocked, and why.
    Blocked,
    /// The re-verification queue the run emitted.
    Queue,
}

impl ReportSection {
    /// The section a CLI token names, when the token is one of
    /// the six the writer accepts.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "promotions" => Some(Self::Promotions),
            "supersessions" => Some(Self::Supersessions),
            "regenerated" => Some(Self::Regenerated),
            "flags" => Some(Self::Flags),
            "blocked" => Some(Self::Blocked),
            "queue" => Some(Self::Queue),
            _ => None,
        }
    }

    /// The heading the section carries in the report body.
    #[must_use]
    pub fn heading(self) -> &'static str {
        match self {
            Self::Promotions => "Promotions",
            Self::Supersessions => "Supersessions",
            Self::Regenerated => "Regenerated",
            Self::Flags => "Flags",
            Self::Blocked => "Blocked",
            Self::Queue => "Re-verification queue",
        }
    }
}

/// Whether a file stem carries the report naming pattern, a real
/// calendar date, a dash, and a six digit time. The checker uses
/// this to decide which synthesis files the report shape binds.
#[must_use]
pub(crate) fn is_report_stem(stem: &str) -> bool {
    stem.len() == 17
        && stem.get(..10).is_some_and(is_calendar_date)
        && stem.get(10..11) == Some("-")
        && stem
            .get(11..)
            .is_some_and(|time| time.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Split a report stamp into its date and its six digit time,
/// refusing any stamp that does not carry a real calendar date
/// and a real time of day, since the two become the file name.
/// The validation is the stamps module's whole-stamp rule, with
/// the time required here rather than optional.
fn report_stamp_parts(at: &str) -> std::io::Result<(String, String)> {
    match stamp_parts(at) {
        Some((date, Some(time))) => Ok((date.to_owned(), time)),
        _ => Err(invalid_input(format!(
            "the stamp {at} must carry a real calendar date and time, as 2026-11-24T19:06:02Z"
        ))),
    }
}

/// Create the run's report concept with all six sections in
/// contract order, then stamp its `generated` through the Slice 1
/// stamp, so the one construction of that mapping serves both.
/// The section headings come from the same constant the checker
/// reads, so the writer and the audit cannot drift apart.
fn create_report(
    root: &Path,
    rel: &str,
    target: &Path,
    date: &str,
    at: &str,
) -> std::io::Result<()> {
    let mut text = format!(
        "---\ntype: Synthesis\ntitle: \"Synthesis report {date}\"\ndescription: \"The synthesis pass report for {date}, written by the pass under its contract.\"\n---\n\n# Synthesis report {date}\n"
    );
    for heading in REPORT_HEADINGS {
        let _ = write!(text, "\n## {heading}\n");
    }
    write_via_temp(target, &text)?;
    stamp_generated(root, rel, PASS_ACTOR, at)?;
    Ok(())
}

/// Append one entry to one section of the run's Synthesis report
/// concept, creating the report when this run has none yet, and
/// return the report's path relative to the bundle root. A report
/// created here carries all six sections, so every later append
/// is a lookup. When the synthesis directory or its index is
/// missing, the writer creates the directory and regenerates that
/// layer's index in the same operation, so its own write never
/// leaves the bundle failing the layout check.
///
/// # Errors
///
/// Returns an error when the stamp carries no real date and time,
/// the entry is blank or spans lines, the synthesis directory is
/// a symbolic link, the file at the report path is not a Synthesis
/// concept, or an existing report is missing the named section.
pub fn append_report_entry(
    root: &Path,
    at: &str,
    section: ReportSection,
    entry: &str,
) -> std::io::Result<String> {
    let (date, time) = report_stamp_parts(at)?;
    if entry.trim().is_empty() || entry.contains(['\n', '\r']) {
        return Err(invalid_input(
            "the entry must be one non-empty line".to_owned(),
        ));
    }
    let dir = root.join("synthesis");
    match fs::symlink_metadata(&dir) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(invalid_input(
                "the synthesis directory is a symbolic link, and the writers never write through one"
                    .to_owned(),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&dir)?;
        }
        Err(error) => return Err(error),
    }
    let rel = format!("synthesis/{date}-{time}.md");
    let target = root.join(&rel);
    let mut created = false;
    if !target.exists() {
        create_report(root, &rel, &target, &date, at)?;
        created = true;
    }
    let text = fs::read_to_string(&target)?;
    let document = parse_document(&text);
    let carries_type = document
        .frontmatter
        .as_ref()
        .and_then(|frontmatter| frontmatter.get("type"))
        == Some(&Value::Scalar("Synthesis".to_owned()));
    if document.parse_error.is_some() || !carries_type {
        return Err(invalid_input(format!(
            "the report at {rel} does not carry type Synthesis"
        )));
    }
    // A creation interrupted between the file write and its
    // stamp leaves a report with no generated entry. Heal it
    // here rather than appending over a report that lacks its
    // run stamp, so the state the checker would flag as an error
    // is never the state this writer reports success over.
    let carries_generated = document
        .frontmatter
        .as_ref()
        .is_some_and(|frontmatter| frontmatter.get("generated").is_some());
    let text = if carries_generated {
        text
    } else {
        stamp_generated(root, &rel, PASS_ACTOR, at)?;
        fs::read_to_string(&target)?
    };
    let out = append_to_section(&text, &rel, section, entry)?;
    write_via_temp(&target, &out)?;
    if created || !root.join("synthesis/index.md").exists() {
        let bundle = read_bundle(root);
        regenerate_layer(root, &bundle, Layer::Synthesis)?;
    }
    Ok(rel)
}

/// Insert an entry at the end of its section in an existing
/// report body. The heading search is fence aware, so a heading
/// quoted inside a code fence is content, never the section, and
/// the section ends at the next unfenced level two heading.
/// Existing lines are carried as raw slices, so their endings
/// survive untouched.
fn append_to_section(
    text: &str,
    rel: &str,
    section: ReportSection,
    entry: &str,
) -> std::io::Result<String> {
    let eol = line_ending(text);
    let segments: Vec<&str> = text.split_inclusive('\n').collect();
    let heading_line = format!("## {}", section.heading());
    let mut in_fence = false;
    let mut heading = None;
    for (index, segment) in segments.iter().enumerate() {
        let line = bare_segment(segment);
        if line.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && line == heading_line {
            heading = Some(index);
            break;
        }
    }
    let Some(heading) = heading else {
        return Err(invalid_input(format!(
            "the report at {rel} is missing its {} section",
            section.heading()
        )));
    };
    let mut insert_after = heading;
    let mut in_fence = false;
    for (index, segment) in segments.iter().enumerate().skip(heading + 1) {
        let line = bare_segment(segment);
        if line.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && line.starts_with("## ") {
            break;
        }
        if !line.trim().is_empty() {
            insert_after = index;
        }
    }
    let mut out = String::with_capacity(text.len() + entry.len() + 4);
    for segment in &segments[..=insert_after] {
        out.push_str(segment);
    }
    out.push_str("- ");
    out.push_str(entry);
    out.push_str(eol);
    for segment in &segments[insert_after + 1..] {
        out.push_str(segment);
    }
    Ok(out)
}
