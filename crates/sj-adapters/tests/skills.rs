//! The static skill check (specjutsu/changes/workflow-skills).

use std::path::Path;

use sj_adapters::skills::{self, Problem};

const SKILLS: [&str; 10] = [
    "sj-explore",
    "sj-propose",
    "sj-clarify",
    "sj-principles",
    "sj-apply",
    "sj-debug",
    "sj-review",
    "sj-verify",
    "sj-finish",
    "sj-onboard",
];

const COVERAGE: &str = "| Tool | Item | Skill | Depth |
| --- | --- | --- | --- |
| Superpowers | brainstorming | sj-explore | brainstorming.md |
| Superpowers | writing-plans | sj-propose | SKILL.md |
| Superpowers | executing-plans | sj-apply | SKILL.md |
| Superpowers | test-driven-development | sj-apply | writing-good-tests.md |
| Superpowers | subagent-driven-development | sj-apply | task-reviewer-prompt.md |
| Superpowers | dispatching-parallel-agents | sj-apply | SKILL.md |
| Superpowers | systematic-debugging | sj-debug | root-cause-tracing.md |
| Superpowers | requesting-code-review | sj-review | SKILL.md |
| Superpowers | receiving-code-review | sj-review | SKILL.md |
| Superpowers | verification-before-completion | sj-verify | SKILL.md |
| Superpowers | finishing-a-development-branch | sj-finish | SKILL.md |
| Superpowers | using-git-worktrees | sj-finish | SKILL.md |
| OpenSpec | explore | sj-explore |
| OpenSpec | propose | sj-propose |
| OpenSpec | apply | sj-apply |
| OpenSpec | archive | sj-finish |
| OpenSpec | sync | sj-finish |
| OpenSpec | update | sj-propose |
| OpenSpec | onboard | sj-onboard |
| Spec Kit | constitution | sj-principles |
| Spec Kit | specify | sj-propose |
| Spec Kit | clarify | sj-clarify |
| Spec Kit | plan | sj-propose |
| Spec Kit | tasks | sj-propose |
| Spec Kit | analyze | sj-propose |
| Spec Kit | checklist | sj-review |
| Spec Kit | implement | sj-apply |
";

/// An onboarding body that satisfies SKILL-005..007.
const ONBOARD: &str = "Stop if `git status` shows uncommitted changes or \
`changes/sj-onboard-demo/` exists. Create the change `sj-onboard-demo` with `sj spec new`, \
run `sj test DEMO-001` red, then green, then `sj task done`, then `sj verify`. At the end, \
delete `changes/sj-onboard-demo/`, the demo test and the demo code. Don't archive it.";

/// An sj-apply body that satisfies SKILL-003 and SKILL-011.
const APPLY: &str = "Run `sj next` and give it to an implementer with `implementer-prompt.md`. \
Then send the task to a fresh reviewer subagent with `task-reviewer-prompt.md`. Findings go \
back to the implementer and then to a fresh re-reviewer with `re-review-prompt.md`, for at \
most 3 rounds. After approval, run `sj test AUTH-001` and `sj task done` yourself.";

/// An implementer prompt that satisfies SKILL-013.
const IMPLEMENTER: &str = "Run `sj test` red, then green. Never run `sj task done`. Report \
DONE, DONE_WITH_CONCERNS, BLOCKED or NEEDS_CONTEXT.\n";

/// A task-reviewer prompt that satisfies SKILL-012.
const REVIEWER: &str = "Read the scenario text. Don't trust the implementer's report. Run \
`git diff` and `sj verify`. Say whether the test would fail without the change. Answer \
APPROVED or CHANGES_REQUESTED.\n";

/// An sj-explore body that satisfies SKILL-019.
const EXPLORE: &str = "Run `sj status`. Go wide before narrowing: list many ideas, then cut \
them down. Use `brainstorming.md`.";

/// A brainstorming reference that satisfies SKILL-020.
const BRAINSTORMING: &str = "Present the design one section at a time. Run a YAGNI pass.\n";

/// The sections every discipline skill has (SKILL-014).
const DISCIPLINE: &str = "\n## Rationalizations\n\nNone.\n\n## Red flags\n\nNone.\n";

