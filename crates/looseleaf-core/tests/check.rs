//! Integration tests for the bundle checker, run against a clean
//! fixture bundle and copies of it with one planted defect each. A
//! test that cannot fail for the defect it plants is worthless, so
//! every defect test asserts the exact rule code it exists to catch.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use looseleaf_core::check_bundle;

mod common;

use common::clean_fixture;

fn defect_copy(name: &str) -> PathBuf {
    common::fixture_copy("check", name)
}

fn mutate(dir: &Path, rel: &str, edit: impl Fn(&str) -> String) {
    let path = dir.join(rel);
    let text = fs::read_to_string(&path).expect("read fixture file");
    fs::write(&path, edit(&text)).expect("write mutated fixture");
}

fn remove(dir: &Path, rel: &str) {
    fs::remove_file(dir.join(rel)).expect("remove fixture file");
}

fn write_file(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture dir");
    fs::write(&path, text).expect("write fixture file");
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
    assert_eq!(report.files_checked, 22);
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

#[test]
fn source_escaping_the_bundle_is_caught() {
    let dir = defect_copy("source-escape");
    mutate(&dir, "people/ada.md", |text| {
        text.replace("resource: daily/2026-11-01.md", "resource: /etc/passwd")
    });
    let found = codes(&dir);
    assert!(found.contains(&"SRC_UNRESOLVED"), "got {found:?}");
    let dir = defect_copy("source-climb");
    mutate(&dir, "people/ada.md", |text| {
        text.replace("resource: daily/2026-11-01.md", "resource: ../../MEMORY.md")
    });
    let found = codes(&dir);
    assert!(found.contains(&"SRC_UNRESOLVED"), "got {found:?}");
}

#[test]
fn source_entry_without_a_resource_is_caught() {
    let dir = defect_copy("source-no-resource");
    mutate(&dir, "people/ada.md", |text| {
        text.replace("    resource: daily/2026-11-01.md\n", "")
    });
    let found = codes(&dir);
    assert!(found.contains(&"SRC_UNRESOLVED"), "got {found:?}");
}

#[test]
fn missing_root_files_are_caught() {
    let dir = defect_copy("no-root-index");
    remove(&dir, "index.md");
    assert!(codes(&dir).contains(&"LAYOUT_ROOT_INDEX"));
    let dir = defect_copy("no-root-log");
    remove(&dir, "log.md");
    assert!(codes(&dir).contains(&"LAYOUT_ROOT_LOG"));
}

#[test]
fn missing_layer_files_are_caught() {
    let dir = defect_copy("no-layer-index");
    remove(&dir, "people/index.md");
    assert!(codes(&dir).contains(&"LAYOUT_DIR_INDEX"));
    let dir = defect_copy("no-layer-log");
    remove(&dir, "daily/log.md");
    assert!(codes(&dir).contains(&"LAYOUT_DIR_LOG"));
}

#[test]
fn unknown_directory_and_stray_root_file_are_caught() {
    let dir = defect_copy("unknown-dir");
    fs::create_dir(dir.join("notes")).expect("create stray dir");
    assert!(codes(&dir).contains(&"LAYOUT_UNKNOWN_DIR"));
    let dir = defect_copy("root-file");
    write_file(&dir, "stray.md", "# Stray\n");
    assert!(codes(&dir).contains(&"LAYOUT_ROOT_FILE"));
}

#[test]
fn root_index_fields_are_caught() {
    let dir = defect_copy("index-version");
    mutate(&dir, "index.md", |text| {
        text.replace("okf_version: \"0.3\"", "okf_version: \"0.2\"")
    });
    assert!(codes(&dir).contains(&"INDEX_VERSION"));
    let dir = defect_copy("index-field");
    mutate(&dir, "index.md", |text| {
        text.replace("license: MIT OR Apache-2.0\n", "")
    });
    assert!(codes(&dir).contains(&"INDEX_FIELD"));
}

#[test]
fn concept_frontmatter_defects_are_caught() {
    let dir = defect_copy("fm-parse");
    mutate(&dir, "people/ada.md", |text| {
        text.replace("type: Person", "type: [Person")
    });
    assert!(codes(&dir).contains(&"FM_PARSE"));
    let dir = defect_copy("fm-missing");
    write_file(&dir, "people/ada.md", "# Ada\n\nNo frontmatter.\n");
    assert!(codes(&dir).contains(&"FM_MISSING"));
    let dir = defect_copy("fm-type-mismatch");
    mutate(&dir, "people/ada.md", |text| {
        text.replace("type: Person", "type: Group")
    });
    assert!(codes(&dir).contains(&"FM_TYPE_MISMATCH"));
    let dir = defect_copy("fm-title");
    mutate(&dir, "people/ada.md", |text| {
        text.replace("title: \"Ada\"\n", "")
    });
    assert!(codes(&dir).contains(&"FM_TITLE"));
    let dir = defect_copy("fm-description");
    mutate(&dir, "curated/standing-facts.md", |text| {
        text.replace(
            "description: \"Facts that currently hold, promoted from the daily logs.\"\n",
            "",
        )
    });
    assert!(codes(&dir).contains(&"FM_DESCRIPTION"));
    let dir = defect_copy("fm-generated");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "generated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\n",
            "",
        )
    });
    assert!(codes(&dir).contains(&"FM_GENERATED"));
}

