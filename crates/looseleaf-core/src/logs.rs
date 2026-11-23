//! Appending dated entries to the bundle's log files.
//!
//! A log is append-only history under dated section headings,
//! newest first. The mechanical step the synthesis contract needs
//! is a dated append that never rewrites existing sections. An
//! append whose date equals the newest section's date joins that
//! section. A newer date opens a new section at the top. An older
//! date is refused, because placing it would mean editing history
//! into order rather than appending to it. The date is always a
//! command argument, never a clock the tool reads.
//!
//! The profile names three logs, the bundle log at the root and
//! the curated and daily layer logs. Appending to any other layer's
//! log is refused, because the profile gives no other layer a log
//! to append to. A missing log is created with the heading the
//! clean fixture carries, since a checker finding about the missing
//! file should not block the append that repairs it.

use std::fs;
use std::path::Path;

use crate::bundle::Layer;
use crate::check::is_calendar_date;

/// Which of the bundle's logs an append targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogScope {
    /// The bundle log, `log.md` at the root.
    Root,
    /// A layer log, `log.md` inside the layer's directory.
    Layer(Layer),
}

impl LogScope {
    /// The bundle-relative path of the log this scope names.
    fn rel_path(self) -> String {
        match self {
            LogScope::Root => "log.md".to_owned(),
            LogScope::Layer(layer) => match layer.dir_name() {
                Some(dir) => format!("{dir}/log.md"),
                None => "log.md".to_owned(),
            },
        }
    }

    /// The title heading a log created for this scope carries.
    fn heading(self) -> &'static str {
        match self {
            LogScope::Root => "# Bundle log",
            LogScope::Layer(Layer::Curated) => "# Curated layer log",
            LogScope::Layer(Layer::Daily) => "# Daily layer log",
            LogScope::Layer(_) => "# Layer log",
        }
    }

    /// Whether the profile gives this scope a log at all.
    fn allowed(self) -> bool {
        matches!(
            self,
            LogScope::Root | LogScope::Layer(Layer::Curated | Layer::Daily)
        )
    }
}

fn invalid(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

/// Append one entry under a date in the log the scope names,
/// returning the bundle-relative path of the log written. The
/// entry is a single line, rendered as one bullet under the dated
/// section. The write goes to a sibling temporary file and is
/// renamed over the target.
///
/// # Errors
///
/// Returns an `InvalidInput` error when the scope names a log the
/// profile does not give, when the date is not a real calendar
/// date, when the entry is empty or spans lines, when the date is
/// older than the log's newest section, or when the existing log
/// carries no title heading or a section heading that is not a
/// date. Returns an I/O error when the log cannot be read or
/// written.
pub fn append_log(
    root: &Path,
    scope: LogScope,
    date: &str,
    entry: &str,
) -> std::io::Result<String> {
    if !scope.allowed() {
        return Err(invalid(
            "the profile gives logs only to the bundle root and the curated and daily layers"
                .to_owned(),
        ));
    }
    if !is_calendar_date(date) {
        return Err(invalid(format!("{date} is not a real calendar date")));
    }
    let entry = entry.trim();
    if entry.is_empty() || entry.contains(['\n', '\r']) {
        return Err(invalid("a log entry is one non-empty line".to_owned()));
    }
    let rel = scope.rel_path();
    let target = root.join(&rel);
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
    Ok(rel)
}

/// Render a log with one entry appended, per the rules in the
/// module documentation. The existing text is carried line by line
/// and rejoined with the document's own line ending, so nothing
/// already in the log is rewritten beyond the join.
fn render_append(current: &str, date: &str, entry: &str) -> std::io::Result<String> {
    let eol = if current.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let lines: Vec<&str> = current.lines().collect();
    let Some(heading_at) = lines.iter().position(|line| line.starts_with("# ")) else {
        return Err(invalid("the log carries no title heading".to_owned()));
    };
    let bullet = format!("- {entry}");
    let first_section = lines.iter().position(|line| line.starts_with("## "));
    let Some(first) = first_section else {
        let section = format!("## {date}");
        let mut out: Vec<&str> = lines[..=heading_at].to_vec();
        out.push("");
        out.push(&section);
        out.push("");
        out.push(&bullet);
        out.extend_from_slice(&lines[heading_at + 1..]);
        let mut text = out.join(eol);
        text.push_str(eol);
        return Ok(text);
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
        let section = [format!("## {date}"), String::new(), bullet];
        let mut out: Vec<String> = lines[..first].iter().map(ToString::to_string).collect();
        out.extend(section);
        out.push(String::new());
        out.extend(lines[first..].iter().map(ToString::to_string));
        let mut text = out.join(eol);
        text.push_str(eol);
        return Ok(text);
    }
    // The date equals the newest section's date, so the entry joins
    // that section after its last content line.
    let section_end = lines[first + 1..]
        .iter()
        .position(|line| line.starts_with("## "))
        .map_or(lines.len(), |at| first + 1 + at);
    let last_content = (first + 1..section_end)
        .rev()
        .find(|&at| !lines[at].trim().is_empty())
        .unwrap_or(first);
    let mut out: Vec<&str> = lines[..=last_content].to_vec();
    out.push(&bullet);
    out.extend_from_slice(&lines[last_content + 1..]);
    let mut text = out.join(eol);
    text.push_str(eol);
    Ok(text)
}