/// A body that satisfies every per-skill rule.
fn body(name: &str) -> String {
    let needs = match name {
        "sj-apply" => &format!("{APPLY} Read `writing-good-tests.md`.\n{DISCIPLINE}"),
        "sj-debug" => &format!(
            "Reproduce it with `sj test AUTH-001`. Trace it with `root-cause-tracing.md`, \
             guard it with `defense-in-depth.md`, and wait with `condition-based-waiting.md`. \
             After 3 failed fixes, question the design.\n{DISCIPLINE}"
        ),
        "sj-verify" => &format!(
            "Run `sj verify`. A subagent's report is not evidence.\n{DISCIPLINE}"
        ),
        "sj-finish" => "Run `sj verify`, then `sj spec archive`.",
        "sj-propose" => "Run `sj spec new`, then `sj spec validate` and `sj spec check`. Read principles.md and record each deviation in design.md.",
        "sj-clarify" => "Run `sj spec check`.",
        "sj-review" => "Read principles.md and record each deviation in design.md.",
        "sj-onboard" => ONBOARD,
        "sj-explore" => EXPLORE,
        _ => "Run `sj status`.",
    };
    format!("# {name}\n\n{needs}\n")
}

fn skill_md(name: &str, body: &str) -> String {
    let when = if name == "sj-explore" {
        "Use when brainstorming or testing the check."
    } else {
        "Use when testing the check."
    };
    format!("---\nname: {name}\ndescription: {when}\n---\n\n{body}")
}

fn put(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A skills folder that checks clean.
fn valid() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for s in SKILLS {
        put(dir.path(), &format!("{s}/SKILL.md"), &skill_md(s, &body(s)));
    }
    put(dir.path(), "sj-apply/implementer-prompt.md", IMPLEMENTER);
    put(dir.path(), "sj-apply/task-reviewer-prompt.md", REVIEWER);
    put(
        dir.path(),
        "sj-apply/re-review-prompt.md",
        "Check each finding.\n",
    );
    put(
        dir.path(),
        "sj-apply/writing-good-tests.md",
        "# Good tests\n",
    );
    for f in [
        "root-cause-tracing.md",
        "defense-in-depth.md",
        "condition-based-waiting.md",
    ] {
        put(dir.path(), &format!("sj-debug/{f}"), "# Technique\n");
    }
    put(dir.path(), "sj-explore/brainstorming.md", BRAINSTORMING);
    put(dir.path(), "COVERAGE.md", COVERAGE);
    dir
}

fn messages(dir: &Path) -> Vec<String> {
    skills::check(dir)
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect()
}

#[test]
fn skill_001_a_complete_fixture_checks_clean() {
    let dir = valid();
    assert_eq!(messages(dir.path()), Vec::<String>::new());
}

