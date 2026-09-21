#[doc(hidden)]
pub mod backend_contract;
mod dibi;
mod memory;
pub mod mock;
mod runtime;
mod trx;
mod turso;

use anyhow::Result;
use bytes::Bytes;
use dibi::{DibiDatabase, DibiTransaction};
pub use libsql_hrana::proto::Value;
use memory::{MemoryDatabase, MemoryTransaction};
use std::collections::HashSet;
use std::future::Future;
pub use trx::{
    ConflictDetails, ConflictKey, DocGet, DocHandle, DocKey, Document, Trx, TrxControl, TrxRead,
    TrxResult,
};
use turso::{TursoDatabase, TursoTransaction};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredDoc {
    pub(crate) data: Bytes,
    pub(crate) version: i64,
}

pub fn text_value(s: impl Into<String>) -> Value {
    Value::Text {
        value: s.into().into(),
    }
}

pub fn integer_value(i: i64) -> Value {
    Value::Integer { value: i }
}

pub enum BatchOp<'a> {
    Put {
        pk: &'a str,
        sk: &'a str,
        data: &'a [u8],
    },
    Delete {
        pk: &'a str,
        sk: &'a str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminScanRequest {
    pub after: Option<(String, String)>,
    pub limit: usize,
    pub pk_prefix: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminDocument {
    pub pk: String,
    pub sk: String,
    pub data: Bytes,
    pub version: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminScanPage {
    pub documents: Vec<AdminDocument>,
    pub next: Option<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminWriteOp {
    Create {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Put {
        pk: String,
        sk: String,
        expected_version: i64,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
        expected_version: i64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminWriteOutcome {
    Applied,
    Conflict(ConflictDetails),
}

pub(crate) enum ConditionalOp {
    Create {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Put {
        pk: String,
        sk: String,
        expected_version: i64,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
        expected_version: i64,
    },
    Check {
        pk: String,
        sk: String,
        expected_version: Option<i64>,
    },
}

pub(crate) enum ConditionalOutcome {
    Applied,
    Conflict(ConflictDetails),
}

pub(crate) fn conditional_key_and_expected(operation: &ConditionalOp) -> (&str, &str, Option<i64>) {
    match operation {
        ConditionalOp::Create { pk, sk, .. } => (pk, sk, None),
        ConditionalOp::Put {
            pk,
            sk,
            expected_version,
            ..
        }
        | ConditionalOp::Delete {
            pk,
            sk,
            expected_version,
        } => (pk, sk, Some(*expected_version)),
        ConditionalOp::Check {
            pk,
            sk,
            expected_version,
        } => (pk, sk, *expected_version),
    }
}

pub(crate) fn validate_conditional_keys(operations: &[ConditionalOp]) -> Result<()> {
    let mut keys = HashSet::with_capacity(operations.len());
    for operation in operations {
        let (pk, sk, _) = conditional_key_and_expected(operation);
        if !keys.insert((pk, sk)) {
            anyhow::bail!("duplicate conditional write key: {pk}/{sk}");
        }
    }
    Ok(())
}

pub struct RawStatement {
    pub sql: String,
    pub args: Vec<Value>,
}

pub struct RawStatementResult {
    pub column_names: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub affected_row_count: u64,
    pub rows_read: u64,
    pub rows_written: u64,
    pub query_duration_ms: f64,
}

pub enum RawTransactionOutcome {
    Committed {
        statement_results: Vec<RawStatementResult>,
    },
    RolledBack {
        failed_statement_index: usize,
        error_message: String,
    },
}

pub fn turso() -> Database {
    let url = std::env::var("TURSO_URL").expect("TURSO_URL must be set");
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");
    turso_with_config(url, auth_token)
}

pub fn turso_with_config(url: String, auth_token: String) -> Database {
    Database {
        inner: DatabaseInner::Turso(TursoDatabase::new(url, auth_token)),
        mock_state: mock::MockState::default(),
    }
}

pub fn memory() -> Database {
    Database {
        inner: DatabaseInner::Memory(MemoryDatabase::new()),
        mock_state: mock::MockState::default(),
    }
}

pub fn dibi() -> Database {
    let url = std::env::var("DIBI_URL").expect("DIBI_URL must be set");
    dibi_with_config(url)
}

pub fn dibi_with_config(url: String) -> Database {
    Database {
        inner: DatabaseInner::Dibi(DibiDatabase::new(url)),
        mock_state: mock::MockState::default(),
    }
}

#[derive(Clone)]
pub struct Database {
    inner: DatabaseInner,
    mock_state: mock::MockState,
}

impl Database {
    pub async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        if let Some(result) = self.mock_state.try_match(mock::MockOp::Get, pk, sk) {
            return match result {
                mock::MockResult::OkGet(data) => Ok(data.map(Bytes::from)),
                mock::MockResult::Err(msg) => Err(anyhow::anyhow!("{}", msg)),
                _ => unreachable!(),
            };
        }
        match &self.inner {
            DatabaseInner::Turso(db) => db.get(pk, sk).await,
            DatabaseInner::Memory(db) => db.get(pk, sk).await,
            DatabaseInner::Dibi(db) => db.get(pk, sk).await,
        }
    }

    pub async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        if let Some(result) = self.mock_state.try_match(mock::MockOp::Put, pk, sk) {
            return match result {
                mock::MockResult::OkVoid => Ok(()),
                mock::MockResult::Err(msg) => Err(anyhow::anyhow!("{}", msg)),
                _ => unreachable!(),
            };
        }
        match &self.inner {
            DatabaseInner::Turso(db) => db.put(pk, sk, data).await,
            DatabaseInner::Memory(db) => db.put(pk, sk, data).await,
            DatabaseInner::Dibi(db) => db.put(pk, sk, data).await,
        }
    }

    pub async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        if let Some(result) = self.mock_state.try_match(mock::MockOp::Delete, pk, sk) {
            return match result {
                mock::MockResult::OkVoid => Ok(()),
                mock::MockResult::Err(msg) => Err(anyhow::anyhow!("{}", msg)),
                _ => unreachable!(),
            };
        }
        match &self.inner {
            DatabaseInner::Turso(db) => db.delete(pk, sk).await,
            DatabaseInner::Memory(db) => db.delete(pk, sk).await,
            DatabaseInner::Dibi(db) => db.delete(pk, sk).await,
        }
    }

    // --- Mock API ---

    pub fn mock_get(&self, pk: &str, sk: &str) -> mock::MockGetBuilder<'_> {
        mock::MockGetBuilder::new(self, pk.to_string(), sk.to_string())
    }

    pub fn mock_put(&self, pk: &str, sk: &str) -> mock::MockPutBuilder<'_> {
        mock::MockPutBuilder::new(self, pk.to_string(), sk.to_string())
    }

    pub fn mock_delete(&self, pk: &str, sk: &str) -> mock::MockDeleteBuilder<'_> {
        mock::MockDeleteBuilder::new(self, pk.to_string(), sk.to_string())
    }

    pub fn clear_mocks(&self) {
        self.mock_state.clear();
    }

    pub(crate) fn add_mock_rule(&self, rule: mock::MockRule) {
        self.mock_state.push(rule);
    }

    #[tracing::instrument(skip_all, fields(pk = %pk.as_ref(), limit = limit))]
    pub async fn query<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        pk: S1,
        after_sk: Option<S2>,
        limit: usize,
    ) -> Result<Vec<(String, Bytes)>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.query(pk, after_sk, limit).await,
            DatabaseInner::Memory(db) => db.query(pk, after_sk, limit).await,
            DatabaseInner::Dibi(db) => db.query(pk, after_sk, limit).await,
        }
    }

    #[tracing::instrument(skip_all, fields(limit = limit))]
    pub async fn scan(
        &self,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<Vec<(String, String, Bytes)>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.scan(after, limit).await,
            DatabaseInner::Memory(db) => db.scan(after, limit).await,
            DatabaseInner::Dibi(db) => db.scan(after, limit).await,
        }
    }

    pub async fn admin_scan(&self, request: AdminScanRequest) -> Result<AdminScanPage> {
        if request.limit == 0 {
            return Ok(AdminScanPage {
                documents: Vec::new(),
                next: request.after,
            });
        }

        if let DatabaseInner::Dibi(db) = &self.inner {
            return db.admin_scan(request).await;
        }

        let page_limit = request.limit;
        let mut cursor = request.after;
        let mut documents = Vec::with_capacity(request.limit);

        loop {
            let page = self
                .scan(
                    cursor.as_ref().map(|(pk, sk)| (pk.as_str(), sk.as_str())),
                    page_limit,
                )
                .await?;

            if page.is_empty() {
                return Ok(AdminScanPage {
                    documents,
                    next: None,
                });
            }

            let last_key = page.last().map(|(pk, sk, _)| (pk.clone(), sk.clone()));
            let page_is_full = page.len() == page_limit;

            for (pk, sk, _data) in page {
                if request
                    .pk_prefix
                    .as_deref()
                    .is_some_and(|prefix| !pk.starts_with(prefix))
                {
                    continue;
                }

                let next = (pk.clone(), sk.clone());
                let Some(stored) = self.get_with_version(&pk, &sk).await? else {
                    continue;
                };
                documents.push(AdminDocument {
                    pk,
                    sk,
                    data: stored.data,
                    version: stored.version,
                });

                if documents.len() >= request.limit {
                    return Ok(AdminScanPage {
                        next: Some(next),
                        documents,
                    });
                }
            }

            cursor = last_key;
            if !page_is_full {
                return Ok(AdminScanPage {
                    documents,
                    next: None,
                });
            }
        }
    }

    pub async fn admin_write_batch(
        &self,
        operations: &[AdminWriteOp],
    ) -> Result<AdminWriteOutcome> {
        if operations.is_empty() {
            return Ok(AdminWriteOutcome::Applied);
        }

        let mut write_keys = HashSet::with_capacity(operations.len());
        for operation in operations {
            let (pk, sk) = match operation {
                AdminWriteOp::Create { pk, sk, .. }
                | AdminWriteOp::Put { pk, sk, .. }
                | AdminWriteOp::Delete { pk, sk, .. } => (pk, sk),
            };
            if !write_keys.insert((pk.as_str(), sk.as_str())) {
                anyhow::bail!("duplicate admin write key: {pk}/{sk}");
            }
        }

        let conditional_operations: Vec<ConditionalOp> = operations
            .iter()
            .map(|operation| match operation {
                AdminWriteOp::Create { pk, sk, data } => ConditionalOp::Create {
                    pk: pk.clone(),
                    sk: sk.clone(),
                    data: data.clone(),
                },
                AdminWriteOp::Put {
                    pk,
                    sk,
                    expected_version,
                    data,
                } => ConditionalOp::Put {
                    pk: pk.clone(),
                    sk: sk.clone(),
                    expected_version: *expected_version,
                    data: data.clone(),
                },
                AdminWriteOp::Delete {
                    pk,
                    sk,
                    expected_version,
                } => ConditionalOp::Delete {
                    pk: pk.clone(),
                    sk: sk.clone(),
                    expected_version: *expected_version,
                },
            })
            .collect();

        let outcome = match &self.inner {
            DatabaseInner::Turso(db) => db.conditional_write_batch(&conditional_operations).await?,
            DatabaseInner::Memory(db) => {
                db.conditional_write_batch(&conditional_operations).await?
            }
            DatabaseInner::Dibi(db) => {
                db.admin_conditional_write_batch(&conditional_operations)
                    .await?
            }
        };
        Ok(match outcome {
            ConditionalOutcome::Applied => AdminWriteOutcome::Applied,
            ConditionalOutcome::Conflict(details) => AdminWriteOutcome::Conflict(details),
        })
    }

    #[tracing::instrument(skip_all, fields(ops = ops.len()))]
    pub async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.batch(ops).await,
            DatabaseInner::Memory(db) => db.batch(ops).await,
            DatabaseInner::Dibi(db) => db.batch(ops).await,
        }
    }

    pub(crate) async fn conditional_write_batch(
        &self,
        operations: &[ConditionalOp],
    ) -> Result<ConditionalOutcome> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.conditional_write_batch(operations).await,
            DatabaseInner::Memory(db) => db.conditional_write_batch(operations).await,
            DatabaseInner::Dibi(db) => db.conditional_write_batch(operations).await,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn transaction(&self) -> Result<Transaction> {
        match &self.inner {
            DatabaseInner::Turso(db) => Ok(Transaction {
                inner: TransactionInner::Turso(db.transaction().await?),
            }),
            DatabaseInner::Memory(db) => Ok(Transaction {
                inner: TransactionInner::Memory(db.transaction().await?),
            }),
            DatabaseInner::Dibi(db) => Ok(Transaction {
                inner: TransactionInner::Dibi(DibiTransaction::new(db.clone())),
            }),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn trx<F, Fut, Out, Cancel, E>(&self, f: F) -> TrxResult<Out, Cancel, E>
    where
        F: FnMut(Trx) -> Fut,
        Fut: Future<Output = Result<TrxControl<Out, Cancel>, E>>,
        E: From<anyhow::Error>,
    {
        trx::run(self.clone(), f).await
    }

    #[tracing::instrument(skip_all, fields(sql = %sql))]
    pub async fn execute_raw(
        &self,
        sql: &str,
        args: Vec<Value>,
        want_rows: bool,
    ) -> Result<Vec<Vec<Value>>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_raw(sql, args, want_rows).await,
            DatabaseInner::Memory(db) => db.execute_raw(sql, args, want_rows).await,
            DatabaseInner::Dibi(_) => {
                anyhow::bail!("raw SQL is only supported by the Turso backend")
            }
        }
    }

    /// Runs the statements as a single transaction (all-or-nothing) and
    /// returns, per statement, the result rows together with the engine's own
    /// `rows_read` / `rows_written` counters — the numbers Turso bills by.
    /// A statement failure rolls the whole transaction back and is reported as
    /// [`RawTransactionOutcome::RolledBack`], not as `Err`.
    ///
    /// Turso backend only; the in-memory test backend rejects it.
    #[tracing::instrument(skip_all, fields(statements = statements.len()))]
    pub async fn execute_raw_transactional(
        &self,
        statements: &[RawStatement],
    ) -> Result<RawTransactionOutcome> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_raw_transactional(statements).await,
            DatabaseInner::Memory(_) => {
                anyhow::bail!("execute_raw_transactional is only supported on the Turso backend")
            }
            DatabaseInner::Dibi(_) => {
                anyhow::bail!("raw SQL is only supported by the Turso backend")
            }
        }
    }

    #[tracing::instrument(skip_all, fields(ops = ops.len()))]
    pub async fn execute_ops(&self, ops: Vec<DbOp>) -> Result<Vec<DbResult>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_ops(ops).await,
            DatabaseInner::Memory(db) => db.execute_ops(ops).await,
            DatabaseInner::Dibi(db) => db.execute_ops(ops).await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub(crate) async fn get_with_version(&self, pk: &str, sk: &str) -> Result<Option<StoredDoc>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.get_with_version(pk, sk).await,
            DatabaseInner::Memory(db) => db.get_with_version(pk, sk).await,
            DatabaseInner::Dibi(db) => db.get_with_version(pk, sk).await,
        }
    }

    #[tracing::instrument(skip_all, fields(reads = keys.len()))]
    pub(crate) async fn batch_get_with_version(
        &self,
        keys: &[(String, String)],
    ) -> Result<Vec<Option<StoredDoc>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        match &self.inner {
            DatabaseInner::Turso(db) => db.batch_get_with_version(keys).await,
            DatabaseInner::Memory(db) => {
                let mut output = Vec::with_capacity(keys.len());
                for (pk, sk) in keys {
                    output.push(db.get_with_version(pk, sk).await?);
                }
                Ok(output)
            }
            DatabaseInner::Dibi(db) => db.batch_get_with_version(keys).await,
        }
    }
}

