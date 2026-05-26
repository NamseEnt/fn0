pub mod add;
pub mod admin;
pub mod build;
pub mod cron;
pub mod deploy;
pub mod dev;
pub mod domain;
pub mod env;
pub mod fe_runtime;
pub mod init;
pub mod login;

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
    Deploy {
        #[arg(short, long)]
        project: Option<PathBuf>,
        #[arg(long)]
        name: Option<String>,
    },
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
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
}

#[derive(Subcommand)]
pub enum DomainCommands {
    /// Attach a custom domain to this project (CNAME-based)
    Add {
        domain: String,
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    Remove {
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    Status {
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
