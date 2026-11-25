//! Integration tests for the Synthesis report writer. The created
//! report is pinned byte for byte, appends are pinned by their
//! landing section, and every refusal is asserted to write nothing
//! beyond what the test itself wrote.

mod common;

use std::fs;

use looseleaf_core::check::check_bundle;
use looseleaf_core::report::{ReportSection, append_report_entry};

fn fixture_copy(name: &str) -> std::path::PathBuf {
    common::fixture_copy("report", name)
}

const AT: &str = "2026-11-24T19:06:02Z";
const REL: &str = "synthesis/2026-11-24-190602.md";

#[test]
fn a_first_entry_creates_the_report_and_its_layer_index() {
    let dir = fixture_copy("create");
    let rel = append_report_entry(
        &dir,
        AT,
        ReportSection::Promotions,
        "Promoted claim 22222222222222222222222222222222 into curated/standing-facts.md",
    )
    .expect("write the first entry");
    assert_eq!(rel, REL);
    let text = fs::read_to_string(dir.join(REL)).expect("read the report");
    assert_eq!(
        text,
        "---\ntype: Synthesis\ntitle: \"Synthesis report 2026-11-24\"\ndescription: \"The synthesis pass report for 2026-11-24, written by the pass under its contract.\"\ngenerated:\n  by: process:looseleaf-synthesis\n  at: 2026-11-24T19:06:02Z\n---\n\n# Synthesis report 2026-11-24\n\n## Promotions\n- Promoted claim 22222222222222222222222222222222 into curated/standing-facts.md\n\n## Supersessions\n\n## Regenerated\n\n## Flags\n\n## Blocked\n\n## Re-verification queue\n"
    );
    let index = fs::read_to_string(dir.join("synthesis/index.md")).expect("read the index");
    assert_eq!(
        index,
        "# Synthesis\n\n* [Field notes](field-notes.md)\n* [Synthesis report 2026-11-24](2026-11-24-190602.md)\n"
    );
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn entries_land_in_their_own_sections_in_order() {
    let dir = fixture_copy("sections");
    append_report_entry(
        &dir,
        AT,
        ReportSection::Flags,
        "Flagged bank/reflections.md for shape",
    )
    .expect("flag entry");
    append_report_entry(
        &dir,
        AT,
        ReportSection::Promotions,
        "Promoted the first claim",
    )
    .expect("first promotion");
    append_report_entry(
        &dir,
        AT,
        ReportSection::Promotions,
        "Promoted the second claim",
    )
    .expect("second promotion");
    append_report_entry(
        &dir,
        AT,
        ReportSection::Queue,
        "queue curated/standing-facts.md changed in this pass",
    )
    .expect("queue entry");
    let text = fs::read_to_string(dir.join(REL)).expect("read the report");
    let promotions = text.find("## Promotions").expect("promotions heading");
    let first = text
        .find("- Promoted the first claim")
        .expect("first entry");
    let second = text
        .find("- Promoted the second claim")
        .expect("second entry");
    let flags = text.find("## Flags").expect("flags heading");
    let flagged = text
        .find("- Flagged bank/reflections.md for shape")
        .expect("flag entry");
    let queue = text
        .find("## Re-verification queue")
        .expect("queue heading");
    let queued = text
        .find("- queue curated/standing-facts.md changed in this pass")
        .expect("queue entry");
    assert!(promotions < first && first < second && second < flags);
    assert!(flags < flagged && flagged < queue);
    assert!(queue < queued);
}

#[test]
fn two_runs_on_one_day_write_two_reports() {
    let dir = fixture_copy("two-runs");
    let first =
        append_report_entry(&dir, AT, ReportSection::Flags, "First run flag").expect("first run");
    let second = append_report_entry(
        &dir,
        "2026-11-24T21:30:00Z",
        ReportSection::Flags,
        "Second run flag",
    )
    .expect("second run");
    assert_eq!(first, REL);
    assert_eq!(second, "synthesis/2026-11-24-213000.md");
    let first_text = fs::read_to_string(dir.join(first)).expect("read the first");
    let second_text = fs::read_to_string(dir.join(second)).expect("read the second");
    assert!(first_text.contains("- First run flag"));
    assert!(!first_text.contains("Second run flag"));
    assert!(second_text.contains("- Second run flag"));
}

#[test]
fn write_report_refuses_bad_stamps_entries_and_shapes() {
    let dir = fixture_copy("refusals");
    fs::remove_dir_all(dir.join("synthesis")).expect("clear the standing synthesis layer");
    let error = append_report_entry(&dir, "2026-11-24", ReportSection::Flags, "An entry")
        .expect_err("a stamp with no time must refuse");
    assert!(
        error.to_string().contains("real calendar date and time"),
        "got {error}"
    );
    let error = append_report_entry(
        &dir,
        "2026-11-24T25:00:00Z",
        ReportSection::Flags,
        "An entry",
    )
    .expect_err("an impossible time must refuse");
    assert!(
        error.to_string().contains("real calendar date and time"),
        "got {error}"
    );
    let error = append_report_entry(&dir, AT, ReportSection::Flags, "  ")
        .expect_err("a blank entry must refuse");
    assert!(
        error.to_string().contains("one non-empty line"),
        "got {error}"
    );
    let error = append_report_entry(&dir, AT, ReportSection::Flags, "two\nlines")
        .expect_err("a two line entry must refuse");
    assert!(
        error.to_string().contains("one non-empty line"),
        "got {error}"
    );
    assert!(
        !dir.join("synthesis").exists(),
        "refusals before creation write nothing"
    );
    assert_eq!(ReportSection::from_token("minutes"), None);
    assert_eq!(
        ReportSection::from_token("queue"),
        Some(ReportSection::Queue)
    );
}

#[test]
fn write_report_refuses_a_foreign_concept_and_a_broken_report() {
    let dir = fixture_copy("foreign");
    fs::create_dir_all(dir.join("synthesis")).expect("make the layer");
    let foreign = "---\ntype: Topic\ntitle: \"Not a report\"\ndescription: \"A topic concept at the report path.\"\ngenerated: { by: \"human:david\", at: \"2026-11-02T00:00:00Z\" }\n---\n\n# Not a report\n";
    fs::write(dir.join(REL), foreign).expect("write the foreign concept");
    let error = append_report_entry(&dir, AT, ReportSection::Flags, "An entry")
        .expect_err("a foreign concept must refuse");
    assert!(error.to_string().contains("type Synthesis"), "got {error}");
    assert_eq!(
        fs::read_to_string(dir.join(REL)).expect("read it back"),
        foreign,
        "a refusal writes nothing"
    );

    let dir = fixture_copy("broken");
    append_report_entry(&dir, AT, ReportSection::Flags, "A flag").expect("create the report");
    let path = dir.join(REL);
    let text = fs::read_to_string(&path).expect("read the report");
    let broken = text.replace("## Blocked\n", "");
    assert_ne!(broken, text);
    fs::write(&path, &broken).expect("break the report");
    let error = append_report_entry(&dir, AT, ReportSection::Blocked, "A blocked promotion")
        .expect_err("a report missing the section must refuse");
    assert!(
        error.to_string().contains("missing its Blocked section"),
        "got {error}"
    );
}

#[test]
fn the_checker_warns_on_a_report_missing_a_section() {
    let dir = fixture_copy("shape");
    fs::create_dir_all(dir.join("synthesis")).expect("make the layer");
    fs::write(
        dir.join("synthesis/index.md"),
        "# Synthesis\n\n* [Synthesis report 2026-11-24](2026-11-24-190602.md)\n",
    )
    .expect("write the index");
    fs::write(
        dir.join(REL),
        "---\ntype: Synthesis\ntitle: \"Synthesis report 2026-11-24\"\ndescription: \"The synthesis pass report for 2026-11-24, written by the pass under its contract.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-24T19:06:02Z\" }\n---\n\n# Synthesis report 2026-11-24\n\n## Promotions\n\n## Supersessions\n\n## Regenerated\n\n## Flags\n\n## Re-verification queue\n",
    )
    .expect("write the broken report");
    let report = check_bundle(&dir);
    let codes: Vec<&str> = report.findings.iter().map(|finding| finding.code).collect();
    assert!(codes.contains(&"REPORT_SHAPE"), "got {codes:?}");
    fs::write(
        dir.join("synthesis/notes.md"),
        "---\ntype: Synthesis\ntitle: \"Notes\"\ndescription: \"A synthesis concept that is not a run report.\"\ngenerated: { by: \"human:david\", at: \"2026-11-02T00:00:00Z\" }\n---\n\n# Notes\n",
    )
    .expect("write a non-report concept");
    let report = check_bundle(&dir);
    let shape_paths: Vec<&str> = report
        .findings
        .iter()
        .filter(|finding| finding.code == "REPORT_SHAPE")
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        shape_paths,
        vec![REL],
        "only the report-named file is shape checked"
    );
}

