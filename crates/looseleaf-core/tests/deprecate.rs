//! Integration tests for concept deprecation. The act's three
//! keys are asserted through the parsed frontmatter and the
//! checker, and refusals are asserted to write nothing.

mod common;

use std::fs;
use std::path::PathBuf;

use looseleaf_core::bundle::read_bundle;
use looseleaf_core::check::check_bundle;
use looseleaf_core::frontmatter::Value;
use looseleaf_core::stamps::{Deprecation, deprecate_concept};

fn fixture_copy(name: &str) -> PathBuf {
    common::fixture_copy("deprecate", name)
}

const AT: &str = "2026-11-24T22:00:00Z";

fn scalar_at(dir: &std::path::Path, path: &str, key: &str) -> Option<String> {
    let bundle = read_bundle(dir);
    let file = bundle
        .files
        .iter()
        .find(|file| file.rel_path == path)
        .expect("the file is in the bundle");
    match file.frontmatter.as_ref()?.get(key) {
        Some(Value::Scalar(text)) => Some(text.clone()),
        _ => None,
    }
}

#[test]
fn deprecate_writes_the_three_keys_and_then_reports_current() {
    let dir = fixture_copy("basic");
    let outcome = deprecate_concept(&dir, "people/ada.md", "curated/standing-facts.md", AT)
        .expect("deprecate ada");
    assert_eq!(outcome, Deprecation::Deprecated);
    assert_eq!(
        scalar_at(&dir, "people/ada.md", "status").as_deref(),
        Some("deprecated")
    );
    assert_eq!(
        scalar_at(&dir, "people/ada.md", "superseded_by").as_deref(),
        Some("curated/standing-facts.md")
    );
    let text = fs::read_to_string(dir.join("people/ada.md")).expect("read ada");
    let outcome = deprecate_concept(&dir, "people/ada.md", "curated/standing-facts.md", AT)
        .expect("repeat the deprecation");
    assert_eq!(outcome, Deprecation::AlreadyCurrent);
    assert_eq!(
        fs::read_to_string(dir.join("people/ada.md")).expect("reread ada"),
        text,
        "a repeat writes nothing"
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn deprecate_completes_a_partial_act() {
    let dir = fixture_copy("partial");
    let path = dir.join("people/ada.md");
    let text = fs::read_to_string(&path).expect("read ada");
    let mutated = text.replace(
        "generated: { by:",
        "superseded_by: curated/standing-facts.md\ngenerated: { by:",
    );
    assert_ne!(mutated, text);
    fs::write(&path, &mutated).expect("write the pointer only state");
    let outcome = deprecate_concept(&dir, "people/ada.md", "curated/standing-facts.md", AT)
        .expect("complete the act");
    assert_eq!(outcome, Deprecation::Deprecated);
    assert_eq!(
        scalar_at(&dir, "people/ada.md", "status").as_deref(),
        Some("deprecated")
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn deprecate_refuses_a_repoint_and_writes_nothing() {
    let dir = fixture_copy("repoint");
    deprecate_concept(&dir, "people/ada.md", "curated/standing-facts.md", AT)
        .expect("deprecate ada");
    fs::create_dir_all(dir.join("topics")).expect("make the layer");
    fs::write(
        dir.join("topics/new-home.md"),
        "---\ntype: Topic\ntitle: \"New home\"\ndescription: \"A topic concept standing as a second replacement.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\n---\n\n# New home\n",
    )
    .expect("write a second replacement");
    let before = fs::read_to_string(dir.join("people/ada.md")).expect("read ada");
    let error = deprecate_concept(&dir, "people/ada.md", "topics/new-home.md", AT)
        .expect_err("repointing must refuse");
    assert!(error.to_string().contains("new decision"), "got {error}");
    assert_eq!(
        fs::read_to_string(dir.join("people/ada.md")).expect("reread ada"),
        before
    );
}

#[test]
fn deprecate_refuses_the_targets_it_may_not_touch() {
    let dir = fixture_copy("refusals");
    fs::create_dir_all(dir.join("directives")).expect("make the layer");
    fs::write(
        dir.join("directives/standing-order.md"),
        "---\ntype: Directive\ntitle: \"Standing order\"\ndescription: \"A standing directive for the fixture.\"\ngenerated: { by: \"human:david\", at: \"2026-11-02T01:00:00Z\" }\n---\n\n# Standing order\n",
    )
    .expect("write a directive");
    fs::write(dir.join("directives/index.md"), "# Directives\n").expect("write the index");
    let cases: [(&str, &str, &str); 6] = [
        (
            "daily/2026-11-01.md",
            "curated/standing-facts.md",
            "daily layer",
        ),
        ("claims.md", "curated/standing-facts.md", "registry"),
        (
            "directives/standing-order.md",
            "curated/standing-facts.md",
            "directive",
        ),
        ("people/ada.md", "people/ada.md", "itself"),
        ("people/ada.md", "daily/2026-11-01.md", "daily layer"),
        ("people/ada.md", "curated/missing.md", "no file at"),
    ];
    for (path, replacement, phrase) in cases {
        let target = dir.join(path);
        let before = fs::read_to_string(&target).ok();
        let error =
            deprecate_concept(&dir, path, replacement, AT).expect_err("deprecate must refuse");
        assert!(
            error.to_string().contains(phrase),
            "for {path} toward {replacement}, got {error}"
        );
        assert_eq!(
            fs::read_to_string(&target).ok(),
            before,
            "a refusal writes nothing, for {path}"
        );
    }
    let error = deprecate_concept(
        &dir,
        "people/ada.md",
        "curated/standing-facts.md",
        "not-a-date",
    )
    .expect_err("a garbage stamp must refuse");
    assert!(
        error.to_string().contains("real calendar date"),
        "got {error}"
    );
}
