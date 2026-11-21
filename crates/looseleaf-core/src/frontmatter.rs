//! Parsing for the looseleaf frontmatter subset of YAML.
//!
//! The subset is defined in `docs/PROFILE.md` section 5. It covers
//! top-level `key: value` entries, plain and quoted scalars, flow
//! mappings and sequences on one line, block sequences of scalars or
//! mappings, and block mappings nested one level under a key. Anything
//! outside the subset is rejected with an error rather than guessed at.

/// A value in the frontmatter subset. Scalars keep their text form,
/// because the profile checks text, not YAML typing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A plain, single-quoted or double-quoted scalar, unquoted.
    Scalar(String),
    /// A block or flow sequence.
    List(Vec<Value>),
    /// A block or flow mapping, entries in document order.
    Map(Vec<(String, Value)>),
}

impl Value {
    /// The scalar text when this value is a scalar.
    #[must_use]
    pub fn as_scalar(&self) -> Option<&str> {
        match self {
            Value::Scalar(text) => Some(text),
            Value::List(_) | Value::Map(_) => None,
        }
    }

    /// The entries when this value is a mapping.
    #[must_use]
    pub fn as_map(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Map(entries) => Some(entries),
            Value::Scalar(_) | Value::List(_) => None,
        }
    }

    /// Look up a key in a mapping value.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_map()?
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }
}

/// Parsed frontmatter, entries in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    entries: Vec<(String, Value)>,
}

impl Frontmatter {
    /// Every entry, in document order.
    #[must_use]
    pub fn entries(&self) -> &[(String, Value)] {
        &self.entries
    }

    /// Look up a top-level key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// Look up a top-level key and return its scalar text.
    #[must_use]
    pub fn get_scalar(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_scalar)
    }
}

/// A Markdown document split into frontmatter and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// The parsed frontmatter, when the document opens with a fence
    /// and the block parses inside the subset.
    pub frontmatter: Option<Frontmatter>,
    /// The parse failure, when the document opens with a fence but the
    /// block is malformed or outside the subset.
    pub parse_error: Option<String>,
    /// Everything after the closing fence, or the whole text when the
    /// document has no frontmatter.
    pub body: String,
}

/// Split a Markdown document into frontmatter and body and parse the
/// frontmatter block when present.
#[must_use]
pub fn parse_document(text: &str) -> Document {
    let Some(rest) = strip_opening_fence(text) else {
        return Document {
            frontmatter: None,
            parse_error: None,
            body: text.to_owned(),
        };
    };
    let mut block_lines: Vec<&str> = Vec::new();
    let mut body_start: Option<usize> = None;
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        offset += line.len();
        if trimmed == "---" {
            body_start = Some(offset);
            break;
        }
        block_lines.push(trimmed);
    }
    let Some(body_start) = body_start else {
        return Document {
            frontmatter: None,
            parse_error: Some("frontmatter fence is never closed".to_owned()),
            body: String::new(),
        };
    };
    let body = rest[body_start..].to_owned();
    match parse_frontmatter(&block_lines) {
        Ok(frontmatter) => Document {
            frontmatter: Some(frontmatter),
            parse_error: None,
            body,
        },
        Err(message) => Document {
            frontmatter: None,
            parse_error: Some(message),
            body,
        },
    }
}

fn strip_opening_fence(text: &str) -> Option<&str> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    Some(rest)
}

/// Parse a scalar in the subset, removing quotes and resolving the
/// escapes the subset allows.
#[must_use]
pub fn parse_scalar(raw: &str) -> String {
    let text = raw.trim();
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        let inner = &text[1..text.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(other) => out.push(other),
                    None => out.push('\\'),
                }
            } else {
                out.push(ch);
            }
        }
        return out;
    }
    if text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'') {
        return text[1..text.len() - 1].replace("''", "'");
    }
    text.to_owned()
}

struct Line {
    number: usize,
    indent: usize,
    content: String,
}

fn is_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

/// Split an entry line into its key and the inline remainder, when the
/// line is a mapping entry in the subset.
fn split_entry(content: &str) -> Option<(&str, &str)> {
    let colon = content.find(':')?;
    let key = &content[..colon];
    if key.is_empty() || !key.chars().all(is_key_char) {
        return None;
    }
    let rest = &content[colon + 1..];
    if rest.is_empty() {
        return Some((key, ""));
    }
    let rest = rest.strip_prefix(' ')?;
    Some((key, rest))
}

