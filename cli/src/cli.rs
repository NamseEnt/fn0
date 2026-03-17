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
    Deploy {
        #[arg(long)]
        code_id: u64,
        #[arg(long)]
        code_version: u64,
    },
    Local {
        #[arg(short, long)]
        port: Option<u16>,
    },
}
