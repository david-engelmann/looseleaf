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

use std::fs;
use std::path::Path;

use crate::bundle::{Layer, read_bundle};
use crate::check::is_calendar_date;
use crate::frontmatter::{Value, parse_document};
use crate::indexes::regenerate_layer;
use crate::stamps::{PASS_ACTOR, stamp_generated};
use crate::write::{invalid_input, line_ending};

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
fn report_stamp_parts(at: &str) -> std::io::Result<(String, String)> {
    let refused = || {
        invalid_input(format!(
            "the stamp {at} must carry a real calendar date and time, as 2026-11-24T19:06:02Z"
        ))
    };
    let Some(date) = at.get(..10) else {
        return Err(refused());
    };
    if !is_calendar_date(date) || at.get(10..11) != Some("T") {
        return Err(refused());
    }
    let Some(time) = at.get(11..19) else {
        return Err(refused());
    };
    let digits =
        |range: std::ops::Range<usize>| time.get(range).and_then(|part| part.parse::<u32>().ok());
    let (Some(hours), Some(minutes), Some(seconds)) = (digits(0..2), digits(3..5), digits(6..8))
    else {
        return Err(refused());
    };
    if time.get(2..3) != Some(":")
        || time.get(5..6) != Some(":")
        || hours > 23
        || minutes > 59
        || seconds > 59
    {
        return Err(refused());
    }
    Ok((
        date.to_owned(),
        format!("{hours:02}{minutes:02}{seconds:02}"),
    ))
}

fn bare_segment(segment: &str) -> &str {
    segment.trim_end_matches(['\n', '\r'])
}

fn write_report_file(target: &Path, text: &str) -> std::io::Result<()> {
    let temp = target.with_extension("md.tmp-looseleaf");
    fs::write(&temp, text)?;
    fs::rename(&temp, target)
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
        let text = format!(
            "---\ntype: Synthesis\ntitle: \"Synthesis report {date}\"\ndescription: \"The synthesis pass report for {date}, written by the pass under its contract.\"\n---\n\n# Synthesis report {date}\n\n## Promotions\n\n## Supersessions\n\n## Regenerated\n\n## Flags\n\n## Blocked\n\n## Re-verification queue\n"
        );
        write_report_file(&target, &text)?;
        stamp_generated(root, &rel, PASS_ACTOR, at)?;
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
    let eol = line_ending(&text);
    let segments: Vec<&str> = text.split_inclusive('\n').collect();
    let heading_line = format!("## {}", section.heading());
    let Some(heading) = segments
        .iter()
        .position(|segment| bare_segment(segment) == heading_line)
    else {
        return Err(invalid_input(format!(
            "the report at {rel} is missing its {} section",
            section.heading()
        )));
    };
    let mut insert_after = heading;
    for (index, segment) in segments.iter().enumerate().skip(heading + 1) {
        if bare_segment(segment).starts_with("## ") {
            break;
        }
        if !bare_segment(segment).trim().is_empty() {
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
    write_report_file(&target, &out)?;
    if created || !root.join("synthesis/index.md").exists() {
        let bundle = read_bundle(root);
        regenerate_layer(root, &bundle, Layer::Synthesis)?;
    }
    Ok(rel)
}
