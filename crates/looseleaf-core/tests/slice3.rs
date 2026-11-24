//! Integration tests for Phase 2 Slice 3, the synthesis pass
//! mechanics. Staleness is a report against a supplied stamp and
//! never an edit. Promotion carries a claim's identity, quote and
//! sources unchanged, lands after the target's last claim block,
//! refuses drift, and writes the superseded claims section entry
//! in the shape the registry scanner reads, after which a registry
//! regeneration leaves the bundle checking clean.

use std::fs;
use std::path::{Path, PathBuf};

use looseleaf_core::check_bundle;
use looseleaf_core::pass::{Promotion, promote_claim, staleness_report};
use looseleaf_core::registry::regenerate_registry;

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
    let dir = std::env::temp_dir().join(format!("looseleaf-slice3-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear old fixture copy");
    }
    copy_dir(&clean_fixture(), &dir);
    dir
}

#[test]
fn staleness_reports_against_the_supplied_stamp_only() {
    let dir = fixture_copy("stale");
    let ada = fs::read_to_string(dir.join("people/ada.md")).expect("read ada");
    let ada = ada.replace(
        "x-looseleaf-rank: 1\n",
        "x-looseleaf-rank: 1\nverified: { by: \"human:david\", at: \"2026-10-15T00:00:00Z\" }\nstale_after: \"2026-11-01\"\n",
    );
    fs::write(dir.join("people/ada.md"), ada).expect("give ada a review date");
    let stale = staleness_report(&dir, "2026-11-03T00:00:00Z").expect("report");
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].path, "people/ada.md");
    assert_eq!(stale[0].stale_after, "2026-11-01");
    assert_eq!(
        stale[0].verified_at.as_deref(),
        Some("2026-10-15T00:00:00Z")
    );
    let not_yet = staleness_report(&dir, "2026-11-01T00:00:00Z").expect("report");
    assert_eq!(not_yet.len(), 0, "the date itself has not passed");
    let before = staleness_report(&dir, "2026-10-31").expect("report");
    assert_eq!(before.len(), 0);
    assert!(staleness_report(&dir, "not a stamp").is_err());
}

#[test]
fn promotion_carries_the_claim_unchanged_and_lands_before_the_section() {
    let dir = fixture_copy("promote");
    let outcome = promote_claim(
        &dir,
        "22222222222222222222222222222222",
        "curated/standing-facts.md",
        None,
    )
    .expect("promote the tea claim");
    assert_eq!(outcome, Promotion::Promoted);
    let text = fs::read_to_string(dir.join("curated/standing-facts.md")).expect("read target");
    let block = "- [preference|medium] The user prefers tea over coffee.\n  - claim: 22222222222222222222222222222222\n  - quote: \"tea, not coffee\"\n  - sources: message:m-2\n\n## Superseded claims";
    assert!(text.contains(block), "got\n{text}");
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn promotion_of_a_carried_claim_is_a_no_op() {
    let dir = fixture_copy("promote-noop");
    let before = fs::read_to_string(dir.join("curated/standing-facts.md")).expect("read target");
    let outcome = promote_claim(
        &dir,
        "33333333333333333333333333333333",
        "curated/standing-facts.md",
        Some("the standup time changed"),
    )
    .expect("promote the standup claim again");
    assert_eq!(outcome, Promotion::AlreadyPresent);
    let after = fs::read_to_string(dir.join("curated/standing-facts.md")).expect("read target");
    assert_eq!(after, before);
}

#[test]
fn promotion_refuses_drift_and_writes_nothing() {
    let dir = fixture_copy("promote-drift");
    let target = dir.join("curated/standing-facts.md");
    let text = fs::read_to_string(&target).expect("read target");
    let drifted = text.replace("standup moves to 10:00", "standup moved to ten");
    fs::write(&target, &drifted).expect("drift the curated quote");
    let error = promote_claim(
        &dir,
        "33333333333333333333333333333333",
        "curated/standing-facts.md",
        Some("the standup time changed"),
    )
    .expect_err("promotion must refuse drift");
    assert!(error.to_string().contains("drift"), "got {error}");
    let after = fs::read_to_string(&target).expect("read target");
    assert_eq!(
        after, drifted,
        "a refused promotion must not touch the target"
    );
}

#[test]
fn a_superseding_promotion_writes_the_section_entry() {
    let dir = fixture_copy("promote-supersede");
    let daily = "---\ntype: Daily Log\ntitle: \"Daily log 2026-11-03\"\ndescription: \"Extraction log for 2026-11-03 with claim blocks.\"\ngenerated: { by: \"process:memory-extraction\", at: \"2026-11-03T00:00:00Z\" }\n---\n\n## Verified extraction\n\n- [preference|high] The user prefers coffee over tea now.\n  - claim: 44444444444444444444444444444444\n  - quote: \"coffee these days\"\n  - sources: message:m-4\n  - supersedes: 22222222222222222222222222222222\n";
    fs::write(dir.join("daily/2026-11-03.md"), daily).expect("write the new daily log");
    let curated = "---\ntype: Curated Memory\ntitle: \"Preferences\"\ndescription: \"Preferences that currently hold.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-03T01:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\n---\n\n# Preferences\n";
    fs::write(dir.join("curated/preferences.md"), curated).expect("write the target");
    let outcome = promote_claim(
        &dir,
        "44444444444444444444444444444444",
        "curated/preferences.md",
        Some("the user switched to coffee"),
    )
    .expect("promote the coffee claim");
    assert_eq!(outcome, Promotion::Promoted);
    let text = fs::read_to_string(dir.join("curated/preferences.md")).expect("read target");
    let entry = "- 22222222222222222222222222222222 The user prefers tea over coffee. Replaced by 44444444444444444444444444444444 on 2026-11-03, the user switched to coffee.";
    assert!(text.contains(entry), "got\n{text}");
    assert!(text.contains("- [preference|high] The user prefers coffee over tea now."));
    regenerate_registry(&dir, "2026-11-03T00:00:00Z").expect("regenerate the registry");
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn promotion_requires_a_reason_target_and_claim_that_exist() {
    let dir = fixture_copy("promote-errors");
    let daily = "---\ntype: Daily Log\ntitle: \"Daily log 2026-11-03\"\ndescription: \"Extraction log for 2026-11-03 with claim blocks.\"\ngenerated: { by: \"process:memory-extraction\", at: \"2026-11-03T00:00:00Z\" }\n---\n\n## Verified extraction\n\n- [preference|high] The user prefers coffee over tea now.\n  - claim: 44444444444444444444444444444444\n  - quote: \"coffee these days\"\n  - sources: message:m-4\n  - supersedes: 22222222222222222222222222222222\n";
    fs::write(dir.join("daily/2026-11-03.md"), daily).expect("write the new daily log");
    assert!(
        promote_claim(
            &dir,
            "44444444444444444444444444444444",
            "curated/standing-facts.md",
            None,
        )
        .is_err(),
        "a superseding promotion without a reason is refused"
    );
    assert!(
        promote_claim(
            &dir,
            "99999999999999999999999999999999",
            "curated/standing-facts.md",
            None,
        )
        .is_err(),
        "an unknown claim is refused"
    );
    assert!(
        promote_claim(
            &dir,
            "22222222222222222222222222222222",
            "people/ada.md",
            None,
        )
        .is_err(),
        "promotion lands only in the curated layer"
    );
}
