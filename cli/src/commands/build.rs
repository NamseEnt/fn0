use crate::config::{Config, LanguageEnvironment};
use color_eyre::{eyre::eyre, Result};
use std::process::Command;

pub async fn execute() -> Result<()> {
    println!("🔨 Building project...\n");

    let config = Config::load("fn0.toml").map_err(|e| {
        eyre!(
            "Failed to load fn0.toml: {}. Make sure you're in a fn0 project directory.",
            e
        )
    })?;

    match config.language_env {
        LanguageEnvironment::TypescriptBunHono => {
            println!("Running bun run build...");
            let status = Command::new("bun")
                .args(["run", "build"])
                .status()
                .map_err(|e| eyre!("Failed to execute 'bun run build': {}", e))?;

            if !status.success() {
                return Err(eyre!("Build failed"));
            }

            println!("\n✅ Build completed successfully!");
        }
    }

    Ok(())
}
