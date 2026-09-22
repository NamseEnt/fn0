#[cfg(all(test, not(target_arch = "wasm32")))]
mod backend_contract;
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
    DocDbBasicOperation, DocDbBasicResult, DocDbBatchOperation, DocDbCondition, DocDbDocument,
    DocDbError, DocDbKey, DocDbMutation, DocDbObservedDocument, DocDbOperation, DocDbResponse,
    DocDbResult, DocDbTransactOutcome,
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

pub fn semantic_with_config(url: String) -> Database {
    Database {
        inner: DatabaseInner::Remote(RemoteDatabase::new(url)),
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
            DatabaseInner::Remote(db) => db.get(pk, sk).await,
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
        }
    }

    #[tracing::instrument(skip_all, fields(ops = ops.len()))]
    pub async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.batch(ops).await,
            DatabaseInner::Memory(db) => db.batch(ops).await,
            DatabaseInner::Remote(db) => db.batch(ops).await,
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
            DocDbOperation::Batch { operations } => {
                let batch_operations: Vec<BatchOp<'_>> = operations
                    .iter()
                    .map(|operation| match operation {
                        DocDbBatchOperation::Put { key, data } => BatchOp::Put {
                            pk: &key.pk,
                            sk: &key.sk,
                            data,
                        },
                        DocDbBatchOperation::Delete { key } => BatchOp::Delete {
                            pk: &key.pk,
                            sk: &key.sk,
                        },
                    })
                    .collect();
                self.batch(&batch_operations).await?;
                Ok(DocDbResponse::new(DocDbResult::Batch))
            }
            DocDbOperation::BatchGetObserved { keys } => {
                let keys = keys
                    .into_iter()
                    .map(|key| (key.pk, key.sk))
                    .collect::<Vec<_>>();
                let documents = self
                    .batch_get_observed(&keys)
                    .await?
                    .into_iter()
                    .map(|document| match document {
                        ObservedDocument::Present { data, revision } => {
                            DocDbObservedDocument::Present {
                                data: data.to_vec(),
                                revision,
                            }
                        }
                        ObservedDocument::Missing { revision } => {
                            DocDbObservedDocument::Missing { revision }
                        }
                    })
                    .collect();
                Ok(DocDbResponse::new(DocDbResult::BatchGetObserved {
                    documents,
                }))
            }
            DocDbOperation::ExecuteOps { operations } => {
                self.execute_semantic_ops(operations).await
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
        }
    }

    async fn execute_semantic_ops(
        &self,
        operations: Vec<DocDbBasicOperation>,
    ) -> Result<DocDbResponse> {
        let db_operations = match operations
            .iter()
            .map(basic_operation_to_db_op)
            .collect::<Result<Vec<_>>>()
        {
            Ok(operations) => operations,
            Err(error) => {
                return Ok(DocDbResponse::error(DocDbError::InvalidRequest {
                    message: error.to_string(),
                }));
            }
        };
        let db_results = self.execute_ops(db_operations).await?;
        if db_results.len() != operations.len() {
            anyhow::bail!(
                "semantic execute_ops result count mismatch: expected {}, got {}",
                operations.len(),
                db_results.len()
            );
        }
        let results = operations
            .into_iter()
            .zip(db_results)
            .map(|(operation, result)| basic_result_from_db_result(operation, result))
            .collect::<Result<Vec<_>>>()?;
        Ok(DocDbResponse::new(DocDbResult::ExecuteOps { results }))
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
            DatabaseInner::Remote(_) => {
                anyhow::bail!("raw SQL is not supported by semantic doc-db RPC")
            }
        }
    }

    #[tracing::instrument(skip_all, fields(ops = ops.len()))]
    pub async fn execute_ops(&self, ops: Vec<DbOp>) -> Result<Vec<DbResult>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_ops(ops).await,
            DatabaseInner::Memory(db) => db.execute_ops(ops).await,
            DatabaseInner::Remote(db) => db.execute_ops(ops).await,
        }
    }

    #[tracing::instrument(skip_all, fields(pk = %pk, sk = %sk))]
    pub(crate) async fn get_observed(&self, pk: &str, sk: &str) -> Result<ObservedDocument> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.get_observed(pk, sk).await,
            DatabaseInner::Memory(db) => db.get_observed(pk, sk).await,
            DatabaseInner::Remote(db) => db.get_observed(pk, sk).await,
        }
    }

    #[tracing::instrument(skip_all, fields(reads = keys.len()))]
    pub(crate) async fn batch_get_observed(
        &self,
        keys: &[(String, String)],
    ) -> Result<Vec<ObservedDocument>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        if let DatabaseInner::Remote(db) = &self.inner {
            return db.batch_get_observed(keys).await;
        }
        let mut out = Vec::with_capacity(keys.len());
        for (pk, sk) in keys {
            out.push(self.get_observed(pk, sk).await?);
        }
        Ok(out)
    }
}

