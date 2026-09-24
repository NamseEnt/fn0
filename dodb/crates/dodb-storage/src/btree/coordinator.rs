use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, Error, ObservedState, PrimaryKey, Result, Revision, RevisionState, SortKey,
    TransactionMutation, TransactionRequest, TransactionResult,
};

use super::format::{MAX_OVERFLOW_PAGES, PageData, ValueRef};
use super::{
    BTreeStore, BatchRequest, BatchResponse, CheckpointReport, InvariantReport, StorageMetrics,
    validate_encoded_key,
};
use crate::DurableFile;
use crate::wal::WalMetrics;

/// Internal scheduling limits for one shard coordinator.
///
/// A group is a physical durability unit only. Each mutation request remains
/// an independent logical transaction inside the group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoordinatorConfig {
    pub queue_capacity: usize,
    pub max_group_requests: usize,
    pub max_group_bytes: usize,
    pub max_collection_delay: Duration,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 256,
            max_group_requests: 64,
            max_group_bytes: 4 * 1024 * 1024,
            max_collection_delay: Duration::ZERO,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoordinatorMetrics {
    pub groups: u64,
    pub queued_requests: u64,
    pub logical_transactions: u64,
    pub overloaded_requests: u64,
    pub queue_wait_nanos: u64,
    pub batch_collection_nanos: u64,
    pub processing_nanos: u64,
    pub max_group_requests: usize,
    pub max_group_bytes: usize,
}

/// Async single-shard facade. The coordinator is deliberately thin; all tree
/// mutation work remains in the synchronous engine and is directly testable.
pub struct AsyncShard<
    F: DurableFile + Send + 'static,
    W: DurableFile + Send + 'static = super::NoWal,
> {
    request_tx: tokio::sync::mpsc::Sender<QueuedRequest>,
    close_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    coordinator_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    metrics: Arc<Mutex<Option<WalMetrics>>>,
    storage_metrics: Arc<Mutex<Option<StorageMetrics>>>,
    coordinator_metrics: Arc<Mutex<CoordinatorMetrics>>,
    read_view: Arc<RwLock<ReadViewState>>,
    _marker: std::marker::PhantomData<(F, W)>,
}

impl<F: DurableFile + Send + 'static, W: DurableFile + Send + 'static> AsyncShard<F, W> {
    pub fn start(store: BTreeStore<F, W>, queue_capacity: usize) -> Self {
        Self::start_with_config(
            store,
            CoordinatorConfig {
                queue_capacity,
                ..CoordinatorConfig::default()
            },
        )
    }

