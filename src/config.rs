use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
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
pub struct PortsConfig {
    pub offset: u16,
    #[serde(flatten)]
    pub ports: HashMap<String, u16>,
}

/// A named profile that layers over the base config, overriding only the
/// fields it sets. All fields are optional so "absent" differs from "empty".
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Profile {
    pub windows: Option<Vec<WindowConfig>>,
    pub post_create: Option<Vec<HookConfig>>,
    pub post_activate: Option<Vec<HookConfig>>,
    pub pre_teardown: Option<Vec<HookConfig>>,
    pub environment: Option<HashMap<String, String>>,
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
    pub ports: PortsConfig,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    /// Resolve the effective config for an optional profile name. With `None`,
    /// the base (top-level) config is the default and is returned unchanged.
    /// With `Some(name)`, the named profile is layered on top of the base; an
    /// unknown name is an error so creation fails fast.
    pub fn resolve_profile(mut self, name: Option<&str>) -> Result<Config> {
        let Some(name) = name else {
            return Ok(self);
        };
        let Some(profile) = self.profiles.get(name).cloned() else {
            bail!("Profile '{}' not found in yati.toml", name);
        };
        self.apply_profile(profile);
        Ok(self)
    }

    fn apply_profile(&mut self, p: Profile) {
        if let Some(w) = p.windows {
            self.tmux.windows = w;
        }
        if let Some(h) = p.post_create {
            self.post_create = h;
        }
        if let Some(h) = p.post_activate {
            self.post_activate = h;
        }
        if let Some(h) = p.pre_teardown {
            self.pre_teardown = h;
        }
        if let Some(env) = p.environment {
            self.environment.extend(env);
        }
    }
}

pub fn expand_template(value: &str, project: &str, branch: &str) -> String {
    value
        .replace("{{project}}", project)
        .replace("{{branch}}", branch)
}

pub fn sanitize_compose_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect()
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

/// Read the persisted profile name for a worktree from its `.yati_index` file.
/// The profile is stored as a `YATI_PROFILE=<name>` line; returns `None` when
/// no profile was recorded (i.e. the worktree uses the base config).
pub fn read_profile(worktree_path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(worktree_path.join(".yati_index")).ok()?;
    contents
        .lines()
        .find_map(|l| l.strip_prefix("YATI_PROFILE="))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> Config {
        toml::from_str(
            r#"
            post_create = ["base-create"]

            [tmux]
            windows = [{ name = "base" }]

            [environment]
            SHARED = "base"
            ONLY_BASE = "keep"

            [profiles.backend]
            windows = [{ name = "api", command = "go run ./cmd/api" }]
            environment = { SHARED = "backend", ONLY_BACKEND = "yes" }
            "#,
        )
        .unwrap()
    }

    #[test]
    fn no_profile_returns_base_unchanged() {
        let cfg = base_config().resolve_profile(None).unwrap();
        assert_eq!(cfg.tmux.windows.len(), 1);
        assert_eq!(cfg.tmux.windows[0].name, "base");
        assert_eq!(cfg.post_create.len(), 1);
        assert_eq!(cfg.environment.get("SHARED").unwrap(), "base");
    }

    #[test]
    fn named_profile_overrides_only_set_fields() {
        let cfg = base_config().resolve_profile(Some("backend")).unwrap();
        // windows overridden by the profile
        assert_eq!(cfg.tmux.windows.len(), 1);
        assert_eq!(cfg.tmux.windows[0].name, "api");
        // post_create not set on the profile → base retained
        assert_eq!(cfg.post_create.len(), 1);
        assert_eq!(cfg.post_create[0].command(), "base-create");
    }

    #[test]
    fn environment_merges_profile_over_base() {
        let cfg = base_config().resolve_profile(Some("backend")).unwrap();
        assert_eq!(cfg.environment.get("SHARED").unwrap(), "backend"); // profile wins
        assert_eq!(cfg.environment.get("ONLY_BASE").unwrap(), "keep"); // base survives
        assert_eq!(cfg.environment.get("ONLY_BACKEND").unwrap(), "yes"); // profile-only added
    }

    #[test]
    fn missing_profile_errors() {
        assert!(base_config().resolve_profile(Some("nope")).is_err());
    }

    #[test]
    fn bundled_example_parses() {
        let cfg: Config = toml::from_str(include_str!("../example.yati.toml")).unwrap();
        // The example documents two named profiles that each override windows.
        assert!(cfg.profiles.contains_key("backend"));
        let resolved = cfg.resolve_profile(Some("frontend")).unwrap();
        assert!(resolved.tmux.windows.iter().any(|w| w.name == "dev"));
    }
}
