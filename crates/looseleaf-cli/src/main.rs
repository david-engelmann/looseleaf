//! The looseleaf command line tool.
//!
//! With no arguments the tool reports its own version and the version
//! of the Open Knowledge Format profile it targets. The `check`
//! subcommand checks a bundle directory against the profile in
//! `docs/PROFILE.md` and prints one line per finding, exiting 1 when
//! any finding is an error and 0 otherwise. The `write-registry`
//! subcommand regenerates the bundle's claim registry, the
//! `write-indexes` subcommand regenerates every layer index, the
//! `write-root-index` subcommand regenerates the root index body
//! under its preserved frontmatter, and the `append-log` subcommand
//! appends one dated entry to a bundle or layer log. The `staleness`
//! subcommand reports concepts whose review date has passed at a
//! supplied stamp, and the `promote` subcommand promotes one daily
//! claim into a curated concept. Arguments are
//! strict. Anything the tool does not recognise is a usage error with
//! exit code 2, never something silently ignored, and a non-UTF-8
//! argument is reported rather than panicking, because a bundle path
//! with non-UTF-8 bytes is legal on the systems looseleaf runs on.

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use looseleaf_core::logs::LogScope;

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
        "write-indexes" => run_write_indexes(&args[1..]),
        "write-root-index" => run_write_root_index(&args[1..]),
        "append-log" => run_append_log(&args[1..]),
        "staleness" => run_staleness(&args[1..]),
        "promote" => run_promote(&args[1..]),
        other => {
            eprintln!("unknown subcommand {other}");
            eprintln!(
                "usage: looseleaf [check <bundle>] [write-registry <bundle> --at <timestamp>] [write-indexes <bundle>] [write-root-index <bundle>] [append-log <bundle> --date <date> --entry <text> [--layer <curated|daily>]] [staleness <bundle> --at <timestamp>] [promote <bundle> --claim <id> --to <path> [--reason <text>]]"
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

fn run_write_indexes(args: &[OsString]) -> ExitCode {
    if args.len() != 1 {
        eprintln!("usage: looseleaf write-indexes <bundle>");
        return ExitCode::from(2);
    }
    match looseleaf_core::indexes::regenerate_indexes(Path::new(&args[0])) {
        Ok(written) if written.is_empty() => {
            println!("indexes already current");
            ExitCode::SUCCESS
        }
        Ok(written) => {
            for rel in &written {
                println!("index regenerated, {rel}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not regenerate the indexes, {error}");
            ExitCode::FAILURE
        }
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

fn run_write_root_index(args: &[OsString]) -> ExitCode {
    if args.len() != 1 {
        eprintln!("usage: looseleaf write-root-index <bundle>");
        return ExitCode::from(2);
    }
    match looseleaf_core::indexes::regenerate_root_index(Path::new(&args[0])) {
        Ok(Some(rel)) => {
            println!("root index regenerated, {rel}");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            println!("root index already current");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not regenerate the root index, {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_append_log(args: &[OsString]) -> ExitCode {
    let shape_ok = (args.len() == 5 || args.len() == 7)
        && args[1].to_str() == Some("--date")
        && args[3].to_str() == Some("--entry")
        && (args.len() == 5 || args[5].to_str() == Some("--layer"));
    if !shape_ok {
        eprintln!(
            "usage: looseleaf append-log <bundle> --date <date> --entry <text> [--layer <curated|daily>]"
        );
        return ExitCode::from(2);
    }
    let (Some(date), Some(entry)) = (args[2].to_str(), args[4].to_str()) else {
        eprintln!("the --date and --entry values must be valid UTF-8");
        return ExitCode::from(2);
    };
    let scope = if args.len() == 7 {
        match args[6].to_str() {
            Some("curated") => LogScope::Curated,
            Some("daily") => LogScope::Daily,
            _ => {
                eprintln!("--layer names curated or daily, the layers the profile gives a log");
                return ExitCode::from(2);
            }
        }
    } else {
        LogScope::Root
    };
    match looseleaf_core::logs::append_log(Path::new(&args[0]), scope, date, entry) {
        Ok(rel) => {
            println!("log entry appended, {rel}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not append the log entry, {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_staleness(args: &[OsString]) -> ExitCode {
    if args.len() != 3 || args[1].to_str() != Some("--at") {
        eprintln!("usage: looseleaf staleness <bundle> --at <timestamp>");
        return ExitCode::from(2);
    }
    let Some(at) = args[2].to_str() else {
        eprintln!("the --at timestamp is not valid UTF-8");
        return ExitCode::from(2);
    };
    match looseleaf_core::pass::staleness_report(Path::new(&args[0]), at) {
        Ok(stale) => {
            for entry in &stale {
                println!(
                    "stale {} stale_after {} verified {}",
                    entry.path,
                    entry.stale_after,
                    entry.verified_at.as_deref().unwrap_or("never")
                );
            }
            println!("{} stale concepts at {at}", stale.len());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not compute staleness, {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_promote(args: &[OsString]) -> ExitCode {
    let shape_ok = (args.len() == 5 || args.len() == 7)
        && args[1].to_str() == Some("--claim")
        && args[3].to_str() == Some("--to")
        && (args.len() == 5 || args[5].to_str() == Some("--reason"));
    if !shape_ok {
        eprintln!("usage: looseleaf promote <bundle> --claim <id> --to <path> [--reason <text>]");
        return ExitCode::from(2);
    }
    let (Some(claim), Some(target)) = (args[2].to_str(), args[4].to_str()) else {
        eprintln!("the --claim and --to values must be valid UTF-8");
        return ExitCode::from(2);
    };
    let reason = if args.len() == 7 {
        let Some(reason) = args[6].to_str() else {
            eprintln!("the --reason value must be valid UTF-8");
            return ExitCode::from(2);
        };
        Some(reason)
    } else {
        None
    };
    match looseleaf_core::pass::promote_claim(Path::new(&args[0]), claim, target, reason) {
        Ok(looseleaf_core::pass::Promotion::Promoted) => {
            println!("claim promoted, {claim} into {target}");
            ExitCode::SUCCESS
        }
        Ok(looseleaf_core::pass::Promotion::AlreadyPresent) => {
            println!("claim already promoted, {claim} in {target}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not promote the claim, {error}");
            ExitCode::FAILURE
        }
    }
}
