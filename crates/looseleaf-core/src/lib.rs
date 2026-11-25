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
//! bundles, checks them against the profile, and writes the parts of
//! a bundle that are generated, the claim registry, the layer and
//! root indexes, the dated log appends, and the synthesis pass
//! mechanics of staleness reporting and claim promotion, through
//! byte-stable writers that preserve whatever they do not own. The
//! pass paperwork lives here too, the frontmatter stamps, the
//! Synthesis report concept each run is audited against, the
//! re-verification queue a pass hands on, and the deprecation of
//! a concept in favour of its replacement. It
//! never calls a model. The judgment half of looseleaf, the synthesis
//! pass that decides what to promote and what to supersede, ships as
//! a written contract that a host agent executes. See DECISIONS.md in
//! the repository root and the profile in `docs/PROFILE.md`.

pub mod bundle;
pub mod check;
pub mod claim;
pub mod frontmatter;
pub mod indexes;
pub mod logs;
pub mod pass;
pub mod registry;
pub mod report;
pub mod stamps;
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
    use super::OKF_VERSION;

    #[test]
    fn profile_targets_okf_v03() {
        assert_eq!(OKF_VERSION, "0.3");
    }
}
