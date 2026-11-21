//! Integration tests for the writer slice. The registry tests prove
//! three things against the clean fixture. The renderer reproduces
//! the fixture's hand-written registry byte for byte at the same
//! stamp, which pins the generated format. Regeneration is a no-op on
//! a current registry and repairs a stale one, after which the
//! checker agrees the bundle is clean, which pins writer and checker
//! to the same rules.

use std::fs;
use std::path::{Path, PathBuf};

use looseleaf_core::bundle::read_bundle;
use looseleaf_core::check_bundle;
use looseleaf_core::frontmatter::Value;
use looseleaf_core::registry::{regenerate_registry, render_registry};
use looseleaf_core::write::upsert_frontmatter;

const STAMP: &str = "2026-11-02T00:00:00Z";

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
    let dir = std::env::temp_dir().join(format!("looseleaf-write-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear old fixture copy");
    }
    copy_dir(&clean_fixture(), &dir);
    dir
}

#[test]
fn renderer_reproduces_the_fixture_registry_byte_for_byte() {
    let bundle = read_bundle(&clean_fixture());
    let rendered = render_registry(&bundle, STAMP);
    let expected =
        fs::read_to_string(clean_fixture().join("claims.md")).expect("read fixture registry");
    assert_eq!(rendered, expected);
}

#[test]
fn regeneration_is_a_no_op_on_a_current_registry() {
    let dir = fixture_copy("noop");
    let changed = regenerate_registry(&dir, STAMP).expect("regenerate");
    assert!(!changed);
}

#[test]
fn regeneration_repairs_a_missing_registry_and_the_checker_agrees() {
    let dir = fixture_copy("missing");
    fs::remove_file(dir.join("claims.md")).expect("remove registry");
    let stale = check_bundle(&dir);
    assert!(
        stale
            .findings
            .iter()
            .any(|finding| finding.code == "LAYOUT_REGISTRY_MISSING"),
        "the missing registry must be a finding before repair, got {:?}",
        stale.findings
    );
    let changed = regenerate_registry(&dir, STAMP).expect("regenerate");
    assert!(changed);
    let expected =
        fs::read_to_string(clean_fixture().join("claims.md")).expect("read fixture registry");
    let repaired = fs::read_to_string(dir.join("claims.md")).expect("read repaired registry");
    assert_eq!(repaired, expected);
    let after = check_bundle(&dir);
    assert!(
        after.findings.is_empty(),
        "the repaired bundle must check clean, got {:?}",
        after.findings
    );
}

#[test]
fn regeneration_picks_up_a_new_claim() {
    let dir = fixture_copy("new-claim");
    let daily_path = dir.join("daily").join("2026-11-02.md");
    let daily = fs::read_to_string(&daily_path).expect("read daily file");
    let extended = format!(
        "{daily}\n- [fact|medium] The workshop has a second room.\n  - claim: 44444444444444444444444444444444\n  - quote: \"a second room exists\"\n  - sources: message:m-4\n"
    );
    fs::write(&daily_path, extended).expect("extend daily file");
    let changed = regenerate_registry(&dir, STAMP).expect("regenerate");
    assert!(changed);
    let registry = fs::read_to_string(dir.join("claims.md")).expect("read registry");
    assert!(
        registry.contains("- 44444444444444444444444444444444 current."),
        "the new claim must appear as current, got:\n{registry}"
    );
    let after = check_bundle(&dir);
    assert!(
        after.findings.is_empty(),
        "the bundle must check clean after regeneration, got {:?}",
        after.findings
    );
}

#[test]
fn upsert_on_a_bundle_file_preserves_the_extension_key() {
    let dir = fixture_copy("upsert");
    let person_path = dir.join("people").join("ada.md");
    let before = fs::read_to_string(&person_path).expect("read person file");
    let verified = Value::List(vec![Value::Map(vec![
        ("by".to_owned(), Value::Scalar("human".to_owned())),
        (
            "at".to_owned(),
            Value::Scalar("2026-11-21T00:00:00Z".to_owned()),
        ),
    ])]);
    let result = upsert_frontmatter(&before, "verified", &verified).expect("upsert");
    assert!(result.changed);
    assert!(
        result.text.contains("x-looseleaf-rank: 1\n"),
        "the extension key line must survive, got:\n{}",
        result.text
    );
    fs::write(&person_path, &result.text).expect("write person file");
    let after = check_bundle(&dir);
    assert!(
        after.findings.is_empty(),
        "the bundle must still check clean, got {:?}",
        after.findings
    );
}
