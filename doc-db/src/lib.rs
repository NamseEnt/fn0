#[cfg(all(test, not(target_arch = "wasm32")))]
mod backend_contract;
#[cfg(not(target_arch = "wasm32"))]
mod dodb;
mod memory;
pub mod mock;
mod remote;
mod runtime;
mod transaction;
mod trx;
mod turso;

use anyhow::Result;
use bytes::Bytes;
pub use doc_db_protocol::DocDbRevision;
use doc_db_protocol::{
    DocDbCondition, DocDbDocument, DocDbError, DocDbKey, DocDbMutation, DocDbObservedDocument,
    DocDbOperation, DocDbResponse, DocDbResult, DocDbTransactOutcome,
};
#[cfg(not(target_arch = "wasm32"))]
pub use dodb::{
    DodbConfig, DodbConnection, FN0_CONTROL_DODB_TENANT_ID, dodb_tenant_id, project_tenant_id,
};
pub use libsql_hrana::proto::Value;
use memory::{MemoryDatabase, MemoryTransaction};
use remote::RemoteDatabase;
use std::future::Future;
pub(crate) use transaction::{
    ObservedDocument, TransactCondition, TransactConflict, TransactMutation, TransactOutcome,
    TransactRequest, revision_from_backend, revision_to_backend, validate_transact_request,
};
pub use trx::{
    ConflictDetails, ConflictKey, DocGet, DocHandle, DocKey, Document, Trx, TrxControl, TrxRead,
    TrxResult,
};
use turso::{TursoDatabase, TursoTransaction};

pub fn text_value(s: impl Into<String>) -> Value {
    Value::Text {
        value: s.into().into(),
    }
}

pub fn integer_value(i: i64) -> Value {
    Value::Integer { value: i }
}

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

pub fn semantic() -> Database {
    let url = std::env::var("FN0_DOC_DB_URL").expect("FN0_DOC_DB_URL must be set");
    semantic_with_config(url)
}

/// Creates the backend-neutral document database used by normal fn0 guests.
///
/// The runtime supplies the endpoint and routes it through the semantic doc-db
/// RPC boundary. The host backend is intentionally not part of this API.
pub fn database() -> Database {
    semantic()
}