#[test]
fn skill_001_coverage_is_complete() {
    let dir = valid();
    let coverage = COVERAGE
        .replace(
            "| Superpowers | brainstorming | sj-explore | brainstorming.md |\n",
            "",
        )
        .replace("| OpenSpec | onboard | sj-onboard |\n", "")
        .replace(
            "| OpenSpec | sync | sj-finish |",
            "| OpenSpec | sync | sj-finish |\n| OpenSpec | sync | sj-apply |",
        )
        .replace(
            "| Spec Kit | plan | sj-propose |",
            "| Spec Kit | plan | sj-plan |",
        );
    put(dir.path(), "COVERAGE.md", &coverage);
    let got = messages(dir.path());
    for want in [
        "coverage: Superpowers brainstorming is not mapped",
        "coverage: OpenSpec onboard is not mapped",
        "coverage: OpenSpec sync is mapped twice",
        "coverage: Spec Kit plan maps to unknown skill sj-plan",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}

#[test]
fn skill_001_a_missing_skill_or_coverage_file_fails() {
    let dir = valid();
    std::fs::remove_dir_all(dir.path().join("sj-clarify")).unwrap();
    std::fs::remove_dir_all(dir.path().join("sj-onboard")).unwrap();
    std::fs::remove_file(dir.path().join("COVERAGE.md")).unwrap();
    let got = messages(dir.path());
    assert!(
        got.iter().any(|m| m == "sj-onboard: missing SKILL.md"),
        "{got:?}"
    );
    assert!(
        got.iter().any(|m| m == "sj-clarify: missing SKILL.md"),
        "{got:?}"
    );
    assert!(
        got.iter().any(|m| m == "coverage: missing COVERAGE.md"),
        "{got:?}"
    );
}

#[test]
fn skill_002_an_oversized_or_foreign_skill_fails_the_check() {
    let dir = valid();
    let big = format!("{}{}", body("sj-propose"), "x".repeat(7000));
    put(
        dir.path(),
        "sj-propose/SKILL.md",
        &skill_md("sj-propose", &big),
    );
    put(
        dir.path(),
        "sj-apply/SKILL.md",
        &skill_md(
            "sj-apply",
            &format!("{}\nThen run `openspec status`.\n", body("sj-apply")),
        ),
    );
    let got = messages(dir.path());
    let n = big.len();
    assert!(
        got.iter()
            .any(|m| m == &format!("sj-propose: body is {n} bytes (limit 6000)")),
        "{got:?}"
    );
    assert!(
        got.iter()
            .any(|m| m == "sj-apply: names a foreign command (openspec status)"),
        "{got:?}"
    );
}

#[test]
fn skill_002_other_foreign_commands_and_bad_frontmatter() {
    let dir = valid();
    put(
        dir.path(),
        "sj-explore/SKILL.md",
        &skill_md(
            "sj-explore",
            "Use /speckit.plan, then superpowers:brainstorming. Specs use OpenSpec's format.\n",
        ),
    );
    put(
        dir.path(),
        "sj-debug/SKILL.md",
        "---\nname: debug\ndescription: Debugging.\n---\n\nRun `sj test`.\n",
    );
    put(dir.path(), "sj-verify/SKILL.md", "Run `sj verify`.\n");
    let got = messages(dir.path());
    for want in [
        "sj-explore: names a foreign command (/speckit)",
        "sj-explore: names a foreign command (superpowers:)",
        "sj-debug: name is debug, expected sj-debug",
        "sj-debug: description does not say when to use it",
        "sj-verify: no frontmatter",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
    // Naming the format is fine; only its commands are foreign.
    assert!(!got.iter().any(|m| m.contains("(OpenSpec")), "{got:?}");
}

#[test]
fn skill_003_each_skill_names_its_required_commands() {
    let dir = valid();
    put(
        dir.path(),
        "sj-apply/SKILL.md",
        &skill_md("sj-apply", "Run `sj next` and `sj test AUTH-001`.\n"),
    );
    put(
        dir.path(),
        "sj-finish/SKILL.md",
        &skill_md("sj-finish", "Open the PR.\n"),
    );
    let got = messages(dir.path());
    for want in [
        "sj-apply: does not name `sj task done`",
        "sj-finish: does not name `sj verify`",
        "sj-finish: does not name `sj spec archive`",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
    assert!(
        !got.iter().any(|m| m == "sj-apply: does not name `sj next`"),
        "{got:?}"
    );
}

#[test]
fn skill_004_planning_and_review_reference_the_principles_file() {
    let dir = valid();
    put(
        dir.path(),
        "sj-review/SKILL.md",
        &skill_md("sj-review", "Review the diff.\n"),
    );
    put(
        dir.path(),
        "sj-propose/SKILL.md",
        &skill_md(
            "sj-propose",
            "Run `sj spec new`, `sj spec validate` and `sj spec check`. Read principles.md.\n",
        ),
    );
    let got = messages(dir.path());
    for want in [
        "sj-review: does not name principles.md",
        "sj-review: does not record deviations in design.md",
        "sj-propose: does not record deviations in design.md",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}

#[test]
fn skill_001_this_repos_skills_check_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    for s in SKILLS {
        assert!(dir.join(s).join("SKILL.md").is_file(), "{s} is not shipped");
    }
    assert_eq!(messages(&dir), Vec::<String>::new());
}

#[test]
fn skill_002_this_repos_skill_bodies_are_within_the_limit() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    for s in SKILLS {
        let text = std::fs::read_to_string(dir.join(s).join("SKILL.md")).unwrap();
        assert!(text.len() <= 6000 + 400, "{s}: {} bytes", text.len());
    }
}

#[test]
fn skill_001_openspec_onboard_maps_to_sj_onboard() {
    let dir = valid();
    let coverage = COVERAGE.replace("| OpenSpec | onboard | sj-onboard |\n", "");
    put(dir.path(), "COVERAGE.md", &coverage);
    std::fs::remove_dir_all(dir.path().join("sj-onboard")).unwrap();
    let got = messages(dir.path());
    for want in [
        "coverage: OpenSpec onboard is not mapped",
        "sj-onboard: missing SKILL.md",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}

fn onboard(body: &str) -> Vec<String> {
    let dir = valid();
    put(
        dir.path(),
        "sj-onboard/SKILL.md",
        &skill_md("sj-onboard", body),
    );
    messages(dir.path())
}

// spec: SKILL-005
#[test]
fn skill_005_onboard_names_the_full_loop() {
    let got = onboard("Run `sj verify`, then `sj spec new`, then `sj test`.\n");
    for want in [
        "sj-onboard: does not name `sj task done`",
        "sj-onboard: names `sj verify` before `sj spec new`",
        "sj-onboard: does not name the scratch change sj-onboard-demo",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
    assert!(!onboard(ONBOARD).iter().any(|m| m.starts_with("sj-onboard")));
}

// spec: SKILL-006
#[test]
fn skill_006_onboard_cleans_up_after_itself() {
    let got = onboard(&ONBOARD.replace(
        "At the end, delete `changes/sj-onboard-demo/`, the demo test and the demo code. Don't archive it.",
        "At the end, run `sj spec archive sj-onboard-demo`.",
    ));
    for want in [
        "sj-onboard: does not delete changes/sj-onboard-demo/, the demo test and the demo code",
        "sj-onboard: runs `sj spec archive`",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
}

// spec: SKILL-007
#[test]
fn skill_007_onboard_refuses_a_dirty_start() {
    let got = onboard(&ONBOARD.replace(
        "Stop if `git status` shows uncommitted changes or `changes/sj-onboard-demo/` exists. ",
        "",
    ));
    assert!(
        got.iter().any(|m| m
            == "sj-onboard: does not stop on uncommitted changes or an existing sj-onboard-demo"),
        "{got:?}"
    );
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let text = std::fs::read_to_string(dir.join("sj-onboard/SKILL.md")).unwrap();
    assert!(text.len() <= 6000 + 400, "{} bytes", text.len());
    assert_eq!(messages(&dir), Vec::<String>::new());
}

/// A setup skill body that satisfies SKILL-008 and SKILL-009.
const SETUP: &str = "Run `sj --version`. If it is missing, ask the user, then run `install.sh`, \
or `cargo install specjutsu` on Windows. Keep an existing `specjutsu.toml`. Run `sj adopt` on \
an existing project, then `sj init --agents auto`, then `sj doctor` and `sj status`.";

fn setup(body: &str) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "specjutsu/SKILL.md",
        &skill_md("specjutsu", body),
    );
    skills::check_setup(dir.path())
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect()
}

#[test]
fn skill_008_the_setup_skill_names_the_setup_steps_in_order() {
    assert_eq!(setup(SETUP), Vec::<String>::new());
    let got = setup(&SETUP.replace("`sj doctor` and ", ""));
    assert!(
        got.contains(&"specjutsu: does not name `sj doctor`".to_string()),
        "{got:?}"
    );
    let swapped = "Ask the user. Keep `specjutsu.toml`. Run `sj --version`, `install.sh`, \
`cargo install specjutsu`, `sj init --agents auto`, then `sj adopt`, `sj doctor`, `sj status`.";
    let got = setup(swapped);
    assert!(
        got.contains(&"specjutsu: names `sj init --agents` before `sj adopt`".to_string()),
        "{got:?}"
    );
    let got = setup(&SETUP.replace("`cargo install specjutsu`", "a build from source"));
    assert!(
        got.contains(&"specjutsu: does not name `cargo install specjutsu`".to_string()),
        "{got:?}"
    );
    let dir = tempfile::tempdir().unwrap();
    let got: Vec<String> = skills::check_setup(dir.path())
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect();
    assert_eq!(got, vec!["specjutsu: missing SKILL.md".to_string()]);
}

#[test]
fn skill_009_the_setup_skill_never_forces() {
    let got = setup(&format!("{SETUP} If that fails, run `sj init --force`."));
    assert!(
        got.contains(&"specjutsu: passes `--force`".to_string()),
        "{got:?}"
    );
    let got = setup(
        &SETUP
            .replace("ask the user, then ", "")
            .replace("Keep an existing `specjutsu.toml`. ", ""),
    );
    assert!(
        got.contains(&"specjutsu: does not ask before installing".to_string()),
        "{got:?}"
    );
    assert!(
        got.contains(&"specjutsu: does not keep an existing specjutsu.toml".to_string()),
        "{got:?}"
    );
}

#[test]
fn skill_008_this_repos_setup_skill_checks_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got: Vec<String> = skills::check_setup(&dir)
        .unwrap()
        .iter()
        .map(Problem::to_string)
        .collect();
    assert_eq!(got, Vec::<String>::new());
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/skills/specjutsu");
    assert!(!assets.exists(), "the setup skill must not be embedded");
}

// spec: SKILL-010
#[test]
fn skill_010_a_missing_unnamed_or_oversized_reference_fails_the_check() {
    let dir = valid();
    put(
        dir.path(),
        "sj-debug/SKILL.md",
        &skill_md(
            "sj-debug",
            &format!(
                "{}\nTrace it back with `root-cause-tracing.md`, then add checks from \
                 [defense in depth](defense-in-depth.md). Record it in design.md.\n",
                body("sj-debug")
            ),
        ),
    );
    std::fs::remove_file(dir.path().join("sj-debug/root-cause-tracing.md")).unwrap();
    put(dir.path(), "sj-debug/notes.md", "# Notes\n");
    put(
        dir.path(),
        "sj-debug/defense-in-depth.md",
        &"x".repeat(9000),
    );
    let got = messages(dir.path());
    for want in [
        "sj-debug: names missing reference root-cause-tracing.md",
        "sj-debug: reference notes.md is not named in SKILL.md",
        "sj-debug: reference defense-in-depth.md is 9000 bytes (limit 8000)",
    ] {
        assert!(got.iter().any(|m| m == want), "{want}: {got:?}");
    }
    assert!(
        !got.iter().any(|m| m.contains("design.md")),
        "project files are not references: {got:?}"
    );
}

/// Not linked to a scenario: it passes on the base branch too, and only guards SKILL-010's
/// rule against false positives.
#[test]
fn named_references_within_the_limit_check_clean() {
    let dir = valid();
    put(
        dir.path(),
        "sj-debug/SKILL.md",
        &skill_md(
            "sj-debug",
            &format!("{}\nSee `root-cause-tracing.md`.\n", body("sj-debug")),
        ),
    );
    put(dir.path(), "sj-debug/root-cause-tracing.md", "# Tracing\n");
    assert_eq!(messages(dir.path()), Vec::<String>::new());
}

// spec: SKILL-002
#[test]
fn skill_002_a_foreign_command_in_a_reference_fails_the_check() {
    let dir = valid();
    put(
        dir.path(),
        "sj-debug/SKILL.md",
        &skill_md(
            "sj-debug",
            &format!("{}\nSee `root-cause-tracing.md`.\n", body("sj-debug")),
        ),
    );
    put(
        dir.path(),
        "sj-debug/root-cause-tracing.md",
        "Then run `openspec status`.\n",
    );
    let got = messages(dir.path());
    assert!(
        got.iter()
            .any(|m| m
                == "sj-debug: root-cause-tracing.md names a foreign command (openspec status)"),
        "{got:?}"
    );
}

fn has(got: &[String], want: &str) -> bool {
    got.iter().any(|m| m == want)
}

// spec: SKILL-011
#[test]
fn skill_011_sj_apply_names_the_per_task_review_loop() {
    let dir = valid();
    std::fs::remove_file(dir.path().join("sj-apply/task-reviewer-prompt.md")).unwrap();
    std::fs::remove_file(dir.path().join("sj-apply/re-review-prompt.md")).unwrap();
    put(
        dir.path(),
        "sj-apply/SKILL.md",
        &skill_md(
            "sj-apply",
            "Run `sj next`, give it to an implementer with `implementer-prompt.md`, then \
             review each diff yourself. Run `sj test AUTH-001`, then `sj task done`.\n",
        ),
    );
    let got = messages(dir.path());
    for want in [
        "sj-apply: does not name task-reviewer-prompt.md",
        "sj-apply: does not name re-review-prompt.md",
        "sj-apply: does not send each task to a fresh reviewer subagent",
        "sj-apply: does not limit the fix loop to 3 rounds",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got = messages(&shipped);
    assert!(!got.iter().any(|m| m.starts_with("sj-apply")), "{got:?}");
    assert!(shipped.join("sj-apply/task-reviewer-prompt.md").is_file());
    assert!(shipped.join("sj-apply/re-review-prompt.md").is_file());
}

/// Not linked to a scenario: it passes on the base branch too, and only guards SKILL-011's
/// phrase matching against line breaks and case.
#[test]
fn review_phrases_match_across_line_breaks_and_case() {
    let dir = valid();
    put(
        dir.path(),
        "sj-apply/SKILL.md",
        &skill_md(
            "sj-apply",
            &body("sj-apply").replace("fresh reviewer subagent", "Fresh\n  reviewer\nsubagent"),
        ),
    );
    assert_eq!(messages(dir.path()), Vec::<String>::new());
}

// spec: SKILL-012
#[test]
fn skill_012_the_task_reviewer_works_from_evidence() {
    let dir = valid();
    put(
        dir.path(),
        "sj-apply/task-reviewer-prompt.md",
        "Read the implementer's summary and check `git diff`. Answer APPROVED or \
         CHANGES_REQUESTED, and say whether the test would fail without the change.\n",
    );
    let got = messages(dir.path());
    for want in [
        "sj-apply: task-reviewer-prompt.md does not name sj verify",
        "sj-apply: task-reviewer-prompt.md does not say to ignore the implementer's report",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
    put(
        dir.path(),
        "sj-apply/task-reviewer-prompt.md",
        &REVIEWER
            .replace("APPROVED or CHANGES_REQUESTED", "yes or no")
            .replace("Read the scenario text. ", ""),
    );
    let got = messages(dir.path());
    for want in [
        "sj-apply: task-reviewer-prompt.md does not name APPROVED",
        "sj-apply: task-reviewer-prompt.md does not name CHANGES_REQUESTED",
        "sj-apply: task-reviewer-prompt.md does not name the scenario text",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got = messages(&shipped);
    assert!(
        !got.iter()
            .any(|m| m.starts_with("sj-apply: task-reviewer-prompt.md")),
        "{got:?}"
    );
}

// spec: SKILL-013
#[test]
fn skill_013_the_implementer_reports_a_status_and_never_closes_the_task() {
    let dir = valid();
    put(
        dir.path(),
        "sj-apply/implementer-prompt.md",
        &IMPLEMENTER.replace("Never run `sj task done`.", "Then run `sj task done`."),
    );
    let got = messages(dir.path());
    assert!(
        has(&got, "sj-apply: implementer-prompt.md runs sj task done"),
        "{got:?}"
    );
    put(
        dir.path(),
        "sj-apply/implementer-prompt.md",
        &IMPLEMENTER.replace("DONE, DONE_WITH", "DONE_WITH"),
    );
    let got = messages(dir.path());
    assert!(
        has(&got, "sj-apply: implementer-prompt.md does not name DONE"),
        "the `done` in `sj task done` is not the DONE status: {got:?}"
    );
    put(
        dir.path(),
        "sj-apply/implementer-prompt.md",
        "Run `sj test`.\n",
    );
    let got = messages(dir.path());
    assert!(
        has(
            &got,
            "sj-apply: implementer-prompt.md does not name NEEDS_CONTEXT"
        ),
        "{got:?}"
    );
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/sj-apply");
    let text = std::fs::read_to_string(shipped.join("implementer-prompt.md")).unwrap();
    for want in [
        "sj test",
        "DONE",
        "DONE_WITH_CONCERNS",
        "BLOCKED",
        "NEEDS_CONTEXT",
    ] {
        assert!(
            text.contains(want),
            "shipped implementer prompt lacks {want}"
        );
    }
}

fn with_body(dir: &Path, name: &str, body: &str) {
    put(dir, &format!("{name}/SKILL.md"), &skill_md(name, body));
}

// spec: SKILL-014
#[test]
fn skill_014_a_discipline_skill_without_its_sections_fails_the_check() {
    let dir = valid();
    with_body(
        dir.path(),
        "sj-debug",
        &body("sj-debug").replace("## Red flags", "## Flags"),
    );
    with_body(
        dir.path(),
        "sj-verify",
        &body("sj-verify").replace("## Rationalizations", "## Excuses"),
    );
    with_body(
        dir.path(),
        "sj-apply",
        &body("sj-apply").replace(" Read `writing-good-tests.md`.", ""),
    );
    std::fs::remove_file(dir.path().join("sj-apply/writing-good-tests.md")).unwrap();
    let got = messages(dir.path());
    for want in [
        "sj-debug: has no ## Red flags section",
        "sj-verify: has no ## Rationalizations section",
        "sj-apply: does not name writing-good-tests.md",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got = messages(&shipped);
    assert!(
        !got.iter()
            .any(|m| m.contains("section") || m.contains("writing-good-tests")),
        "{got:?}"
    );
}

// spec: SKILL-015
#[test]
fn skill_015_sj_debug_names_its_techniques_and_the_three_fix_rule() {
    let dir = valid();
    with_body(
        dir.path(),
        "sj-debug",
        &body("sj-debug")
            .replace(", and wait with `condition-based-waiting.md`", "")
            .replace("After 3 failed fixes, question the design.", "Keep trying."),
    );
    std::fs::remove_file(dir.path().join("sj-debug/condition-based-waiting.md")).unwrap();
    let got = messages(dir.path());
    for want in [
        "sj-debug: does not name condition-based-waiting.md",
        "sj-debug: does not question the design after 3 failed fixes",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got = messages(&shipped);
    assert!(!got.iter().any(|m| m.starts_with("sj-debug")), "{got:?}");
    for f in [
        "root-cause-tracing.md",
        "defense-in-depth.md",
        "condition-based-waiting.md",
    ] {
        assert!(
            shipped.join("sj-debug").join(f).is_file(),
            "{f} is not shipped"
        );
    }
}

// spec: SKILL-016
#[test]
fn skill_016_sj_verify_does_not_trust_a_subagents_report() {
    let dir = valid();
    with_body(
        dir.path(),
        "sj-verify",
        &body("sj-verify").replace(" A subagent's report is not evidence.", ""),
    );
    let got = messages(dir.path());
    assert!(
        has(
            &got,
            "sj-verify: does not say a subagent's report is not evidence"
        ),
        "{got:?}"
    );
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got = messages(&shipped);
    assert!(!got.iter().any(|m| m.starts_with("sj-verify")), "{got:?}");
}

// spec: SKILL-017
#[test]
fn skill_017_a_depth_file_that_does_not_exist_fails_the_check() {
    let dir = valid();
    let coverage = COVERAGE
        .replace(
            "| systematic-debugging | sj-debug | root-cause-tracing.md |",
            "| systematic-debugging | sj-debug | root-cause.md |",
        )
        .replace(
            "| verification-before-completion | sj-verify | SKILL.md |",
            "| verification-before-completion | sj-verify | |",
        )
        .replace(
            "| brainstorming | sj-explore | brainstorming.md |",
            "| brainstorming | sj-explore | ../sj-apply/SKILL.md |",
        );
    put(dir.path(), "COVERAGE.md", &coverage);
    let got = messages(dir.path());
    for want in [
        "coverage: Superpowers systematic-debugging depth sj-debug/root-cause.md does not exist",
        "coverage: Superpowers verification-before-completion has no depth",
        "coverage: Superpowers brainstorming depth sj-explore/../sj-apply/SKILL.md is not in the skill folder",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
    assert!(
        !got.iter()
            .any(|m| m.contains("OpenSpec") || m.contains("Spec Kit")),
        "OpenSpec and Spec Kit rows need no depth: {got:?}"
    );
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let text = std::fs::read_to_string(shipped.join("COVERAGE.md")).unwrap();
    assert!(
        text.contains("| Tool | Item | Skill | Depth |"),
        "shipped COVERAGE.md has no Depth column"
    );
    assert!(
        text.contains("| Superpowers | systematic-debugging | sj-debug | root-cause-tracing.md |"),
        "shipped COVERAGE.md lacks the debugging depth"
    );
    assert_eq!(messages(&shipped), Vec::<String>::new());
}

// spec: SKILL-018
#[test]
fn skill_018_brainstorming_triggers_sj_explore() {
    let dir = valid();
    put(
        dir.path(),
        "sj-explore/SKILL.md",
        &format!(
            "---\nname: sj-explore\ndescription: Use when an idea is vague.\n---\n\n{EXPLORE}\n"
        ),
    );
    let got = messages(dir.path());
    assert!(
        has(&got, "sj-explore: description does not name brainstorming"),
        "{got:?}"
    );
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let got = messages(&shipped);
    assert!(!got.iter().any(|m| m.starts_with("sj-explore")), "{got:?}");
}

// spec: SKILL-019
#[test]
fn skill_019_sj_explore_goes_wide_before_narrowing() {
    let dir = valid();
    with_body(
        dir.path(),
        "sj-explore",
        "Run `sj status`. Give two or three approaches.\n",
    );
    std::fs::remove_file(dir.path().join("sj-explore/brainstorming.md")).unwrap();
    let got = messages(dir.path());
    for want in [
        "sj-explore: does not go wide before narrowing",
        "sj-explore: does not name brainstorming.md",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
}

// spec: SKILL-020
#[test]
fn skill_020_the_brainstorming_reference_carries_the_technique() {
    let dir = valid();
    put(
        dir.path(),
        "sj-explore/brainstorming.md",
        "List ideas, then show the whole design at once.\n",
    );
    let got = messages(dir.path());
    for want in [
        "sj-explore: brainstorming.md does not name YAGNI",
        "sj-explore: brainstorming.md does not present the design one section at a time",
    ] {
        assert!(has(&got, want), "{want}: {got:?}");
    }
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let text = std::fs::read_to_string(shipped.join("COVERAGE.md")).unwrap();
    assert!(
        text.contains("| Superpowers | brainstorming | sj-explore | brainstorming.md |"),
        "shipped COVERAGE.md lacks the brainstorming depth"
    );
    assert!(shipped.join("sj-explore/brainstorming.md").is_file());
    assert_eq!(messages(&shipped), Vec::<String>::new());
}

// spec: SKILL-021
#[test]
fn skill_021_sj_propose_names_the_infer_workflow() {
    let (_, text) = sj_adapters::claude::SKILLS
        .iter()
        .find(|(name, _)| *name == "sj-propose")
        .unwrap();
    let start = text
        .find("\n## From existing tests")
        .expect("sj-propose has no `From existing tests` section");
    let rest = &text[start + 1..];
    let section = match rest[3..].find("\n## ") {
        Some(end) => &rest[..end + 3],
        None => rest,
    };
    for want in [
        "sj spec infer",
        "--write",
        "[NEEDS CLARIFICATION",
        "sj spec check",
    ] {
        assert!(section.contains(want), "section lacks {want}: {section}");
    }
    assert!(
        section.to_lowercase().contains("replace"),
        "section does not say to replace the markers: {section}"
    );
    assert!(
        section.contains("the test"),
        "section does not point at the test: {section}"
    );
}

// spec: SKILL-022
#[test]
fn skill_022_the_skills_name_for_all_and_the_property_libraries() {
    let skill = |want: &str| {
        sj_adapters::claude::SKILLS
            .iter()
            .find(|(name, _)| *name == want)
            .unwrap()
            .1
    };
    let propose = skill("sj-propose");
    assert!(
        propose.contains("\n- **FOR ALL** "),
        "sj-propose shows no `- **FOR ALL**` step"
    );
    let apply = skill("sj-apply");
    for want in [
        "FOR ALL",
        "proptest",
        "Hypothesis",
        "fast-check",
        "`not-property`",
    ] {
        assert!(apply.contains(want), "sj-apply lacks {want}");
    }
    assert_eq!(
        messages(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../skills")
                .as_path()
        ),
        Vec::<String>::new()
    );
}
