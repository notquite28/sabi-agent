//! Configuration loading for CLI flags, environment variables, and paths.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/config.ts`
//! - `pi/packages/coding-agent/src/core/settings-manager.ts`
//!
//! Simplifications:
//! - Stores persistent user config and credentials under `~/.sabi/`.
//! - No settings UI, package config, themes, or model registry yet.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cwd: PathBuf,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub exa_api_key: Option<String>,
}

/// User-level presets stored in `~/.sabi/config.toml`.
/// Secrets are stored separately in `~/.sabi/auth.toml`.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(alias = "RUST_PI_MODEL")]
    pub model: Option<String>,
    #[serde(alias = "RUST_PI_BASE_URL")]
    pub base_url: Option<String>,
}

/// Auth credentials stored in `~/.sabi/auth.toml`.
/// This file is created with 0o600 permissions.
#[derive(Debug, Default, Deserialize, Serialize)]
struct AuthFile {
    openai_api_key: Option<String>,
    exa_api_key: Option<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Self::load_for_cwd(std::env::current_dir()?)
    }

    pub fn load_for_cwd(cwd: PathBuf) -> anyhow::Result<Self> {
        let api_key = openai_api_key()?;
        let exa_api_key = exa_api_key();

        // Load user-level presets from ~/.sabi/config.toml.
        let user_config = load_user_config();

        // Load optional per-project overrides from sabi.toml in the current directory.
        let project_config = load_project_config(&cwd);

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
            cwd,
            model,
            base_url,
            api_key,
            exa_api_key,
        })
    }
}

/// Returns the path to `~/.sabi`, or None if HOME is not set.
pub fn sabi_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".sabi"))
}

pub fn user_config_path() -> Option<PathBuf> {
    Some(sabi_dir()?.join("config.toml"))
}

pub fn auth_path() -> Option<PathBuf> {
    Some(sabi_dir()?.join("auth.toml"))
}

pub fn sessions_dir() -> Option<PathBuf> {
    Some(sabi_dir()?.join("sessions"))
}

pub fn history_path() -> Option<PathBuf> {
    Some(sabi_dir()?.join("history"))
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

pub fn openai_api_key() -> anyhow::Result<String> {
    let auth = load_auth_file();
    auth.as_ref()
        .and_then(|a| non_empty(a.openai_api_key.as_deref()))
        .map(str::to_owned)
        .or_else(|| non_empty_env("OPENAI_API_KEY"))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "OPENAI_API_KEY not found. Add it to ~/.sabi/auth.toml or set it in your environment"
            )
        })
}

pub fn exa_api_key() -> Option<String> {
    let auth = load_auth_file();
    auth.as_ref()
        .and_then(|a| non_empty(a.exa_api_key.as_deref()))
        .map(str::to_owned)
        .or_else(|| non_empty_env("EXA_API_KEY"))
}

fn non_empty_env(name: &str) -> Option<String> {
    let value = std::env::var(name).ok()?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    let value = value?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn load_auth_file() -> Option<AuthFile> {
    let path = auth_path()?;
    let contents = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&contents)
        .map_err(|e| {
            eprintln!("warning: failed to parse auth file {}: {e}", path.display());
            e
        })
        .ok()
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

/// Writes the auth file with 0o600 permissions so only the owner can read it.
pub async fn write_auth_file(openai_key: &str, exa_key: Option<&str>) -> anyhow::Result<()> {
    let path = auth_path().ok_or_else(|| anyhow::anyhow!("cannot resolve auth file path"))?;
    let auth = AuthFile {
        openai_api_key: Some(openai_key.to_string()),
        exa_api_key: exa_key.map(|s| s.to_string()),
    };
    let content = toml::to_string(&auth)?;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(&path, content).await?;

    // Set restrictive permissions (owner read/write only).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        _lock: MutexGuard<'static, ()>,
        home: Option<std::ffi::OsString>,
        openai: Option<std::ffi::OsString>,
        exa: Option<std::ffi::OsString>,
        cwd: PathBuf,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                _lock: ENV_LOCK.lock().expect("env lock poisoned"),
                home: std::env::var_os("HOME"),
                openai: std::env::var_os("OPENAI_API_KEY"),
                exa: std::env::var_os("EXA_API_KEY"),
                cwd: std::env::current_dir().expect("current dir exists"),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            restore_var("HOME", self.home.as_ref());
            restore_var("OPENAI_API_KEY", self.openai.as_ref());
            restore_var("EXA_API_KEY", self.exa.as_ref());
            std::env::set_current_dir(&self.cwd).expect("restore cwd");
        }
    }

    fn restore_var(name: &str, value: Option<&std::ffi::OsString>) {
        match value {
            Some(value) => std::env::set_var(name, value),
            None => std::env::remove_var(name),
        }
    }

    fn temp_home(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("sabi-agent-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join(".sabi")).expect("create temp .sabi");
        path
    }

    #[test]
    #[serial]
    fn api_keys_load_from_sabi_auth_before_environment() {
        let _guard = EnvGuard::new();
        let home = temp_home("auth-first");
        std::env::set_var("HOME", &home);
        std::env::set_var("OPENAI_API_KEY", "env-openai");
        std::env::set_var("EXA_API_KEY", "env-exa");
        std::fs::write(
            home.join(".sabi/auth.toml"),
            "openai_api_key = \"auth-openai\"\nexa_api_key = \"auth-exa\"\n",
        )
        .expect("write auth file");

        assert_eq!(openai_api_key().expect("openai key"), "auth-openai");
        assert_eq!(exa_api_key().as_deref(), Some("auth-exa"));
    }

    #[test]
    #[serial]
    fn working_directory_dotenv_is_not_loaded() {
        let _guard = EnvGuard::new();
        let home = temp_home("no-dotenv");
        let cwd = home.join("workspace");
        std::fs::create_dir_all(&cwd).expect("create cwd");
        std::fs::write(cwd.join(".env"), "OPENAI_API_KEY=dotenv-openai\n").expect("write dotenv");
        std::env::set_var("HOME", &home);
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("EXA_API_KEY");
        std::env::set_current_dir(&cwd).expect("set cwd");

        assert!(openai_api_key().is_err());
    }
}
