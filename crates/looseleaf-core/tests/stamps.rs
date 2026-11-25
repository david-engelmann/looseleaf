//! Integration tests for the frontmatter stamps. The stamp shapes
//! are asserted through the parsed frontmatter and the checker, and
//! every refusal is asserted to write nothing.

mod common;

use std::fs;
use std::path::Path;

use looseleaf_core::bundle::{ConceptFile, read_bundle};
use looseleaf_core::check::check_bundle;
use looseleaf_core::frontmatter::Value;
use looseleaf_core::stamps::{PASS_ACTOR, Stamp, stamp_generated, stamp_verified};

fn fixture_copy(name: &str) -> std::path::PathBuf {
    common::fixture_copy("stamps", name)
}

fn concept(dir: &Path, rel: &str) -> ConceptFile {
    read_bundle(dir)
        .files
        .into_iter()
        .find(|file| file.rel_path == rel)
        .unwrap_or_else(|| panic!("no concept at {rel}"))
}

fn by_at(value: &Value) -> (String, String) {
    let Value::Map(entries) = value else {
        panic!("expected a mapping, got {value:?}");
    };
    let mut by = None;
    let mut at = None;
    for (key, item) in entries {
        if let Value::Scalar(text) = item {
            match key.as_str() {
                "by" => by = Some(text.clone()),
                "at" => at = Some(text.clone()),
                _ => {}
            }
        }
    }
    (by.expect("a by"), at.expect("an at"))
}

fn verified_records(dir: &Path, rel: &str) -> Vec<(String, String)> {
    let file = concept(dir, rel);
    let frontmatter = file.frontmatter.as_ref().expect("frontmatter");
    match frontmatter.get("verified") {
        None => Vec::new(),
        Some(value @ Value::Map(_)) => vec![by_at(value)],
        Some(Value::List(items)) => items.iter().map(by_at).collect(),
        Some(other) => panic!("unexpected verified shape {other:?}"),
    }
}

