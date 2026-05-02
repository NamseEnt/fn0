pub mod add;
pub mod admin;
pub mod build;
pub mod deploy;
pub mod dev;
pub mod domain;
pub mod fe_runtime;
pub mod init;
pub mod queue;
pub mod rename;

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
    /// Start the development server with hot reload
    Dev {
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,

        /// Port to listen on (auto-selects from 3000 if not specified)
        #[arg(short = 'P', long)]
        port: Option<u16>,
    },

    /// Initialize a new Forte project
    Init {
        /// Project name
        name: String,
    },

    /// Add a new page or action
    Add {
        #[command(subcommand)]
        command: AddCommands,
    },

    /// Build the project locally without deploying
    Build {
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,
    },

    /// Build and deploy the project to fn0 cloud
    Deploy {
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,
    },

    /// Rename the deployed project (migrates Turso DB and switches subdomain)
    Rename {
        /// New project name
        new_name: String,

        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,
    },

    /// Queue management commands
    Queue {
        #[command(subcommand)]
        command: QueueCommands,
    },

    /// Admin task commands
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },

    /// Custom domain management
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },
}

#[derive(Subcommand)]
pub enum DomainCommands {
    /// Attach a custom domain to this project (CNAME-based)
    Add {
        /// Domain to attach (e.g. www.example.com)
        domain: String,
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    /// Detach the custom domain from this project
    Remove {
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    /// Show custom domain status for this project
    Status {
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum AdminCommands {
    /// Run an admin task against the deployed app
    Run {
        /// Task name (matches src/admin/<name>.rs)
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
    /// Run an admin task against a locally-running `forte dev`
    RunLocal {
        /// Task name
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
pub enum QueueCommands {
    /// Flush dead queue tasks back to pending
    Flush {
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        project: Option<PathBuf>,

        /// Only flush tasks with this name
        #[arg(short, long)]
        task_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AddCommands {
    /// Add a new page
    Page {
        /// Page path (e.g., "product/[id]")
        path: String,
    },

    /// Add a new action
    Action {
        /// Action path (e.g., "user/login")
        path: String,
    },
}
