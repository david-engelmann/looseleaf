//! Integration tests for the bundle checker, run against a clean
//! fixture bundle and copies of it with one planted defect each. A
//! test that cannot fail for the defect it plants is worthless, so
//! every defect test asserts the exact rule code it exists to catch.

use std::fs;
use std::path::{Path, PathBuf};

use looseleaf_core::check_bundle;

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

fn defect_copy(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("looseleaf-check-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear old fixture copy");
    }
    copy_dir(&clean_fixture(), &dir);
    dir
}

fn mutate(dir: &Path, rel: &str, edit: impl Fn(&str) -> String) {
    let path = dir.join(rel);
    let text = fs::read_to_string(&path).expect("read fixture file");
    fs::write(&path, edit(&text)).expect("write mutated fixture");
}

fn codes(dir: &Path) -> Vec<&'static str> {
    check_bundle(dir)
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect()
}

#[test]
fn clean_bundle_passes_without_findings() {
    let report = check_bundle(&clean_fixture());
    assert_eq!(
        report.findings.len(),
        0,
        "clean fixture drew findings: {:?}",
        report.findings
    );
    assert!(report.passed());
    assert_eq!(report.files_checked, 11);
}

#[test]
fn missing_type_is_caught() {
    let dir = defect_copy("missing-type");
    mutate(&dir, "curated/standing-facts.md", |text| {
        text.replace("type: Curated Memory\n", "")
    });
    let found = codes(&dir);
    assert!(found.contains(&"FM_TYPE_MISSING"), "got {found:?}");
}

#[test]
fn dangling_supersession_is_caught() {
    let dir = defect_copy("dangling");
    mutate(&dir, "daily/2026-11-02.md", |text| {
        text.replace(
            "supersedes: 11111111111111111111111111111111",
            "supersedes: 99999999999999999999999999999999",
        )
    });
    let found = codes(&dir);
    assert!(
        found.contains(&"CLAIM_SUPERSEDES_DANGLING"),
        "got {found:?}"
    );
}

#[test]
fn duplicate_claim_identity_is_caught() {
    let dir = defect_copy("duplicate");
    mutate(&dir, "daily/2026-11-02.md", |text| {
        format!(
            "{text}\n- [fact|high] A repeated identity.\n  - claim: 22222222222222222222222222222222\n  - quote: \"tea, not coffee\"\n  - sources: message:m-9\n"
        )
    });
    let found = codes(&dir);
    assert!(found.contains(&"CLAIM_DUP_ID"), "got {found:?}");
}

#[test]
fn registry_drift_is_caught() {
    let dir = defect_copy("registry");
    mutate(&dir, "claims.md", |text| {
        text.replace("- 22222222222222222222222222222222 current.\n", "")
    });
    let found = codes(&dir);
    assert!(found.contains(&"REGISTRY_STALE"), "got {found:?}");
}

#[test]
fn unresolved_source_is_caught() {
    let dir = defect_copy("source");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "resource: daily/2026-11-01.md",
            "resource: daily/2026-12-25.md",
        )
    });
    let found = codes(&dir);
    assert!(found.contains(&"SRC_UNRESOLVED"), "got {found:?}");
}
