use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::{config, git, tmux};

pub fn run(branch_name: &str) -> Result<()> {
    let project_name = git::repo_name()?;
    let session_name = format!("{}/{}", project_name, branch_name);

    let home = dirs::home_dir().context("Could not determine home directory")?;
    let worktree_path = home.join(".yati").join(&project_name).join(branch_name);

    if !worktree_path.exists() {
        bail!("No yati worktree found for branch '{}'", branch_name);
    }

    if tmux::session_exists(&session_name) {
        println!("Switching to existing session '{}'", session_name);
        tmux::attach_or_switch(&session_name)?;
    } else {
        let entries = git::worktree_list()?;
        let main_worktree = entries.first().context("No worktrees found")?;
        let config = config::load_config(&main_worktree.path)?;

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

        println!("Creating tmux session '{}'", session_name);
        tmux::new_session(&session_name, &worktree_path)?;
        tmux::setup_windows(&session_name, &worktree_path, &config.tmux.windows)?;
    }

    Ok(())
}
