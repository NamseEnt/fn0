use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, c_char, c_void};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use dodb_core::{
    DocumentKey, Error, PrimaryKey, Revision, RevisionState, SortKey, TenantId,
    TransactionCondition, TransactionConflict, TransactionMutation, TransactionRequest,
};
use dodb_protocol::{
    MAX_STORAGE_ENCODED_KEY_SIZE, ProtocolLimits, ResponseEnvelope, encode_response,
};
use dodb_service::{
    DodbService, ExecutionBudget, Request, Response, ServiceFuture, ShutdownFuture,
    TransactionOutcome,
};
use tokio::sync::{Notify, OnceCell, Semaphore};

const USER_KEY_PREFIX: u8 = 1;
const METADATA_COUNTER_KEY: &[u8] = &[0, 1];
const VALUE_FORMAT: u8 = 1;
const PRESENT_TAG: u8 = 1;
const MISSING_TAG: u8 = 2;

unsafe extern "C" {
    fn dodb_rocksdb_open(path: *const c_char, error: *mut *mut c_char) -> *mut c_void;
    fn dodb_rocksdb_close(database: *mut c_void);
    fn dodb_rocksdb_iterator_new(
        database: *mut c_void,
        start_key: *const u8,
        start_key_length: usize,
        error: *mut *mut c_char,
    ) -> *mut c_void;
    fn dodb_rocksdb_iterator_valid(iterator: *mut c_void) -> i32;
    fn dodb_rocksdb_iterator_key(iterator: *mut c_void, key_length: *mut usize) -> *const u8;
    fn dodb_rocksdb_iterator_value(iterator: *mut c_void, value_length: *mut usize) -> *const u8;
    fn dodb_rocksdb_iterator_status(iterator: *mut c_void, error: *mut *mut c_char) -> i32;
    fn dodb_rocksdb_iterator_next(iterator: *mut c_void);
    fn dodb_rocksdb_iterator_destroy(iterator: *mut c_void);
    fn dodb_rocksdb_begin(database: *mut c_void, error: *mut *mut c_char) -> *mut c_void;
    fn dodb_rocksdb_get(
        database: *mut c_void,
        key: *const u8,
        key_length: usize,
        value: *mut *mut u8,
        value_length: *mut usize,
        error: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_transaction_get_for_update(
        transaction: *mut c_void,
        key: *const u8,
        key_length: usize,
        value: *mut *mut u8,
        value_length: *mut usize,
        error: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_transaction_put(
        transaction: *mut c_void,
        key: *const u8,
        key_length: usize,
        value: *const u8,
        value_length: usize,
        error: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_transaction_commit(transaction: *mut c_void, error: *mut *mut c_char) -> i32;
    fn dodb_rocksdb_transaction_rollback(transaction: *mut c_void);
    fn dodb_rocksdb_transaction_destroy(transaction: *mut c_void);
    fn dodb_rocksdb_value_free(value: *mut u8);
    fn dodb_rocksdb_error_free(error: *mut c_char);
}

#[derive(Clone, Debug)]
pub struct RocksDbServiceConfig {
    pub data_dir: PathBuf,
    pub max_blocking_operations: usize,
}

impl RocksDbServiceConfig {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            max_blocking_operations: 8,
        }
    }
}

pub struct RocksDbService {
    data_dir: PathBuf,
    databases: DashMap<TenantId, Arc<OnceCell<Arc<TenantDatabase>>>>,
    blocking_operations: Arc<Semaphore>,
    blocking_operation_limit: usize,
    lifecycle: LifecycleCoordinator,
    #[cfg(test)]
    blocking_operation_hook: std::sync::Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

struct TenantDatabase {
    handle: Arc<RocksDbHandle>,
}

struct RocksDbHandle(NonNull<c_void>);

unsafe impl Send for RocksDbHandle {}
unsafe impl Sync for RocksDbHandle {}

struct TransactionGuard {
    handle: NonNull<c_void>,
    _database: Arc<RocksDbHandle>,
    active: bool,
}

struct TenantReadIterator {
    handle: NonNull<c_void>,
    _database: Arc<RocksDbHandle>,
}

struct LifecycleCoordinator {
    state: Mutex<LifecycleState>,
    active_requests: AtomicUsize,
    changed: Notify,
    #[cfg(test)]
    shutdown_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

struct LifecycleState {
    shutting_down: bool,
}

struct RequestLease<'service> {
    lifecycle: &'service LifecycleCoordinator,
}

struct ShutdownLease<'service> {
    lifecycle: &'service LifecycleCoordinator,
}

impl RocksDbService {
    pub fn new(config: RocksDbServiceConfig) -> Result<Self, Error> {
        if config.max_blocking_operations == 0 {
            return Err(Error::invalid_request(
                "max_blocking_operations must be nonzero",
            ));
        }
        std::fs::create_dir_all(&config.data_dir)?;
        Ok(Self {
            data_dir: config.data_dir,
            databases: DashMap::new(),
            blocking_operations: Arc::new(Semaphore::new(config.max_blocking_operations)),
            blocking_operation_limit: config.max_blocking_operations,
            lifecycle: LifecycleCoordinator::new(),
            #[cfg(test)]
            blocking_operation_hook: std::sync::Mutex::new(None),
        })
    }

    pub async fn shutdown(&self) {
        let _shutdown_lease = self.lifecycle.begin_shutdown().await;
        let mut blocking_permits = Vec::with_capacity(self.blocking_operation_limit);
        for _ in 0..self.blocking_operation_limit {
            if let Ok(permit) = Arc::clone(&self.blocking_operations).acquire_owned().await {
                blocking_permits.push(permit);
            }
        }
        let cells = self
            .databases
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect::<Vec<_>>();
        self.databases.clear();
        let _ = tokio::task::spawn_blocking(move || drop(cells)).await;
        drop(blocking_permits);
    }

    pub fn tenant_path(&self, tenant: TenantId) -> PathBuf {
        self.data_dir.join(format!("tenant-{}", tenant.get()))
    }

    async fn database(
        &self,
        tenant: TenantId,
        create: bool,
    ) -> Result<Option<Arc<TenantDatabase>>, Error> {
        let path = self.tenant_path(tenant);
        if !create {
            let path_exists = self.run_blocking(move || Ok(path.exists())).await?;
            if !path_exists {
                return Ok(None);
            }
        }
        let path = self.tenant_path(tenant);
        let cell = self
            .databases
            .entry(tenant)
            .or_insert_with(|| Arc::new(OnceCell::new()))
            .clone();
        let semaphore = Arc::clone(&self.blocking_operations);
        let database = cell
            .get_or_try_init(|| async move {
                let permit = semaphore
                    .acquire_owned()
                    .await
                    .map_err(|_| Error::overloaded("blocking RocksDB executor is closed"))?;
                tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    TenantDatabase::open(&path).map(Arc::new)
                })
                .await
                .map_err(|error| Error::recovery(format!("RocksDB open task failed: {error}")))?
            })
            .await?;
        Ok(Some(Arc::clone(database)))
    }

    async fn run_blocking<T, F>(&self, operation: F) -> Result<T, Error>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, Error> + Send + 'static,
    {
        let permit = Arc::clone(&self.blocking_operations)
            .acquire_owned()
            .await
            .map_err(|_| Error::overloaded("blocking RocksDB executor is closed"))?;
        #[cfg(test)]
        let operation_hook = self.blocking_operation_hook.lock().unwrap().clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            #[cfg(test)]
            if let Some(operation_hook) = operation_hook {
                operation_hook();
            }
            operation()
        })
        .await
        .map_err(|error| Error::recovery(format!("RocksDB operation task failed: {error}")))?
    }

    async fn execute_request(
        &self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> Result<Response, Error> {
        let _request_lease = self.lifecycle.begin_request().await;
        match request {
            Request::Get { key } => {
                let state = match self.database(tenant, false).await? {
                    Some(database) => self.run_blocking(move || database.get(&key)).await?,
                    None => RevisionState::missing(Revision::ZERO),
                };
                let response = Response::Get(state);
                enforce_response_budget(&response, budget)?;
                Ok(response)
            }
            Request::Put { key, value } => {
                let database = self
                    .database(tenant, true)
                    .await?
                    .ok_or_else(|| Error::invariant("created tenant database was absent"))?;
                let request = TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put { key, value }],
                );
                let result = self
                    .run_blocking(move || database.transact(request))
                    .await?;
                let revision = result
                    .revision
                    .ok_or_else(|| Error::invariant("put transaction returned no revision"))?;
                let response = Response::Put(revision);
                enforce_response_budget(&response, budget)?;
                Ok(response)
            }
            Request::Delete { key } => {
                let database = self
                    .database(tenant, true)
                    .await?
                    .ok_or_else(|| Error::invariant("created tenant database was absent"))?;
                let request =
                    TransactionRequest::new(Vec::new(), vec![TransactionMutation::Delete { key }]);
                let result = self
                    .run_blocking(move || database.transact(request))
                    .await?;
                let revision = result
                    .revision
                    .ok_or_else(|| Error::invariant("delete transaction returned no revision"))?;
                let response = Response::Delete(revision);
                enforce_response_budget(&response, budget)?;
                Ok(response)
            }
            Request::Transact { request } => {
                request.validate()?;
                let has_mutations = !request.mutations.is_empty();
                let database = self.database(tenant, has_mutations).await?;
                let result = match database {
                    Some(database) => {
                        self.run_blocking(move || database.transact(request))
                            .await?
                    }
                    None => evaluate_uncreated(&request)?,
                };
                let outcome = match result.commit_lsn {
                    Some(commit_lsn) => TransactionOutcome::committed(commit_lsn),
                    None => TransactionOutcome::conditions_satisfied(),
                };
                let response = Response::Transact(outcome);
                enforce_response_budget(&response, budget)?;
                Ok(response)
            }
            Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => {
                let rows = match self.database(tenant, false).await? {
                    Some(database) => {
                        self.run_blocking(move || {
                            database.query(&pk, exclusive_after_sk.as_ref(), limit, budget)
                        })
                        .await?
                    }
                    None => {
                        let response = Response::Query(Vec::new());
                        enforce_response_budget(&response, budget)?;
                        return Ok(response);
                    }
                };
                Ok(Response::Query(rows))
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                let rows = match self.database(tenant, false).await? {
                    Some(database) => {
                        self.run_blocking(move || {
                            database.scan(exclusive_after_key.as_ref(), limit, budget)
                        })
                        .await?
                    }
                    None => {
                        let response = Response::Scan(Vec::new());
                        enforce_response_budget(&response, budget)?;
                        return Ok(response);
                    }
                };
                Ok(Response::Scan(rows))
            }
        }
    }
}

