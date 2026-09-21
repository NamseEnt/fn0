use crate::{
    AdminScanPage, AdminScanRequest, BatchOp, ConditionalOp, ConditionalOutcome, DbOp, DbResult,
    StoredDoc,
};
use anyhow::{Result, bail};
use bytes::Bytes;
use dibi_protocol::{
    AdminItem, ExecuteOperation, ExecuteResult, Key, MAX_BATCH_OPS, MAX_FRAME_SIZE,
    MAX_QUERY_LIMIT, Opcode, RequestOperation, ResponsePayload, Status, TransactWriteOperation,
    VersionedItem, WriteOperation, decode_response_frame, decode_response_payload,
    encode_request_frame_checked,
};
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

pub(crate) trait DibiTransport: Send + Sync {
    fn request<'a>(
        &'a self,
        endpoint: &'a str,
        frame: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;
}

struct RuntimeDibiTransport;

impl DibiTransport for RuntimeDibiTransport {
    fn request<'a>(
        &'a self,
        endpoint: &'a str,
        frame: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
        Box::pin(crate::runtime::dibi_request(endpoint, frame))
    }
}

#[derive(Clone)]
pub(crate) struct DibiDatabase {
    endpoint: String,
    next_request_id: Arc<AtomicU64>,
    transport: Arc<dyn DibiTransport>,
}

