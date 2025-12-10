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
}