    pub fn start_with_config(mut store: BTreeStore<F, W>, config: CoordinatorConfig) -> Self {
        let read_view = Arc::new(RwLock::new(ReadViewState {
            view: CommittedReadView::from_store(&mut store).ok(),
            broken: None,
        }));
        store.enable_published_page_tracking();

        let effective_config = CoordinatorConfig {
            queue_capacity: config.queue_capacity.max(1),
            max_group_requests: config.max_group_requests.max(1),
            max_group_bytes: config.max_group_bytes.max(1),
            ..config
        };
        let (request_tx, request_rx) = tokio::sync::mpsc::channel(effective_config.queue_capacity);
        let (close_tx, close_rx) = tokio::sync::oneshot::channel();
        let metrics = Arc::new(Mutex::new(None));
        let storage_metrics = Arc::new(Mutex::new(None));
        let coordinator_metrics = Arc::new(Mutex::new(CoordinatorMetrics::default()));
        let shared = CoordinatorShared {
            metrics: Arc::clone(&metrics),
            storage_metrics: Arc::clone(&storage_metrics),
            coordinator_metrics: Arc::clone(&coordinator_metrics),
            read_view: Arc::clone(&read_view),
        };
        let coordinator_task = tokio::spawn(coordinator(
            store,
            request_rx,
            close_rx,
            effective_config,
            shared,
        ));
        Self {
            request_tx,
            close_tx: Mutex::new(Some(close_tx)),
            coordinator_task: Mutex::new(Some(coordinator_task)),
            metrics,
            storage_metrics,
            coordinator_metrics,
            read_view,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn execute(&self, request: BatchRequest) -> Result<BatchResponse> {
        self.execute_with_response_budget(request, usize::MAX).await
    }

    pub async fn execute_with_response_budget(
        &self,
        request: BatchRequest,
        max_response_bytes: usize,
    ) -> Result<BatchResponse> {
        if let Some(result) = self.try_read(&request, max_response_bytes) {
            return result;
        }
        match self
            .send(CoordinatorOperation::Batch {
                request,
                max_response_bytes,
            })
            .await?
        {
            CoordinatorResponse::Batch(response) => Ok(response),
            _ => Err(Error::invariant(
                "coordinator returned the wrong batch response",
            )),
        }
    }

    pub async fn execute_transaction(
        &self,
        request: TransactionRequest,
    ) -> Result<TransactionResult> {
        match self
            .send(CoordinatorOperation::Transaction(request))
            .await?
        {
            CoordinatorResponse::Transaction(result) => Ok(result),
            _ => Err(Error::invariant(
                "coordinator returned the wrong transaction response",
            )),
        }
    }

    pub async fn transact_get(&self, keys: Vec<DocumentKey>) -> Result<Vec<RevisionState>> {
        self.transact_get_with_response_budget(keys, usize::MAX)
            .await
    }

    pub async fn transact_get_with_response_budget(
        &self,
        keys: Vec<DocumentKey>,
        max_response_bytes: usize,
    ) -> Result<Vec<RevisionState>> {
        match self
            .send(CoordinatorOperation::TransactGet {
                keys,
                max_response_bytes,
            })
            .await?
        {
            CoordinatorResponse::TransactGet(result) => Ok(result),
            _ => Err(Error::invariant(
                "coordinator returned the wrong point-read response",
            )),
        }
    }

    pub async fn observe(&self, keys: Vec<DocumentKey>) -> Result<Vec<ObservedState>> {
        match self.send(CoordinatorOperation::Observe(keys)).await? {
            CoordinatorResponse::Observe(result) => Ok(result),
            _ => Err(Error::invariant(
                "coordinator returned the wrong observed state response",
            )),
        }
    }

    pub async fn checkpoint(&self) -> Result<CheckpointReport> {
        match self.send(CoordinatorOperation::Checkpoint).await? {
            CoordinatorResponse::Checkpoint(report) => Ok(report),
            _ => Err(Error::invariant(
                "coordinator returned the wrong checkpoint response",
            )),
        }
    }

    /// Runs the storage invariant checker at the coordinator serialization
    /// point. This is an explicit inspection hook for validation tools; it is
    /// not part of the client protocol.
    pub async fn check_invariants(&self) -> Result<InvariantReport> {
        match self.send(CoordinatorOperation::CheckInvariants).await? {
            CoordinatorResponse::Invariants(report) => Ok(report),
            _ => Err(Error::invariant(
                "coordinator returned the wrong invariant response",
            )),
        }
    }

    fn try_read(
        &self,
        request: &BatchRequest,
        max_response_bytes: usize,
    ) -> Option<Result<BatchResponse>> {
        let state = self.read_view.read().ok()?;
        if let Some(message) = &state.broken {
            return Some(Err(Error::durability(format!(
                "storage shard is not serving after an uncertain persistence failure: {message}"
            ))));
        }
        let view = state.view.as_ref()?;
        let mut budget = match super::ResponseBudget::new(max_response_bytes) {
            Ok(budget) => budget,
            Err(error) => return Some(Err(error)),
        };
        let result = match request {
            BatchRequest::Get { key } => view
                .get_with_budget(key, &mut budget)
                .map(BatchResponse::Get),
            BatchRequest::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => view
                .query_with_budget(pk, exclusive_after_sk.as_ref(), *limit, &mut budget)
                .map(BatchResponse::Query),
            BatchRequest::Scan {
                exclusive_after_key,
                limit,
            } => view
                .scan_with_budget(exclusive_after_key.as_ref(), *limit, &mut budget)
                .map(BatchResponse::Scan),
            BatchRequest::Put { .. } | BatchRequest::Delete { .. } => return None,
        };
        Some(result)
    }

    async fn send(&self, operation: CoordinatorOperation) -> Result<CoordinatorResponse> {
        let response_rx = match try_enqueue(&self.request_tx, operation) {
            Ok(response_rx) => response_rx,
            Err(Error::Overloaded(message)) => {
                record_overload(&self.coordinator_metrics);
                return Err(Error::overloaded(message));
            }
            Err(error) => return Err(error),
        };
        response_rx
            .await
            .map_err(|_| Error::invariant("storage coordinator dropped a response"))?
    }

    pub async fn shutdown(&self) -> Result<()> {
        let close_tx = self
            .close_tx
            .lock()
            .map_err(|_| Error::invariant("storage coordinator close lock is poisoned"))?
            .take();
        if let Some(close_tx) = close_tx {
            let _ = close_tx.send(());
        }
        let coordinator_task = self
            .coordinator_task
            .lock()
            .map_err(|_| Error::invariant("storage coordinator task lock is poisoned"))?
            .take();
        if let Some(coordinator_task) = coordinator_task {
            coordinator_task.await.map_err(|error| {
                Error::invariant(format!("storage coordinator task failed: {error}"))
            })?;
        }
        Ok(())
    }

    pub async fn close(self) -> Result<()> {
        self.shutdown().await
    }

    pub fn wal_metrics(&self) -> Option<WalMetrics> {
        self.metrics.lock().ok().and_then(|metrics| metrics.clone())
    }

    pub fn coordinator_metrics(&self) -> CoordinatorMetrics {
        self.coordinator_metrics
            .lock()
            .map(|metrics| metrics.clone())
            .unwrap_or_default()
    }

    pub fn storage_metrics(&self) -> Option<StorageMetrics> {
        self.storage_metrics
            .lock()
            .ok()
            .and_then(|metrics| metrics.clone())
    }
}

struct QueuedRequest {
    operation: CoordinatorOperation,
    response_tx: tokio::sync::oneshot::Sender<Result<CoordinatorResponse>>,
    enqueued_at: Instant,
}

#[derive(Clone)]
enum CoordinatorOperation {
    Batch {
        request: BatchRequest,
        max_response_bytes: usize,
    },
    Transaction(TransactionRequest),
    TransactGet {
        keys: Vec<DocumentKey>,
        max_response_bytes: usize,
    },
    Observe(Vec<DocumentKey>),
    Checkpoint,
    CheckInvariants,
}

enum CoordinatorResponse {
    Batch(BatchResponse),
    Transaction(TransactionResult),
    TransactGet(Vec<RevisionState>),
    Observe(Vec<ObservedState>),
    Checkpoint(CheckpointReport),
    Invariants(InvariantReport),
}

fn try_enqueue(
    request_tx: &tokio::sync::mpsc::Sender<QueuedRequest>,
    operation: CoordinatorOperation,
) -> Result<tokio::sync::oneshot::Receiver<Result<CoordinatorResponse>>> {
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    request_tx
        .try_send(QueuedRequest {
            operation,
            response_tx,
            enqueued_at: Instant::now(),
        })
        .map_err(|error| match error {
            tokio::sync::mpsc::error::TrySendError::Full(_) => {
                Error::overloaded("storage coordinator queue is full")
            }
            tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                Error::invariant("storage coordinator is closed")
            }
        })?;
    Ok(response_rx)
}

struct ReadViewState {
    view: Option<CommittedReadView>,
    broken: Option<String>,
}

struct CoordinatorShared {
    metrics: Arc<Mutex<Option<WalMetrics>>>,
    storage_metrics: Arc<Mutex<Option<StorageMetrics>>>,
    coordinator_metrics: Arc<Mutex<CoordinatorMetrics>>,
    read_view: Arc<RwLock<ReadViewState>>,
}

async fn coordinator<F: DurableFile + Send + 'static, W: DurableFile + Send + 'static>(
    mut store: BTreeStore<F, W>,
    mut request_rx: tokio::sync::mpsc::Receiver<QueuedRequest>,
    mut close_rx: tokio::sync::oneshot::Receiver<()>,
    config: CoordinatorConfig,
    shared: CoordinatorShared,
) {
    let mut pending = None;
    loop {
        let first = if let Some(request) = pending.take() {
            Some(request)
        } else {
            tokio::select! {
                request = request_rx.recv() => request,
                _ = &mut close_rx => return,
            }
        };
        let Some(first) = first else { return };
        let (requests, next_pending, close_requested, collection_nanos) =
            collect_group(first, &mut request_rx, &mut close_rx, config).await;
        pending = next_pending;

        let queue_wait_nanos = requests
            .iter()
            .map(|request| elapsed_nanos(request.enqueued_at))
            .fold(0u64, u64::saturating_add);
        let group_bytes = requests
            .iter()
            .map(|request| operation_size(&request.operation))
            .fold(0usize, usize::saturating_add);
        let logical_transactions = requests
            .iter()
            .filter(|request| is_mutation(&request.operation))
            .count();
        record_group(
            &shared.coordinator_metrics,
            requests.len(),
            logical_transactions,
            group_bytes,
            queue_wait_nanos,
            collection_nanos,
        );

        let operations = requests
            .iter()
            .map(|request| request.operation.clone())
            .collect::<Vec<_>>();
        let mut queued_requests = requests.into_iter().map(Some).collect::<Vec<_>>();
        let mut operation_start = 0;
        while operation_start < operations.len() {
            let operation_end = if is_mutation(&operations[operation_start]) {
                operations[operation_start..]
                    .iter()
                    .position(|operation| !is_mutation(operation))
                    .map_or(operations.len(), |offset| operation_start + offset)
            } else {
                operation_start + 1
            };
            let processing_started = Instant::now();
            let segment_result =
                process_segment(&mut store, &operations[operation_start..operation_end]);
            record_processing(
                &shared.coordinator_metrics,
                elapsed_nanos(processing_started),
            );
            if let Some(message) = store.degraded_reason() {
                mark_read_view_broken(&shared.read_view, message);
            }

            let read_view_result = publish_read_updates(&shared.read_view, &mut store);
            let segment_result = match (segment_result, read_view_result) {
                (Ok(result), Ok(())) => Ok(result),
                (Err(error), Ok(())) => Err(error),
                (_, Err(error)) => Err(error),
            };
            if let Ok(mut current) = shared.metrics.lock() {
                *current = store.wal_metrics().ok().flatten();
            }
            if let Ok(mut current) = shared.storage_metrics.lock() {
                *current = Some(store.storage_metrics());
            }
            match segment_result {
                Ok(responses) => {
                    for (queued, response) in queued_requests[operation_start..operation_end]
                        .iter_mut()
                        .zip(responses)
                    {
                        if let Some(queued) = queued.take() {
                            let _ = queued.response_tx.send(response);
                        }
                    }
                    operation_start = operation_end;
                }
                Err(error) => {
                    for queued in &mut queued_requests[operation_start..] {
                        if let Some(queued) = queued.take() {
                            let _ = queued.response_tx.send(Err(batch_error(&error)));
                        }
                    }
                    break;
                }
            }
        }
        if close_requested {
            return;
        }
    }
}

async fn collect_group(
    first: QueuedRequest,
    request_rx: &mut tokio::sync::mpsc::Receiver<QueuedRequest>,
    close_rx: &mut tokio::sync::oneshot::Receiver<()>,
    config: CoordinatorConfig,
) -> (Vec<QueuedRequest>, Option<QueuedRequest>, bool, u64) {
    let started = Instant::now();
    let mut requests = vec![first];
    let mut bytes = operation_size(&requests[0].operation);
    let mut pending = None;
    let mut close_requested = false;

    // One scheduler yield lets already-runnable callers enqueue without
    // imposing a timer delay on an otherwise idle first request.
    tokio::task::yield_now().await;
    loop {
        if requests.len() >= config.max_group_requests {
            break;
        }
        match request_rx.try_recv() {
            Ok(request) => {
                let request_bytes = operation_size(&request.operation);
                if can_add(requests.len(), bytes, request_bytes, config) {
                    bytes = bytes.saturating_add(request_bytes);
                    requests.push(request);
                } else {
                    pending = Some(request);
                    break;
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty)
            | Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
        }
    }

    if requests.len() > 1
        && pending.is_none()
        && requests.len() < config.max_group_requests
        && config.max_collection_delay > Duration::ZERO
    {
        let deadline = tokio::time::sleep(config.max_collection_delay);
        tokio::pin!(deadline);
        loop {
            tokio::select! {
                request = request_rx.recv() => {
                    let Some(request) = request else { break };
                    let request_bytes = operation_size(&request.operation);
                    if can_add(requests.len(), bytes, request_bytes, config) {
                        bytes = bytes.saturating_add(request_bytes);
                        requests.push(request);
                        if requests.len() >= config.max_group_requests {
                            break;
                        }
                    } else {
                        pending = Some(request);
                        break;
                    }
                }
                _ = &mut deadline => break,
                _ = &mut *close_rx => {
                    close_requested = true;
                    break;
                }
            }
        }
    }

    (requests, pending, close_requested, elapsed_nanos(started))
}

fn can_add(
    request_count: usize,
    current_bytes: usize,
    request_bytes: usize,
    config: CoordinatorConfig,
) -> bool {
    request_count < config.max_group_requests
        && current_bytes.saturating_add(request_bytes) <= config.max_group_bytes
}

fn process_segment<F: DurableFile, W: DurableFile>(
    store: &mut BTreeStore<F, W>,
    operations: &[CoordinatorOperation],
) -> Result<Vec<Result<CoordinatorResponse>>> {
    if operations.iter().all(is_mutation) {
        let transaction_requests = operations
            .iter()
            .map(as_transaction_request)
            .collect::<Result<Vec<_>>>()?;
        let results = store.apply_transaction_group(&transaction_requests)?;
        return Ok(results
            .into_iter()
            .zip(operations)
            .map(|(result, operation)| {
                result.and_then(|transaction| match operation {
                    CoordinatorOperation::Batch {
                        request: BatchRequest::Put { .. },
                        ..
                    } => transaction
                        .commit_lsn
                        .ok_or_else(|| Error::invariant("put transaction has no commit LSN"))
                        .map(|commit_lsn| {
                            CoordinatorResponse::Batch(BatchResponse::Put(Revision::from(
                                commit_lsn,
                            )))
                        }),
                    CoordinatorOperation::Batch {
                        request: BatchRequest::Delete { .. },
                        ..
                    } => transaction
                        .commit_lsn
                        .ok_or_else(|| Error::invariant("delete transaction has no commit LSN"))
                        .map(|commit_lsn| {
                            CoordinatorResponse::Batch(BatchResponse::Delete(Revision::from(
                                commit_lsn,
                            )))
                        }),
                    CoordinatorOperation::Transaction(_) => {
                        Ok(CoordinatorResponse::Transaction(transaction))
                    }
                    _ => unreachable!("non-mutation operation entered a write group"),
                })
            })
            .collect());
    }

    let responses = operations
        .iter()
        .map(|operation| match operation {
            CoordinatorOperation::Batch {
                request,
                max_response_bytes,
            } => store
                .apply_batch_with_response_budget(
                    std::slice::from_ref(request),
                    *max_response_bytes,
                )
                .map(|mut responses| CoordinatorResponse::Batch(responses.remove(0))),
            CoordinatorOperation::Transaction(request) => store
                .transact(request.clone())
                .map(CoordinatorResponse::Transaction),
            CoordinatorOperation::TransactGet {
                keys,
                max_response_bytes,
            } => store
                .transact_get_with_response_budget(keys, *max_response_bytes)
                .map(CoordinatorResponse::TransactGet),
            CoordinatorOperation::Observe(keys) => {
                store.observe(keys).map(CoordinatorResponse::Observe)
            }
            CoordinatorOperation::Checkpoint => {
                store.checkpoint().map(CoordinatorResponse::Checkpoint)
            }
            CoordinatorOperation::CheckInvariants => store
                .check_invariants()
                .map(CoordinatorResponse::Invariants),
        })
        .collect();
    Ok(responses)
}

fn is_mutation(operation: &CoordinatorOperation) -> bool {
    matches!(
        operation,
        CoordinatorOperation::Batch {
            request: BatchRequest::Put { .. },
            ..
        } | CoordinatorOperation::Batch {
            request: BatchRequest::Delete { .. },
            ..
        } | CoordinatorOperation::Transaction(_)
    )
}

fn as_transaction_request(operation: &CoordinatorOperation) -> Result<TransactionRequest> {
    match operation {
        CoordinatorOperation::Batch {
            request: BatchRequest::Put { key, value },
            ..
        } => Ok(TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: key.clone(),
                value: value.clone(),
            }],
        )),
        CoordinatorOperation::Batch {
            request: BatchRequest::Delete { key },
            ..
        } => Ok(TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Delete { key: key.clone() }],
        )),
        CoordinatorOperation::Transaction(request) => Ok(request.clone()),
        _ => Err(Error::invariant(
            "non-mutation operation cannot be committed",
        )),
    }
}

