mod cli;
mod server;
mod sqld;
mod ssr;
mod tools;

use anyhow::Result;
use clap::Parser;
use cli::{AddCommands, Cli, Commands, QueueCommands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev { project, port } => {
            let options = cli::dev::DevOptions {
                project_dir: project.unwrap_or_else(|| ".".into()),
                port,
            };
            cli::dev::run(options).await?;
        }

        Commands::Init { name } => {
            cli::init::run(&name)?;
        }

        Commands::Add { command } => match command {
            AddCommands::Page { path } => {
                cli::add::add_page(&path)?;
            }
            AddCommands::Action { path } => {
                cli::add::add_action(&path)?;
            }
        },

        Commands::Build { project } => {
            let project_dir = project.unwrap_or_else(|| ".".into());
            cli::build::run_build(cli::build::BuildOptions { project_dir }).await?;
        }

        Commands::Deploy { project } => {
            let project_dir = project.unwrap_or_else(|| ".".into());
            cli::deploy::run(project_dir).await?;
        }

        Commands::Queue { command } => match command {
            QueueCommands::Flush { project, task_name } => {
                let project_dir = project.unwrap_or_else(|| ".".into());
                cli::queue::flush(&project_dir, task_name.as_deref()).await?;
            }
        },
    }

    Ok(())
}
