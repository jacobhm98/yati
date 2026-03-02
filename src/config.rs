use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum HookConfig {
    Simple(String),
    Detailed {
        command: String,
        #[serde(default, rename = "async")]
        r#async: bool,
    },
}

impl HookConfig {
    pub fn command(&self) -> &str {
        match self {
            HookConfig::Simple(s) => s,
            HookConfig::Detailed { command, .. } => command,
        }
    }

    pub fn is_async(&self) -> bool {
        match self {
            HookConfig::Simple(_) => false,
            HookConfig::Detailed { r#async, .. } => *r#async,
        }
    }
}

impl Default for HookConfig {
    fn default() -> Self {
        HookConfig::Simple(String::new())
    }
}

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
    pub post_create: Vec<HookConfig>,
    pub post_activate: Vec<HookConfig>,
    pub pre_teardown: Vec<HookConfig>,
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