impl DibiDatabase {
    pub(crate) fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            next_request_id: Arc::new(AtomicU64::new(1)),
            transport: Arc::new(RuntimeDibiTransport),
        }
    }

    #[cfg(test)]
    fn with_transport(endpoint: String, transport: Arc<dyn DibiTransport>) -> Self {
        Self {
            endpoint,
            next_request_id: Arc::new(AtomicU64::new(1)),
            transport,
        }
    }

    async fn request(&self, operation: RequestOperation) -> Result<ResponsePayload> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let opcode = operation.opcode();
        let frame = encode_request_frame_checked(request_id, "", &operation)
            .map_err(|error| anyhow::anyhow!("Dibi request exceeds protocol limits: {error}"))?;
        if frame.len() > MAX_FRAME_SIZE {
            bail!("Dibi request exceeds the maximum frame size")
        }

        let response = self.transport.request(&self.endpoint, &frame).await?;
        let response_frame = decode_response_frame(&response)
            .map_err(|error| anyhow::anyhow!("Dibi protocol error: {error}"))?;
        if response_frame.request_id != request_id {
            bail!(
                "Dibi protocol error: response request ID mismatch (expected {request_id}, got {})",
                response_frame.request_id
            )
        }
        let status = response_frame.status;
        if status == Status::Conflict
            && !matches!(
                opcode,
                Opcode::TransactWriteItems | Opcode::AdminTransactWriteItems
            )
        {
            bail!("Dibi protocol error: CONFLICT is invalid for {opcode:?}")
        }
        let payload = decode_response_payload(opcode, status, &response_frame.payload)
            .map_err(|error| anyhow::anyhow!("Dibi protocol error: {error}"))?;
        match status {
            Status::Ok | Status::Conflict => Ok(payload),
            Status::InvalidRequest
            | Status::NotFound
            | Status::Unauthorized
            | Status::InternalError => match payload {
                ResponsePayload::ErrorMessage(message) => {
                    bail!("Dibi {status:?}: {message}")
                }
                _ => bail!("Dibi protocol error: missing error message for {status:?}"),
            },
        }
    }

    pub(crate) async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match self
            .request(RequestOperation::Get {
                pk: pk.to_owned(),
                sk: sk.to_owned(),
            })
            .await?
        {
            ResponsePayload::Found { found, data } => {
                if found {
                    data.map(Bytes::from)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Dibi protocol error: found GET without data")
                        })
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            payload => bail!("Dibi protocol error: unexpected GET response {payload:?}"),
        }
    }

    pub(crate) async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        match self
            .request(RequestOperation::Put {
                pk: pk.to_owned(),
                sk: sk.to_owned(),
                data: data.to_vec(),
            })
            .await?
        {
            ResponsePayload::CommitId(_) => Ok(()),
            payload => bail!("Dibi protocol error: unexpected PUT response {payload:?}"),
        }
    }

    pub(crate) async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        match self
            .request(RequestOperation::Delete {
                pk: pk.to_owned(),
                sk: sk.to_owned(),
            })
            .await?
        {
            ResponsePayload::CommitId(_) => Ok(()),
            payload => bail!("Dibi protocol error: unexpected DELETE response {payload:?}"),
        }
    }

    pub(crate) async fn query<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        pk: S1,
        after_sk: Option<S2>,
        limit: usize,
    ) -> Result<Vec<(String, Bytes)>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let pk = pk.as_ref().to_owned();
        let mut cursor = after_sk.map(|value| value.as_ref().to_owned());
        let mut output = Vec::with_capacity(limit.min(MAX_QUERY_LIMIT as usize));
        while output.len() < limit {
            let page_limit = (limit - output.len()).min(MAX_QUERY_LIMIT as usize);
            let page = match self
                .request(RequestOperation::Query {
                    pk: pk.clone(),
                    after_sk: cursor.clone(),
                    limit: page_limit as u32,
                })
                .await?
            {
                ResponsePayload::QueryItems(items) => items,
                payload => bail!("Dibi protocol error: unexpected QUERY response {payload:?}"),
            };
            if page.is_empty() {
                break;
            }
            let last_sk = page.last().map(|item| item.sk.clone());
            output.extend(
                page.into_iter()
                    .map(|item| (item.sk, Bytes::from(item.data))),
            );
            if output.len() >= limit {
                output.truncate(limit);
                break;
            }
            let Some(last_sk) = last_sk else {
                break;
            };
            if cursor.as_deref() == Some(last_sk.as_str()) {
                bail!("Dibi protocol error: QUERY cursor did not advance")
            }
            cursor = Some(last_sk);
            if output.len() < limit && output.is_empty() {
                break;
            }
        }
        Ok(output)
    }

    pub(crate) async fn scan(
        &self,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<Vec<(String, String, Bytes)>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut cursor = after.map(|(pk, sk)| Key {
            pk: pk.to_owned(),
            sk: sk.to_owned(),
        });
        let mut output = Vec::with_capacity(limit.min(MAX_QUERY_LIMIT as usize));
        while output.len() < limit {
            let page_limit = (limit - output.len()).min(MAX_QUERY_LIMIT as usize);
            let page = match self
                .request(RequestOperation::Scan {
                    cursor: cursor.clone(),
                    limit: page_limit as u32,
                })
                .await?
            {
                ResponsePayload::ScanItems(items) => items,
                payload => bail!("Dibi protocol error: unexpected SCAN response {payload:?}"),
            };
            if page.is_empty() {
                break;
            }
            let last_key = page.last().map(|item| (item.pk.clone(), item.sk.clone()));
            output.extend(
                page.into_iter()
                    .map(|item| (item.pk, item.sk, Bytes::from(item.data))),
            );
            if output.len() >= limit {
                output.truncate(limit);
                break;
            }
            let Some((last_pk, last_sk)) = last_key else {
                break;
            };
            if cursor
                .as_ref()
                .is_some_and(|key| key.pk == last_pk && key.sk == last_sk)
            {
                bail!("Dibi protocol error: SCAN cursor did not advance")
            }
            cursor = Some(Key {
                pk: last_pk,
                sk: last_sk,
            });
        }
        Ok(output)
    }

    pub(crate) async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        ensure_atomic_operation_count(ops.len(), "BATCH")?;
        let operations = ops
            .iter()
            .map(|operation| match operation {
                BatchOp::Put { pk, sk, data } => WriteOperation::Put {
                    pk: (*pk).to_owned(),
                    sk: (*sk).to_owned(),
                    data: (*data).to_vec(),
                },
                BatchOp::Delete { pk, sk } => WriteOperation::Delete {
                    pk: (*pk).to_owned(),
                    sk: (*sk).to_owned(),
                },
            })
            .collect();
        match self.request(RequestOperation::Batch(operations)).await? {
            ResponsePayload::OptionalCommitId(_) => Ok(()),
            payload => bail!("Dibi protocol error: unexpected BATCH response {payload:?}"),
        }
    }

    pub(crate) async fn execute_ops(&self, ops: Vec<DbOp>) -> Result<Vec<DbResult>> {
        let can_use_execute_ops = ops.len() <= MAX_BATCH_OPS
            && ops.iter().all(|operation| match operation {
                DbOp::Query { limit: None, .. } => false,
                DbOp::Query {
                    limit: Some(limit), ..
                } => *limit <= MAX_QUERY_LIMIT as usize,
                _ => true,
            });
        if !can_use_execute_ops {
            let mut results = Vec::with_capacity(ops.len());
            for operation in ops {
                results.push(match operation {
                    DbOp::Get { pk, sk } => DbResult::Single(self.get(&pk, &sk).await?),
                    DbOp::Query {
                        pk,
                        after_sk,
                        limit,
                    } => DbResult::Multiple(
                        self.query(&pk, after_sk.as_deref(), limit.unwrap_or(usize::MAX))
                            .await?,
                    ),
                    DbOp::Put { pk, sk, data } => {
                        self.put(&pk, &sk, &data).await?;
                        DbResult::Done
                    }
                    DbOp::Delete { pk, sk } => {
                        self.delete(&pk, &sk).await?;
                        DbResult::Done
                    }
                });
            }
            return Ok(results);
        }

        let protocol_operations = ops
            .iter()
            .map(|operation| match operation {
                DbOp::Get { pk, sk } => ExecuteOperation::Get {
                    pk: pk.clone(),
                    sk: sk.clone(),
                },
                DbOp::Query {
                    pk,
                    after_sk,
                    limit,
                } => ExecuteOperation::Query {
                    pk: pk.clone(),
                    after_sk: after_sk.clone(),
                    limit: limit.unwrap_or(0) as u32,
                },
                DbOp::Put { pk, sk, data } => ExecuteOperation::Put {
                    pk: pk.clone(),
                    sk: sk.clone(),
                    data: data.clone(),
                },
                DbOp::Delete { pk, sk } => ExecuteOperation::Delete {
                    pk: pk.clone(),
                    sk: sk.clone(),
                },
            })
            .collect();
        let payload = self
            .request(RequestOperation::ExecuteOps(protocol_operations))
            .await?;
        let ResponsePayload::ExecuteResults(results) = payload else {
            bail!("Dibi protocol error: unexpected EXECUTE_OPS response {payload:?}")
        };
        if results.len() != ops.len() {
            bail!(
                "Dibi protocol error: EXECUTE_OPS returned {} results for {} operations",
                results.len(),
                ops.len()
            )
        }
        results
            .into_iter()
            .zip(ops)
            .map(|(result, operation)| match (result, operation) {
                (ExecuteResult::Done, DbOp::Put { .. } | DbOp::Delete { .. }) => Ok(DbResult::Done),
                (
                    ExecuteResult::Single { found, data },
                    DbOp::Get { .. },
                ) => Ok(DbResult::Single(if found {
                    Some(Bytes::from(data.ok_or_else(|| {
                        anyhow::anyhow!("Dibi protocol error: GET result is missing data")
                    })?))
                } else {
                    None
                })),
                (ExecuteResult::Multiple(items), DbOp::Query { .. }) => Ok(DbResult::Multiple(
                    items
                        .into_iter()
                        .map(|item| (item.sk, Bytes::from(item.data)))
                        .collect(),
                )),
                (unexpected, operation) => bail!(
                    "Dibi protocol error: unexpected EXECUTE_OPS result {unexpected:?} for {operation:?}"
                ),
            })
            .collect()
    }

    pub(crate) async fn get_with_version(&self, pk: &str, sk: &str) -> Result<Option<StoredDoc>> {
        match self
            .request(RequestOperation::GetWithVersion {
                pk: pk.to_owned(),
                sk: sk.to_owned(),
            })
            .await?
        {
            ResponsePayload::Versioned {
                found,
                version,
                data,
            } => versioned_document(found, version, data),
            payload => {
                bail!("Dibi protocol error: unexpected GET_WITH_VERSION response {payload:?}")
            }
        }
    }

    pub(crate) async fn batch_get_with_version(
        &self,
        keys: &[(String, String)],
    ) -> Result<Vec<Option<StoredDoc>>> {
        let mut output = Vec::with_capacity(keys.len());
        for key_chunk in keys.chunks(MAX_BATCH_OPS) {
            let payload = self
                .request(RequestOperation::BatchGetWithVersion(
                    key_chunk
                        .iter()
                        .map(|(pk, sk)| Key {
                            pk: pk.clone(),
                            sk: sk.clone(),
                        })
                        .collect(),
                ))
                .await?;
            let ResponsePayload::VersionedItems(items) = payload else {
                bail!("Dibi protocol error: unexpected BATCH_GET_WITH_VERSION response {payload:?}")
            };
            if items.len() != key_chunk.len() {
                bail!(
                    "Dibi protocol error: BATCH_GET_WITH_VERSION returned {} results for {} keys",
                    items.len(),
                    key_chunk.len()
                )
            }
            for item in items {
                output.push(versioned_item(item)?);
            }
        }
        Ok(output)
    }

    pub(crate) async fn admin_scan(&self, request: AdminScanRequest) -> Result<AdminScanPage> {
        if request.limit == 0 {
            return Ok(AdminScanPage {
                documents: Vec::new(),
                next: request.after,
            });
        }
        let mut cursor = request.after.map(|(pk, sk)| Key { pk, sk });
        let mut documents = Vec::with_capacity(request.limit);
        while documents.len() < request.limit {
            let page_limit = (request.limit - documents.len()).min(MAX_QUERY_LIMIT as usize);
            let payload = self
                .request(RequestOperation::AdminScan {
                    cursor: cursor.clone(),
                    limit: page_limit as u32,
                    pk_prefix: request.pk_prefix.clone(),
                })
                .await?;
            let ResponsePayload::AdminScan { items, next_cursor } = payload else {
                bail!("Dibi protocol error: unexpected ADMIN_SCAN response {payload:?}")
            };
            let page_count = items.len();
            documents.extend(items.into_iter().map(admin_document));
            if documents.len() >= request.limit {
                documents.truncate(request.limit);
                return Ok(AdminScanPage {
                    documents,
                    next: next_cursor.map(|key| (key.pk, key.sk)),
                });
            }
            let Some(next_cursor) = next_cursor else {
                return Ok(AdminScanPage {
                    documents,
                    next: None,
                });
            };
            if page_count == 0
                && cursor
                    .as_ref()
                    .is_some_and(|key| key.pk == next_cursor.pk && key.sk == next_cursor.sk)
            {
                bail!("Dibi protocol error: ADMIN_SCAN cursor did not advance")
            }
            cursor = Some(next_cursor);
        }
        Ok(AdminScanPage {
            documents,
            next: cursor.map(|key| (key.pk, key.sk)),
        })
    }

    pub(crate) async fn conditional_write_batch(
        &self,
        operations: &[ConditionalOp],
    ) -> Result<ConditionalOutcome> {
        self.conditional_write_batch_with_opcode(operations, false)
            .await
    }

    pub(crate) async fn admin_conditional_write_batch(
        &self,
        operations: &[ConditionalOp],
    ) -> Result<ConditionalOutcome> {
        self.conditional_write_batch_with_opcode(operations, true)
            .await
    }

    async fn conditional_write_batch_with_opcode(
        &self,
        operations: &[ConditionalOp],
        admin: bool,
    ) -> Result<ConditionalOutcome> {
        ensure_atomic_operation_count(operations.len(), "conditional write")?;
        validate_conditional_keys(operations)?;
        let protocol_operations = operations
            .iter()
            .map(protocol_conditional_operation)
            .collect();
        let operation = if admin {
            RequestOperation::AdminTransactWriteItems(protocol_operations)
        } else {
            RequestOperation::TransactWriteItems(protocol_operations)
        };
        match self.request(operation).await? {
            ResponsePayload::OptionalCommitId(_) => Ok(ConditionalOutcome::Applied),
            ResponsePayload::ConditionalConflicts(conflicts) => {
                Ok(ConditionalOutcome::Conflict(crate::ConflictDetails {
                    keys: conflicts
                        .into_iter()
                        .map(|conflict| crate::ConflictKey {
                            key: crate::DocKey::new(conflict.pk, conflict.sk),
                            expected_version: conflict.expected_version,
                            actual_version: conflict.actual_version,
                        })
                        .collect(),
                }))
            }
            payload => bail!("Dibi protocol error: unexpected conditional response {payload:?}"),
        }
    }
}

