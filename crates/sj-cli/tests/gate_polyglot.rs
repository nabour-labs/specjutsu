//! GATE-026, GATE-030..035 and GATE-038 (polyglot-gate) and GATE-046..047
//! (fix-base-run-no-report) and GATE-056..062 (gate-pilot-fixes) CLI scenarios: the gate across
//! suites.
//!
//! A fake `pnpm` on PATH stands in for vitest and Playwright (Unix only). Its vitest run fails
//! without a report unless `pnpm install` ran in the package first, fails to load the test file
//! when the test imports `refreshToken` and `src/auth.ts` has none, and otherwise passes only
//! when `src/auth.ts` holds the fix (`rejectExpired`).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

const BASE_SPEC: &str =
    "## Requirements\n\n### Requirement: Auth\nThe system SHALL authenticate.\n\n\
    #### Scenario: AUTH-001 Accepts a valid token\n- **WHEN** x\n- **THEN** y\n";

const AUTH_004: &str =
    "\n#### Scenario: AUTH-004 Rejects an expired token\n- **WHEN** x\n- **THEN** y\n";

const CHAT_002: &str = "\n#### Scenario: CHAT-002 Sends a message\n- **WHEN** x\n- **THEN** y\n";

const CONFIG: &str = r#"schema = 1

[[package]]
path = "apps/api"
sources = ["apps/api/src/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/api/src/**/*.test.ts"]
setup = "pnpm install --frozen-lockfile"

[[package]]
path = "apps/app"
sources = ["apps/app/src/**"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]
red_on_base = "skip"
reason = "needs staging API"
"#;

const BUGGY: &str = "export const check = (t) => true\n";
const FIXED: &str = "export const check = (t) => rejectExpired(t)\n";

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A repository on `main` with the config, a buggy `auth.ts` and installed head dependencies,
/// switched to branch `feature`.
fn repo() -> Repo {
    repo_with(CONFIG, BUGGY, false)
}

/// Like `repo`, with the given config and base `auth.ts`. With `deps_on_base`, the installed
/// marker is committed, so a base worktree has dependencies without `setup`.
fn repo_with(config: &str, auth: &str, deps_on_base: bool) -> Repo {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    write(r, "specjutsu.toml", config);
    if !deps_on_base {
        write(r, ".gitignore", "node_modules/\n");
    }
    write(r, "pnpm-lock.yaml", "");
    write(r, "openspec/specs/auth/spec.md", BASE_SPEC);
    write(r, "apps/api/src/auth.ts", auth);
    write(r, "apps/app/src/chat.ts", "export const send = () => {}\n");
    // The head checkout has its dependencies; a base worktree only gets them from `setup`.
    write(r, "apps/api/node_modules/.installed", "");
    repo.commit("base");
    repo.branch("feature");
    repo
}

fn add_auth_004(repo: &Repo, test: &str, source: &str) {
    let r = repo.path();
    write(
        r,
        "openspec/specs/auth/spec.md",
        &format!("{BASE_SPEC}{AUTH_004}"),
    );
    write(r, "apps/api/src/auth.test.ts", test);
    write(r, "apps/api/src/auth.ts", source);
    repo.commit("AUTH-004");
}

const TEST: &str = "import { check } from './auth'\n\
    it(\"AUTH-004 rejects expired token\", () => { expect(check('old')).toBe(false) })\n";

