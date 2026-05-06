mod cli;
mod commands;
mod config;
mod generators;
mod prompts;
mod utils;

use clap::Parser;
use cli::{AdminCommands, Cli, Commands, DomainCommands, SecretsCommands};
use color_eyre::Result;

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = tokio::task::LocalSet::new();
    rt.block_on(local.run_until(async_main()))
}

async fn async_main() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            commands::init::execute(name).await?;
        }
        Commands::Build => {
            commands::build::execute().await?;
        }
        Commands::Deploy => {
            commands::deploy::execute().await?;
        }
        Commands::Destroy => {
            commands::destroy::execute().await?;
        }
        Commands::Login { token } => {
            commands::login::execute(token).await?;
        }
        Commands::Local { port } => {
            commands::local::execute(port).await?;
        }
        Commands::Admin { command } => match command {
            AdminCommands::Run {
                task,
                project,
                input_file,
                input,
                timeout_seconds,
            } => {
                commands::admin::run(task, project, input_file, input, timeout_seconds).await?;
            }
        },
        Commands::Domain { command } => match command {
            DomainCommands::Add { domain } => {
                commands::domain::add(&domain).await?;
            }
            DomainCommands::Remove => {
                commands::domain::remove().await?;
            }
            DomainCommands::Status => {
                commands::domain::status().await?;
            }
        },
        Commands::Secrets { command } => match command {
            SecretsCommands::Set { key, value } => {
                commands::secrets::set(key, value).await?;
            }
            SecretsCommands::List => {
                commands::secrets::list().await?;
            }
            SecretsCommands::Unset { key } => {
                commands::secrets::unset(key).await?;
            }
        },
    }

    Ok(())
}
