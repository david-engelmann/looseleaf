//! Integration tests for the re-verification queue. Membership is
//! the contract's created or changed set, validated path by path,
//! and the queue is a list that never writes.

mod common;

use std::fs;
use std::path::Path;

use looseleaf_core::pass::reverification_queue;

fn fixture_copy(name: &str) -> std::path::PathBuf {
    common::fixture_copy("queue", name)
}

const AT: &str = "2026-11-24T19:06:02Z";

fn paths(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn the_queue_carries_the_changed_set_in_order_without_duplicates() {
    let dir = fixture_copy("membership");
    let queue = reverification_queue(
        &dir,
        AT,
        &paths(&[
            "curated/standing-facts.md",
            "people/ada.md",
            "curated/standing-facts.md",
        ]),
    )
    .expect("assemble the queue");
    assert_eq!(
        queue,
        paths(&["curated/standing-facts.md", "people/ada.md"])
    );
    let empty = reverification_queue(&dir, AT, &[]).expect("an empty queue is a queue");
    assert_eq!(empty, Vec::<String>::new());
}

#[test]
fn the_queue_never_writes() {
    let dir = fixture_copy("read-only");
    let snapshot = |root: &Path| {
        let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(current) = stack.pop() {
            for entry in fs::read_dir(&current).expect("read a directory") {
                let entry = entry.expect("a directory entry");
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    entries.push((
                        path.strip_prefix(root)
                            .expect("under the root")
                            .to_string_lossy()
                            .into_owned(),
                        fs::read(&path).expect("read a file"),
                    ));
                }
            }
        }
        entries.sort();
        entries
    };
    let before = snapshot(&dir);
    reverification_queue(&dir, AT, &paths(&["people/ada.md"])).expect("assemble the queue");
    assert_eq!(snapshot(&dir), before, "the queue is a list, never an edit");
}

#[test]
fn the_queue_refuses_paths_it_cannot_carry() {
    let dir = fixture_copy("refusals");
    fs::write(
        dir.join("notes.md"),
        "---\ntype: Topic\ntitle: \"Stray\"\ndescription: \"A file at the bundle root.\"\ngenerated: { by: \"human:david\", at: \"2026-11-02T00:00:00Z\" }\n---\n\n# Stray\n",
    )
    .expect("write a root file");
    let cases: [(&str, &str); 6] = [
        ("daily/2026-11-01.md", "daily layer"),
        ("claims.md", "registry"),
        ("curated/index.md", "not a concept"),
        ("notes.md", "not a concept"),
        ("curated/missing.md", "no file at"),
        ("directives/house-rules.md", "directive concept"),
    ];
    for (path, phrase) in cases {
        let error =
            reverification_queue(&dir, AT, &paths(&[path])).expect_err("the queue must refuse");
        assert!(
            error.to_string().contains(phrase),
            "for {path}, got {error}"
        );
    }
    let error = reverification_queue(&dir, "not-a-date", &paths(&["people/ada.md"]))
        .expect_err("a garbage stamp must refuse");
    assert!(
        error.to_string().contains("real calendar date"),
        "got {error}"
    );
}

#[test]
fn the_queue_refuses_a_path_carrying_a_separator() {
    // The queue prints one line per path, so its refusal set
    // is the printer's whole line alphabet, not the ASCII
    // pair alone.
    let dir = fixture_copy("separator-refusal");
    fs::write(
        dir.join("curated/weird\u{2028}name.md"),
        "---\ntype: Curated Memory\ntitle: \"Separator name\"\ndescription: \"A concept whose file name carries a line separator.\"\ngenerated: { by: \"process:looseleaf-synthesis\", at: \"2026-11-02T01:00:00Z\" }\nsources:\n  - id: day-one\n    resource: daily/2026-11-01.md\n    title: Daily log for 2026-11-01\n---\n\n# Separator name\n",
    )
    .expect("plant the concept");
    let error = reverification_queue(&dir, AT, &paths(&["curated/weird\u{2028}name.md"]))
        .expect_err("the queue must refuse");
    assert!(
        error.to_string().contains("carries a line break"),
        "got {error}"
    );
    // The refusal fires on the caller's path before any file
    // lookup, so the set's remaining members pin against
    // constructed paths, each refused for its line break and
    // not for a missing file.
    for separator in ['\u{b}', '\u{c}', '\u{85}', '\u{2029}'] {
        let path = format!("curated/weird{separator}name.md");
        let error = reverification_queue(&dir, AT, &paths(&[path.as_str()]))
            .expect_err("the queue must refuse");
        assert!(
            error.to_string().contains("carries a line break"),
            "for {path:?}, got {error}"
        );
    }
}
