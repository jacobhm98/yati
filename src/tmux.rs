use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn is_in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub fn new_session(name: &str, working_dir: &Path) -> Result<()> {
    let output = Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            name,
            "-c",
            &working_dir.to_string_lossy(),
        ])
        .output()
        .context("Failed to run tmux new-session")?;
    if !output.status.success() {
        bail!(
            "tmux new-session failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    if is_in_tmux() {
        let output = Command::new("tmux")
            .args(["switch-client", "-t", name])
            .output()
            .context("Failed to run tmux switch-client")?;
        if !output.status.success() {
            bail!(
                "tmux switch-client failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }

    Ok(())
}

pub fn session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn attach_or_switch(name: &str) -> Result<()> {
    if is_in_tmux() {
        let output = Command::new("tmux")
            .args(["switch-client", "-t", name])
            .output()
            .context("Failed to run tmux switch-client")?;
        if !output.status.success() {
            bail!(
                "tmux switch-client failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    } else {
        let status = Command::new("tmux")
            .args(["attach", "-t", name])
            .status()
            .context("Failed to run tmux attach")?;
        if !status.success() {
            bail!("tmux attach failed");
        }
    }
    Ok(())
}

pub fn switch_to_last_session() -> bool {
    // Try switching to the last (previous) session
    if Command::new("tmux")
        .args(["switch-client", "-l"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }
    // Fall back to the next session
    Command::new("tmux")
        .args(["switch-client", "-n"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn kill_session(name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["kill-session", "-t", name])
        .output()
        .context("Failed to run tmux kill-session")?;
    if !output.status.success() {
        bail!(
            "tmux kill-session failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}
