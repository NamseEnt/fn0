mod memory;
pub mod mock;
mod trx;
mod turso;

use anyhow::Result;
use bytes::Bytes;
pub use libsql_hrana::proto::Value;
use memory::{MemoryDatabase, MemoryTransaction};
use std::future::Future;
pub use trx::{
    ConflictDetails, ConflictKey, DocGet, DocHandle, DocKey, Document, Trx, TrxControl, TrxRead,
    TrxResult,
};
use turso::{StoredDoc, TursoDatabase, TursoTransaction};

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

#[derive(Clone)]
pub enum WriteOp {
    Insert {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Update {
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

pub struct CommitOutcome {
    pub affected_counts: Vec<u64>,
    pub conflict: Option<ConflictInfo>,
}

pub struct ConflictInfo {
    pub step_index: usize,
    pub message: String,
}

pub fn turso() -> Database {
    let url = std::env::var("TURSO_URL").unwrap_or("http://127.0.0.1:8080".to_string());
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default();
    turso_with_config(url, auth_token)
}

pub fn turso_with_config(url: String, auth_token: String) -> Database {
    Database {
        inner: DatabaseInner::Turso(TursoDatabase::new(url, auth_token)),
        mock_state: mock::MockState::default(),
    }
}

/// Creates an in-memory database for unit testing.
///
/// This backend uses a `BTreeMap` to store data entirely in memory,
/// requiring no external server (sqld, Turso, etc.). It supports all
/// standard forte-db operations including optimistic transactions (`trx`).
///
/// ```ignore
/// let db = forte_db::memory();
/// db.put("pk", "sk", b"data").await?;
/// ```
pub fn memory() -> Database {
    Database {
        inner: DatabaseInner::Memory(MemoryDatabase::new()),
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
        }
    }

    #[tracing::instrument(skip_all, fields(ops = ops.len()))]
    pub async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.batch(ops).await,
            DatabaseInner::Memory(db) => db.batch(ops).await,
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
        }
    }

    #[tracing::instrument(skip_all, fields(ops = ops.len()))]
    pub async fn execute_ops(&self, ops: Vec<DbOp>) -> Result<Vec<DbResult>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_ops(ops).await,
            DatabaseInner::Memory(db) => db.execute_ops(ops).await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub(crate) async fn get_with_version(&self, pk: &str, sk: &str) -> Result<Option<StoredDoc>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.get_with_version(pk, sk).await,
            DatabaseInner::Memory(db) => db.get_with_version(pk, sk).await,
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
        let mut out = Vec::with_capacity(keys.len());
        for (pk, sk) in keys {
            out.push(self.get_with_version(pk, sk).await?);
        }
        Ok(out)
    }

    #[tracing::instrument(skip_all, fields(reads = keys.len()))]
    pub(crate) async fn begin_immediate_with_reads(
        &self,
        keys: &[(String, String)],
    ) -> Result<(Transaction, Vec<Option<StoredDoc>>)> {
        match &self.inner {
            DatabaseInner::Turso(db) => {
                let (tx, docs) = db.begin_immediate_with_reads(keys).await?;
                Ok((
                    Transaction {
                        inner: TransactionInner::Turso(tx),
                    },
                    docs,
                ))
            }
            DatabaseInner::Memory(db) => {
                let (tx, docs) = db.begin_immediate_with_reads(keys).await?;
                Ok((
                    Transaction {
                        inner: TransactionInner::Memory(tx),
                    },
                    docs,
                ))
            }
        }
    }
}

#[derive(Clone)]
enum DatabaseInner {
    Turso(TursoDatabase),
    Memory(MemoryDatabase),
}

pub struct Transaction {
    inner: TransactionInner,
}

enum TransactionInner {
    Turso(TursoTransaction),
    Memory(MemoryTransaction),
}

impl Transaction {
    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub async fn get(&mut self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.get(pk, sk).await,
            TransactionInner::Memory(tx) => tx.get(pk, sk).await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk, bytes = data.len()))]
    pub async fn put(&mut self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.put(pk, sk, data).await,
            TransactionInner::Memory(tx) => tx.put(pk, sk, data).await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub async fn delete(&mut self, pk: &str, sk: &str) -> Result<()> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.delete(pk, sk).await,
            TransactionInner::Memory(tx) => tx.delete(pk, sk).await,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn commit(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.commit().await,
            TransactionInner::Memory(tx) => tx.commit().await,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn rollback(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.rollback().await,
            TransactionInner::Memory(tx) => tx.rollback().await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub(crate) async fn get_with_version(
        &mut self,
        pk: &str,
        sk: &str,
    ) -> Result<Option<StoredDoc>> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.get_with_version(pk, sk).await,
            TransactionInner::Memory(tx) => tx.get_with_version(pk, sk).await,
        }
    }

    #[tracing::instrument(skip_all, fields(sql = %sql))]
    pub(crate) async fn execute_stmt(
        &mut self,
        sql: &str,
        args: Vec<Value>,
        want_rows: bool,
    ) -> Result<libsql_hrana::proto::StmtResult> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.execute_stmt(sql, args, want_rows).await,
            TransactionInner::Memory(tx) => tx.execute_stmt(sql, args, want_rows).await,
        }
    }

    #[tracing::instrument(skip_all, fields(writes = writes.len()))]
    pub(crate) async fn apply_writes_and_commit(
        &mut self,
        writes: &[WriteOp],
    ) -> Result<CommitOutcome> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.apply_writes_and_commit(writes).await,
            TransactionInner::Memory(tx) => tx.apply_writes_and_commit(writes).await,
        }
    }

    #[tracing::instrument(skip_all, fields(reads = keys.len()))]
    pub(crate) async fn batch_get_with_version(
        &mut self,
        keys: &[(String, String)],
    ) -> Result<Vec<Option<StoredDoc>>> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.batch_get_with_version(keys).await,
            TransactionInner::Memory(tx) => tx.batch_get_with_version(keys).await,
        }
    }
}

pub fn default_db() -> Database {
    #[cfg(feature = "dev-test")]
    {
        memory()
    }
    #[cfg(not(feature = "dev-test"))]
    {
        turso()
    }
}

pub async fn trx<F, Fut, Out, Cancel, E>(f: F) -> TrxResult<Out, Cancel, E>
where
    F: FnMut(Trx) -> Fut,
    Fut: Future<Output = Result<TrxControl<Out, Cancel>, E>>,
    E: From<anyhow::Error>,
{
    default_db().trx(f).await
}

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

    async fn send(self) -> Result<Self::Output> {
        self.send_with(&default_db()).await
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