#[test]
fn an_unstamped_report_is_healed_before_the_append() {
    let dir = fixture_copy("healing");
    fs::create_dir_all(dir.join("synthesis")).expect("make the layer");
    fs::write(
        dir.join("synthesis/index.md"),
        "# Synthesis\n\n* [Synthesis report 2026-11-24](2026-11-24-190602.md)\n",
    )
    .expect("write the index");
    fs::write(
        dir.join(REL),
        "---\ntype: Synthesis\ntitle: \"Synthesis report 2026-11-24\"\ndescription: \"The synthesis pass report for 2026-11-24, written by the pass under its contract.\"\n---\n\n# Synthesis report 2026-11-24\n\n## Promotions\n\n## Supersessions\n\n## Regenerated\n\n## Flags\n\n## Blocked\n\n## Re-verification queue\n",
    )
    .expect("write an unstamped report, as an interrupted creation leaves");
    append_report_entry(&dir, AT, ReportSection::Flags, "A flag").expect("append the entry");
    let text = fs::read_to_string(dir.join(REL)).expect("read the report");
    assert!(
        text.contains("generated:"),
        "the append heals the missing stamp, got:\n{text}"
    );
    assert!(text.contains("- A flag"));
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn a_fenced_heading_is_content_not_a_section() {
    let dir = fixture_copy("fence");
    append_report_entry(&dir, AT, ReportSection::Flags, "A flag").expect("create the report");
    let path = dir.join(REL);
    let text = fs::read_to_string(&path).expect("read the report");
    let fenced = text.replace(
        "## Promotions\n",
        "```\n## Promotions\n```\n\n## Promotions\n",
    );
    assert_ne!(fenced, text);
    fs::write(&path, &fenced).expect("plant the fenced heading");
    append_report_entry(&dir, AT, ReportSection::Promotions, "A real promotion")
        .expect("append the entry");
    let text = fs::read_to_string(&path).expect("read the report");
    let fence_end = text.find("```\n\n## Promotions").expect("the fenced block");
    let entry = text.find("- A real promotion").expect("the entry");
    assert!(entry > fence_end, "the entry lands in the real section");
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}

#[test]
fn the_checker_warns_on_out_of_order_sections_and_an_uppercase_extension() {
    let dir = fixture_copy("order");
    fs::create_dir_all(dir.join("synthesis")).expect("make the layer");
    fs::write(dir.join("synthesis/index.md"), "# Synthesis\n").expect("write the index");
    let body = "---\ntype: Synthesis\ntitle: \"Synthesis report 2026-11-24\"\ndescription: \"The synthesis pass report for 2026-11-24, written by the pass under its contract.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-24T19:06:02Z\" }\n---\n\n# Synthesis report 2026-11-24\n\n## Flags\n\n## Promotions\n\n## Supersessions\n\n## Regenerated\n\n## Blocked\n\n## Re-verification queue\n";
    fs::write(dir.join("synthesis/2026-11-24-190602.MD"), body).expect("write the report");
    let report = check_bundle(&dir);
    let shape: Vec<&str> = report
        .findings
        .iter()
        .filter(|finding| finding.code == "REPORT_SHAPE")
        .map(|finding| finding.message.as_str())
        .collect();
    assert_eq!(shape.len(), 1, "got {:?}", report.findings);
    assert!(shape[0].contains("out of contract order"), "got {shape:?}");
}

#[cfg(unix)]
#[test]
fn write_report_refuses_a_symlinked_synthesis_directory() {
    let dir = fixture_copy("symlink");
    fs::remove_dir_all(dir.join("synthesis")).expect("clear the standing synthesis layer");
    let outside = dir.parent().expect("a parent").join("report-outside");
    fs::create_dir_all(&outside).expect("make the outside directory");
    std::os::unix::fs::symlink(&outside, dir.join("synthesis")).expect("link the layer");
    let error = append_report_entry(&dir, AT, ReportSection::Flags, "An entry")
        .expect_err("a symlinked layer must refuse");
    assert!(error.to_string().contains("symbolic link"), "got {error}");
    assert!(
        !outside.join("2026-11-24-190602.md").exists(),
        "nothing lands outside the bundle"
    );
}

#[test]
fn an_existing_layer_without_an_index_gains_one() {
    let dir = fixture_copy("index-upkeep");
    fs::remove_file(dir.join("synthesis/index.md")).expect("drop the standing index");
    append_report_entry(&dir, AT, ReportSection::Flags, "A flag").expect("write the entry");
    let index = fs::read_to_string(dir.join("synthesis/index.md")).expect("read the index");
    assert!(index.contains("(2026-11-24-190602.md)"), "got {index}");
    let report = check_bundle(&dir);
    assert_eq!(report.findings.len(), 0, "got {:?}", report.findings);
}