impl LifecycleCoordinator {
    fn new() -> Self {
        Self {
            state: Mutex::new(LifecycleState {
                shutting_down: false,
            }),
            active_requests: AtomicUsize::new(0),
            changed: Notify::new(),
            #[cfg(test)]
            shutdown_hook: Mutex::new(None),
        }
    }

    async fn begin_request(&self) -> RequestLease<'_> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let accepted = {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.shutting_down {
                    false
                } else {
                    self.active_requests.fetch_add(1, Ordering::AcqRel);
                    true
                }
            };
            if accepted {
                return RequestLease { lifecycle: self };
            }
            notified.await;
        }
    }

    async fn begin_shutdown(&self) -> ShutdownLease<'_> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let started = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.shutting_down {
                    false
                } else {
                    state.shutting_down = true;
                    true
                }
            };
            if started {
                break;
            }
            notified.await;
        }

        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.active_requests.load(Ordering::Acquire) == 0 {
                break;
            }
            notified.await;
        }
        #[cfg(test)]
        if let Some(shutdown_hook) = self
            .shutdown_hook
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            shutdown_hook();
        }
        ShutdownLease { lifecycle: self }
    }
}

impl Drop for RequestLease<'_> {
    fn drop(&mut self) {
        self.lifecycle
            .active_requests
            .fetch_sub(1, Ordering::AcqRel);
        self.lifecycle.changed.notify_waiters();
    }
}

