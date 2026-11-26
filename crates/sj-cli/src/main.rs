use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

/// specjutsu — make AI coding agents prove their work.
#[derive(Parser)]
#[command(name = "sj", version = env!("SJ_VERSION"), about)]
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
        /// Spec root relative to the repository root (overrides specjutsu.toml; default specjutsu, or openspec when only that folder exists).
        #[arg(long)]
        spec_root: Option<PathBuf>,
        /// Print JSON instead of the compact report.
        #[arg(long)]
        json: bool,
        /// Exit 1 on gaps or orphans the baseline doesn't list, stale baseline entries, or FOR ALL scenarios with no property test.
        #[arg(long)]
        strict: bool,
        /// Write every current gap and orphan to <spec root>/trace-baseline.txt (never not-property entries).
        #[arg(long, conflicts_with_all = ["json", "strict"])]
        write_baseline: bool,
    },
    /// Check the change's tests for removals, skips, focus and lost assertions; works without specs.
    Audit {
        /// Base ref to compare against, e.g. origin/main.
        #[arg(long)]
        base: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, conflicts_with = "format")]
        json: bool,
        /// Output format: text (default), json, or markdown (a PR comment body).
        #[arg(long, value_enum)]
        format: Option<GateFormat>,
        /// Look up added dependencies on crates.io, npm and PyPI (makes network requests).
        #[arg(long)]
        registry: bool,
    },
    /// Find scenarios whose code changed while their spec and tests did not, and the reverse.
    Drift {
        /// Base ref to compare against, e.g. origin/main.
        #[arg(long)]
        base: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        spec_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Coverage per scenario, for spec drift detection.
    Coverage {
        #[command(subcommand)]
        command: CoverageCommand,
    },
    /// Write a shields.io endpoint badge with the share of tested scenarios.
    Badge {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        spec_root: Option<PathBuf>,
        /// Where to write the JSON (default .specjutsu/badge.json under the root).
        #[arg(long)]
        out: Option<PathBuf>,
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
        #[arg(long)]
        spec_root: Option<PathBuf>,
        /// Warn instead of failing when a test cannot be overlaid on base.
        #[arg(long)]
        allow_unverifiable: bool,
        #[arg(long, conflicts_with = "format")]
        json: bool,
        /// Output format: text (default), json, or markdown (a PR comment body).
        #[arg(long, value_enum)]
        format: Option<GateFormat>,
    },
    /// Run the tests (all, one package, or one scenario's); print failures only; record the run.
    Test {
        /// A scenario ID (e.g. GATE-004) or a Cargo package name. Omit to run everything.
        scope: Option<String>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        spec_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
        /// Do not write the run to the evidence ledger.
        #[arg(long)]
        no_ledger: bool,
    },
    /// Start a project: write specjutsu.toml and create the spec folders.
    Init {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Replace an existing specjutsu.toml.
        #[arg(long)]
        force: bool,
        /// Install agent adapters (comma-separated: claude, codex, cursor, cline, copilot, gemini, windsurf, kiro, antigravity, agents-md, auto).
        #[arg(long, value_delimiter = ',')]
        agents: Vec<String>,
        /// Remove agent adapters (comma-separated, as for --agents).
        #[arg(long, value_delimiter = ',', conflicts_with = "agents")]
        remove_agents: Vec<String>,
        /// Install or remove only the skills, in ~/.claude/skills and ~/.agents/skills, for
        /// every project. Hooks and instruction blocks stay per repository.
        #[arg(long)]
        global: bool,
    },
    /// Hook entrypoint for agent tools: `sj hook claude session-start|pre-edit`,
    /// `sj hook codex|cursor|copilot|gemini|windsurf|kiro|antigravity pre-edit`, or
    /// `sj hook check-edit <path>` (exit 0 allows, 2 blocks) for any other tool.
    Hook {
        /// The agent tool (claude, codex, cursor, copilot, gemini, windsurf, kiro, antigravity),
        /// or check-edit.
        agent: String,
        /// The hook event (session-start or pre-edit), or the path for check-edit.
        event: String,
        /// Project root (default: $CLAUDE_PROJECT_DIR, else the current directory).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Regenerate installed agent adapters; hand-edited files are kept unless --force.
    Update {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Replace generated files even when they were edited by hand.
        #[arg(long)]
        force: bool,
        /// Print the files the update would write, keep or remove, and change nothing.
        #[arg(long)]
        dry_run: bool,
        /// With --dry-run: print the plan as `{"files": [{"path", "action"}]}`.
        #[arg(long, requires = "dry_run")]
        json: bool,
        /// Refresh the skills installed with `sj init --global` instead of the repository's.
        #[arg(long)]
        global: bool,
    },
    /// Detect an existing project's packages and runners and write specjutsu.toml.
    Adopt {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Print the config instead of writing it.
        #[arg(long)]
        dry_run: bool,
        /// Replace an existing specjutsu.toml.
        #[arg(long)]
        force: bool,
    },
    /// Check the setup: project file, packages, runner executables and spec folders.
    Doctor {
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Create, list, validate, check and archive spec changes.
    Spec {
        #[command(subcommand)]
        command: SpecCommand,
    },
    /// List scenarios without IDs and the IDs they would get; --write assigns them in place.
    Ids {
        #[command(subcommand)]
        command: Option<IdsCommand>,
        /// Insert the IDs into the spec files.
        #[arg(long)]
        write: bool,
        #[arg(long, default_value = ".", global = true)]
        root: PathBuf,
        #[arg(long, global = true)]
        spec_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
        /// Capabilities, or paths to spec files or folders, to limit the run to; all when none.
        /// A path is resolved against --root (or is absolute) and selects the capability of
        /// every open spec file under it. A capability named `prefixes` can only be named by
        /// path, since `sj ids prefixes` is the subcommand.
        targets: Vec<String>,
    },
    /// Convert another tool's spec folders into the specjutsu layout (dry run unless --write).
    Import {
        #[command(subcommand)]
        command: ImportCommand,
    },
    /// Replace OpenSpec, Spec Kit or Superpowers with specjutsu (dry run unless --write).
    Migrate {
        /// The tool to migrate from: openspec, speckit or superpowers. Repeatable.
        #[arg(long = "from", required_unless_present = "undo", value_parser = ["openspec", "speckit", "superpowers"])]
        from: Vec<String>,
        /// Apply the plan. Needs a clean git working tree.
        #[arg(long)]
        write: bool,
        /// With the preview, print a unified diff of every file it would create or change.
        #[arg(long, conflicts_with = "write")]
        diff: bool,
        /// Put back what the last --write changed, from its undo manifest.
        #[arg(long, conflicts_with_all = ["from", "write", "diff"])]
        undo: bool,
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Where each open change stands: tasks, scenario evidence and the next task.
    Status {
        /// Only this change (default: every open change).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// One-screen project dashboard: capabilities, open changes, evidence totals, orphans.
    View {
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// The next task's context: its scenarios, changed files and test commands.
    Next {
        /// Only this change (default: the first open change with an open task).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        /// List open [P] tasks in batches that share no test file.
        #[arg(long)]
        parallel: bool,
        /// Base ref for changed files (default: origin/HEAD, origin/main, main, ...).
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// What to write next for one artifact of a change: template, context, rules, next IDs.
    Instructions {
        change: String,
        /// proposal, specs, design or tasks (default: the first one not yet written).
        artifact: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// List the templates and where each resolves from: the project or the built-in.
    Templates {
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// Local check before a PR: the gate's checks without red-on-base.
    Verify {
        /// Only this change (default: every open change).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        /// Base ref (default: origin/HEAD, origin/main, origin/master, main, master).
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Check off tasks in a change's tasks.md.
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// Per-scenario evidence from recorded runs: latest result and red-before-green.
    Ledger {
        /// Report only this scenario ID.
        id: Option<String>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Bytes the agent-loop commands printed against the raw output they replaced.
    Gain {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        json: bool,
        /// Count only records at or after this RFC 3339 time, e.g. 2026-11-22T11:00:00Z.
        #[arg(long, value_parser = parse_rfc3339)]
        since: Option<chrono::DateTime<chrono::FixedOffset>>,
        /// Delete the records and print how many were removed.
        #[arg(long, conflicts_with_all = ["since", "json"])]
        reset: bool,
    },
    /// Print a shell completion script to stdout, e.g. `sj completions zsh > ~/.zfunc/_sj`.
    Completions {
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(clap::Args)]
struct SpecArgs {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Spec root relative to the repository root (overrides specjutsu.toml; default specjutsu, or openspec when only that folder exists).
    #[arg(long)]
    spec_root: Option<PathBuf>,
}

#[derive(Subcommand)]
enum SpecCommand {
    /// Scaffold changes/<change>/ with proposal, design, tasks and specs/.
    New {
        /// Kebab-case change name.
        change: String,
        /// Scaffold a bug fix: current, expected and unchanged behaviour, and `Kind: bug`.
        #[arg(long)]
        bug: bool,
        #[command(flatten)]
        args: SpecArgs,
    },
    /// Print one capability spec, change (open or archived) or scenario with its evidence.
    Show {
        /// A scenario ID, change name, archived change or capability.
        item: String,
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// Open changes with task progress and scenario counts.
    List {
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// Check spec format, delta structure and that every task names its scenarios.
    Validate {
        /// Only this change (default: specs/ and every open change).
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
        /// Fail on warnings too.
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        json: bool,
    },
    /// Scenarios no task covers, tasks naming unknown IDs, and open questions.
    Check {
        change: String,
        #[command(flatten)]
        args: SpecArgs,
        #[arg(long)]
        json: bool,
    },
    /// Draft `[keep]` scenarios from the untraced tests under the given paths (dry run).
    Infer {
        /// Test files or folders, relative to --root or absolute.
        #[arg(required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
        /// The capability the drafts belong to; its prefix numbers them.
        #[arg(long)]
        capability: Option<String>,
        /// Write the drafts as a change and link each test with a `spec:` comment.
        #[arg(long)]
        write: bool,
        /// The change to write (default `infer-<capability>`).
        #[arg(long, requires = "write")]
        change: Option<String>,
        #[command(flatten)]
        args: SpecArgs,
    },
    /// Apply each change's deltas to specs/ and move it to changes/archive/. With several
    /// changes, all are checked and merged first; one failure archives none.
    Archive {
        #[arg(required_unless_present = "all_done", num_args = 1..)]
        changes: Vec<String>,
        /// Archive every open change whose tasks are all checked, in name order.
        #[arg(long, conflicts_with = "changes")]
        all_done: bool,
        #[command(flatten)]
        args: SpecArgs,
        /// Archive despite validation errors and open tasks.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum IdsCommand {
    /// Print an [ids.prefixes] table with a short prefix for every capability folder.
    Prefixes {
        /// Add the capabilities missing from [ids.prefixes] to specjutsu.toml.
        #[arg(long)]
        write: bool,
    },
}

#[derive(Subcommand)]
enum ImportCommand {
    /// Spec Kit features (`specs/NNN-<name>/`) and the constitution.
    Speckit {
        #[command(flatten)]
        args: SpecArgs,
        /// Write the files (default: print the plan).
        #[arg(long)]
        write: bool,
        /// Overwrite targets that already exist.
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
}

/// Shells `sj completions` supports, in the order the error message lists them.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    Elvish,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(s: CompletionShell) -> Self {
        match s {
            CompletionShell::Bash => Self::Bash,
            CompletionShell::Zsh => Self::Zsh,
            CompletionShell::Fish => Self::Fish,
            CompletionShell::PowerShell => Self::PowerShell,
            CompletionShell::Elvish => Self::Elvish,
        }
    }
}

#[derive(Subcommand)]
enum TaskCommand {
    /// Check a task's box; strict mode requires red-before-green for its scenarios.
    Done {
        change: String,
        /// The task's number, e.g. 2.1.
        task: String,
        #[command(flatten)]
        args: SpecArgs,
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
        Command::Gate {
            base,
            since,
            root,
            spec_root,
            allow_unverifiable,
            json,
            format,
        } => {
            let spec_root = spec_root_for(&root, spec_root)?;
            let format = format.unwrap_or(if json {
                GateFormat::Json
            } else {
                GateFormat::Text
            });
            let shown_base = base.clone().unwrap_or_default();
            let report = match (base, since) {
                (_, Some(since)) => sj_core::gate::history(&root, &since)?,
                (Some(base), None) => sj_core::gate::run(&sj_core::gate::Options {
                    root,
                    spec_root,
                    base,
                    allow_unverifiable,
                })?,
                (None, None) => anyhow::bail!("pass --base <ref> or --since <commit>"),
            };
            if format == GateFormat::Json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if format == GateFormat::Markdown {
                print!("{}", sj_core::ci::render_markdown(&report, &shown_base));
            } else {
                println!("{}", report.summary);
                for f in &report.failures {
                    println!("fail    {f}");
                }
                for w in &report.warnings {
                    println!("warn    {w}");
                }
                for i in &report.info {
                    println!("info    {i}");
                }
                for e in &report.exemptions {
                    println!("exempt  {e}");
                }
            }
            Ok(if report.pass {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Command::Test {
            scope,
            root,
            spec_root,
            json,
            no_ledger,
        } => {
            let spec_root = spec_root_for(&root, spec_root)?;
            test_cmd(scope, root, spec_root, json, no_ledger)
        }
        Command::Init {
            root,
            force,
            agents,
            remove_agents,
            global,
        } => {
            if global {
                return global_init_cmd(&root, &agents, &remove_agents, force);
            }
            if !remove_agents.is_empty() {
                return remove_agents_cmd(&root, &remove_agents);
            }
            if agents.is_empty() {
                return init_cmd(&root, force);
            }
            sj_adapters::agents::check(&agents)?;
            let auto = agents.iter().any(|a| a == "auto");
            let (agents, found) = sj_adapters::agents::resolve(&root, &agents);
            if auto {
                let note = if found { "" } else { " (no agent files found)" };
                println!("agents: {}{note}", agents.join(", "));
            }
            if force || !root.join(sj_core::config::FILE).exists() {
                // Its exit code is always success; errors come back as `Err`.
                let _ = init_cmd(&root, force)?;
            }
            let code = agents_cmd(&root, &agents, false, false)?;
            if agents.iter().any(|a| a == "copilot") {
                println!("{}", sj_adapters::copilot::NOTICE);
            }
            Ok(code)
        }
        Command::Hook { agent, event, root } => Ok(hook_cmd(&agent, &event, root)),
        Command::Completions { shell } => {
            let shell: clap_complete::Shell = shell.into();
            clap_complete::generate(shell, &mut Cli::command(), "sj", &mut std::io::stdout());
            Ok(ExitCode::SUCCESS)
        }
        Command::Update {
            root,
            force,
            dry_run,
            json,
            global,
        } => {
            if global {
                return global_update_cmd(force, dry_run, json);
            }
            let installed = sj_adapters::agents::installed(&root);
            if dry_run {
                let steps = if installed.is_empty() {
                    Vec::new()
                } else {
                    sj_adapters::agents::plan(&root, &installed, force)?
                };
                if json {
                    let files: Vec<_> = steps
                        .iter()
                        .map(|s| serde_json::json!({"path": s.path(), "action": s.action()}))
                        .collect();
                    let v = serde_json::json!({ "files": files });
                    println!("{}", serde_json::to_string_pretty(&v)?);
                    return Ok(ExitCode::SUCCESS);
                }
                if installed.is_empty() {
                    println!("no adapters installed");
                    return Ok(ExitCode::SUCCESS);
                }
                if steps.is_empty() {
                    println!("adapters up to date");
                }
                for s in &steps {
                    println!("{} {}", s.action(), s.path());
                }
                return Ok(ExitCode::SUCCESS);
            }
            if installed.is_empty() {
                println!("no adapters installed");
                return Ok(ExitCode::SUCCESS);
            }
            agents_cmd(&root, &installed, force, true)
        }
        Command::Doctor { root } => {
            let problems = sj_core::doctor::check(&root);
            if problems.is_empty() {
                println!("doctor: ok");
            } else {
                println!("doctor: {}", plural(problems.len(), "problem"));
            }
            for p in &problems {
                println!("{p}");
            }
            for (agent, level) in sj_adapters::enforcement::levels(&root) {
                println!("{agent}: {level}");
            }
            let mut skills = Vec::new();
            if sj_adapters::global::has_skills(&root) {
                skills.push("project");
            }
            if sj_adapters::global::home().is_some_and(|h| sj_adapters::global::has_skills(&h)) {
                skills.push("global");
            }
            if !skills.is_empty() {
                println!("skills: {}", skills.join(", "));
            }
            Ok(if problems.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Command::Adopt {
            root,
            dry_run,
            force,
        } => adopt_cmd(&root, dry_run, force),
        Command::Spec { command } => spec_cmd(command),
        Command::Ids {
            command: Some(IdsCommand::Prefixes { .. }),
            write: true,
            ..
        }
        | Command::Ids {
            command: Some(IdsCommand::Prefixes { .. }),
            json: true,
            ..
        } => {
            anyhow::bail!(
                "`sj ids --write` and `--json` do not apply to `prefixes`; \
                 use `sj ids prefixes --write`"
            )
        }
        Command::Ids {
            command: Some(IdsCommand::Prefixes { write }),
            root,
            spec_root,
            ..
        } => ids_prefixes_cmd(&root, spec_root, write),
        Command::Ids {
            command: None,
            write,
            root,
            spec_root,
            json,
            targets,
        } => ids_cmd(write, &root, spec_root, json, &targets),
        Command::Import {
            command:
                ImportCommand::Speckit {
                    args,
                    write,
                    force,
                    json,
                },
        } => import_speckit_cmd(&args, write, force, json),
        Command::Migrate {
            from,
            write,
            diff,
            undo,
            root,
        } => {
            if undo {
                migrate_undo_cmd(&root)
            } else {
                migrate_cmd(&root, &from, write, diff)
            }
        }
        Command::Status { change, args, json } => status_cmd(change, &args, json),
        Command::View { args, json } => view_cmd(&args, json),
        Command::Task {
            command: TaskCommand::Done { change, task, args },
        } => task_done_cmd(&change, &task, &args),
        Command::Next {
            change,
            args,
            parallel,
            base,
            json,
        } => next_cmd(change, &args, parallel, base, json),
        Command::Verify {
            change,
            args,
            base,
            json,
        } => verify_cmd(change, &args, base, json),
        Command::Instructions {
            change,
            artifact,
            args,
            json,
        } => instructions_cmd(&change, artifact.as_deref(), &args, json),
        Command::Templates { args, json } => templates_cmd(&args, json),
        Command::Ledger { id, root, json } => {
            sj_core::config::load(&root)?;
            ledger_cmd(id, root, json)
        }
        Command::Gain {
            root,
            json,
            since,
            reset,
        } => {
            sj_core::config::load(&root)?;
            if reset {
                let n = sj_core::gain::reset(&root)?;
                println!(
                    "removed {n} gain {}",
                    if n == 1 { "record" } else { "records" }
                );
                return Ok(ExitCode::SUCCESS);
            }
            gain_cmd(&root, json, since)
        }
        Command::Audit {
            base,
            root,
            json,
            format,
            registry,
        } => {
            let report = sj_core::audit::run(&sj_core::audit::Options {
                root,
                base: base.clone(),
                registry,
            })?;
            match format.unwrap_or(if json {
                GateFormat::Json
            } else {
                GateFormat::Text
            }) {
                GateFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                GateFormat::Markdown => {
                    print!("{}", sj_core::ci::render_audit_markdown(&report, &base))
                }
                GateFormat::Text => print!("{}", report.render()),
            }
            Ok(if report.pass {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Command::Drift {
            base,
            root,
            spec_root,
            json,
        } => {
            let spec_root = spec_root_for(&root, spec_root)?;
            let report = sj_core::drift::run(&sj_core::drift::Options {
                root,
                spec_root,
                base,
            })?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print!("{}", report.render());
            }
            Ok(if report.pass {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Command::Coverage {
            command: CoverageCommand::Map { root, spec_root },
        } => {
            let spec_root = spec_root_for(&root, spec_root)?;
            match sj_core::drift::build_map(&root, &spec_root)? {
                sj_core::drift::MapOutcome::UpToDate => println!("coverage map up to date"),
                sj_core::drift::MapOutcome::Built {
                    scenarios,
                    files,
                    shared_lines,
                } => {
                    println!(
                        "coverage map: {} over {} → {}",
                        plural(scenarios, "scenario"),
                        plural(files, "file"),
                        sj_core::drift::MAP_PATH
                    );
                    if shared_lines > 0 {
                        println!("{} dropped", plural(shared_lines, "shared line"));
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Badge {
            root,
            spec_root,
            out,
        } => {
            let config = sj_core::config::load(&root)?;
            let spec_root = spec_root.unwrap_or_else(|| PathBuf::from(&config.spec_root));
            let spec_dir = root.join(&spec_root);
            let scenarios = sj_core::spec::load_scenarios(&spec_dir)?;
            let scan = sj_core::testscan::scan_project(&root, &config)?;
            let report = sj_core::trace::build_report(&scenarios, &scan.tests);
            let badge = sj_core::ci::badge(&spec_dir, &scenarios, &report);
            let out = out.unwrap_or_else(|| root.join(".specjutsu/badge.json"));
            if let Some(dir) = out.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(&out, format!("{}\n", serde_json::to_string(&badge)?))?;
            println!("badge: {} → {}", badge.message, out.display());
            Ok(ExitCode::SUCCESS)
        }
        Command::Trace {
            root,
            spec_root,
            json,
            strict,
            write_baseline,
        } => {
            let config = sj_core::config::load(&root)?;
            let spec_root = spec_root.unwrap_or_else(|| PathBuf::from(&config.spec_root));
            let scenarios = sj_core::spec::load_scenarios(&root.join(&spec_root))?;
            let scan = sj_core::testscan::scan_project(&root, &config)?;
            let mut report = sj_core::trace::build_report(&scenarios, &scan.tests);
            report.warnings = scan
                .unsuited
                .iter()
                .map(|p| format!("no test suite: {p}"))
                .collect();
            if write_baseline {
                let baseline = sj_core::trace::Baseline::from_report(&report);
                let file = root.join(&spec_root).join(sj_core::trace::BASELINE_FILE);
                std::fs::write(&file, baseline.render())?;
                println!(
                    "baseline: {} {} → {}",
                    baseline.entries.len(),
                    if baseline.entries.len() == 1 {
                        "entry"
                    } else {
                        "entries"
                    },
                    file.display()
                );
                return Ok(ExitCode::SUCCESS);
            }
            let baseline_file = root.join(&spec_root).join(sj_core::trace::BASELINE_FILE);
            let baseline = if baseline_file.exists() {
                let text = std::fs::read_to_string(&baseline_file)
                    .with_context(|| baseline_file.display().to_string())?;
                Some(
                    sj_core::trace::Baseline::parse(&text)
                        .with_context(|| baseline_file.display().to_string())?,
                )
            } else {
                None
            };
            let known = Known::new(&report, baseline.as_ref());
            if json && baseline.is_some() {
                let view = KnownReport::new(&report, &known);
                println!("{}", serde_json::to_string_pretty(&view)?);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_compact(&report, &root, &known, baseline.is_some());
            }
            Ok(
                if strict && (known.fails_strict() || !report.not_property.is_empty()) {
                    ExitCode::from(1)
                } else {
                    ExitCode::SUCCESS
                },
            )
        }
    }
}

fn init_cmd(root: &Path, force: bool) -> Result<ExitCode> {
    use sj_core::config::{Config, FILE};

    let file = root.join(FILE);
    if file.exists() && !force {
        anyhow::bail!("{FILE} already exists; pass --force to replace it");
    }
    let config = Config::default_for(root);
    std::fs::write(&file, config.to_toml())?;
    sj_core::config::ignore_ledger(root)?;
    let spec_root = root.join(&config.spec_root);
    for dir in ["specs", "changes"] {
        std::fs::create_dir_all(spec_root.join(dir))?;
    }
    println!(
        "init: wrote {FILE} (spec root {}, mode {}, {})",
        config.spec_root,
        config.mode.as_str(),
        plural(config.packages.len(), "package")
    );
    Ok(ExitCode::SUCCESS)
}

/// Agent hooks never fail the tool call on an internal error: they warn and allow.
fn hook_cmd(agent: &str, event: &str, root: Option<PathBuf>) -> ExitCode {
    use sj_core::hook::{self, Decision};
    let root = root
        .or_else(|| std::env::var_os("CLAUDE_PROJECT_DIR").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    if agent == "check-edit" {
        return check_edit(&root, event);
    }
    if agent == "cursor" && event == "pre-edit" {
        return cursor_pre_edit(&root);
    }
    if agent == "codex" && event == "pre-edit" {
        return patch_pre_edit(
            &root,
            sj_adapters::codex::targets,
            sj_adapters::codex::deny,
            true,
        );
    }
    if agent == "copilot" && event == "pre-edit" {
        return patch_pre_edit(
            &root,
            sj_adapters::copilot::targets,
            sj_adapters::copilot::deny,
            false,
        );
    }
    if agent == "windsurf" && event == "pre-edit" {
        return exit_code_pre_edit(&root, sj_adapters::windsurf::target);
    }
    if agent == "kiro" && event == "pre-edit" {
        return exit_code_pre_edit(&root, sj_adapters::kiro::target);
    }
    if agent == "antigravity" && event == "pre-edit" {
        return decision_pre_edit(&root, sj_adapters::antigravity::target);
    }
    if agent == "gemini" && event == "pre-edit" {
        return decision_pre_edit(&root, sj_adapters::gemini::target);
    }
    if agent != "claude" {
        eprintln!("specjutsu: hook error: unknown agent `{agent}`");
        return ExitCode::SUCCESS;
    }
    match event {
        "session-start" => match hook::session_start(&root) {
            Ok(text) => print!("{text}"),
            Err(e) => println!("specjutsu: {e:#}"),
        },
        "pre-edit" => {
            let mut payload = String::new();
            let decided = std::io::Read::read_to_string(&mut std::io::stdin(), &mut payload)
                .map_err(anyhow::Error::from)
                .and_then(|_| hook::pre_edit(&root, &payload));
            match decided {
                Ok(Decision::Allow) => {}
                Ok(Decision::Warn(reasons)) => {
                    for r in reasons {
                        println!("specjutsu: {r}");
                    }
                }
                Ok(Decision::Block(reasons)) => {
                    for r in reasons {
                        eprintln!("{r}");
                    }
                    return ExitCode::from(2);
                }
                Err(e) => eprintln!("specjutsu: hook error: {e:#}"),
            }
        }
        _ => eprintln!("specjutsu: hook error: unknown event `{event}`"),
    }
    ExitCode::SUCCESS
}

/// `sj hook cursor pre-edit`: always one JSON answer on stdout, since Cursor blocks on
/// anything else. Unreadable input and internal errors allow, with a warning on stderr.
fn cursor_pre_edit(root: &Path) -> ExitCode {
    use sj_adapters::cursor;
    use sj_core::hook::{self, Decision};
    let mut input = String::new();
    let decided = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
        .map_err(anyhow::Error::from)
        .and_then(|_| cursor::target(&input))
        .and_then(|path| hook::decide(root, &path));
    match decided {
        Ok(Decision::Block(reasons)) => print!("{}", cursor::deny(&reasons)),
        Ok(Decision::Warn(reasons)) => {
            for r in reasons {
                eprintln!("specjutsu: {r}");
            }
            print!("{}", cursor::allow());
        }
        Ok(Decision::Allow) => print!("{}", cursor::allow()),
        Err(e) => {
            eprintln!("specjutsu: hook error: {e:#}");
            print!("{}", cursor::allow());
        }
    }
    ExitCode::SUCCESS
}

/// `sj hook antigravity|gemini pre-edit`: always one JSON decision on stdout, the edited path
/// read by `target`. Unreadable input and internal errors allow, with a warning on stderr.
fn decision_pre_edit(root: &Path, target: fn(&str) -> Result<String>) -> ExitCode {
    use sj_adapters::antigravity;
    use sj_core::hook::{self, Decision};
    let mut input = String::new();
    let decided = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
        .map_err(anyhow::Error::from)
        .and_then(|_| target(&input))
        .and_then(|path| hook::decide(root, &path));
    match decided {
        Ok(Decision::Block(reasons)) => print!("{}", antigravity::deny(&reasons)),
        Ok(Decision::Warn(reasons)) => {
            for r in reasons {
                eprintln!("specjutsu: {r}");
            }
            print!("{}", antigravity::allow());
        }
        Ok(Decision::Allow) => print!("{}", antigravity::allow()),
        Err(e) => {
            eprintln!("specjutsu: hook error: {e:#}");
            print!("{}", antigravity::allow());
        }
    }
    ExitCode::SUCCESS
}

/// `sj hook codex|copilot pre-edit`: print `deny` when any file the call touches is
/// blocked; print nothing to allow. Each reason names its file when `name_files` is set or
/// the call touches several files. Always exits 0. Unreadable input and
/// internal errors allow, with a warning on stderr.
fn patch_pre_edit(
    root: &Path,
    targets: fn(&str) -> Result<Vec<String>>,
    deny: fn(&[String]) -> String,
    name_files: bool,
) -> ExitCode {
    use sj_core::hook::{self, Decision};
    let mut input = String::new();
    let paths = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
        .map_err(anyhow::Error::from)
        .and_then(|_| targets(&input));
    let paths = match paths {
        Ok(p) => p,
        Err(e) => {
            eprintln!("specjutsu: hook error: {e:#}");
            return ExitCode::SUCCESS;
        }
    };
    let mut blocked = Vec::new();
    for path in &paths {
        match hook::decide(root, path) {
            Ok(Decision::Allow) => {}
            Ok(Decision::Warn(reasons)) => {
                for r in reasons {
                    eprintln!("specjutsu: {path}: {r}");
                }
            }
            Ok(Decision::Block(reasons)) => {
                if name_files || paths.len() > 1 {
                    blocked.extend(reasons.into_iter().map(|r| format!("{path}: {r}")));
                } else {
                    blocked.extend(reasons);
                }
            }
            Err(e) => eprintln!("specjutsu: hook error: {path}: {e:#}"),
        }
    }
    if !blocked.is_empty() {
        print!("{}", deny(&blocked));
    }
    ExitCode::SUCCESS
}

/// `sj hook windsurf|kiro pre-edit`: the path read by `target`, then the answer of
/// `sj hook check-edit`. Unreadable input allows, with a warning on stderr.
fn exit_code_pre_edit(root: &Path, target: fn(&str) -> Result<String>) -> ExitCode {
    let mut input = String::new();
    let path = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
        .map_err(anyhow::Error::from)
        .and_then(|_| target(&input));
    match path {
        Ok(path) => check_edit(root, &path),
        Err(e) => {
            eprintln!("specjutsu: hook error: {e:#}");
            ExitCode::SUCCESS
        }
    }
}

/// `sj hook check-edit <path>`: exit 0 to allow, 2 to block, reasons on stderr. Like the
/// agent hooks, an internal error warns and allows.
fn check_edit(root: &Path, path: &str) -> ExitCode {
    use sj_core::hook::{self, Decision};
    match hook::decide(root, path) {
        Ok(Decision::Allow) => {}
        Ok(Decision::Warn(reasons)) => {
            for r in reasons {
                eprintln!("specjutsu: {r}");
            }
        }
        Ok(Decision::Block(reasons)) => {
            for r in reasons {
                eprintln!("{r}");
            }
            return ExitCode::from(2);
        }
        Err(e) => eprintln!("specjutsu: hook error: {e:#}"),
    }
    ExitCode::SUCCESS
}

/// The home folder for `--global`, or an error saying how to set it.
fn home_dir() -> Result<PathBuf> {
    sj_adapters::global::home()
        .ok_or_else(|| anyhow::anyhow!("--global needs HOME (or USERPROFILE) to be set"))
}

/// A path under the home folder as `--global` prints it.
fn home_path(p: &str) -> String {
    format!("~/{p}")
}

fn print_home_changes(changes: &[sj_adapters::claude::Change], none: &str) {
    use sj_adapters::claude::Change;
    if changes.is_empty() {
        println!("{none}");
    }
    for c in changes {
        let c = match c {
            Change::Created(p) => Change::Created(home_path(p)),
            Change::Updated(p) => Change::Updated(home_path(p)),
            Change::Removed(p) => Change::Removed(home_path(p)),
            Change::Kept(p) => Change::Kept(home_path(p)),
        };
        println!("{c}");
    }
}

/// `sj init --agents|--remove-agents <list> --global`: skills only, in the home folders.
fn global_init_cmd(
    root: &Path,
    agents: &[String],
    remove: &[String],
    force: bool,
) -> Result<ExitCode> {
    use sj_adapters::global;
    let home = home_dir()?;
    if !remove.is_empty() {
        sj_adapters::agents::check(remove)?;
        let (remove, _) = sj_adapters::agents::resolve(root, remove);
        print_home_changes(&global::remove(&home, &remove)?, "no skills to remove");
        return Ok(ExitCode::SUCCESS);
    }
    if agents.is_empty() {
        anyhow::bail!("--global needs --agents or --remove-agents");
    }
    sj_adapters::agents::check(agents)?;
    let (agents, _) = sj_adapters::agents::resolve(root, agents);
    let changes = sj_adapters::plan::apply(global::plan(&home, &agents, force)?)?;
    print_home_changes(&changes, "skills up to date");
    Ok(ExitCode::SUCCESS)
}

/// `sj update --global`: refresh the skill folders in the home folder that hold `sj-*`
/// skills. Like `sj update`, a kept hand-edited skill exits 1.
fn global_update_cmd(force: bool, dry_run: bool, json: bool) -> Result<ExitCode> {
    use sj_adapters::claude::Change;
    use sj_adapters::global;
    let home = home_dir()?;
    if !global::has_skills(&home) {
        println!("no global skills installed");
        return Ok(ExitCode::SUCCESS);
    }
    let steps = global::update_plan(&home, force)?;
    if dry_run {
        if json {
            let files: Vec<_> = steps
                .iter()
                .map(|s| serde_json::json!({"path": home_path(s.path()), "action": s.action()}))
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({ "files": files }))?
            );
        } else {
            if steps.is_empty() {
                println!("skills up to date");
            }
            for s in &steps {
                println!("{} {}", s.action(), home_path(s.path()));
            }
        }
        return Ok(ExitCode::SUCCESS);
    }
    let changes = sj_adapters::plan::apply(steps)?;
    print_home_changes(&changes, "skills up to date");
    let kept = changes.iter().any(|c| matches!(c, Change::Kept(_)));
    Ok(if kept {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Install or refresh each named adapter and print what changed. `agents` is resolved:
/// no `auto`, and `agents-md` listed where it is implied.
fn agents_cmd(root: &Path, agents: &[String], force: bool, update: bool) -> Result<ExitCode> {
    use sj_adapters::claude::Change;
    let changes = sj_adapters::plan::apply(sj_adapters::agents::plan(root, agents, force)?)?;
    if changes.is_empty() {
        println!("adapters up to date");
    }
    for c in &changes {
        println!("{c}");
    }
    let kept = changes.iter().any(|c| matches!(c, Change::Kept(_)));
    Ok(if kept && update {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Remove each named adapter. The AGENTS.md block goes when `agents-md` is named, or when
/// an agent that reads it is removed and no Codex or Cursor hook is left that needs it.
fn remove_agents_cmd(root: &Path, agents: &[String]) -> Result<ExitCode> {
    use sj_adapters::{
        agents_md, antigravity, claude, codex, copilot, cursor, gemini, kiro, windsurf,
    };
    sj_adapters::agents::check(agents)?;
    let agents: Vec<String> = if agents.iter().any(|a| a == "auto") {
        sj_adapters::agents::installed(root)
    } else {
        agents.to_vec()
    };
    let mut changes = Vec::new();
    for a in &agents {
        match a.as_str() {
            "claude" => changes.extend(claude::remove(root)?),
            "codex" => changes.extend(codex::remove(root)?),
            "cursor" => changes.extend(cursor::remove(root)?),
            "copilot" => changes.extend(copilot::remove(root)?),
            "gemini" => changes.extend(gemini::remove(root)?),
            "windsurf" => changes.extend(windsurf::remove(root)?),
            "kiro" => changes.extend(kiro::remove(root)?),
            "antigravity" => changes.extend(antigravity::remove(root)?),
            _ => {}
        }
    }
    let readers = agents.iter().any(|a| {
        matches!(
            a.as_str(),
            "codex"
                | "cursor"
                | "cline"
                | "copilot"
                | "gemini"
                | "windsurf"
                | "kiro"
                | "antigravity"
        )
    });
    let still_needed = codex::is_installed(root)
        || cursor::is_installed(root)
        || copilot::is_installed(root)
        || gemini::is_installed(root)
        || windsurf::is_installed(root)
        || kiro::is_installed(root)
        || antigravity::is_installed(root);
    if agents.iter().any(|a| a == "agents-md") || (readers && !still_needed) {
        changes.extend(agents_md::remove(root)?);
    }
    let skill_readers =
        codex::is_installed(root) || gemini::is_installed(root) || antigravity::is_installed(root);
    if !skill_readers && sj_adapters::shared_skills::is_installed(root) {
        changes.extend(sj_adapters::shared_skills::remove(root)?);
    }
    if changes.is_empty() {
        println!("no adapter files to remove");
    }
    for c in &changes {
        println!("{c}");
    }
    Ok(ExitCode::SUCCESS)
}

fn adopt_cmd(root: &Path, dry_run: bool, force: bool) -> Result<ExitCode> {
    use sj_core::config::FILE;

    let file = root.join(FILE);
    if !dry_run && file.exists() && !force {
        anyhow::bail!("{FILE} already exists; pass --force to replace it, or --dry-run to compare");
    }
    let detected = sj_core::adopt::detect(root)?;
    let config = &detected.config;
    if dry_run {
        print!("{}", config.to_toml());
    } else {
        std::fs::write(&file, config.to_toml())?;
        sj_core::config::ignore_ledger(root)?;
        let spec_root = root.join(&config.spec_root);
        for dir in ["specs", "changes"] {
            std::fs::create_dir_all(spec_root.join(dir))?;
        }
        println!(
            "adopt: wrote {FILE} (spec root {}, {})",
            config.spec_root,
            plural(config.packages.len(), "package")
        );
        for p in &config.packages {
            let runners: Vec<_> = p.suites.iter().map(|s| s.runner.as_str()).collect();
            let runners = if runners.is_empty() {
                "-".to_string()
            } else {
                runners.join(", ")
            };
            println!("package {}  {runners}", p.path);
        }
    }
    for n in &detected.notes {
        eprintln!("note    {n}");
    }
    Ok(ExitCode::SUCCESS)
}

/// Load the project file (failing on a bad one) and pick the spec root: the flag, then the file.
fn spec_root_for(root: &Path, flag: Option<PathBuf>) -> Result<PathBuf> {
    let config = sj_core::config::load(root)?;
    Ok(flag.unwrap_or_else(|| PathBuf::from(config.spec_root)))
}

fn test_cmd(
    scope: Option<String>,
    root: PathBuf,
    spec_root: PathBuf,
    json: bool,
    no_ledger: bool,
) -> Result<ExitCode> {
    use sj_core::ledger::{self, Recorded};
    use sj_core::run::{run_tests, Scope};
    use std::fmt::Write as _;

    let run = run_tests(&root, &spec_root, &Scope::parse(scope.as_deref()))?;
    // The ledger's answer settles "in a git repository" for the gain record too.
    let record_gain = !no_ledger
        && match ledger::record(&root, &run) {
            Ok(Recorded::Records(_)) => true,
            Ok(Recorded::NotGit) => {
                eprintln!("warn    not a git repository; run not recorded");
                false
            }
            Err(e) => {
                eprintln!("warn    run not recorded: {e:#}");
                true
            }
        };
    // Built in full first so the bytes printed can be counted for the gain record.
    let mut out = String::new();
    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&run)?)?;
    } else if run.build_failed {
        if run.suites.len() > 1 {
            let names: Vec<&str> = run.suites.iter().map(|s| s.name.as_str()).collect();
            writeln!(out, "test: fail — build failed ({})", names.join(", "))?;
        } else {
            writeln!(out, "test: fail — build failed")?;
        }
        for l in &run.build_errors {
            writeln!(out, "        {l}")?;
        }
    } else {
        let mut counts = Vec::new();
        if run.failed > 0 {
            counts.push(format!("{} failed", run.failed));
        }
        counts.push(format!("{} passed", run.passed));
        if run.ignored > 0 {
            counts.push(format!("{} ignored", run.ignored));
        }
        let verdict = if run.pass { "pass" } else { "fail" };
        let failed: Vec<&str> = run
            .suites
            .iter()
            .filter(|s| !s.pass)
            .map(|s| s.name.as_str())
            .collect();
        let suffix = if run.suites.len() > 1 && !failed.is_empty() {
            format!(" (failed: {})", failed.join(", "))
        } else {
            String::new()
        };
        writeln!(out, "test: {verdict} — {}{suffix}", counts.join(", "))?;
        for t in run
            .tests
            .iter()
            .filter(|t| t.result == sj_core::run::Outcome::Failed)
        {
            if t.ids.is_empty() {
                writeln!(out, "fail    {}", t.path)?;
            } else {
                writeln!(out, "fail    {}  {}", t.path, t.ids.join(", "))?;
            }
            if let Some(ex) = &t.excerpt {
                for l in &ex.lines {
                    writeln!(out, "        {l}")?;
                }
                if ex.omitted > 0 {
                    writeln!(out, "        … {} more", plural(ex.omitted, "line"))?;
                }
            }
        }
        for s in run.suites.iter().filter(|s| !s.build_errors.is_empty()) {
            writeln!(out, "build failed ({})", s.name)?;
            for l in &s.build_errors {
                writeln!(out, "        {l}")?;
            }
        }
    }
    print_and_record_gain(&root, record_gain, "test", &out, run.raw_bytes);
    Ok(if run.pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

/// Print `out` to stdout as is, then, if `record`, record its length against `raw` in the gain
/// records (`sj_core::gain::enabled` decides `record` for commands without a ledger). A record
/// that can't be written warns on stderr and the command carries on.
fn print_and_record_gain(root: &Path, record: bool, command: &str, out: &str, raw: u64) {
    use std::io::Write as _;
    print!("{out}");
    let _ = std::io::stdout().flush();
    if !record {
        return;
    }
    let r = sj_core::gain::Record::now(command, out.len() as u64, raw);
    if let Err(e) = sj_core::gain::append(root, &r) {
        eprintln!("warn    gain not recorded: {e:#}");
    }
}

fn ledger_cmd(id: Option<String>, root: PathBuf, json: bool) -> Result<ExitCode> {
    let loaded = sj_core::ledger::load(&root)?;
    if loaded.skipped > 0 {
        eprintln!(
            "warn    skipped {} unreadable ledger {}",
            loaded.skipped,
            if loaded.skipped == 1 { "line" } else { "lines" }
        );
    }
    let mut report = sj_core::ledger::report(&loaded.records);
    if let Some(id) = &id {
        report.retain(|s| &s.id == id);
    }
    if json {
        let v = serde_json::json!({ "scenarios": report, "skipped": loaded.skipped });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else if let (Some(id), true) = (&id, report.is_empty()) {
        println!("ledger: no records for {id}");
    } else {
        let proven = report.iter().filter(|s| s.proven).count();
        println!(
            "ledger: {}, {proven} proven red-before-green",
            plural(report.len(), "scenario")
        );
        for s in &report {
            let tag = if s.proven { "proven" } else { "open" };
            println!("{tag:<8}{}  latest {}", s.id, s.latest.as_str());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn parse_rfc3339(s: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, String> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map_err(|e| format!("not an RFC 3339 time, e.g. 2026-11-22T11:00:00Z ({e})"))
}

fn gain_cmd(
    root: &Path,
    json: bool,
    since: Option<chrono::DateTime<chrono::FixedOffset>>,
) -> Result<ExitCode> {
    let mut loaded = sj_core::gain::load(root)?;
    if let Some(t) = since {
        loaded = sj_core::gain::since(loaded, t);
    }
    if loaded.skipped > 0 {
        eprintln!(
            "warn    skipped {} unreadable gain {}",
            loaded.skipped,
            if loaded.skipped == 1 {
                "record"
            } else {
                "records"
            }
        );
    }
    let summary = sj_core::gain::summarize(&loaded.records);
    if json {
        let totals = |t: &sj_core::gain::Totals| {
            serde_json::json!({
                "runs": t.runs,
                "printed": t.printed,
                "raw": t.raw,
                "saved_pct": t.saved_percent(),
            })
        };
        let commands: Vec<serde_json::Value> = summary
            .commands
            .iter()
            .map(|(c, t)| {
                let mut v = totals(t);
                v["command"] = c.as_str().into();
                v
            })
            .collect();
        let mut total = totals(&summary.total);
        total["tokens_saved"] = summary.total.tokens_saved().into();
        let report = serde_json::json!({
            "commands": commands,
            "total": total,
            "skipped": loaded.skipped,
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(ExitCode::SUCCESS);
    }
    if loaded.records.is_empty() {
        println!("no gain records yet");
        return Ok(ExitCode::SUCCESS);
    }
    let rows: Vec<(&str, &sj_core::gain::Totals)> = summary
        .commands
        .iter()
        .map(|(c, t)| (c.as_str(), t))
        .chain([("total", &summary.total)])
        .collect();
    let width = |f: fn(&sj_core::gain::Totals) -> String| {
        rows.iter().map(|(_, t)| f(t).len()).max().unwrap_or(0)
    };
    let name_w = rows.iter().map(|(c, _)| c.len()).max().unwrap_or(0);
    let runs_w = width(|t| t.runs.to_string());
    let printed_w = width(|t| t.printed.to_string());
    let raw_w = width(|t| t.raw.to_string());
    let pct_w = width(|t| format!("{}%", t.saved_percent()));
    for (i, (name, t)) in rows.iter().enumerate() {
        let mut line = format!(
            "{name:<name_w$}  {:>runs_w$}  {:>printed_w$}  {:>raw_w$}  {:>pct_w$}",
            t.runs,
            t.printed,
            t.raw,
            format!("{}%", t.saved_percent()),
        );
        if i == rows.len() - 1 {
            line.push_str(&format!("  ~{} tokens saved", t.tokens_saved()));
        }
        println!("{line}");
    }
    Ok(ExitCode::SUCCESS)
}

fn spec_paths(args: &SpecArgs) -> Result<(PathBuf, PathBuf)> {
    let spec_root = spec_root_for(&args.root, args.spec_root.clone())?;
    Ok((args.root.clone(), args.root.join(spec_root)))
}

fn location(p: &sj_core::authoring::validate::Problem, root: &Path) -> String {
    if p.line == 0 {
        format!("{} ({})", p.message, rel(&p.file, root))
    } else {
        format!("{} ({}:{})", p.message, rel(&p.file, root), p.line)
    }
}

fn spec_cmd(command: SpecCommand) -> Result<ExitCode> {
    use sj_core::authoring::{self, archive, check, validate};
    match command {
        SpecCommand::New { change, bug, args } => {
            let (root, spec_root) = spec_paths(&args)?;
            let dir = authoring::new_change(&spec_root, &change, bug)?;
            println!("created {}", rel(&dir, &root));
            for (prefix, n) in sj_core::ids::highest_numbers(&spec_root)? {
                println!("next {prefix}-{:03}", n + 1);
            }
            Ok(ExitCode::SUCCESS)
        }
        SpecCommand::List { args, json } => {
            let (_, spec_root) = spec_paths(&args)?;
            let rows = authoring::list(&spec_root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if rows.is_empty() {
                println!("no open changes");
            } else {
                let width = rows.iter().map(|r| r.name.len()).max().unwrap_or(0);
                for r in &rows {
                    println!(
                        "{:<width$}  {}/{} tasks  {}",
                        r.name,
                        r.tasks_done,
                        r.tasks_total,
                        plural(r.scenarios, "scenario")
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        SpecCommand::Validate {
            change,
            args,
            strict,
            json,
        } => {
            let (root, spec_root) = spec_paths(&args)?;
            let r = validate::validate(&spec_root, change.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                if r.errors.is_empty() && r.warnings.is_empty() {
                    println!("validate: ok");
                } else {
                    println!(
                        "validate: {}, {}",
                        plural(r.errors.len(), "error"),
                        plural(r.warnings.len(), "warning")
                    );
                }
                for p in &r.errors {
                    println!("fail    {}", location(p, &root));
                }
                for p in &r.warnings {
                    println!("warn    {}", location(p, &root));
                }
            }
            let failed = !r.errors.is_empty() || (strict && !r.warnings.is_empty());
            Ok(if failed {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        SpecCommand::Check { change, args, json } => {
            let (_, spec_root) = spec_paths(&args)?;
            let r = check::check(&spec_root, &change)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                if r.is_clean() {
                    println!("check: ok");
                } else {
                    println!("check: {}", plural(r.problems(), "problem"));
                }
                for id in &r.uncovered {
                    println!("uncovered {id}");
                }
                for u in &r.unknown {
                    println!(
                        "unknown {} ({}:{})",
                        u.id,
                        rel(&u.file, Path::new("")),
                        u.line
                    );
                }
                for c in &r.clarify {
                    println!(
                        "clarify {} ({}:{})",
                        c.question,
                        rel(&c.file, Path::new("")),
                        c.line
                    );
                }
                for e in &r.errors {
                    println!("fail    {e}");
                }
                for w in &r.warnings {
                    println!("warn    {w}");
                }
            }
            Ok(if r.is_clean() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        SpecCommand::Show { item, args, json } => show_cmd(&item, &args, json),
        SpecCommand::Infer {
            paths,
            capability,
            write,
            change,
            args,
        } => {
            let (root, spec_root) = spec_paths(&args)?;
            let config = sj_core::config::load(&root)?;
            let run = authoring::infer::dry_run(
                &root,
                &config,
                &spec_root,
                &paths,
                capability.as_deref(),
            )?;
            let (created, drafts) = if write {
                let change = change.unwrap_or_else(|| format!("infer-{}", run.capability));
                let (dir, drafts) = authoring::infer::write(&root, &spec_root, &change, &run)?;
                (Some(dir), drafts)
            } else {
                (None, run.drafts.clone())
            };
            for d in &drafts {
                println!(
                    "{} [keep] {}  ({}:{})",
                    d.id,
                    d.heading,
                    rel(&d.file, Path::new("")),
                    d.line
                );
            }
            let left: Vec<String> = [(run.linked, "linked"), (run.skipped, "skipped")]
                .into_iter()
                .filter(|(n, _)| *n > 0)
                .map(|(n, what)| format!("{n} {what}"))
                .collect();
            let mut count = format!("{} drafted", plural(run.drafts.len(), "test"));
            if !left.is_empty() {
                count.push_str(&format!(", {} left out", left.join(" and ")));
            }
            println!("{count}");
            if let Some(dir) = created {
                println!("created {}", rel(&dir, &root));
            }
            Ok(ExitCode::SUCCESS)
        }
        SpecCommand::Archive {
            changes,
            all_done,
            args,
            force,
        } => {
            let (root, spec_root) = spec_paths(&args)?;
            let opts = archive::Options {
                force,
                date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            };
            let changes = if all_done {
                archive::finished(&spec_root)?
            } else {
                changes
            };
            if changes.is_empty() {
                println!("nothing to archive");
                return Ok(ExitCode::SUCCESS);
            }
            let names: Vec<&str> = changes.iter().map(String::as_str).collect();
            match archive::plan(&spec_root, &names, &opts)? {
                archive::Planned::Refused(reasons) => {
                    println!("archive: refused");
                    for p in &reasons {
                        println!("fail    {}", location(p, &root));
                    }
                    Ok(ExitCode::from(1))
                }
                archive::Planned::Ready(plan) => {
                    for a in archive::apply(plan)? {
                        print_archived(&a, &root);
                    }
                    Ok(ExitCode::SUCCESS)
                }
            }
        }
    }
}

/// The lines for one archived change: each spec it wrote, then where it moved.
fn print_archived(a: &sj_core::authoring::archive::Archived, root: &Path) {
    for s in &a.specs {
        let file = rel(&s.file, root);
        if s.created {
            println!("created {file} (+{})", s.added);
        } else {
            println!(
                "updated {file} (+{} ~{} -{} →{})",
                s.added, s.modified, s.removed, s.renamed
            );
        }
    }
    println!("archived {} → {}", a.change, rel(&a.dest, root));
}

fn import_speckit_cmd(args: &SpecArgs, write: bool, force: bool, json: bool) -> Result<ExitCode> {
    use sj_core::import::speckit::{self, Content};
    let config = sj_core::config::load(&args.root)?;
    let (root, spec_root) = spec_paths(args)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let plan = speckit::plan(&root, &spec_root, &config.id_prefixes, &today)?;
    if write {
        speckit::write(&plan, force)?;
    }
    if json {
        let v = serde_json::json!({
            "features": plan.features,
            "principles": plan.principles,
            "written": write,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(ExitCode::SUCCESS);
    }
    let n = plural(plan.features.len(), "feature");
    if write {
        println!("import: wrote {n}");
    } else {
        println!("import: {n} (dry run; --write converts them)");
    }
    for f in &plan.features {
        println!(
            "{}  {}  {}, {}, {}",
            f.name,
            if f.finished { "finished" } else { "open" },
            plural(f.requirements, "requirement"),
            plural(f.scenarios, "scenario"),
            plural(f.gaps.len(), "gap")
        );
        for o in &f.outputs {
            match &o.content {
                Content::Copy(src) => {
                    println!("  {} → {}", rel(src, &root), rel(&o.path, &root))
                }
                Content::Text(_) => println!("  → {}", rel(&o.path, &root)),
            }
        }
        for g in &f.gaps {
            println!("gap {g} (no scenario)");
        }
    }
    if let Some(p) = &plan.principles {
        if let Content::Copy(src) = &p.content {
            println!("principles {} → {}", rel(src, &root), rel(&p.path, &root));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn migrate_undo_cmd(root: &Path) -> Result<ExitCode> {
    let done = sj_core::migrate::undo(root)?;
    println!(
        "migrate: undone ({} restored, {} deleted; left uncommitted)",
        plural(done.restored, "file"),
        done.deleted
    );
    Ok(ExitCode::SUCCESS)
}

fn migrate_cmd(root: &Path, from: &[String], write: bool, diff: bool) -> Result<ExitCode> {
    use sj_core::migrate::{self, Tool};
    let tools: Vec<Tool> = from.iter().filter_map(|f| Tool::parse(f)).collect();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let plan = migrate::plan(root, &tools, &today)?;
    let names: Vec<&str> = plan.tools.iter().map(|t| t.as_str()).collect();
    if write {
        migrate::apply(root, &plan)?;
        println!("migrate: wrote {}", names.join(", "));
    } else {
        println!(
            "migrate: {} (dry run; --write applies it)",
            names.join(", ")
        );
    }
    for l in &plan.lines {
        println!("{l}");
    }
    if diff {
        print!("{}", migrate::diff(root, &tools, &today)?);
    }
    if !write {
        return Ok(ExitCode::SUCCESS);
    }
    let f = migrate::finish(root, &plan)?;
    if f.doctor.is_empty() {
        println!("doctor  ok");
    }
    for p in &f.doctor {
        println!("doctor  {p}");
    }
    let r = &f.validate;
    if r.errors.is_empty() && r.warnings.is_empty() {
        println!("validate ok");
    } else {
        println!(
            "validate {}, {}",
            plural(r.errors.len(), "error"),
            plural(r.warnings.len(), "warning")
        );
    }
    for p in &r.errors {
        println!("fail    {}", location(p, root));
    }
    for p in &r.warnings {
        println!("warn    {}", location(p, root));
    }
    println!(
        "git: {} created, {} changed, {} removed",
        f.created, f.changed, f.removed
    );
    println!("rollback: sj migrate --undo");
    Ok(if f.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn instructions_cmd(
    change: &str,
    artifact: Option<&str>,
    args: &SpecArgs,
    json: bool,
) -> Result<ExitCode> {
    use sj_core::agent_loop::instructions::{instructions, Outcome};
    let (root, spec_root) = spec_paths(args)?;
    let config = sj_core::config::load(&root)?;
    match instructions(&root, &spec_root, &config, change, artifact)? {
        Outcome::Artifact(i) => {
            for w in &i.warnings {
                eprintln!("warn    {w}");
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&i)?);
            } else {
                print!("{}", i.render());
            }
        }
        Outcome::AllWritten { change } => {
            let message = format!("all artifacts written; run sj next {change}");
            if json {
                let v =
                    serde_json::json!({ "change": change, "artifact": null, "message": message });
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!("{message}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn templates_cmd(args: &SpecArgs, json: bool) -> Result<ExitCode> {
    let (root, spec_root) = spec_paths(args)?;
    let all = sj_core::authoring::templates(&root, &spec_root);
    if json {
        println!("{}", serde_json::to_string_pretty(&all)?);
    } else {
        let width = all.iter().map(|t| t.name.len()).max().unwrap_or(0);
        for t in &all {
            println!("{:<width$}  {}", t.name, t.path);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn status_cmd(change: Option<String>, args: &SpecArgs, json: bool) -> Result<ExitCode> {
    use sj_core::agent_loop::status;
    use std::fmt::Write as _;
    let (root, spec_root) = spec_paths(args)?;
    let changes = status::changes(&spec_root, change.as_deref())?;
    let rows = status::status_of_changes(&root, &changes)?;
    // Built in full first so the bytes printed can be counted against the change folders read.
    let mut out = String::new();
    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&rows)?)?;
    } else if rows.is_empty() {
        writeln!(out, "no open changes")?;
    } else {
        for r in &rows {
            for l in status::render(r) {
                writeln!(out, "{l}")?;
            }
        }
    }
    let raw = changes
        .iter()
        .map(|c| sj_core::gain::dir_bytes(&c.dir))
        .sum();
    let record = sj_core::gain::enabled(&root, false);
    print_and_record_gain(&root, record, "status", &out, raw);
    Ok(ExitCode::SUCCESS)
}

fn next_cmd(
    change: Option<String>,
    args: &SpecArgs,
    parallel: bool,
    base: Option<String>,
    json: bool,
) -> Result<ExitCode> {
    use sj_core::agent_loop::next;
    use std::fmt::Write as _;
    let (root, spec_root) = spec_paths(args)?;
    // Built in full first so the bytes printed can be counted against the change folder read.
    let mut out = String::new();
    let name = if parallel {
        match next::parallel_change(&spec_root, change.as_deref())? {
            None => {
                writeln!(out, "next: no open tasks")?;
                None
            }
            Some(name) => {
                let batches = next::parallel(&root, &spec_root, &name)?;
                if json {
                    let v = serde_json::json!({ "change": name, "batches": batches });
                    writeln!(out, "{}", serde_json::to_string_pretty(&v)?)?;
                } else if batches.is_empty() {
                    writeln!(out, "next: no parallel tasks")?;
                } else {
                    for (i, b) in batches.iter().enumerate() {
                        writeln!(out, "batch {}", i + 1)?;
                        for t in b {
                            writeln!(out, "  {t}")?;
                        }
                    }
                }
                Some(name)
            }
        }
    } else {
        match next::next(&root, &spec_root, change.as_deref(), base.as_deref())? {
            None => {
                writeln!(out, "{}", if json { "null" } else { "next: no open tasks" })?;
                None
            }
            Some(n) => {
                if json {
                    writeln!(out, "{}", serde_json::to_string_pretty(&n)?)?;
                } else {
                    out.push_str(&next::render(&n));
                }
                Some(n.change)
            }
        }
    };
    // Raw is the change folder reported on; with no open task none was, so 0. A failed
    // lookup also counts 0, so recording never changes the output or the exit code.
    let raw = name
        .and_then(|name| sj_core::authoring::find_change(&spec_root, &name).ok())
        .map_or(0, |c| sj_core::gain::dir_bytes(&c.dir));
    let record = sj_core::gain::enabled(&root, false);
    print_and_record_gain(&root, record, "next", &out, raw);
    Ok(ExitCode::SUCCESS)
}

fn verify_cmd(
    change: Option<String>,
    args: &SpecArgs,
    base: Option<String>,
    json: bool,
) -> Result<ExitCode> {
    use sj_core::agent_loop::verify;
    use std::fmt::Write as _;
    let spec_root = spec_root_for(&args.root, args.spec_root.clone())?;
    let r = verify::verify(&verify::Options {
        root: args.root.clone(),
        spec_root,
        change,
        base,
    })?;
    for w in &r.warnings {
        eprintln!("warn    {w}");
    }
    // Built in full first so the bytes printed can be counted for the gain record.
    let mut out = String::new();
    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&r)?)?;
    } else if r.ok {
        writeln!(out, "verify: ok")?;
    } else {
        writeln!(out, "verify: {}", plural(r.problems.len(), "problem"))?;
        for p in &r.problems {
            writeln!(out, "{}", p.message)?;
        }
    }
    // `verify` has already refused to run outside a git repository.
    print_and_record_gain(&args.root, true, "verify", &out, r.raw_bytes);
    Ok(if r.ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn task_done_cmd(change: &str, task: &str, args: &SpecArgs) -> Result<ExitCode> {
    use sj_core::agent_loop::task::{self, Outcome};
    let config = sj_core::config::load(&args.root)?;
    let (root, spec_root) = spec_paths(args)?;
    match task::done(&spec_root, &root, change, task, config.mode)? {
        Outcome::AlreadyDone { text } => println!("task: already done {text}"),
        Outcome::Done {
            text,
            warnings,
            not_property,
            notes,
        } => {
            println!("task: done {text}");
            for n in &notes {
                println!("warn    {n}");
            }
            for u in &warnings {
                println!("warn    unproven {} ({})", u.id, u.why);
            }
            for l in &not_property {
                println!("warn    {l}");
            }
        }
        Outcome::Refused {
            unproven,
            not_property,
            notes,
        } => {
            let mut why = Vec::new();
            if !unproven.is_empty() {
                why.push(format!("{} unproven", unproven.len()));
            }
            if !not_property.is_empty() {
                why.push(format!("{} not-property", not_property.len()));
            }
            println!("task: refused — {}", why.join(", "));
            for n in &notes {
                println!("warn    {n}");
            }
            for u in &unproven {
                println!("unproven {} ({})", u.id, u.why);
            }
            for l in &not_property {
                println!("{l}");
            }
            return Ok(ExitCode::from(1));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn ids_prefixes_cmd(root: &Path, spec_root: Option<PathBuf>, write: bool) -> Result<ExitCode> {
    let path = root.join("specjutsu.toml");
    if write && !path.exists() {
        anyhow::bail!(
            "specjutsu.toml not found; run `sj init` first, then `sj ids prefixes --write`"
        );
    }
    let config = sj_core::config::load(root)?;
    let spec_root = spec_root.unwrap_or_else(|| PathBuf::from(&config.spec_root));
    let table = sj_core::ids::prefix_table(&root.join(&spec_root), &config.id_prefixes)?;
    println!("[ids.prefixes]");
    for (cap, prefix) in &table {
        println!("{cap} = \"{prefix}\"");
    }
    if write {
        let missing: std::collections::BTreeMap<String, String> = table
            .into_iter()
            .filter(|(cap, _)| !config.id_prefixes.contains_key(cap))
            .collect();
        if missing.is_empty() {
            return Ok(ExitCode::SUCCESS);
        }
        let before = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let after = sj_core::ids::add_prefix_lines(&before, &missing);
        // Write only when the result parses and holds exactly the old and new prefixes.
        let mut expected = config.id_prefixes.clone();
        expected.extend(missing.clone());
        let spliced = sj_core::config::Config::parse(&after).map(|c| c.id_prefixes);
        if !matches!(&spliced, Ok(p) if *p == expected) {
            let lines: Vec<String> = missing
                .iter()
                .map(|(cap, prefix)| format!("{cap} = \"{prefix}\""))
                .collect();
            anyhow::bail!(
                "specjutsu.toml: cannot find an `[ids.prefixes]` table to add to; \
                 add these lines to it by hand:\n{}",
                lines.join("\n")
            );
        }
        std::fs::write(&path, after).with_context(|| format!("writing {}", path.display()))?;
        let n = missing.len();
        println!(
            "added {n} {} to specjutsu.toml",
            if n == 1 { "prefix" } else { "prefixes" }
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn ids_cmd(
    write: bool,
    root: &Path,
    spec_root: Option<PathBuf>,
    json: bool,
    targets: &[String],
) -> Result<ExitCode> {
    let config = sj_core::config::load(root)?;
    let spec_root = spec_root.unwrap_or_else(|| PathBuf::from(&config.spec_root));
    let only = if targets.is_empty() {
        None
    } else {
        Some(sj_core::ids::resolve_targets(
            root,
            &root.join(&spec_root),
            targets,
        )?)
    };
    let plan =
        sj_core::ids::plan_for(&root.join(&spec_root), &config.id_prefixes, only.as_deref())?;
    let files = if write {
        sj_core::ids::check_prefixes_free(&root.join(&spec_root), &config.id_prefixes, &plan)?;
        sj_core::ids::write(&plan)?
    } else {
        0
    };
    if json {
        let assignments: Vec<serde_json::Value> = plan
            .iter()
            .map(|a| {
                let mut v = serde_json::json!({
                    "id": a.id,
                    "name": a.name,
                    "file": rel(&a.file, root),
                    "line": a.line,
                });
                // `note` is absent unless the original gets a new number instead of its
                // MODIFIED copies' ID (IDS-018/019). Its `kind` is `taken` (with `copy`, and
                // `by`, the scenario that holds the ID) or `disagree` (with `copies`); each
                // scenario is `{id, name, file, line}`.
                if let Some(note) = &a.note {
                    let scn = |s: &sj_core::ids::Scenario| {
                        serde_json::json!({
                            "id": s.id,
                            "name": s.name,
                            "file": rel(&s.file, root),
                            "line": s.line,
                        })
                    };
                    v["note"] = match note {
                        sj_core::ids::Note::Taken { copy, by } => serde_json::json!({
                            "kind": "taken",
                            "copy": scn(copy),
                            "by": scn(by),
                        }),
                        sj_core::ids::Note::Disagree { copies } => serde_json::json!({
                            "kind": "disagree",
                            "copies": copies.iter().map(scn).collect::<Vec<_>>(),
                        }),
                    };
                }
                v
            })
            .collect();
        let v = serde_json::json!({
            "missing": plan.len(),
            "written": write && !plan.is_empty(),
            "assignments": assignments,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        if plan.len() == 1 {
            println!("1 scenario without an ID");
        } else {
            println!("{} scenarios without IDs", plan.len());
        }
        for a in &plan {
            println!("{} {} ({}:{})", a.id, a.name, rel(&a.file, root), a.line);
        }
        let scn = |s: &sj_core::ids::Scenario| {
            format!("{} {} ({}:{})", s.id, s.name, rel(&s.file, root), s.line)
        };
        for a in &plan {
            let why = match &a.note {
                None => continue,
                Some(sj_core::ids::Note::Taken { copy, by }) => {
                    format!("{} is taken by {}", scn(copy), scn(by))
                }
                Some(sj_core::ids::Note::Disagree { copies }) => format!(
                    "copies disagree: {}",
                    copies.iter().map(scn).collect::<Vec<_>>().join(", ")
                ),
            };
            println!(
                "note: {why}; {} ({}:{}) gets {}; give the delta copies {}",
                a.name,
                rel(&a.file, root),
                a.line,
                a.id,
                a.id
            );
        }
        if !write {
            // With targets, clashes outside the planned capabilities don't matter (IDS-021),
            // and the ones that do are reported without stopping the listing.
            let long = sj_core::ids::long_prefixes(
                &root.join(&spec_root),
                &config.id_prefixes,
                &plan,
                only.is_some(),
            )?;
            for (cap, long, short) in &long.hints {
                println!(
                    "hint: {cap} would use {long}; sj ids prefixes --write maps it to {short}"
                );
            }
            for c in &long.clashes {
                println!("note: {c}");
            }
        }
        if write && !plan.is_empty() {
            println!(
                "wrote {} to {}",
                if plan.len() == 1 {
                    "1 ID".to_string()
                } else {
                    format!("{} IDs", plan.len())
                },
                plural(files, "file")
            );
        }
    }
    Ok(if write || plan.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
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

/// Per gap and per orphan of a report, whether the baseline lists it (all false without one),
/// and the baseline entries the report no longer has, rendered.
struct Known {
    gaps: Vec<bool>,
    orphans: Vec<bool>,
    stale: Vec<String>,
}

impl Known {
    fn new(r: &sj_core::trace::Report, baseline: Option<&sj_core::trace::Baseline>) -> Self {
        Known {
            gaps: r
                .gaps
                .iter()
                .map(|g| baseline.is_some_and(|b| b.knows_gap(g)))
                .collect(),
            orphans: r
                .orphans
                .iter()
                .map(|o| baseline.is_some_and(|b| b.knows_orphan(o)))
                .collect(),
            stale: baseline.map_or_else(Vec::new, |b| {
                b.stale(r).iter().map(|e| e.to_string()).collect()
            }),
        }
    }

    /// A gap or orphan the baseline doesn't list, or a stale baseline entry.
    fn fails_strict(&self) -> bool {
        self.gaps.iter().chain(&self.orphans).any(|k| !k) || !self.stale.is_empty()
    }
}

/// A report item with its `known` flag appended, its own fields kept in order.
#[derive(serde::Serialize)]
struct Flagged<'a, T> {
    #[serde(flatten)]
    item: &'a T,
    known: bool,
}

/// `Report` as `--json` prints it when a baseline exists: same fields, same order, then the
/// stale baseline entries.
#[derive(serde::Serialize)]
struct KnownReport<'a> {
    mapped: &'a [sj_core::trace::Mapped],
    gaps: Vec<Flagged<'a, sj_core::spec::Scenario>>,
    orphans: Vec<Flagged<'a, sj_core::trace::Orphan>>,
    unidentified: &'a [sj_core::spec::Scenario],
    not_property: &'a [String],
    warnings: &'a [String],
    stale: &'a [String],
}

impl<'a> KnownReport<'a> {
    fn new(r: &'a sj_core::trace::Report, known: &'a Known) -> Self {
        fn flag<'a, T>(items: &'a [T], flags: &[bool]) -> Vec<Flagged<'a, T>> {
            items
                .iter()
                .zip(flags)
                .map(|(item, &known)| Flagged { item, known })
                .collect()
        }
        KnownReport {
            mapped: &r.mapped,
            gaps: flag(&r.gaps, &known.gaps),
            orphans: flag(&r.orphans, &known.orphans),
            unidentified: &r.unidentified,
            not_property: &r.not_property,
            warnings: &r.warnings,
            stale: &known.stale,
        }
    }
}

fn print_compact(r: &sj_core::trace::Report, root: &Path, known: &Known, has_baseline: bool) {
    let count = |flags: &[bool], word: &str| {
        let total = plural(flags.len(), word);
        if has_baseline {
            format!("{total} ({} known)", flags.iter().filter(|k| **k).count())
        } else {
            total
        }
    };
    println!(
        "trace: {} mapped, {}, {}, {} without ID",
        r.mapped.len(),
        count(&known.gaps, "gap"),
        count(&known.orphans, "orphan"),
        r.unidentified.len()
    );
    for w in &r.warnings {
        println!("warn    {w}");
    }
    for (g, _) in r.gaps.iter().zip(&known.gaps).filter(|(_, k)| !**k) {
        println!(
            "gap     {} {}  ({}:{})",
            g.id.as_deref().unwrap_or_default(),
            g.name,
            rel(&g.file, root),
            g.line
        );
    }
    for (o, _) in r.orphans.iter().zip(&known.orphans).filter(|(_, k)| !**k) {
        println!(
            "orphan  {} {}  ({}:{})",
            o.id,
            o.test.name,
            rel(&o.test.file, root),
            o.test.line
        );
    }
    for (id, tests) in r.not_property_tests() {
        let names: Vec<&str> = tests.iter().map(|t| t.name.as_str()).collect();
        println!("not-property {id} {}", names.join(" "));
    }
    for e in &known.stale {
        println!("stale {e} (remove from {})", sj_core::trace::BASELINE_FILE);
    }
    for u in &r.unidentified {
        println!("no-id   {}  ({}:{})", u.name, rel(&u.file, root), u.line);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum GateFormat {
    Text,
    Json,
    Markdown,
}

#[derive(Subcommand)]
enum CoverageCommand {
    /// Run each scenario's tests under its suite's `coverage` command and record the lines
    /// they run in .specjutsu/coverage-map.json.
    Map {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        spec_root: Option<PathBuf>,
    },
}

fn show_cmd(item: &str, args: &SpecArgs, json: bool) -> Result<ExitCode> {
    use sj_core::authoring::show::{self, Outcome};

    let (root, spec_root) = spec_paths(args)?;
    let config = sj_core::config::load(&root)?;
    match show::show(&root, &spec_root, &config, item)? {
        Outcome::Found(found) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&found)?);
            } else {
                print!("{}", show::render(&found));
            }
            Ok(ExitCode::SUCCESS)
        }
        Outcome::Ambiguous(folders) => {
            eprintln!("sj: `{item}` names several archived changes; use a folder name:");
            for f in folders {
                eprintln!("  {f}");
            }
            Ok(ExitCode::from(1))
        }
        Outcome::Unknown(near) => {
            eprintln!("sj: no scenario, change or capability named `{item}`");
            if !near.is_empty() {
                eprintln!("did you mean: {}", near.join(", "));
            }
            Ok(ExitCode::from(1))
        }
    }
}

fn view_cmd(args: &SpecArgs, json: bool) -> Result<ExitCode> {
    let (root, spec_root) = spec_paths(args)?;
    let config = sj_core::config::load(&root)?;
    let v = sj_core::agent_loop::view::view(&root, &spec_root, &config)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        for line in sj_core::agent_loop::view::render(&v) {
            println!("{line}");
        }
    }
    Ok(ExitCode::SUCCESS)
}
