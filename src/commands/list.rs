use anyhow::{Context, Result};
use std::path::Path;

use crate::git;

pub fn run() -> Result<()> {
    let yati_base = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".yati");

    if !yati_base.exists() {
        println!("No yati-managed worktrees found.");
        return Ok(());
    }

    let mut projects: Vec<_> = std::fs::read_dir(&yati_base)
        .context("Could not read ~/.yati directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    projects.sort_by_key(|e| e.file_name());

    if projects.is_empty() {
        println!("No yati-managed worktrees found.");
        return Ok(());
    }

    let mut first = true;
    for project in &projects {
        let project_name = project.file_name();
        let project_path = project.path();

        let mut worktrees = find_worktrees(&project_path);
        if worktrees.is_empty() {
            continue;
        }
        worktrees.sort();

        if !first {
            println!();
        }
        first = false;

        println!("{}:", project_name.to_string_lossy());
        for branch in &worktrees {
            let wt_path = project_path.join(branch);
            let short_head = git::head_short(&wt_path).unwrap_or_else(|_| "???????".to_string());
            println!("  {} ({})", branch, short_head);
        }
    }

    if first {
        println!("No yati-managed worktrees found.");
    }

    Ok(())
}

/// Recursively find worktree directories under `base`.
/// A worktree is identified by containing a `.git` file (not directory).
/// Returns branch names relative to `base` (e.g. "main", "feature/foo").
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
            // This is a worktree leaf
            if let Ok(rel) = path.strip_prefix(base) {
                results.push(rel.to_string_lossy().to_string());
            }
        } else {
            // Recurse to handle branch names with slashes
            find_worktrees_recursive(base, &path, results);
        }
    }
}