fn batch_error(error: &Error) -> Error {
    match error {
        Error::InvalidInput(message) => {
            Error::invalid_input(format!("batch was not published: {message}"))
        }
        Error::InvalidRequest(message) => {
            Error::invalid_request(format!("batch was not published: {message}"))
        }
        Error::Overloaded(message) => Error::overloaded(message.clone()),
        Error::ResponseTooLarge(message) => Error::response_too_large(message.clone()),
        Error::Conflict(conflict) => Error::conflict(conflict.clone()),
        Error::Corruption(message) => {
            Error::corruption(format!("batch was not published: {message}"))
        }
        Error::Io(error) => Error::Io(std::io::Error::new(
            error.kind(),
            format!("batch was not published: {error}"),
        )),
        Error::UnsupportedFormat(message) => {
            Error::unsupported_format(format!("batch was not published: {message}"))
        }
        Error::DurabilityFailure(message) => {
            Error::durability(format!("batch was not published: {message}"))
        }
        Error::RecoveryFailure(message) => {
            Error::recovery(format!("batch was not published: {message}"))
        }
        Error::CheckpointFailure(message) => {
            Error::checkpoint(format!("batch was not published: {message}"))
        }
        Error::InternalInvariantViolation(message) => {
            Error::invariant(format!("batch was not published: {message}"))
        }
    }
}