#[test]
fn extension_key_without_the_prefix_is_caught() {
    let dir = defect_copy("fm-extension");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "x-looseleaf-rank: 1\n",
            "x-looseleaf-rank: 1\nx-daybook-rank: 2\n",
        )
    });
    assert!(codes(&dir).contains(&"FM_EXTENSION_PREFIX"));
}

#[test]
fn verified_with_a_bad_shape_is_caught() {
    let dir = defect_copy("fm-verified");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "x-looseleaf-rank: 1\n",
            "x-looseleaf-rank: 1\nverified: yes\n",
        )
    });
    assert!(codes(&dir).contains(&"FM_VERIFIED"));
}

#[test]
fn concept_supersession_defects_are_caught() {
    let dir = defect_copy("fm-superseded-by");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "x-looseleaf-rank: 1\n",
            "x-looseleaf-rank: 1\nsuperseded_by: people/nobody.md\n",
        )
    });
    assert!(codes(&dir).contains(&"FM_SUPERSEDED_BY"));
    let dir = defect_copy("fm-status");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "x-looseleaf-rank: 1\n",
            "x-looseleaf-rank: 1\nstatus: deprecated\n",
        )
    });
    assert!(codes(&dir).contains(&"FM_STATUS"));
}

#[test]
fn missing_sources_are_caught() {
    let dir = defect_copy("src-required");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "sources:\n  - id: day-one\n    resource: daily/2026-11-01.md\n    title: Daily log for 2026-11-01\n",
            "",
        )
    });
    assert!(codes(&dir).contains(&"SRC_REQUIRED"));
}

#[test]
fn claim_shape_defects_are_caught() {
    let dir = defect_copy("claim-no-id");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        format!(
            "{text}\n- [fact|high] A claim with no identity.\n  - quote: \"no id\"\n  - sources: message:m-9\n"
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_NO_ID"));
    let dir = defect_copy("claim-no-quote");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        format!(
            "{text}\n- [fact|high] A claim with no quote.\n  - claim: 44444444444444444444444444444444\n  - sources: message:m-9\n"
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_NO_QUOTE"));
    let dir = defect_copy("claim-kind");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        text.replace("- [fact|high]", "- [rumour|high]")
    });
    assert!(codes(&dir).contains(&"CLAIM_KIND"));
    let dir = defect_copy("claim-confidence");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        text.replace("- [fact|high]", "- [fact|certain]")
    });
    assert!(codes(&dir).contains(&"CLAIM_CONFIDENCE"));
    let dir = defect_copy("claim-dup-subitem");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        format!(
            "{text}\n- [fact|high] A block with two identities.\n  - claim: 44444444444444444444444444444444\n  - claim: 55555555555555555555555555555555\n  - quote: \"twice\"\n  - sources: message:m-9\n"
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_DUP_SUBITEM"));
}

