mod cli;
mod server;
mod sqld;
mod tools;

use anyhow::Result;
use clap::Parser;
use cli::{AddCommands, AdminCommands, Cli, Commands, DomainCommands, QueueCommands};

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = tokio::task::LocalSet::new();
    rt.block_on(local.run_until(async_main()))
}

async fn async_main() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
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

        Commands::Rename { new_name, project } => {
            let project_dir = project.unwrap_or_else(|| ".".into());
            cli::rename::run(project_dir, new_name).await?;
        }

        Commands::Queue { command } => match command {
            QueueCommands::Flush { project, task_name } => {
                let project_dir = project.unwrap_or_else(|| ".".into());
                cli::queue::flush(&project_dir, task_name.as_deref()).await?;
            }
        },

        Commands::Admin { command } => match command {
            AdminCommands::Run {
                task,
                project,
                input_file,
                input,
                timeout_seconds,
            } => {
                let project_dir = project.unwrap_or_else(|| ".".into());
                cli::admin::run(task, project_dir, input_file, input, timeout_seconds).await?;
            }
            AdminCommands::RunLocal {
                task,
                port,
                input_file,
                input,
                timeout_seconds,
            } => {
                cli::admin::run_local(task, port, input_file, input, timeout_seconds).await?;
            }
        },

        Commands::Domain { command } => match command {
            DomainCommands::Add { domain, project } => {
                let project_dir = project.unwrap_or_else(|| ".".into());
                cli::domain::add(project_dir, domain).await?;
            }
            DomainCommands::Remove { project } => {
                let project_dir = project.unwrap_or_else(|| ".".into());
                cli::domain::remove(project_dir).await?;
            }
            DomainCommands::Status { project } => {
                let project_dir = project.unwrap_or_else(|| ".".into());
                cli::domain::status(project_dir).await?;
            }
        },
    }

    Ok(())
}
