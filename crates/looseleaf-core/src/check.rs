//! The bundle checker. Each rule implements one clause of
//! `docs/PROFILE.md` and produces findings with stable codes, so a
//! synthesis pass report or a CI log can cite a rule by code.
//!
//! Registry checking compares identity sets in both directions. A
//! claim the layers carry that the registry lacks is stale, and a
//! registry entry no layer carries is equally stale, because the
//! profile defines staleness as the sets differing. A superseded
//! claim must appear in the registry as a withheld entry in the
//! rendered format, with its replacement named and its reason
//! carried whenever the curated record carries one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::OKF_VERSION;
use crate::bundle::{Bundle, ConceptFile, LAYERS, Layer, read_bundle};
use crate::frontmatter::Value;
use crate::registry::section_entries_in;

/// How serious a finding is. Errors fail the check. Warnings are
/// reported for the next synthesis pass and do not fail it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    for link in &bundle.skipped_links {
        out.warning(
            "LINK_SKIPPED",
            link,
            "a symbolic link was not followed, so whatever it names is unchecked".to_owned(),
        );
    }
    check_layout(&bundle, &mut out);
    check_root_index(&bundle, &mut out);
    for file in &bundle.files {
        if file.read_error {
            out.error(
                "READ_ERROR",
                &file.rel_path,
                "the file could not be read as UTF-8 text".to_owned(),
            );
            continue;
        }
        if file.is_managed_concept() {
            check_concept(&bundle, file, &mut out);
        }
        if file.layer == Layer::Bank && !file.reserved {
            check_bank_shape(file, &mut out);
        }
        if file.layer == Layer::Daily && !file.reserved {
            check_daily_name(file, &mut out);
        }
        if file.layer == Layer::People && !file.reserved {
            check_person_sections(file, &mut out);
        }
        if file.layer == Layer::Synthesis && !file.reserved {
            check_report_shape(file, &mut out);
        }
        if file.reserved && file.rel_path.ends_with("log.md") {
            check_log_order(file, &mut out);
        }
    }
    check_claims(&bundle, &mut out);
    check_concept_cycles(&bundle, &mut out);
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
        let known = LAYERS
            .iter()
            .any(|layer| layer.dir_name() == Some(dir.as_str()));
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
            out.error(
                "LAYOUT_DIR_INDEX",
                dir,
                format!("layer {dir} carries no index.md"),
            );
        }
        if matches!(layer, Layer::Daily | Layer::Curated)
            && bundle.file(&format!("{dir}/log.md")).is_none()
        {
            out.error(
                "LAYOUT_DIR_LOG",
                dir,
                format!("layer {dir} carries no log.md"),
            );
        }
    }
    for file in &bundle.files {
        if file.layer == Layer::Root
            && !file.reserved
            && !file.is_registry
            && !file.rel_path.contains('/')
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
    for key in ["title", "description", "license"] {
        match frontmatter.get_scalar(key) {
            Some(text) if !text.trim().is_empty() => {}
            _ => out.error(
                "INDEX_FIELD",
                "index.md",
                format!("the bundle index names no {key}, which the profile fixes as part of its frontmatter"),
            ),
        }
    }
}

/// The top-level frontmatter keys the profile itself defines. Any
/// other key is an extension key and carries the `x-looseleaf-`
/// prefix by the profile's rule, so a host writer's own keys stay
/// visible as foreign instead of passing as profile keys.
const PROFILE_KEYS: [&str; 11] = [
    "type",
    "title",
    "description",
    "generated",
    "sources",
    "verified",
    "stale_after",
    "status",
    "superseded_by",
    "okf_version",
    "license",
];

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
            entries
                .iter()
                .any(|(k, v)| k == "by" && v.as_scalar().is_some())
                && entries
                    .iter()
                    .any(|(k, v)| k == "at" && v.as_scalar().is_some())
        });
    if !generated_ok {
        out.error(
            "FM_GENERATED",
            path,
            "generated is missing or lacks its by and at entries".to_owned(),
        );
    }
    for (key, _) in frontmatter.entries() {
        if !PROFILE_KEYS.contains(&key.as_str()) && !key.starts_with("x-looseleaf-") {
            out.warning(
                "FM_EXTENSION_PREFIX",
                path,
                format!(
                    "key {key} is not a profile key and carries no x-looseleaf- prefix, so it reads as a profile key while being an extension"
                ),
            );
        }
    }
    check_verified_shape(file, frontmatter, out);
    check_stale_after_shape(file, frontmatter, out);
    check_concept_supersession(bundle, file, frontmatter, out);
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

