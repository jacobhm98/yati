use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn is_in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub fn new_window(name: &str, working_dir: &Path) -> Result<()> {
    let output = Command::new("tmux")
        .args([
            "new-window",
            "-n",
            name,
            "-c",
            &working_dir.to_string_lossy(),
        ])
        .output()
        .context("Failed to run tmux new-window")?;
    if !output.status.success() {
        bail!(
            "tmux new-window failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn kill_current_window() -> Result<()> {
    let output = Command::new("tmux")
        .args(["kill-window"])
        .output()
        .context("Failed to run tmux kill-window")?;
    if !output.status.success() {
        bail!(
            "tmux kill-window failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}
