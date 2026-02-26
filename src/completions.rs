use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use clap_complete::engine::CompletionCandidate;

/// Complete `yati activate` with `project/branch` targets from `~/.yati/`.
pub fn complete_activate_target(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();
    let Some(yati_base) = dirs::home_dir().map(|h| h.join(".yati")) else {
        return Vec::new();
    };
    if !yati_base.exists() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let projects = match std::fs::read_dir(&yati_base) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    for entry in projects.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let project_name = entry.file_name();
        let project_str = project_name.to_string_lossy();

        for branch in find_worktrees(&path) {
            let target = format!("{}/{}", project_str, branch);
            if target.starts_with(prefix.as_ref()) {
                candidates.push(CompletionCandidate::new(target));
            }
        }
    }

    candidates
}

/// Complete `yati create` with existing git branch names.
pub fn complete_create_branch(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();
    let output = match Command::new("git")
        .args(["for-each-ref", "refs/heads/", "--format=%(refname:short)"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with(prefix.as_ref()))
        .map(|line| CompletionCandidate::new(line.to_string()))
        .collect()
}

/// Find worktree directories under `base`.
/// A worktree is identified by containing a `.git` file (not directory).
fn find_worktrees(base: &Path) -> Vec<String> {
    let mut results = Vec::new();
    find_worktrees_recursive(base, base, &mut results);
    results
}

fn find_worktrees_recursive(base: &Path, current: &Path, results: &mut Vec<String>) {
    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let git_path = path.join(".git");
        if git_path.exists() && git_path.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                results.push(rel.to_string_lossy().to_string());
            }
        } else {
            find_worktrees_recursive(base, &path, results);
        }
    }
}