#[cfg(unix)]
fn fake_bin(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let pnpm = r#"#!/bin/sh
echo "$1 $2 $PWD" >> "$FAKE_LOG"
if [ "$1" = install ]; then
  if [ -n "$FAKE_SETUP_FAIL" ]; then
    echo " ERR_PNPM_OUTDATED_LOCKFILE  Cannot install with frozen-lockfile" >&2
    exit 1
  fi
  mkdir -p node_modules && touch node_modules/.installed
  exit 0
fi
if [ "$2" = playwright ]; then
  printf '<testsuites><testsuite name="e2e"><testcase classname="e2e/chat.spec.ts" name="sends a message"/></testsuite></testsuites>' > "$PLAYWRIGHT_JUNIT_OUTPUT_NAME"
  exit 0
fi
out=""
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
[ -f node_modules/.installed ] || { echo "Error: Cannot find module 'vitest'" >&2; exit 1; }
[ -f src/auth.test.ts ] || { echo "No test files found" >&2; exit 1; }
case_='<testcase classname="src/auth.test.ts" name="AUTH-004 rejects expired token"'
if grep -q refreshToken src/auth.test.ts && ! grep -q refreshToken src/auth.ts; then
  body='<testcase classname="src/auth.test.ts" name="src/auth.test.ts"><failure message="SyntaxError: The requested module does not provide an export named refreshToken"/></testcase>'
  code=1
elif grep -q rejectExpired src/auth.ts; then
  body="$case_/>"
  code=0
else
  body="$case_><failure message=\"expected false, got true\"/></testcase>"
  code=1
fi
printf '<testsuites><testsuite name="vitest">%s</testsuite></testsuites>' "$body" > "$out"
exit $code
"#;
    let p = dir.join("pnpm");
    std::fs::write(&p, pnpm).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// `sj gate --base main` with the fake `pnpm` first on PATH; returns the command and its log.
#[cfg(unix)]
fn gate(repo: &Repo, bin: &tempfile::TempDir) -> (Command, PathBuf) {
    fake_bin(bin.path());
    let log = bin.path().join("calls.log");
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut cmd = Command::cargo_bin("sj").unwrap();
    cmd.args(["gate", "--base", "main", "--root"])
        .arg(repo.path())
        .env("PATH", path)
        .env("FAKE_LOG", &log);
    (cmd, log)
}

#[cfg(unix)]
fn calls(log: &Path) -> Vec<String> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

#[cfg(unix)]
#[test]
fn gate_030_a_co_located_test_is_overlaid_without_its_source_file() {
    let repo = repo();
    add_auth_004(&repo, TEST, FIXED);
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.assert().success().stdout(predicate::str::starts_with(
        "gate: pass — 1 scenario proven (1 red:assert)\n",
    ));
    // The head checkout is untouched.
    assert_eq!(
        std::fs::read_to_string(repo.path().join("apps/api/src/auth.ts")).unwrap(),
        FIXED
    );
}

#[cfg(unix)]
#[test]
fn gate_031_base_dependencies_are_installed_before_the_base_run() {
    let repo = repo();
    add_auth_004(&repo, TEST, FIXED);
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, log) = gate(&repo, &bin);
    cmd.assert().success();
    let calls = calls(&log);
    let installs: Vec<usize> = (0..calls.len())
        .filter(|&i| calls[i].starts_with("install --frozen-lockfile"))
        .collect();
    assert_eq!(installs.len(), 1, "{calls:#?}");
    let install_dir = calls[installs[0]].rsplit(' ').next().unwrap();
    assert!(install_dir.ends_with("/apps/api"), "{calls:#?}");
    // The base vitest run comes after the install, in the same directory.
    assert!(
        calls[installs[0] + 1..]
            .iter()
            .any(|c| c.starts_with("exec vitest") && c.ends_with(install_dir)),
        "{calls:#?}"
    );
}

#[cfg(unix)]
#[test]
fn gate_032_a_failing_base_setup_is_an_environment_error() {
    let repo = repo();
    add_auth_004(&repo, TEST, FIXED);
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.env("FAKE_SETUP_FAIL", "1")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("pnpm install --frozen-lockfile"))
        .stderr(predicate::str::contains("ERR_PNPM_OUTDATED_LOCKFILE"));
}

#[cfg(unix)]
#[test]
fn gate_033_a_missing_symbol_on_base_is_red_compile() {
    let repo = repo();
    add_auth_004(
        &repo,
        "import { refreshToken } from './auth'\n\
         it(\"AUTH-004 rejects expired token\", () => { expect(refreshToken('old')).toBe(null) })\n",
        "export const refreshToken = (t) => rejectExpired(t)\n",
    );
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.assert().success().stdout(predicate::str::starts_with(
        "gate: pass — 1 scenario proven (1 red:compile)\n",
    ));
}

