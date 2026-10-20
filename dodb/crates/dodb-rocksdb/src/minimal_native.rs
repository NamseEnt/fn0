use std::ffi::{CString, c_void};
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use dodb_core::{
    DocumentKey, Error, PrimaryKey, Revision, RevisionState, SortKey, TenantId,
    TransactionConflict, TransactionMutation, TransactionRequest,
};
use dodb_service::{
    Document, DodbService, ExecutionBudget, Request, Response, ServiceFuture, ShutdownFuture,
    TransactionOutcome,
};

use crate::{
    DEFAULT_REVISION_RANGE_SIZE, METADATA_COUNTER_KEY, ReadResponseBudget, RevisionAllocatorMode,
    RocksDbHandle, TransactionGuard, USER_KEY_PREFIX, condition_matches, counter_bytes_to_u64,
    decode_and_free_get_value, decode_state, decode_value_header, dodb_rocksdb_begin,
    dodb_rocksdb_get, dodb_rocksdb_iterator_new, encode_state, enforce_response_budget,
    evaluate_uncreated, ffi_error, transaction_get_for_update, transaction_put, user_key,
    user_partition_prefix, validate_encoded_key,
};

const REVISION_RANGE_SIZE: u64 = DEFAULT_REVISION_RANGE_SIZE;

#[derive(Clone)]
pub struct MinimalNativeConfig {
    pub data_dir: PathBuf,
    pub revision_allocator: RevisionAllocatorMode,
}

impl MinimalNativeConfig {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            revision_allocator: RevisionAllocatorMode::ReservedRanges,
        }
    }
}

pub struct MinimalNativeService {
    data_dir: PathBuf,
    tenants: DashMap<TenantId, Arc<MinimalNativeTenant>>,
    tenant_open: Mutex<()>,
}

struct MinimalNativeTenant {
    handle: Arc<RocksDbHandle>,
    revisions: Mutex<RevisionRange>,
}

#[derive(Default)]
struct RevisionRange {
    next: u64,
    end: u64,
}

struct MinimalIterator {
    handle: NonNull<c_void>,
    _database: Arc<RocksDbHandle>,
}

unsafe impl Send for MinimalIterator {}

impl MinimalNativeService {
    pub fn new(config: MinimalNativeConfig) -> Result<Self, Error> {
        std::fs::create_dir_all(&config.data_dir)?;
        if config.revision_allocator != RevisionAllocatorMode::ReservedRanges {
            return Err(Error::invalid_input(
                "minimal native baseline requires reserved-range revisions",
            ));
        }
        Ok(Self {
            data_dir: config.data_dir,
            tenants: DashMap::new(),
            tenant_open: Mutex::new(()),
        })
    }

    pub fn tenant_path(&self, tenant: TenantId) -> PathBuf {
        self.data_dir.join(format!("tenant-{}", tenant.get()))
    }

    fn tenant(
        &self,
        tenant: TenantId,
        create: bool,
    ) -> Result<Option<Arc<MinimalNativeTenant>>, Error> {
        let _open_guard = self
            .tenant_open
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(database) = self.tenants.get(&tenant) {
            return Ok(Some(Arc::clone(database.value())));
        }
        let path = self.tenant_path(tenant);
        if !create && !path.exists() {
            return Ok(None);
        }
        if create {
            std::fs::create_dir_all(&path)?;
        }
        let path_text = path
            .to_str()
            .ok_or_else(|| Error::invalid_input("tenant path is not valid UTF-8"))?;
        let path_c = CString::new(path_text)
            .map_err(|_| Error::invalid_input("tenant path contains a NUL byte"))?;
        let mut error = std::ptr::null_mut();
        let handle = unsafe { crate::dodb_rocksdb_open(path_c.as_ptr(), &mut error) };
        let handle = NonNull::new(handle).ok_or_else(|| ffi_error(error, "RocksDB open failed"))?;
        let database = Arc::new(MinimalNativeTenant {
            handle: Arc::new(RocksDbHandle(handle)),
            revisions: Mutex::new(RevisionRange::default()),
        });
        let entry = self
            .tenants
            .entry(tenant)
            .or_insert_with(|| Arc::clone(&database));
        Ok(Some(Arc::clone(entry.value())))
    }

