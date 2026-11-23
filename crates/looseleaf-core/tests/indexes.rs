//! Integration tests for layer index regeneration. The renderer
//! reproduces the clean fixture's hand-written indexes byte for
//! byte, which pins the generated format the same way the registry
//! tests pin claims.md. Regeneration is a no-op on current indexes
//! and repairs a missing one, after which the checker agrees the
//! bundle is clean.

use std::fs;
use std::path::{Path, PathBuf};

use looseleaf_core::bundle::{Layer, read_bundle};
use looseleaf_core::check_bundle;
use looseleaf_core::indexes::{regenerate_indexes, render_layer_index};

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
    let dir = std::env::temp_dir().join(format!("looseleaf-indexes-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear old fixture copy");
    }
    copy_dir(&clean_fixture(), &dir);
    dir
}

#[test]
fn renderer_reproduces_the_fixture_indexes_byte_for_byte() {
    let bundle = read_bundle(&clean_fixture());
    for (layer, rel) in [
        (Layer::Curated, "curated/index.md"),
        (Layer::People, "people/index.md"),
        (Layer::Daily, "daily/index.md"),
    ] {
        let rendered = render_layer_index(&bundle, layer);
        let expected = fs::read_to_string(clean_fixture().join(rel)).expect("read fixture index");
        assert_eq!(rendered, expected, "index for {rel}");
    }
}

#[test]
fn regeneration_is_a_no_op_on_current_indexes() {
    let dir = fixture_copy("noop");
    let written = regenerate_indexes(&dir).expect("regenerate");
    assert_eq!(written.len(), 0, "got {written:?}");
}

#[test]
fn regeneration_repairs_a_missing_index_and_the_checker_agrees() {
    let dir = fixture_copy("repair");
    fs::remove_file(dir.join("people/index.md")).expect("remove index");
    let written = regenerate_indexes(&dir).expect("regenerate");
    assert_eq!(written, ["people/index.md"]);
    let restored = fs::read_to_string(dir.join("people/index.md")).expect("read restored index");
    let expected =
        fs::read_to_string(clean_fixture().join("people/index.md")).expect("read fixture index");
    assert_eq!(restored, expected);
    let report = check_bundle(&dir);
    assert_eq!(
        report.findings.len(),
        0,
        "the bundle must check clean after repair, got {:?}",
        report.findings
    );
}

#[test]
fn a_new_person_lands_in_rank_order() {
    let dir = fixture_copy("rank");
    let bob = "---\ntype: Person\ntitle: \"Bob\"\ndescription: \"Teammate who reviews the fixture.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\nx-looseleaf-rank: 0\n---\n\n# Bob\n";
    fs::write(dir.join("people/bob.md"), bob).expect("write bob");
    let written = regenerate_indexes(&dir).expect("regenerate");
    assert_eq!(written, ["people/index.md"]);
    let index = fs::read_to_string(dir.join("people/index.md")).expect("read index");
    assert_eq!(index, "# People\n\n* [Bob](bob.md)\n* [Ada](ada.md)\n");
}

#[test]
fn an_empty_layer_gets_a_heading_and_no_bullets() {
    let dir = fixture_copy("empty");
    fs::create_dir(dir.join("topics")).expect("create topics dir");
    let written = regenerate_indexes(&dir).expect("regenerate");
    assert_eq!(written, ["topics/index.md"]);
    let index = fs::read_to_string(dir.join("topics/index.md")).expect("read index");
    assert_eq!(index, "# Topics\n");
}
