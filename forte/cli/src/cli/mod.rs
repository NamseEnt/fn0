pub mod add;
pub mod admin;
pub mod build;
pub mod cloudflare;
pub mod cron;
pub mod deploy;
pub mod destroy;
pub mod dev;
pub mod domain;
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
    Deploy {
        #[arg(short, long)]
        project: Option<PathBuf>,
        #[arg(long)]
        name: Option<String>,
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
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },
    /// Run this project's storage, CDN and custom domain on your own
    /// Cloudflare account
    Cloudflare {
        #[command(subcommand)]
        command: CloudflareCommands,
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
pub enum DomainCommands {
    /// Attach a custom domain to this project
    Add {
        domain: String,
        /// Required for a project on your own Cloudflare account: the origin
        /// certificate is signed here, because fn0 holds no token that can
        /// sign one
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long)]
        zone_id: Option<String>,
        #[arg(long)]
        api_token: Option<String>,
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
pub enum CloudflareCommands {
    /// Store and verify the credentials for your Cloudflare account
    Connect {
        #[arg(long)]
        account_id: String,
        #[arg(long)]
        zone_id: String,
        /// Account API token with Workers R2 Storage Edit, Zone Read, Cache
        /// Purge and SSL and Certificates Edit
        #[arg(long)]
        api_token: String,
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
