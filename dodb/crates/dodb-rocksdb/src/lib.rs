use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, c_char, c_void};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use dodb_core::{
    DocumentKey, Error, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionConflict, TransactionMutation, TransactionRequest,
};
use dodb_protocol::{ProtocolLimits, ResponseEnvelope, encode_response};
use dodb_service::{
    DodbService, ExecutionBudget, Request, Response, ServiceFuture, ShutdownFuture,
    TransactionOutcome,
};
use tokio::sync::{OnceCell, Semaphore};

const USER_KEY_PREFIX: u8 = 1;
const METADATA_COUNTER_KEY: &[u8] = &[0, 1];
const VALUE_FORMAT: u8 = 1;
const PRESENT_TAG: u8 = 1;
const MISSING_TAG: u8 = 2;

unsafe extern "C" {
    fn dodb_rocksdb_open(path: *const c_char, error: *mut *mut c_char) -> *mut c_void;
    fn dodb_rocksdb_close(database: *mut c_void);
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
}

struct TenantDatabase {
    handle: Mutex<NonNull<c_void>>,
}

unsafe impl Send for TenantDatabase {}
unsafe impl Sync for TenantDatabase {}

struct TransactionGuard {
    handle: NonNull<c_void>,
    active: bool,
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
        })
    }

    pub async fn shutdown(&self) {
        let cells = self
            .databases
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect::<Vec<_>>();
        self.databases.clear();
        for cell in cells {
            let _ = self
                .run_blocking(move || {
                    drop(cell);
                    Ok(())
                })
                .await;
        }
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
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
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
            Request::Query { .. } | Request::Scan { .. } => Err(Error::invalid_request(
                "Query and Scan are not implemented by dodb-rocksdb R1a",
            )),
        }
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
            handle: Mutex::new(handle),
        })
    }

    fn get(&self, key: &DocumentKey) -> Result<RevisionState, Error> {
        let _guard = self
            .handle
            .lock()
            .map_err(|_| Error::invariant("tenant RocksDB lock was poisoned"))?;
        let encoded_key = user_key(key);
        let mut value = std::ptr::null_mut();
        let mut value_length = 0;
        let mut error = std::ptr::null_mut();
        let result = unsafe {
            dodb_rocksdb_get(
                _guard.as_ptr(),
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

    fn transact(&self, request: TransactionRequest) -> Result<LogicalTransactionResult, Error> {
        request.validate()?;
        let guard = self
            .handle
            .lock()
            .map_err(|_| Error::invariant("tenant RocksDB lock was poisoned"))?;
        let mut error = std::ptr::null_mut();
        let transaction = unsafe { dodb_rocksdb_begin(guard.as_ptr(), &mut error) };
        let transaction = NonNull::new(transaction)
            .ok_or_else(|| ffi_error(error, "RocksDB transaction begin failed"))?;
        let mut transaction_guard = TransactionGuard {
            handle: transaction,
            active: true,
        };
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

impl Drop for TenantDatabase {
    fn drop(&mut self) {
        let handle = self
            .handle
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { dodb_rocksdb_close(handle.as_ptr()) };
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
