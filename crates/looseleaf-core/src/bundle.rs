//! Reading a bundle directory into the model the checker works on.
//!
//! The walker is deliberately forgiving. It collects what it finds and
//! lets the checker report what is wrong, because a bundle that cannot
//! be read at all is exactly the bundle its owner needs findings about.

use std::fs;
use std::path::{Path, PathBuf};

use crate::claim::{scan_claims, Claim};
use crate::frontmatter::{parse_document, Frontmatter};

/// The layer a file belongs to, derived from its top-level directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// The bundle root itself.
    Root,
    /// `curated/`, type Curated Memory.
    Curated,
    /// `people/`, type Person.
    People,
    /// `groups/`, type Group.
    Groups,
    /// `topics/`, type Topic.
    Topics,
    /// `daily/`, type Daily Log.
    Daily,
    /// `bank/`, type Memory Bank.
    Bank,
    /// `synthesis/`, type Synthesis.
    Synthesis,
    /// `directives/`, type Directive.
    Directives,
}

impl Layer {
    /// The concept type the profile expects for files in this layer,
    /// when the layer holds concepts at all.
    #[must_use]
    pub fn expected_type(self) -> Option<&'static str> {
        match self {
            Layer::Root => None,
            Layer::Curated => Some("Curated Memory"),
            Layer::People => Some("Person"),
            Layer::Groups => Some("Group"),
            Layer::Topics => Some("Topic"),
            Layer::Daily => Some("Daily Log"),
            Layer::Bank => Some("Memory Bank"),
            Layer::Synthesis => Some("Synthesis"),
            Layer::Directives => Some("Directive"),
        }
    }

    /// The top-level directory name for a layer, when it has one.
    #[must_use]
    pub fn dir_name(self) -> Option<&'static str> {
        match self {
            Layer::Root => None,
            Layer::Curated => Some("curated"),
            Layer::People => Some("people"),
            Layer::Groups => Some("groups"),
            Layer::Topics => Some("topics"),
            Layer::Daily => Some("daily"),
            Layer::Bank => Some("bank"),
            Layer::Synthesis => Some("synthesis"),
            Layer::Directives => Some("directives"),
        }
    }

    fn from_dir(name: &str) -> Option<Layer> {
        match name {
            "curated" => Some(Layer::Curated),
            "people" => Some(Layer::People),
            "groups" => Some(Layer::Groups),
            "topics" => Some(Layer::Topics),
            "daily" => Some(Layer::Daily),
            "bank" => Some(Layer::Bank),
            "synthesis" => Some(Layer::Synthesis),
            "directives" => Some(Layer::Directives),
            _ => None,
        }
    }
}

/// Every layer that holds concepts, in profile order.
pub const LAYERS: [Layer; 8] = [
    Layer::Curated,
    Layer::People,
    Layer::Groups,
    Layer::Topics,
    Layer::Daily,
    Layer::Bank,
    Layer::Synthesis,
    Layer::Directives,
];

/// One Markdown file in the bundle, read and parsed.
#[derive(Debug, Clone)]
pub struct ConceptFile {
    /// The path relative to the bundle root, with `/` separators.
    pub rel_path: String,
    /// The layer derived from the top-level directory.
    pub layer: Layer,
    /// True for `index.md` and `log.md`, which the standard reserves
    /// and which are never concepts.
    pub reserved: bool,
    /// True for the root `claims.md`, the claim registry.
    pub is_registry: bool,
    /// The parsed frontmatter, when present and inside the subset.
    pub frontmatter: Option<Frontmatter>,
    /// The frontmatter failure, when a fence is present but the block
    /// is malformed or outside the subset.
    pub fm_error: Option<String>,
    /// The document body.
    pub body: String,
    /// Claim blocks scanned from the body. Only the daily and curated
    /// layers carry claim blocks under the convention.
    pub claims: Vec<Claim>,
}

impl ConceptFile {
    /// A managed concept is a file the frontmatter profile binds, a
    /// non-reserved file in a layer directory, or the registry.
    #[must_use]
    pub fn is_managed_concept(&self) -> bool {
        !self.reserved && (self.layer != Layer::Root || self.is_registry)
    }

    /// The concept type this file is expected to declare.
    #[must_use]
    pub fn expected_type(&self) -> Option<&'static str> {
        if self.is_registry {
            return Some("Claim Registry");
        }
        self.layer.expected_type()
    }
}

/// A bundle read from disk.
#[derive(Debug, Clone)]
pub struct Bundle {
    /// The bundle root on disk.
    pub root: PathBuf,
    /// Every Markdown file found, sorted by relative path.
    pub files: Vec<ConceptFile>,
    /// The top-level directory names found, sorted.
    pub top_dirs: Vec<String>,
}

impl Bundle {
    /// The file at a bundle-relative path, when present.
    #[must_use]
    pub fn file(&self, rel_path: &str) -> Option<&ConceptFile> {
        self.files.iter().find(|file| file.rel_path == rel_path)
    }

    /// Whether a layer directory is present at the bundle root.
    #[must_use]
    pub fn has_dir(&self, name: &str) -> bool {
        self.top_dirs.iter().any(|dir| dir == name)
    }
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|entry| entry.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            if name != "target" {
                collect_markdown(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}

/// Read a bundle from a directory on disk.
#[must_use]
pub fn read_bundle(root: &Path) -> Bundle {
    let mut paths = Vec::new();
    collect_markdown(root, &mut paths);
    let mut top_dirs: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if !name.starts_with('.') {
                    top_dirs.push(name);
                }
            }
        }
    }
    top_dirs.sort();
    let mut files = Vec::new();
    for path in paths {
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let rel_path = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        let file_name = relative
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let first = relative
            .components()
            .next()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        let layer = if rel_path.contains('/') {
            Layer::from_dir(&first).unwrap_or(Layer::Root)
        } else {
            Layer::Root
        };
        let reserved = file_name == "index.md" || file_name == "log.md";
        let is_registry = rel_path == "claims.md";
        let text = fs::read_to_string(&path).unwrap_or_default();
        let document = parse_document(&text);
        let claims = if matches!(layer, Layer::Daily | Layer::Curated) && !reserved {
            scan_claims(&document.body)
        } else {
            Vec::new()
        };
        files.push(ConceptFile {
            rel_path,
            layer,
            reserved,
            is_registry,
            frontmatter: document.frontmatter,
            fm_error: document.parse_error,
            body: document.body,
            claims,
        });
    }
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Bundle {
        root: root.to_path_buf(),
        files,
        top_dirs,
    }
}
