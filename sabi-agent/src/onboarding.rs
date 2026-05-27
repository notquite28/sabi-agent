//! First-launch onboarding flow.
//!
//! Guides new users through initial setup: creating ~/.sabi/config.toml
//! with default presets, and explaining where API keys belong.

use std::io::Write;

use anyhow::Result;

use crate::config::{user_config_path, ConfigFile};

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

    println!("┌─ API Keys ─────────────────────────────────────────────────┐");
    println!("│ API keys (OPENAI_API_KEY, EXA_API_KEY) are kept separate   │");
    println!("│ from config files for security. You can:                   │");
    println!("│                                                            │");
    println!("│   1. Export them as environment variables                  │");
    println!("│   2. Create a .env file in your working directory          │");
    println!("│                                                            │");
    println!("│ Example .env file:                                         │");
    println!("│   OPENAI_API_KEY=sk-...                                    │");
    println!("│   EXA_API_KEY=your-exa-key                                 │");
    println!("└────────────────────────────────────────────────────────────┘\n");

    print!("Would you like to configure default presets now? [Y/n] ");
    std::io::stdout().flush()?;

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;

    let (model, base_url) = if answer.trim().eq_ignore_ascii_case("n") {
        println!("\nSkipping setup. You can run this again later by deleting ~/.sabi/config.toml");
        println!("or by editing that file directly.\n");
        (None, None)
    } else {
        println!("\n┌─ Default Presets ──────────────────────────────────────────┐");
        println!("│ These settings are saved to ~/.sabi/config.toml            │");
        println!("│ and can be overridden per-project with sabi.toml           │");
        println!("└────────────────────────────────────────────────────────────┘\n");

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
         \n{}",
        toml::to_string(&config)?
    );

    // Ensure parent directory exists before writing.
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&config_path, config_content).await?;

    if configured {
        println!();
        println!("✓ Config saved to {}", config_path.display());
        println!();
        println!("Next steps:");
        println!("  1. Set your API keys: export OPENAI_API_KEY=...");
        println!("  2. Run: cargo run -- --check-provider");
        println!("  3. Start chatting: cargo run");
        println!();
    }

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