pub(crate) struct DibiTransaction {
    db: DibiDatabase,
    pending: Vec<PendingDibiWrite>,
    overlay: BTreeMap<(String, String), Option<Bytes>>,
}

enum PendingDibiWrite {
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

impl DibiTransaction {
    pub(crate) fn new(db: DibiDatabase) -> Self {
        Self {
            db,
            pending: Vec::new(),
            overlay: BTreeMap::new(),
        }
    }

    pub(crate) async fn get(&mut self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        if let Some(value) = self.overlay.get(&(pk.to_owned(), sk.to_owned())) {
            return Ok(value.clone());
        }
        self.db.get(pk, sk).await
    }

    pub(crate) async fn put(&mut self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        self.pending.push(PendingDibiWrite::Put {
            pk: pk.to_owned(),
            sk: sk.to_owned(),
            data: data.to_vec(),
        });
        self.overlay.insert(
            (pk.to_owned(), sk.to_owned()),
            Some(Bytes::copy_from_slice(data)),
        );
        Ok(())
    }

    pub(crate) async fn delete(&mut self, pk: &str, sk: &str) -> Result<()> {
        self.pending.push(PendingDibiWrite::Delete {
            pk: pk.to_owned(),
            sk: sk.to_owned(),
        });
        self.overlay.insert((pk.to_owned(), sk.to_owned()), None);
        Ok(())
    }