fn operation_size(operation: &CoordinatorOperation) -> usize {
    match operation {
        CoordinatorOperation::Batch {
            request: BatchRequest::Put { key, value },
            ..
        } => key
            .encode()
            .len()
            .saturating_add(value.len())
            .saturating_add(64),
        CoordinatorOperation::Batch {
            request: BatchRequest::Delete { key },
            ..
        } => key.encode().len().saturating_add(64),
        CoordinatorOperation::Batch {
            request: BatchRequest::Get { key },
            ..
        } => key.encode().len(),
        CoordinatorOperation::Batch {
            request:
                BatchRequest::Query {
                    pk,
                    exclusive_after_sk,
                    ..
                },
            ..
        } => pk.as_bytes().len().saturating_add(
            exclusive_after_sk
                .as_ref()
                .map_or(0, |key| key.as_bytes().len()),
        ),
        CoordinatorOperation::Batch {
            request:
                BatchRequest::Scan {
                    exclusive_after_key,
                    ..
                },
            ..
        } => exclusive_after_key
            .as_ref()
            .map_or(0, |key| key.encode().len()),
        CoordinatorOperation::Transaction(request) => request
            .conditions
            .iter()
            .map(|condition| condition.key().encode().len().saturating_add(32))
            .chain(request.mutations.iter().map(|mutation| {
                match mutation {
                    TransactionMutation::Put { key, value } => key
                        .encode()
                        .len()
                        .saturating_add(value.len())
                        .saturating_add(32),
                    TransactionMutation::Delete { key } => key.encode().len().saturating_add(32),
                }
            }))
            .fold(0usize, usize::saturating_add),
        CoordinatorOperation::TransactGet { keys, .. } => keys
            .iter()
            .map(|key| key.encode().len())
            .fold(0usize, usize::saturating_add),
        CoordinatorOperation::Observe(keys) => keys
            .iter()
            .map(|key| key.encode().len())
            .fold(0usize, usize::saturating_add),
        CoordinatorOperation::Checkpoint => 0,
        CoordinatorOperation::CheckInvariants => 0,
    }
}