#[test]
fn reverse_link_defects_are_caught() {
    let dir = defect_copy("reverse-in-daily");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        text.replace(
            "  - sources: message:m-1\n",
            "  - sources: message:m-1\n  - superseded-by: 33333333333333333333333333333333\n",
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_REVERSE_IN_DAILY"));
    let dir = defect_copy("reverse-dangling");
    mutate(&dir, "curated/standing-facts.md", |text| {
        text.replace(
            "  - supersedes: 11111111111111111111111111111111\n",
            "  - supersedes: 11111111111111111111111111111111\n  - superseded-by: 22222222222222222222222222222222\n",
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_REVERSE_DANGLING"));
}

#[test]
fn supersession_chain_defects_are_caught() {
    let dir = defect_copy("supersede-self");
    mutate(&dir, "daily/2026-11-02.md", |text| {
        text.replace(
            "supersedes: 11111111111111111111111111111111",
            "supersedes: 33333333333333333333333333333333",
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_SUPERSEDES_SELF"));
    let dir = defect_copy("supersede-cycle");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        text.replace(
            "  - sources: message:m-1\n",
            "  - sources: message:m-1\n  - supersedes: 33333333333333333333333333333333\n",
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_SUPERSEDE_CYCLE"));
    let dir = defect_copy("supersede-fork");
    mutate(&dir, "daily/2026-11-02.md", |text| {
        format!(
            "{text}\n- [update|high] Another replacement.\n  - claim: 44444444444444444444444444444444\n  - quote: \"standup moves again\"\n  - sources: message:m-4\n  - supersedes: 11111111111111111111111111111111\n"
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_SUPERSEDE_FORK"));
}

#[test]
fn promotion_drift_is_caught() {
    let dir = defect_copy("promotion-drift");
    mutate(&dir, "curated/standing-facts.md", |text| {
        text.replace(
            "quote: \"standup moves to 10:00\"",
            "quote: \"standup moves to 10:30\"",
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_PROMOTION_DRIFT"));
}

#[test]
fn missing_superseded_section_is_caught() {
    let dir = defect_copy("section-missing");
    mutate(&dir, "curated/standing-facts.md", |text| {
        let without_section = text.replace(
            "## Superseded claims\n\n- 11111111111111111111111111111111 The team standup is at 09:30. Replaced by 33333333333333333333333333333333 on 2026-11-02, the standup time changed.\n",
            "",
        );
        format!(
            "{without_section}\n- [fact|high] The team standup is at 09:30.\n  - claim: 11111111111111111111111111111111\n  - quote: \"standup is at 09:30 daily\"\n  - sources: message:m-1\n"
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_SECTION_MISSING"));
}

#[test]
fn claim_source_defects_are_caught() {
    let dir = defect_copy("claim-src");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        format!(
            "{text}\n- [fact|high] A claim citing a missing file.\n  - claim: 44444444444444444444444444444444\n  - quote: \"cited\"\n  - sources: daily/2026-12-25.md#claim:44444444444444444444444444444444\n"
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_SRC_UNRESOLVED"));
    let dir = defect_copy("claim-src-line");
    mutate(&dir, "daily/2026-11-01.md", |text| {
        format!(
            "{text}\n- [fact|high] A claim citing a line number.\n  - claim: 44444444444444444444444444444444\n  - quote: \"cited\"\n  - sources: daily/2026-11-01.md#L12\n"
        )
    });
    assert!(codes(&dir).contains(&"CLAIM_SRC_UNRESOLVED"));
}

#[test]
fn registry_defects_are_caught() {
    let dir = defect_copy("registry-phantom");
    mutate(&dir, "claims.md", |text| {
        format!("{text}- 99999999999999999999999999999999 current.\n")
    });
    assert!(codes(&dir).contains(&"REGISTRY_STALE"));
    let dir = defect_copy("registry-withheld-missing");
    mutate(&dir, "claims.md", |text| {
        text.replace(
            "- 11111111111111111111111111111111 withheld, superseded by 33333333333333333333333333333333 on 2026-11-02. Reason, the standup time changed.\n",
            "",
        )
    });
    assert!(codes(&dir).contains(&"REGISTRY_WITHHELD"));
    let dir = defect_copy("registry-withheld-reason");
    mutate(&dir, "claims.md", |text| {
        text.replace(
            "Reason, the standup time changed.",
            "Reason, a different reason.",
        )
    });
    assert!(codes(&dir).contains(&"REGISTRY_WITHHELD"));
}

#[test]
fn bank_shape_is_caught() {
    let dir = defect_copy("bank-shape");
    write_file(&dir, "bank/index.md", "# Memory bank\n");
    let mut body = String::from(
        "---\ntype: Memory Bank\ntitle: \"Bank\"\ndescription: \"A bank concept.\"\ngenerated: { by: \"process:test\", at: \"2026-11-02T00:00:00Z\" }\nsources:\n  - id: root-index\n    resource: index.md\n---\n\n",
    );
    for line in 0..251 {
        writeln!(body, "bank line {line}").expect("write to a string");
    }
    write_file(&dir, "bank/bank.md", &body);
    assert!(codes(&dir).contains(&"BANK_SHAPE"));
}

#[test]
fn daily_name_must_be_a_calendar_date() {
    let dir = defect_copy("daily-name");
    let text = fs::read_to_string(dir.join("daily/2026-11-01.md")).expect("read daily file");
    write_file(&dir, "daily/2026-13-45.md", &text);
    remove(&dir, "daily/2026-11-01.md");
    assert!(codes(&dir).contains(&"DAILY_NAME"));
}

#[test]
fn person_sections_are_caught() {
    let dir = defect_copy("person-sections");
    mutate(&dir, "people/ada.md", |text| {
        text.replace("## Strengthening\n\n- Keep the standup time current.\n", "")
    });
    assert!(codes(&dir).contains(&"PERSON_SECTIONS"));
}

#[test]
fn log_order_is_caught() {
    let dir = defect_copy("log-order");
    mutate(&dir, "log.md", |text| {
        format!("{text}\n## 2026-11-01\n\n- Earlier.\n\n## 2026-11-03\n\n- Later.\n")
    });
    assert!(codes(&dir).contains(&"LOG_ORDER"));
}

#[test]
fn unreadable_file_is_caught() {
    let dir = defect_copy("read-error");
    fs::write(dir.join("people/broken.md"), [0xff, 0xfe, 0xfd]).expect("write broken file");
    assert!(codes(&dir).contains(&"READ_ERROR"));
}

#[cfg(unix)]
#[test]
fn symbolic_links_are_reported_not_followed() {
    let dir = defect_copy("link");
    std::os::unix::fs::symlink(dir.join("people/ada.md"), dir.join("people/linked.md"))
        .expect("create symlink");
    let found = codes(&dir);
    assert!(found.contains(&"LINK_SKIPPED"), "got {found:?}");
}

#[test]
fn a_malformed_stale_after_draws_a_warning() {
    let dir = defect_copy("stale-after-shape");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "x-looseleaf-rank: 1",
            "stale_after: soon\nx-looseleaf-rank: 1",
        )
    });
    assert!(
        codes(&dir).contains(&"FM_STALE_AFTER"),
        "a stale_after with no real date must be flagged"
    );
    let dir = defect_copy("stale-after-valid");
    mutate(&dir, "people/ada.md", |text| {
        text.replace(
            "x-looseleaf-rank: 1",
            "stale_after: 2026-11-01\nx-looseleaf-rank: 1",
        )
    });
    assert!(
        !codes(&dir).contains(&"FM_STALE_AFTER"),
        "a dated stale_after is the shape the profile states"
    );
}

fn deprecate_toward(dir: &Path, rel: &str, target: &str) {
    mutate(dir, rel, |text| {
        text.replace(
            "at: \"2026-11-02T01:00:00Z\" }",
            &format!(
                "at: \"2026-11-02T01:00:00Z\" }}\nstatus: deprecated\nsuperseded_by: {target}"
            ),
        )
    });
}

fn cycle_paths(dir: &Path) -> Vec<String> {
    let mut paths: Vec<String> = check_bundle(dir)
        .findings
        .iter()
        .filter(|finding| finding.code == "FM_SUPERSEDE_CYCLE")
        .map(|finding| finding.path.clone())
        .collect();
    paths.sort();
    paths
}

fn write_third_concept(dir: &Path, target: &str) {
    write_file(
        dir,
        "topics/third.md",
        &format!(
            "---\ntype: Topic\ntitle: \"Third\"\ndescription: \"A third concept planted by the cycle tests.\"\ngenerated: {{ by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }}\nstatus: deprecated\nsuperseded_by: {target}\nsources:\n  - id: day-one\n    resource: daily/2026-11-01.md\n    title: Daily log for 2026-11-01\n---\n\n# Third\n"
        ),
    );
    write_file(
        dir,
        "topics/index.md",
        "# Topics\n\n* [Standup notes](standup-notes.md)\n* [Third](third.md)\n",
    );
}

#[test]
fn a_two_concept_cycle_is_caught() {
    let dir = defect_copy("concept-cycle-two");
    deprecate_toward(&dir, "people/ada.md", "curated/standing-facts.md");
    deprecate_toward(&dir, "curated/standing-facts.md", "people/ada.md");
    assert_eq!(
        cycle_paths(&dir),
        ["curated/standing-facts.md", "people/ada.md"],
        "one finding per concept on the cycle"
    );
}

#[test]
fn a_three_concept_cycle_is_caught() {
    let dir = defect_copy("concept-cycle-three");
    deprecate_toward(&dir, "people/ada.md", "topics/third.md");
    deprecate_toward(&dir, "curated/standing-facts.md", "people/ada.md");
    write_third_concept(&dir, "curated/standing-facts.md");
    assert_eq!(
        cycle_paths(&dir),
        [
            "curated/standing-facts.md",
            "people/ada.md",
            "topics/third.md"
        ]
    );
}

#[test]
fn a_self_pointer_is_a_cycle_of_one() {
    let dir = defect_copy("concept-cycle-self");
    deprecate_toward(&dir, "people/ada.md", "people/ada.md");
    assert_eq!(cycle_paths(&dir), ["people/ada.md"]);
}

#[test]
fn a_chain_leading_into_a_cycle_does_not_fire_for_the_tail() {
    let dir = defect_copy("concept-cycle-tail");
    deprecate_toward(&dir, "people/ada.md", "curated/standing-facts.md");
    deprecate_toward(&dir, "curated/standing-facts.md", "people/ada.md");
    write_third_concept(&dir, "people/ada.md");
    assert_eq!(
        cycle_paths(&dir),
        ["curated/standing-facts.md", "people/ada.md"],
        "the tail concept is deprecated in good faith and does not fire"
    );
}

#[test]
fn a_converging_chain_is_not_a_cycle() {
    let dir = defect_copy("concept-converge");
    deprecate_toward(&dir, "people/ada.md", "curated/standing-facts.md");
    write_third_concept(&dir, "curated/standing-facts.md");
    assert!(
        cycle_paths(&dir).is_empty(),
        "two concepts replaced by one live successor is ordinary"
    );
}

#[test]
fn a_finding_on_a_line_break_concept_stays_raw_in_the_report() {
    // The CLI printer escapes finding paths and messages for
    // its lines; the core report's own values are never
    // escaped, so a consumer of the report sees the true path
    // and the true message, line breaks included.
    let dir = defect_copy("raw-finding");
    write_file(
        &dir,
        "curated/weird\nname.md",
        "---\ntype: Curated Memory\ntitle: \"Weird name\"\ndescription: \"A concept whose file name and replacement both carry line breaks.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nsources:\n  - id: day-one\n    resource: daily/2026-11-01.md\n    title: Daily log for 2026-11-01\nsuperseded_by: \"curated/no\\nsuch.md\"\n---\n\n# Weird name\n\nBody text.\n",
    );
    let report = check_bundle(&dir);
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "FM_SUPERSEDED_BY")
        .expect("the plant draws its superseded_by finding");
    assert!(
        finding.path.contains('\n'),
        "the core path stays raw, got {:?}",
        finding.path
    );
    assert!(
        finding.message.contains('\n'),
        "the core message stays raw, got {:?}",
        finding.message
    );
}