#[cfg(unix)]
#[test]
fn gate_034_an_e2e_suite_that_skips_red_on_base_is_unverifiable() {
    let repo = repo();
    let r = repo.path();
    write(
        r,
        "openspec/specs/auth/spec.md",
        &format!("{BASE_SPEC}{CHAT_002}"),
    );
    write(
        r,
        "apps/app/e2e/chat.spec.ts",
        "import { test, expect } from '@playwright/test'\n\n// spec: CHAT-002\ntest(\"sends a message\", async ({ page }) => {\n  await expect(page.getByText(\"sent\")).toBeVisible()\n})\n",
    );
    repo.commit("CHAT-002");
    let line = "unverifiable CHAT-002 sends a message (needs staging API)";

    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, log) = gate(&repo, &bin);
    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains(format!("fail    {line}")));
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.arg("--allow-unverifiable")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("warn    {line}")));
    // Nothing ran on base: no install, and Playwright only at head.
    let calls = calls(&log);
    assert!(
        !calls.iter().any(|c| c.starts_with("install")),
        "{calls:#?}"
    );
    assert!(
        calls.iter().any(|c| c.starts_with("exec playwright")),
        "{calls:#?}"
    );
}

#[test]
fn gate_035_only_files_matching_a_package_s_sources_need_a_scenario() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    write(
        r,
        "specjutsu.toml",
        "schema = 1\n\n[[package]]\npath = \"backend\"\nsources = [\"backend/src/**/*.py\"]\n\n\
         [[package.suite]]\nrunner = \"pytest\"\ntests = [\"backend/tests/**\"]\n",
    );
    write(r, "openspec/specs/auth/spec.md", BASE_SPEC);
    write(r, "backend/src/diff.py", "A = 1\n");
    write(r, "backend/notebooks/explore.py", "B = 1\n");
    repo.commit("base");
    repo.branch("feature");
    write(r, "backend/src/diff.py", "A = 2\n");
    write(r, "backend/notebooks/explore.py", "B = 2\n");
    repo.commit("change");

    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", "main", "--root"])
        .arg(r)
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "fail    unspecified change: backend/src/diff.py\n",
        ));
}

#[test]
fn gate_038_files_on_the_non_source_list_pass_without_an_exemption() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    let r = repo.path();
    write(
        r,
        "specjutsu.toml",
        "schema = 1\nnon_source = [\"infra/**\"]\n\n[[package]]\npath = \"backend\"\n\
         sources = [\"backend/src/**\"]\n\n[[package.suite]]\nrunner = \"pytest\"\ntests = []\n",
    );
    write(r, "openspec/specs/auth/spec.md", BASE_SPEC);
    write(r, "infra/main.tf", "a = 1\n");
    repo.commit("base");
    repo.branch("feature");
    write(r, "infra/main.tf", "a = 2\n");
    repo.commit("change");

    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", "main", "--root"])
        .arg(r)
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "gate: pass — no changed scenarios to prove\n",
        ));
}

/// CONFIG without the vitest suite's `setup`.
fn config_without_setup() -> String {
    CONFIG.replace("setup = \"pnpm install --frozen-lockfile\"\n", "")
}

#[cfg(unix)]
#[test]
fn gate_046_a_base_run_without_installed_dependencies_exits_2() {
    // Base already rejects expired tokens, so the test would pass there: it must not count as red.
    let repo = repo_with(&config_without_setup(), FIXED, false);
    add_auth_004(&repo, TEST, &format!("{FIXED}// tidy\n"));
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.assert()
        .code(2)
        .stdout(predicate::str::contains("red:compile").not())
        .stderr(predicate::str::contains("apps/api:vitest"))
        .stderr(predicate::str::contains("Cannot find module 'vitest'"))
        .stderr(predicate::str::contains("setup"));
}

#[cfg(unix)]
#[test]
fn gate_047_keep_a_base_report_with_a_failed_test_file_is_still_red_compile() {
    let repo = repo_with(&config_without_setup(), BUGGY, true);
    add_auth_004(
        &repo,
        "import { refreshToken } from './auth'\n\
         it(\"AUTH-004 rejects expired token\", () => { expect(refreshToken('old')).toBe(null) })\n",
        "export const refreshToken = (t) => rejectExpired(t)\n",
    );
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    cmd.assert().success().stdout(predicate::str::starts_with(
        "gate: pass — 1 scenario proven (1 red:compile)\n",
    ));
}