    fn execute_request(
        &self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> Result<Response, Error> {
        let response = match request {
            Request::Get { key } => {
                let state = match self.tenant(tenant, false)? {
                    Some(database) => database.get(&key, budget)?,
                    None => RevisionState::missing(Revision::ZERO),
                };
                Response::Get(state)
            }
            Request::Put { key, value } => {
                let database = self.tenant(tenant, true)?.ok_or_else(|| {
                    Error::invariant("minimal native tenant database was not created")
                })?;
                let revision = database.transact(TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put { key, value }],
                ))?;
                Response::Put(
                    revision.ok_or_else(|| {
                        Error::invariant("minimal native put returned no revision")
                    })?,
                )
            }
            Request::Delete { key } => {
                let database = self.tenant(tenant, true)?.ok_or_else(|| {
                    Error::invariant("minimal native tenant database was not created")
                })?;
                let revision = database.transact(TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Delete { key }],
                ))?;
                Response::Delete(revision.ok_or_else(|| {
                    Error::invariant("minimal native delete returned no revision")
                })?)
            }
            Request::Transact { request } => {
                request.validate()?;
                let database = self.tenant(tenant, !request.mutations.is_empty())?;
                let revision = match database {
                    Some(database) => database.transact(request)?,
                    None => {
                        let result = evaluate_uncreated(&request)?;
                        result.revision
                    }
                };
                Response::Transact(match revision {
                    Some(revision) => {
                        TransactionOutcome::committed(dodb_core::Lsn::new(revision.get()))
                    }
                    None => TransactionOutcome::conditions_satisfied(),
                })
            }
            Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => {
                let documents = match self.tenant(tenant, false)? {
                    Some(database) => {
                        database.query(&pk, exclusive_after_sk.as_ref(), limit, budget)?
                    }
                    None => Vec::new(),
                };
                Response::Query(documents)
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                let documents = match self.tenant(tenant, false)? {
                    Some(database) => database.scan(exclusive_after_key.as_ref(), limit, budget)?,
                    None => Vec::new(),
                };
                Response::Scan(documents)
            }
        };
        enforce_response_budget(&response, budget)?;
        Ok(response)
    }
}

impl DodbService for MinimalNativeService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move { self.execute_request(tenant, request, budget) })
    }

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async move { self.tenants.clear() })
    }
}

