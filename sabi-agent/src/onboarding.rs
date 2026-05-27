//! First-launch onboarding flow.
//!
//! Guides new users through initial setup: creating ~/.sabi/config.toml
//! with default presets, and ~/.sabi/auth.toml with API keys.

use std::io::Write;

use anyhow::Result;

use crate::config::{user_config_path, write_auth_file, ConfigFile};

/// Runs onboarding if the user has no ~/.sabi/config.toml yet.
pub async fn run_if_needed() -> Result<()> {
    let config_path = match user_config_path() {
        Some(path) => path,
        None => {
            // If we cannot resolve the home directory, skip onboarding silently.
            return Ok(());
        }
    };

    if config_path.exists() {
        // User has already been onboarded.
        return Ok(());
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║               Welcome to Sabi Agent!                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("It looks like this is your first time running Sabi Agent.");
    println!("Let me help you get set up.\n");

    println!("All your settings live in ~/.sabi/:");
    println!("  ~/.sabi/config.toml  – presets (model, base_url)");
    println!("  ~/.sabi/auth.toml    – API keys (created with restricted permissions)");
    println!("  ~/.sabi/sessions/    – conversation history");
    println!("  ~/.sabi/history      – command history\n");

    print!("Would you like to configure default presets now? [Y/n] ");
    std::io::stdout().flush()?;

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;

    let (model, base_url) = if answer.trim().eq_ignore_ascii_case("n") {
        println!("\nSkipping setup. You can run this again later by deleting ~/.sabi/config.toml");
        println!("or by editing that file directly.\n");
        (None, None)
    } else {
        println!();
        let model = ask_with_default("Default model", "gpt-5.5");
        let base_url = ask_with_default("Default base URL", "https://api.avemujica.moe/v1");
        (Some(model), Some(base_url))
    };

    // Remember whether the user configured anything so we can show the success banner.
    let configured = model.is_some();

    // Build config struct and serialize to TOML so special characters are escaped safely.
    let config = ConfigFile { model, base_url };
    let config_content = format!(
        "# Sabi Agent user-level presets\n\
         # These can be overridden per-project with sabi.toml\n\
         # You can also use env vars: RUST_PI_MODEL, RUST_PI_BASE_URL\n\
         \n{}",
        toml::to_string(&config)?
    );

    // Ensure parent directory exists before writing.
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&config_path, config_content).await?;

    if configured {
        println!("✓ Config saved to {}", config_path.display());
    }

    // Offer to set up auth file
    println!();
    print!("Would you like to store your API keys in ~/.sabi/auth.toml? [Y/n] ");
    std::io::stdout().flush()?;

    let mut auth_answer = String::new();
    std::io::stdin().read_line(&mut auth_answer)?;

    if !auth_answer.trim().eq_ignore_ascii_case("n") {
        println!();
        print!("OpenAI API key: ");
        std::io::stdout().flush()?;
        let mut openai_key = String::new();
        std::io::stdin().read_line(&mut openai_key)?;
        let openai_key = openai_key.trim();

        print!("Exa API key (optional, press Enter to skip): ");
        std::io::stdout().flush()?;
        let mut exa_key = String::new();
        std::io::stdin().read_line(&mut exa_key)?;
        let exa_key = exa_key.trim();
        let exa_key = if exa_key.is_empty() {
            None
        } else {
            Some(exa_key)
        };

        if !openai_key.is_empty() {
            write_auth_file(openai_key, exa_key).await?;
            println!("✓ Auth file saved to ~/.sabi/auth.toml (permissions: 600)");
        }
    }

    println!();
    println!("Next steps:");
    println!("  1. Run: cargo run -- --check-provider");
    println!("  2. Start chatting: cargo run");
    println!();

    Ok(())
}

fn ask_with_default(prompt: &str, default: &str) -> String {
    print!("{} [{}]: ", prompt, default);
    let _ = std::io::stdout().flush();

    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);

    let trimmed = input.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}