/// Whether one verification record carries scalar `by` and `at`
/// entries. A `verified` value is one such record or a list of them,
/// because a concept can be verified more than once.
fn verified_record_ok(value: &Value) -> bool {
    value.as_map().is_some_and(|entries| {
        entries
            .iter()
            .any(|(k, v)| k == "by" && v.as_scalar().is_some())
            && entries
                .iter()
                .any(|(k, v)| k == "at" && v.as_scalar().is_some())
    })
}

fn check_verified_shape(
    file: &ConceptFile,
    frontmatter: &crate::frontmatter::Frontmatter,
    out: &mut Findings,
) {
    let Some(verified) = frontmatter.get("verified") else {
        return;
    };
    let shape_ok = match verified {
        Value::Map(_) => verified_record_ok(verified),
        Value::List(items) => !items.is_empty() && items.iter().all(verified_record_ok),
        Value::Scalar(_) => false,
    };
    if !shape_ok {
        out.error(
            "FM_VERIFIED",
            &file.rel_path,
            "verified is present but does not carry by and at entries naming the actor and the time".to_owned(),
        );
    }
}

fn check_stale_after_shape(
    file: &ConceptFile,
    frontmatter: &crate::frontmatter::Frontmatter,
    out: &mut Findings,
) {
    let Some(raw) = frontmatter.get_scalar("stale_after") else {
        return;
    };
    let date_ok = raw.trim().get(..10).is_some_and(is_calendar_date);
    if !date_ok {
        out.warning(
            "FM_STALE_AFTER",
            &file.rel_path,
            format!(
                "stale_after carries {raw}, whose first ten characters are not a real calendar date, so staleness reporting skips this concept"
            ),
        );
    }
}

fn check_concept_supersession(
    bundle: &Bundle,
    file: &ConceptFile,
    frontmatter: &crate::frontmatter::Frontmatter,
    out: &mut Findings,
) {
    let path = &file.rel_path;
    let status = frontmatter.get_scalar("status");
    let superseded_by = frontmatter.get_scalar("superseded_by");
    if let Some(target) = superseded_by {
        match resolve_in_bundle(bundle, target) {
            Ok(_) => {}
            Err(reason) => out.error(
                "FM_SUPERSEDED_BY",
                path,
                format!("superseded_by names {target}, which {reason}"),
            ),
        }
        if status != Some("deprecated") {
            out.warning(
                "FM_STATUS",
                path,
                "the concept names a superseded_by replacement but its status is not deprecated"
                    .to_owned(),
            );
        }
    } else if status == Some("deprecated") {
        out.warning(
            "FM_STATUS",
            path,
            "the concept is deprecated but names no superseded_by replacement".to_owned(),
        );
    }
}