    pub(crate) async fn commit(self) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let operations: Vec<BatchOp<'_>> = self
            .pending
            .iter()
            .map(|operation| match operation {
                PendingDibiWrite::Put { pk, sk, data } => BatchOp::Put { pk, sk, data },
                PendingDibiWrite::Delete { pk, sk } => BatchOp::Delete { pk, sk },
            })
            .collect();
        self.db.batch(&operations).await
    }

    pub(crate) async fn rollback(self) -> Result<()> {
        Ok(())
    }
}

fn ensure_atomic_operation_count(count: usize, operation: &str) -> Result<()> {
    if count > MAX_BATCH_OPS {
        bail!(
            "Dibi {operation} exceeds the atomic operation limit of {MAX_BATCH_OPS}; it cannot be split"
        )
    }
    Ok(())
}

fn validate_conditional_keys(operations: &[ConditionalOp]) -> Result<()> {
    let mut keys = std::collections::HashSet::with_capacity(operations.len());
    for operation in operations {
        let (pk, sk) = conditional_key(operation);
        if !keys.insert((pk, sk)) {
            bail!("duplicate conditional write key: {pk}/{sk}")
        }
    }
    Ok(())
}

fn conditional_key(operation: &ConditionalOp) -> (&str, &str) {
    match operation {
        ConditionalOp::Create { pk, sk, .. }
        | ConditionalOp::Put { pk, sk, .. }
        | ConditionalOp::Delete { pk, sk, .. }
        | ConditionalOp::Check { pk, sk, .. } => (pk, sk),
    }
}