impl MinimalNativeTenant {
    fn get(&self, key: &DocumentKey, budget: ExecutionBudget) -> Result<RevisionState, Error> {
        validate_encoded_key(key)?;
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
            2 => Err(ffi_error(error, "RocksDB Get failed")),
            _ => unsafe { decode_and_free_get_value(value, value_length, budget) },
        }
    }

    fn transact(&self, request: TransactionRequest) -> Result<Option<Revision>, Error> {
        request.validate()?;
        let mut transaction = begin_transaction(&self.handle)?;
        let mut locked_states = std::collections::BTreeMap::new();
        let mut transaction_keys = std::collections::BTreeMap::new();
        for condition in &request.conditions {
            transaction_keys
                .entry(user_key(condition.key()))
                .or_insert(());
        }
        for mutation in &request.mutations {
            transaction_keys.insert(user_key(mutation.key()), ());
        }
        for encoded_key in transaction_keys.keys() {
            let mut lock_wait_nanos = 0;
            let mut lock_wait_count = 0;
            let encoded = transaction_get_for_update(
                &transaction,
                encoded_key,
                true,
                false,
                &mut lock_wait_nanos,
                &mut lock_wait_count,
                None,
            )?;
            let state = match encoded {
                Some(value) => decode_state(&value)?,
                None => RevisionState::missing(Revision::ZERO),
            };
            locked_states.insert(encoded_key.clone(), state);
        }
        for condition in &request.conditions {
            let actual = locked_states
                .get(&user_key(condition.key()))
                .ok_or_else(|| Error::invariant("minimal native condition key was not locked"))?;
            if !condition_matches(condition, actual) {
                transaction.rollback();
                return Err(Error::conflict(TransactionConflict {
                    key: condition.key().clone(),
                    expected: condition.expectation(),
                    actual: actual.observed(),
                }));
            }
        }
        if request.mutations.is_empty() {
            transaction.rollback();
            return Ok(None);
        }
        let revision = self.allocate_revision()?;
        for mutation in &request.mutations {
            let state = match mutation {
                TransactionMutation::Put { value, .. } => {
                    RevisionState::present(value.clone(), revision)
                }
                TransactionMutation::Delete { .. } => RevisionState::missing(revision),
            };
            transaction_put(
                &transaction,
                &user_key(mutation.key()),
                &encode_state(&state),
            )?;
        }
        commit_transaction(transaction)?;
        Ok(Some(revision))
    }

    fn allocate_revision(&self) -> Result<Revision, Error> {
        let mut range = self
            .revisions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if range.next == range.end {
            let reservation = begin_transaction(&self.handle)?;
            let mut lock_wait_nanos = 0;
            let mut lock_wait_count = 0;
            let current = transaction_get_for_update(
                &reservation,
                METADATA_COUNTER_KEY,
                true,
                false,
                &mut lock_wait_nanos,
                &mut lock_wait_count,
                None,
            )?
            .as_deref()
            .map(counter_bytes_to_u64)
            .transpose()?
            .unwrap_or(0);
            let first = current
                .checked_add(1)
                .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?;
            let end = current
                .checked_add(REVISION_RANGE_SIZE)
                .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?;
            transaction_put(&reservation, METADATA_COUNTER_KEY, &end.to_be_bytes())?;
            commit_transaction(reservation)?;
            range.next = first;
            range.end = end
                .checked_add(1)
                .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?;
        }
        let revision = range.next;
        range.next = range
            .next
            .checked_add(1)
            .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?;
        Ok(Revision::new(revision))
    }

    fn iterator(&self, start_key: &[u8]) -> Result<MinimalIterator, Error> {
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
            .map(|handle| MinimalIterator {
                handle,
                _database: Arc::clone(&self.handle),
            })
            .ok_or_else(|| ffi_error(error, "RocksDB snapshot iterator creation failed"))
    }

    fn query(
        &self,
        pk: &PrimaryKey,
        cursor: Option<&SortKey>,
        limit: usize,
        response_budget: ExecutionBudget,
    ) -> Result<Vec<Document>, Error> {
        let mut budget = ReadResponseBudget::new(response_budget.max_response_bytes)?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        validate_encoded_key(&DocumentKey::new(pk.as_bytes().to_vec(), Vec::new()))?;
        if let Some(cursor) = cursor {
            validate_encoded_key(&DocumentKey::new(
                pk.as_bytes().to_vec(),
                cursor.as_bytes().to_vec(),
            ))?;
        }
        let start_key = cursor
            .map(|sort_key| {
                user_key(&DocumentKey::new(
                    pk.as_bytes().to_vec(),
                    sort_key.as_bytes().to_vec(),
                ))
            })
            .unwrap_or_else(|| user_partition_prefix(pk));
        let iterator = self.iterator(&start_key)?;
        let mut documents = Vec::new();
        while let Some((encoded_key, value_pointer, value_length)) = iterator.current()? {
            if encoded_key.first() != Some(&USER_KEY_PREFIX) {
                break;
            }
            let document_key = decode_document_key(&encoded_key)?;
            if document_key.pk != *pk {
                break;
            }
            if cursor.is_some_and(|cursor| document_key.sk <= *cursor) {
                iterator.next();
                continue;
            }
            if let Some((revision, payload_length)) =
                decode_present_record(value_pointer, value_length)?
            {
                budget.reserve_document(&document_key, payload_length)?;
                documents.push(Document {
                    key: document_key,
                    value: unsafe {
                        std::slice::from_raw_parts(value_pointer.add(10), payload_length).to_vec()
                    },
                    revision,
                });
                if documents.len() == limit {
                    break;
                }
            }
            iterator.next();
        }
        Ok(documents)
    }

    fn scan(
        &self,
        cursor: Option<&DocumentKey>,
        limit: usize,
        response_budget: ExecutionBudget,
    ) -> Result<Vec<Document>, Error> {
        let mut budget = ReadResponseBudget::new(response_budget.max_response_bytes)?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        if let Some(cursor) = cursor {
            validate_encoded_key(cursor)?;
        }
        let start_key = cursor
            .map(user_key)
            .unwrap_or_else(|| vec![USER_KEY_PREFIX]);
        let iterator = self.iterator(&start_key)?;
        let mut documents = Vec::new();
        while let Some((encoded_key, value_pointer, value_length)) = iterator.current()? {
            if encoded_key.first() != Some(&USER_KEY_PREFIX) {
                break;
            }
            let document_key = decode_document_key(&encoded_key)?;
            if cursor.is_some_and(|cursor| document_key <= *cursor) {
                iterator.next();
                continue;
            }
            if let Some((revision, payload_length)) =
                decode_present_record(value_pointer, value_length)?
            {
                budget.reserve_document(&document_key, payload_length)?;
                documents.push(Document {
                    key: document_key,
                    value: unsafe {
                        std::slice::from_raw_parts(value_pointer.add(10), payload_length).to_vec()
                    },
                    revision,
                });
                if documents.len() == limit {
                    break;
                }
            }
            iterator.next();
        }
        Ok(documents)
    }
}

