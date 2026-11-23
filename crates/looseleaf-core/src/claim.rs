//! Scanning for claim blocks in Daily Log and Curated Memory bodies.
//!
//! The convention is defined in `docs/PROFILE.md` section 3. A claim
//! block is a list item whose text opens with a `[kind|confidence]`
//! tag, followed by indented sub-items carrying the claim identity, a
//! verbatim quote, sources, an optional supersedes link and, in the
//! curated layer only, the reverse link the synthesis pass writes.

use crate::frontmatter::parse_scalar;

/// One claim block found in a concept body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    /// The claim identity, when the block carries a `claim` sub-item.
    pub id: Option<String>,
    /// The kind from the opening tag, for example `fact`.
    pub kind: String,
    /// The confidence from the opening tag, for example `high`.
    pub confidence: String,
    /// The claim text after the opening tag.
    pub text: String,
    /// The verbatim quote, when the block carries a `quote` sub-item.
    pub quote: Option<String>,
    /// The sources from the `sources` sub-item, comma separated in the
    /// block and split here.
    pub sources: Vec<String>,
    /// The identity this claim replaces, when the block carries a
    /// `supersedes` sub-item.
    pub supersedes: Option<String>,
    /// The identity of the claim that replaced this one, when the
    /// block carries a `superseded-by` sub-item. The synthesis pass
    /// writes this reverse link into the curated layer only.
    pub superseded_by: Option<String>,
    /// The names of sub-items that appeared more than once in this
    /// block. The first value wins and the repetition is recorded
    /// here so the checker can flag it, because a repeated identity
    /// sub-item silently discarding a claim identity is exactly the
    /// failure the convention exists to prevent.
    pub repeated: Vec<String>,
    /// The one-based line number of the block's opening item, kept for
    /// finding messages.
    pub line: usize,
}

/// Parse an opening item line into kind, confidence and claim text,
/// when the line opens a claim block.
fn parse_opening(line: &str) -> Option<(String, String, String)> {
    let rest = line.strip_prefix("- [")?;
    let close = rest.find("] ")?;
    let tag = &rest[..close];
    let (kind, confidence) = tag.split_once('|')?;
    let kind = kind.trim();
    let confidence = confidence.trim();
    if kind.is_empty() || confidence.is_empty() {
        return None;
    }
    Some((
        kind.to_owned(),
        confidence.to_owned(),
        rest[close + 2..].trim().to_owned(),
    ))
}

/// Which sub-item a line carries, by its index in the seen table.
fn subitem_index(sub: &str) -> Option<usize> {
    if sub.starts_with("claim:") {
        Some(0)
    } else if sub.starts_with("quote:") {
        Some(1)
    } else if sub.starts_with("sources:") {
        Some(2)
    } else if sub.starts_with("supersedes:") {
        Some(3)
    } else if sub.starts_with("superseded-by:") {
        Some(4)
    } else {
        None
    }
}

/// The sub-item names by their index in the seen table.
const SUBITEM_NAMES: [&str; 5] = ["claim", "quote", "sources", "supersedes", "superseded-by"];

/// Scan a concept body for claim blocks, in document order.
///
/// A block's sub-items must be contiguous under the opening item. Any
/// other line, a heading, a fresh top-level item, a paragraph or a
/// blank line, ends the block, so an indented list later in the body
/// can never attach to a claim it merely follows.
#[must_use]
pub fn scan_claims(body: &str) -> Vec<Claim> {
    let mut claims: Vec<Claim> = Vec::new();
    let mut attached = false;
    let mut seen = [false; 5];
    for (index, line) in body.lines().enumerate() {
        if let Some((kind, confidence, text)) = parse_opening(line) {
            claims.push(Claim {
                id: None,
                kind,
                confidence,
                text,
                quote: None,
                sources: Vec::new(),
                supersedes: None,
                superseded_by: None,
                repeated: Vec::new(),
                line: index + 1,
            });
            seen = [false; 5];
            attached = true;
            continue;
        }
        let sub = if attached {
            line.strip_prefix("  - ")
        } else {
            None
        };
        let Some(sub) = sub else {
            attached = false;
            continue;
        };
        let Some(slot) = subitem_index(sub) else {
            continue;
        };
        let Some(current) = claims.last_mut() else {
            continue;
        };
        if seen[slot] {
            current.repeated.push(SUBITEM_NAMES[slot].to_owned());
            continue;
        }
        seen[slot] = true;
        let value = sub.split_once(':').map_or("", |(_, rest)| rest);
        match slot {
            0 => current.id = Some(parse_scalar(value)),
            1 => current.quote = Some(parse_scalar(value)),
            2 => {
                current.sources = value
                    .split(',')
                    .map(parse_scalar)
                    .filter(|part| !part.is_empty())
                    .collect();
            }
            3 => current.supersedes = Some(parse_scalar(value)),
            _ => current.superseded_by = Some(parse_scalar(value)),
        }
    }
    claims
}

#[cfg(test)]
mod tests {
    use super::scan_claims;

    #[test]
    fn scans_the_real_block_shape() {
        let body = "## Verified extraction\n\n- [fact|high] The user's name is David.\n  - claim: f49ece3f572d99160cd755bd6798e855\n  - quote: \"Hey David, I'm your personal agent.\"\n  - sources: message:fee78aee-ff6c-406c-b234-beec38239659\n- [event|high] The user named their assistant \"dog\".\n  - claim: c70da4a9b64721802aedd4ecfa5b3e22\n  - quote: \"dog\"\n  - sources: message:127aa997-6780-4bfb-b753-d34a2cdf3d5b, message:assistant-msg-7eb4622b\n  - supersedes: f49ece3f572d99160cd755bd6798e855\n";
        let claims = scan_claims(body);
        assert_eq!(claims.len(), 2);
        assert_eq!(claims[0].kind, "fact");
        assert_eq!(
            claims[0].id.as_deref(),
            Some("f49ece3f572d99160cd755bd6798e855")
        );
        assert_eq!(
            claims[0].quote.as_deref(),
            Some("Hey David, I'm your personal agent.")
        );
        assert_eq!(claims[1].sources.len(), 2);
        assert_eq!(
            claims[1].supersedes.as_deref(),
            Some("f49ece3f572d99160cd755bd6798e855")
        );
    }

    #[test]
    fn ignores_ordinary_lists() {
        let body = "- A shopping list item\n- [notatag] no pipe here\n  - claim: orphan\n";
        let claims = scan_claims(body);
        assert_eq!(claims.len(), 0);
    }

    #[test]
    fn a_repeated_subitem_keeps_the_first_value_and_is_recorded() {
        let body = "- [fact|high] A claim with two identity lines.\n  - claim: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n  - claim: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n  - quote: \"one\"\n  - quote: \"two\"\n";
        let claims = scan_claims(body);
        assert_eq!(claims.len(), 1);
        assert_eq!(
            claims[0].id.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(claims[0].quote.as_deref(), Some("one"));
        assert_eq!(claims[0].repeated, ["claim", "quote"]);
    }

    #[test]
    fn the_reverse_link_is_scanned() {
        let body = "- [fact|high] An old claim.\n  - claim: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n  - quote: \"old\"\n  - superseded-by: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n";
        let claims = scan_claims(body);
        assert_eq!(
            claims[0].superseded_by.as_deref(),
            Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
    }

    #[test]
    fn a_blank_line_ends_the_block() {
        let body = "- [fact|high] A claim.\n  - claim: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\n  - quote: \"too late\"\n";
        let claims = scan_claims(body);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].quote, None);
    }
}
