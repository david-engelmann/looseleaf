//! Writing frontmatter in the looseleaf subset.
//!
//! Two primitives, both byte-stable in the sense the synthesis
//! contract requires. The serializer renders a parsed frontmatter to
//! the canonical block form, so the same value always renders to the
//! same bytes. The upsert edits one key in an existing document by
//! replacing only that key's lines, so every other line of the file,
//! including keys this tool does not know, stays byte-identical.
//!
//! The canonical form uses block sequences and block mappings, never
//! flow form. A document that arrives with flow entries keeps them
//! until its owner edits that key, because the upsert touches nothing
//! it was not asked to touch.

use std::path::Path;

use crate::frontmatter::{Frontmatter, Value};

/// Render a scalar in the subset, quoting only when the plain form
/// would not parse back to the same text.
#[must_use]
pub fn render_scalar(text: &str) -> String {
    if plain_safe(text) {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// The line ending a document uses, CRLF when it carries any and
/// LF otherwise. Shared by every writer that inserts lines into an
/// existing body, so the rule cannot drift between them.
pub(crate) fn line_ending(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

/// An `InvalidInput` error with the given message. The writer
/// modules share this constructor so a caller distinguishing on
/// error kind gets the same answer from each of them.
pub(crate) fn invalid_input(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

/// A `NotFound` error with the given message, the sibling of
/// `invalid_input` for names a bundle does not carry.
pub(crate) fn not_found(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::NotFound, message)
}

fn plain_safe(text: &str) -> bool {
    if text.is_empty() || text.trim() != text {
        return false;
    }
    let first = text.chars().next().unwrap_or(' ');
    if matches!(
        first,
        '"' | '\'' | '[' | ']' | '{' | '}' | '#' | '|' | '>' | '&' | '*' | '!'
    ) {
        return false;
    }
    if text.starts_with("- ") || text == "-" {
        return false;
    }
    if text.contains(": ") || text.ends_with(':') || text.contains(" #") {
        return false;
    }
    !text.contains('\n') && !text.contains('\t') && !text.contains('\r')
}

fn render_entry(lines: &mut Vec<String>, indent: usize, key: &str, value: &Value) {
    let pad = " ".repeat(indent);
    match value {
        Value::Scalar(text) => {
            lines.push(format!("{pad}{key}: {}", render_scalar(text)));
        }
        Value::List(items) => {
            if items.is_empty() {
                // An empty collection renders in flow form, because a
                // bare key would reparse as an empty scalar and the
                // value would be silently destroyed.
                lines.push(format!("{pad}{key}: []"));
                return;
            }
            lines.push(format!("{pad}{key}:"));
            render_list(lines, indent + 2, items);
        }
        Value::Map(entries) => {
            if entries.is_empty() {
                lines.push(format!("{pad}{key}: {{}}"));
                return;
            }
            lines.push(format!("{pad}{key}:"));
            for (child_key, child_value) in entries {
                render_entry(lines, indent + 2, child_key, child_value);
            }
        }
    }
}

fn render_list(lines: &mut Vec<String>, indent: usize, items: &[Value]) {
    let pad = " ".repeat(indent);
    for item in items {
        match item {
            Value::Scalar(text) => {
                lines.push(format!("{pad}- {}", render_scalar(text)));
            }
            Value::List(nested) => {
                if nested.is_empty() {
                    lines.push(format!("{pad}- []"));
                    continue;
                }
                lines.push(format!("{pad}-"));
                render_list(lines, indent + 2, nested);
            }
            Value::Map(entries) => {
                if entries.is_empty() {
                    lines.push(format!("{pad}- {{}}"));
                    continue;
                }
                let mut first = true;
                for (key, value) in entries {
                    if first {
                        // The first entry rides on the dash line.
                        // It renders through the same entry path as
                        // every other entry, at the entry indent,
                        // and only its first line trades the entry
                        // padding for the dash.
                        let mut rendered = Vec::new();
                        render_entry(&mut rendered, indent + 2, key, value);
                        // Invariant the slice below rests on,
                        // every line render_entry pushes begins
                        // with exactly indent + 2 spaces.
                        for (index, line) in rendered.into_iter().enumerate() {
                            if index == 0 {
                                lines.push(format!("{pad}- {}", &line[indent + 2..]));
                            } else {
                                lines.push(line);
                            }
                        }
                        first = false;
                    } else {
                        render_entry(lines, indent + 2, key, value);
                    }
                }
            }
        }
    }
}

/// Serialize a parsed frontmatter to the canonical block form, with
/// no fences. The output ends with a newline.
#[must_use]
pub fn serialize_frontmatter(frontmatter: &Frontmatter) -> String {
    let mut lines: Vec<String> = Vec::new();
    for (key, value) in frontmatter.entries() {
        render_entry(&mut lines, 0, key, value);
    }
    let mut out = lines.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

/// Render one top-level entry in canonical form, with no trailing
/// newline handling beyond the lines themselves.
fn render_top_entry(key: &str, value: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    render_entry(&mut lines, 0, key, value);
    lines
}

/// The outcome of an upsert, so a caller can tell a change from a
/// no-op and skip the write entirely when nothing moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upsert {
    /// The full document text after the edit.
    pub text: String,
    /// Whether the text differs from the input.
    pub changed: bool,
}

/// Set one top-level frontmatter key in a Markdown document.
///
/// The key's existing entry, including any block lines under it, is
/// replaced with the canonical rendering of the new value. When the
/// key is absent it is appended at the end of the frontmatter. Every
/// other line of the document, frontmatter and body, is preserved
/// byte for byte, which is what keeps keys this tool does not know
/// intact across edits it makes for other reasons.
///
/// # Errors
///
/// Returns an error string when the document has no frontmatter
/// fence, the fence is never closed, or the existing frontmatter does
/// not parse in the subset, because an edit on a document whose shape
/// is not understood would be a guess.
pub fn upsert_frontmatter(text: &str, key: &str, value: &Value) -> Result<Upsert, String> {
    let document = crate::frontmatter::parse_document(text);
    if let Some(error) = &document.parse_error {
        return Err(format!("frontmatter does not parse in the subset, {error}"));
    }
    if document.frontmatter.is_none() {
        return Err("the document carries no frontmatter to edit".to_owned());
    }
    // Work on line segments that keep their own endings, so a CRLF
    // document keeps CRLF endings everywhere, including the lines this
    // edit does not touch. Splitting with str::lines and rejoining
    // would silently rewrite every ending in the file.
    let eol = if text.starts_with("---\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let segments: Vec<&str> = text.split_inclusive('\n').collect();
    let bare = bare_segment;
    // The opening fence is segment 0. Find the closing fence.
    let Some(close) = segments
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, segment)| bare(segment).trim_end() == "---")
        .map(|(index, _)| index)
    else {
        return Err("frontmatter fence is never closed".to_owned());
    };
    // Locate the existing entry for the key at indent zero.
    let mut start: Option<usize> = None;
    let mut end: usize = close;
    for (index, segment) in segments.iter().enumerate().take(close).skip(1) {
        let line = bare(segment);
        if line.is_empty() || line.starts_with(' ') {
            continue;
        }
        if let Some((found, _)) = split_top_entry(line) {
            if start.is_some() {
                end = index;
                break;
            }
            if found == key {
                start = Some(index);
            }
        }
    }
    let rendered: Vec<String> = render_top_entry(key, value)
        .into_iter()
        .map(|line| format!("{line}{eol}"))
        .collect();
    let mut out = String::with_capacity(text.len());
    if let Some(begin) = start {
        for segment in &segments[..begin] {
            out.push_str(segment);
        }
        for line in &rendered {
            out.push_str(line);
        }
        for segment in &segments[end..] {
            out.push_str(segment);
        }
    } else {
        for segment in &segments[..close] {
            out.push_str(segment);
        }
        for line in &rendered {
            out.push_str(line);
        }
        for segment in &segments[close..] {
            out.push_str(segment);
        }
    }
    let changed = out != text;
    Ok(Upsert { text: out, changed })
}

/// A line segment without its ending, for fence and entry detection
/// that must work the same on LF and CRLF documents.
pub(crate) fn bare_segment(segment: &str) -> &str {
    segment.trim_end_matches(['\n', '\r'])
}

/// Write a document through a sibling temporary file and a rename
/// over the target, so a failed write never truncates the file
/// that was there. Every writer in the crate writes this way.
pub(crate) fn write_via_temp(target: &Path, text: &str) -> std::io::Result<()> {
    let temp = target.with_extension("md.tmp-looseleaf");
    std::fs::write(&temp, text)?;
    std::fs::rename(&temp, target)
}

/// Split a top-level frontmatter line into key and remainder, when it
/// is an entry line. The key rule is the parser's own, shared through
/// `is_valid_key`, so the reader and the writer cannot diverge.
fn split_top_entry(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(':')?;
    let key = &line[..colon];
    if !crate::frontmatter::is_valid_key(key) {
        return None;
    }
    Some((key, &line[colon + 1..]))
}

#[cfg(test)]
mod tests {
    use super::{render_scalar, serialize_frontmatter, upsert_frontmatter};
    use crate::frontmatter::{Value, parse_document};

    fn fm(text: &str) -> crate::frontmatter::Frontmatter {
        parse_document(&format!("---\n{text}---\n"))
            .frontmatter
            .expect("frontmatter should parse")
    }

    #[test]
    fn round_trip_holds_for_the_prototype_shapes() {
        let source = "type: Daily Log\ntitle: \"Daily log 2026-09-16\"\nx-daybook-claim-count: 12\ngenerated: { by: \"process:memory-extraction\", at: \"2026-09-16T00:00:00Z\" }\nsources:\n  - id: memory-md\n    resource: MEMORY.md\n    title: Source file\n";
        let parsed = fm(source);
        let rendered = serialize_frontmatter(&parsed);
        let reparsed = fm(&rendered);
        assert_eq!(parsed, reparsed);
        let rendered_again = serialize_frontmatter(&reparsed);
        assert_eq!(
            rendered, rendered_again,
            "serialization must be a fixed point"
        );
    }

    #[test]
    fn scalar_quoting_is_minimal_and_safe() {
        assert_eq!(render_scalar("Daily Log"), "Daily Log");
        assert_eq!(render_scalar("process:x"), "process:x");
        assert_eq!(render_scalar("has: colon"), "\"has: colon\"");
        assert_eq!(render_scalar(""), "\"\"");
        // An interior double quote parses back unchanged in the plain
        // form, so D13's rule, quote only when the plain form would
        // not parse back to the same text, leaves it plain.
        assert_eq!(render_scalar("say \"hi\""), "say \"hi\"");
        assert_eq!(render_scalar("a\rb"), "\"a\\rb\"");
    }

    #[test]
    fn empty_collections_survive_a_round_trip() {
        let parsed = fm("type: Person\ntags: []\ngenerated: {}\n");
        let rendered = serialize_frontmatter(&parsed);
        assert!(rendered.contains("tags: []"), "got:\n{rendered}");
        assert!(rendered.contains("generated: {}"), "got:\n{rendered}");
        let reparsed = fm(&rendered);
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn upsert_preserves_crlf_line_endings() {
        let doc = "---\r\ntype: Person\r\ntitle: \"Alan\"\r\n---\r\n\r\n# Alan\r\n";
        let value = Value::Scalar("Group".to_owned());
        let result = upsert_frontmatter(doc, "type", &value).expect("upsert should work");
        assert!(result.changed);
        assert!(
            result.text.contains("type: Group\r\n"),
            "the edited line takes the document ending, got:\n{:?}",
            result.text
        );
        assert!(
            result.text.contains("title: \"Alan\"\r\n"),
            "untouched lines keep their endings, got:\n{:?}",
            result.text
        );
        assert!(!result.text.replace("\r\n", "").contains('\n'));
        let noop = upsert_frontmatter(&result.text, "type", &value).expect("second upsert");
        assert!(!noop.changed, "a same-value upsert on CRLF is a no-op");
    }

    #[test]
    fn golden_bytes_for_the_list_dispatch_matrix() {
        // Byte pins over the shapes the list renderer
        // special-cases, captured before the render dispatch
        // consolidation so the consolidation must hold them.
        let cases: [(&str, &str); 9] = [
            ("v: [{ by: a, at: b }]\n", "v:\n  - by: a\n    at: b\n"),
            ("v: [{ k: [] }]\n", "v:\n  - k: []\n"),
            ("v: [{ k: [x, y] }]\n", "v:\n  - k:\n      - x\n      - y\n"),
            ("v: [{ k: {} }]\n", "v:\n  - k: {}\n"),
            (
                "v: [{ k: { a: \"1\", b: \"2\" } }]\n",
                "v:\n  - k:\n      a: 1\n      b: 2\n",
            ),
            ("v: [[{ by: a }]]\n", "v:\n  -\n    - by: a\n"),
            ("v: [x, [y, z]]\n", "v:\n  - x\n  -\n    - y\n    - z\n"),
            ("v: [{ by: \"a: b\" }]\n", "v:\n  - by: \"a: b\"\n"),
            (
                "v: [{ by: a }, { k: [x, y] }]\n",
                "v:\n  - by: a\n  - k:\n      - x\n      - y\n",
            ),
        ];
        for (index, (input, expected)) in cases.iter().enumerate() {
            let rendered = serialize_frontmatter(&fm(input));
            assert_eq!(&rendered, expected, "shape {index} from {input}");
        }
    }

    #[test]
    fn upsert_reports_its_error_cases() {
        let value = Value::Scalar("x".to_owned());
        assert!(upsert_frontmatter("# No frontmatter\n", "type", &value).is_err());
        assert!(upsert_frontmatter("---\ntype: Person\n", "type", &value).is_err());
        assert!(upsert_frontmatter("---\ntype: [unclosed\n---\n", "type", &value).is_err());
    }

    #[test]
    fn upsert_preserves_unknown_keys_byte_for_byte() {
        let doc = "---\ntype: Person\nx-daybook-rank: 5\ngenerated: { by: \"process:old\", at: \"2026-01-01T00:00:00Z\" }\ntitle: \"Alan\"\n---\n\n# Alan\n";
        let value = Value::Map(vec![
            ("by".to_owned(), Value::Scalar("process:new".to_owned())),
            (
                "at".to_owned(),
                Value::Scalar("2026-11-21T00:00:00Z".to_owned()),
            ),
        ]);
        let result = upsert_frontmatter(doc, "generated", &value).expect("upsert should work");
        assert!(result.changed);
        assert!(
            result.text.contains("x-daybook-rank: 5\n"),
            "the unknown key line must survive untouched, got:\n{}",
            result.text
        );
        assert!(result.text.contains("type: Person\n"));
        assert!(
            result
                .text
                .contains("generated:\n  by: process:new\n  at: 2026-11-21T00:00:00Z\n")
        );
        assert!(result.text.ends_with("# Alan\n"));
    }

    #[test]
    fn upsert_appends_a_missing_key_before_the_fence() {
        let doc = "---\ntype: Person\ntitle: \"Alan\"\n---\n\nBody.\n";
        let value = Value::Scalar("2026-11-21".to_owned());
        let result = upsert_frontmatter(doc, "x-looseleaf-checked", &value).expect("upsert");
        assert!(
            result
                .text
                .contains("title: \"Alan\"\nx-looseleaf-checked: 2026-11-21\n---")
        );
    }

    #[test]
    fn upsert_of_the_same_value_is_a_no_op_when_canonical() {
        let doc = "---\ntype: Person\ntitle: \"Alan\"\n---\n\nBody.\n";
        let value = Value::Scalar("Person".to_owned());
        let result = upsert_frontmatter(doc, "type", &value).expect("upsert");
        assert!(!result.changed);
        assert_eq!(result.text, doc);
    }
}
