//! End-to-end tests for the Phase 3 command line surface. The
//! core library is tested in its own crate; these pin the stdout
//! lines and usage discipline the phase scope fixed for the four
//! new subcommands, which no core test can reach.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture_copy(name: &str) -> PathBuf {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../looseleaf-core/tests/fixtures/clean");
    let dir =
        std::env::temp_dir().join(format!("looseleaf-cli-test-{}-{name}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear the old copy");
    }
    copy_dir(&source, &dir);
    dir
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create the target");
    for entry in fs::read_dir(source).expect("read the fixture") {
        let entry = entry.expect("a fixture entry");
        let from = entry.path();
        let to = target.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            fs::copy(&from, &to).expect("copy a fixture file");
        }
    }
}

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(dir: &Path, args: &[&str]) -> Outcome {
    let output = Command::new(env!("CARGO_BIN_EXE_looseleaf"))
        .arg(args[0])
        .arg(dir)
        .args(&args[1..])
        .output()
        .expect("run the binary");
    Outcome {
        code: output.status.code().expect("an exit code"),
        stdout: String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    }
}

#[test]
fn the_stamp_commands_print_their_pinned_lines() {
    let dir = fixture_copy("stamps");
    let outcome = run(
        &dir,
        &[
            "stamp-generated",
            "--path",
            "people/ada.md",
            "--at",
            "2026-11-24T00:00:00Z",
        ],
    );
    assert_eq!(outcome.code, 0, "stderr {}", outcome.stderr);
    assert_eq!(outcome.stdout, "generated stamped, people/ada.md\n");
    let outcome = run(
        &dir,
        &[
            "stamp-generated",
            "--path",
            "people/ada.md",
            "--at",
            "2026-11-24T00:00:00Z",
        ],
    );
    assert_eq!(outcome.stdout, "generated already current, people/ada.md\n");
    let outcome = run(
        &dir,
        &[
            "stamp-verified",
            "--path",
            "people/ada.md",
            "--by",
            "checker",
            "--at",
            "2026-11-24T09:00:00Z",
        ],
    );
    assert_eq!(outcome.stdout, "verification stamped, people/ada.md\n");
    let outcome = run(
        &dir,
        &[
            "stamp-verified",
            "--path",
            "people/ada.md",
            "--by",
            "checker",
            "--at",
            "2026-11-24T09:00:00Z",
        ],
    );
    assert_eq!(
        outcome.stdout,
        "verification already recorded, people/ada.md\n"
    );
}

#[test]
fn write_report_and_queue_print_their_pinned_lines() {
    let dir = fixture_copy("report-queue");
    let outcome = run(
        &dir,
        &[
            "write-report",
            "--at",
            "2026-11-24T19:06:02Z",
            "--section",
            "queue",
            "--entry",
            "queue people/ada.md changed in this pass",
        ],
    );
    assert_eq!(outcome.code, 0, "stderr {}", outcome.stderr);
    assert_eq!(
        outcome.stdout,
        "report entry written, synthesis/2026-11-24-190602.md\n"
    );
    let outcome = run(
        &dir,
        &[
            "queue",
            "--at",
            "2026-11-24T19:06:02Z",
            "--changed",
            "people/ada.md",
            "--changed",
            "curated/standing-facts.md",
        ],
    );
    assert_eq!(outcome.code, 0, "stderr {}", outcome.stderr);
    assert_eq!(
        outcome.stdout,
        "queue people/ada.md changed in this pass\nqueue curated/standing-facts.md changed in this pass\n2 concepts in the re-verification queue at 2026-11-24T19:06:02Z\n"
    );
}

#[test]
fn deprecate_prints_its_pinned_lines() {
    let dir = fixture_copy("deprecate");
    let outcome = run(
        &dir,
        &[
            "deprecate",
            "--path",
            "people/ada.md",
            "--superseded-by",
            "curated/standing-facts.md",
            "--at",
            "2026-11-24T22:00:00Z",
        ],
    );
    assert_eq!(outcome.code, 0, "stderr {}", outcome.stderr);
    assert_eq!(outcome.stdout, "concept deprecated, people/ada.md\n");
    let outcome = run(
        &dir,
        &[
            "deprecate",
            "--path",
            "people/ada.md",
            "--superseded-by",
            "curated/standing-facts.md",
            "--at",
            "2026-11-24T22:00:00Z",
        ],
    );
    assert_eq!(
        outcome.stdout,
        "concept already deprecated, people/ada.md\n"
    );
}

#[test]
fn staleness_escapes_line_breaks_in_its_printed_fields() {
    let dir = fixture_copy("staleness-escape");
    std::fs::write(
        dir.join("curated/weird\nname.md"),
        "---\ntype: Curated Memory\ntitle: \"Weird\"\ndescription: \"A concept whose file name carries a line break.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nstale_after: 2026-11-01\nverified: { by: \"process:looseleaf-synthesis\", at: \"2026-11-20T00:00:00Z\\nforged\" }\n---\n\n# Weird\n",
    )
    .expect("write the line break concept");
    let outcome = run(&dir, &["staleness", "--at", "2026-11-24T00:00:00Z"]);
    assert_eq!(outcome.code, 0, "stderr {}", outcome.stderr);
    assert!(
        outcome.stdout.contains(
            "stale curated/weird\\nname.md stale_after 2026-11-01 verified 2026-11-20T00:00:00Z\\nforged"
        ),
        "both fields print escaped on one line, got {}",
        outcome.stdout
    );
    assert_eq!(
        outcome.stdout.lines().count(),
        2,
        "one stale line and the closing count, nothing forged, got {}",
        outcome.stdout
    );
}

#[test]
fn closing_lines_escape_the_echoed_stamp() {
    let dir = fixture_copy("closing-escape");
    let outcome = run(
        &dir,
        &["staleness", "--at", "2026-11-24\n0 stale concepts at 1970-01-01"],
    );
    assert_eq!(outcome.code, 0, "stderr {}", outcome.stderr);
    assert_eq!(
        outcome.stdout,
        "0 stale concepts at 2026-11-24\\n0 stale concepts at 1970-01-01\n",
        "the echoed stamp cannot forge a second summary line"
    );
    let outcome = run(&dir, &["queue", "--at", "2026-11-24\nforged"]);
    assert_eq!(outcome.code, 0, "stderr {}", outcome.stderr);
    assert_eq!(
        outcome.stdout,
        "0 concepts in the re-verification queue at 2026-11-24\\nforged\n",
        "the queue closing line carries the same discipline"
    );
}

#[test]
fn usage_errors_exit_two() {
    let dir = fixture_copy("usage-deprecate");
    let outcome = run(&dir, &["deprecate", "--path", "people/ada.md"]);
    assert_eq!(outcome.code, 2, "stderr {}", outcome.stderr);
    let dir = fixture_copy("usage");
    let outcome = run(
        &dir,
        &["queue", "--at", "2026-11-24T19:06:02Z", "--changed"],
    );
    assert_eq!(outcome.code, 2, "a dangling --changed is a usage error");
    let outcome = run(
        &dir,
        &[
            "write-report",
            "--at",
            "2026-11-24T19:06:02Z",
            "--section",
            "minutes",
            "--entry",
            "An entry",
        ],
    );
    assert_eq!(outcome.code, 2, "an unknown section token is a usage error");
    let outcome = run(&dir, &["stamp-generated", "--path", "people/ada.md"]);
    assert_eq!(outcome.code, 2, "a short argument list is a usage error");
}
