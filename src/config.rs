use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WindowConfig {
    pub name: String,
    pub command: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct TmuxConfig {
    pub windows: Vec<WindowConfig>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub copy_files: Vec<String>,
    pub exclude: Vec<String>,
    pub post_create: Vec<String>,
    pub pre_teardown: Vec<String>,
    pub tmux: TmuxConfig,
}

pub fn load_config(repo_root: &Path) -> Result<Config> {
    let config_path = repo_root.join("yati.toml");
    if !config_path.exists() {
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    let config: Config =
        toml::from_str(&contents).with_context(|| "Failed to parse yati.toml")?;
    Ok(config)
}
