use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::{config, git, tmux};

/// Parse a target string into (project_name, branch_name).
///
/// - If target contains `/`: first try interpreting the first segment as a project name.
///   Check if `~/.yati/<first_segment>/<rest>` exists. If yes, use that split.
///   Otherwise, fall back to treating the whole string as a branch in the current project.
/// - If target has no `/`: treat as a branch name in the current project.
fn parse_target(target: &str) -> Result<(String, String)> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let yati_base = home.join(".yati");

    if let Some(slash_pos) = target.find('/') {
        let first_segment = &target[..slash_pos];
        let rest = &target[slash_pos + 1..];

        if !rest.is_empty() && yati_base.join(first_segment).join(rest).exists() {
            return Ok((first_segment.to_string(), rest.to_string()));
        }
    }

    // Fall back to current project
    let project = git::repo_name()
        .context("Not in a git repository. Use <project>/<branch> syntax for cross-project activation.")?;
    Ok((project, target.to_string()))
}

pub fn run(target: &str) -> Result<()> {
    let (project_name, branch_name) = parse_target(target)?;
    let session_name = format!("{}/{}", project_name, branch_name);

    let home = dirs::home_dir().context("Could not determine home directory")?;
    let worktree_path = home.join(".yati").join(&project_name).join(&branch_name);

    if !worktree_path.exists() {
        bail!("No yati worktree found for '{}'", target);
    }

    if tmux::session_exists(&session_name) {
        println!("Switching to existing session '{}'", session_name);
        tmux::attach_or_switch(&session_name)?;
    } else {
        let entries = git::worktree_list_from(&worktree_path)?;
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
