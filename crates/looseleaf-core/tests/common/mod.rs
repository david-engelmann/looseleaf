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

/// The text of a shared plant document, one of the planted
/// concept documents the test files share from the fixtures
/// directory beside the clean fixture.
pub fn shared_plant(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read_to_string(&path).expect("read the shared plant")
}

/// A sentinel file outside a fixture bundle that a planted
/// symbolic link points at. The file is removed when the
/// guard drops, so a failed assertion cleans up as well as
/// a passed one.
#[cfg(unix)]
pub struct OutsideSentinel {
    path: PathBuf,
}

#[cfg(unix)]
impl OutsideSentinel {
    /// The sentinel file's path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(unix)]
impl Drop for OutsideSentinel {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Plant a symbolic link at a bundle writer's temporary
/// path, pointing at a fresh sentinel file outside the
/// bundle, and return the sentinel's guard. The tag names
/// the calling test file, so concurrent test files never
/// share a sentinel.
#[cfg(unix)]
pub fn plant_temp_symlink(bundle: &Path, temp_rel: &str, tag: &str) -> OutsideSentinel {
    let outside =
        std::env::temp_dir().join(format!("looseleaf-outside-{tag}-{}", std::process::id()));
    fs::write(&outside, "sentinel\n").expect("outside file plantable");
    std::os::unix::fs::symlink(&outside, bundle.join(temp_rel)).expect("symlink plantable");
    OutsideSentinel { path: outside }
}

/// Assert what a writer facing a planted temp link must
/// have done. The file the link points at is untouched, no
/// symbolic link stands at the target, and no temporary
/// file remains beside it. The write's own landing is the
/// calling test's assertion to make.
#[cfg(unix)]
pub fn assert_temp_write_not_followed(
    bundle: &Path,
    temp_rel: &str,
    target_rel: &str,
    outside: &OutsideSentinel,
) {
    assert_eq!(
        fs::read_to_string(outside.path()).expect("outside readable"),
        "sentinel\n",
        "the file the link points at is untouched"
    );
    assert!(
        !fs::symlink_metadata(bundle.join(target_rel))
            .expect("target metadata")
            .file_type()
            .is_symlink(),
        "no symbolic link stands at the target"
    );
    assert!(
        fs::symlink_metadata(bundle.join(temp_rel)).is_err(),
        "no temporary file remains beside the target"
    );
}
