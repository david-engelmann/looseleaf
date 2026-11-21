//! The looseleaf command line tool.
//!
//! At the scaffold stage the tool reports its own version and the
//! version of the Open Knowledge Format profile it targets. Bundle
//! subcommands land with the first build slice. See DECISIONS.md in the
//! repository root for what is planned and what is deliberately out of
//! scope.

fn main() {
    println!("looseleaf {}", env!("CARGO_PKG_VERSION"));
    println!("OKF profile version {}", looseleaf_core::OKF_VERSION);
    println!("No subcommands yet. The bundle checker is the first build slice.");
}