/// Writes `script` as a fake `pnpm` in `dir`.
#[cfg(unix)]
fn write_pnpm(dir: &Path, script: &str) {
    use std::os::unix::fs::PermissionsExt;
    let p = dir.join("pnpm");
    std::fs::write(&p, script).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A fake `pnpm` whose vitest run fails to load `src/users.test.ts` while
/// `src/testing/admin.ts` is missing, fails STAFF-073's assertion while `src/users.ts` lists no
/// admin role (base), and otherwise passes it (head).
#[cfg(unix)]
const PNPM_ADMIN: &str = r#"#!/bin/sh
out=""
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
case_='<testcase classname="src/users.test.ts" name="STAFF-073 lists admins"'
if [ ! -f src/testing/admin.ts ]; then
  body='<testcase classname="src/users.test.ts" name="src/users.test.ts"><failure message="Failed to load url ./testing/admin"/></testcase>'
  code=1
elif grep -q "'admin'" src/users.ts; then
  body="$case_/>"
  code=0
else
  body="$case_><failure message=\"AssertionError: expected [] to include 'admin'\"/></testcase>"
  code=1
fi
printf '<testsuites><testsuite name="vitest">%s</testsuite></testsuites>' "$body" > "$out"
exit $code
"#;

/// A fake `pnpm` whose vitest run fails to load `src/usage.test.ts` while `src/usage.ts` is
/// missing, and otherwise passes USAGE-001.
#[cfg(unix)]
const PNPM_USAGE: &str = r#"#!/bin/sh
out=""
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
if [ -f src/usage.ts ]; then
  body='<testcase classname="src/usage.test.ts" name="USAGE-001 counts requests"/>'
  code=0
else
  body='<testcase classname="src/usage.test.ts" name="src/usage.test.ts"><failure message="Failed to load url ./usage (resolved id: ./usage) in src/usage.test.ts. Does the file exist?"/></testcase>'
  code=1
fi
printf '<testsuites><testsuite name="vitest">%s</testsuite></testsuites>' "$body" > "$out"
exit $code
"#;

/// `sj gate --base main <args>` with `pnpm_script` as the fake `pnpm`.
#[cfg(unix)]
fn gate_output(repo: &Repo, pnpm_script: &str, args: &[&str]) -> std::process::Output {
    let bin = tempfile::tempdir().unwrap();
    write_pnpm(bin.path(), pnpm_script);
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", "main"])
        .args(args)
        .arg("--root")
        .arg(repo.path())
        .env("PATH", path)
        .output()
        .unwrap()
}

/// `sj gate --base main --json` with `pnpm_script` as the fake `pnpm`: exit code and report.
#[cfg(unix)]
fn gate_json(repo: &Repo, pnpm_script: &str) -> (Option<i32>, serde_json::Value) {
    let out = gate_output(repo, pnpm_script, &["--json"]);
    let report = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {out:?}"));
    (out.status.code(), report)
}

/// The report's `failures` reasons.
#[cfg(unix)]
fn failures(report: &serde_json::Value) -> Vec<&str> {
    report["failures"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f.as_str())
        .collect()
}

/// Whether the report's `proven` lists `id` with red kind `red`.
#[cfg(unix)]
fn proven_as(report: &serde_json::Value, id: &str, red: &str) -> bool {
    report["proven"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == id && p["red"] == red)
}

/// GATE-056's change, on a repository built with `config`: base has `apps/api/src/users.ts`
/// with no admin role; the change adds the role, `apps/api/src/testing/admin.ts` and
/// `apps/api/src/users.test.ts`, whose STAFF-073 test imports `./testing/admin`.
#[cfg(unix)]
fn staff_073_repo(config: &str) -> Repo {
    let repo = repo_with(config, BUGGY, true);
    let r = repo.path();
    repo.git(&["checkout", "-q", "main"]);
    write(r, "apps/api/src/users.ts", "export const roles = []\n");
    repo.commit("users");
    repo.git(&["checkout", "-q", "-B", "feature"]);
    write(
        r,
        "openspec/specs/staff/spec.md",
        "## Requirements\n\n### Requirement: Staff\nThe system SHALL list staff.\n\n\
         #### Scenario: STAFF-073 Lists admins\n- **WHEN** x\n- **THEN** y\n",
    );
    write(
        r,
        "apps/api/src/users.ts",
        "export const roles = ['admin']\n",
    );
    write(
        r,
        "apps/api/src/testing/admin.ts",
        "export const admin = { role: 'admin' }\n",
    );
    write(
        r,
        "apps/api/src/users.test.ts",
        "import { admin } from './testing/admin'\nimport { roles } from './users'\n\
         it(\"STAFF-073 lists admins\", () => { expect(roles).toContain(admin.role) })\n",
    );
    repo.commit("STAFF-073");
    repo
}

#[cfg(unix)]
#[test]
fn gate_056_a_helper_the_change_added_but_did_not_overlay_fails_the_gate() {
    let repo = staff_073_repo(&config_without_setup());
    let (code, report) = gate_json(&repo, PNPM_ADMIN);
    assert_eq!(code, Some(1), "{report:#}");
    let failures = failures(&report);
    assert!(
        failures.contains(
            &"not-overlaid STAFF-073 lists admins (apps/api/src/testing/admin.ts; add it to the suite's support globs)"
        ),
        "{failures:#?}"
    );
    assert!(
        !proven_as(&report, "STAFF-073", "red:compile"),
        "{report:#}"
    );
}

#[cfg(unix)]
#[test]
fn gate_057_keep_an_overlaid_helper_lets_the_test_fail_on_an_assertion() {
    let tests = "tests = [\"apps/api/src/**/*.test.ts\"]\n";
    let config = config_without_setup().replacen(
        tests,
        &format!("{tests}support = [\"apps/api/src/testing/**\"]\n"),
        1,
    );
    assert!(config.contains("support = [\"apps/api/src/testing/**\"]"));
    let repo = staff_073_repo(&config);
    let (_, report) = gate_json(&repo, PNPM_ADMIN);
    assert!(proven_as(&report, "STAFF-073", "red:assert"), "{report:#}");
    let failures = failures(&report);
    assert!(
        !failures.iter().any(|f| f.starts_with("not-overlaid")),
        "{failures:#?}"
    );
}

#[cfg(unix)]
#[test]
fn gate_058_keep_a_new_source_module_missing_on_base_is_still_red_compile() {
    let repo = repo_with(&config_without_setup(), BUGGY, true);
    let r = repo.path();
    write(
        r,
        "openspec/specs/usage/spec.md",
        "## Requirements\n\n### Requirement: Usage\nThe system SHALL count usage.\n\n\
         #### Scenario: USAGE-001 Counts requests\n- **WHEN** x\n- **THEN** y\n",
    );
    write(r, "apps/api/src/usage.ts", "export const count = () => 1\n");
    write(
        r,
        "apps/api/src/usage.test.ts",
        "import { count } from './usage'\n\
         it(\"USAGE-001 counts requests\", () => { expect(count()).toBe(1) })\n",
    );
    repo.commit("USAGE-001");
    let (_, report) = gate_json(&repo, PNPM_USAGE);
    assert!(proven_as(&report, "USAGE-001", "red:compile"), "{report:#}");
    let failures = failures(&report);
    assert!(
        !failures.iter().any(|f| f.starts_with("not-overlaid")),
        "{failures:#?}"
    );
}

#[cfg(unix)]
#[test]
fn gate_061_an_assert_suite_rejects_a_compile_red() {
    let tests = "tests = [\"apps/api/src/**/*.test.ts\"]\n";
    let config =
        config_without_setup().replacen(tests, &format!("{tests}red_on_base = \"assert\"\n"), 1);
    assert!(config.contains("red_on_base = \"assert\""));
    let repo = repo_with(&config, BUGGY, true);
    add_auth_004(
        &repo,
        "import { refreshToken } from './auth'\n\
         it(\"AUTH-004 rejects expired token\", () => { expect(refreshToken('old')).toBe(null) })\n",
        "export const refreshToken = (t) => rejectExpired(t)\n",
    );
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    let out = cmd.arg("--json").output().unwrap();
    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {out:?}"));
    assert_eq!(out.status.code(), Some(1), "{report:#}");
    let failures = failures(&report);
    assert!(
        failures.contains(
            &"compile-red AUTH-004 rejects expired token (suite requires an assertion failure on base)"
        ),
        "{failures:#?}"
    );
    let proven = report["proven"].as_array().unwrap();
    assert!(!proven.iter().any(|p| p["id"] == "AUTH-004"), "{report:#}");
}

#[cfg(unix)]
#[test]
fn gate_061_an_assert_suite_still_proves_an_assertion_failure_red_assert() {
    let tests = "tests = [\"apps/api/src/**/*.test.ts\"]\n";
    let config =
        config_without_setup().replacen(tests, &format!("{tests}red_on_base = \"assert\"\n"), 1);
    let repo = repo_with(&config, BUGGY, true);
    add_auth_004(&repo, TEST, FIXED);
    let bin = tempfile::tempdir().unwrap();
    let (mut cmd, _) = gate(&repo, &bin);
    let out = cmd.arg("--json").output().unwrap();
    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {out:?}"));
    assert!(proven_as(&report, "AUTH-004", "red:assert"), "{report:#}");
}

/// A fake `pnpm` whose vitest run passes STAFF-063 whatever the code: the behaviour exists on
/// base already.
#[cfg(unix)]
const PNPM_STAFF_063: &str = r#"#!/bin/sh
out=""
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
printf '<testsuites><testsuite name="vitest"><testcase classname="src/users.test.ts" name="STAFF-063 lists admins"/></testsuite></testsuites>' > "$out"
exit 0
"#;

/// GATE-059's change: base has STAFF-063 with no test and `apps/api/src/users.ts` listing the
/// admin role; the change adds `apps/api/src/users.test.ts`, a test for STAFF-063.
#[cfg(unix)]
fn staff_063_repo() -> Repo {
    let repo = repo_with(&config_without_setup(), BUGGY, true);
    let r = repo.path();
    repo.git(&["checkout", "-q", "main"]);
    write(
        r,
        "openspec/specs/staff/spec.md",
        "## Requirements\n\n### Requirement: Staff\nThe system SHALL list staff.\n\n\
         #### Scenario: STAFF-063 Lists admins\n- **WHEN** x\n- **THEN** y\n",
    );
    write(
        r,
        "apps/api/src/users.ts",
        "export const roles = ['admin']\n",
    );
    repo.commit("STAFF-063 spec");
    repo.git(&["checkout", "-q", "-B", "feature"]);
    write(
        r,
        "apps/api/src/users.test.ts",
        "import { roles } from './users'\n\
         it(\"STAFF-063 lists admins\", () => { expect(roles).toContain('admin') })\n",
    );
    repo.commit("test(STAFF-063)");
    repo
}

#[cfg(unix)]
#[test]
fn gate_059_a_test_linked_to_existing_behaviour_passes_the_gate() {
    let repo = staff_063_repo();
    let (code, report) = gate_json(&repo, PNPM_STAFF_063);
    assert_eq!(code, Some(0), "{report:#}");
    assert_eq!(report["pass"], true, "{report:#}");
    assert_eq!(
        report["summary"], "gate: pass — 1 scenario proven (1 green:linked)",
        "{report:#}"
    );
    assert_eq!(
        report["proven"],
        serde_json::json!([{ "id": "STAFF-063", "red": "green:linked", "property": false }]),
        "{report:#}"
    );
}

#[cfg(unix)]
#[test]
fn gate_059_markdown_lists_linked_behaviour_apart_from_the_table() {
    let repo = staff_063_repo();
    let out = gate_output(&repo, PNPM_STAFF_063, &["--format", "markdown"]);
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(
        md.contains(
            "\n**Linked existing behaviour**\n\n- STAFF-063 green:linked (`STAFF-063 lists admins`)\n"
        ),
        "{md}"
    );
    assert!(!md.contains("| STAFF-063"), "{md}");
}

#[cfg(unix)]
#[test]
fn gate_060_a_linked_scenario_does_not_cover_a_source_change() {
    let repo = staff_063_repo();
    write(
        repo.path(),
        "apps/api/src/users.ts",
        "export const roles = ['admin', 'owner']\n",
    );
    repo.commit("users");
    let (code, report) = gate_json(&repo, PNPM_STAFF_063);
    assert_eq!(code, Some(1), "{report:#}");
    assert_eq!(
        failures(&report),
        vec!["unspecified change: apps/api/src/users.ts"],
        "{report:#}"
    );
    assert!(
        proven_as(&report, "STAFF-063", "green:linked"),
        "{report:#}"
    );
}

/// A fake `pnpm` like `PNPM_STAFF_063`, except that STAFF-063's test fails while
/// `src/users.ts` lists the owner role (head).
#[cfg(unix)]
const PNPM_STAFF_063_FAILS_AT_HEAD: &str = r#"#!/bin/sh
out=""
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
case_='<testcase classname="src/users.test.ts" name="STAFF-063 lists admins"'
if grep -q owner src/users.ts; then
  body="$case_><failure message=\"AssertionError: expected only admin\"/></testcase>"
  code=1
else
  body="$case_/>"
  code=0
fi
printf '<testsuites><testsuite name="vitest">%s</testsuite></testsuites>' "$body" > "$out"
exit $code
"#;

#[cfg(unix)]
#[test]
fn gate_060_a_linked_scenario_failing_at_head_still_does_not_cover_a_source_change() {
    let repo = staff_063_repo();
    write(
        repo.path(),
        "apps/api/src/users.ts",
        "export const roles = ['admin', 'owner']\n",
    );
    repo.commit("users");
    let (code, report) = gate_json(&repo, PNPM_STAFF_063_FAILS_AT_HEAD);
    assert_eq!(code, Some(1), "{report:#}");
    let failures = failures(&report);
    assert!(
        failures.contains(&"failing STAFF-063 lists admins"),
        "{failures:#?}"
    );
    assert!(
        failures.contains(&"unspecified change: apps/api/src/users.ts"),
        "{failures:#?}"
    );
}

/// A fake `pnpm` whose vitest run fails to load `src/staff.test.ts` while
/// `src/testing/index.ts` has no `makeAdmin` export (base), and otherwise passes STAFF-074.
#[cfg(unix)]
const PNPM_MAKE_ADMIN: &str = r#"#!/bin/sh
out=""
for a in "$@"; do
  case "$a" in --outputFile.junit=*) out="${a#--outputFile.junit=}" ;; esac
