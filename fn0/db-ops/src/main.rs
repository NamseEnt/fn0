use anyhow::{Context, Result, anyhow, bail};
use base64::Engine;
use clap::{Args, Parser, Subcommand};
use doc_db::{DodbConfig, DodbConnection};
use doc_db_protocol::{
    DocDbCondition, DocDbKey, DocDbMutation, DocDbObservedDocument, DocDbOperation, DocDbRequest,
    DocDbResult, DocDbTransactOutcome,
};
use serde::Serialize;
use std::io::IsTerminal;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, stdin};

const CONTROL_PROJECT_ID: &str = "fn0-control";
const DEFAULT_DODB_ADDR: &str = "127.0.0.1:18445";
const DEFAULT_DODB_SERVER_NAME: &str = "dodb.internal";
const DEFAULT_DODB_ROOT_CERT: &str = "/etc/dodb/server.crt";
const MAX_QUERY_LIMIT: u64 = 1000;

#[derive(Parser)]
#[command(name = "fn0-db-ops")]
struct Cli {
    #[arg(long, global = true, default_value = DEFAULT_DODB_ADDR)]
    dodb_addr: SocketAddr,
    #[arg(long, global = true, default_value = DEFAULT_DODB_SERVER_NAME)]
    dodb_server_name: String,
    #[arg(long, global = true, default_value = DEFAULT_DODB_ROOT_CERT)]
    dodb_root_cert: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    GetObserved(KeyArgs),
    Put(KeyArgs),
    PutIfMissing(KeyArgs),
    PutIfRevision(RevisionArgs),
    Query(QueryArgs),
}

#[derive(Args)]
struct KeyArgs {
    #[arg(long)]
    pk: String,
    #[arg(long)]
    sk: String,
}

#[derive(Args)]
struct RevisionArgs {
    #[command(flatten)]
    key: KeyArgs,
    #[arg(long)]
    expected_revision: u64,
}

#[derive(Args)]
struct QueryArgs {
    #[arg(long)]
    pk: String,
    #[arg(long)]
    after_sk: Option<String>,
    #[arg(long)]
    limit: u64,
}

#[derive(Serialize)]
struct ObservedOutput {
    found: bool,
    revision: u64,
    data_base64: Option<String>,
}

#[derive(Serialize)]
struct QueryOutput {
    documents: Vec<QueryDocument>,
}

#[derive(Serialize)]
struct QueryDocument {
    pk: String,
    sk: String,
    data_base64: String,
}

#[derive(Serialize)]
struct WriteOutput {
    written: bool,
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => {}
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(error.exit_code().unwrap_or(1));
        }
    }
}

trait ExitCode {
    fn exit_code(&self) -> Option<i32>;
}

#[derive(Debug)]
struct Conflict;

impl std::fmt::Display for Conflict {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("conditional write conflict")
    }
}

impl std::error::Error for Conflict {}

