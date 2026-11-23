//! Scanning for claim blocks in Daily Log and Curated Memory bodies.
//!
//! The convention is defined in `docs/PROFILE.md` section 3. A claim
//! block is a list item whose text opens with a `[kind|confidence]`
//! tag, followed by indented sub-items carrying the claim identity, a
//! verbatim quote, sources and an optional supersedes link.

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

/// Scan a concept body for claim blocks, in document order.
#[must_use]
pub fn scan_claims(body: &str) -> Vec<Claim> {
    let mut claims: Vec<Claim> = Vec::new();
    let mut attached = false;
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
                line: index + 1,
            });
            attached = true;
            continue;
        }
        if line.starts_with('#') || line.starts_with("- ") {
            // A heading or a fresh top-level item ends the block above
            // it, so a later indented list cannot attach by accident.
            attached = false;
        }
        if !attached {
            continue;
        }
        let Some(current) = claims.last_mut() else {
            continue;
        };
        let Some(sub) = line.strip_prefix("  - ") else {
            continue;
        };
        if let Some(value) = sub.strip_prefix("claim:") {
            current.id = Some(parse_scalar(value));
        } else if let Some(value) = sub.strip_prefix("quote:") {
            current.quote = Some(parse_scalar(value));
        } else if let Some(value) = sub.strip_prefix("sources:") {
            current.sources = value
                .split(',')
                .map(parse_scalar)
                .filter(|part| !part.is_empty())
                .collect();
        } else if let Some(value) = sub.strip_prefix("supersedes:") {
            current.supersedes = Some(parse_scalar(value));
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
}
