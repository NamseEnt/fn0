use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fn0")]
#[command(about = "fn0 CLI - A project initialization tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },
    Build,
    Deploy,
    Destroy {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        yes: bool,
    },
    Rename {
        new_name: String,
    },
    Login {
        token: Option<String>,
    },
    Local {
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Invalidate the edge copy of public objects
    Purge {
        /// Keys inside the project's public namespace, e.g. captures/1/0.mp4
        #[arg(required = true)]
        keys: Vec<String>,
        #[arg(short, long)]
        project: Option<String>,
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
    Set {
        key: String,
        value: String,
        #[arg(long)]
        secret: bool,
    },
    List,
    Unset {
        key: String,
    },
}

#[derive(Subcommand)]
pub enum DomainCommands {
    Add { domain: String },
    Remove,
    Status,
}

#[derive(Subcommand)]
pub enum AdminCommands {
    Run {
        task: String,
        #[arg(short, long)]
        project: Option<String>,
        #[arg(long)]
        input_file: Option<std::path::PathBuf>,
        #[arg(long)]
        input: Option<String>,
        #[arg(long, default_value_t = 300)]
        timeout_seconds: u64,
    },
}
