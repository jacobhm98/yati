use anyhow::{bail, Context, Result};
use std::fs;
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

    let entries = git::worktree_list_from(&worktree_path)?;
    let main_worktree = entries.first().context("No worktrees found")?;
    let config = config::load_config(&main_worktree.path)?
        .resolve_profile(config::read_profile(&worktree_path).as_deref())?;

    let index = fs::read_to_string(worktree_path.join(".yati_index"))
        .ok()
        .and_then(|s| s.lines().next().and_then(|l| l.trim().parse::<u32>().ok()))
        .unwrap_or(0);

    if tmux::session_exists(&session_name) {
        println!("Switching to existing session '{}'", session_name);
    } else {
        println!("Creating tmux session '{}'", session_name);
        tmux::new_session(&session_name, &worktree_path)?;
        tmux::setup_environment(&session_name, &config, &project_name, &branch_name, index)?;
        tmux::setup_windows(&session_name, &worktree_path, &config.tmux.windows)?;
    }

    tmux::setup_environment(&session_name, &config, &project_name, &branch_name, index)?;

    for hook in &config.post_activate {
        let cmd = hook.command();
        println!("Running post_activate hook: {}", cmd);
        let status = Command::new("sh")
            .args(["-c", cmd])
            .current_dir(&worktree_path)
            .status()
            .with_context(|| format!("Failed to run hook: {}", cmd))?;
        if !status.success() {
            eprintln!("Warning: post_activate hook failed: {}", cmd);
        }
    }

    tmux::attach_or_switch(&session_name)?;

    Ok(())
}
