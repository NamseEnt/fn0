pub mod add;
pub mod admin;
pub mod build;
pub mod cloud;
pub mod cron;
pub mod deploy;
pub mod destroy;
pub mod dev;
pub mod env;
pub mod fe_runtime;
pub mod init;
pub mod login;
pub mod open;
pub mod project_config;
pub mod purge;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "forte")]
#[command(about = "Forte - Fullstack Framework", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Dev {
        #[arg(short, long)]
        project: Option<PathBuf>,
        #[arg(short = 'P', long)]
        port: Option<u16>,
    },
    Init {
        name: String,
        #[arg(long)]
        dev: bool,
    },
    Login {
        #[arg(long)]
        token: Option<String>,
    },
    Add {
        #[command(subcommand)]
        command: AddCommands,
    },
    Build {
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    /// Build and deploy this project
    ///
    /// Never prompts. A project that has not been through `forte cloud init`
    /// is refused rather than set up here, so this behaves the same in CI as
    /// it does on a terminal.
    Deploy {
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    /// Delete the deployed project and all of its resources
    Destroy {
        #[arg(long)]
        yes: bool,
    },
    /// Print the deployed app URL and open it in the browser
    Open {
        #[arg(short, long)]
        project: Option<PathBuf>,
        /// Print the URL without opening a browser
        #[arg(long)]
        print: bool,
    },
    /// Invalidate the edge copy of public objects
    Purge {
        /// Keys inside the project's public namespace, e.g. captures/1/0.mp4
        #[arg(required = true)]
        keys: Vec<String>,
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
    /// Set this project up on your own Cloudflare account
    Cloud {
        #[command(subcommand)]
        command: CloudCommands,
    },
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },
}

#[derive(Subcommand)]
pub enum EnvCommands {
    /// Set an env entry (plain by default, encrypted when --secret)
    Set {
        key: String,
        value: String,
        #[arg(long)]
        secret: bool,
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    /// Convert a legacy .env file into env.local.yaml
    Migrate {
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum CloudCommands {
    /// Give this project an identity, a Cloudflare account and a domain
    ///
    /// Interactive, because it asks for a Cloudflare API token and for a
    /// choice between two trust models. A token passed as an argument would
    /// land in shell history and in `ps`; here it is read hidden and never
    /// written down.
    ///
    /// Run it again to change the domain the project answers on. That means
    /// signing a new origin certificate, which needs the same token, so this
    /// is the only command that can do it.
    Init {
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum AdminCommands {
    Run {
        task: String,
        #[arg(short, long)]
        project: Option<PathBuf>,
        #[arg(long)]
        input_file: Option<PathBuf>,
        #[arg(long)]
        input: Option<String>,
        #[arg(long, default_value_t = 300)]
        timeout_seconds: u64,
    },
    RunLocal {
        task: String,
        #[arg(short = 'P', long, default_value_t = 3000)]
        port: u16,
        #[arg(long)]
        input_file: Option<PathBuf>,
        #[arg(long)]
        input: Option<String>,
        #[arg(long, default_value_t = 300)]
        timeout_seconds: u64,
    },
}

#[derive(Subcommand)]
pub enum AddCommands {
    Page { path: String },
    Action { path: String },
}
