use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::{config, git, tmux};

pub fn run(force: bool) -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let yati_base = home.join(".yati");

    let relative = cwd
        .strip_prefix(&yati_base)
        .map_err(|_| anyhow::anyhow!("Not in a yati worktree (current directory is not under ~/.yati/)"))?;

    // Extract project and branch from path: <project>/<branch>/...
    let components: Vec<_> = relative.components().collect();
    if components.len() < 2 {
        bail!("Cannot determine project/branch from path: {}", cwd.display());
    }
    let project = components[0].as_os_str().to_string_lossy().to_string();
    let branch = components[1].as_os_str().to_string_lossy().to_string();
    let worktree_path = yati_base.join(&project).join(&branch);

    // Find main worktree to load config from
    let entries = git::worktree_list()?;
    let main_worktree = entries
        .first()
        .context("No worktrees found")?;
    let config = config::load_config(&main_worktree.path)?;

    // Run pre_teardown hooks
    for hook in &config.pre_teardown {
        println!("Running pre_teardown hook: {}", hook);
        let status = Command::new("sh")
            .args(["-c", hook])
            .current_dir(&worktree_path)
            .status()
            .with_context(|| format!("Failed to run hook: {}", hook))?;
        if !status.success() {
            eprintln!("Warning: pre_teardown hook failed: {}", hook);
        }
    }

    println!("Removing worktree at {}", worktree_path.display());
    git::worktree_remove(&worktree_path, force)?;

    // Clean up empty parent directories
    let project_dir = yati_base.join(&project);
    cleanup_empty_dir(&project_dir);
    cleanup_empty_dir(&yati_base);

    let session_name = format!("{}/{}", project, branch);

    if tmux::is_in_tmux() {
        println!("Killing tmux session '{}'...", session_name);
        tmux::switch_to_last_session();
        tmux::kill_session(&session_name)?;
    }

    println!("Worktree '{}' removed successfully", branch);
    Ok(())
}

fn cleanup_empty_dir(path: &PathBuf) {
    if path.is_dir() {
        if let Ok(mut entries) = std::fs::read_dir(path) {
            if entries.next().is_none() {
                let _ = std::fs::remove_dir(path);
            }
        }
    }
}