fn basic_operation_to_db_op(operation: &DocDbBasicOperation) -> Result<DbOp> {
    Ok(match operation {
        DocDbBasicOperation::Get { key } => DbOp::Get {
            pk: key.pk.clone(),
            sk: key.sk.clone(),
        },
        DocDbBasicOperation::Query {
            pk,
            after_sk,
            limit,
        } => DbOp::Query {
            pk: pk.clone(),
            after_sk: after_sk.clone(),
            limit: limit
                .map(|value| semantic_limit(value).map_err(anyhow::Error::msg))
                .transpose()?,
        },
        DocDbBasicOperation::Put { key, data } => DbOp::Put {
            pk: key.pk.clone(),
            sk: key.sk.clone(),
            data: data.clone(),
        },
        DocDbBasicOperation::Delete { key } => DbOp::Delete {
            pk: key.pk.clone(),
            sk: key.sk.clone(),
        },
    })
}

fn basic_result_from_db_result(
    operation: DocDbBasicOperation,
    result: DbResult,
) -> Result<DocDbBasicResult> {
    match (operation, result) {
        (DocDbBasicOperation::Get { .. }, DbResult::Single(data)) => Ok(DocDbBasicResult::Get {
            data: data.map(|value| doc_db_protocol::BinaryDocument {
                data: value.to_vec(),
            }),
        }),
        (DocDbBasicOperation::Query { pk, .. }, DbResult::Multiple(documents)) => {
            Ok(DocDbBasicResult::Query {
                documents: documents
                    .into_iter()
                    .map(|(sk, data)| DocDbDocument {
                        key: DocDbKey::new(pk.clone(), sk),
                        data: data.to_vec(),
                    })
                    .collect(),
            })
        }
        (DocDbBasicOperation::Put { .. }, DbResult::Done) => Ok(DocDbBasicResult::Put),
        (DocDbBasicOperation::Delete { .. }, DbResult::Done) => Ok(DocDbBasicResult::Delete),
        (operation, result) => anyhow::bail!(
            "semantic execute_ops result did not match operation: {operation:?} / {result:?}"
        ),
    }
}

