use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// specjutsu — make AI coding agents prove their work.
#[derive(Parser)]
#[command(name = "sj", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Map spec scenario IDs to tests; list gaps and orphans.
    Trace {
        /// Repository root (defaults to the current directory).
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Spec root relative to the repository root.
        #[arg(long, default_value = "openspec")]
        spec_root: PathBuf,
        /// Print JSON instead of the compact report.
        #[arg(long)]
        json: bool,
        /// Exit 1 when there are gaps or orphans.
        #[arg(long)]
        strict: bool,
    },
    /// CI check: every changed scenario tested, red on base, green on head.
    Gate {
        /// Base ref to compare against, e.g. origin/main.
        #[arg(long, required_unless_present = "since")]
        base: Option<String>,
        /// Only check commit history: red commit before green, since this commit.
        #[arg(long, conflicts_with = "base")]
        since: Option<String>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, default_value = "openspec")]
        spec_root: PathBuf,
        /// Warn instead of failing when a test cannot be overlaid on base.
        #[arg(long)]
        allow_unverifiable: bool,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("sj: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    match Cli::parse().command {
        Command::Gate { .. } => todo!("GATE-018, GATE-019, GATE-020"),
        Command::Trace {
            root,
            spec_root,
            json,
            strict,
        } => {
            let scenarios = sj_core::spec::load_scenarios(&root.join(&spec_root))?;
            let tests = sj_core::testscan::scan_tree(&root)?;
            let report = sj_core::trace::build_report(&scenarios, &tests);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_compact(&report, &root);
            }
            Ok(if strict && !report.is_clean() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn print_compact(r: &sj_core::trace::Report, root: &Path) {
    println!(
        "trace: {} mapped, {}, {}, {} without ID",
        r.mapped.len(),
        plural(r.gaps.len(), "gap"),
        plural(r.orphans.len(), "orphan"),
        r.unidentified.len()
    );
    for g in &r.gaps {
        println!(
            "gap     {} {}  ({}:{})",
            g.id.as_deref().unwrap_or_default(),
            g.name,
            rel(&g.file, root),
            g.line
        );
    }
    for o in &r.orphans {
        println!(
            "orphan  {} {}  ({}:{})",
            o.id,
            o.test.name,
            rel(&o.test.file, root),
            o.test.line
        );
    }
    for u in &r.unidentified {
        println!("no-id   {}  ({}:{})", u.name, rel(&u.file, root), u.line);
    }
}
