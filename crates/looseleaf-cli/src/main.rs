//! The looseleaf command line tool.
//!
//! With no arguments the tool reports its own version and the version
//! of the Open Knowledge Format profile it targets. The `check`
//! subcommand checks a bundle directory against the profile in
//! `docs/PROFILE.md` and prints one line per finding, exiting 1 when
//! any finding is an error and 0 otherwise. The `write-registry`
//! subcommand regenerates the bundle's claim registry.

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => {
            println!("looseleaf {}", env!("CARGO_PKG_VERSION"));
            println!("OKF profile version {}", looseleaf_core::OKF_VERSION);
            println!("Run looseleaf check <bundle> to check a bundle against the profile.");
            ExitCode::SUCCESS
        }
        Some("check") => run_check(&args[1..]),
        Some("write-registry") => run_write_registry(&args[1..]),
        Some(other) => {
            eprintln!("unknown subcommand {other}");
            eprintln!(
                "usage: looseleaf [check <bundle>] [write-registry <bundle> --at <timestamp>]"
            );
            ExitCode::from(2)
        }
    }
}

fn run_check(args: &[String]) -> ExitCode {
    let Some(path) = args.first() else {
        eprintln!("usage: looseleaf check <bundle>");
        return ExitCode::from(2);
    };
    let report = looseleaf_core::check_bundle(Path::new(path));
    for finding in &report.findings {
        println!(
            "{} {} {}: {}",
            finding.severity.label(),
            finding.code,
            finding.path,
            finding.message
        );
    }
    println!(
        "checked {} files, {} errors, {} warnings",
        report.files_checked,
        report.errors(),
        report.warnings()
    );
    if report.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_write_registry(args: &[String]) -> ExitCode {
    let Some(path) = args.first() else {
        eprintln!("usage: looseleaf write-registry <bundle> --at <timestamp>");
        return ExitCode::from(2);
    };
    let at = args
        .windows(2)
        .find(|pair| pair[0] == "--at")
        .map(|pair| pair[1].clone());
    let Some(at) = at else {
        eprintln!("write-registry needs --at <timestamp>, the stamp for the generated entry");
        return ExitCode::from(2);
    };
    match looseleaf_core::registry::regenerate_registry(Path::new(path), &at) {
        Ok(true) => {
            println!("registry regenerated");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            println!("registry already current");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not regenerate the registry, {error}");
            ExitCode::FAILURE
        }
    }
}
