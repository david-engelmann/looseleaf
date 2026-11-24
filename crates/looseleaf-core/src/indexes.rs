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
//! The root index is regenerated here too, as a line-preserving
//! edit. Its frontmatter is human owned and is preserved byte for
//! byte, its heading line is preserved as written, and the body
//! below the heading is the generated layer list, one bullet per
//! layer present in the bundle, in profile order, with the layer's
//! concept count.

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
        // The link target is the path inside the layer directory,
        // so a concept in a nested subdirectory still points at
        // its own file.
        let target = match layer.dir_name() {
            Some(dir) => file
                .rel_path
                .strip_prefix(dir)
                .and_then(|rest| rest.strip_prefix('/'))
                .unwrap_or(name),
            None => name,
        };
        let text = if layer == Layer::Daily {
            name.strip_suffix(".md").unwrap_or(name).to_owned()
        } else {
            title_of(file)
        };
        lines.push(format!("* [{text}]({target})"));
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
/// replacement cannot be written, and an `InvalidInput` error when
/// a layer directory present in the bundle is a symbolic link.
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
        // The bundle walker never follows a symbolic link, so a
        // layer directory that is one holds no concepts this tool
        // can see, and writing its index would land outside the
        // bundle. Refuse rather than follow it.
        if fs::symlink_metadata(root.join(dir)).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err(crate::write::invalid_input(format!(
                "the layer directory {dir} is a symbolic link, and the writers never write through one"
            )));
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

/// Split a document into its raw frontmatter span, fences included,
/// and the body that follows it. Returns `None` when the document
/// carries no frontmatter block. The span is a slice of the input,
/// so a renderer can carry it into its output byte for byte.
pub(crate) fn frontmatter_span(text: &str) -> Option<(&str, &str)> {
    // A leading byte order mark rides inside the span, so a
    // renderer carrying the span byte for byte keeps the mark.
    let bom = if text.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let inner = &text[bom..];
    let mut offset = 0;
    let mut lines = inner.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return None;
    }
    offset += first.len();
    for line in lines {
        offset += line.len();
        if line.trim_end_matches(['\r', '\n']).trim_end() == "---" {
            return Some((&text[..bom + offset], &text[bom + offset..]));
        }
    }
    None
}

/// Render the root `index.md` for a bundle from its current text.
/// The frontmatter span and everything in the body up to and
/// including the heading line are carried over as a raw slice, so
/// they survive byte for byte. The body below the heading is the
/// layer list, one bullet per layer directory present in the
/// bundle, in profile order, each with the layer's concept count.
/// The heading is the first `# ` line outside a fenced code block.
/// Returns `None` when the current text carries no frontmatter or
/// no heading, which are the checker's findings to report, not the
/// writer's to invent.
#[must_use]
pub fn render_root_index(bundle: &Bundle, current: &str) -> Option<String> {
    let (span, body) = frontmatter_span(current)?;
    let mut in_fence = false;
    let mut consumed = 0;
    let mut heading_end: Option<usize> = None;
    for segment in body.split_inclusive('\n') {
        let line = segment.trim_end_matches(['\r', '\n']);
        if line.starts_with("```") {
            in_fence = !in_fence;
        } else if !in_fence && line.starts_with("# ") {
            heading_end = Some(consumed + segment.len());
            break;
        }
        consumed += segment.len();
    }
    let heading_end = heading_end?;
    let eol = crate::write::line_ending(current);
    let mut out = String::from(span);
    out.push_str(&body[..heading_end]);
    if !out.ends_with('\n') {
        out.push_str(eol);
    }
    let mut bullets: Vec<String> = Vec::new();
    for layer in LAYERS {
        let Some(dir) = layer.dir_name() else {
            continue;
        };
        if !bundle.has_dir(dir) {
            continue;
        }
        let count = entries_for(bundle, layer).len();
        let noun = if count == 1 { "concept" } else { "concepts" };
        bullets.push(format!("* [{}]({dir}/) - {count} {noun}", heading(layer)));
    }
    if !bullets.is_empty() {
        out.push_str(eol);
        out.push_str(eol);
        out.push_str(&bullets.join(eol));
    }
    out.push_str(eol);
    Some(out)
}

/// Regenerate the root `index.md` of the bundle at the given root,
/// returning its bundle-relative path when it changed and `None`
/// when it was already current. The write goes to a sibling
/// temporary file and is renamed over the target.
///
/// # Errors
///
/// Returns an I/O error when the root index cannot be read or
/// written, and an `InvalidInput` error when it carries no
/// frontmatter or no heading for the renderer to preserve.
pub fn regenerate_root_index(root: &Path) -> std::io::Result<Option<String>> {
    let target = root.join("index.md");
    let current = fs::read_to_string(&target)?;
    let bundle = read_bundle(root);
    let Some(rendered) = render_root_index(&bundle, &current) else {
        return Err(crate::write::invalid_input(
            "the root index carries no frontmatter or no heading to preserve".to_owned(),
        ));
    };
    if rendered == current {
        return Ok(None);
    }
    let temp = root.join("index.md.tmp-looseleaf");
    fs::write(&temp, rendered)?;
    fs::rename(&temp, &target)?;
    Ok(Some("index.md".to_owned()))
}