fn protocol_conditional_operation(operation: &ConditionalOp) -> TransactWriteOperation {
    match operation {
        ConditionalOp::Create { pk, sk, data } => TransactWriteOperation::Create {
            pk: pk.clone(),
            sk: sk.clone(),
            data: data.clone(),
        },
        ConditionalOp::Put {
            pk,
            sk,
            expected_version,
            data,
        } => TransactWriteOperation::Put {
            pk: pk.clone(),
            sk: sk.clone(),
            expected_version: *expected_version,
            data: data.clone(),
        },
        ConditionalOp::Delete {
            pk,
            sk,
            expected_version,
        } => TransactWriteOperation::Delete {
            pk: pk.clone(),
            sk: sk.clone(),
            expected_version: *expected_version,
        },
        ConditionalOp::Check {
            pk,
            sk,
            expected_version,
        } => TransactWriteOperation::ConditionCheck {
            pk: pk.clone(),
            sk: sk.clone(),
            expected_version: *expected_version,
        },
    }
}

fn versioned_document(
    found: bool,
    version: Option<i64>,
    data: Option<Vec<u8>>,
) -> Result<Option<StoredDoc>> {
    if !found {
        return Ok(None);
    }
    let version =
        version.ok_or_else(|| anyhow::anyhow!("Dibi protocol error: missing document version"))?;
    let data = data.ok_or_else(|| anyhow::anyhow!("Dibi protocol error: missing document data"))?;
    Ok(Some(StoredDoc {
        data: Bytes::from(data),
        version,
    }))
}

fn versioned_item(item: VersionedItem) -> Result<Option<StoredDoc>> {
    versioned_document(item.found, item.version, item.data)
}

