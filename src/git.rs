use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to run git")?;
    if !output.status.success() {
        bail!(
            "Not in a git repository: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let path = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();
    Ok(PathBuf::from(path))
}

pub fn repo_name() -> Result<String> {
    let root = repo_root()?;
    let name = root
        .file_name()
        .context("Could not determine repo name")?
        .to_string_lossy()
        .to_string();
    Ok(name)
}

pub fn validate_branch_name(name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["check-ref-format", "--branch", name])
        .output()
        .context("Failed to run git check-ref-format")?;
    if !output.status.success() {
        bail!("Invalid branch name: {}", name);
    }
    Ok(())
}

pub fn worktree_add(path: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            branch,
            &path.to_string_lossy(),
        ])
        .output()
        .context("Failed to run git worktree add")?;
    if output.status.success() {
        return Ok(());
    }

    // Branch might already exist, try without -b
    let output = Command::new("git")
        .args(["worktree", "add", &path.to_string_lossy(), branch])
        .output()
        .context("Failed to run git worktree add")?;
    if !output.status.success() {
        bail!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn has_dirty_tracked_files(path: &Path, repo_dir: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(["-C", &path.to_string_lossy(), "status", "--porcelain"])
        .current_dir(repo_dir)
        .output()
        .context("Failed to run git status")?;
    let text = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;
    Ok(text
        .lines()
        .any(|line| !line.starts_with("??")))
}

fn has_untracked_non_ignored_files(path: &Path, repo_dir: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args([
            "-C",
            &path.to_string_lossy(),
            "ls-files",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(repo_dir)
        .output()
        .context("Failed to run git ls-files")?;
    let text = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;
    Ok(!text.trim().is_empty())
}

fn clean_ignored_files(path: &Path, repo_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["-C", &path.to_string_lossy(), "clean", "-fdX"])
        .current_dir(repo_dir)
        .output()
        .context("Failed to run git clean")?;
    if !output.status.success() {
        bail!(
            "git clean -fdX failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn worktree_remove(path: &Path, force: bool, repo_dir: &Path) -> Result<()> {
    if !force {
        // Without --force: only block on genuinely dirty state
        if has_dirty_tracked_files(path, repo_dir)? || has_untracked_non_ignored_files(path, repo_dir)? {
            bail!(
                "Worktree has uncommitted changes or untracked files.\n\
                 Use 'yati teardown --force' to remove it anyway."
            );
        }
    }

    // Clean gitignored files (e.g. .env, node_modules) so git worktree remove succeeds
    clean_ignored_files(path, repo_dir)?;

    let output = Command::new("git")
        .args(["worktree", "remove", "--force", &path.to_string_lossy()])
        .current_dir(repo_dir)
        .output()
        .context("Failed to run git worktree remove")?;

    if !output.status.success() {
        // Fall back to manual removal + prune if git worktree remove still fails
        if path.exists() {
            std::fs::remove_dir_all(path)
                .with_context(|| format!("Failed to remove {}", path.display()))?;
        }
        worktree_prune(repo_dir)?;
    }

    Ok(())
}

#[derive(Debug)]
pub struct WorktreeEntry {
    pub path: PathBuf,
    pub head: String,
    pub branch: String,
}

pub fn worktree_prune(repo_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(repo_dir)
        .output()
        .context("Failed to run git worktree prune")?;
    if !output.status.success() {
        bail!(
            "git worktree prune failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn branch_delete(branch: &str, force: bool, repo_dir: &Path) -> Result<()> {
    let flag = if force { "-D" } else { "-d" };
    let output = Command::new("git")
        .args(["branch", flag, branch])
        .current_dir(repo_dir)
        .output()
        .context("Failed to run git branch delete")?;
    if !output.status.success() {
        bail!(
            "git branch {} failed: {}",
            flag,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn worktree_list() -> Result<Vec<WorktreeEntry>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .context("Failed to run git worktree list")?;
    if !output.status.success() {
        bail!(
            "git worktree list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let text = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;
    let mut entries = Vec::new();
    let mut path = None;
    let mut head = String::new();
    let mut branch = String::new();

    for line in text.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(p));
        } else if let Some(h) = line.strip_prefix("HEAD ") {
            head = h.to_string();
        } else if let Some(b) = line.strip_prefix("branch ") {
            // branch is like refs/heads/main
            branch = b
                .strip_prefix("refs/heads/")
                .unwrap_or(b)
                .to_string();
        } else if line.is_empty() {
            if let Some(p) = path.take() {
                entries.push(WorktreeEntry {
                    path: p,
                    head: std::mem::take(&mut head),
                    branch: std::mem::take(&mut branch),
                });
            }
        }
    }
    // Handle last entry if no trailing blank line
    if let Some(p) = path.take() {
        entries.push(WorktreeEntry {
            path: p,
            head,
            branch,
        });
    }
    Ok(entries)
}