impl Drop for ShutdownLease<'_> {
    fn drop(&mut self) {
        let mut state = self
            .lifecycle
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.shutting_down = false;
        drop(state);
        self.lifecycle.changed.notify_waiters();
    }
}

impl TenantDatabase {
    fn open(path: &Path) -> Result<Self, Error> {
        let path = path
            .to_str()
            .ok_or_else(|| Error::invalid_input("tenant path is not valid UTF-8"))?;
        let path = std::ffi::CString::new(path)
            .map_err(|_| Error::invalid_input("tenant path contains a NUL byte"))?;
        let mut error = std::ptr::null_mut();
        let handle = unsafe { dodb_rocksdb_open(path.as_ptr(), &mut error) };
        let handle = NonNull::new(handle).ok_or_else(|| ffi_error(error, "RocksDB open failed"))?;
        Ok(Self {
            handle: Arc::new(RocksDbHandle(handle)),
        })
    }

    fn get(&self, key: &DocumentKey) -> Result<RevisionState, Error> {
        let encoded_key = user_key(key);
        let mut value = std::ptr::null_mut();
        let mut value_length = 0;
        let mut error = std::ptr::null_mut();
        let result = unsafe {
            dodb_rocksdb_get(
                self.handle.0.as_ptr(),
                encoded_key.as_ptr(),
                encoded_key.len(),
                &mut value,
                &mut value_length,
                &mut error,
            )
        };
        match result {
            1 => Ok(RevisionState::missing(Revision::ZERO)),
            2 => Err(ffi_error(error, "RocksDB get failed")),
            _ => {
                let bytes = unsafe { copy_and_free_value(value, value_length) };
                decode_state(&bytes)
            }
        }
    }

    fn read_iterator(&self, start_key: &[u8]) -> Result<TenantReadIterator, Error> {
        let mut error = std::ptr::null_mut();
        let iterator = unsafe {
            dodb_rocksdb_iterator_new(
                self.handle.0.as_ptr(),
                start_key.as_ptr(),
                start_key.len(),
                &mut error,
            )
        };
        NonNull::new(iterator)
            .map(|handle| TenantReadIterator {
                handle,
                _database: Arc::clone(&self.handle),
            })
            .ok_or_else(|| ffi_error(error, "RocksDB snapshot iterator creation failed"))
    }

    fn query(
        &self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
        maximum_response_bytes: ExecutionBudget,
    ) -> Result<Vec<dodb_service::Document>, Error> {
        let mut budget = ReadResponseBudget::new(maximum_response_bytes.max_response_bytes)?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        validate_encoded_key(&DocumentKey::new(pk.as_bytes().to_vec(), Vec::new()))?;
        if let Some(cursor) = exclusive_after_sk {
            validate_encoded_key(&DocumentKey::new(
                pk.as_bytes().to_vec(),
                cursor.as_bytes().to_vec(),
            ))?;
        }
        let start_key = exclusive_after_sk
            .map(|sort_key| {
                user_key(&DocumentKey::new(
                    pk.as_bytes().to_vec(),
                    sort_key.as_bytes().to_vec(),
                ))
            })
            .unwrap_or_else(|| user_partition_prefix(pk));
        let iterator = self.read_iterator(&start_key)?;
        let mut documents = Vec::new();
        while let Some((encoded_key, value_pointer, value_length)) = iterator.current()? {
            if encoded_key.first() != Some(&USER_KEY_PREFIX) {
                break;
            }
            let document_key = DocumentKey::decode(&encoded_key[1..]).map_err(|error| {
                Error::corruption(format!("RocksDB document key decode failed: {error}"))
            })?;
            if document_key.pk != *pk {
                break;
            }
            if exclusive_after_sk.is_some_and(|cursor| document_key.sk <= *cursor) {
                iterator.next();
                continue;
            }
            if let Some((revision, value_length)) =
                decode_present_record(value_pointer, value_length)?
            {
                budget.reserve_document(&document_key, value_length)?;
                documents.push(dodb_service::Document {
                    key: document_key,
                    value: unsafe {
                        std::slice::from_raw_parts(value_pointer.add(10), value_length).to_vec()
                    },
                    revision,
                });
                if documents.len() == limit {
                    return Ok(documents);
                }
            }
            iterator.next();
        }
        Ok(documents)
    }

