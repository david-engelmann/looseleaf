//! The looseleaf command line tool.
//!
//! With no arguments the tool reports its own version and the version
//! of the Open Knowledge Format profile it targets. The `check`
//! subcommand checks a bundle directory against the profile in
//! `docs/PROFILE.md` and prints one line per finding, exiting 1 when
//! any finding is an error and 0 otherwise. The `write-registry`
//! subcommand regenerates the bundle's claim registry. Arguments are
//! strict. Anything the tool does not recognise is a usage error with
//! exit code 2, never something silently ignored, and a non-UTF-8
//! argument is reported rather than panicking, because a bundle path
//! with non-UTF-8 bytes is legal on the systems looseleaf runs on.

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let Some(first) = args.first() else {
        println!("looseleaf {}", env!("CARGO_PKG_VERSION"));
        println!("OKF profile version {}", looseleaf_core::OKF_VERSION);
        println!("Run looseleaf check <bundle> to check a bundle against the profile.");
        return ExitCode::SUCCESS;
    };
    let Some(command) = first.to_str() else {
        eprintln!("the subcommand is not valid UTF-8");
        return ExitCode::from(2);
    };
    match command {
        "check" => run_check(&args[1..]),
        "write-registry" => run_write_registry(&args[1..]),
        other => {
            eprintln!("unknown subcommand {other}");
            eprintln!(
                "usage: looseleaf [check <bundle>] [write-registry <bundle> --at <timestamp>]"
            );
            ExitCode::from(2)
        }
    }
}

fn run_check(args: &[OsString]) -> ExitCode {
    if args.len() != 1 {
        eprintln!("usage: looseleaf check <bundle>");
        return ExitCode::from(2);
    }
    let report = looseleaf_core::check_bundle(Path::new(&args[0]));
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

fn run_write_registry(args: &[OsString]) -> ExitCode {
    if args.len() != 3 || args[1].to_str() != Some("--at") {
        eprintln!("usage: looseleaf write-registry <bundle> --at <timestamp>");
        return ExitCode::from(2);
    }
    let Some(at) = args[2].to_str() else {
        eprintln!("the --at timestamp is not valid UTF-8");
        return ExitCode::from(2);
    };
    match looseleaf_core::registry::regenerate_registry(Path::new(&args[0]), at) {
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
