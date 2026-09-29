#[path = "../../common/mod.rs"]
mod common;

use std::path::PathBuf;
use std::sync::Mutex;

use common::driver::{self, Args, Attempt, Engine, EngineFactory, RetryKind};
use common::workload::Mutation;
use serde_json::{json, Value};
use turso::params::Params;
use turso::{Builder, Connection, Database, Statement};

const CACHE_SIZE_KIB: i64 = 32_768;
const TURSO_TAG: &str = env!("CROSSDB_TURSO_TAG");
const TURSO_COMMIT: &str = env!("CROSSDB_TURSO_COMMIT");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Journal {
    Wal,
    Mvcc,
}

impl Journal {
    fn from_args(args: &Args) -> Self {
        match args.option("journal").unwrap_or("wal") {
            "wal" => Self::Wal,
            "mvcc" => Self::Mvcc,
            other => panic!("unknown journal {other}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Wal => "wal",
            Self::Mvcc => "mvcc",
        }
    }
}

fn group_commit_from_args(args: &Args) -> bool {
    match args.option("group-commit").unwrap_or("on") {
        "on" => true,
        "off" => false,
        other => panic!("unknown group-commit value {other}"),
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should build")
}

fn database_path(args: &Args) -> PathBuf {
    args.data_dir.join("kv.db")
}

async fn pragma_value(connection: &Connection, pragma: &str) -> Value {
    match connection.query(format!("PRAGMA {pragma}"), ()).await {
        Ok(mut rows) => match rows.next().await {
            Ok(Some(row)) => match row.get_value(0) {
                Ok(turso::Value::Integer(value)) => json!(value),
                Ok(turso::Value::Text(value)) => json!(value),
                Ok(turso::Value::Real(value)) => json!(value),
                Ok(other) => json!(format!("{other:?}")),
                Err(error) => json!(format!("error: {error}")),
            },
            Ok(None) => Value::Null,
            Err(error) => json!(format!("error: {error}")),
        },
        Err(error) => json!(format!("error: {error}")),
    }
}

async fn execute_pragma(connection: &Connection, statement: &str) {
    let mut rows = connection
        .query(statement, ())
        .await
        .unwrap_or_else(|error| panic!("{statement} failed: {error}"));
    while let Ok(Some(_)) = rows.next().await {}
}

fn classify(error: turso::Error) -> Attempt {
    match error {
        turso::Error::Busy(message) => Attempt::Retryable(RetryKind::Busy, message),
        turso::Error::BusySnapshot(message) => {
            Attempt::Retryable(RetryKind::BusySnapshot, message)
        }
        turso::Error::Error(message) if message.to_ascii_lowercase().contains("conflict") => {
            Attempt::Retryable(RetryKind::Conflict, message)
        }
        other => Attempt::Failed(format!("{other:?}")),
    }
}

struct TursoEngine {
    database: Mutex<Database>,
    journal: Journal,
    group_commit: bool,
    data_dir: PathBuf,
}

struct TursoWriter {
    runtime: tokio::runtime::Runtime,
    connection: Connection,
    begin: Statement,
    upsert: Statement,
    commit: Statement,
}

impl TursoEngine {
    fn connection(&self) -> Connection {
        self.database
            .lock()
            .unwrap()
            .connect()
            .expect("turso connection should open")
    }
}

async fn rollback(connection: &Connection) {
    let _ = connection.execute("ROLLBACK", ()).await;
    assert!(
        connection.is_autocommit().unwrap_or(false),
        "rollback must leave the connection outside a transaction"
    );
}

impl Engine for TursoEngine {
    type Writer = TursoWriter;

    fn open_writer(&self, _writer_id: usize) -> TursoWriter {
        let runtime = runtime();
        let connection = self.connection();
        let (begin, upsert, commit) = runtime.block_on(async {
            execute_pragma(&connection, "PRAGMA synchronous = FULL").await;
            execute_pragma(&connection, &format!("PRAGMA cache_size = -{CACHE_SIZE_KIB}")).await;
            let synchronous = pragma_value(&connection, "synchronous").await;
            assert_eq!(synchronous, json!(2), "synchronous must be FULL on every connection");
            let journal = pragma_value(&connection, "journal_mode").await;
            assert_eq!(journal, json!(self.journal.as_str()), "journal mode must match");
            if self.journal == Journal::Mvcc {
                let group_commit = pragma_value(&connection, "mvcc_group_commit").await;
                assert_eq!(
                    group_commit,
                    json!(i64::from(self.group_commit)),
                    "mvcc_group_commit must match on every connection"
                );
            }
            let begin_sql = match self.journal {
                Journal::Wal => "BEGIN",
                Journal::Mvcc => "BEGIN CONCURRENT",
            };
            let begin = connection.prepare(begin_sql).await.expect("prepare begin");
            let upsert = connection
                .prepare("INSERT INTO kv (k, v) VALUES (?1, ?2) ON CONFLICT (k) DO UPDATE SET v = excluded.v")
                .await
                .expect("prepare upsert");
            let commit = connection.prepare("COMMIT").await.expect("prepare commit");
            (begin, upsert, commit)
        });
        TursoWriter {
            runtime,
            connection,
            begin,
            upsert,
            commit,
        }
    }

    fn attempt(&self, writer: &mut TursoWriter, mutations: &[Mutation]) -> Attempt {
        let TursoWriter {
            runtime,
            connection,
            begin,
            upsert,
            commit,
        } = writer;
        runtime.block_on(async {
            if let Err(error) = begin.execute(()).await {
                rollback(connection).await;
                return classify(error);
            }
            for mutation in mutations {
                let params = Params::Positional(vec![
                    turso::Value::Blob(mutation.key.clone()),
                    turso::Value::Blob(mutation.value.clone()),
                ]);
                if let Err(error) = upsert.execute(params).await {
                    rollback(connection).await;
                    return classify(error);
                }
            }
            match commit.execute(()).await {
                Ok(_) => {
                    assert!(
                        connection.is_autocommit().unwrap_or(false),
                        "commit must end the transaction"
                    );
                    Attempt::Committed
                }
                Err(error) => {
                    rollback(connection).await;
                    classify(error)
                }
            }
        })
    }

    fn seed(&self, writer: &mut TursoWriter, rows: &[Mutation]) {
        let TursoWriter {
            runtime,
            connection,
            upsert,
            ..
        } = writer;
        runtime.block_on(async {
            connection.execute("BEGIN", ()).await.expect("seed begin");
            for row in rows {
                let params = Params::Positional(vec![
                    turso::Value::Blob(row.key.clone()),
                    turso::Value::Blob(row.value.clone()),
                ]);
                upsert.execute(params).await.expect("seed upsert");
            }
            connection.execute("COMMIT", ()).await.expect("seed commit");
        });
    }

    fn read(&self, writer: &mut TursoWriter, key: &[u8]) -> Option<Vec<u8>> {
        let TursoWriter {
            runtime,
            connection,
            ..
        } = writer;
        runtime.block_on(async {
            let mut rows = connection
                .query(
                    "SELECT v FROM kv WHERE k = ?1",
                    Params::Positional(vec![turso::Value::Blob(key.to_vec())]),
                )
                .await
                .expect("select should run");
            let row = rows.next().await.expect("select should step")?;
            match row.get_value(0).expect("value column") {
                turso::Value::Blob(bytes) => Some(bytes),
                other => panic!("unexpected value type {other:?}"),
            }
        })
    }

    fn count_rows(&self, writer: &mut TursoWriter) -> u64 {
        let TursoWriter {
            runtime,
            connection,
            ..
        } = writer;
        runtime.block_on(async {
            let mut rows = connection
                .query("SELECT count(*) FROM kv", ())
                .await
                .expect("count should run");
            let row = rows.next().await.expect("count step").expect("count row");
            match row.get_value(0).expect("count value") {
                turso::Value::Integer(count) => count as u64,
                other => panic!("unexpected count {other:?}"),
            }
        })
    }

    fn settings(&self, writer: &mut TursoWriter) -> Value {
        let TursoWriter {
            runtime,
            connection,
            ..
        } = writer;
        runtime.block_on(async {
            let mut settings = serde_json::Map::new();
            for pragma in [
                "journal_mode",
                "synchronous",
                "mvcc_group_commit",
                "cache_size",
                "page_size",
                "busy_timeout",
                "mvcc_checkpoint_threshold",
                "wal_autocheckpoint",
            ] {
                settings.insert(pragma.to_owned(), pragma_value(connection, pragma).await);
            }
            settings.insert("begin_statement".into(), json!(match self.journal {
                Journal::Wal => "BEGIN",
                Journal::Mvcc => "BEGIN CONCURRENT",
            }));
            settings.insert("io_backend".into(), json!("default (PlatformIO, syscall on Linux)"));
            settings.insert("per_connection_checks".into(), json!("synchronous=2, journal_mode, mvcc_group_commit asserted on every writer connection"));
            Value::Object(settings)
        })
    }

    fn metrics(&self) -> Value {
        driver::directory_listing(&self.data_dir)
    }

    fn monitor_sample(&self) -> Value {
        driver::directory_listing(&self.data_dir)
    }
}

struct TursoFactory;

impl TursoFactory {
    fn open(args: &Args, fresh: bool) -> TursoEngine {
        let journal = Journal::from_args(args);
        let group_commit = group_commit_from_args(args);
        let path = database_path(args);
        let database = runtime().block_on(async {
            let database = Builder::new_local(path.to_str().unwrap())
                .build()
                .await
                .expect("turso database should open");
            let connection = database.connect().expect("setup connection");
            if fresh {
                execute_pragma(&connection, &format!("PRAGMA journal_mode = {}", journal.as_str()))
                    .await;
                execute_pragma(&connection, "PRAGMA synchronous = FULL").await;
                if journal == Journal::Mvcc {
                    let value = if group_commit { "on" } else { "off" };
                    execute_pragma(&connection, &format!("PRAGMA mvcc_group_commit = {value}"))
                        .await;
                }
                connection
                    .execute("CREATE TABLE kv (k BLOB PRIMARY KEY, v BLOB NOT NULL)", ())
                    .await
                    .expect("create table");
            } else if journal == Journal::Mvcc {
                let value = if group_commit { "on" } else { "off" };
                execute_pragma(&connection, &format!("PRAGMA mvcc_group_commit = {value}")).await;
            }
            let effective = pragma_value(&connection, "journal_mode").await;
            assert_eq!(effective, json!(journal.as_str()), "effective journal mode");
            database
        });
        TursoEngine {
            database: Mutex::new(database),
            journal,
            group_commit,
            data_dir: args.data_dir.clone(),
        }
    }
}

impl EngineFactory for TursoFactory {
    type Engine = TursoEngine;

    fn engine_name(args: &Args) -> String {
        match Journal::from_args(args) {
            Journal::Wal => "turso-wal".to_owned(),
            Journal::Mvcc => {
                if group_commit_from_args(args) {
                    "turso-mvcc-gc".to_owned()
                } else {
                    "turso-mvcc-nogc".to_owned()
                }
            }
        }
    }

    fn build_info() -> Value {
        json!({
            "database": "turso",
            "tag": TURSO_TAG,
            "commit": TURSO_COMMIT,
            "binding": "turso Rust crate (bindings/rust), embedded in-process",
        })
    }

    fn create(args: &Args) -> TursoEngine {
        Self::open(args, true)
    }

    fn reopen(args: &Args) -> TursoEngine {
        Self::open(args, false)
    }

    fn close(engine: TursoEngine) -> Value {
        let data_dir = engine.data_dir.clone();
        drop(engine);
        json!({"files_after_close": driver::directory_listing(&data_dir)})
    }
}

fn main() {
    driver::bench_main::<TursoFactory>();
}