    fn scan(
        &self,
        exclusive_after_key: Option<&DocumentKey>,
        limit: usize,
        maximum_response_bytes: ExecutionBudget,
    ) -> Result<Vec<dodb_service::Document>, Error> {
        let mut budget = ReadResponseBudget::new(maximum_response_bytes.max_response_bytes)?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        if let Some(cursor) = exclusive_after_key {
            validate_encoded_key(cursor)?;
        }
        let start_key = exclusive_after_key
            .map(user_key)
            .unwrap_or_else(|| vec![USER_KEY_PREFIX]);
        let iterator = self.read_iterator(&start_key)?;
        let mut documents = Vec::new();
        while let Some((encoded_key, value_pointer, value_length)) = iterator.current()? {
            if encoded_key.first() != Some(&USER_KEY_PREFIX) {
                break;
            }
            let document_key = DocumentKey::decode(&encoded_key[1..]).map_err(|error| {
                Error::corruption(format!("RocksDB document key decode failed: {error}"))
            })?;
            if exclusive_after_key.is_some_and(|cursor| document_key <= *cursor) {
                iterator.next();
                continue;
            }
            if let Some((revision, value_length)) =
                decode_present_record(value_pointer, value_length)?
            {
                budget.reserve_document(&document_key, value_length)?;
                documents.push(dodb_service::Document {
                    key: document_key,
                    value: unsafe {
                        std::slice::from_raw_parts(value_pointer.add(10), value_length).to_vec()
                    },
                    revision,
                });
                if documents.len() == limit {
                    return Ok(documents);
                }
            }
            iterator.next();
        }
        Ok(documents)
    }

    fn transact(&self, request: TransactionRequest) -> Result<LogicalTransactionResult, Error> {
        self.transact_with_begin_hook(request, || {})
    }

    fn transact_with_begin_hook<F>(
        &self,
        request: TransactionRequest,
        on_begin: F,
    ) -> Result<LogicalTransactionResult, Error>
    where
        F: FnOnce(),
    {
        request.validate()?;
        let mut error = std::ptr::null_mut();
        let transaction = unsafe { dodb_rocksdb_begin(self.handle.0.as_ptr(), &mut error) };
        let transaction = NonNull::new(transaction)
            .ok_or_else(|| ffi_error(error, "RocksDB transaction begin failed"))?;
        let mut transaction_guard = TransactionGuard {
            handle: transaction,
            _database: Arc::clone(&self.handle),
            active: true,
        };
        on_begin();
        let mut encoded_keys = BTreeSet::new();
        for condition in &request.conditions {
            encoded_keys.insert(user_key(condition.key()));
        }
        for mutation in &request.mutations {
            encoded_keys.insert(user_key(mutation.key()));
        }
        let mut states = BTreeMap::new();
        for encoded_key in encoded_keys {
            let state = match transaction_get_for_update(&transaction_guard, &encoded_key)? {
                Some(encoded_state) => decode_state(&encoded_state)?,
                None => RevisionState::missing(Revision::ZERO),
            };
            states.insert(encoded_key, state);
        }
        for condition in &request.conditions {
            let actual = states
                .get(&user_key(condition.key()))
                .ok_or_else(|| Error::invariant("transaction condition state was not locked"))?;
            if !condition_matches(condition, actual) {
                return Err(Error::conflict(TransactionConflict {
                    key: condition.key().clone(),
                    expected: condition.expectation(),
                    actual: actual.observed(),
                }));
            }
        }
        if request.mutations.is_empty() {
            transaction_guard.rollback();
            return Ok(LogicalTransactionResult {
                commit_lsn: None,
                revision: None,
            });
        }
        let counter = transaction_get_for_update(&transaction_guard, METADATA_COUNTER_KEY)?;
        let current_revision = counter
            .as_deref()
            .map(counter_bytes_to_u64)
            .transpose()?
            .unwrap_or(0);
        let next_revision = current_revision
            .checked_add(1)
            .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?;
        for mutation in &request.mutations {
            let (key, state) = match mutation {
                TransactionMutation::Put { key, value } => (
                    user_key(key),
                    RevisionState::present(value.clone(), Revision::new(next_revision)),
                ),
                TransactionMutation::Delete { key } => (
                    user_key(key),
                    RevisionState::missing(Revision::new(next_revision)),
                ),
            };
            let encoded_state = encode_state(&state);
            transaction_put(&transaction_guard, &key, &encoded_state)?;
        }
        transaction_put(
            &transaction_guard,
            METADATA_COUNTER_KEY,
            &next_revision.to_be_bytes(),
        )?;
        let mut commit_error = std::ptr::null_mut();
        let commit_result = unsafe {
            dodb_rocksdb_transaction_commit(transaction_guard.handle.as_ptr(), &mut commit_error)
        };
        if commit_result != 0 {
            return Err(Error::durability(unsafe {
                take_error(commit_error, "RocksDB transaction commit failed")
            }));
        }
        transaction_guard.finish();
        Ok(LogicalTransactionResult {
            commit_lsn: Some(dodb_core::Lsn::new(next_revision)),
            revision: Some(Revision::new(next_revision)),
        })
    }
}

