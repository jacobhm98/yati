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
