//! Configuration loading for CLI flags, environment variables, and paths.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/config.ts`
//! - `pi/packages/coding-agent/src/core/settings-manager.ts`
//!
//! Simplifications:
//! - Starts with environment variables and defaults only.
//! - No settings UI, package config, themes, or model registry yet.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cwd: PathBuf,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

/// User-level presets stored in `~/.sabi/config.toml`.
/// Only non-sensitive fields; API keys stay in env/.env only.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    pub model: Option<String>,
    pub base_url: Option<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        // API keys must come from env/.env only — never from config files.
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| {
                anyhow::anyhow!(
                    "OPENAI_API_KEY environment variable is required. \
                     Set it in your environment or create a .env file in the current directory."
                )
            })?;

        // Load user-level presets from ~/.sabi/config.toml.
        let user_config = load_user_config();

        // Load optional per-project overrides from sabi.toml in the current directory.
        let project_config = load_project_config(std::env::current_dir()?.as_ref());

        // Resolution order: project config > user config > env var > default.
        let model = project_config
            .as_ref()
            .and_then(|c| c.model.clone())
            .or_else(|| user_config.as_ref().and_then(|c| c.model.clone()))
            .or_else(|| std::env::var("RUST_PI_MODEL").ok())
            .unwrap_or_else(|| "gpt-5.5".to_string());

        let base_url = project_config
            .as_ref()
            .and_then(|c| c.base_url.clone())
            .or_else(|| user_config.as_ref().and_then(|c| c.base_url.clone()))
            .or_else(|| std::env::var("RUST_PI_BASE_URL").ok())
            .unwrap_or_else(|| "https://api.avemujica.moe/v1".to_string());

        Ok(Self {
            cwd: std::env::current_dir()?,
            model,
            base_url,
            api_key,
        })
    }
}

fn load_user_config() -> Option<ConfigFile> {
    let path = user_config_path()?;
    let contents = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&contents)
        .map_err(|e| {
            eprintln!(
                "warning: failed to parse user config {}: {e}",
                path.display()
            );
            e
        })
        .ok()
}

pub fn user_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".sabi").join("config.toml"))
}

fn load_project_config(cwd: &Path) -> Option<ConfigFile> {
    let path = cwd.join("sabi.toml");
    let contents = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&contents)
        .map_err(|e| {
            eprintln!(
                "warning: failed to parse project config {}: {e}",
                path.display()
            );
            e
        })
        .ok()
}