impl MinimalIterator {
    fn current(&self) -> Result<Option<(Vec<u8>, *const u8, usize)>, Error> {
        if unsafe { crate::dodb_rocksdb_iterator_valid(self.handle.as_ptr()) } == 0 {
            let mut error = std::ptr::null_mut();
            let result =
                unsafe { crate::dodb_rocksdb_iterator_status(self.handle.as_ptr(), &mut error) };
            if result != 0 {
                return Err(ffi_error(error, "RocksDB iterator failed"));
            }
            return Ok(None);
        }
        let mut key_length = 0;
        let key =
            unsafe { crate::dodb_rocksdb_iterator_key(self.handle.as_ptr(), &mut key_length) };
        let mut value_length = 0;
        let value =
            unsafe { crate::dodb_rocksdb_iterator_value(self.handle.as_ptr(), &mut value_length) };
        let key = unsafe { std::slice::from_raw_parts(key, key_length) }.to_vec();
        Ok(Some((key, value, value_length)))
    }

    fn next(&self) {
        unsafe { crate::dodb_rocksdb_iterator_next(self.handle.as_ptr()) };
    }
}

impl Drop for MinimalIterator {
    fn drop(&mut self) {
        unsafe { crate::dodb_rocksdb_iterator_destroy(self.handle.as_ptr()) };
    }
}

fn begin_transaction(database: &Arc<RocksDbHandle>) -> Result<TransactionGuard, Error> {
    let mut error = std::ptr::null_mut();
    let transaction = unsafe { dodb_rocksdb_begin(database.0.as_ptr(), &mut error) };
    let handle = NonNull::new(transaction)
        .ok_or_else(|| ffi_error(error, "RocksDB transaction begin failed"))?;
    Ok(TransactionGuard {
        handle,
        _database: Arc::clone(database),
        active: true,
    })
}

fn commit_transaction(mut transaction: TransactionGuard) -> Result<(), Error> {
    let mut error = std::ptr::null_mut();
    let result =
        unsafe { crate::dodb_rocksdb_transaction_commit(transaction.handle.as_ptr(), &mut error) };
    if result != 0 {
        return Err(Error::durability(unsafe {
            crate::take_error(error, "RocksDB transaction commit failed")
        }));
    }
    transaction.finish();
    Ok(())
}

fn decode_document_key(encoded_key: &[u8]) -> Result<DocumentKey, Error> {
    DocumentKey::decode(&encoded_key[1..])
        .map_err(|error| Error::corruption(format!("RocksDB document key decode failed: {error}")))
}

fn decode_present_record(
    encoded_pointer: *const u8,
    encoded_length: usize,
) -> Result<Option<(Revision, usize)>, Error> {
    if encoded_length < 10 || encoded_pointer.is_null() {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let encoded = unsafe { std::slice::from_raw_parts(encoded_pointer, encoded_length) };
    let (revision, value_length) = decode_value_header(encoded)?;
    Ok(value_length.map(|value_length| (revision, value_length)))
}