impl ExitCode for anyhow::Error {
    fn exit_code(&self) -> Option<i32> {
        self.downcast_ref::<Conflict>().map(|_| 3)
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let certificate = std::fs::read(&cli.dodb_root_cert).with_context(|| {
        format!(
            "could not read root certificate {}",
            cli.dodb_root_cert.display()
        )
    })?;
    let config = DodbConfig::new(cli.dodb_addr, cli.dodb_server_name, vec![certificate]);
    let connection = DodbConnection::connect(&config)
        .await
        .context("could not connect to dodb")?;
    let database = doc_db::dodb_with_connection(&connection, CONTROL_PROJECT_ID)
        .context("could not select fn0-control tenant")?;

    match cli.command {
        Command::GetObserved(arguments) => {
            let response = database
                .execute_semantic(DocDbRequest::new(DocDbOperation::GetObserved {
                    key: key(arguments),
                }))
                .await?;
            let DocDbResult::GetObserved { document } = response.result else {
                bail!("unexpected semantic response for get-observed")
            };
            let output = match document {
                DocDbObservedDocument::Present { data, revision } => ObservedOutput {
                    found: true,
                    revision: revision.value(),
                    data_base64: Some(base64::engine::general_purpose::STANDARD.encode(data)),
                },
                DocDbObservedDocument::Missing { revision } => ObservedOutput {
                    found: false,
                    revision: revision.map_or(0, |revision| revision.value()),
                    data_base64: None,
                },
            };
            print_json(&output)?;
        }
        Command::Put(arguments) => {
            let data = read_stdin().await?;
            database.put(&arguments.pk, &arguments.sk, &data).await?;
            print_json(&WriteOutput { written: true })?;
        }
        Command::PutIfMissing(arguments) => {
            let data = read_stdin().await?;
            conditional_put(&database, arguments, data, None).await?;
            print_json(&WriteOutput { written: true })?;
        }
        Command::PutIfRevision(arguments) => {
            let data = read_stdin().await?;
            conditional_put(
                &database,
                arguments.key,
                data,
                Some(arguments.expected_revision),
            )
            .await?;
            print_json(&WriteOutput { written: true })?;
        }
        Command::Query(arguments) => {
            validate_query_limit(arguments.limit)?;
            let response = database
                .execute_semantic(DocDbRequest::new(DocDbOperation::Query {
                    pk: arguments.pk,
                    after_sk: arguments.after_sk,
                    limit: arguments.limit,
                }))
                .await?;
            let DocDbResult::Query { documents } = response.result else {
                bail!("unexpected semantic response for query")
            };
            print_json(&QueryOutput {
                documents: documents
                    .into_iter()
                    .map(|document| QueryDocument {
                        pk: document.key.pk,
                        sk: document.key.sk,
                        data_base64: base64::engine::general_purpose::STANDARD
                            .encode(document.data),
                    })
                    .collect(),
            })?;
        }
    }
    connection.close();
    Ok(())
}

fn validate_query_limit(limit: u64) -> Result<()> {
    if limit == 0 || limit > MAX_QUERY_LIMIT {
        bail!("--limit must be between 1 and {MAX_QUERY_LIMIT}")
    }
    Ok(())
}

fn key(arguments: KeyArgs) -> DocDbKey {
    DocDbKey::new(arguments.pk, arguments.sk)
}

async fn conditional_put(
    database: &doc_db::Database,
    arguments: KeyArgs,
    data: Vec<u8>,
    expected_revision: Option<u64>,
) -> Result<()> {
    let document_key = key(arguments);
    let condition = match expected_revision {
        Some(revision) => DocDbCondition::RevisionEquals {
            key: document_key.clone(),
            expected_revision: doc_db_protocol::DocDbRevision::new(revision),
        },
        None => DocDbCondition::NotExists {
            key: document_key.clone(),
        },
    };
    let response = database
        .execute_semantic(DocDbRequest::new(DocDbOperation::Transact {
            conditions: vec![condition],
            mutations: vec![DocDbMutation::Put {
                key: document_key,
                data,
            }],
        }))
        .await?;
    match response.result {
        DocDbResult::Transact {
            outcome: DocDbTransactOutcome::Committed,
        } => Ok(()),
        DocDbResult::Transact {
            outcome: DocDbTransactOutcome::Conflict { .. },
        } => Err(anyhow!(Conflict)),
        _ => bail!("unexpected semantic response for conditional put"),
    }
}

async fn read_stdin() -> Result<Vec<u8>> {
    if std::io::stdin().is_terminal() {
        bail!("document data must be provided through stdin")
    }
    let mut data = Vec::new();
    stdin().read_to_end(&mut data).await?;
    Ok(data)
}

fn print_json(value: &impl Serialize) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodb_server::{
        DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerTlsConfig,
    };
    use rcgen::generate_simple_self_signed;
    use std::sync::Arc;