/// Detect cycles in the concept supersession graph. The walk
/// follows each managed concept's `superseded_by`, read as a
/// scalar and resolved to its normalized form, and stops at a
/// concept with no pointer, at a pointer whose target is not a
/// managed concept, and when it returns to a concept already on
/// the current walk. That return is the cycle, and one finding
/// fires per concept on it. A chain that only leads into a
/// cycle from outside fires nothing for its tail.
fn check_concept_cycles(bundle: &Bundle, out: &mut Findings) {
    let managed: BTreeSet<&str> = bundle
        .files
        .iter()
        .filter(|file| file.is_managed_concept())
        .map(|file| file.rel_path.as_str())
        .collect();
    let mut edges: BTreeMap<&str, String> = BTreeMap::new();
    for file in &bundle.files {
        if !file.is_managed_concept() {
            continue;
        }
        let Some(frontmatter) = &file.frontmatter else {
            continue;
        };
        let Some(target) = frontmatter.get_scalar("superseded_by") else {
            continue;
        };
        let Ok(normalized) = resolve_in_bundle(bundle, target) else {
            continue;
        };
        if managed.contains(normalized.as_str()) {
            edges.insert(file.rel_path.as_str(), normalized);
        }
    }
    let mut reported: BTreeSet<Vec<&str>> = BTreeSet::new();
    for start in edges.keys() {
        let mut walk: Vec<&str> = Vec::new();
        let mut current = *start;
        loop {
            if let Some(index) = walk.iter().position(|node| *node == current) {
                let mut cycle: Vec<&str> = walk[index..].to_vec();
                cycle.sort_unstable();
                if reported.insert(cycle.clone()) {
                    for member in &cycle {
                        let target = &edges[*member];
                        out.error(
                            "FM_SUPERSEDE_CYCLE",
                            member,
                            format!(
                                "superseded_by names {target}, and the chain forms a cycle through {}, so no concept in the cycle has a live successor",
                                cycle.join(", ")
                            ),
                        );
                    }
                }
                break;
            }
            walk.push(current);
            let Some(next) = edges.get(current) else {
                break;
            };
            current = next;
        }
    }
}

/// Resolve a resource path against the bundle root, lexically, and
/// say where it lands or why it cannot be inside the bundle. The
/// lexical walk is the containment rule. A path that is absolute or
/// climbs out with `..` is outside, whatever the filesystem holds,
/// and only a path that stays inside is tested for existence.
fn resolve_in_bundle(bundle: &Bundle, resource: &str) -> Result<String, String> {
    let path_part = resource.split('#').next().unwrap_or("");
    if path_part.is_empty() {
        return Err("names no path".to_owned());
    }
    if path_part.starts_with('/') {
        return Err("is an absolute path outside the bundle".to_owned());
    }
    let mut parts: Vec<&str> = Vec::new();
    for component in path_part.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err("climbs out of the bundle".to_owned());
                }
            }
            name => parts.push(name),
        }
    }
    if parts.is_empty() {
        return Err("names the bundle root, not a file in it".to_owned());
    }
    let normalized = parts.join("/");
    if bundle.root.join(&normalized).exists() {
        Ok(normalized)
    } else {
        Err(format!("does not resolve to {normalized} in the bundle"))
    }
}