fn semantic_limit(limit: u64) -> std::result::Result<usize, String> {
    usize::try_from(limit).map_err(|_| "limit does not fit the host usize".to_string())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod semantic_tests {
    use super::*;
    use doc_db_protocol::{
        DocDbBasicOperation, DocDbBatchOperation, DocDbCondition, DocDbKey, DocDbMutation,
        DocDbObservedDocument, DocDbOperation, DocDbRequest, DocDbResult, DocDbRevision,
        DocDbTransactOutcome,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

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

        let batch = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::Batch {
                operations: vec![DocDbBatchOperation::Put {
                    key: DocDbKey::new("pk", "batch"),
                    data: b"batch".to_vec(),
                }],
            }))
            .await
            .unwrap();
        assert_eq!(batch.result, DocDbResult::Batch);

        let observed = database
            .execute_semantic(DocDbRequest::new(DocDbOperation::BatchGetObserved {
                keys: vec![DocDbKey::new("pk", "a"), DocDbKey::new("pk", "missing")],
            }))
            .await
            .unwrap();
        let DocDbResult::BatchGetObserved { documents } = &observed.result else {
            panic!("expected observed result");
        };
        assert_eq!(documents.len(), 2);
        assert!(matches!(
            documents[0],
            DocDbObservedDocument::Present { .. }
        ));
        assert!(matches!(
            documents[1],
            DocDbObservedDocument::Missing { revision: None }
        ));
        let DocDbObservedDocument::Present { revision, .. } = &documents[0] else {
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
    async fn converts_remote_responses_to_database_results() {
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
            DocDbResponse::new(DocDbResult::ExecuteOps {
                results: vec![
                    doc_db_protocol::DocDbBasicResult::Get {
                        data: Some(doc_db_protocol::BinaryDocument {
                            data: b"batched-get".to_vec(),
                        }),
                    },
                    doc_db_protocol::DocDbBasicResult::Query {
                        documents: vec![doc_db_protocol::DocDbDocument {
                            key: DocDbKey::new("batched-pk", "batched-sk"),
                            data: b"batched-query".to_vec(),
                        }],
                    },
                    doc_db_protocol::DocDbBasicResult::Put,
                    doc_db_protocol::DocDbBasicResult::Delete,
                ],
            }),
            DocDbResponse::new(DocDbResult::BatchGetObserved {
                documents: vec![
                    DocDbObservedDocument::Present {
                        data: b"observed".to_vec(),
                        revision: DocDbRevision::new(7),
                    },
                    DocDbObservedDocument::Missing { revision: None },
                ],
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
        let batched = database
            .execute_ops(vec![
                DbOp::Get {
                    pk: "pk".to_string(),
                    sk: "batched-get".to_string(),
                },
                DbOp::Query {
                    pk: "batched-pk".to_string(),
                    after_sk: Some("before".to_string()),
                    limit: Some(4),
                },
                DbOp::Put {
                    pk: "pk".to_string(),
                    sk: "batched-put".to_string(),
                    data: b"put".to_vec(),
                },
                DbOp::Delete {
                    pk: "pk".to_string(),
                    sk: "batched-delete".to_string(),
                },
            ])
            .await
            .unwrap();
        assert_eq!(batched.len(), 4);
        assert!(matches!(
            batched[0],
            DbResult::Single(Some(ref data)) if data.as_ref() == b"batched-get"
        ));
        assert!(matches!(
            batched[1],
            DbResult::Multiple(ref documents)
                if documents == &[("batched-sk".to_string(), Bytes::from_static(b"batched-query"))]
        ));
        assert!(matches!(batched[2], DbResult::Done));
        assert!(matches!(batched[3], DbResult::Done));
        let observed = database
            .batch_get_observed(&[
                ("pk".to_string(), "present".to_string()),
                ("pk".to_string(), "missing".to_string()),
            ])
            .await
            .unwrap();
        assert!(matches!(
            observed.as_slice(),
            [
                ObservedDocument::Present { .. },
                ObservedDocument::Missing { revision: None },
            ]
        ));
        let ObservedDocument::Present { revision, .. } = &observed[0] else {
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
        assert!(matches!(
            operations[5],
            DocDbOperation::ExecuteOps { ref operations }
                if operations.len() == 4
                    && matches!(operations[0], DocDbBasicOperation::Get { .. })
                    && matches!(
                        operations[1],
                        DocDbBasicOperation::Query { limit: Some(4), .. }
                    )
                    && matches!(operations[2], DocDbBasicOperation::Put { .. })
                    && matches!(operations[3], DocDbBasicOperation::Delete { .. })
        ));
        assert!(matches!(
            operations[6],
            DocDbOperation::BatchGetObserved { .. }
        ));
        assert!(matches!(operations[7], DocDbOperation::Transact { .. }));
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
}

#[derive(Clone)]
enum DatabaseInner {
    Turso(TursoDatabase),
    Memory(MemoryDatabase),
    Remote(RemoteDatabase),
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

#[derive(Debug)]
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
