//! Integration tests for Phase 2 Slice 2, root index regeneration
//! and dated log appends. The root renderer reproduces the clean
//! fixture's root index byte for byte and preserves its frontmatter
//! exactly when the body has gone stale. Log appends join the
//! newest section on an equal date, open a new section at the top
//! on a newer date, and refuse an older date, and the bundle still
//! checks clean after each mechanical edit.

use std::fs;
use std::path::{Path, PathBuf};

use looseleaf_core::bundle::{Layer, read_bundle};
use looseleaf_core::check_bundle;
use looseleaf_core::indexes::{regenerate_root_index, render_root_index};
use looseleaf_core::logs::{LogScope, append_log};

fn clean_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("clean")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create fixture copy");
    for entry in fs::read_dir(from).expect("read fixture dir") {
        let entry = entry.expect("fixture entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy fixture file");
        }
    }
}

fn fixture_copy(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("looseleaf-slice2-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear old fixture copy");
    }
    copy_dir(&clean_fixture(), &dir);
    dir
}

#[test]
fn root_renderer_reproduces_the_fixture_byte_for_byte() {
    let bundle = read_bundle(&clean_fixture());
    let current = fs::read_to_string(clean_fixture().join("index.md")).expect("read fixture index");
    let rendered = render_root_index(&bundle, &current).expect("render root index");
    assert_eq!(rendered, current);
}

#[test]
fn root_regeneration_is_a_no_op_when_current() {
    let dir = fixture_copy("root-noop");
    let written = regenerate_root_index(&dir).expect("regenerate root index");
    assert_eq!(written, None);
}

#[test]
fn root_regeneration_preserves_the_frontmatter_byte_for_byte() {
    let dir = fixture_copy("root-stale");
    let original = fs::read_to_string(dir.join("index.md")).expect("read root index");
    let stale = original.replace(
        "* [People](people/) - 1 concept",
        "* [People](people/) - 9 concepts",
    );
    fs::write(dir.join("index.md"), stale).expect("make the body stale");
    let written = regenerate_root_index(&dir).expect("regenerate root index");
    assert_eq!(written.as_deref(), Some("index.md"));
    let restored = fs::read_to_string(dir.join("index.md")).expect("read restored index");
    assert_eq!(restored, original);
}

#[test]
fn a_new_layer_joins_the_root_list_in_profile_order() {
    let dir = fixture_copy("root-layer");
    fs::create_dir(dir.join("topics")).expect("create topics dir");
    let topic = "---\ntype: Topic\ntitle: \"Fixture design\"\ndescription: \"How the clean fixture is shaped.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\n---\n\n# Fixture design\n";
    fs::write(dir.join("topics/fixture-design.md"), topic).expect("write topic");
    let written = regenerate_root_index(&dir).expect("regenerate root index");
    assert_eq!(written.as_deref(), Some("index.md"));
    let index = fs::read_to_string(dir.join("index.md")).expect("read root index");
    assert!(index.contains("* [Topics](topics/) - 1 concept\n* [Daily logs](daily/) - 2 concepts"));
}

#[test]
fn an_append_on_the_newest_date_joins_that_section() {
    let dir = fixture_copy("log-join");
    let rel = append_log(
        &dir,
        LogScope::Root,
        "2026-11-02",
        "Regenerated the claim registry.",
    )
    .expect("append log entry");
    assert_eq!(rel, "log.md");
    let log = fs::read_to_string(dir.join("log.md")).expect("read log");
    assert_eq!(
        log,
        "# Bundle log\n\n## 2026-11-02\n\n- Claim 11111111111111111111111111111111 superseded by claim 33333333333333333333333333333333.\n- Regenerated the claim registry.\n"
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn an_append_on_a_newer_date_opens_a_section_at_the_top() {
    let dir = fixture_copy("log-new");
    append_log(&dir, LogScope::Root, "2026-11-03", "Ran a synthesis pass.").expect("append");
    let log = fs::read_to_string(dir.join("log.md")).expect("read log");
    assert_eq!(
        log,
        "# Bundle log\n\n## 2026-11-03\n\n- Ran a synthesis pass.\n\n## 2026-11-02\n\n- Claim 11111111111111111111111111111111 superseded by claim 33333333333333333333333333333333.\n"
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn an_append_on_an_older_date_is_refused() {
    let dir = fixture_copy("log-old");
    let error = append_log(&dir, LogScope::Root, "2026-11-01", "Backfill.").expect_err("refuse");
    assert!(error.to_string().contains("older"), "got {error}");
    let log = fs::read_to_string(dir.join("log.md")).expect("read log");
    let expected = fs::read_to_string(clean_fixture().join("log.md")).expect("read fixture log");
    assert_eq!(log, expected, "a refused append must not touch the log");
}

#[test]
fn a_layer_append_lands_in_the_layer_log() {
    let dir = fixture_copy("log-layer");
    let rel = append_log(
        &dir,
        LogScope::Layer(Layer::Daily),
        "2026-11-03",
        "Appended the 2026-11-03 log.",
    )
    .expect("append layer entry");
    assert_eq!(rel, "daily/log.md");
    let log = fs::read_to_string(dir.join("daily/log.md")).expect("read daily log");
    assert!(log.starts_with("# Daily layer log\n\n## 2026-11-03\n"));
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn a_missing_log_is_created_with_its_heading() {
    let dir = fixture_copy("log-create");
    fs::remove_file(dir.join("curated/log.md")).expect("remove curated log");
    let rel = append_log(
        &dir,
        LogScope::Layer(Layer::Curated),
        "2026-11-03",
        "Promoted a claim.",
    )
    .expect("append creates the log");
    assert_eq!(rel, "curated/log.md");
    let log = fs::read_to_string(dir.join("curated/log.md")).expect("read created log");
    assert_eq!(
        log,
        "# Curated layer log\n\n## 2026-11-03\n\n- Promoted a claim.\n"
    );
}

#[test]
fn appends_reject_bad_dates_entries_and_scopes() {
    let dir = fixture_copy("log-reject");
    assert!(append_log(&dir, LogScope::Root, "2026-13-40", "Nope.").is_err());
    assert!(append_log(&dir, LogScope::Root, "2026-11-03", "two\nlines").is_err());
    assert!(append_log(&dir, LogScope::Root, "2026-11-03", "  ").is_err());
    assert!(
        append_log(&dir, LogScope::Layer(Layer::People), "2026-11-03", "Nope.").is_err(),
        "the profile gives the people layer no log"
    );
}