fn parse_frontmatter(lines: &[&str]) -> Result<Frontmatter, String> {
    let mut logical: Vec<Line> = Vec::new();
    for (index, raw) in lines.iter().enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        if raw.contains('\t') {
            return Err(format!(
                "line {} uses a tab, which is outside the subset",
                index + 1
            ));
        }
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        logical.push(Line {
            number: index + 1,
            indent,
            content: raw.trim_start().to_owned(),
        });
    }
    let mut pos = 0usize;
    let entries = parse_map(&logical, &mut pos, 0, 1)?;
    if pos != logical.len() {
        return Err(format!(
            "line {} does not fit the mapping above it",
            logical[pos].number
        ));
    }
    Ok(Frontmatter { entries })
}

const MAX_DEPTH: usize = 4;

fn parse_map(
    lines: &[Line],
    pos: &mut usize,
    indent: usize,
    depth: usize,
) -> Result<Vec<(String, Value)>, String> {
    if depth > MAX_DEPTH {
        return Err("frontmatter nests deeper than the subset allows".to_owned());
    }
    let mut entries: Vec<(String, Value)> = Vec::new();
    while *pos < lines.len() {
        let line = &lines[*pos];
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(format!("line {} is indented past its mapping", line.number));
        }
        if line.content.starts_with("- ") || line.content == "-" {
            break;
        }
        let Some((key, rest)) = split_entry(&line.content) else {
            return Err(format!("line {} is not a key and value entry", line.number));
        };
        if entries.iter().any(|(name, _)| name == key) {
            return Err(format!("duplicate key {key} in one mapping"));
        }
        let key = key.to_owned();
        *pos += 1;
        let value = if rest.is_empty() {
            parse_block_value(lines, pos, indent, depth)?
        } else {
            parse_flow(rest).map_err(|message| format!("line {} {message}", line.number))?
        };
        entries.push((key, value));
    }
    Ok(entries)
}

fn parse_block_value(
    lines: &[Line],
    pos: &mut usize,
    parent_indent: usize,
    depth: usize,
) -> Result<Value, String> {
    if *pos >= lines.len() || lines[*pos].indent <= parent_indent {
        return Ok(Value::Scalar(String::new()));
    }
    let child_indent = lines[*pos].indent;
    if lines[*pos].content.starts_with("- ") || lines[*pos].content == "-" {
        parse_list(lines, pos, child_indent, depth + 1)
    } else {
        let entries = parse_map(lines, pos, child_indent, depth + 1)?;
        Ok(Value::Map(entries))
    }
}

fn parse_list(
    lines: &[Line],
    pos: &mut usize,
    indent: usize,
    depth: usize,
) -> Result<Value, String> {
    if depth > MAX_DEPTH {
        return Err("frontmatter nests deeper than the subset allows".to_owned());
    }
    let mut items: Vec<Value> = Vec::new();
    while *pos < lines.len() {
        let line = &lines[*pos];
        if line.indent != indent || !(line.content.starts_with("- ") || line.content == "-") {
            break;
        }
        let item_text = line.content.strip_prefix("- ").unwrap_or("").to_owned();
        let item_number = line.number;
        *pos += 1;
        if item_text.is_empty() {
            items.push(Value::Scalar(String::new()));
            continue;
        }
        if let Some((key, rest)) = split_entry(&item_text) {
            // A mapping item. Its first entry rides on the dash line and
            // its remaining entries align two columns past the dash.
            let mut entries: Vec<(String, Value)> = Vec::new();
            let key = key.to_owned();
            let first = if rest.is_empty() {
                parse_block_value(lines, pos, indent + 2, depth)?
            } else {
                parse_flow(rest).map_err(|m| format!("line {item_number} {m}"))?
            };
            entries.push((key, first));
            let more = parse_map(lines, pos, indent + 2, depth + 1)?;
            entries.extend(more);
            items.push(Value::Map(entries));
        } else {
            let value = parse_flow(&item_text).map_err(|m| format!("line {item_number} {m}"))?;
            items.push(value);
        }
    }
    Ok(Value::List(items))
}

