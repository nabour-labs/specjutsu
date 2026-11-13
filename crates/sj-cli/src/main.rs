use std::path::PathBuf;
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
        Command::Trace { .. } => todo!("TRACE-006"),
    }
}