fn check_sources(bundle: &Bundle, file: &ConceptFile, value: &Value, out: &mut Findings) {
    let mut resources: Vec<Option<String>> = Vec::new();
    match value {
        Value::List(items) => {
            for item in items {
                match item {
                    Value::Map(_) => {
                        resources.push(
                            item.get("resource")
                                .and_then(Value::as_scalar)
                                .map(str::to_owned),
                        );
                    }
                    Value::Scalar(text) => resources.push(Some(text.clone())),
                    Value::List(_) => resources.push(None),
                }
            }
        }
        Value::Scalar(text) => resources.push(Some(text.clone())),
        Value::Map(_) => {
            resources.push(
                value
                    .get("resource")
                    .and_then(Value::as_scalar)
                    .map(str::to_owned),
            );
        }
    }
    for resource in resources {
        let Some(resource) = resource else {
            out.error(
                "SRC_UNRESOLVED",
                &file.rel_path,
                "a sources entry names no resource, so the concept carries no provenance for it"
                    .to_owned(),
            );
            continue;
        };
        if resource.starts_with("https://") || resource.starts_with("http://") {
            continue;
        }
        if let Err(reason) = resolve_in_bundle(bundle, &resource) {
            out.error(
                "SRC_UNRESOLVED",
                &file.rel_path,
                format!("source resource {resource} {reason}"),
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

/// The six section headings a Person body keeps, from the design
/// record, the shape the source system uses.
const PERSON_SECTIONS: [&str; 6] = [
    "Facts",
    "History",
    "The relationship",
    "In common",
    "Open threads",
    "Strengthening",
];

fn check_person_sections(file: &ConceptFile, out: &mut Findings) {
    let mut missing: Vec<&str> = Vec::new();
    for section in PERSON_SECTIONS {
        let heading = format!("## {section}");
        if !file.body.lines().any(|line| line.trim_end() == heading) {
            missing.push(section);
        }
    }
    if !missing.is_empty() {
        out.warning(
            "PERSON_SECTIONS",
            &file.rel_path,
            format!(
                "person body lacks the sections {}",
                missing
                    .iter()
                    .map(|name| format!("## {name}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
}

/// A synthesis file named as a pass report carries the report
/// shape the writer creates, six sections in contract order. A
/// synthesis concept under any other name is not the writer's
/// business and is left alone. The walker collects Markdown
/// case-insensitively, so the name test here is too, and a
/// heading quoted inside a code fence is content, not a section.
fn check_report_shape(file: &ConceptFile, out: &mut Findings) {
    let name = file.rel_path.rsplit('/').next().unwrap_or_default();
    let stem = match name.get(name.len().saturating_sub(3)..) {
        Some(suffix) if suffix.eq_ignore_ascii_case(".md") => {
            name.get(..name.len() - 3).unwrap_or(name)
        }
        _ => name,
    };
    if !crate::report::is_report_stem(stem) {
        return;
    }
    let mut positions: Vec<Option<usize>> = Vec::new();
    let mut in_fence = false;
    let mut found: Vec<(usize, &str)> = Vec::new();
    for (index, line) in file.body.lines().enumerate() {
        let line = line.trim_end();
        if line.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        for section in crate::report::REPORT_HEADINGS {
            if line == format!("## {section}") {
                found.push((index, section));
            }
        }
    }
    for section in crate::report::REPORT_HEADINGS {
        positions.push(
            found
                .iter()
                .find(|(_, name)| *name == section)
                .map(|(index, _)| *index),
        );
    }
    let missing: Vec<&str> = crate::report::REPORT_HEADINGS
        .iter()
        .zip(&positions)
        .filter(|(_, position)| position.is_none())
        .map(|(section, _)| *section)
        .collect();
    if !missing.is_empty() {
        out.warning(
            "REPORT_SHAPE",
            &file.rel_path,
            format!(
                "pass report lacks the sections {}",
                missing
                    .iter()
                    .map(|name| format!("## {name}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        return;
    }
    let ordered = positions
        .iter()
        .map(|position| position.expect("presence was just checked"))
        .collect::<Vec<_>>();
    if ordered.windows(2).any(|pair| pair[0] > pair[1]) {
        out.warning(
            "REPORT_SHAPE",
            &file.rel_path,
            "pass report carries its six sections out of contract order".to_owned(),
        );
    }
}

fn check_log_order(file: &ConceptFile, out: &mut Findings) {
    let mut previous: Option<String> = None;
    for line in file.body.lines() {
        let Some(rest) = line.strip_prefix("## ") else {
            continue;
        };
        let date = rest.trim();
        if !is_calendar_date(date) {
            continue;
        }
        if let Some(prev) = &previous
            && date > prev.as_str()
        {
            out.warning(
                "LOG_ORDER",
                &file.rel_path,
                format!(
                    "log section {date} follows {prev}, and the bundle log runs newest date first"
                ),
            );
            return;
        }
        previous = Some(date.to_owned());
    }
}

/// Whether a `YYYY-MM-DD` shaped string is a real calendar date.
pub(crate) fn is_calendar_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = text[0..4].parse().unwrap_or(0);
    let month: u32 = text[5..7].parse().unwrap_or(0);
    let day: u32 = text[8..10].parse().unwrap_or(0);
    if !(1..=12).contains(&month) || day == 0 {
        return false;
    }
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if leap {
                29
            } else {
                28
            }
        }
    };
    day <= days
}

fn check_daily_name(file: &ConceptFile, out: &mut Findings) {
    let name = file.rel_path.rsplit('/').next().unwrap_or(&file.rel_path);
    let stem = name.strip_suffix(".md").unwrap_or(name);
    if !is_calendar_date(stem) {
        out.error(
            "DAILY_NAME",
            &file.rel_path,
            "daily log files are named by ISO date, and the name is not a calendar date".to_owned(),
        );
    }
}

/// The claim kinds the profile names as in use.
const CLAIM_KINDS: [&str; 7] = [
    "fact",
    "event",
    "preference",
    "decision",
    "state change",
    "update",
    "commitment",
];

/// The confidences the profile names.
const CLAIM_CONFIDENCES: [&str; 3] = ["high", "medium", "low"];

fn check_claims(bundle: &Bundle, out: &mut Findings) {
    // Identity maps per layer. An identity may appear once in daily
    // and once in curated, which is a promotion, and nowhere else.
    let mut daily: BTreeMap<String, &ConceptFile> = BTreeMap::new();
    let mut curated: BTreeMap<String, &ConceptFile> = BTreeMap::new();
    let mut daily_claims: BTreeMap<String, &crate::claim::Claim> = BTreeMap::new();
    for file in &bundle.files {
        if !matches!(file.layer, Layer::Daily | Layer::Curated) {
            continue;
        }
        for claim in &file.claims {
            check_claim_shape(bundle, file, claim, out);
            let Some(id) = &claim.id else {
                continue;
            };
            let map = if file.layer == Layer::Daily {
                &mut daily
            } else {
                &mut curated
            };
            if map.contains_key(id) {
                out.error(
                    "CLAIM_DUP_ID",
                    &file.rel_path,
                    format!("claim identity {id} appears twice in one layer"),
                );
            } else {
                map.insert(id.clone(), file);
                if file.layer == Layer::Daily {
                    daily_claims.insert(id.clone(), claim);
                }
            }
        }
    }
    let known: BTreeSet<&String> = daily.keys().chain(curated.keys()).collect();
    check_supersession_graph(bundle, &known, out);
    check_promotions(&curated, &daily_claims, out);
    check_reverse_links(bundle, &known, out);
    check_sections(bundle, out);
    check_registry(bundle, &known, out);
}

fn check_claim_shape(
    bundle: &Bundle,
    file: &ConceptFile,
    claim: &crate::claim::Claim,
    out: &mut Findings,
) {
    let path = &file.rel_path;
    if !CLAIM_KINDS.contains(&claim.kind.as_str()) {
        out.error(
            "CLAIM_KIND",
            path,
            format!(
                "claim at line {} is tagged with kind {}, which the profile does not name",
                claim.line, claim.kind
            ),
        );
    }
    if !CLAIM_CONFIDENCES.contains(&claim.confidence.as_str()) {
        out.error(
            "CLAIM_CONFIDENCE",
            path,
            format!(
                "claim at line {} is tagged with confidence {}, which the profile does not name",
                claim.line, claim.confidence
            ),
        );
    }
    if !claim.repeated.is_empty() {
        out.error(
            "CLAIM_DUP_SUBITEM",
            path,
            format!(
                "claim at line {} repeats the sub-items {}, and the first value was kept",
                claim.line,
                claim.repeated.join(", ")
            ),
        );
    }
    let Some(id) = &claim.id else {
        out.error(
            "CLAIM_NO_ID",
            path,
            format!(
                "claim at line {} opens with [{}|{}] but carries no claim sub-item",
                claim.line, claim.kind, claim.confidence
            ),
        );
        return;
    };
    if claim.quote.is_none() {
        out.error(
            "CLAIM_NO_QUOTE",
            path,
            format!("claim {id} carries no quote sub-item"),
        );
    }
    if file.layer == Layer::Daily && claim.superseded_by.is_some() {
        out.error(
            "CLAIM_REVERSE_IN_DAILY",
            path,
            format!(
                "claim {id} carries a superseded-by reverse link in a daily file, and reverse links live only in the curated layer"
            ),
        );
    }
    check_claim_sources(bundle, file, claim, id, out);
}

fn check_claim_sources(
    bundle: &Bundle,
    file: &ConceptFile,
    claim: &crate::claim::Claim,
    id: &str,
    out: &mut Findings,
) {
    for source in &claim.sources {
        if source.starts_with("message:")
            || source.starts_with("https://")
            || source.starts_with("http://")
        {
            continue;
        }
        if !source.contains(".md") {
            continue;
        }
        let (path_part, anchor) = match source.split_once('#') {
            Some((head, tail)) => (head, Some(tail)),
            None => (source.as_str(), None),
        };
        let normalized = match resolve_in_bundle(bundle, path_part) {
            Ok(normalized) => normalized,
            Err(reason) => {
                out.error(
                    "CLAIM_SRC_UNRESOLVED",
                    &file.rel_path,
                    format!("claim {id} names source {source}, which {reason}"),
                );
                continue;
            }
        };
        let Some(anchor) = anchor else {
            continue;
        };
        if let Some(anchor_id) = anchor.strip_prefix("claim:") {
            let carries = bundle.file(&normalized).is_some_and(|target| {
                target
                    .claims
                    .iter()
                    .any(|other| other.id.as_deref() == Some(anchor_id))
            });
            if !carries {
                out.error(
                    "CLAIM_SRC_UNRESOLVED",
                    &file.rel_path,
                    format!(
                        "claim {id} names source {source}, but {normalized} carries no claim {anchor_id}"
                    ),
                );
            }
        } else if (anchor.starts_with('L') && anchor[1..].chars().all(|c| c.is_ascii_digit()))
            || (!anchor.is_empty() && anchor.chars().all(|c| c.is_ascii_digit()))
        {
            out.error(
                "CLAIM_SRC_UNRESOLVED",
                &file.rel_path,
                format!(
                    "claim {id} names source {source} by line number, and the profile names the file and the claim, never a line"
                ),
            );
        }
    }
}

fn check_supersession_graph(bundle: &Bundle, known: &BTreeSet<&String>, out: &mut Findings) {
    let mut edges: BTreeMap<&str, &str> = BTreeMap::new();
    let mut claimants: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for file in &bundle.files {
        if !matches!(file.layer, Layer::Daily | Layer::Curated) {
            continue;
        }
        for claim in &file.claims {
            let (Some(id), Some(target)) = (&claim.id, &claim.supersedes) else {
                continue;
            };
            if id == target {
                out.error(
                    "CLAIM_SUPERSEDES_SELF",
                    &file.rel_path,
                    format!("claim {target} supersedes itself"),
                );
                continue;
            }
            if !known.contains(target) {
                out.error(
                    "CLAIM_SUPERSEDES_DANGLING",
                    &file.rel_path,
                    format!("claim supersedes {target}, which no layer carries"),
                );
                continue;
            }
            claimants
                .entry(target.as_str())
                .or_default()
                .push(id.as_str());
            edges.entry(id.as_str()).or_insert(target.as_str());
        }
    }
    for (target, ids) in &claimants {
        let mut distinct: Vec<&str> = ids.clone();
        distinct.sort_unstable();
        distinct.dedup();
        if distinct.len() > 1 {
            out.warning(
                "CLAIM_SUPERSEDE_FORK",
                "claims.md",
                format!(
                    "claim {target} is superseded by more than one claim, {}",
                    distinct.join(", ")
                ),
            );
        }
    }
    // Cycle detection. A chain that returns to its start means no
    // claim in the loop is current, which is a broken chain.
    let mut reported: BTreeSet<Vec<&str>> = BTreeSet::new();
    for start in edges.keys() {
        let mut walk: Vec<&str> = Vec::new();
        let mut current = *start;
        loop {
            if let Some(index) = walk.iter().position(|node| *node == current) {
                let mut cycle: Vec<&str> = walk[index..].to_vec();
                cycle.sort_unstable();
                if reported.insert(cycle.clone()) {
                    out.error(
                        "CLAIM_SUPERSEDE_CYCLE",
                        "claims.md",
                        format!(
                            "claims {} supersede each other in a cycle, so none of them is current",
                            cycle.join(", ")
                        ),
                    );
                }
                break;
            }
            walk.push(current);
            let Some(next) = edges.get(current) else {
                break;
            };
            current = next;
        }
    }
}

fn check_promotions(
    curated: &BTreeMap<String, &ConceptFile>,
    daily_claims: &BTreeMap<String, &crate::claim::Claim>,
    out: &mut Findings,
) {
    for (id, curated_file) in curated {
        let Some(daily_claim) = daily_claims.get(id) else {
            continue;
        };
        let Some(curated_claim) = curated_file
            .claims
            .iter()
            .find(|claim| claim.id.as_deref() == Some(id.as_str()))
        else {
            continue;
        };
        let mut drifted: Vec<&str> = Vec::new();
        if curated_claim.text != daily_claim.text {
            drifted.push("text");
        }
        if curated_claim.quote != daily_claim.quote {
            drifted.push("quote");
        }
        if curated_claim.sources != daily_claim.sources {
            drifted.push("sources");
        }
        if !drifted.is_empty() {
            out.error(
                "CLAIM_PROMOTION_DRIFT",
                &curated_file.rel_path,
                format!(
                    "promoted claim {id} differs from its daily original in its {}, and a changed copy is a new claim, not a promotion",
                    drifted.join(", ")
                ),
            );
        }
    }
}

fn check_reverse_links(bundle: &Bundle, known: &BTreeSet<&String>, out: &mut Findings) {
    // Index every claim by identity for the link comparison.
    let mut by_id: BTreeMap<&str, &crate::claim::Claim> = BTreeMap::new();
    for file in &bundle.files {
        for claim in &file.claims {
            if let Some(id) = &claim.id {
                by_id.entry(id.as_str()).or_insert(claim);
            }
        }
    }
    for file in &bundle.files {
        if file.layer != Layer::Curated {
            continue;
        }
        for claim in &file.claims {
            let (Some(id), Some(replacement)) = (&claim.id, &claim.superseded_by) else {
                continue;
            };
            if !known.contains(replacement) {
                out.error(
                    "CLAIM_REVERSE_DANGLING",
                    &file.rel_path,
                    format!("claim {id} names superseded-by {replacement}, which no layer carries"),
                );
                continue;
            }
            let agrees = by_id
                .get(replacement.as_str())
                .and_then(|other| other.supersedes.as_ref())
                .is_some_and(|target| target == id);
            if !agrees {
                out.error(
                    "CLAIM_REVERSE_DANGLING",
                    &file.rel_path,
                    format!(
                        "claim {id} names superseded-by {replacement}, but that claim does not supersede it"
                    ),
                );
            }
        }
    }
}

fn check_sections(bundle: &Bundle, out: &mut Findings) {
    // For every supersession whose replaced claim has been promoted
    // into a curated concept, that concept must carry the superseded
    // claims section entry the profile requires, because the section
    // is where the registry's date and reason come from.
    for file in &bundle.files {
        if file.layer != Layer::Curated {
            continue;
        }
        let entries = section_entries_in(&file.body);
        for claim in &file.claims {
            let Some(id) = &claim.id else {
                continue;
            };
            let claimants: Vec<&str> = bundle
                .files
                .iter()
                .flat_map(|other| other.claims.iter())
                .filter(|c| c.supersedes.as_deref() == Some(id.as_str()))
                .filter_map(|c| c.id.as_deref())
                .collect();
            if claimants.is_empty() {
                continue;
            }
            let covered = entries
                .iter()
                .any(|entry| entry.old == *id && claimants.contains(&entry.replacement.as_str()));
            if !covered {
                out.error(
                    "CLAIM_SECTION_MISSING",
                    &file.rel_path,
                    format!(
                        "curated concept holds superseded claim {id} but its superseded claims section names no entry for it"
                    ),
                );
            }
        }
    }
}

fn check_registry(bundle: &Bundle, known: &BTreeSet<&String>, out: &mut Findings) {
    let Some(registry) = bundle.file("claims.md") else {
        return;
    };
    // Parse the registry's entries. The rendered format is one flat
    // list under the registry heading, one bullet per claim, whose
    // first token is the claim identity and whose withheld entries
    // open with the word withheld.
    let mut listed: BTreeSet<String> = BTreeSet::new();
    let mut withheld_lines: Vec<&str> = Vec::new();
    for line in registry.body.lines() {
        if let Some(rest) = line.strip_prefix("- ") {
            let id = rest.split_whitespace().next().unwrap_or("");
            if !id.is_empty() {
                listed.insert(id.to_owned());
            }
            if rest.contains(" withheld, superseded by ") {
                withheld_lines.push(line);
            }
        }
    }
    for id in known {
        if !listed.contains(id.as_str()) {
            out.error(
                "REGISTRY_STALE",
                "claims.md",
                format!("claim {id} is missing from the registry"),
            );
        }
    }
    for id in &listed {
        if !known.contains(&id) {
            out.error(
                "REGISTRY_STALE",
                "claims.md",
                format!("the registry names claim {id}, which no layer carries"),
            );
        }
    }
    // A superseded claim must be presented as withheld, in the
    // rendered format, with its replacement named, and with its
    // reason whenever the curated record carries one.
    let mut replacements: BTreeMap<&str, &str> = BTreeMap::new();
    for file in &bundle.files {
        if !matches!(file.layer, Layer::Daily | Layer::Curated) {
            continue;
        }
        for claim in &file.claims {
            if let (Some(id), Some(target)) = (&claim.id, &claim.supersedes) {
                if !known.contains(target) {
                    // A pointer no layer carries is the supersession
                    // graph's dangling finding, and the renderer can
                    // only present claims the bundle carries, so the
                    // pointer creates no withheld obligation here.
                    continue;
                }
                // First claimant wins, matching the renderer's
                // deterministic choice for a forked supersession.
                replacements.entry(target.as_str()).or_insert(id.as_str());
            }
        }
    }
    let section_entries: Vec<crate::registry::SupersededEntry> = bundle
        .files
        .iter()
        .filter(|file| file.layer == Layer::Curated)
        .flat_map(|file| section_entries_in(&file.body))
        .collect();
    for (old, new) in &replacements {
        let prefix = format!("- {old} withheld, superseded by {new}");
        let line = withheld_lines.iter().find(|line| line.starts_with(&prefix));
        match line {
            None => out.error(
                "REGISTRY_WITHHELD",
                "claims.md",
                format!(
                    "superseded claim {old} is not presented as withheld with its replacement {new} on one line"
                ),
            ),
            Some(line) => {
                let reason = section_entries
                    .iter()
                    .find(|entry| entry.old == *old && entry.replacement == *new)
                    .and_then(|entry| entry.reason.as_ref());
                if let Some(reason) = reason
                    && !line.contains(&format!("Reason, {reason}"))
                {
                    out.error(
                        "REGISTRY_WITHHELD",
                        "claims.md",
                        format!(
                            "the withheld line for claim {old} does not carry the reason the curated record gives"
                        ),
                    );
                }
            }
        }
    }
}
