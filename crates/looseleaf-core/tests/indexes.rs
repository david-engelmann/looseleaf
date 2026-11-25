//! Integration tests for layer index regeneration. The renderer
//! reproduces the clean fixture's hand-written indexes byte for
//! byte, which pins the generated format the same way the registry
//! tests pin claims.md. Regeneration is a no-op on current indexes
//! and repairs a missing one, after which the checker agrees the
//! bundle is clean.

use std::fs;

use looseleaf_core::bundle::{Layer, read_bundle};
use looseleaf_core::check_bundle;
use looseleaf_core::indexes::{regenerate_indexes, render_layer_index};

mod common;

use common::clean_fixture;

fn fixture_copy(name: &str) -> std::path::PathBuf {
    common::fixture_copy("indexes", name)
}

#[test]
fn renderer_reproduces_the_fixture_indexes_byte_for_byte() {
    let bundle = read_bundle(&clean_fixture());
    for (layer, rel) in [
        (Layer::Curated, "curated/index.md"),
        (Layer::People, "people/index.md"),
        (Layer::Groups, "groups/index.md"),
        (Layer::Topics, "topics/index.md"),
        (Layer::Daily, "daily/index.md"),
        (Layer::Bank, "bank/index.md"),
        (Layer::Synthesis, "synthesis/index.md"),
        (Layer::Directives, "directives/index.md"),
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
    fs::remove_dir_all(dir.join("topics")).expect("clear the standing topics layer");
    fs::create_dir(dir.join("topics")).expect("create topics dir");
    let written = regenerate_indexes(&dir).expect("regenerate");
    assert_eq!(written, ["topics/index.md"]);
    let index = fs::read_to_string(dir.join("topics/index.md")).expect("read index");
    assert_eq!(index, "# Topics\n");
}

#[test]
fn a_nested_concept_links_by_its_path_inside_the_layer() {
    let dir = fixture_copy("index-nested");
    let nested = dir.join("curated/sub");
    fs::create_dir_all(&nested).expect("create nested dir");
    fs::write(
        nested.join("inner.md"),
        "---\ntype: Curated Memory\ntitle: \"Inner note\"\ndescription: \"A nested concept.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\n---\n\n# Inner note\n",
    )
    .expect("write nested concept");
    let bundle = read_bundle(&dir);
    let rendered = render_layer_index(&bundle, Layer::Curated);
    assert!(
        rendered.contains("* [Inner note](sub/inner.md)"),
        "got\n{rendered}"
    );
}

#[cfg(unix)]
#[test]
fn regeneration_refuses_a_symlinked_layer_directory() {
    let dir = fixture_copy("index-symlink");
    let external =
        std::env::temp_dir().join(format!("looseleaf-indexes-external-{}", std::process::id()));
    if external.exists() {
        fs::remove_dir_all(&external).expect("clear external");
    }
    fs::create_dir_all(&external).expect("create external");
    fs::write(external.join("index.md"), "# External\n").expect("write external index");
    fs::remove_dir_all(dir.join("curated")).expect("remove curated dir");
    std::os::unix::fs::symlink(&external, dir.join("curated")).expect("link curated");
    let error = regenerate_indexes(&dir).expect_err("refuse a symlinked layer");
    assert!(error.to_string().contains("symbolic link"), "got {error}");
    let external_index =
        fs::read_to_string(external.join("index.md")).expect("read external index");
    assert_eq!(
        external_index, "# External\n",
        "nothing may be written through the link"
    );
    fs::remove_dir_all(&external).expect("clean external");
}