    async fn test_database() -> (
        doc_db::Database,
        DodbConnection,
        Arc<DodbServer<LocalTenantService>>,
        tokio::task::JoinHandle<std::result::Result<(), dodb_server::ServerError>>,
        tempfile::TempDir,
    ) {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let directory = tempfile::tempdir().unwrap();
        let certified =
            generate_simple_self_signed(vec![DEFAULT_DODB_SERVER_NAME.to_owned()]).unwrap();
        let certificate = certified.cert.der().to_vec();
        let private_key = certified.signing_key.serialize_der();
        let service = Arc::new(
            LocalTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_owned(),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let server = Arc::new(
            DodbServer::bind(
                service,
                DodbServerConfig {
                    listen_addr: "127.0.0.1:0".parse().unwrap(),
                    tls: ServerTlsConfig::from_der(vec![certificate.clone()], private_key).unwrap(),
                    protocol_limits: dodb_protocol::ProtocolLimits::default(),
                    max_connections: 8,
                    max_concurrent_streams: 64,
                    max_concurrent_requests: 64,
                },
            )
            .unwrap(),
        );
        let running_server = Arc::clone(&server);
        let task = tokio::spawn(async move { running_server.run().await });
        let connection = DodbConnection::connect(&DodbConfig::new(
            server.local_addr().unwrap(),
            DEFAULT_DODB_SERVER_NAME,
            vec![certificate],
        ))
        .await
        .unwrap();
        let database = doc_db::dodb_with_connection(&connection, CONTROL_PROJECT_ID).unwrap();
        (database, connection, server, task, directory)
    }

    fn key_arguments(pk: &str, sk: &str) -> KeyArgs {
        KeyArgs {
            pk: pk.to_owned(),
            sk: sk.to_owned(),
        }
    }

    #[test]
    fn query_limit_is_bounded_and_conflicts_have_a_distinct_exit_code() {
        assert!(validate_query_limit(1).is_ok());
        assert!(validate_query_limit(MAX_QUERY_LIMIT).is_ok());
        assert!(validate_query_limit(0).is_err());
        assert!(validate_query_limit(MAX_QUERY_LIMIT + 1).is_err());
        assert_eq!(anyhow!(Conflict).exit_code(), Some(3));
        assert_eq!(anyhow!("transport failure").exit_code(), None);
    }

    #[tokio::test]
    async fn semantic_operator_commands_preserve_bytes_and_report_conflicts() {
        let (database, _connection, server, task, _directory) = test_database().await;
        let missing = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::GetObserved {
                key: DocDbKey::new("binary", "one"),
            }))
            .await
            .unwrap();
        assert!(matches!(
            missing.result,
            DocDbResult::GetObserved {
                document: DocDbObservedDocument::Missing { .. }
            }
        ));

        let payload = vec![0, 255, 17, 128, 0];
        database.put("binary", "one", &payload).await.unwrap();
        let observed = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::GetObserved {
                key: DocDbKey::new("binary", "one"),
            }))
            .await
            .unwrap();
        let DocDbResult::GetObserved {
            document: DocDbObservedDocument::Present { data, revision },
        } = observed.result
        else {
            panic!("expected present observed document")
        };
        assert_eq!(data, payload);
        assert_eq!(
            database
                .get("binary", "one")
                .await
                .unwrap()
                .unwrap()
                .as_ref(),
            payload
        );

        conditional_put(&database, key_arguments("seed", ""), b"seed".to_vec(), None)
            .await
            .unwrap();
        assert!(
            conditional_put(
                &database,
                key_arguments("seed", ""),
                b"overwrite".to_vec(),
                None
            )
            .await
            .is_err()
        );

        conditional_put(
            &database,
            key_arguments("binary", "one"),
            b"updated".to_vec(),
            Some(revision.value()),
        )
        .await
        .unwrap();
        assert!(
            conditional_put(
                &database,
                key_arguments("binary", "one"),
                b"stale".to_vec(),
                Some(revision.value())
            )
            .await
            .is_err()
        );

        task.abort();
        let _ = server;
    }

    #[tokio::test]
    async fn semantic_query_orders_and_paginates_by_sort_key_in_control_tenant() {
        let (database, connection, _server, task, _directory) = test_database().await;
        for sort_key in ["c", "a", "b"] {
            database
                .put("ordered", sort_key, sort_key.as_bytes())
                .await
                .unwrap();
        }
        let first = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Query {
                pk: "ordered".to_owned(),
                after_sk: None,
                limit: 2,
            }))
            .await
            .unwrap();
        let DocDbResult::Query { documents } = first.result else {
            panic!("expected query response")
        };
        assert_eq!(
            documents
                .iter()
                .map(|document| document.key.sk.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        let second = database.query("ordered", Some("b"), 2).await.unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].0, "c");

        let other_tenant = doc_db::dodb_with_connection(&connection, "00000001").unwrap();
        assert!(other_tenant.get("ordered", "a").await.unwrap().is_none());
        task.abort();
    }
}
