//! Appending dated entries to the bundle's log files.
//!
//! A log is append-only history under dated section headings,
//! newest first. The mechanical step the synthesis contract needs
//! is a dated append that never rewrites existing sections. An
//! append whose date equals the newest section's date joins that
//! section after its last entry. A newer date opens a new section
//! at the top. An older date is refused, because placing it would
//! mean editing history into order rather than appending to it.
//! The date is always a command argument, never a clock the tool
//! reads.
//!
//! The profile names three logs, the bundle log at the root and
//! the curated and daily layer logs, and the scope type names
//! exactly those three. A missing log is created with the heading
//! the clean fixture carries, since a checker finding about the
//! missing file should not block the append that repairs it.
//! Existing lines are carried as raw slices with their own line
//! endings, so an append never rewrites a byte outside the lines
//! it inserts.

use std::fs;
use std::path::Path;

use crate::check::is_calendar_date;
use crate::write::{invalid_input as invalid, line_ending};

/// Which of the bundle's logs an append targets. The profile gives
/// exactly three logs, so the type has exactly three variants and
/// an unlogged layer cannot be named at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogScope {
    /// The bundle log, `log.md` at the root.
    Root,
    /// The curated layer log, `curated/log.md`.
    Curated,
    /// The daily layer log, `daily/log.md`.
    Daily,
}

impl LogScope {
    /// The bundle-relative path of the log this scope names.
    fn rel_path(self) -> &'static str {
        match self {
            LogScope::Root => "log.md",
            LogScope::Curated => "curated/log.md",
            LogScope::Daily => "daily/log.md",
        }
    }

    /// The layer directory this scope's log lives in, when it
    /// lives in one.
    fn layer_dir(self) -> Option<&'static str> {
        match self {
            LogScope::Root => None,
            LogScope::Curated => Some("curated"),
            LogScope::Daily => Some("daily"),
        }
    }

    /// The title heading a log created for this scope carries.
    fn heading(self) -> &'static str {
        match self {
            LogScope::Root => "# Bundle log",
            LogScope::Curated => "# Curated layer log",
            LogScope::Daily => "# Daily layer log",
        }
    }
}

/// Append one entry under a date in the log the scope names,
/// returning the bundle-relative path of the log written. The
/// entry is a single line, rendered as one bullet under the dated
/// section. The write goes to a sibling temporary file and is
/// renamed over the target.
///
/// # Errors
///
/// Returns an `InvalidInput` error when the date is not a real
/// calendar date, when the entry is empty or spans lines, when
/// the date is older than the log's newest section, when the
/// existing log carries no title heading or a section heading
/// that is not a date, or when the layer directory the log lives
/// in is a symbolic link. Returns an I/O error when the log
/// cannot be read or written.
pub fn append_log(
    root: &Path,
    scope: LogScope,
    date: &str,
    entry: &str,
) -> std::io::Result<String> {
    if !is_calendar_date(date) {
        return Err(invalid(format!("{date} is not a real calendar date")));
    }
    let entry = entry.trim();
    if entry.is_empty() || entry.contains(['\n', '\r']) {
        return Err(invalid("a log entry is one non-empty line".to_owned()));
    }
    // The bundle walker never follows a symbolic link, so a layer
    // directory that is one is outside the bundle this tool sees,
    // and appending its log would write outside the bundle.
    if let Some(dir) = scope.layer_dir()
        && fs::symlink_metadata(root.join(dir)).is_ok_and(|meta| meta.file_type().is_symlink())
    {
        return Err(invalid(format!(
            "the layer directory {dir} is a symbolic link, and the writers never write through one"
        )));
    }
    let rel = scope.rel_path();
    let target = root.join(rel);
    let current = match fs::read_to_string(&target) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let rendered = match current {
        None => format!("{}\n\n## {date}\n\n- {entry}\n", scope.heading()),
        Some(text) => render_append(&text, date, entry)?,
    };
    let temp = target.with_extension("md.tmp-looseleaf");
    fs::write(&temp, rendered)?;
    fs::rename(&temp, &target)?;
    Ok(rel.to_owned())
}

/// Render a log with one entry appended, per the rules in the
/// module documentation. Existing lines are carried as raw slices
/// of the input, endings included, so nothing already in the log
/// is rewritten beyond the inserted lines.
fn render_append(current: &str, date: &str, entry: &str) -> std::io::Result<String> {
    let eol = line_ending(current);
    let segments: Vec<&str> = current.split_inclusive('\n').collect();
    let lines: Vec<&str> = current.lines().collect();
    let Some(heading_at) = lines.iter().position(|line| line.starts_with("# ")) else {
        return Err(invalid("the log carries no title heading".to_owned()));
    };
    let bullet = format!("- {entry}");
    let Some(first) = lines.iter().position(|line| line.starts_with("## ")) else {
        // No dated section yet, so the first one opens under the
        // title heading.
        let mut out = String::new();
        out.push_str(&segments[..=heading_at].concat());
        if !out.ends_with('\n') {
            out.push_str(eol);
        }
        out.push_str(eol);
        out.push_str("## ");
        out.push_str(date);
        out.push_str(eol);
        out.push_str(eol);
        out.push_str(&bullet);
        out.push_str(eol);
        out.push_str(&segments[heading_at + 1..].concat());
        if !out.ends_with('\n') {
            out.push_str(eol);
        }
        return Ok(out);
    };
    let newest = lines[first].trim_start_matches('#').trim();
    if !is_calendar_date(newest) {
        return Err(invalid(format!(
            "the log's newest section heading {newest} is not a date"
        )));
    }
    if date < newest {
        return Err(invalid(format!(
            "the entry date {date} is older than the log's newest section {newest}"
        )));
    }
    if date > newest {
        let mut out = String::new();
        out.push_str(&segments[..first].concat());
        if !out.ends_with('\n') {
            out.push_str(eol);
        }
        out.push_str("## ");
        out.push_str(date);
        out.push_str(eol);
        out.push_str(eol);
        out.push_str(&bullet);
        out.push_str(eol);
        out.push_str(eol);
        out.push_str(&segments[first..].concat());
        if !out.ends_with('\n') {
            out.push_str(eol);
        }
        return Ok(out);
    }
    // The date equals the newest section's date, so the entry joins
    // that section after its last entry, or after its last content
    // line when the section carries no entries yet.
    let section_end = lines[first + 1..]
        .iter()
        .position(|line| line.starts_with("## "))
        .map_or(lines.len(), |at| first + 1 + at);
    let last_entry = (first + 1..section_end)
        .rev()
        .find(|&at| lines[at].starts_with("- "));
    let join_at = last_entry.unwrap_or_else(|| {
        (first + 1..section_end)
            .rev()
            .find(|&at| !lines[at].trim().is_empty())
            .unwrap_or(first)
    });
    let mut out = String::new();
    out.push_str(&segments[..=join_at].concat());
    if !out.ends_with('\n') {
        out.push_str(eol);
    }
    out.push_str(&bullet);
    out.push_str(eol);
    out.push_str(&segments[join_at + 1..].concat());
    if !out.ends_with('\n') {
        out.push_str(eol);
    }
    Ok(out)
}