impl Drop for RocksDbHandle {
    fn drop(&mut self) {
        unsafe { dodb_rocksdb_close(self.0.as_ptr()) };
    }
}

impl TransactionGuard {
    fn rollback(&mut self) {
        if self.active {
            unsafe { dodb_rocksdb_transaction_rollback(self.handle.as_ptr()) };
            self.active = false;
        }
    }

    fn finish(&mut self) {
        if self.active {
            unsafe { dodb_rocksdb_transaction_destroy(self.handle.as_ptr()) };
            self.active = false;
        }
    }
}

impl TenantReadIterator {
    fn current(&self) -> Result<Option<(Vec<u8>, *const u8, usize)>, Error> {
        if unsafe { dodb_rocksdb_iterator_valid(self.handle.as_ptr()) } == 0 {
            let mut error = std::ptr::null_mut();
            let result = unsafe { dodb_rocksdb_iterator_status(self.handle.as_ptr(), &mut error) };
            if result != 0 {
                return Err(ffi_error(error, "RocksDB iterator failed"));
            }
            return Ok(None);
        }
        let mut key_length = 0;
        let key = unsafe { dodb_rocksdb_iterator_key(self.handle.as_ptr(), &mut key_length) };
        let mut value_length = 0;
        let value = unsafe { dodb_rocksdb_iterator_value(self.handle.as_ptr(), &mut value_length) };
        let key = unsafe { std::slice::from_raw_parts(key, key_length) }.to_vec();
        Ok(Some((key, value, value_length)))
    }

    fn next(&self) {
        unsafe { dodb_rocksdb_iterator_next(self.handle.as_ptr()) };
    }
}

impl Drop for TenantReadIterator {
    fn drop(&mut self) {
        unsafe { dodb_rocksdb_iterator_destroy(self.handle.as_ptr()) };
    }
}

impl Drop for TransactionGuard {
    fn drop(&mut self) {
        self.rollback();
    }
}

#[derive(Clone, Copy)]
struct LogicalTransactionResult {
    commit_lsn: Option<dodb_core::Lsn>,
    revision: Option<Revision>,
}

impl DodbService for RocksDbService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move { self.execute_request(tenant, request, budget).await })
    }

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async move { self.shutdown().await })
    }
}

fn user_key(key: &DocumentKey) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(key.encoded_len() + 1);
    encoded.push(USER_KEY_PREFIX);
    encoded.extend_from_slice(&key.encode());
    encoded
}

fn user_partition_prefix(pk: &PrimaryKey) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(pk.as_bytes().len() + 3);
    encoded.push(USER_KEY_PREFIX);
    let partition = DocumentKey::new(pk.as_bytes().to_vec(), Vec::new()).encode();
    encoded.extend_from_slice(&partition[..partition.len() - 2]);
    encoded
}

fn validate_encoded_key(key: &DocumentKey) -> Result<(), Error> {
    let encoded_length = key.encoded_len();
    if encoded_length > MAX_STORAGE_ENCODED_KEY_SIZE {
        return Err(Error::invalid_input(format!(
            "encoded document key is {encoded_length} bytes, maximum is {MAX_STORAGE_ENCODED_KEY_SIZE}"
        )));
    }
    Ok(())
}