pub fn semantic_with_config(url: String) -> Database {
    Database {
        inner: DatabaseInner::Remote(RemoteDatabase::new(url)),
        mock_state: mock::MockState::default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn dodb_with_connection(connection: &DodbConnection, project_id: &str) -> Result<Database> {
    Ok(Database {
        inner: DatabaseInner::Dodb(connection.database(project_id)?),
        mock_state: mock::MockState::default(),
    })
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
            DatabaseInner::Remote(db) => db.get(pk, sk).await,
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(db) => db.get(pk, sk).await,
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
            DatabaseInner::Remote(db) => db.put(pk, sk, data).await,
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(db) => db.put(pk, sk, data).await,
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
            DatabaseInner::Remote(db) => db.delete(pk, sk).await,
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(db) => db.delete(pk, sk).await,
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
            DatabaseInner::Remote(db) => {
                db.query(pk.as_ref(), after_sk.as_ref().map(AsRef::as_ref), limit)
                    .await
            }
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(db) => {
                db.query(pk.as_ref(), after_sk.as_ref().map(AsRef::as_ref), limit)
                    .await
            }
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
            DatabaseInner::Remote(db) => db.scan(after, limit).await,
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(db) => db.scan(after, limit).await,
        }
    }

    pub async fn execute_semantic(
        &self,
        request: doc_db_protocol::DocDbRequest,
    ) -> Result<doc_db_protocol::DocDbResponse> {
        if matches!(&self.inner, DatabaseInner::Remote(_)) {
            anyhow::bail!("semantic execution is only available on a host database")
        }

        match request.operation {
            DocDbOperation::Get { key } => {
                let data = self.get(&key.pk, &key.sk).await?;
                Ok(DocDbResponse::new(DocDbResult::Get {
                    data: data.map(|value| doc_db_protocol::BinaryDocument {
                        data: value.to_vec(),
                    }),
                }))
            }
            DocDbOperation::Put { key, data } => {
                self.put(&key.pk, &key.sk, &data).await?;
                Ok(DocDbResponse::new(DocDbResult::Put))
            }
            DocDbOperation::Delete { key } => {
                self.delete(&key.pk, &key.sk).await?;
                Ok(DocDbResponse::new(DocDbResult::Delete))
            }
            DocDbOperation::Query {
                pk,
                after_sk,
                limit,
            } => {
                let limit = match semantic_limit(limit) {
                    Ok(limit) => limit,
                    Err(message) => {
                        return Ok(DocDbResponse::error(DocDbError::InvalidRequest { message }));
                    }
                };
                let documents = self
                    .query(&pk, after_sk.as_deref(), limit)
                    .await?
                    .into_iter()
                    .map(|(sk, data)| DocDbDocument {
                        key: DocDbKey::new(pk.clone(), sk),
                        data: data.to_vec(),
                    })
                    .collect();
                Ok(DocDbResponse::new(DocDbResult::Query { documents }))
            }
            DocDbOperation::Scan { after, limit } => {
                let limit = match semantic_limit(limit) {
                    Ok(limit) => limit,
                    Err(message) => {
                        return Ok(DocDbResponse::error(DocDbError::InvalidRequest { message }));
                    }
                };
                let after_refs = after.as_ref().map(|key| (key.pk.as_str(), key.sk.as_str()));
                let documents = self
                    .scan(after_refs, limit)
                    .await?
                    .into_iter()
                    .map(|(pk, sk, data)| DocDbDocument {
                        key: DocDbKey::new(pk, sk),
                        data: data.to_vec(),
                    })
                    .collect();
                Ok(DocDbResponse::new(DocDbResult::Scan { documents }))
            }
            DocDbOperation::GetObserved { key } => {
                let document = match self.get_observed(&key.pk, &key.sk).await? {
                    ObservedDocument::Present { data, revision } => {
                        DocDbObservedDocument::Present {
                            data: data.to_vec(),
                            revision,
                        }
                    }
                    ObservedDocument::Missing { revision } => {
                        DocDbObservedDocument::Missing { revision }
                    }
                };
                Ok(DocDbResponse::new(DocDbResult::GetObserved { document }))
            }
            DocDbOperation::Transact {
                conditions,
                mutations,
            } => {
                let conditions = conditions
                    .into_iter()
                    .map(|condition| match condition {
                        DocDbCondition::RevisionEquals {
                            key,
                            expected_revision,
                        } => TransactCondition::RevisionEquals {
                            pk: key.pk,
                            sk: key.sk,
                            expected_revision,
                        },
                        DocDbCondition::Exists { key } => TransactCondition::Exists {
                            pk: key.pk,
                            sk: key.sk,
                        },
                        DocDbCondition::NotExists { key } => TransactCondition::NotExists {
                            pk: key.pk,
                            sk: key.sk,
                        },
                    })
                    .collect::<Vec<_>>();
                let mutations = mutations
                    .into_iter()
                    .map(|mutation| match mutation {
                        DocDbMutation::Put { key, data } => TransactMutation::Put {
                            pk: key.pk,
                            sk: key.sk,
                            data,
                        },
                        DocDbMutation::Delete { key } => TransactMutation::Delete {
                            pk: key.pk,
                            sk: key.sk,
                        },
                    })
                    .collect::<Vec<_>>();
                let request = TransactRequest {
                    conditions,
                    mutations,
                };
                if let Err(error) = validate_transact_request(&request) {
                    return Ok(DocDbResponse::error(DocDbError::InvalidRequest {
                        message: error.to_string(),
                    }));
                }
                let outcome = self.transact(&request).await?;
                let outcome = match outcome.conflict {
                    Some(conflict) => DocDbTransactOutcome::Conflict {
                        condition_index: conflict.condition_index,
                    },
                    None => DocDbTransactOutcome::Committed,
                };
                Ok(DocDbResponse::new(DocDbResult::Transact { outcome }))
            }
            DocDbOperation::AdminPurgeProject { .. } => {
                Ok(DocDbResponse::error(DocDbError::InvalidRequest {
                    message: "admin purge must be handled by the trusted host service".to_string(),
                }))
            }
        }
    }

    pub async fn admin_purge_project(&self, project_id: &str) -> Result<u64> {
        match &self.inner {
            DatabaseInner::Remote(db) => db.admin_purge_project(project_id).await,
            _ => anyhow::bail!("admin project purge requires the semantic remote database"),
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
            DatabaseInner::Remote(_) => {
                anyhow::bail!("explicit transactions are not supported by semantic doc-db RPC")
            }
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(_) => {
                anyhow::bail!("explicit transactions are not supported by the dodb backend")
            }
        }
    }

    #[tracing::instrument(
        skip_all,
        fields(conditions = request.conditions.len(), mutations = request.mutations.len())
    )]
    pub(crate) async fn transact(&self, request: &TransactRequest) -> Result<TransactOutcome> {
        validate_transact_request(request)?;
        match &self.inner {
            DatabaseInner::Turso(db) => db.transact(request).await,
            DatabaseInner::Memory(db) => db.transact(request).await,
            DatabaseInner::Remote(db) => db.transact(request).await,
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(db) => db.transact(request).await,
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
            DatabaseInner::Remote(_) => {
                anyhow::bail!("raw SQL is not supported by semantic doc-db RPC")
            }
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(_) => {
                anyhow::bail!("raw SQL is not supported by the dodb backend")
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
            DatabaseInner::Turso(db) => db.execute_raw_transactional(statements, true).await,
            DatabaseInner::Memory(_) => {
                anyhow::bail!("execute_raw_transactional is only supported on the Turso backend")
            }
            DatabaseInner::Remote(_) => {
                anyhow::bail!("raw SQL is not supported by semantic doc-db RPC")
            }
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(_) => {
                anyhow::bail!("raw SQL is not supported by the dodb backend")
            }
        }
    }

    #[tracing::instrument(skip_all, fields(statements = statements.len()))]
    pub async fn execute_raw_transactional_readonly(
        &self,
        statements: &[RawStatement],
    ) -> Result<RawTransactionOutcome> {
        if statements.iter().any(|statement| {
            !statement
                .sql
                .trim_start()
                .to_ascii_uppercase()
                .starts_with("SELECT ")
        }) {
            anyhow::bail!("execute_raw_transactional_readonly accepts SELECT statements only")
        }
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_raw_transactional(statements, false).await,
            DatabaseInner::Memory(_) => {
                anyhow::bail!(
                    "execute_raw_transactional_readonly is only supported on the Turso backend"
                )
            }
            DatabaseInner::Remote(_) => {
                anyhow::bail!("raw SQL is not supported by semantic doc-db RPC")
            }
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(_) => {
                anyhow::bail!("raw SQL is not supported by the dodb backend")
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) async fn execute_op(&self, op: DbOp) -> Result<DbResult> {
        match op {
            DbOp::Get { pk, sk } => self.get(&pk, &sk).await.map(DbResult::Single),
            DbOp::Query {
                pk,
                after_sk,
                limit,
            } => self
                .query(&pk, after_sk.as_deref(), limit.unwrap_or(usize::MAX))
                .await
                .map(DbResult::Multiple),
            DbOp::Put { pk, sk, data } => {
                self.put(&pk, &sk, &data).await?;
                Ok(DbResult::Done)
            }
            DbOp::Delete { pk, sk } => {
                self.delete(&pk, &sk).await?;
                Ok(DbResult::Done)
            }
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub(crate) async fn get_observed(&self, pk: &str, sk: &str) -> Result<ObservedDocument> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.get_observed(pk, sk).await,
            DatabaseInner::Memory(db) => db.get_observed(pk, sk).await,
            DatabaseInner::Remote(db) => db.get_observed(pk, sk).await,
            #[cfg(not(target_arch = "wasm32"))]
            DatabaseInner::Dodb(db) => db.get_observed(pk, sk).await,
        }
    }
}

fn semantic_limit(limit: u64) -> std::result::Result<usize, String> {
    usize::try_from(limit).map_err(|_| "limit does not fit the host usize".to_string())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod semantic_tests {
    use super::*;
    use doc_db_protocol::{
        DocDbCondition, DocDbKey, DocDbMutation, DocDbObservedDocument, DocDbOperation,
        DocDbRequest, DocDbResult, DocDbRevision, DocDbTransactOutcome,
    };
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::Barrier;

    static DATABASE_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[derive(serde::Deserialize, serde::Serialize)]
    struct TransactionTestDoc {
        id: String,
    }

    impl Document for TransactionTestDoc {
        fn key(&self) -> DocKey {
            DocKey::new("TransactionTestDoc", format!("id={}", self.id))
        }
    }

    async fn read_http_body(stream: &mut TcpStream) -> Vec<u8> {
        let mut bytes = Vec::new();
        let header_end;
        loop {
            let mut chunk = [0u8; 4096];
            let read = stream.read(&mut chunk).await.unwrap();
            assert!(read > 0);
            bytes.extend_from_slice(&chunk[..read]);
            if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                header_end = end + 4;
                break;
            }
        }
        let headers = String::from_utf8_lossy(&bytes[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then_some(value.trim())
            })
            .unwrap()
            .parse::<usize>()
            .unwrap();
        while bytes.len() < header_end + content_length {
            let mut chunk = [0u8; 4096];
            let read = stream.read(&mut chunk).await.unwrap();
            assert!(read > 0);
            bytes.extend_from_slice(&chunk[..read]);
        }
        bytes[header_end..header_end + content_length].to_vec()
    }

    async fn start_remote_server(
        responses: Vec<DocDbResponse>,
    ) -> (String, tokio::task::JoinHandle<Vec<DocDbOperation>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let mut operations = Vec::new();
            for response in responses {
                let (mut stream, _) = listener.accept().await.unwrap();
                let body = read_http_body(&mut stream).await;
                let request = doc_db_protocol::decode_request(&body).unwrap();
                operations.push(request.operation);
                let response_body = doc_db_protocol::encode_response(&response).unwrap();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response_body.len()
                );
                stream.write_all(header.as_bytes()).await.unwrap();
                stream.write_all(&response_body).await.unwrap();
            }
            operations
        });
        (format!("http://{address}/rpc"), handle)
    }

    async fn start_barrier_remote_server() -> (String, tokio::task::JoinHandle<Vec<DocDbOperation>>)
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let operations = Arc::new(Mutex::new(Vec::new()));
        let handle = tokio::spawn(async move {
            let mut handlers = Vec::new();
            for _ in 0..2 {
                let (stream, _) = listener.accept().await.unwrap();
                let barrier = barrier.clone();
                let operations = operations.clone();
                handlers.push(tokio::spawn(async move {
                    let mut stream = stream;
                    let body = read_http_body(&mut stream).await;
                    let request = doc_db_protocol::decode_request(&body).unwrap();
                    let response = match &request.operation {
                        DocDbOperation::Get { key } => DocDbResponse::new(DocDbResult::Get {
                            data: Some(doc_db_protocol::BinaryDocument {
                                data: key.sk.as_bytes().to_vec(),
                            }),
                        }),
                        operation => panic!("unexpected operation: {operation:?}"),
                    };
                    operations.lock().unwrap().push(request.operation);
                    barrier.wait().await;
                    let response_body = doc_db_protocol::encode_response(&response).unwrap();
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        response_body.len()
                    );
                    stream.write_all(header.as_bytes()).await.unwrap();
                    stream.write_all(&response_body).await.unwrap();
                }));
            }
            for handler in handlers {
                handler.await.unwrap();
            }
            operations.lock().unwrap().clone()
        });
        (format!("http://{address}/rpc"), handle)
    }

    #[tokio::test]
    async fn maps_semantic_operations_to_the_backend_contract() {
        let database = memory();
        let binary_data = vec![0, 1, 127, 128, 255, 0];

        let put = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Put {
                key: DocDbKey::new("pk", "b"),
                data: binary_data.clone(),
            }))
            .await
            .unwrap();
        assert_eq!(put.result, DocDbResult::Put);

        database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Put {
                key: DocDbKey::new("pk", "a"),
                data: b"a".to_vec(),
            }))
            .await
            .unwrap();
        database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Put {
                key: DocDbKey::new("other", "a"),
                data: b"other".to_vec(),
            }))
            .await
            .unwrap();

        let get = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Get {
                key: DocDbKey::new("pk", "b"),
            }))
            .await
            .unwrap();
        assert_eq!(
            get.result,
            DocDbResult::Get {
                data: Some(doc_db_protocol::BinaryDocument { data: binary_data })
            }
        );

        let query = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Query {
                pk: "pk".to_string(),
                after_sk: Some("a".to_string()),
                limit: 1,
            }))
            .await
            .unwrap();
        assert_eq!(
            query.result,
            DocDbResult::Query {
                documents: vec![doc_db_protocol::DocDbDocument {
                    key: DocDbKey::new("pk", "b"),
                    data: vec![0, 1, 127, 128, 255, 0],
                }]
            }
        );

        let scan = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Scan {
                after: Some(DocDbKey::new("other", "a")),
                limit: 1,
            }))
            .await
            .unwrap();
        assert_eq!(
            scan.result,
            DocDbResult::Scan {
                documents: vec![doc_db_protocol::DocDbDocument {
                    key: DocDbKey::new("pk", "a"),
                    data: b"a".to_vec(),
                }]
            }
        );

        let observed = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::GetObserved {
                key: DocDbKey::new("pk", "a"),
            }))
            .await
            .unwrap();
        let DocDbResult::GetObserved { document } = &observed.result else {
            panic!("expected observed result");
        };
        assert!(matches!(document, DocDbObservedDocument::Present { .. }));
        let DocDbObservedDocument::Present { revision, .. } = document else {
            panic!("expected present observation");
        };
        assert_eq!(*revision, DocDbRevision::new(0));
    }

    #[tokio::test]
    async fn maps_transaction_conditions_and_mutations_and_conflict_condition() {
        let database = memory();
        for (sk, data) in [
            ("version", b"version" as &[u8]),
            ("update", b"old" as &[u8]),
            ("delete", b"delete" as &[u8]),
        ] {
            database.put("pk", sk, data).await.unwrap();
        }

        let committed = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Transact {
                conditions: vec![
                    DocDbCondition::RevisionEquals {
                        key: DocDbKey::new("pk", "version"),
                        expected_revision: DocDbRevision::new(0),
                    },
                    DocDbCondition::NotExists {
                        key: DocDbKey::new("pk", "missing"),
                    },
                    DocDbCondition::NotExists {
                        key: DocDbKey::new("pk", "insert"),
                    },
                    DocDbCondition::RevisionEquals {
                        key: DocDbKey::new("pk", "update"),
                        expected_revision: DocDbRevision::new(0),
                    },
                    DocDbCondition::RevisionEquals {
                        key: DocDbKey::new("pk", "delete"),
                        expected_revision: DocDbRevision::new(0),
                    },
                ],
                mutations: vec![
                    DocDbMutation::Put {
                        key: DocDbKey::new("pk", "insert"),
                        data: b"insert".to_vec(),
                    },
                    DocDbMutation::Put {
                        key: DocDbKey::new("pk", "update"),
                        data: b"new".to_vec(),
                    },
                    DocDbMutation::Delete {
                        key: DocDbKey::new("pk", "delete"),
                    },
                ],
            }))
            .await
            .unwrap();
        assert_eq!(
            committed.result,
            DocDbResult::Transact {
                outcome: DocDbTransactOutcome::Committed
            }
        );
        assert_eq!(
            database.get("pk", "insert").await.unwrap(),
            Some(Bytes::from_static(b"insert"))
        );
        assert_eq!(
            database.get("pk", "update").await.unwrap(),
            Some(Bytes::from_static(b"new"))
        );
        assert_eq!(database.get("pk", "delete").await.unwrap(), None);

        let conflict = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Transact {
                conditions: vec![DocDbCondition::RevisionEquals {
                    key: DocDbKey::new("pk", "update"),
                    expected_revision: DocDbRevision::new(0),
                }],
                mutations: vec![],
            }))
            .await
            .unwrap();
        assert_eq!(
            conflict.result,
            DocDbResult::Transact {
                outcome: DocDbTransactOutcome::Conflict { condition_index: 0 }
            }
        );
    }

    #[tokio::test]
    async fn tuple_send_with_uses_concurrent_single_operation_requests() {
        struct RawGet(&'static str);

        impl DbRequest for RawGet {
            type Output = Option<Vec<u8>>;

            fn prepare(self) -> Prepared<Self::Output> {
                Prepared {
                    ops: vec![DbOp::Get {
                        pk: "pk".to_string(),
                        sk: self.0.to_string(),
                    }],
                    parse: Box::new(|iter| match iter.next().unwrap() {
                        DbResult::Single(value) => Ok(value.map(|bytes| bytes.to_vec())),
                        result => panic!("unexpected result: {result:?}"),
                    }),
                }
            }
        }

        let (url, server) = start_barrier_remote_server().await;
        let database = semantic_with_config(url);
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            (RawGet("first"), RawGet("second")).send_with(&database),
        )
        .await
        .expect("tuple requests should be concurrent")
        .unwrap();
        assert_eq!(result, (Some(b"first".to_vec()), Some(b"second".to_vec())));

        let operations = server.await.unwrap();
        assert_eq!(operations.len(), 2);
        assert!(
            operations
                .iter()
                .all(|operation| matches!(operation, DocDbOperation::Get { .. }))
        );
    }

    #[tokio::test]
    async fn tuple_send_with_settles_other_operations_before_returning_error() {
        struct FailingGet;

        impl DbRequest for FailingGet {
            type Output = Option<Vec<u8>>;

            fn prepare(self) -> Prepared<Self::Output> {
                Prepared {
                    ops: vec![DbOp::Get {
                        pk: "pk".to_string(),
                        sk: "failure".to_string(),
                    }],
                    parse: Box::new(|iter| match iter.next().unwrap() {
                        DbResult::Single(value) => Ok(value.map(|bytes| bytes.to_vec())),
                        result => panic!("unexpected result: {result:?}"),
                    }),
                }
            }
        }

        struct Put;

        impl DbRequest for Put {
            type Output = ();

            fn prepare(self) -> Prepared<Self::Output> {
                Prepared {
                    ops: vec![DbOp::Put {
                        pk: "pk".to_string(),
                        sk: "completed".to_string(),
                        data: b"done".to_vec(),
                    }],
                    parse: Box::new(|iter| match iter.next().unwrap() {
                        DbResult::Done => Ok(()),
                        result => panic!("unexpected result: {result:?}"),
                    }),
                }
            }
        }

        let database = memory();
        database
            .mock_get("pk", "failure")
            .returns_err("expected failure");
        let error = (FailingGet, Put).send_with(&database).await.unwrap_err();
        assert_eq!(error.to_string(), "expected failure");
        assert_eq!(
            database.get("pk", "completed").await.unwrap().as_deref(),
            Some(b"done".as_slice())
        );
    }

    #[tokio::test]
    async fn rejects_duplicate_transaction_keys_as_invalid_requests() {
        let database = memory();

        let duplicate_condition = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Transact {
                conditions: vec![
                    DocDbCondition::Exists {
                        key: DocDbKey::new("pk", "duplicate"),
                    },
                    DocDbCondition::NotExists {
                        key: DocDbKey::new("pk", "duplicate"),
                    },
                ],
                mutations: vec![],
            }))
            .await
            .unwrap();
        assert!(matches!(
            duplicate_condition.result,
            DocDbResult::Error {
                error: doc_db_protocol::DocDbError::InvalidRequest { .. }
            }
        ));

        let duplicate_mutation = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Transact {
                conditions: vec![],
                mutations: vec![
                    DocDbMutation::Put {
                        key: DocDbKey::new("pk", "duplicate"),
                        data: b"one".to_vec(),
                    },
                    DocDbMutation::Delete {
                        key: DocDbKey::new("pk", "duplicate"),
                    },
                ],
            }))
            .await
            .unwrap();
        assert!(matches!(
            duplicate_mutation.result,
            DocDbResult::Error {
                error: doc_db_protocol::DocDbError::InvalidRequest { .. }
            }
        ));
    }

    #[tokio::test]
    async fn converts_single_remote_responses_to_database_results() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let responses = vec![
            DocDbResponse::new(DocDbResult::Get {
                data: Some(doc_db_protocol::BinaryDocument {
                    data: vec![0, 128, 255],
                }),
            }),
            DocDbResponse::new(DocDbResult::Put),
            DocDbResponse::new(DocDbResult::Delete),
            DocDbResponse::new(DocDbResult::Query {
                documents: vec![doc_db_protocol::DocDbDocument {
                    key: DocDbKey::new("pk", "query-sk"),
                    data: b"query".to_vec(),
                }],
            }),
            DocDbResponse::new(DocDbResult::Scan {
                documents: vec![doc_db_protocol::DocDbDocument {
                    key: DocDbKey::new("scan-pk", "scan-sk"),
                    data: b"scan".to_vec(),
                }],
            }),
            DocDbResponse::new(DocDbResult::GetObserved {
                document: DocDbObservedDocument::Present {
                    data: b"observed".to_vec(),
                    revision: DocDbRevision::new(7),
                },
            }),
            DocDbResponse::new(DocDbResult::Transact {
                outcome: DocDbTransactOutcome::Conflict { condition_index: 4 },
            }),
        ];
        let (url, server) = start_remote_server(responses).await;
        let database = semantic_with_config(url);

        assert_eq!(
            database.get("pk", "get").await.unwrap(),
            Some(Bytes::from(vec![0, 128, 255]))
        );
        database.put("pk", "put", b"put").await.unwrap();
        database.delete("pk", "delete").await.unwrap();
        assert_eq!(
            database.query("pk", Some("after"), 3).await.unwrap(),
            vec![("query-sk".to_string(), Bytes::from_static(b"query"))]
        );
        assert_eq!(
            database
                .scan(Some(("after-pk", "after-sk")), 2)
                .await
                .unwrap(),
            vec![(
                "scan-pk".to_string(),
                "scan-sk".to_string(),
                Bytes::from_static(b"scan")
            )]
        );
        let observed = database.get_observed("pk", "present").await.unwrap();
        let ObservedDocument::Present { revision, .. } = &observed else {
            panic!("expected present observation");
        };
        assert_eq!(*revision, DocDbRevision::new(7));
        let conflict = database
            .transact(&TransactRequest {
                conditions: vec![TransactCondition::NotExists {
                    pk: "pk".to_string(),
                    sk: "missing".to_string(),
                }],
                mutations: vec![],
            })
            .await
            .unwrap();
        assert_eq!(conflict.conflict.unwrap().condition_index, 4);

        let operations = server.await.unwrap();
        assert!(matches!(operations[0], DocDbOperation::Get { .. }));
        assert!(matches!(operations[1], DocDbOperation::Put { .. }));
        assert!(matches!(operations[2], DocDbOperation::Delete { .. }));
        assert!(matches!(
            operations[3],
            DocDbOperation::Query { limit: 3, .. }
        ));
        assert!(matches!(
            operations[4],
            DocDbOperation::Scan { limit: 2, .. }
        ));
        assert!(matches!(operations[5], DocDbOperation::GetObserved { .. }));
        assert!(matches!(operations[6], DocDbOperation::Transact { .. }));
    }

    #[tokio::test]
    async fn rejects_invalid_remote_transaction_conflict_index() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let (url, server) = start_remote_server(vec![DocDbResponse::new(DocDbResult::Transact {
            outcome: DocDbTransactOutcome::Conflict { condition_index: 3 },
        })])
        .await;
        let database = semantic_with_config(url);

        let result = database
            .trx(|trx| async move {
                let handle = trx.create(TransactionTestDoc {
                    id: "invalid-conflict-index".to_string(),
                })?;
                drop(handle);
                trx.commit::<(), ()>(())
            })
            .await;

        let is_invalid_conflict_error = match result {
            TrxResult::Err(error) => error
                .to_string()
                .contains("backend returned invalid transaction conflict condition_index"),
            _ => false,
        };
        assert!(is_invalid_conflict_error);
        let operations = server.await.unwrap();
        assert!(matches!(
            operations.as_slice(),
            [DocDbOperation::Transact { conditions, mutations }]
                if conditions.len() == 1 && mutations.len() == 1
        ));
    }

    #[tokio::test]
    async fn database_uses_the_semantic_rpc_endpoint() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let (url, server) = start_remote_server(vec![DocDbResponse::new(DocDbResult::Get {
            data: Some(doc_db_protocol::BinaryDocument {
                data: b"from-semantic-rpc".to_vec(),
            }),
        })])
        .await;
        let _environment_guard = DATABASE_ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("FN0_DOC_DB_URL");
        // Environment mutation is process-global; this test serializes it with
        // the other constructor test state in this module.
        unsafe { std::env::set_var("FN0_DOC_DB_URL", &url) };

        let result = database().get("pk", "sk").await;

        match previous {
            Some(value) => unsafe { std::env::set_var("FN0_DOC_DB_URL", value) },
            None => unsafe { std::env::remove_var("FN0_DOC_DB_URL") },
        }
        assert_eq!(
            result.unwrap(),
            Some(Bytes::from_static(b"from-semantic-rpc"))
        );
        let operations = server.await.unwrap();
        assert!(matches!(
            operations.as_slice(),
            [DocDbOperation::Get { .. }]
        ));
    }
}

