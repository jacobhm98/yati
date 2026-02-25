use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config::WindowConfig;

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

pub fn rename_window(session: &str, name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["rename-window", "-t", session, name])
        .output()
        .context("Failed to run tmux rename-window")?;
    if !output.status.success() {
        bail!(
            "tmux rename-window failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn create_window(
    session: &str,
    name: &str,
    working_dir: &Path,
    command: Option<&str>,
) -> Result<()> {
    let output = Command::new("tmux")
        .args([
            "new-window",
            "-t",
            session,
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
    if let Some(cmd) = command {
        let target = format!("{}:{}", session, name);
        send_keys(&target, cmd)?;
    }
    Ok(())
}

pub fn send_keys(target: &str, keys: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["send-keys", "-t", target, keys, "Enter"])
        .output()
        .context("Failed to run tmux send-keys")?;
    if !output.status.success() {
        bail!(
            "tmux send-keys failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn select_window(session: &str, window: &str) -> Result<()> {
    let target = format!("{}:{}", session, window);
    let output = Command::new("tmux")
        .args(["select-window", "-t", &target])
        .output()
        .context("Failed to run tmux select-window")?;
    if !output.status.success() {
        bail!(
            "tmux select-window failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn setup_windows(session: &str, working_dir: &Path, windows: &[WindowConfig]) -> Result<()> {
    if windows.is_empty() {
        return Ok(());
    }

    let first = &windows[0];
    rename_window(session, &first.name)?;
    if let Some(cmd) = &first.command {
        let target = format!("{}:{}", session, first.name);
        send_keys(&target, cmd)?;
    }

    for window in &windows[1..] {
        create_window(session, &window.name, working_dir, window.command.as_deref())?;
    }

    select_window(session, &first.name)?;

    Ok(())
}

pub fn detach() -> Result<()> {
    let output = Command::new("tmux")
        .args(["detach-client"])
        .output()
        .context("Failed to run tmux detach-client")?;
    if !output.status.success() {
        bail!(
            "tmux detach-client failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
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
