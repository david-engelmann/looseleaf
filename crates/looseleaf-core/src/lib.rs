//! Core library for looseleaf.
//!
//! Looseleaf is a memory system for AI agents that keeps what an agent
//! knows in plain Markdown files the user owns. The files form a bundle
//! that follows the Open Knowledge Format (OKF) at version 0.3, with a
//! looseleaf profile layered on top. The profile fixes the bundle layout,
//! a claim block convention that gives every claim a stable identity, a
//! verbatim quote and a source, and a generated claim registry that
//! holds the supersession chain for the whole bundle.
//!
//! This crate holds the deterministic half of the system. It reads
//! bundles and checks them against the profile. It never calls a
//! model. The judgment half of looseleaf, the synthesis pass that
//! decides what to promote and what to supersede, ships as a written
//! contract that a host agent executes. See DECISIONS.md in the
//! repository root and the profile in `docs/PROFILE.md`.
//!
//! The first build slice carries the bundle reader and the checker.
//! Writing, index regeneration and registry regeneration land in
//! later slices.

pub mod bundle;
pub mod check;
pub mod claim;
pub mod frontmatter;
pub mod registry;
pub mod write;

pub use check::{CheckReport, Finding, Severity, check_bundle};

/// The version of the Open Knowledge Format that the looseleaf profile
/// targets. Bundles declare this version in the `okf_version` field of
/// their root index file.
pub const OKF_VERSION: &str = "0.3";

/// The name of the looseleaf profile, used in generated file stamps so a
/// reader can tell which tool wrote a generated file and under which
/// profile.
pub const PROFILE_NAME: &str = "looseleaf";

#[cfg(test)]
mod tests {
    use super::{OKF_VERSION, PROFILE_NAME};

    #[test]
    fn profile_targets_okf_v03() {
        assert_eq!(OKF_VERSION, "0.3");
    }

    #[test]
    fn profile_name_is_looseleaf() {
        assert_eq!(PROFILE_NAME, "looseleaf");
    }
}
