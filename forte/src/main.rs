mod codegen;
mod commands;
mod config;
mod generator;
mod parser;
mod templates;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "forte")]
#[command(about = "A full-stack Rust+React framework with type-safe routing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Forte project
    Init {
        /// Project name
        project_name: String,
    },
    /// Start development server
    Dev,
    /// Build for production
    Build,
    /// Run tests
    Test,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project_name } => {
            commands::init::execute(&project_name)?;
        }
        Commands::Dev => {
            commands::dev::execute()?;
        }
        Commands::Build => {
            commands::build::execute()?;
        }
        Commands::Test => {
            commands::test::execute()?;
        }
    }

    Ok(())
}
