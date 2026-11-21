//! The bundle checker. Each rule implements one clause of
//! `docs/PROFILE.md` and produces findings with stable codes, so a
//! synthesis pass report or a CI log can cite a rule by code.
//!
//! Registry checking in this slice verifies that every claim identity
//! in the daily and curated layers is named in the registry and that a
//! superseded claim is presented as withheld with its replacement on
//! the same line. Byte-level regeneration of the registry belongs to
//! the writer slice, not to this checker.

use std::collections::BTreeMap;
use std::path::Path;

use crate::bundle::{read_bundle, Bundle, ConceptFile, Layer, LAYERS};
use crate::frontmatter::Value;
use crate::OKF_VERSION;

/// How serious a finding is. Errors fail the check. Warnings are
/// reported for the next synthesis pass and do not fail it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Advisory. The check still passes.
    Warning,
    /// A profile violation. The check fails.
    Error,
}

impl Severity {
    /// The label printed in check output.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Severity::Warning => "WARN",
            Severity::Error => "ERROR",
        }
    }
}

/// One rule violation or advisory, tied to a file in the bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The severity of the finding.
    pub severity: Severity,
    /// The stable rule code, for example `FM_TYPE_MISMATCH`.
    pub code: &'static str,
    /// The bundle-relative path the finding belongs to.
    pub path: String,
    /// A one-line statement of what is wrong.
    pub message: String,
}

/// The outcome of checking a bundle.
#[derive(Debug, Clone)]
pub struct CheckReport {
    /// Every finding, sorted by path, then code, then message.
    pub findings: Vec<Finding>,
    /// How many Markdown files were read.
    pub files_checked: usize,
}

impl CheckReport {
    /// How many findings are errors.
    #[must_use]
    pub fn errors(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }

    /// How many findings are warnings.
    #[must_use]
    pub fn warnings(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }

    /// Whether the check passes, meaning no error findings.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.errors() == 0
    }
}

struct Findings {
    items: Vec<Finding>,
}

impl Findings {
    fn new() -> Self {
        Findings { items: Vec::new() }
    }

    fn error(&mut self, code: &'static str, path: &str, message: String) {
        self.items.push(Finding {
            severity: Severity::Error,
            code,
            path: path.to_owned(),
            message,
        });
    }

    fn warning(&mut self, code: &'static str, path: &str, message: String) {
        self.items.push(Finding {
            severity: Severity::Warning,
            code,
            path: path.to_owned(),
            message,
        });
    }
}

/// Check a bundle directory against the looseleaf profile.
#[must_use]
pub fn check_bundle(root: &Path) -> CheckReport {
    let bundle = read_bundle(root);
    let mut out = Findings::new();
    check_layout(&bundle, &mut out);
    check_root_index(&bundle, &mut out);
    for file in &bundle.files {
        if file.is_managed_concept() {
            check_concept(&bundle, file, &mut out);
        }
        if file.layer == Layer::Bank && !file.reserved {
            check_bank_shape(file, &mut out);
        }
        if file.layer == Layer::Daily && !file.reserved {
            check_daily_name(file, &mut out);
        }
    }
    check_claims(&bundle, &mut out);
    let files_checked = bundle.files.len();
    out.items
        .sort_by(|a, b| (&a.path, a.code, &a.message).cmp(&(&b.path, b.code, &b.message)));
    CheckReport {
        findings: out.items,
        files_checked,
    }
}

fn check_layout(bundle: &Bundle, out: &mut Findings) {
    if bundle.file("index.md").is_none() {
        out.error(
            "LAYOUT_ROOT_INDEX",
            "index.md",
            "the bundle root has no index.md".to_owned(),
        );
    }
    if bundle.file("log.md").is_none() {
        out.error(
            "LAYOUT_ROOT_LOG",
            "log.md",
            "the bundle root has no log.md".to_owned(),
        );
    }
    if bundle.file("claims.md").is_none() {
        out.error(
            "LAYOUT_REGISTRY_MISSING",
            "claims.md",
            "the bundle root has no claim registry".to_owned(),
        );
    }
    for dir in &bundle.top_dirs {
        if dir == "target" {
            continue;
        }
        let known = LAYERS.iter().any(|layer| layer.dir_name() == Some(dir.as_str()));
        if !known {
            out.warning(
                "LAYOUT_UNKNOWN_DIR",
                dir,
                format!("directory {dir} is not a profile layer"),
            );
        }
    }
    for layer in LAYERS {
        let Some(dir) = layer.dir_name() else {
            continue;
        };
        if !bundle.has_dir(dir) {
            continue;
        }
        if bundle.file(&format!("{dir}/index.md")).is_none() {
            out.warning(
                "LAYOUT_DIR_INDEX",
                dir,
                format!("layer {dir} carries no index.md"),
            );
        }
        if matches!(layer, Layer::Daily | Layer::Curated)
            && bundle.file(&format!("{dir}/log.md")).is_none()
        {
            out.warning(
                "LAYOUT_DIR_LOG",
                dir,
                format!("layer {dir} carries no log.md"),
            );
        }
    }
    for file in &bundle.files {
        if file.layer == Layer::Root && !file.reserved && !file.is_registry && !file.rel_path.contains('/')
        {
            out.warning(
                "LAYOUT_ROOT_FILE",
                &file.rel_path,
                "a Markdown file at the bundle root belongs in a layer directory".to_owned(),
            );
        }
    }
}

