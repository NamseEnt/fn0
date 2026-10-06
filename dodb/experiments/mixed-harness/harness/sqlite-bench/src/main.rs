#![recursion_limit = "256"]

#[path = "../../common/mod.rs"]
mod common;

use std::path::PathBuf;
use std::time::Duration;

use common::driver::{self, Args, Attempt, Engine, EngineFactory, RetryKind};
use common::workload::Mutation;
use rusqlite::{
    Connection, Error, ErrorCode, OpenFlags, OptionalExtension, TransactionBehavior, params,
};
use serde_json::{Value, json};

const CACHE_KIB: i64 = 65_536;

struct SqliteEngine {
    path: PathBuf,
}

impl SqliteEngine {
    fn open_connection(&self) -> Connection {
        let connection = Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .expect("SQLite connection should open");
        connection
            .busy_timeout(Duration::from_secs(30))
            .expect("busy timeout should set");
        connection
            .execute_batch(&format!(
                "PRAGMA synchronous=FULL; PRAGMA cache_size=-{CACHE_KIB}; PRAGMA temp_store=MEMORY;"
            ))
            .expect("SQLite connection pragmas should apply");
        connection
    }
}

impl Engine for SqliteEngine {
    type Writer = Connection;

    fn open_writer(&self, _writer_id: usize) -> Self::Writer {
        self.open_connection()
    }

    fn attempt(&self, writer: &mut Self::Writer, mutations: &[Mutation]) -> Attempt {
        let transaction = match writer.transaction_with_behavior(TransactionBehavior::Immediate) {
            Ok(transaction) => transaction,
            Err(error) => return classify(error),
        };
        for mutation in mutations {
            if let Err(error) = transaction.execute(
                "INSERT INTO kv (k, v) VALUES (?1, ?2) ON CONFLICT(k) DO UPDATE SET v=excluded.v",
                params![mutation.key, mutation.value],
            ) {
                let _ = transaction.rollback();
                return classify(error);
            }
        }
        match transaction.commit() {
            Ok(()) => Attempt::Committed,
            Err(error) => classify(error),
        }
    }

    fn seed(&self, writer: &mut Self::Writer, rows: &[Mutation]) {
        let transaction = writer
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .expect("SQLite seed transaction should begin");
        for row in rows {
            transaction
                .execute(
                    "INSERT INTO kv (k, v) VALUES (?1, ?2) ON CONFLICT(k) DO UPDATE SET v=excluded.v",
                    params![row.key, row.value],
                )
                .expect("SQLite seed row should write");
        }
        transaction.commit().expect("SQLite seed should commit");
    }

    fn read(&self, writer: &mut Self::Writer, key: &[u8]) -> Option<Vec<u8>> {
        writer
            .query_row("SELECT v FROM kv WHERE k=?1", params![key], |row| {
                row.get(0)
            })
            .optional()
            .expect("SQLite point read should succeed")
    }

    fn query(&self, writer: &mut Self::Writer, primary_key: &[u8], limit: usize) -> Vec<Mutation> {
        let mut lower = primary_key.to_vec();
        lower.extend_from_slice(&[0; 8]);
        let mut upper = primary_key.to_vec();
        upper.extend_from_slice(&[u8::MAX; 8]);
        let mut statement = writer
            .prepare("SELECT k, v FROM kv WHERE k >= ?1 AND k <= ?2 ORDER BY k LIMIT ?3")
            .expect("SQLite query should prepare");
        statement
            .query_map(params![lower, upper, limit as i64], |row| {
                Ok(Mutation {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })
            .expect("SQLite query should run")
            .map(|row| row.expect("SQLite row should read"))
            .collect()
    }

    fn count_rows(&self, writer: &mut Self::Writer) -> u64 {
        writer
            .query_row("SELECT count(*) FROM kv", [], |row| row.get(0))
            .expect("SQLite row count should succeed")
    }

    fn settings(&self, writer: &mut Self::Writer) -> Value {
        let journal: String = writer
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode should be available");
        let synchronous: i64 = writer
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .expect("synchronous should be available");
        let cache_size: i64 = writer
            .query_row("PRAGMA cache_size", [], |row| row.get(0))
            .expect("cache size should be available");
        json!({
            "sqlite_version": rusqlite::version(),
            "journal_mode": journal,
            "synchronous": synchronous,
            "cache_size_pages": cache_size,
            "schema": "CREATE TABLE kv (k BLOB PRIMARY KEY, v BLOB NOT NULL) WITHOUT ROWID",
            "busy_timeout_ms": 30000,
        })
    }

    fn metrics(&self) -> Value {
        json!({})
    }

    fn monitor_sample(&self) -> Value {
        json!({})
    }
}

fn classify(error: Error) -> Attempt {
    match error {
        Error::SqliteFailure(code, message)
            if matches!(
                code.code,
                ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
            ) =>
        {
            Attempt::Retryable(RetryKind::Busy, message.unwrap_or_else(|| code.to_string()))
        }
        other => Attempt::Failed(other.to_string()),
    }
}

struct SqliteFactory;

impl EngineFactory for SqliteFactory {
    type Engine = SqliteEngine;

    fn engine_name(_args: &Args) -> String {
        "sqlite-wal".to_owned()
    }

    fn build_info() -> Value {
        json!({
            "database": "SQLite",
            "version": rusqlite::version(),
            "binding": "rusqlite 0.37.0 with bundled SQLite",
        })
    }

    fn create(args: &Args) -> Self::Engine {
        let path = args.data_dir.join("sqlite.db");
        let engine = SqliteEngine { path };
        let connection = engine.open_connection();
        let journal: String = connection
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .expect("SQLite WAL mode should enable");
        assert_eq!(journal.to_ascii_lowercase(), "wal");
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS kv (k BLOB PRIMARY KEY, v BLOB NOT NULL) WITHOUT ROWID;",
            )
            .expect("SQLite table should create");
        engine
    }

    fn reopen(args: &Args) -> Self::Engine {
        Self::create(args)
    }

    fn close(engine: Self::Engine) -> Value {
        json!({ "database_path": engine.path })
    }
}

fn main() {
    driver::bench_main::<SqliteFactory>();
}
