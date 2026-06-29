use anyhow::{bail, Context, Result};
use std::fs;

use crate::git;

/// Barebones starter config written by `yati init`, mirroring example.yati.toml.
const TEMPLATE: &str = include_str!("../../example.yati.toml");

pub fn run() -> Result<()> {
    let repo_root = git::repo_root()?;
    let config_path = repo_root.join("yati.toml");

    if config_path.exists() {
        bail!("yati.toml already exists at {}", config_path.display());
    }

    fs::write(&config_path, TEMPLATE)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    println!("Created {}", config_path.display());
    Ok(())
}
