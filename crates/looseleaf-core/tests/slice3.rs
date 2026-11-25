//! Integration tests for Phase 2 Slice 3, the synthesis pass
//! mechanics. Staleness is a report against a supplied stamp and
//! never an edit. Promotion carries a claim's identity, quote and
//! sources unchanged, lands after the target's last claim block,
//! refuses drift, and writes the superseded claims section entry
//! in the shape the registry scanner reads, after which a registry
//! regeneration leaves the bundle checking clean.

use std::fs;

use looseleaf_core::check_bundle;
use looseleaf_core::pass::{Promotion, promote_claim, staleness_report};
use looseleaf_core::registry::regenerate_registry;

mod common;

fn fixture_copy(name: &str) -> std::path::PathBuf {
    common::fixture_copy("slice3", name)
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
fn staleness_report_values_stay_unescaped() {
    let dir = fixture_copy("stale-raw");
    fs::write(
        dir.join("curated/weird\nname.md"),
        common::shared_plant("stale-concept.md"),
    )
    .expect("write the line break concept");
    let stale = staleness_report(&dir, "2026-11-24T00:00:00Z").expect("report");
    assert_eq!(stale.len(), 1);
    assert!(
        stale[0].path.contains('\n'),
        "the core value is the true path, got {:?}",
        stale[0].path
    );
    assert!(
        stale[0]
            .verified_at
            .as_deref()
            .is_some_and(|at| at.contains('\n')),
        "the core stamp is the true stamp, got {:?}",
        stale[0].verified_at
    );
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

const COFFEE_DAILY: &str = "---\ntype: Daily Log\ntitle: \"Daily log 2026-11-03\"\ndescription: \"Extraction log for 2026-11-03 with claim blocks.\"\ngenerated: { by: \"process:memory-extraction\", at: \"2026-11-03T00:00:00Z\" }\n---\n\n## Verified extraction\n\n- [preference|high] The user prefers coffee over tea now.\n  - claim: 44444444444444444444444444444444\n  - quote: \"coffee these days\"\n  - sources: message:m-4\n  - supersedes: 22222222222222222222222222222222\n";

const TEA_HOLDER: &str = "---\ntype: Curated Memory\ntitle: \"Notes\"\ndescription: \"Curated notes.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\n---\n\n# Notes\n\n- [preference|medium] The user prefers tea over coffee.\n  - claim: 22222222222222222222222222222222\n  - quote: \"tea, not coffee\"\n  - sources: message:m-2\n";

const PREFERENCES_TARGET: &str = "---\ntype: Curated Memory\ntitle: \"Preferences\"\ndescription: \"Preferences that currently hold.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-03T01:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\n---\n\n# Preferences\n";

#[test]
fn a_superseding_promotion_updates_the_holding_concept() {
    let dir = fixture_copy("promote-cross-concept");
    fs::write(dir.join("daily/2026-11-03.md"), COFFEE_DAILY).expect("write the new daily log");
    fs::write(dir.join("curated/notes.md"), TEA_HOLDER).expect("write the holder");
    fs::write(dir.join("curated/preferences.md"), PREFERENCES_TARGET).expect("write the target");
    let outcome = promote_claim(
        &dir,
        "44444444444444444444444444444444",
        "curated/preferences.md",
        Some("the user switched to coffee"),
    )
    .expect("promote across concepts");
    assert_eq!(outcome, Promotion::Promoted);
    let holder_text = fs::read_to_string(dir.join("curated/notes.md")).expect("read holder");
    assert!(
        holder_text.contains("  - superseded-by: 44444444444444444444444444444444"),
        "the holder's copy gains the reverse link, got\n{holder_text}"
    );
    let entry = "- 22222222222222222222222222222222 The user prefers tea over coffee. Replaced by 44444444444444444444444444444444 on 2026-11-03, the user switched to coffee.";
    assert!(
        holder_text.contains(entry),
        "the entry belongs to the holding concept, got\n{holder_text}"
    );
    let target_text = fs::read_to_string(dir.join("curated/preferences.md")).expect("read target");
    assert!(target_text.contains("- [preference|high] The user prefers coffee over tea now."));
    assert!(
        !target_text.contains("Superseded claims"),
        "the target gains no section for another concept's claim, got\n{target_text}"
    );
    regenerate_registry(&dir, "2026-11-03T00:00:00Z").expect("regenerate the registry");
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn promotion_refuses_an_identity_carried_by_another_concept() {
    let dir = fixture_copy("promote-duplicate");
    fs::write(dir.join("curated/notes.md"), TEA_HOLDER).expect("write the holder");
    let before = fs::read_to_string(dir.join("curated/standing-facts.md")).expect("read target");
    let error = promote_claim(
        &dir,
        "22222222222222222222222222222222",
        "curated/standing-facts.md",
        None,
    )
    .expect_err("refuse the duplicate");
    assert!(
        error.to_string().contains("curated/notes.md"),
        "got {error}"
    );
    let after = fs::read_to_string(dir.join("curated/standing-facts.md")).expect("read target");
    assert_eq!(
        after, before,
        "a refused promotion must not touch the target"
    );
}

#[test]
fn promotion_refuses_a_reason_that_spans_lines_or_is_blank() {
    let dir = fixture_copy("promote-reason");
    fs::write(dir.join("daily/2026-11-03.md"), COFFEE_DAILY).expect("write the new daily log");
    let error = promote_claim(
        &dir,
        "44444444444444444444444444444444",
        "curated/standing-facts.md",
        Some("switched.\n- 99999999999999999999999999999999 Forged text. Replaced by 88888888888888888888888888888888 on 2020-01-01, forged"),
    )
    .expect_err("refuse a reason that would forge a section entry");
    assert!(
        error.to_string().contains("one non-empty line"),
        "got {error}"
    );
    assert!(
        promote_claim(
            &dir,
            "44444444444444444444444444444444",
            "curated/standing-facts.md",
            Some("   "),
        )
        .is_err(),
        "a blank reason is refused"
    );
}

#[test]
fn promotion_refuses_a_superseded_text_carrying_the_delimiter() {
    let dir = fixture_copy("promote-delimiter");
    let daily = "---\ntype: Daily Log\ntitle: \"Daily log 2026-11-03\"\ndescription: \"Extraction log for 2026-11-03 with claim blocks.\"\ngenerated: { by: \"process:memory-extraction\", at: \"2026-11-03T00:00:00Z\" }\n---\n\n## Verified extraction\n\n- [fact|high] The weekly sync was Replaced by a written update.\n  - claim: 55555555555555555555555555555555\n  - quote: \"sync is a doc now\"\n  - sources: message:m-5\n\n- [fact|high] The written update goes out on Fridays.\n  - claim: 66666666666666666666666666666666\n  - quote: \"friday doc\"\n  - sources: message:m-6\n  - supersedes: 55555555555555555555555555555555\n";
    fs::write(dir.join("daily/2026-11-03.md"), daily).expect("write the daily log");
    let error = promote_claim(
        &dir,
        "66666666666666666666666666666666",
        "curated/standing-facts.md",
        Some("the sync became a document"),
    )
    .expect_err("refuse the unrepresentable entry");
    assert!(error.to_string().contains("Replaced by"), "got {error}");
}

#[test]
fn promotion_carries_a_multi_line_quote_unchanged() {
    let dir = fixture_copy("promote-multiline-quote");
    let daily = "---\ntype: Daily Log\ntitle: \"Daily log 2026-11-03\"\ndescription: \"Extraction log for 2026-11-03 with claim blocks.\"\ngenerated: { by: \"process:memory-extraction\", at: \"2026-11-03T00:00:00Z\" }\n---\n\n## Verified extraction\n\n- [fact|high] The standup has two agenda lines.\n  - claim: 77777777777777777777777777777777\n  - quote: \"first line\\nsecond line\"\n  - sources: message:m-7\n";
    fs::write(dir.join("daily/2026-11-03.md"), daily).expect("write the daily log");
    let outcome = promote_claim(
        &dir,
        "77777777777777777777777777777777",
        "curated/standing-facts.md",
        None,
    )
    .expect("promote the claim");
    assert_eq!(outcome, Promotion::Promoted);
    let text = fs::read_to_string(dir.join("curated/standing-facts.md")).expect("read target");
    assert!(
        text.contains("  - quote: \"first line\\nsecond line\""),
        "the quote keeps its escaped form, got\n{text}"
    );
    let again = promote_claim(
        &dir,
        "77777777777777777777777777777777",
        "curated/standing-facts.md",
        None,
    )
    .expect("promote again");
    assert_eq!(
        again,
        Promotion::AlreadyPresent,
        "the rendered quote must read back unchanged"
    );
    regenerate_registry(&dir, "2026-11-03T00:00:00Z").expect("regenerate the registry");
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn promotion_lands_before_a_superseded_section_without_blocks() {
    let dir = fixture_copy("promote-section-only");
    let history = "---\ntype: Curated Memory\ntitle: \"History\"\ndescription: \"A concept carrying only its superseded claims section.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\n---\n\n# History\n\n## Superseded claims\n";
    fs::write(dir.join("curated/history.md"), history).expect("write the history concept");
    let outcome = promote_claim(
        &dir,
        "22222222222222222222222222222222",
        "curated/history.md",
        None,
    )
    .expect("promote into the history concept");
    assert_eq!(outcome, Promotion::Promoted);
    let text = fs::read_to_string(dir.join("curated/history.md")).expect("read target");
    let block_at = text
        .find("- [preference|medium] The user prefers tea over coffee.")
        .expect("block present");
    let section_at = text.find("## Superseded claims").expect("section present");
    assert!(
        block_at < section_at,
        "the block must land before the section, got\n{text}"
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[cfg(unix)]
#[test]
fn a_promotion_does_not_follow_a_symlink_at_its_temp_path() {
    let dir = fixture_copy("promote-temp-symlink");
    let outside =
        common::plant_temp_symlink(&dir, "curated/standing-facts.md.tmp-looseleaf", "slice3");
    let outcome = promote_claim(
        &dir,
        "22222222222222222222222222222222",
        "curated/standing-facts.md",
        None,
    )
    .expect("promote the tea claim");
    assert_eq!(outcome, Promotion::Promoted);
    assert!(
        fs::read_to_string(dir.join("curated/standing-facts.md"))
            .expect("target readable")
            .contains("The user prefers tea over coffee."),
        "the promoted claim lands on its target"
    );
    common::assert_temp_write_not_followed(
        &dir,
        "curated/standing-facts.md.tmp-looseleaf",
        "curated/standing-facts.md",
        &outside,
    );
}