/// Parse an inline value, a flow mapping, a flow sequence or a scalar.
fn parse_flow(text: &str) -> Result<Value, String> {
    let text = text.trim();
    if text.starts_with('{') {
        if !text.ends_with('}') {
            return Err("has an unclosed flow mapping".to_owned());
        }
        let inner = &text[1..text.len() - 1];
        let mut entries: Vec<(String, Value)> = Vec::new();
        for part in split_top_level(inner, ',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let Some((key, rest)) = split_entry(part) else {
                return Err(format!(
                    "has a flow entry that is not a key and value, {part}"
                ));
            };
            if entries.iter().any(|(name, _)| name == key) {
                return Err(format!("repeats the flow key {key}"));
            }
            entries.push((key.to_owned(), parse_flow(rest)?));
        }
        return Ok(Value::Map(entries));
    }
    if text.starts_with('[') {
        if !text.ends_with(']') {
            return Err("has an unclosed flow sequence".to_owned());
        }
        let inner = &text[1..text.len() - 1];
        let mut items = Vec::new();
        for part in split_top_level(inner, ',') {
            let part = part.trim();
            if !part.is_empty() {
                items.push(parse_flow(part)?);
            }
        }
        return Ok(Value::List(items));
    }
    Ok(Value::Scalar(parse_scalar(text)))
}

/// Split on a separator that appears outside quotes and brackets.
fn split_top_level(text: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' if in_double => {
                current.push(ch);
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '\'' if !in_double => {
                in_single = !in_single;
                current.push(ch);
            }
            '"' if !in_single => {
                in_double = !in_double;
                current.push(ch);
            }
            '[' | '{' if !in_single && !in_double => {
                depth += 1;
                current.push(ch);
            }
            ']' | '}' if !in_single && !in_double => {
                depth -= 1;
                current.push(ch);
            }
            _ if ch == separator && depth == 0 && !in_single && !in_double => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    parts.push(current);
    parts
}

#[cfg(test)]
mod tests {
    use super::{Value, parse_document, parse_scalar};

    fn scalar(value: &Value) -> &str {
        value.as_scalar().expect("expected a scalar")
    }

    #[test]
    fn parses_the_prototype_shapes() {
        let text = "---\ntype: Daily Log\ntitle: \"Daily log 2026-09-16\"\nx-daybook-claim-count: 12\ngenerated: { by: \"process:memory-extraction\", at: \"2026-09-16T00:00:00Z\" }\nsources:\n  - id: memory-md\n    resource: MEMORY.md\n    title: Source file\n---\n\nBody.\n";
        let doc = parse_document(text);
        let fm = doc.frontmatter.expect("frontmatter should parse");
        assert_eq!(fm.get_scalar("type"), Some("Daily Log"));
        assert_eq!(fm.get_scalar("x-daybook-claim-count"), Some("12"));
        let generated = fm.get("generated").expect("generated present");
        assert_eq!(
            generated.get("by").map(scalar),
            Some("process:memory-extraction")
        );
        let sources = fm.get("sources").expect("sources present");
        let Value::List(items) = sources else {
            panic!("sources should be a list");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("resource").map(scalar), Some("MEMORY.md"));
        assert_eq!(doc.body, "\nBody.\n");
    }

    #[test]
    fn parses_block_mapping_and_flow_list() {
        let text = "---\ntype: Person\ngenerated:\n  by: human\n  at: \"2026-11-20T00:00:00Z\"\ntags: [a, b]\n---\n";
        let doc = parse_document(text);
        let fm = doc.frontmatter.expect("frontmatter should parse");
        let generated = fm.get("generated").expect("generated present");
        assert_eq!(generated.get("by").map(scalar), Some("human"));
        let Value::List(tags) = fm.get("tags").expect("tags present") else {
            panic!("tags should be a list");
        };
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn rejects_tabs_and_unclosed_fences() {
        let tabbed = parse_document("---\ntype: Person\n\tbad: 1\n---\n");
        assert!(tabbed.parse_error.is_some());
        let unclosed = parse_document("---\ntype: Person\n");
        assert!(unclosed.parse_error.is_some());
        let plain = parse_document("# Just a body\n");
        assert!(plain.frontmatter.is_none());
        assert!(plain.parse_error.is_none());
    }

    #[test]
    fn scalar_unquoting() {
        assert_eq!(parse_scalar("\"a \\\"b\\\" c\""), "a \"b\" c");
        assert_eq!(parse_scalar("'it''s'"), "it's");
        assert_eq!(parse_scalar("plain text"), "plain text");
    }
}