fn decode_present_record(
    encoded_pointer: *const u8,
    encoded_length: usize,
) -> Result<Option<(Revision, usize)>, Error> {
    if encoded_length < 10 {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let encoded = unsafe { std::slice::from_raw_parts(encoded_pointer, encoded_length) };
    if encoded[0] != VALUE_FORMAT {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let revision = Revision::new(u64::from_be_bytes(
        encoded[2..10]
            .try_into()
            .map_err(|_| Error::corruption("invalid RocksDB revision bytes"))?,
    ));
    match encoded[1] {
        PRESENT_TAG => Ok(Some((revision, encoded_length - 10))),
        MISSING_TAG if encoded.len() == 10 => Ok(None),
        MISSING_TAG => Err(Error::corruption("tombstone contains value bytes")),
        _ => Err(Error::corruption("unknown RocksDB logical value state")),
    }
}

struct ReadResponseBudget {
    maximum_wire: usize,
    maximum_memory: usize,
    used_wire: usize,
    used_memory: usize,
}

impl ReadResponseBudget {
    fn new(maximum: usize) -> Result<Self, Error> {
        if maximum < 17 {
            return Err(Error::response_too_large(format!(
                "response envelope exceeds {maximum} bytes"
            )));
        }
        Ok(Self {
            maximum_wire: maximum,
            maximum_memory: maximum.saturating_add(64 * 1024),
            used_wire: 17,
            used_memory: 0,
        })
    }

    fn reserve_document(&mut self, key: &DocumentKey, value_length: usize) -> Result<(), Error> {
        let materialized = key
            .pk
            .as_bytes()
            .len()
            .saturating_add(key.sk.as_bytes().len())
            .saturating_add(value_length)
            .saturating_add(64);
        let wire = key
            .pk
            .as_bytes()
            .len()
            .saturating_add(key.sk.as_bytes().len())
            .saturating_add(value_length)
            .saturating_add(20);
        let next_wire = self
            .used_wire
            .checked_add(wire)
            .ok_or_else(|| Error::response_too_large("encoded response size overflows"))?;
        let next_memory = self
            .used_memory
            .checked_add(materialized.max(wire))
            .ok_or_else(|| Error::response_too_large("response materialization size overflows"))?;
        if next_wire > self.maximum_wire || next_memory > self.maximum_memory {
            return Err(Error::response_too_large(format!(
                "response materialization exceeds {} bytes",
                self.maximum_wire
            )));
        }
        self.used_wire = next_wire;
        self.used_memory = next_memory;
        Ok(())
    }
}

fn encode_state(state: &RevisionState) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.push(VALUE_FORMAT);
    match state {
        RevisionState::Present { value, revision } => {
            encoded.push(PRESENT_TAG);
            encoded.extend_from_slice(&revision.get().to_be_bytes());
            encoded.extend_from_slice(value);
        }
        RevisionState::Missing { revision } => {
            encoded.push(MISSING_TAG);
            encoded.extend_from_slice(&revision.get().to_be_bytes());
        }
    }
    encoded
}

fn decode_state(encoded: &[u8]) -> Result<RevisionState, Error> {
    if encoded.len() < 10 || encoded[0] != VALUE_FORMAT {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let revision = Revision::new(u64::from_be_bytes(
        encoded[2..10]
            .try_into()
            .map_err(|_| Error::corruption("invalid RocksDB revision bytes"))?,
    ));
    match encoded[1] {
        PRESENT_TAG => Ok(RevisionState::present(encoded[10..].to_vec(), revision)),
        MISSING_TAG if encoded.len() == 10 => Ok(RevisionState::missing(revision)),
        MISSING_TAG => Err(Error::corruption("tombstone contains value bytes")),
        _ => Err(Error::corruption("unknown RocksDB logical value state")),
    }
}

fn counter_bytes_to_u64(encoded: &[u8]) -> Result<u64, Error> {
    Ok(u64::from_be_bytes(encoded.try_into().map_err(|_| {
        Error::corruption("RocksDB revision counter has invalid length")
    })?))
}

fn condition_matches(condition: &TransactionCondition, actual: &RevisionState) -> bool {
    match condition {
        TransactionCondition::RevisionEquals {
            expected_revision, ..
        } => actual.revision() == *expected_revision,
        TransactionCondition::Exists { .. } => !actual.is_missing(),
        TransactionCondition::NotExists { .. } => actual.is_missing(),
    }
}

fn evaluate_uncreated(request: &TransactionRequest) -> Result<LogicalTransactionResult, Error> {
    let actual = RevisionState::missing(Revision::ZERO);
    for condition in &request.conditions {
        if !condition_matches(condition, &actual) {
            return Err(Error::conflict(TransactionConflict {
                key: condition.key().clone(),
                expected: condition.expectation(),
                actual: actual.observed(),
            }));
        }
    }
    Ok(LogicalTransactionResult {
        commit_lsn: None,
        revision: None,
    })
}

fn transaction_put(transaction: &TransactionGuard, key: &[u8], value: &[u8]) -> Result<(), Error> {
    let mut error = std::ptr::null_mut();
    let result = unsafe {
        dodb_rocksdb_transaction_put(
            transaction.handle.as_ptr(),
            key.as_ptr(),
            key.len(),
            value.as_ptr(),
            value.len(),
            &mut error,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(ffi_error(error, "RocksDB transaction write failed"))
    }
}

fn transaction_get_for_update(
    transaction: &TransactionGuard,
    key: &[u8],
) -> Result<Option<Vec<u8>>, Error> {
    let mut value = std::ptr::null_mut();
    let mut value_length = 0;
    let mut error = std::ptr::null_mut();
    let result = unsafe {
        dodb_rocksdb_transaction_get_for_update(
            transaction.handle.as_ptr(),
            key.as_ptr(),
            key.len(),
            &mut value,
            &mut value_length,
            &mut error,
        )
    };
    match result {
        1 => Ok(None),
        2 => Err(ffi_error(error, "RocksDB transaction read failed")),
        _ => Ok(Some(unsafe { copy_and_free_value(value, value_length) })),
    }
}

fn enforce_response_budget(response: &Response, budget: ExecutionBudget) -> Result<(), Error> {
    let limits = ProtocolLimits {
        max_response_frame_size: budget.max_response_bytes,
        ..ProtocolLimits::default()
    };
    encode_response(&ResponseEnvelope::Success(response.clone()), limits)
        .map(|_| ())
        .map_err(|error| Error::response_too_large(format!("response exceeds budget: {error}")))
}

unsafe fn copy_and_free_value(value: *mut u8, value_length: usize) -> Vec<u8> {
    let bytes = unsafe { std::slice::from_raw_parts(value, value_length) }.to_vec();
    unsafe { dodb_rocksdb_value_free(value) };
    bytes
}

fn ffi_error(error: *mut c_char, fallback: &str) -> Error {
    let message = unsafe { take_error(error, fallback) };
    Error::Io(std::io::Error::other(message))
}

unsafe fn take_error(error: *mut c_char, fallback: &str) -> String {
    let Some(error) = NonNull::new(error) else {
        return fallback.to_owned();
    };
    let message = unsafe { CStr::from_ptr(error.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    unsafe { dodb_rocksdb_error_free(error.as_ptr()) };
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;
    use tempfile::TempDir;

    fn temporary_db_directory() -> TempDir {
        match std::env::var_os("DODB_TEST_DATA_DIR") {
            Some(path) => TempDir::new_in(path).unwrap(),
            None => TempDir::new().unwrap(),
        }
    }

    #[test]
    fn get_completes_while_same_tenant_transaction_is_open() {
        let directory = temporary_db_directory();
        let database = Arc::new(TenantDatabase::open(directory.path()).unwrap());
        let key = DocumentKey::new(b"concurrent-read".to_vec(), b"key".to_vec());
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"committed".to_vec(),
                }],
            ))
            .unwrap();

        let (transaction_started_sender, transaction_started_receiver) = mpsc::channel();
        let (continue_transaction_sender, continue_transaction_receiver) = mpsc::channel();
        let transaction_database = Arc::clone(&database);
        let transaction_key = key.clone();
        let transaction = std::thread::spawn(move || {
            transaction_database.transact_with_begin_hook(
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: transaction_key,
                        value: b"uncommitted".to_vec(),
                    }],
                ),
                || {
                    transaction_started_sender.send(()).unwrap();
                    continue_transaction_receiver
                        .recv_timeout(Duration::from_secs(10))
                        .expect("transaction continuation timed out");
                },
            )
        });

        transaction_started_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("transaction did not reach its open-transaction hook");
        let (get_result_sender, get_result_receiver) = mpsc::channel();
        let get_database = Arc::clone(&database);
        let get_key = key.clone();
        let get = std::thread::spawn(move || {
            get_result_sender.send(get_database.get(&get_key)).unwrap();
        });
        let state = get_result_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("Get waited for the open transaction");
        assert_eq!(
            state.unwrap(),
            RevisionState::present(b"committed".to_vec(), Revision::new(1))
        );

        continue_transaction_sender.send(()).unwrap();
        transaction.join().unwrap().unwrap();
        get.join().unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_waits_for_an_in_flight_request_and_service_reopens() {
        let directory = temporary_db_directory();
        let service =
            Arc::new(RocksDbService::new(RocksDbServiceConfig::new(directory.path())).unwrap());
        let tenant = TenantId::new(41);
        let key = DocumentKey::new(b"lifecycle".to_vec(), b"key".to_vec());
        service
            .execute(
                tenant,
                Request::Put {
                    key: key.clone(),
                    value: b"alive".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();

        let (operation_started_sender, operation_started_receiver) = mpsc::channel();
        let (continue_operation_sender, continue_operation_receiver) = mpsc::channel();
        let continue_operation_receiver = std::sync::Mutex::new(continue_operation_receiver);
        let should_pause = Arc::new(AtomicBool::new(true));
        *service.blocking_operation_hook.lock().unwrap() = Some(Arc::new(move || {
            if should_pause.swap(false, Ordering::SeqCst) {
                operation_started_sender.send(()).unwrap();
                continue_operation_receiver
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10))
                    .expect("request continuation timed out");
            }
        }));

        let request_service = Arc::clone(&service);
        let request = tokio::spawn(async move {
            request_service
                .execute(tenant, Request::Get { key }, ExecutionBudget::new(1024))
                .await
        });
        tokio::task::spawn_blocking(move || {
            operation_started_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("request did not reach its blocking operation");
        })
        .await
        .unwrap();

        let shutdown_service = Arc::clone(&service);
        let (shutdown_started_sender, shutdown_started_receiver) = tokio::sync::oneshot::channel();
        let shutdown = tokio::spawn(async move {
            let _ = shutdown_started_sender.send(());
            shutdown_service.shutdown().await;
        });
        shutdown_started_receiver.await.unwrap();
        continue_operation_sender.send(()).unwrap();

        assert_eq!(
            request.await.unwrap().unwrap(),
            Response::Get(RevisionState::present(b"alive".to_vec(), Revision::new(1)))
        );
        shutdown.await.unwrap();
        *service.blocking_operation_hook.lock().unwrap() = None;

        assert_eq!(
            service
                .execute(
                    tenant,
                    Request::Get {
                        key: DocumentKey::new(b"lifecycle".to_vec(), b"key".to_vec())
                    },
                    ExecutionBudget::new(1024),
                )
                .await
                .unwrap(),
            Response::Get(RevisionState::present(b"alive".to_vec(), Revision::new(1)))
        );
        service.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_drains_a_blocking_operation_after_its_request_is_cancelled() {
        let directory = temporary_db_directory();
        let service =
            Arc::new(RocksDbService::new(RocksDbServiceConfig::new(directory.path())).unwrap());
        let tenant = TenantId::new(42);
        let key = DocumentKey::new(b"cancelled-request".to_vec(), b"key".to_vec());
        service
            .execute(
                tenant,
                Request::Put {
                    key: key.clone(),
                    value: b"alive".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();

        let (operation_started_sender, operation_started_receiver) = mpsc::channel();
        let (continue_operation_sender, continue_operation_receiver) = mpsc::channel();
        let continue_operation_receiver = std::sync::Mutex::new(continue_operation_receiver);
        let operation_index = Arc::new(AtomicUsize::new(0));
        *service.blocking_operation_hook.lock().unwrap() = Some(Arc::new(move || {
            if operation_index.fetch_add(1, Ordering::SeqCst) == 1 {
                operation_started_sender.send(()).unwrap();
                continue_operation_receiver
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10))
                    .expect("request continuation timed out");
            }
        }));

        let request_service = Arc::clone(&service);
        let request_key = key.clone();
        let request = tokio::spawn(async move {
            request_service
                .execute(
                    tenant,
                    Request::Get { key: request_key },
                    ExecutionBudget::new(1024),
                )
                .await
        });
        tokio::task::spawn_blocking(move || {
            operation_started_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("RocksDB Get did not reach its blocking operation");
        })
        .await
        .unwrap();
        request.abort();
        assert!(request.await.unwrap_err().is_cancelled());

        let (shutdown_started_sender, shutdown_started_receiver) = mpsc::channel();
        *service.lifecycle.shutdown_hook.lock().unwrap() = Some(Arc::new(move || {
            shutdown_started_sender.send(()).unwrap();
        }));
        let shutdown_service = Arc::clone(&service);
        let shutdown = tokio::spawn(async move { shutdown_service.shutdown().await });
        tokio::task::spawn_blocking(move || {
            shutdown_started_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("shutdown did not close request admission");
        })
        .await
        .unwrap();
        continue_operation_sender.send(()).unwrap();
        shutdown.await.unwrap();
        *service.blocking_operation_hook.lock().unwrap() = None;
        *service.lifecycle.shutdown_hook.lock().unwrap() = None;

        assert_eq!(
            service
                .execute(tenant, Request::Get { key }, ExecutionBudget::new(1024),)
                .await
                .unwrap(),
            Response::Get(RevisionState::present(b"alive".to_vec(), Revision::new(1)))
        );
        service.shutdown().await;
    }

    #[test]
    fn read_iterator_keeps_database_alive_after_tenant_owner_drops() {
        let directory = temporary_db_directory();
        let database = TenantDatabase::open(directory.path()).unwrap();
        let key = DocumentKey::new(b"iterator-lifetime".to_vec(), b"key".to_vec());
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"retained".to_vec(),
                }],
            ))
            .unwrap();
        let iterator = database
            .read_iterator(&user_partition_prefix(&PrimaryKey::new(
                b"iterator-lifetime".to_vec(),
            )))
            .unwrap();
        drop(database);
        assert_eq!(iterator_value(&iterator), (key, b"retained".to_vec()));
        drop(iterator);
    }

    #[test]
    fn read_iterator_keeps_one_snapshot_across_atomic_transaction_commit() {
        let directory = temporary_db_directory();
        let database = TenantDatabase::open(directory.path()).unwrap();
        let first_key = DocumentKey::new(b"snapshot".to_vec(), b"a".to_vec());
        let second_key = DocumentKey::new(b"snapshot".to_vec(), b"b".to_vec());
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: first_key.clone(),
                        value: b"old-a".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: second_key.clone(),
                        value: b"old-b".to_vec(),
                    },
                ],
            ))
            .unwrap();

        let iterator = database
            .read_iterator(&user_partition_prefix(&PrimaryKey::new(
                b"snapshot".to_vec(),
            )))
            .unwrap();
        assert_eq!(
            iterator_value(&iterator),
            (first_key.clone(), b"old-a".to_vec())
        );
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: first_key,
                        value: b"new-a".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: second_key.clone(),
                        value: b"new-b".to_vec(),
                    },
                ],
            ))
            .unwrap();
        iterator.next();
        assert_eq!(
            iterator_value(&iterator),
            (second_key.clone(), b"old-b".to_vec())
        );
        drop(iterator);

        let current = database
            .query(
                &PrimaryKey::new(b"snapshot".to_vec()),
                None,
                10,
                ExecutionBudget::new(1024),
            )
            .unwrap();
        assert_eq!(current[0].value, b"new-a");
        assert_eq!(current[1].value, b"new-b");
    }

    fn iterator_value(iterator: &TenantReadIterator) -> (DocumentKey, Vec<u8>) {
        let (encoded_key, value_pointer, value_length) = iterator.current().unwrap().unwrap();
        let document_key = DocumentKey::decode(&encoded_key[1..]).unwrap();
        let (revision, content_length) = decode_present_record(value_pointer, value_length)
            .unwrap()
            .unwrap();
        let content =
            unsafe { std::slice::from_raw_parts(value_pointer.add(10), content_length).to_vec() };
        assert!(revision.get() > 0);
        (document_key, content)
    }
}
