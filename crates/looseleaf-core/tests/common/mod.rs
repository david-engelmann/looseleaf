//! The shared fixture harness for the integration tests. Every
//! test file copies the clean fixture into its own temporary
//! directory before mutating it, and the copying lives here so a
//! harness fix lands once instead of once per test file.

#![allow(
    dead_code,
    reason = "each integration test crate uses the subset of the harness it needs"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The clean fixture bundle's directory.
pub fn clean_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("clean")
}

/// Copy a directory tree, file by file.
pub fn copy_dir(from: &Path, to: &Path) {
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

/// A writable copy of the clean fixture in a temporary directory
/// named for the calling test file and the test's own tag.
pub fn fixture_copy(prefix: &str, name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("looseleaf-{prefix}-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clear old fixture copy");
    }
    copy_dir(&clean_fixture(), &dir);
    dir
}
