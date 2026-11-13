//! Throwaway git repos with a tiny Cargo crate, for gate tests. Shared with sj-cli via `#[path]`.
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

pub struct Repo {
    pub dir: TempDir,
}

const CARGO_TOML: &str =
    "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n";

impl Repo {
    /// A repo on `main` with Cargo.toml, .gitignore and `src/lib.rs` = `lib`, committed as "base".
    pub fn with_lib(lib: &str) -> Self {
        let repo = Repo {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.write("Cargo.toml", CARGO_TOML);
        repo.write(".gitignore", "target/\nCargo.lock\n");
        repo.write("src/lib.rs", lib);
        repo.write("openspec/specs/.gitkeep", "");
        repo.commit("base");
        repo
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write(&self, rel: &str, content: &str) {
        let p = self.path().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args([
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.com",
            ])
            .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
            .args(args)
            .current_dir(self.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    pub fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", msg]);
    }

    pub fn branch(&self, name: &str) {
        self.git(&["checkout", "-q", "-b", name]);
    }

    /// Add a change folder `openspec/changes/<change>/` with the given scenarios.
    pub fn add_change(&self, change: &str, scenarios: &[(&str, &str)]) {
        self.write(
            &format!("openspec/changes/{change}/proposal.md"),
            "## Why\n\nTest.\n",
        );
        self.write(
            &format!("openspec/changes/{change}/specs/math/spec.md"),
            &spec_md(scenarios),
        );
    }
}

pub fn spec_md(scenarios: &[(&str, &str)]) -> String {
    let mut s = String::from(
        "## ADDED Requirements\n\n### Requirement: Math\nThe system SHALL do math.\n\n",
    );
    for (id, name) in scenarios {
        s.push_str(&format!(
            "#### Scenario: {id} {name}\n- **WHEN** called\n- **THEN** it works\n\n"
        ));
    }
    s
}