fn elapsed_nanos(started: Instant) -> u64 {
    started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX)
}

fn value_length(value: &ValueRef) -> Result<usize> {
    match value {
        ValueRef::Inline(bytes) => Ok(bytes.len()),
        ValueRef::Overflow { length, .. } => usize::try_from(*length)
            .map_err(|_| Error::corruption("overflow value length does not fit usize")),
    }
}

fn record_overload(metrics: &Arc<Mutex<CoordinatorMetrics>>) {
    if let Ok(mut metrics) = metrics.lock() {
        metrics.overloaded_requests = metrics.overloaded_requests.saturating_add(1);
    }
}

fn record_group(
    metrics: &Arc<Mutex<CoordinatorMetrics>>,
    group_requests: usize,
    logical_transactions: usize,
    group_bytes: usize,
    queue_wait_nanos: u64,
    collection_nanos: u64,
) {
    if let Ok(mut metrics) = metrics.lock() {
        metrics.groups = metrics.groups.saturating_add(1);
        metrics.queued_requests = metrics
            .queued_requests
            .saturating_add(group_requests as u64);
        metrics.logical_transactions = metrics
            .logical_transactions
            .saturating_add(logical_transactions.try_into().unwrap_or(u64::MAX));
        metrics.queue_wait_nanos = metrics.queue_wait_nanos.saturating_add(queue_wait_nanos);
        metrics.batch_collection_nanos = metrics
            .batch_collection_nanos
            .saturating_add(collection_nanos);
        metrics.max_group_requests = metrics.max_group_requests.max(group_requests);
        metrics.max_group_bytes = metrics.max_group_bytes.max(group_bytes);
    }
}

