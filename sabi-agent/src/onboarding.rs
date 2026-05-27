//! First-launch onboarding flow.
//!
//! Guides new users through initial setup: creating ~/.sabi/config.toml
//! with default presets, and explaining where API keys belong.

use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;

/// Runs onboarding if the user has no ~/.sabi/config.toml yet.
/// Returns true if onboarding ran (or was skipped), false if the user
/// wants to quit without configuring anything.
pub async fn run_if_needed() -> Result<bool> {
    let config_path = match user_config_path() {
        Some(path) => path,
        None => {
            // If we cannot resolve the home directory, skip onboarding silently.
            return Ok(true);
        }
    };

    if config_path.exists() {
        // User has already been onboarded.
        return Ok(true);
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

    if answer.trim().eq_ignore_ascii_case("n") {
        println!("\nSkipping setup. You can run this again later by deleting ~/.sabi/config.toml");
        println!("or by editing that file directly.\n");
        // Still create an empty config so we don't ask again.
        create_empty_config(&config_path).await?;
        return Ok(true);
    }

    // Collect presets
    println!("\n┌─ Default Presets ──────────────────────────────────────────┐");
    println!("│ These settings are saved to ~/.sabi/config.toml            │");
    println!("│ and can be overridden per-project with sabi.toml           │");
    println!("└────────────────────────────────────────────────────────────┘\n");

    let model = ask_with_default(
        "Default model",
        "gpt-5.5",
    )?;

    let base_url = ask_with_default(
        "Default base URL",
        "https://api.avemujica.moe/v1",
    )?;

    // Write config file
    let config_dir = config_path.parent().unwrap();
    tokio::fs::create_dir_all(config_dir).await?;

    let config_content = format!(
        "# Sabi Agent user-level presets\n\
         # These can be overridden per-project with sabi.toml\n\
         \n\
         model = \"{}\"\n\
         base_url = \"{}\"\n",
        model, base_url
    );

    tokio::fs::write(&config_path, config_content).await?;

    println!();
    println!("✓ Config saved to {}", config_path.display());
    println!();
    println!("Next steps:");
    println!("  1. Set your API keys: export OPENAI_API_KEY=...");
    println!("  2. Run: cargo run -- --check-provider");
    println!("  3. Start chatting: cargo run");
    println!();

    Ok(true)
}

fn user_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".sabi").join("config.toml"))
}

async fn create_empty_config(path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = "# Sabi Agent user-level presets\n\
                   # Uncomment and edit these to set defaults:\n\
                   # model = \"gpt-5.5\"\n\
                   # base_url = \"https://api.avemujica.moe/v1\"\n";
    tokio::fs::write(path, content).await?;
    Ok(())
}

fn ask_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", prompt, default);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}