#[derive(Clone)]
enum DatabaseInner {
    Turso(TursoDatabase),
    Memory(MemoryDatabase),
    Remote(RemoteDatabase),
    #[cfg(not(target_arch = "wasm32"))]
    Dodb(dodb::DodbDatabase),
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
}

#[doc(hidden)]
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

#[doc(hidden)]
#[derive(Debug)]
pub enum DbResult {
    Single(Option<Bytes>),
    Multiple(Vec<(String, Bytes)>),
    Done,
}

#[doc(hidden)]
pub type DbResultParser<O> = Box<dyn FnOnce(&mut std::vec::IntoIter<DbResult>) -> Result<O> + Send>;

#[doc(hidden)]
pub struct Prepared<O> {
    pub ops: Vec<DbOp>,
    pub parse: DbResultParser<O>,
}

#[allow(async_fn_in_trait)]
pub trait DbRequest: Sized {
    type Output;
    fn prepare(self) -> Prepared<Self::Output>;

    /// Executes every prepared operation concurrently.
    ///
    /// These operations are independent and non-atomic: execution order is
    /// unspecified, and a failure in one operation does not roll back the
    /// others. All started operations settle before an error is returned; the
    /// first error in input order wins when more than one operation fails.
    async fn send_with(self, db: &Database) -> Result<Self::Output> {
        let prepared = self.prepare();
        let results = futures::future::join_all(
            prepared
                .ops
                .into_iter()
                .map(|operation| db.execute_op(operation)),
        )
        .await;
        let mut settled = Vec::with_capacity(results.len());
        let mut first_error = None;
        for result in results {
            match result {
                Ok(result) => settled.push(result),
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        let mut iter = settled.into_iter();
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