fn record_processing(metrics: &Arc<Mutex<CoordinatorMetrics>>, processing_nanos: u64) {
    if let Ok(mut metrics) = metrics.lock() {
        metrics.processing_nanos = metrics.processing_nanos.saturating_add(processing_nanos);
    }
}

fn mark_read_view_broken(read_view: &Arc<RwLock<ReadViewState>>, message: &str) {
    if let Ok(mut state) = read_view.write() {
        state.broken = Some(message.to_owned());
    }
}

fn publish_read_updates<F: DurableFile, W: DurableFile>(
    read_view: &Arc<RwLock<ReadViewState>>,
    store: &mut BTreeStore<F, W>,
) -> Result<()> {
    let updates = store.take_published_page_updates();
    if updates.is_empty() {
        return Ok(());
    }
    let mut state = read_view
        .write()
        .map_err(|_| Error::invariant("committed read view lock is poisoned"))?;
    if let Some(view) = state.view.as_mut() {
        view.apply_updates(updates, store.root_page_id, store.high_water_page_id);
    }
    Ok(())
}

struct CommittedReadView {
    root_page_id: dodb_core::PageId,
    high_water_page_id: dodb_core::PageId,
    pages: BTreeMap<dodb_core::PageId, Arc<PageData>>,
}

impl CommittedReadView {
    fn from_store<F: DurableFile, W: DurableFile>(store: &mut BTreeStore<F, W>) -> Result<Self> {
        let root_page_id = store.root_page_id;
        let high_water_page_id = store.high_water_page_id;
        let mut pages = BTreeMap::new();
        let mut pending = vec![root_page_id];
        while let Some(page_id) = pending.pop() {
            if page_id.get() < super::FIRST_DATA_PAGE || page_id > high_water_page_id {
                return Err(Error::corruption(
                    "committed read view contains an out-of-range page",
                ));
            }
            if pages.contains_key(&page_id) {
                continue;
            }
            let page = Arc::new(store.read_page_from_file(page_id)?);
            match page.as_ref() {
                PageData::Internal {
                    leftmost_child,
                    entries,
                    ..
                } => {
                    pending.push(*leftmost_child);
                    pending.extend(entries.iter().map(|entry| entry.right_child));
                }
                PageData::Leaf {
                    next_leaf, entries, ..
                } => {
                    if let Some(next_leaf) = next_leaf {
                        pending.push(*next_leaf);
                    }
                    for entry in entries {
                        if let Some(ValueRef::Overflow { head, .. }) = &entry.value {
                            pending.push(*head);
                        }
                    }
                }
                PageData::Overflow { next, .. } => {
                    if let Some(next) = next {
                        pending.push(*next);
                    }
                }
                PageData::Free { .. } => {}
            }
            pages.insert(page_id, page);
        }
        Ok(Self {
            root_page_id,
            high_water_page_id,
            pages,
        })
    }

    fn apply_updates(
        &mut self,
        updates: BTreeMap<dodb_core::PageId, PageData>,
        root_page_id: dodb_core::PageId,
        high_water_page_id: dodb_core::PageId,
    ) {
        for (page_id, page) in updates {
            self.pages.insert(page_id, Arc::new(page));
        }
        self.root_page_id = root_page_id;
        self.high_water_page_id = high_water_page_id;
    }

    fn page(&self, page_id: dodb_core::PageId) -> Result<&PageData> {
        if page_id.get() < super::FIRST_DATA_PAGE || page_id > self.high_water_page_id {
            return Err(Error::corruption(format!(
                "page id {} is outside the committed read view",
                page_id.get()
            )));
        }
        self.pages
            .get(&page_id)
            .map(Arc::as_ref)
            .ok_or_else(|| Error::corruption("committed read view is missing a referenced page"))
    }

