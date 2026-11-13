//! Thin wrapper over the `git` CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// A commit's subject and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub subject: String,
    pub body: String,
}

pub struct Git {
    root: PathBuf,
}

impl Git {
    pub fn new(root: &Path) -> Self {
        Git {
            root: root.to_path_buf(),
        }
    }

    /// Run git with `args` in the repo root and return stdout.
    pub fn run(&self, args: &[&str]) -> Result<String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()
            .context("running git (is it installed and on PATH?)")?;
        if !out.status.success() {
            bail!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// Resolve a ref to a commit SHA; errors name the ref and suggest a full fetch.
    pub fn resolve(&self, rev: &str) -> Result<String> {
        let spec = format!("{rev}^{{commit}}");
        match self.run(&["rev-parse", "--verify", "--quiet", &spec]) {
            Ok(sha) if !sha.trim().is_empty() => Ok(sha.trim().to_string()),
            _ => bail!(
                "base ref `{rev}` not found in this repository; fetch it first \
                 (in GitHub Actions use actions/checkout with `fetch-depth: 0`)"
            ),
        }
    }

    pub fn merge_base(&self, a: &str, b: &str) -> Result<String> {
        Ok(self.run(&["merge-base", a, b])?.trim().to_string())
    }

    /// Files that differ between `base` and the working tree, plus untracked files (repo-relative).
    pub fn changed_files(&self, base: &str) -> Result<Vec<String>> {
        let mut files: Vec<String> = self
            .run(&["diff", "--name-only", "--no-renames", base])?
            .lines()
            .chain(
                self.run(&["ls-files", "--others", "--exclude-standard"])?
                    .lines(),
            )
            .map(str::to_string)
            .collect();
        files.sort();
        files.dedup();
        Ok(files)
    }

    /// Every file path in the tree of `rev`.
    pub fn ls_tree(&self, rev: &str) -> Result<Vec<String>> {
        Ok(self
            .run(&["ls-tree", "-r", "--name-only", rev])?
            .lines()
            .map(str::to_string)
            .collect())
    }

    /// Contents of `path` at `rev`.
    pub fn show(&self, rev: &str, path: &str) -> Result<String> {
        self.run(&["show", &format!("{rev}:{path}")])
    }

    /// Commits in `range`, oldest first.
    pub fn log(&self, range: &str) -> Result<Vec<Commit>> {
        let raw = self.run(&["log", "--reverse", "--format=%s%x00%b%x1e", range])?;
        Ok(raw
            .split('\x1e')
            .filter(|c| !c.trim().is_empty())
            .map(|c| {
                let (subject, body) = c.split_once('\0').unwrap_or((c, ""));
                Commit {
                    subject: subject.trim().to_string(),
                    body: body.trim().to_string(),
                }
            })
            .collect())
    }

    pub fn worktree_add(&self, path: &Path, rev: &str) -> Result<()> {
        let p = path.to_string_lossy();
        self.run(&["worktree", "add", "--quiet", "--detach", &p, rev])
            .map(|_| ())
    }

    pub fn worktree_remove(&self, path: &Path) -> Result<()> {
        let p = path.to_string_lossy();
        self.run(&["worktree", "remove", "--force", &p])?;
        self.run(&["worktree", "prune"]).map(|_| ())
    }
}
