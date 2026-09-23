use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use fn0_db_migrate::{
    TursoSource, dodb_config, inventory, migrate, normalize_project_subset, validate_page_size,
    verify,
};
use std::path::PathBuf;

const DEFAULT_PAGE_SIZE: usize = 256;
const DEFAULT_DODB_ADDRESS: &str = "127.0.0.1:18445";
const DEFAULT_DODB_SERVER_NAME: &str = "dodb.internal";
const DEFAULT_DODB_ROOT_CERT: &str = "/etc/dodb/server.crt";
const DEFAULT_MISMATCH_LIMIT: usize = 20;

#[derive(Parser)]
#[command(
    name = "fn0-db-migrate",
    about = "Inventory, migrate, and exactly verify fn0 Turso documents against dodb"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Print only the final machine-readable JSON report to stdout"
    )]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Inventory(InventoryArgs),
    Migrate(MigrateArgs),
    Verify(VerifyArgs),
}

#[derive(Args)]
struct InventoryArgs {
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE)]
    page_size: usize,
}

#[derive(Args)]
struct DodbArgs {
    #[arg(long, default_value = DEFAULT_DODB_ADDRESS)]
    dodb_addr: String,
    #[arg(long, default_value = DEFAULT_DODB_SERVER_NAME)]
    dodb_server_name: String,
    #[arg(long, default_value = DEFAULT_DODB_ROOT_CERT)]
    dodb_root_cert: PathBuf,
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE)]
    page_size: usize,
    #[arg(long = "project-id", action = clap::ArgAction::Append)]
    project_ids: Vec<String>,
    #[arg(long, default_value_t = DEFAULT_MISMATCH_LIMIT)]
    mismatch_limit: usize,
}

#[derive(Args)]
struct MigrateArgs {
    #[command(flatten)]
    dodb: DodbArgs,
    #[arg(
        long,
        help = "Required explicit confirmation before any destination write"
    )]
    apply: bool,
}

#[derive(Args)]
struct VerifyArgs {
    #[command(flatten)]
    dodb: DodbArgs,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let report = match cli.command {
        Command::Inventory(args) => {
            validate_page_size(args.page_size)?;
            let source = TursoSource::from_env()?;
            inventory(&source, args.page_size).await?
        }
        Command::Migrate(args) => {
            require_apply(args.apply)?;
            validate_dodb_args(&args.dodb)?;
            let subset = normalize_project_subset(&args.dodb.project_ids)?;
            let source = TursoSource::from_env()?;
            let connection = connect_dodb(&args.dodb).await?;
            let report = migrate(
                &source,
                &connection,
                subset.unwrap_or_default(),
                args.dodb.page_size,
                args.dodb.project_ids.is_empty(),
                args.dodb.mismatch_limit,
            )
            .await?;
            connection.close();
            report
        }
        Command::Verify(args) => {
            validate_dodb_args(&args.dodb)?;
            let subset = normalize_project_subset(&args.dodb.project_ids)?;
            let source = TursoSource::from_env()?;
            let connection = connect_dodb(&args.dodb).await?;
            let report = verify(
                &source,
                &connection,
                subset.unwrap_or_default(),
                args.dodb.page_size,
                args.dodb.project_ids.is_empty(),
                args.dodb.mismatch_limit,
            )
            .await?;
            connection.close();
            report
        }
    };

    if cli.json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }
    if matches!(report.mode.as_str(), "migrate" | "verify") && !report.verified {
        std::process::exit(2);
    }
    Ok(())
}

fn require_apply(apply: bool) -> Result<()> {
    if !apply {
        bail!("migrate writes to dodb and requires the explicit --apply flag")
    }
    Ok(())
}

fn validate_dodb_args(args: &DodbArgs) -> Result<()> {
    validate_page_size(args.page_size)?;
    if args.mismatch_limit > 1000 {
        bail!("mismatch sample limit must not exceed 1000")
    }
    Ok(())
}

async fn connect_dodb(args: &DodbArgs) -> Result<doc_db::DodbConnection> {
    let root_cert = std::fs::read(&args.dodb_root_cert).with_context(|| {
        format!(
            "failed to read dodb root certificate {}",
            args.dodb_root_cert.display()
        )
    })?;
    let config = dodb_config(&args.dodb_addr, &args.dodb_server_name, &root_cert)?;
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    doc_db::DodbConnection::connect(&config)
        .await
        .context("failed to connect to dodb")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn inventory_has_no_dodb_configuration_requirements() {
        let parsed = Cli::try_parse_from(["fn0-db-migrate", "inventory"]).unwrap();
        assert!(matches!(parsed.command, Command::Inventory(_)));
    }

    #[test]
    fn migration_requires_explicit_apply_at_execution_boundary() {
        let without_apply = Cli::try_parse_from(["fn0-db-migrate", "migrate"]).unwrap();
        assert!(matches!(&without_apply.command, Command::Migrate(_)));
        assert!(require_apply(false).is_err());
        let with_apply = Cli::try_parse_from(["fn0-db-migrate", "migrate", "--apply"]).unwrap();
        assert!(matches!(with_apply.command, Command::Migrate(_)));
        assert!(require_apply(true).is_ok());
    }

    #[test]
    fn page_size_is_bounded() {
        assert!(validate_page_size(1).is_ok());
        assert!(validate_page_size(4096).is_ok());
        assert!(validate_page_size(0).is_err());
        assert!(validate_page_size(4097).is_err());
    }

    #[test]
    fn clap_command_is_well_formed() {
        Cli::command().debug_assert();
    }
}
