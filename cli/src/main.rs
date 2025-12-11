mod cli;
mod commands;
mod config;
mod generators;
mod prompts;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            commands::init::execute(name).await?;
        }
        Commands::Build => {
            commands::build::execute().await?;
        }
        Commands::Local { port } => {
            commands::local::execute(port).await?;
        }
    }

    Ok(())
}
