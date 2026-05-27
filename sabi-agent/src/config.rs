//! Configuration loading for CLI flags, environment variables, and paths.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/config.ts`
//! - `pi/packages/coding-agent/src/core/settings-manager.ts`
//!
//! Simplifications:
//! - Starts with environment variables and defaults only.
//! - No settings UI, package config, themes, or model registry yet.

use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cwd: std::path::PathBuf,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Default, Deserialize)]
struct ConfigFile {
    model: Option<String>,
    base_url: Option<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable is required. Set it or create a .env file in the current directory."))?;

        // Load optional `sabi.toml` from the current directory so that per-project
        // overrides do not leak into other workspaces.
        let file_config = load_config_file(std::env::current_dir()?.as_ref());

        let model = file_config
            .as_ref()
            .and_then(|c| c.model.clone())
            .or_else(|| std::env::var("RUST_PI_MODEL").ok())
            .unwrap_or_else(|| "gpt-5.5".to_string());

        let base_url = file_config
            .as_ref()
            .and_then(|c| c.base_url.clone())
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

fn load_config_file(cwd: &Path) -> Option<ConfigFile> {
    let path = cwd.join("sabi.toml");
    let contents = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&contents)
        .map_err(|e| {
            eprintln!("warning: failed to parse {}: {e}", path.display());
            e
        })
        .ok()
}