fn check_root_index(bundle: &Bundle, out: &mut Findings) {
    let Some(index) = bundle.file("index.md") else {
        return;
    };
    if let Some(error) = &index.fm_error {
        out.error(
            "FM_PARSE",
            "index.md",
            format!("frontmatter does not parse in the subset, {error}"),
        );
        return;
    }
    let Some(frontmatter) = &index.frontmatter else {
        out.error(
            "INDEX_VERSION",
            "index.md",
            "the bundle index carries no frontmatter, so no okf_version".to_owned(),
        );
        return;
    };
    match frontmatter.get_scalar("okf_version") {
        Some(version) if version == OKF_VERSION => {}
        Some(version) => out.error(
            "INDEX_VERSION",
            "index.md",
            format!("okf_version is {version}, the profile targets {OKF_VERSION}"),
        ),
        None => out.error(
            "INDEX_VERSION",
            "index.md",
            "the bundle index names no okf_version".to_owned(),
        ),
    }
}

fn check_concept(bundle: &Bundle, file: &ConceptFile, out: &mut Findings) {
    let path = &file.rel_path;
    if let Some(error) = &file.fm_error {
        out.error(
            "FM_PARSE",
            path,
            format!("frontmatter does not parse in the subset, {error}"),
        );
        return;
    }
    let Some(frontmatter) = &file.frontmatter else {
        out.error(
            "FM_MISSING",
            path,
            "a managed concept carries no frontmatter".to_owned(),
        );
        return;
    };
    if let Some(expected) = file.expected_type() {
        match frontmatter.get_scalar("type") {
            None => out.error(
                "FM_TYPE_MISSING",
                path,
                format!("type is missing, expected {expected}"),
            ),
            Some(actual) if actual != expected => out.error(
                "FM_TYPE_MISMATCH",
                path,
                format!("type is {actual}, expected {expected} for its directory"),
            ),
            _ => {}
        }
    }
    for (key, code) in [("title", "FM_TITLE"), ("description", "FM_DESCRIPTION")] {
        match frontmatter.get_scalar(key) {
            Some(text) if !text.trim().is_empty() => {}
            _ => out.error(code, path, format!("{key} is missing or empty")),
        }
    }
    let generated_ok = frontmatter
        .get("generated")
        .and_then(Value::as_map)
        .is_some_and(|entries| {
            entries.iter().any(|(k, v)| k == "by" && v.as_scalar().is_some())
                && entries.iter().any(|(k, v)| k == "at" && v.as_scalar().is_some())
        });
    if !generated_ok {
        out.error(
            "FM_GENERATED",
            path,
            "generated is missing or lacks its by and at entries".to_owned(),
        );
    }
    let sources_required = matches!(
        file.layer,
        Layer::Curated | Layer::People | Layer::Groups | Layer::Topics | Layer::Bank
    );
    match frontmatter.get("sources") {
        None => {
            if sources_required {
                out.error(
                    "SRC_REQUIRED",
                    path,
                    "sources is required on this concept type".to_owned(),
                );
            }
        }
        Some(value) => check_sources(bundle, file, value, out),
    }
}

fn check_sources(bundle: &Bundle, file: &ConceptFile, value: &Value, out: &mut Findings) {
    let mut resources: Vec<String> = Vec::new();
    match value {
        Value::List(items) => {
            for item in items {
                match item {
                    Value::Map(_) => {
                        if let Some(resource) = item.get("resource").and_then(Value::as_scalar) {
                            resources.push(resource.to_owned());
                        }
                    }
                    Value::Scalar(text) => resources.push(text.clone()),
                    Value::List(_) => {}
                }
            }
        }
        Value::Scalar(text) => resources.push(text.clone()),
        Value::Map(_) => {
            if let Some(resource) = value.get("resource").and_then(Value::as_scalar) {
                resources.push(resource.to_owned());
            }
        }
    }
    for resource in resources {
        if resource.starts_with("https://") || resource.starts_with("http://") {
            continue;
        }
        let path_part = resource.split('#').next().unwrap_or("");
        if path_part.is_empty() || !bundle.root.join(path_part).exists() {
            out.error(
                "SRC_UNRESOLVED",
                &file.rel_path,
                format!("source resource {resource} does not resolve inside the bundle"),
            );
        }
    }
}

