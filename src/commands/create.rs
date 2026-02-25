use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::{config, copy, git, tmux};

pub fn run(branch_name: &str) -> Result<()> {
    let repo_root = git::repo_root()?;
    let project_name = git::repo_name()?;
    git::validate_branch_name(branch_name)?;

    let config = config::load_config(&repo_root)?;

    let yati_base = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".yati")
        .join(&project_name);
    let worktree_path = yati_base.join(branch_name);

    if worktree_path.exists() {
        bail!(
            "Worktree path already exists: {}",
            worktree_path.display()
        );
    }

    println!("Creating worktree at {}", worktree_path.display());
    git::worktree_add(&worktree_path, branch_name)?;

    if !config.copy_files.is_empty() {
        println!("Copying configured files...");
        copy::copy_files(&repo_root, &worktree_path, &config.copy_files, &config.exclude)?;
    }

    for hook in &config.post_create {
        println!("Running post_create hook: {}", hook);
        let status = Command::new("sh")
            .args(["-c", hook])
            .current_dir(&worktree_path)
            .status()
            .with_context(|| format!("Failed to run hook: {}", hook))?;
        if !status.success() {
            eprintln!("Warning: post_create hook failed: {}", hook);
        }
    }

    let session_name = format!("{}/{}", project_name, branch_name);

    if tmux::is_in_tmux() {
        println!("Creating tmux session '{}'", session_name);
        tmux::new_session(&session_name, &worktree_path)?;
    } else {
        println!("Creating detached tmux session '{}'", session_name);
        tmux::new_session(&session_name, &worktree_path)?;
        println!("Attach with: tmux attach -t '{}'", session_name);
    }

    Ok(())
}
