//! Configuration loading for CLI flags, environment variables, and paths.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/config.ts`
//! - `pi/packages/coding-agent/src/core/settings-manager.ts`
//!
//! Simplifications:
//! - Starts with environment variables and defaults only.
//! - No settings UI, package config, themes, or model registry yet.

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cwd: std::path::PathBuf,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY is required for plain chat mode"))?;
        let model = std::env::var("RUST_PI_MODEL").unwrap_or_else(|_| "gpt-5.5".to_string());
        let base_url = std::env::var("RUST_PI_BASE_URL")
            .unwrap_or_else(|_| "https://api.avemujica.moe/v1".to_string());

        Ok(Self {
            cwd: std::env::current_dir()?,
            model,
            base_url,
            api_key,
        })
    }
}