done
if grep -q makeAdmin src/testing/index.ts; then
  body='<testcase classname="src/staff.test.ts" name="STAFF-074 makes an admin"/>'
  code=0
else
  body="<testcase classname=\"src/staff.test.ts\" name=\"src/staff.test.ts\"><failure message=\"SyntaxError: The requested module './testing/index.ts' does not provide an export named 'makeAdmin'\"/></testcase>"
  code=1
fi
printf '<testsuites><testsuite name="vitest">%s</testsuite></testsuites>' "$body" > "$out"
exit $code
"#;

#[cfg(unix)]
#[test]
fn gate_062_a_helper_the_change_modified_but_did_not_overlay_fails_the_gate() {
    let repo = repo_with(&config_without_setup(), BUGGY, true);
    let r = repo.path();
    repo.git(&["checkout", "-q", "main"]);
    write(
        r,
        "apps/api/src/testing/index.ts",
        "export const user = { role: 'user' }\n",
    );
    repo.commit("testing helpers");
    repo.git(&["checkout", "-q", "-B", "feature"]);
    write(
        r,
        "openspec/specs/staff/spec.md",
        "## Requirements\n\n### Requirement: Staff\nThe system SHALL make staff.\n\n\
         #### Scenario: STAFF-074 Makes an admin\n- **WHEN** x\n- **THEN** y\n",
    );
    write(
        r,
        "apps/api/src/testing/index.ts",
        "export const user = { role: 'user' }\n\
         export const makeAdmin = () => ({ role: 'admin' })\n",
    );
    write(
        r,
        "apps/api/src/staff.test.ts",
        "import { makeAdmin } from './testing/index.ts'\n\
         it(\"STAFF-074 makes an admin\", () => { expect(makeAdmin().role).toBe('admin') })\n",
    );
    repo.commit("STAFF-074");
    let (code, report) = gate_json(&repo, PNPM_MAKE_ADMIN);
    assert_eq!(code, Some(1), "{report:#}");
    let failures = failures(&report);
    assert!(
        failures.contains(
            &"not-overlaid STAFF-074 makes an admin (apps/api/src/testing/index.ts; add it to the suite's support globs)"
        ),
        "{failures:#?}"
    );
}