fn check_bank_shape(file: &ConceptFile, out: &mut Findings) {
    let lines = file.body.lines().count();
    if lines > 250 {
        out.warning(
            "BANK_SHAPE",
            &file.rel_path,
            format!("bank concept body runs {lines} lines, past the 250 line shape rule"),
        );
    }
}

fn check_daily_name(file: &ConceptFile, out: &mut Findings) {
    let name = file
        .rel_path
        .rsplit('/')
        .next()
        .unwrap_or(&file.rel_path);
    let stem = name.strip_suffix(".md").unwrap_or(name);
    let bytes = stem.as_bytes();
    let iso = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !iso {
        out.warning(
            "DAILY_NAME",
            &file.rel_path,
            "daily log files are named by ISO date".to_owned(),
        );
    }
}

fn check_claims(bundle: &Bundle, out: &mut Findings) {
    // Identity counts per layer. An identity may appear once in daily
    // and once in curated, which is a promotion, and nowhere else.
    let mut daily: BTreeMap<String, &ConceptFile> = BTreeMap::new();
    let mut curated: BTreeMap<String, &ConceptFile> = BTreeMap::new();
    let mut texts: BTreeMap<String, String> = BTreeMap::new();
    for file in &bundle.files {
        let map = match file.layer {
            Layer::Daily => &mut daily,
            Layer::Curated => &mut curated,
            _ => continue,
        };
        for claim in &file.claims {
            let Some(id) = &claim.id else {
                out.error(
                    "CLAIM_NO_ID",
                    &file.rel_path,
                    format!(
                        "claim at line {} opens with [{}|{}] but carries no claim sub-item",
                        claim.line, claim.kind, claim.confidence
                    ),
                );
                continue;
            };
            if claim.quote.is_none() {
                out.error(
                    "CLAIM_NO_QUOTE",
                    &file.rel_path,
                    format!("claim {id} carries no quote sub-item"),
                );
            }
            if map.contains_key(id) {
                out.error(
                    "CLAIM_DUP_ID",
                    &file.rel_path,
                    format!("claim identity {id} appears twice in one layer"),
                );
            } else {
                map.insert(id.clone(), file);
                if file.layer == Layer::Daily {
                    texts.insert(id.clone(), claim.text.clone());
                }
            }
        }
    }
    let known: std::collections::BTreeSet<&String> =
        daily.keys().chain(curated.keys()).collect();
    for file in &bundle.files {
        if !matches!(file.layer, Layer::Daily | Layer::Curated) {
            continue;
        }
        for claim in &file.claims {
            let Some(target) = &claim.supersedes else {
                continue;
            };
            if claim.id.as_deref() == Some(target.as_str()) {
                out.error(
                    "CLAIM_SUPERSEDES_SELF",
                    &file.rel_path,
                    format!("claim {target} supersedes itself"),
                );
            } else if !known.contains(target) {
                out.error(
                    "CLAIM_SUPERSEDES_DANGLING",
                    &file.rel_path,
                    format!("claim supersedes {target}, which no layer carries"),
                );
            }
        }
    }
    for (id, curated_file) in &curated {
        if let Some(daily_text) = texts.get(id) {
            let curated_claim = curated_file
                .claims
                .iter()
                .find(|claim| claim.id.as_deref() == Some(id.as_str()));
            if let Some(curated_claim) = curated_claim {
                if curated_claim.text != *daily_text {
                    out.warning(
                        "CLAIM_PROMOTION_DRIFT",
                        &curated_file.rel_path,
                        format!(
                            "promoted claim {id} reads differently in curated than in daily, and a wording change is a new claim, not a promotion"
                        ),
                    );
                }
            }
        }
    }
    check_registry(bundle, &known, out);
}

fn check_registry(
    bundle: &Bundle,
    known: &std::collections::BTreeSet<&String>,
    out: &mut Findings,
) {
    let Some(registry) = bundle.file("claims.md") else {
        return;
    };
    for id in known {
        if !registry.body.contains(id.as_str()) {
            out.error(
                "REGISTRY_STALE",
                "claims.md",
                format!("claim {id} is missing from the registry"),
            );
        }
    }
    // A superseded claim must be presented as withheld, with its
    // replacement named on the same line.
    let mut replacements: BTreeMap<&str, &str> = BTreeMap::new();
    for file in &bundle.files {
        for claim in &file.claims {
            if let (Some(id), Some(target)) = (&claim.id, &claim.supersedes) {
                replacements.insert(target.as_str(), id.as_str());
            }
        }
    }
    for (old, new) in &replacements {
        let withheld = registry
            .body
            .lines()
            .any(|line| line.contains(old) && line.contains(new));
        if !withheld {
            out.error(
                "REGISTRY_WITHHELD",
                "claims.md",
                format!(
                    "superseded claim {old} is not presented as withheld with its replacement {new} on one line"
                ),
            );
        }
    }
}