#[derive(Clone)]
enum DatabaseInner {
    Turso(TursoDatabase),
    Memory(MemoryDatabase),
    Dibi(DibiDatabase),
}

pub struct Transaction {
    inner: TransactionInner,
}

enum TransactionInner {
    Turso(TursoTransaction),
    Memory(MemoryTransaction),
    Dibi(DibiTransaction),
}

impl Transaction {
    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub async fn get(&mut self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.get(pk, sk).await,
            TransactionInner::Memory(tx) => tx.get(pk, sk).await,
            TransactionInner::Dibi(tx) => tx.get(pk, sk).await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk, bytes = data.len()))]
    pub async fn put(&mut self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.put(pk, sk, data).await,
            TransactionInner::Memory(tx) => tx.put(pk, sk, data).await,
            TransactionInner::Dibi(tx) => tx.put(pk, sk, data).await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub async fn delete(&mut self, pk: &str, sk: &str) -> Result<()> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.delete(pk, sk).await,
            TransactionInner::Memory(tx) => tx.delete(pk, sk).await,
            TransactionInner::Dibi(tx) => tx.delete(pk, sk).await,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn commit(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.commit().await,
            TransactionInner::Memory(tx) => tx.commit().await,
            TransactionInner::Dibi(tx) => tx.commit().await,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn rollback(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.rollback().await,
            TransactionInner::Memory(tx) => tx.rollback().await,
            TransactionInner::Dibi(tx) => tx.rollback().await,
        }
    }
}

