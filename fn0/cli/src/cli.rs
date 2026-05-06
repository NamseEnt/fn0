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
    Destroy,
    Login {
        token: Option<String>,
    },
    Local {
        #[arg(short, long)]
        port: Option<u16>,
    },
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },
    Secrets {
        #[command(subcommand)]
        command: SecretsCommands,
    },
}

#[derive(Subcommand)]
pub enum SecretsCommands {
    Set { key: String, value: String },
    List,
    Unset { key: String },
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