    fn find_leaf(&self, key: &[u8]) -> Result<dodb_core::PageId> {
        let mut page_id = self.root_page_id;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(page_id) {
                return Err(Error::corruption(
                    "read view tree traversal contains a cycle",
                ));
            }
            match self.page(page_id)? {
                PageData::Leaf { .. } => return Ok(page_id),
                PageData::Internal {
                    leftmost_child,
                    entries,
                    ..
                } => {
                    let index = entries.partition_point(|entry| entry.key.as_slice() <= key);
                    page_id = if index == 0 {
                        *leftmost_child
                    } else {
                        entries[index - 1].right_child
                    };
                }
                _ => return Err(Error::corruption("read view child is not a tree page")),
            }
        }
    }

    fn leftmost_leaf(&self) -> Result<dodb_core::PageId> {
        let mut page_id = self.root_page_id;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(page_id) {
                return Err(Error::corruption(
                    "read view leftmost traversal contains a cycle",
                ));
            }
            match self.page(page_id)? {
                PageData::Leaf { .. } => return Ok(page_id),
                PageData::Internal { leftmost_child, .. } => page_id = *leftmost_child,
                _ => return Err(Error::corruption("read view child is not a tree page")),
            }
        }
    }

    fn read_value(&self, value: &ValueRef) -> Result<Vec<u8>> {
        match value {
            ValueRef::Inline(value) => Ok(value.clone()),
            ValueRef::Overflow { head, length } => {
                let capacity = usize::try_from(*length)
                    .map_err(|_| Error::corruption("overflow value length does not fit usize"))?;
                let mut output = Vec::with_capacity(capacity);
                let mut page_id = Some(*head);
                let mut visited = HashSet::new();
                let mut count = 0usize;
                while let Some(current) = page_id {
                    if !visited.insert(current) || count >= MAX_OVERFLOW_PAGES {
                        return Err(Error::corruption("overflow chain is cyclic or too long"));
                    }
                    let PageData::Overflow {
                        next,
                        total_length,
                        chunk,
                        ..
                    } = self.page(current)?
                    else {
                        return Err(Error::corruption(
                            "value reference points to a non-overflow page",
                        ));
                    };
                    if *total_length != *length {
                        return Err(Error::corruption(
                            "overflow length metadata disagrees with leaf",
                        ));
                    }
                    output.extend_from_slice(chunk);
                    page_id = *next;
                    count += 1;
                }
                if output.len() as u64 != *length {
                    return Err(Error::corruption("overflow content length is invalid"));
                }
                Ok(output)
            }
        }
    }

    fn get_with_budget(
        &self,
        key: &DocumentKey,
        budget: &mut super::ResponseBudget,
    ) -> Result<RevisionState> {
        let encoded = key.encode();
        validate_encoded_key(&encoded)?;
        let leaf_id = self.find_leaf(&encoded)?;
        let PageData::Leaf { entries, .. } = self.page(leaf_id)? else {
            return Err(Error::corruption("read view lookup did not reach a leaf"));
        };
        let Some(entry) = entries.iter().find(|entry| entry.key == encoded) else {
            budget.reserve_state(0)?;
            return Ok(RevisionState::missing(Revision::ZERO));
        };
        match &entry.value {
            Some(value) => {
                budget.reserve_state(value_length(value)?)?;
                Ok(RevisionState::present(
                    self.read_value(value)?,
                    entry.revision,
                ))
            }
            None => {
                budget.reserve_state(0)?;
                Ok(RevisionState::missing(entry.revision))
            }
        }
    }

    fn query_with_budget(
        &self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
        budget: &mut super::ResponseBudget,
    ) -> Result<Vec<super::Document>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let start_key = DocumentKey::new(pk.as_bytes().to_vec(), Vec::new()).encode();
        validate_encoded_key(&start_key)?;
        let mut leaf_id = self.find_leaf(&start_key)?;
        let cursor = exclusive_after_sk
            .map(|sk| DocumentKey::new(pk.as_bytes().to_vec(), sk.as_bytes().to_vec()).encode());
        if let Some(cursor) = &cursor {
            validate_encoded_key(cursor)?;
        }
        let mut first = true;
        let mut visited = HashSet::new();
        let mut rows = Vec::new();
        while rows.len() < limit {
            if !visited.insert(leaf_id) {
                return Err(Error::corruption(
                    "leaf chain contains a cycle during query",
                ));
            }
            let PageData::Leaf {
                next_leaf, entries, ..
            } = self.page(leaf_id)?
            else {
                return Err(Error::corruption("query reached a non-leaf page"));
            };
            for entry in entries {
                if first && cursor.as_ref().is_some_and(|cursor| entry.key <= *cursor) {
                    continue;
                }
                first = false;
                let document_key = DocumentKey::decode(&entry.key).map_err(|error| {
                    Error::corruption(format!("leaf key decode failed: {error}"))
                })?;
                if document_key.pk != *pk {
                    if document_key.pk > *pk {
                        return Ok(rows);
                    }
                    continue;
                }
                if let Some(cursor) = exclusive_after_sk
                    && document_key.sk <= *cursor
                {
                    continue;
                }
                if let Some(value) = &entry.value {
                    budget.reserve_document(&document_key, value_length(value)?)?;
                    rows.push(super::Document {
                        key: document_key,
                        value: self.read_value(value)?,
                        revision: entry.revision,
                    });
                    if rows.len() == limit {
                        return Ok(rows);
                    }
                }
            }
            let Some(next) = next_leaf else { break };
            leaf_id = *next;
            first = false;
        }
        Ok(rows)
    }

    fn scan_with_budget(
        &self,
        cursor: Option<&DocumentKey>,
        limit: usize,
        budget: &mut super::ResponseBudget,
    ) -> Result<Vec<super::Document>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut leaf_id = match cursor {
            Some(key) => {
                let encoded = key.encode();
                validate_encoded_key(&encoded)?;
                self.find_leaf(&encoded)?
            }
            None => self.leftmost_leaf()?,
        };
        let cursor = cursor.map(DocumentKey::encode);
        let mut rows = Vec::new();
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(leaf_id) {
                return Err(Error::corruption("leaf chain contains a cycle during scan"));
            }
            let PageData::Leaf {
                next_leaf, entries, ..
            } = self.page(leaf_id)?
            else {
                return Err(Error::corruption("scan reached a non-leaf page"));
            };
            for entry in entries {
                if cursor.as_ref().is_some_and(|cursor| entry.key <= *cursor) {
                    continue;
                }
                let document_key = DocumentKey::decode(&entry.key).map_err(|error| {
                    Error::corruption(format!("leaf key decode failed: {error}"))
                })?;
                if let Some(value) = &entry.value {
                    budget.reserve_document(&document_key, value_length(value)?)?;
                    rows.push(super::Document {
                        key: document_key,
                        value: self.read_value(value)?,
                        revision: entry.revision,
                    });
                    if rows.len() == limit {
                        return Ok(rows);
                    }
                }
            }
            let Some(next) = next_leaf else { break };
            leaf_id = *next;
        }
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_operation(index: usize) -> CoordinatorOperation {
        CoordinatorOperation::Transaction(TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Delete {
                key: DocumentKey::new(b"test".to_vec(), index.to_le_bytes().to_vec()),
            }],
        ))
    }

    fn test_request(operation: CoordinatorOperation) -> QueuedRequest {
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        QueuedRequest {
            operation,
            response_tx,
            enqueued_at: Instant::now(),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn group_collection_is_bounded_and_deterministic() {
        let (request_tx, mut request_rx) = tokio::sync::mpsc::channel(8);
        request_tx
            .send(test_request(test_operation(1)))
            .await
            .unwrap();
        request_tx
            .send(test_request(test_operation(2)))
            .await
            .unwrap();
        let (_close_tx, mut close_rx) = tokio::sync::oneshot::channel();
        let first = test_request(test_operation(0));
        let (group, pending, close_requested, _) = collect_group(
            first,
            &mut request_rx,
            &mut close_rx,
            CoordinatorConfig {
                max_group_requests: 2,
                max_group_bytes: usize::MAX,
                max_collection_delay: Duration::ZERO,
                ..CoordinatorConfig::default()
            },
        )
        .await;
        assert_eq!(group.len(), 2);
        assert!(pending.is_none());
        assert!(!close_requested);
        assert!(request_rx.try_recv().is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn zero_byte_first_request_keeps_oversized_request_pending() {
        let (request_tx, mut request_rx) = tokio::sync::mpsc::channel(8);
        request_tx
            .send(test_request(test_operation(1)))
            .await
            .unwrap();
        request_tx
            .send(test_request(test_operation(2)))
            .await
            .unwrap();
        let (_close_tx, mut close_rx) = tokio::sync::oneshot::channel();
        let first = test_request(CoordinatorOperation::Checkpoint);
        let config = CoordinatorConfig {
            max_group_requests: 8,
            max_group_bytes: 1,
            max_collection_delay: Duration::ZERO,
            ..CoordinatorConfig::default()
        };

        let (group, pending, close_requested, _) =
            collect_group(first, &mut request_rx, &mut close_rx, config).await;

        assert_eq!(group.len(), 1);
        assert!(matches!(
            &group[0].operation,
            CoordinatorOperation::Checkpoint
        ));
        let pending = pending.expect("oversized request should remain pending");
        let CoordinatorOperation::Transaction(request) = pending.operation else {
            panic!("pending request should be a transaction");
        };
        let TransactionMutation::Delete { key } = &request.mutations[0] else {
            panic!("pending transaction should preserve the first queued request");
        };
        assert_eq!(key.sk.as_bytes(), 1usize.to_le_bytes());
        let queued_after_pending = request_rx.try_recv().unwrap();
        let CoordinatorOperation::Transaction(request) = queued_after_pending.operation else {
            panic!("next queued request should be a transaction");
        };
        let TransactionMutation::Delete { key } = &request.mutations[0] else {
            panic!("next transaction should preserve queue order");
        };
        assert_eq!(key.sk.as_bytes(), 2usize.to_le_bytes());
        assert!(!close_requested);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn oversized_first_request_is_allowed_as_its_own_group() {
        let (request_tx, mut request_rx) = tokio::sync::mpsc::channel(8);
        request_tx
            .send(test_request(test_operation(1)))
            .await
            .unwrap();
        let (_close_tx, mut close_rx) = tokio::sync::oneshot::channel();
        let first = test_request(test_operation(0));
        let config = CoordinatorConfig {
            max_group_requests: 8,
            max_group_bytes: 1,
            max_collection_delay: Duration::ZERO,
            ..CoordinatorConfig::default()
        };

        let (group, pending, close_requested, _) =
            collect_group(first, &mut request_rx, &mut close_rx, config).await;

        assert_eq!(group.len(), 1);
        assert!(pending.is_some());
        assert!(!close_requested);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn full_request_channel_returns_overloaded_without_waiting() {
        let (request_tx, _request_rx) = tokio::sync::mpsc::channel(1);
        try_enqueue(&request_tx, test_operation(0)).unwrap();
        let result = try_enqueue(&request_tx, test_operation(1));
        assert!(matches!(result, Err(Error::Overloaded(_))));
    }
}