#[derive(Debug)]
pub enum DbOp {
    Get {
        pk: String,
        sk: String,
    },
    Query {
        pk: String,
        after_sk: Option<String>,
        limit: Option<usize>,
    },
    Put {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
    },
}

pub enum DbResult {
    Single(Option<Bytes>),
    Multiple(Vec<(String, Bytes)>),
    Done,
}

pub type DbResultParser<O> = Box<dyn FnOnce(&mut std::vec::IntoIter<DbResult>) -> Result<O> + Send>;

pub struct Prepared<O> {
    pub ops: Vec<DbOp>,
    pub parse: DbResultParser<O>,
}

#[allow(async_fn_in_trait)]
pub trait DbRequest: Sized {
    type Output;
    fn prepare(self) -> Prepared<Self::Output>;

    async fn send_with(self, db: &Database) -> Result<Self::Output> {
        let prepared = self.prepare();
        let results = db.execute_ops(prepared.ops).await?;
        let mut iter = results.into_iter();
        (prepared.parse)(&mut iter)
    }
}

macro_rules! impl_db_request_tuple {
    ($($T:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($T: DbRequest),+> DbRequest for ($($T,)+)
        where $($T::Output: 'static),+
        {
            type Output = ($($T::Output,)+);
            fn prepare(self) -> Prepared<Self::Output> {
                let ($($T,)+) = self;
                $(let $T = $T.prepare();)+
                let mut ops = Vec::new();
                $(ops.extend($T.ops);)+
                Prepared {
                    ops,
                    parse: Box::new(move |iter| {
                        Ok(($(($T.parse)(iter)?,)+))
                    }),
                }
            }
        }
    };
}

impl_db_request_tuple!(A);
impl_db_request_tuple!(A, B);
impl_db_request_tuple!(A, B, C);
impl_db_request_tuple!(A, B, C, D);
impl_db_request_tuple!(A, B, C, D, E);
impl_db_request_tuple!(A, B, C, D, E, F);
impl_db_request_tuple!(A, B, C, D, E, F, G);
impl_db_request_tuple!(A, B, C, D, E, F, G, H);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

impl<T: DbRequest> DbRequest for Vec<T>
where
    T::Output: 'static,
{
    type Output = Vec<T::Output>;
    fn prepare(self) -> Prepared<Self::Output> {
        let mut all_ops = Vec::new();
        let mut parsers: Vec<DbResultParser<T::Output>> = Vec::new();
        for item in self {
            let p = item.prepare();
            all_ops.extend(p.ops);
            parsers.push(p.parse);
        }
        Prepared {
            ops: all_ops,
            parse: Box::new(move |iter| parsers.into_iter().map(|p| p(iter)).collect()),
        }
    }
}