#[test]
fn stamp_generated_restamps_and_then_reports_current() {
    let dir = fixture_copy("restamp");
    let before = concept(&dir, "people/ada.md");
    let outcome = stamp_generated(&dir, "people/ada.md", PASS_ACTOR, "2026-11-24T00:00:00Z")
        .expect("stamp generated");
    assert_eq!(outcome, Stamp::Stamped);
    let after = concept(&dir, "people/ada.md");
    let generated = after
        .frontmatter
        .as_ref()
        .expect("frontmatter")
        .get("generated")
        .expect("a generated entry");
    assert_eq!(
        by_at(generated),
        (PASS_ACTOR.to_owned(), "2026-11-24T00:00:00Z".to_owned())
    );
    assert_eq!(after.body, before.body, "the body must not change");
    let text = fs::read_to_string(dir.join("people/ada.md")).expect("read ada");
    let outcome = stamp_generated(&dir, "people/ada.md", PASS_ACTOR, "2026-11-24T00:00:00Z")
        .expect("stamp again");
    assert_eq!(outcome, Stamp::AlreadyCurrent);
    assert_eq!(
        fs::read_to_string(dir.join("people/ada.md")).expect("read ada again"),
        text,
        "a current stamp writes nothing"
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn stamp_generated_honours_an_explicit_actor() {
    let dir = fixture_copy("actor");
    stamp_generated(&dir, "people/ada.md", "human:david", "2026-11-24T00:00:00Z")
        .expect("stamp with a human actor");
    let file = concept(&dir, "people/ada.md");
    let generated = file
        .frontmatter
        .as_ref()
        .expect("frontmatter")
        .get("generated")
        .expect("a generated entry");
    assert_eq!(by_at(generated).0, "human:david");
}

#[test]
fn stamp_verified_walks_the_three_shapes() {
    let dir = fixture_copy("verified");
    let rel = "curated/standing-facts.md";
    assert_eq!(verified_records(&dir, rel), Vec::new());
    let outcome = stamp_verified(&dir, rel, "checker", "2026-11-24T09:00:00Z").expect("first");
    assert_eq!(outcome, Stamp::Stamped);
    assert_eq!(
        verified_records(&dir, rel),
        vec![("checker".to_owned(), "2026-11-24T09:00:00Z".to_owned())]
    );
    stamp_verified(&dir, rel, "checker", "2026-11-24T10:00:00Z").expect("second");
    stamp_verified(&dir, rel, "reviewer", "2026-11-24T11:00:00Z").expect("third");
    assert_eq!(
        verified_records(&dir, rel),
        vec![
            ("checker".to_owned(), "2026-11-24T09:00:00Z".to_owned()),
            ("checker".to_owned(), "2026-11-24T10:00:00Z".to_owned()),
            ("reviewer".to_owned(), "2026-11-24T11:00:00Z".to_owned()),
        ]
    );
    let text = fs::read_to_string(dir.join(rel)).expect("read the concept");
    let outcome = stamp_verified(&dir, rel, "checker", "2026-11-24T10:00:00Z").expect("repeat");
    assert_eq!(outcome, Stamp::AlreadyCurrent);
    assert_eq!(
        fs::read_to_string(dir.join(rel)).expect("read again"),
        text,
        "a recorded verification is never duplicated"
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn stamps_refuse_the_files_they_may_not_touch() {
    let dir = fixture_copy("refusals");
    fs::create_dir_all(dir.join("directives")).expect("make the directives layer");
    fs::write(
        dir.join("directives/standing-order.md"),
        "---\ntype: Directive\ntitle: \"Standing order\"\ndescription: \"A standing order for the fixture.\"\ngenerated: { by: \"human:david\", at: \"2026-11-02T00:00:00Z\" }\n---\n\n# Standing order\n",
    )
    .expect("write the directive");
    let cases: [(&str, &str); 5] = [
        ("daily/2026-11-01.md", "daily layer"),
        ("claims.md", "registry"),
        ("curated/index.md", "reserved"),
        ("curated/missing.md", "no file at"),
        ("directives/standing-order.md", "directive"),
    ];
    for (path, phrase) in cases {
        let error = stamp_generated(&dir, path, PASS_ACTOR, "2026-11-24T00:00:00Z")
            .expect_err("stamp generated must refuse");
        assert!(
            error.to_string().contains(phrase),
            "for {path}, got {error}"
        );
        let error = stamp_verified(&dir, path, "checker", "2026-11-24T00:00:00Z")
            .expect_err("stamp verified must refuse");
        assert!(
            error.to_string().contains(phrase),
            "for {path}, got {error}"
        );
    }
}

#[test]
fn stamps_validate_their_arguments() {
    let dir = fixture_copy("validation");
    let error = stamp_generated(&dir, "people/ada.md", PASS_ACTOR, "not-a-date")
        .expect_err("a garbage stamp must refuse");
    assert!(
        error.to_string().contains("real calendar date"),
        "got {error}"
    );
    let error = stamp_generated(&dir, "people/ada.md", PASS_ACTOR, "2026-02-30T00:00:00Z")
        .expect_err("an impossible date must refuse");
    assert!(
        error.to_string().contains("real calendar date"),
        "got {error}"
    );
    let error = stamp_verified(&dir, "people/ada.md", "", "2026-11-24T00:00:00Z")
        .expect_err("an empty actor must refuse");
    assert!(error.to_string().contains("non-empty"), "got {error}");
    let error = stamp_verified(&dir, "people/ada.md", "two\nlines", "2026-11-24T00:00:00Z")
        .expect_err("a two line actor must refuse");
    assert!(error.to_string().contains("non-empty"), "got {error}");
}

#[test]
fn stamp_verified_refuses_a_malformed_existing_entry() {
    let dir = fixture_copy("malformed");
    let path = dir.join("people/ada.md");
    let text = fs::read_to_string(&path).expect("read ada");
    let mutated = text.replace(
        "generated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }",
        "generated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nverified: soon",
    );
    assert_ne!(mutated, text);
    fs::write(&path, &mutated).expect("write the mutation");
    let error = stamp_verified(&dir, "people/ada.md", "checker", "2026-11-24T00:00:00Z")
        .expect_err("a malformed verified entry must refuse");
    assert!(error.to_string().contains("not a mapping"), "got {error}");
    assert_eq!(
        fs::read_to_string(&path).expect("read ada again"),
        mutated,
        "a refusal writes nothing"
    );
}
