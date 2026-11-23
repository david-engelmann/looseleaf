//! Regenerating the layer indexes, the `index.md` each layer
//! directory carries.
//!
//! A layer index is a generated file. Its format is a heading line
//! and one bullet per concept in the layer, a Markdown link whose
//! target is the file name and whose text is the concept title, with
//! the daily layer linking by ISO date stem instead. Rendering is
//! deterministic, so the same bundle state produces byte-identical
//! output and a stale index is visible in a diff.
//!
//! Ordering follows the profile. People and groups order by the
//! `x-looseleaf-rank` extension key ascending, unranked concepts
//! after ranked ones, with title as the tiebreak. Daily orders by
//! date, which the file names sort by naturally. Every other layer
//! orders by title. Reserved files and the registry never appear as
//! entries, and a layer with no concepts renders a heading and no
//! bullets.
//!
//! The root index is not regenerated here. Its frontmatter is human
//! owned, and regenerating its body is a line-preserving edit that
//! belongs to a later slice.

use std::fs;
use std::path::Path;

use crate::bundle::{Bundle, ConceptFile, LAYERS, Layer, read_bundle};

/// The heading label a layer's index carries.
fn heading(layer: Layer) -> &'static str {
    match layer {
        Layer::Curated => "Curated memory",
        Layer::People => "People",
        Layer::Groups => "Groups",
        Layer::Topics => "Topics",
        Layer::Daily => "Daily logs",
        Layer::Bank => "Memory bank",
        Layer::Synthesis => "Synthesis",
        Layer::Directives => "Directives",
        Layer::Root => "Bundle",
    }
}

/// The title a concept lists under, from its frontmatter, falling
/// back to the file stem when the concept carries no usable title.
/// The checker flags the missing title on its own rule, so the
/// renderer only needs a deterministic text.
fn title_of(file: &ConceptFile) -> String {
    if let Some(frontmatter) = &file.frontmatter
        && let Some(title) = frontmatter.get_scalar("title")
        && !title.trim().is_empty()
    {
        return title.to_owned();
    }
    file.rel_path
        .rsplit('/')
        .next()
        .unwrap_or(&file.rel_path)
        .strip_suffix(".md")
        .unwrap_or(&file.rel_path)
        .to_owned()
}

/// The closeness rank a person or group concept carries, when it
/// carries a numeric one.
fn rank_of(file: &ConceptFile) -> Option<i64> {
    file.frontmatter
        .as_ref()
        .and_then(|frontmatter| frontmatter.get_scalar("x-looseleaf-rank"))
        .and_then(|text| text.trim().parse::<i64>().ok())
}

/// The concepts a layer index lists, in index order.
fn entries_for(bundle: &Bundle, layer: Layer) -> Vec<&ConceptFile> {
    let mut files: Vec<&ConceptFile> = bundle
        .files
        .iter()
        .filter(|file| file.layer == layer && !file.reserved)
        .collect();
    match layer {
        Layer::Daily => {
            files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        }
        Layer::People | Layer::Groups => {
            files.sort_by(|a, b| {
                let rank_a = rank_of(a);
                let rank_b = rank_of(b);
                (rank_a.is_none(), rank_a, title_of(a), &a.rel_path).cmp(&(
                    rank_b.is_none(),
                    rank_b,
                    title_of(b),
                    &b.rel_path,
                ))
            });
        }
        _ => {
            files.sort_by(|a, b| (title_of(a), &a.rel_path).cmp(&(title_of(b), &b.rel_path)));
        }
    }
    files
}

/// Render the full text of one layer's `index.md` for a bundle.
#[must_use]
pub fn render_layer_index(bundle: &Bundle, layer: Layer) -> String {
    let entries = entries_for(bundle, layer);
    let mut lines: Vec<String> = vec![format!("# {}", heading(layer))];
    if !entries.is_empty() {
        lines.push(String::new());
    }
    for file in entries {
        let name = file.rel_path.rsplit('/').next().unwrap_or(&file.rel_path);
        let text = if layer == Layer::Daily {
            name.strip_suffix(".md").unwrap_or(name).to_owned()
        } else {
            title_of(file)
        };
        lines.push(format!("* [{text}]({name})"));
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Regenerate the index of every layer directory present in the
/// bundle at the given root, returning the bundle-relative paths of
/// the indexes that changed. A layer whose directory is absent is
/// skipped, because creating layer directories is a layout act, not
/// a writer act. Each write goes to a sibling temporary file and is
/// renamed over the target, so a failed write never truncates the
/// index that was there.
///
/// # Errors
///
/// Returns an I/O error when an existing index cannot be read or a
/// replacement cannot be written.
pub fn regenerate_indexes(root: &Path) -> std::io::Result<Vec<String>> {
    let bundle = read_bundle(root);
    let mut written: Vec<String> = Vec::new();
    for layer in LAYERS {
        let Some(dir) = layer.dir_name() else {
            continue;
        };
        if !bundle.has_dir(dir) {
            continue;
        }
        let rendered = render_layer_index(&bundle, layer);
        let rel = format!("{dir}/index.md");
        let target = root.join(&rel);
        let current = match fs::read_to_string(&target) {
            Ok(text) => Some(text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        if current.as_deref() == Some(rendered.as_str()) {
            continue;
        }
        let temp = root.join(format!("{dir}/index.md.tmp-looseleaf"));
        fs::write(&temp, rendered)?;
        fs::rename(&temp, &target)?;
        written.push(rel);
    }
    Ok(written)
}