fn admin_document(item: AdminItem) -> crate::AdminDocument {
    crate::AdminDocument {
        pk: item.pk,
        sk: item.sk,
        data: Bytes::from(item.data),
        version: item.version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdminScanRequest, BatchOp, Database, DatabaseInner, DbOp, DbResult, backend_contract,
        memory::MemoryDatabase, mock,
    };
    use dibi_protocol::{
        AdminItem, Conflict, ExecuteOperation, Key, QueryItem, RequestOperation, ResponsePayload,
        ScanItem, Status, TransactWriteOperation, VersionedItem, WriteOperation,
        decode_request_frame, encode_response_frame,
    };
    use std::sync::Mutex;

    #[derive(Clone)]
    struct FakeTransport {
        database: MemoryDatabase,
        requests: Arc<Mutex<Vec<RequestOperation>>>,
    }

    impl FakeTransport {
        fn new() -> Self {
            Self {
                database: MemoryDatabase::new(),
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn database(&self) -> Database {
            Database {
                inner: DatabaseInner::Dibi(DibiDatabase::with_transport(
                    "dibi://test".to_owned(),
                    Arc::new(self.clone()),
                )),
                mock_state: mock::MockState::default(),
            }
        }

        fn requests(&self) -> Vec<RequestOperation> {
            self.requests.lock().unwrap().clone()
        }
    }

    impl DibiTransport for FakeTransport {
        fn request<'a>(
            &'a self,
            _endpoint: &'a str,
            frame: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
            let database = self.database.clone();
            let requests = self.requests.clone();
            let owned_frame = frame.to_vec();
            Box::pin(async move {
                let request = decode_request_frame(&owned_frame)
                    .map_err(|error| anyhow::anyhow!("fake request decode: {error}"))?;
                assert!(request.tenant.is_empty());
                requests.lock().unwrap().push(request.operation.clone());
                let mut response_status = Status::Ok;
                let payload = match request.operation {
                    RequestOperation::Get { pk, sk } => ResponsePayload::Found {
                        found: database.get(&pk, &sk).await?.is_some(),
                        data: database.get(&pk, &sk).await?.map(|value| value.to_vec()),
                    },
                    RequestOperation::Put { pk, sk, data } => {
                        database.put(&pk, &sk, &data).await?;
                        ResponsePayload::CommitId(1)
                    }
                    RequestOperation::Delete { pk, sk } => {
                        database.delete(&pk, &sk).await?;
                        ResponsePayload::CommitId(1)
                    }
                    RequestOperation::Query {
                        pk,
                        after_sk,
                        limit,
                    } => ResponsePayload::QueryItems(
                        database
                            .query(&pk, after_sk.as_deref(), limit as usize)
                            .await?
                            .into_iter()
                            .map(|(sk, data)| QueryItem {
                                sk,
                                data: data.to_vec(),
                            })
                            .collect(),
                    ),
                    RequestOperation::Scan { cursor, limit } => ResponsePayload::ScanItems(
                        database
                            .scan(
                                cursor
                                    .as_ref()
                                    .map(|key| (key.pk.as_str(), key.sk.as_str())),
                                limit as usize,
                            )
                            .await?
                            .into_iter()
                            .map(|(pk, sk, data)| ScanItem {
                                pk,
                                sk,
                                data: data.to_vec(),
                            })
                            .collect(),
                    ),
                    RequestOperation::Batch(operations) => {
                        let batch_operations = operations
                            .iter()
                            .map(|operation| match operation {
                                WriteOperation::Put { pk, sk, data } => {
                                    BatchOp::Put { pk, sk, data }
                                }
                                WriteOperation::Delete { pk, sk } => BatchOp::Delete { pk, sk },
                            })
                            .collect::<Vec<_>>();
                        database.batch(&batch_operations).await?;
                        ResponsePayload::OptionalCommitId(Some(1))
                    }
                    RequestOperation::ExecuteOps(operations) => ResponsePayload::ExecuteResults(
                        fake_execute_ops(&database, operations).await?,
                    ),
                    RequestOperation::GetWithVersion { pk, sk } => {
                        let stored = database.get_with_version(&pk, &sk).await?;
                        ResponsePayload::Versioned {
                            found: stored.is_some(),
                            version: stored.as_ref().map(|value| value.version),
                            data: stored.map(|value| value.data.to_vec()),
                        }
                    }
                    RequestOperation::BatchGetWithVersion(keys) => {
                        let mut items = Vec::with_capacity(keys.len());
                        for key in keys {
                            let stored = database.get_with_version(&key.pk, &key.sk).await?;
                            items.push(VersionedItem {
                                found: stored.is_some(),
                                version: stored.as_ref().map(|value| value.version),
                                data: stored.map(|value| value.data.to_vec()),
                            });
                        }
                        ResponsePayload::VersionedItems(items)
                    }
                    RequestOperation::TransactWriteItems(operations)
                    | RequestOperation::AdminTransactWriteItems(operations) => {
                        let conditional_operations = operations
                            .into_iter()
                            .map(fake_conditional_operation)
                            .collect::<Vec<_>>();
                        match database
                            .conditional_write_batch(&conditional_operations)
                            .await?
                        {
                            ConditionalOutcome::Applied => {
                                ResponsePayload::OptionalCommitId(Some(1))
                            }
                            ConditionalOutcome::Conflict(details) => {
                                response_status = Status::Conflict;
                                ResponsePayload::ConditionalConflicts(
                                    details
                                        .keys
                                        .into_iter()
                                        .map(|key| Conflict {
                                            pk: key.key.pk,
                                            sk: key.key.sk,
                                            expected_version: key.expected_version,
                                            actual_version: key.actual_version,
                                        })
                                        .collect(),
                                )
                            }
                        }
                    }
                    RequestOperation::AdminScan {
                        cursor,
                        limit,
                        pk_prefix,
                    } => {
                        let mut items = Vec::new();
                        let scanned = database
                            .scan(
                                cursor
                                    .as_ref()
                                    .map(|key| (key.pk.as_str(), key.sk.as_str())),
                                limit as usize,
                            )
                            .await?;
                        for (pk, sk, data) in scanned {
                            if pk_prefix
                                .as_deref()
                                .is_some_and(|prefix| !pk.starts_with(prefix))
                            {
                                continue;
                            }
                            let stored =
                                database.get_with_version(&pk, &sk).await?.ok_or_else(|| {
                                    anyhow::anyhow!("fake admin document disappeared")
                                })?;
                            items.push(AdminItem {
                                pk,
                                sk,
                                version: stored.version,
                                data: data.to_vec(),
                            });
                        }
                        let next_cursor = (items.len() == limit as usize)
                            .then(|| {
                                items.last().map(|item| Key {
                                    pk: item.pk.clone(),
                                    sk: item.sk.clone(),
                                })
                            })
                            .flatten();
                        ResponsePayload::AdminScan { items, next_cursor }
                    }
                    RequestOperation::Ping
                    | RequestOperation::Status
                    | RequestOperation::Auth { .. } => ResponsePayload::Empty,
                };
                encode_response_frame(request.request_id, response_status, &payload)
                    .map_err(|error| anyhow::anyhow!("fake response encode: {error}"))
            })
        }
    }

    async fn fake_execute_ops(
        database: &MemoryDatabase,
        operations: Vec<ExecuteOperation>,
    ) -> Result<Vec<ExecuteResult>> {
        let mut results = Vec::with_capacity(operations.len());
        for operation in operations {
            results.push(match operation {
                ExecuteOperation::Get { pk, sk } => {
                    let data = database.get(&pk, &sk).await?;
                    ExecuteResult::Single {
                        found: data.is_some(),
                        data: data.map(|value| value.to_vec()),
                    }
                }
                ExecuteOperation::Query {
                    pk,
                    after_sk,
                    limit,
                } => ExecuteResult::Multiple(
                    database
                        .query(&pk, after_sk.as_deref(), limit as usize)
                        .await?
                        .into_iter()
                        .map(|(sk, data)| QueryItem {
                            sk,
                            data: data.to_vec(),
                        })
                        .collect(),
                ),
                ExecuteOperation::Put { pk, sk, data } => {
                    database.put(&pk, &sk, &data).await?;
                    ExecuteResult::Done
                }
                ExecuteOperation::Delete { pk, sk } => {
                    database.delete(&pk, &sk).await?;
                    ExecuteResult::Done
                }
            });
        }
        Ok(results)
    }

    fn fake_conditional_operation(operation: TransactWriteOperation) -> ConditionalOp {
        match operation {
            TransactWriteOperation::Create { pk, sk, data } => {
                ConditionalOp::Create { pk, sk, data }
            }
            TransactWriteOperation::Put {
                pk,
                sk,
                expected_version,
                data,
            } => ConditionalOp::Put {
                pk,
                sk,
                expected_version,
                data,
            },
            TransactWriteOperation::Delete {
                pk,
                sk,
                expected_version,
            } => ConditionalOp::Delete {
                pk,
                sk,
                expected_version,
            },
            TransactWriteOperation::ConditionCheck {
                pk,
                sk,
                expected_version,
            } => ConditionalOp::Check {
                pk,
                sk,
                expected_version,
            },
        }
    }

    #[tokio::test]
    async fn dibi_backend_contract() {
        backend_contract::run_backend_contract(|| FakeTransport::new().database()).await;
    }

    #[tokio::test]
    async fn dibi_large_reads_page_at_protocol_limit() {
        let transport = FakeTransport::new();
        for key_index in 0..=MAX_QUERY_LIMIT {
            transport
                .database
                .put("large", &format!("{key_index:05}"), &[key_index as u8])
                .await
                .unwrap();
        }
        let database = transport.database();
        let query = database
            .query("large", None::<&str>, MAX_QUERY_LIMIT as usize + 1)
            .await
            .unwrap();
        assert_eq!(query.len(), MAX_QUERY_LIMIT as usize + 1);
        let scan = database
            .scan(None, MAX_QUERY_LIMIT as usize + 1)
            .await
            .unwrap();
        assert_eq!(scan.len(), MAX_QUERY_LIMIT as usize + 1);
        let admin = database
            .admin_scan(AdminScanRequest {
                after: None,
                limit: MAX_QUERY_LIMIT as usize + 1,
                pk_prefix: Some("large".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(admin.documents.len(), MAX_QUERY_LIMIT as usize + 1);
        assert!(
            transport
                .requests()
                .iter()
                .filter(|operation| matches!(operation, RequestOperation::Query { .. }))
                .count()
                >= 2
        );
    }

    #[tokio::test]
    async fn dibi_explicit_transaction_buffers_ordered_writes() {
        let transport = FakeTransport::new();
        let database = transport.database();
        let mut transaction = database.transaction().await.unwrap();
        transaction.put("tx", "a", b"one").await.unwrap();
        assert_eq!(
            transaction.get("tx", "a").await.unwrap(),
            Some(Bytes::from_static(b"one"))
        );
        transaction.put("tx", "b", b"two").await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(
            database.get("tx", "a").await.unwrap(),
            Some(Bytes::from_static(b"one"))
        );
        assert_eq!(
            database.get("tx", "b").await.unwrap(),
            Some(Bytes::from_static(b"two"))
        );

        let mut rollback = database.transaction().await.unwrap();
        rollback.put("tx", "a", b"replacement").await.unwrap();
        rollback.delete("tx", "a").await.unwrap();
        assert_eq!(rollback.get("tx", "a").await.unwrap(), None);
        rollback.rollback().await.unwrap();
        assert_eq!(
            database.get("tx", "a").await.unwrap(),
            Some(Bytes::from_static(b"one"))
        );

        let mut ordered = database.transaction().await.unwrap();
        ordered.put("tx", "ordered", b"first").await.unwrap();
        ordered.put("tx", "ordered", b"second").await.unwrap();
        ordered.commit().await.unwrap();
        assert_eq!(
            database.get("tx", "ordered").await.unwrap(),
            Some(Bytes::from_static(b"second"))
        );
        assert!(
            transport
                .requests()
                .iter()
                .any(|operation| matches!(operation, RequestOperation::Batch(_)))
        );
    }

    #[tokio::test]
    async fn dibi_execute_ops_unlimited_query_preserves_order() {
        let transport = FakeTransport::new();
        for key_index in 0..=MAX_QUERY_LIMIT {
            transport
                .database
                .put("ops-large", &format!("{key_index:05}"), &[key_index as u8])
                .await
                .unwrap();
        }
        let database = transport.database();
        let results = database
            .execute_ops(vec![
                DbOp::Query {
                    pk: "ops-large".to_owned(),
                    after_sk: None,
                    limit: None,
                },
                DbOp::Put {
                    pk: "ops-large".to_owned(),
                    sk: "after".to_owned(),
                    data: b"after".to_vec(),
                },
                DbOp::Get {
                    pk: "ops-large".to_owned(),
                    sk: "after".to_owned(),
                },
            ])
            .await
            .unwrap();
        assert!(
            matches!(&results[0], DbResult::Multiple(items) if items.len() == MAX_QUERY_LIMIT as usize + 1)
        );
        assert!(matches!(results[1], DbResult::Done));
        assert!(
            matches!(&results[2], DbResult::Single(Some(value)) if value == &Bytes::from_static(b"after"))
        );
        assert!(
            !transport
                .requests()
                .iter()
                .any(|operation| matches!(operation, RequestOperation::ExecuteOps(_)))
        );
    }

    #[tokio::test]
    async fn dibi_atomic_requests_are_not_split() {
        let transport = FakeTransport::new();
        let database = transport.database();
        let operations = (0..=MAX_BATCH_OPS)
            .map(|_| BatchOp::Delete {
                pk: "batch",
                sk: "key",
            })
            .collect::<Vec<_>>();
        let error = database.batch(&operations).await.unwrap_err();
        assert!(error.to_string().contains("cannot be split"));
        assert!(transport.requests().is_empty());
    }

    struct StaticTransport {
        response: Vec<u8>,
    }

    impl DibiTransport for StaticTransport {
        fn request<'a>(
            &'a self,
            _endpoint: &'a str,
            _frame: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
            let response = self.response.clone();
            Box::pin(async move { Ok(response) })
        }
    }

    #[tokio::test]
    async fn dibi_malformed_and_error_responses_are_errors() {
        let wrong_id = encode_response_frame(99, Status::Ok, &ResponsePayload::Empty).unwrap();
        let database = Database {
            inner: DatabaseInner::Dibi(DibiDatabase::with_transport(
                "dibi://test".to_owned(),
                Arc::new(StaticTransport { response: wrong_id }),
            )),
            mock_state: mock::MockState::default(),
        };
        assert!(
            database
                .get("pk", "sk")
                .await
                .unwrap_err()
                .to_string()
                .contains("request ID")
        );

        let unauthorized = encode_response_frame(
            1,
            Status::Unauthorized,
            &ResponsePayload::ErrorMessage("denied".to_owned()),
        )
        .unwrap();
        let database = Database {
            inner: DatabaseInner::Dibi(DibiDatabase::with_transport(
                "dibi://test".to_owned(),
                Arc::new(StaticTransport {
                    response: unauthorized,
                }),
            )),
            mock_state: mock::MockState::default(),
        };
        assert!(
            database
                .get("pk", "sk")
                .await
                .unwrap_err()
                .to_string()
                .contains("denied")
        );

        for status in [Status::InvalidRequest, Status::InternalError] {
            let response = encode_response_frame(
                1,
                status,
                &ResponsePayload::ErrorMessage("remote-error".to_owned()),
            )
            .unwrap();
            let database = Database {
                inner: DatabaseInner::Dibi(DibiDatabase::with_transport(
                    "dibi://test".to_owned(),
                    Arc::new(StaticTransport { response }),
                )),
                mock_state: mock::MockState::default(),
            };
            assert!(
                database
                    .get("pk", "sk")
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("remote-error")
            );
        }

        let wrong_payload =
            encode_response_frame(1, Status::Ok, &ResponsePayload::QueryItems(Vec::new())).unwrap();
        let database = Database {
            inner: DatabaseInner::Dibi(DibiDatabase::with_transport(
                "dibi://test".to_owned(),
                Arc::new(StaticTransport {
                    response: wrong_payload,
                }),
            )),
            mock_state: mock::MockState::default(),
        };
        assert!(database.get("pk", "sk").await.is_err());

        let malformed = Database {
            inner: DatabaseInner::Dibi(DibiDatabase::with_transport(
                "dibi://test".to_owned(),
                Arc::new(StaticTransport {
                    response: vec![1, 2, 3],
                }),
            )),
            mock_state: mock::MockState::default(),
        };
        assert!(malformed.get("pk", "sk").await.is_err());
    }
}
