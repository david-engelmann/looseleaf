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
//! claim into a curated concept. The `stamp-generated` and
//! `stamp-verified` subcommands stamp one concept's `generated`
//! and `verified` frontmatter under the synthesis contract, and
//! the `write-report` subcommand appends one entry to a section
//! of the run's Synthesis report concept. The `queue` subcommand
//! prints the re-verification queue a pass hands on, the concepts
//! the pass created or changed in meaning. The `deprecate`
//! subcommand executes a concept deprecation the contract's
//! human gate has decided, in the profile's shape. Arguments are
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
        "stamp-generated" => run_stamp_generated(&args[1..]),
        "stamp-verified" => run_stamp_verified(&args[1..]),
        "write-report" => run_write_report(&args[1..]),
        "queue" => run_queue(&args[1..]),
        "deprecate" => run_deprecate(&args[1..]),
        other => {
            eprintln!("unknown subcommand {other}");
            eprintln!(
                "usage: looseleaf [check <bundle>] [write-registry <bundle> --at <timestamp>] [write-indexes <bundle>] [write-root-index <bundle>] [append-log <bundle> --date <date> --entry <text> [--layer <curated|daily>]] [staleness <bundle> --at <timestamp>] [promote <bundle> --claim <id> --to <path> [--reason <text>]] [stamp-generated <bundle> --path <path> --at <timestamp> [--by <actor>]] [stamp-verified <bundle> --path <path> --by <actor> --at <timestamp>] [write-report <bundle> --at <timestamp> --section <token> --entry <text>] [queue <bundle> --at <timestamp> [--changed <path>]...] [deprecate <bundle> --path <path> --superseded-by <path> --at <timestamp>]"
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

/// Escape a bundle-derived field for one printed line. The
/// staleness lines are pasted into the pass report that audits
/// the run, so a field carrying a line break would forge extra
/// lines in that record. Backslashes double first, then a line
/// break and a carriage return render as two-character
/// sequences. The core report's own values are never escaped,
/// only what this printer puts on the line.
fn escape_line_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
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
                    escape_line_field(&entry.path),
                    entry.stale_after,
                    escape_line_field(entry.verified_at.as_deref().unwrap_or("never"))
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

fn run_stamp_generated(args: &[OsString]) -> ExitCode {
    let shape_ok = (args.len() == 5 || args.len() == 7)
        && args[1].to_str() == Some("--path")
        && args[3].to_str() == Some("--at")
        && (args.len() == 5 || args[5].to_str() == Some("--by"));
    if !shape_ok {
        eprintln!(
            "usage: looseleaf stamp-generated <bundle> --path <path> --at <timestamp> [--by <actor>]"
        );
        return ExitCode::from(2);
    }
    let (Some(path), Some(at)) = (args[2].to_str(), args[4].to_str()) else {
        eprintln!("the --path and --at values must be valid UTF-8");
        return ExitCode::from(2);
    };
    let by = if args.len() == 7 {
        let Some(by) = args[6].to_str() else {
            eprintln!("the --by value must be valid UTF-8");
            return ExitCode::from(2);
        };
        by
    } else {
        looseleaf_core::stamps::PASS_ACTOR
    };
    match looseleaf_core::stamps::stamp_generated(Path::new(&args[0]), path, by, at) {
        Ok(looseleaf_core::stamps::Stamp::Stamped) => {
            println!("generated stamped, {path}");
            ExitCode::SUCCESS
        }
        Ok(looseleaf_core::stamps::Stamp::AlreadyCurrent) => {
            println!("generated already current, {path}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not stamp generated, {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_deprecate(args: &[OsString]) -> ExitCode {
    let shape_ok = args.len() == 7
        && args[1].to_str() == Some("--path")
        && args[3].to_str() == Some("--superseded-by")
        && args[5].to_str() == Some("--at");
    if !shape_ok {
        eprintln!(
            "usage: looseleaf deprecate <bundle> --path <path> --superseded-by <path> --at <timestamp>"
        );
        return ExitCode::from(2);
    }
    let (Some(path), Some(replacement), Some(at)) =
        (args[2].to_str(), args[4].to_str(), args[6].to_str())
    else {
        eprintln!("the --path, --superseded-by and --at values must be valid UTF-8");
        return ExitCode::from(2);
    };
    match looseleaf_core::stamps::deprecate_concept(Path::new(&args[0]), path, replacement, at) {
        Ok(looseleaf_core::stamps::Deprecation::Deprecated) => {
            println!("concept deprecated, {path}");
            ExitCode::SUCCESS
        }
        Ok(looseleaf_core::stamps::Deprecation::AlreadyCurrent) => {
            println!("concept already deprecated, {path}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not deprecate, {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_stamp_verified(args: &[OsString]) -> ExitCode {
    let shape_ok = args.len() == 7
        && args[1].to_str() == Some("--path")
        && args[3].to_str() == Some("--by")
        && args[5].to_str() == Some("--at");
    if !shape_ok {
        eprintln!(
            "usage: looseleaf stamp-verified <bundle> --path <path> --by <actor> --at <timestamp>"
        );
        return ExitCode::from(2);
    }
    let (Some(path), Some(by), Some(at)) = (args[2].to_str(), args[4].to_str(), args[6].to_str())
    else {
        eprintln!("the --path, --by and --at values must be valid UTF-8");
        return ExitCode::from(2);
    };
    match looseleaf_core::stamps::stamp_verified(Path::new(&args[0]), path, by, at) {
        Ok(looseleaf_core::stamps::Stamp::Stamped) => {
            println!("verification stamped, {path}");
            ExitCode::SUCCESS
        }
        Ok(looseleaf_core::stamps::Stamp::AlreadyCurrent) => {
            println!("verification already recorded, {path}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not stamp verified, {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_write_report(args: &[OsString]) -> ExitCode {
    let shape_ok = args.len() == 7
        && args[1].to_str() == Some("--at")
        && args[3].to_str() == Some("--section")
        && args[5].to_str() == Some("--entry");
    if !shape_ok {
        eprintln!(
            "usage: looseleaf write-report <bundle> --at <timestamp> --section <promotions|supersessions|regenerated|flags|blocked|queue> --entry <text>"
        );
        return ExitCode::from(2);
    }
    let (Some(at), Some(token), Some(entry)) =
        (args[2].to_str(), args[4].to_str(), args[6].to_str())
    else {
        eprintln!("the --at, --section and --entry values must be valid UTF-8");
        return ExitCode::from(2);
    };
    let Some(section) = looseleaf_core::report::ReportSection::from_token(token) else {
        eprintln!("unknown report section {token}");
        eprintln!(
            "usage: looseleaf write-report <bundle> --at <timestamp> --section <promotions|supersessions|regenerated|flags|blocked|queue> --entry <text>"
        );
        return ExitCode::from(2);
    };
    match looseleaf_core::report::append_report_entry(Path::new(&args[0]), at, section, entry) {
        Ok(rel) => {
            println!("report entry written, {rel}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not write the report entry, {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_queue(args: &[OsString]) -> ExitCode {
    // The queue grammar is variable-arity, the one runner whose
    // shape is a loop over --changed pairs rather than a fixed
    // shape predicate. A dangling or unrecognised token is a
    // usage error, never something silently ignored.
    let usage = "usage: looseleaf queue <bundle> --at <timestamp> [--changed <path>]...";
    if args.len() < 3 || args[1].to_str() != Some("--at") {
        eprintln!("{usage}");
        return ExitCode::from(2);
    }
    let Some(at) = args[2].to_str() else {
        eprintln!("the --at value must be valid UTF-8");
        return ExitCode::from(2);
    };
    let mut changed: Vec<String> = Vec::new();
    let mut index = 3;
    while index < args.len() {
        if args[index].to_str() != Some("--changed") || index + 1 >= args.len() {
            eprintln!("{usage}");
            return ExitCode::from(2);
        }
        let Some(path) = args[index + 1].to_str() else {
            eprintln!("the --changed value must be valid UTF-8");
            return ExitCode::from(2);
        };
        changed.push(path.to_owned());
        index += 2;
    }
    match looseleaf_core::pass::reverification_queue(Path::new(&args[0]), at, &changed) {
        Ok(queue) => {
            for path in &queue {
                println!("queue {path} changed in this pass");
            }
            println!(
                "{} concepts in the re-verification queue at {at}",
                queue.len()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not assemble the queue, {error}");
            ExitCode::FAILURE
        }
    }
}
