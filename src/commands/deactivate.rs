use anyhow::{bail, Context, Result};

use crate::tmux;

pub fn run() -> Result<()> {
    if !tmux::is_in_tmux() {
        bail!("Not inside a tmux session");
    }

    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let yati_base = home.join(".yati");

    let relative = cwd
        .strip_prefix(&yati_base)
        .map_err(|_| anyhow::anyhow!("Not in a yati worktree (current directory is not under ~/.yati/)"))?;

    let components: Vec<_> = relative.components().collect();
    if components.len() < 2 {
        bail!("Cannot determine project/branch from path: {}", cwd.display());
    }
    let project = components[0].as_os_str().to_string_lossy();
    let branch = components[1].as_os_str().to_string_lossy();

    // Switch to the session we came from, if there was one
    if tmux::switch_to_previous_session() {
        println!("Deactivated '{}/{}'", project, branch);
    } else {
        // No other session — detach from tmux to return to the original terminal
        tmux::detach()?;
        println!("Deactivated '{}/{}', detached from tmux", project, branch);
    }

    Ok(())
}
