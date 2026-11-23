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
            other => out.push(other),
        }
    }
    out.push('"');
    out
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
    !text.contains('\n') && !text.contains('\t')
}

fn render_entry(lines: &mut Vec<String>, indent: usize, key: &str, value: &Value) {
    let pad = " ".repeat(indent);
    match value {
        Value::Scalar(text) => {
            lines.push(format!("{pad}{key}: {}", render_scalar(text)));
        }
        Value::List(items) => {
            lines.push(format!("{pad}{key}:"));
            render_list(lines, indent + 2, items);
        }
        Value::Map(entries) => {
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
                lines.push(format!("{pad}-"));
                render_list(lines, indent + 2, nested);
            }
            Value::Map(entries) => {
                let mut first = true;
                for (key, value) in entries {
                    if first {
                        // The first entry rides on the dash line.
                        match value {
                            Value::Scalar(text) => {
                                lines.push(format!("{pad}- {key}: {}", render_scalar(text)));
                            }
                            Value::List(nested) => {
                                lines.push(format!("{pad}- {key}:"));
                                render_list(lines, indent + 4, nested);
                            }
                            Value::Map(nested_entries) => {
                                lines.push(format!("{pad}- {key}:"));
                                for (k, v) in nested_entries {
                                    render_entry(lines, indent + 4, k, v);
                                }
                            }
                        }
                        first = false;
                    } else {
                        render_entry(lines, indent + 2, key, value);
                    }
                }
                if entries.is_empty() {
                    lines.push(format!("{pad}-"));
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
    let lines: Vec<&str> = text.lines().collect();
    // The opening fence is line 0. Find the closing fence.
    let Some(close) = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim_end_matches('\r') == "---")
        .map(|(index, _)| index)
    else {
        return Err("frontmatter fence is never closed".to_owned());
    };
    // Locate the existing entry for the key at indent zero.
    let mut start: Option<usize> = None;
    let mut end: usize = close;
    for (index, line) in lines.iter().enumerate().take(close).skip(1) {
        let line = *line;
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
    let rendered = render_top_entry(key, value);
    let mut out_lines: Vec<String> = Vec::new();
    if let Some(begin) = start {
        for line in &lines[..begin] {
            out_lines.push((*line).to_owned());
        }
        out_lines.extend(rendered);
        for line in &lines[end..] {
            out_lines.push((*line).to_owned());
        }
    } else {
        for line in &lines[..close] {
            out_lines.push((*line).to_owned());
        }
        out_lines.extend(rendered);
        for line in &lines[close..] {
            out_lines.push((*line).to_owned());
        }
    }
    let mut out = out_lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    let changed = out != text;
    Ok(Upsert { text: out, changed })
}

/// Split a top-level frontmatter line into key and remainder, when it
/// is an entry line. Mirrors the parser's key rules.
fn split_top_entry(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(':')?;
    let key = &line[..colon];
    if key.is_empty()
        || !key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
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
        assert_eq!(render_scalar("say \"hi\""), "\"say \\\"hi\\\"\"");
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
