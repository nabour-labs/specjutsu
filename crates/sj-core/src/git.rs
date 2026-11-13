//! Thin wrapper over the `git` CLI.

use std::path::{Path, PathBuf};

use anyhow::Result;

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
    pub fn run(&self, _args: &[&str]) -> Result<String> {
        todo!("GATE-020")
    }

    /// Resolve a ref to a commit SHA; errors name the ref and suggest a full fetch.
    pub fn resolve(&self, _rev: &str) -> Result<String> {
        todo!("GATE-020")
    }

    pub fn merge_base(&self, _a: &str, _b: &str) -> Result<String> {
        todo!("GATE-001")
    }

    /// Files that differ between `base` and the working tree, plus untracked files (repo-relative).
    pub fn changed_files(&self, _base: &str) -> Result<Vec<String>> {
        todo!("GATE-013")
    }

    /// Every file path in the tree of `rev`.
    pub fn ls_tree(&self, _rev: &str) -> Result<Vec<String>> {
        todo!("GATE-001")
    }

    /// Contents of `path` at `rev`.
    pub fn show(&self, _rev: &str, _path: &str) -> Result<String> {
        todo!("GATE-001")
    }

    /// Commits in `range`, oldest first.
    pub fn log(&self, _range: &str) -> Result<Vec<Commit>> {
        todo!("GATE-021")
    }

    pub fn worktree_add(&self, _path: &Path, _rev: &str) -> Result<()> {
        todo!("GATE-010")
    }

    pub fn worktree_remove(&self, _path: &Path) -> Result<()> {
        todo!("GATE-010")
    }
}
