use crate::{ConditionalOp, ConditionalOutcome, Database};
use anyhow::{Result, anyhow, bail};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    any::type_name,
    cell::UnsafeCell,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
};
use tracing::Instrument;

/// Wrapper around UnsafeCell that asserts Send/Sync when T: Send.
/// DocHandle holds the only outstanding reference within a trx, so concurrent
/// access cannot occur in practice; the trx future itself is the unit of work.
#[repr(transparent)]
pub(crate) struct SyncUnsafeCell<T>(UnsafeCell<T>);

unsafe impl<T: Send> Send for SyncUnsafeCell<T> {}
unsafe impl<T: Send> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DocKey {
    pub pk: String,
    pub sk: String,
}

impl DocKey {
    pub fn new(pk: impl Into<String>, sk: impl Into<String>) -> Self {
        Self {
            pk: pk.into(),
            sk: sk.into(),
        }
    }
}

pub trait Document: Serialize + DeserializeOwned + Send + Sync + 'static {
    fn key(&self) -> DocKey;
}

pub trait DocGet {
    type Doc: Document;
    fn key(&self) -> DocKey;
}

#[allow(async_fn_in_trait)]
pub trait TrxRead: Sized {
    type Output;
    fn collect_keys(&self, keys: &mut Vec<DocKey>);
    async fn finalize(
        self,
        tx: &Trx,
        results: &mut std::vec::IntoIter<Option<crate::StoredDoc>>,
    ) -> Result<Self::Output>;
}

impl<R> TrxRead for R
where
    R: DocGet,
{
    type Output = Option<DocHandle<R::Doc>>;

    fn collect_keys(&self, keys: &mut Vec<DocKey>) {
        keys.push(self.key());
    }

    async fn finalize(
        self,
        tx: &Trx,
        results: &mut std::vec::IntoIter<Option<crate::StoredDoc>>,
    ) -> Result<Self::Output> {
        let stored = results
            .next()
            .ok_or_else(|| anyhow!("trx batch result missing for read"))?;
        let key = self.key();
        tx.inner
            .lock()
            .unwrap()
            .register_loaded::<R::Doc>(key, stored)
    }
}

macro_rules! impl_trx_read_tuple {
    ($($T:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($T: TrxRead),+> TrxRead for ($($T,)+) {
            type Output = ($($T::Output,)+);

            fn collect_keys(&self, keys: &mut Vec<DocKey>) {
                let ($($T,)+) = self;
                $($T.collect_keys(keys);)+
            }

            async fn finalize(
                self,
                tx: &Trx,
                results: &mut std::vec::IntoIter<Option<crate::StoredDoc>>,
            ) -> Result<Self::Output> {
                let ($($T,)+) = self;
                Ok(($($T.finalize(tx, results).await?,)+))
            }
        }
    };
}

impl_trx_read_tuple!(A);
impl_trx_read_tuple!(A, B);
impl_trx_read_tuple!(A, B, C);
impl_trx_read_tuple!(A, B, C, D);
impl_trx_read_tuple!(A, B, C, D, E);
impl_trx_read_tuple!(A, B, C, D, E, F);
impl_trx_read_tuple!(A, B, C, D, E, F, G);
impl_trx_read_tuple!(A, B, C, D, E, F, G, H);
impl_trx_read_tuple!(A, B, C, D, E, F, G, H, I);
impl_trx_read_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_trx_read_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_trx_read_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

pub struct DocHandle<T> {
    data: Arc<SyncUnsafeCell<T>>,
    dirty: Arc<AtomicBool>,
    deleted: Arc<AtomicBool>,
    _alive: Arc<()>,
    _marker: PhantomData<Arc<T>>,
}

impl<T> DocHandle<T> {
    pub fn delete(&self) {
        self.deleted.store(true, Ordering::Release);
    }
}

impl<T> Deref for DocHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.get() }
    }
}

impl<T> DerefMut for DocHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty.store(true, Ordering::Release);
        unsafe { &mut *self.data.get() }
    }
}

pub struct Trx {
    inner: Arc<Mutex<TrxState>>,
}

impl Trx {
    #[tracing::instrument(skip_all)]
    pub async fn get<R>(&self, request: R) -> Result<R::Output>
    where
        R: TrxRead,
    {
        let mut keys = Vec::new();
        request.collect_keys(&mut keys);

        {
            let state = self.inner.lock().unwrap();
            for key in &keys {
                if state.index.contains_key(key) {
                    bail!("duplicate trx key access: {}/{}", key.pk, key.sk);
                }
            }
        }

        let key_pairs: Vec<(String, String)> =
            keys.iter().map(|k| (k.pk.clone(), k.sk.clone())).collect();
        let stored = self.batch_load(&key_pairs).await?;

        let mut iter = stored.into_iter();
        request.finalize(self, &mut iter).await
    }

    pub fn create<T>(&self, doc: T) -> Result<DocHandle<T>>
    where
        T: Document,
    {
        self.inner.lock().unwrap().create(doc)
    }

    pub fn commit<Out, Cancel>(self, out: Out) -> Result<TrxControl<Out, Cancel>> {
        Ok(TrxControl {
            inner: TrxControlInner::Commit(out),
        })
    }

    pub fn cancel<Out, Cancel>(self, reason: Cancel) -> Result<TrxControl<Out, Cancel>> {
        Ok(TrxControl {
            inner: TrxControlInner::Cancel(reason),
        })
    }

    async fn batch_load(&self, keys: &[(String, String)]) -> Result<Vec<Option<crate::StoredDoc>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let db = self.inner.lock().unwrap().db.clone();
        db.batch_get_with_version(keys).await
    }
}

pub struct TrxControl<Out, Cancel> {
    inner: TrxControlInner<Out, Cancel>,
}

enum TrxControlInner<Out, Cancel> {
    Commit(Out),
    Cancel(Cancel),
}

#[derive(Debug)]
pub enum TrxResult<Out, Cancel, Err> {
    Committed(Out),
    Cancelled(Cancel),
    Conflict(ConflictDetails),
    Err(Err),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConflictDetails {
    pub keys: Vec<ConflictKey>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictKey {
    pub key: DocKey,
    pub expected_version: Option<i64>,
    pub actual_version: Option<i64>,
}

const MAX_ATTEMPTS: u32 = 5;
const BACKOFF_BASE_MS: u64 = 50;
const BACKOFF_CAP_MS: u64 = 1000;

pub(crate) async fn run<F, Fut, Out, Cancel, E>(db: Database, mut f: F) -> TrxResult<Out, Cancel, E>
where
    F: FnMut(Trx) -> Fut,
    Fut: std::future::Future<Output = Result<TrxControl<Out, Cancel>, E>>,
    E: From<anyhow::Error>,
{
    let mut attempt: u32 = 0;
    loop {
        let attempt_span = tracing::info_span!("trx_attempt", attempt = attempt);
        let result = run_attempt(&db, &mut f).instrument(attempt_span).await;
        match result {
            AttemptOutcome::Done(r) => return r,
            AttemptOutcome::Conflict(details) => {
                if attempt + 1 >= MAX_ATTEMPTS {
                    return TrxResult::Conflict(details);
                }
                let backoff_span = tracing::info_span!("trx_backoff", attempt = attempt);
                async {
                    let backoff = conflict_backoff(attempt).await;
                    crate::runtime::sleep(backoff).await;
                }
                .instrument(backoff_span)
                .await;
                attempt += 1;
            }
        }
    }
}

enum AttemptOutcome<Out, Cancel, E> {
    Done(TrxResult<Out, Cancel, E>),
    Conflict(ConflictDetails),
}

async fn run_attempt<F, Fut, Out, Cancel, E>(
    db: &Database,
    f: &mut F,
) -> AttemptOutcome<Out, Cancel, E>
where
    F: FnMut(Trx) -> Fut,
    Fut: std::future::Future<Output = Result<TrxControl<Out, Cancel>, E>>,
    E: From<anyhow::Error>,
{
    let state = Arc::new(Mutex::new(TrxState::new(db.clone())));
    let tx = Trx {
        inner: state.clone(),
    };

    let user_span = tracing::info_span!("trx_user_closure");
    let control = match f(tx).instrument(user_span).await {
        Ok(control) => control,
        Err(err) => return AttemptOutcome::Done(TrxResult::Err(err)),
    };

    match control.inner {
        TrxControlInner::Commit(out) => {
            let (commit_db, entries_result) = take_entries(state);
            let entries = match entries_result {
                Ok(e) => e,
                Err(err) => return AttemptOutcome::Done(TrxResult::Err(E::from(err))),
            };

            match commit_entries(commit_db, entries).await {
                Ok(()) => AttemptOutcome::Done(TrxResult::Committed(out)),
                Err(CommitFailure::Conflict(details)) => AttemptOutcome::Conflict(details),
                Err(CommitFailure::Err(err)) => AttemptOutcome::Done(TrxResult::Err(E::from(err))),
            }
        }
        TrxControlInner::Cancel(reason) => AttemptOutcome::Done(TrxResult::Cancelled(reason)),
    }
}

async fn conflict_backoff(attempt: u32) -> std::time::Duration {
    let ceiling = BACKOFF_BASE_MS
        .checked_shl(attempt)
        .unwrap_or(BACKOFF_CAP_MS)
        .min(BACKOFF_CAP_MS);
    let mut buf = [0u8; 8];
    crate::runtime::random_bytes(&mut buf).await;
    let raw = u64::from_le_bytes(buf);
    let delay_ms = raw % (ceiling + 1);
    std::time::Duration::from_millis(delay_ms)
}

struct TrxState {
    db: Database,
    entries: Vec<TrackedEntry>,
    index: HashMap<DocKey, usize>,
}

impl TrxState {
    fn new(db: Database) -> Self {
        Self {
            db,
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }

    fn register_loaded<T>(
        &mut self,
        key: DocKey,
        stored: Option<crate::StoredDoc>,
    ) -> Result<Option<DocHandle<T>>>
    where
        T: Document,
    {
        if self.index.contains_key(&key) {
            bail!("duplicate trx key access: {}/{}", key.pk, key.sk);
        }

        let idx = self.entries.len();
        self.index.insert(key.clone(), idx);

        match stored {
            Some(stored) => {
                let doc = serde_json::from_slice::<T>(&stored.data).map_err(|err| {
                    anyhow!(
                        "failed to deserialize {} at {}/{}: {}",
                        type_name::<T>(),
                        key.pk,
                        key.sk,
                        err
                    )
                })?;
                let (shared, handle) = new_shared_doc(doc);
                self.entries.push(TrackedEntry {
                    key,
                    expected_version: Some(stored.version),
                    state: TrackedState::Managed {
                        shared,
                        created: false,
                    },
                    observed: true,
                });
                Ok(Some(handle))
            }
            None => {
                self.entries.push(TrackedEntry {
                    key,
                    expected_version: None,
                    state: TrackedState::Missing,
                    observed: true,
                });
                Ok(None)
            }
        }
    }

    fn create<T>(&mut self, doc: T) -> Result<DocHandle<T>>
    where
        T: Document,
    {
        let key = doc.key();
        let (shared, handle) = new_shared_doc(doc);

        match self.index.get(&key).copied() {
            None => {
                let idx = self.entries.len();
                self.index.insert(key.clone(), idx);
                self.entries.push(TrackedEntry {
                    key,
                    expected_version: None,
                    state: TrackedState::Managed {
                        shared,
                        created: true,
                    },
                    observed: false,
                });
                Ok(handle)
            }
            Some(idx) => match self.entries.get_mut(idx) {
                Some(TrackedEntry {
                    expected_version: None,
                    state: TrackedState::Missing,
                    ..
                }) => {
                    self.entries[idx].state = TrackedState::Managed {
                        shared,
                        created: true,
                    };
                    self.entries[idx].observed = true;
                    Ok(handle)
                }
                _ => bail!("duplicate trx key access: {}/{}", key.pk, key.sk),
            },
        }
    }

    fn take_entries(&mut self) -> (Database, Result<Vec<TrackedEntry>>) {
        let db = self.db.clone();
        for entry in &self.entries {
            if let TrackedState::Managed { shared, .. } = &entry.state
                && shared.handle_alive.upgrade().is_some()
            {
                let err = anyhow!(
                    "live doc handle escaped trx for key {}/{}; commit outputs must not contain DocHandle values",
                    entry.key.pk,
                    entry.key.sk
                );
                self.entries.clear();
                return (db, Err(err));
            }
        }
        (db, Ok(std::mem::take(&mut self.entries)))
    }
}

struct TrackedEntry {
    key: DocKey,
    expected_version: Option<i64>,
    state: TrackedState,
    observed: bool,
}

impl TrackedEntry {
    fn conditional_op(&self) -> Result<Option<ConditionalOp>> {
        match &self.state {
            TrackedState::Missing => {
                if self.observed {
                    Ok(Some(ConditionalOp::Check {
                        pk: self.key.pk.clone(),
                        sk: self.key.sk.clone(),
                        expected_version: None,
                    }))
                } else {
                    Ok(None)
                }
            }
            TrackedState::Managed { shared, created } => {
                if shared.deleted.load(Ordering::Acquire) {
                    if *created {
                        if self.observed {
                            return Ok(Some(ConditionalOp::Check {
                                pk: self.key.pk.clone(),
                                sk: self.key.sk.clone(),
                                expected_version: None,
                            }));
                        }
                        return Ok(None);
                    }
                    let expected_version = self.expected_version.ok_or_else(|| {
                        anyhow!("existing tracked doc missing expected version for delete")
                    })?;
                    return Ok(Some(ConditionalOp::Delete {
                        pk: self.key.pk.clone(),
                        sk: self.key.sk.clone(),
                        expected_version,
                    }));
                }

                if *created {
                    return Ok(Some(ConditionalOp::Create {
                        pk: self.key.pk.clone(),
                        sk: self.key.sk.clone(),
                        data: (shared.serialize)()?,
                    }));
                }

                if shared.dirty.load(Ordering::Acquire) {
                    let expected_version = self.expected_version.ok_or_else(|| {
                        anyhow!("existing tracked doc missing expected version for update")
                    })?;
                    return Ok(Some(ConditionalOp::Put {
                        pk: self.key.pk.clone(),
                        sk: self.key.sk.clone(),
                        expected_version,
                        data: (shared.serialize)()?,
                    }));
                }

                if self.observed {
                    Ok(Some(ConditionalOp::Check {
                        pk: self.key.pk.clone(),
                        sk: self.key.sk.clone(),
                        expected_version: self.expected_version,
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

enum TrackedState {
    Missing,
    Managed { shared: SharedDoc, created: bool },
}

struct SharedDoc {
    dirty: Arc<AtomicBool>,
    deleted: Arc<AtomicBool>,
    handle_alive: Weak<()>,
    serialize: Box<dyn Fn() -> Result<Vec<u8>> + Send + Sync>,
}

fn new_shared_doc<T>(doc: T) -> (SharedDoc, DocHandle<T>)
where
    T: Document + Send + Sync,
{
    let data = Arc::new(SyncUnsafeCell::new(doc));
    let dirty = Arc::new(AtomicBool::new(false));
    let deleted = Arc::new(AtomicBool::new(false));
    let alive = Arc::new(());

    let serialize_data = data.clone();
    let shared = SharedDoc {
        dirty: dirty.clone(),
        deleted: deleted.clone(),
        handle_alive: Arc::downgrade(&alive),
        serialize: Box::new(move || {
            let doc_ref = unsafe { &*serialize_data.get() };
            serde_json::to_vec(doc_ref).map_err(Into::into)
        }),
    };

    let handle = DocHandle {
        data,
        dirty,
        deleted,
        _alive: alive,
        _marker: PhantomData,
    };

    (shared, handle)
}

fn take_entries(state: Arc<Mutex<TrxState>>) -> (Database, Result<Vec<TrackedEntry>>) {
    state.lock().unwrap().take_entries()
}

enum CommitFailure {
    Conflict(ConflictDetails),
    Err(anyhow::Error),
}

#[tracing::instrument(skip_all, fields(entries = entries.len()))]
async fn commit_entries(
    db: Database,
    entries: Vec<TrackedEntry>,
) -> std::result::Result<(), CommitFailure> {
    let mut operations = Vec::new();
    for entry in &entries {
        if let Some(operation) = entry.conditional_op().map_err(CommitFailure::Err)? {
            operations.push(operation);
        }
    }

    if operations.is_empty() {
        return Ok(());
    }

    match db
        .conditional_write_batch(&operations)
        .await
        .map_err(CommitFailure::Err)?
    {
        ConditionalOutcome::Applied => Ok(()),
        ConditionalOutcome::Conflict(details) => Err(CommitFailure::Conflict(details)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::turso_with_config;

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    struct TestDoc {
        id: String,
        value: i32,
    }

    impl Document for TestDoc {
        fn key(&self) -> DocKey {
            DocKey::new("TestDoc", format!("id={}", self.id))
        }
    }

    struct TestDocGet {
        id: String,
    }

    impl DocGet for TestDocGet {
        type Doc = TestDoc;

        fn key(&self) -> DocKey {
            DocKey::new("TestDoc", format!("id={}", self.id))
        }
    }

    fn test_state() -> TrxState {
        TrxState::new(turso_with_config(
            "http://127.0.0.1:0".to_string(),
            String::new(),
        ))
    }

    #[test]
    fn create_produces_insert_even_without_mutation() {
        let mut state = test_state();
        let _handle = state
            .create(TestDoc {
                id: "a".into(),
                value: 1,
            })
            .expect("create should succeed");

        let write = state.entries[0]
            .conditional_op()
            .expect("write plan")
            .expect("create operation");
        match write {
            ConditionalOp::Create { data, .. } => {
                let doc: TestDoc = serde_json::from_slice(&data).expect("deserialize insert");
                assert_eq!(doc.id, "a");
                assert_eq!(doc.value, 1);
            }
            _ => panic!("expected insert"),
        }
    }

    #[test]
    fn loaded_doc_marks_dirty_on_deref_mut() {
        let mut state = test_state();
        let doc = TestDoc {
            id: "a".into(),
            value: 1,
        };
        let key = TestDocGet { id: "a".into() }.key();
        let mut handle = state
            .register_loaded::<TestDoc>(
                key,
                Some(crate::StoredDoc {
                    data: serde_json::to_vec(&doc).expect("serialize").into(),
                    version: 7,
                }),
            )
            .expect("load should succeed")
            .expect("doc should exist");

        handle.value = 5;

        match state.entries[0]
            .conditional_op()
            .expect("write plan")
            .expect("update operation")
        {
            ConditionalOp::Put {
                expected_version,
                data,
                ..
            } => {
                assert_eq!(expected_version, 7);
                let doc: TestDoc = serde_json::from_slice(&data).expect("deserialize update");
                assert_eq!(doc.value, 5);
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn loaded_doc_delete_produces_delete_write() {
        let mut state = test_state();
        let doc = TestDoc {
            id: "a".into(),
            value: 1,
        };
        let key = TestDocGet { id: "a".into() }.key();
        let handle = state
            .register_loaded::<TestDoc>(
                key,
                Some(crate::StoredDoc {
                    data: serde_json::to_vec(&doc).expect("serialize").into(),
                    version: 7,
                }),
            )
            .expect("load should succeed")
            .expect("doc should exist");

        handle.delete();

        match state.entries[0]
            .conditional_op()
            .expect("write plan")
            .expect("delete operation")
        {
            ConditionalOp::Delete {
                expected_version, ..
            } => assert_eq!(expected_version, 7),
            _ => panic!("expected delete"),
        }
    }

    #[test]
    fn missing_read_can_be_promoted_to_create() {
        let mut state = test_state();
        let key = TestDocGet { id: "a".into() }.key();
        let loaded = state
            .register_loaded::<TestDoc>(key, None)
            .expect("register missing should succeed");
        assert!(loaded.is_none());

        let handle = state
            .create(TestDoc {
                id: "a".into(),
                value: 3,
            })
            .expect("create after missing get should succeed");
        assert_eq!(handle.value, 3);

        assert!(matches!(
            state.entries[0]
                .conditional_op()
                .expect("write plan")
                .expect("create operation"),
            ConditionalOp::Create { .. }
        ));
    }

    #[test]
    fn duplicate_key_access_is_rejected() {
        let mut state = test_state();
        let first = state.register_loaded::<TestDoc>(TestDocGet { id: "a".into() }.key(), None);
        assert!(first.is_ok());

        let second = state.register_loaded::<TestDoc>(TestDocGet { id: "a".into() }.key(), None);
        assert!(second.is_err());
    }

    #[test]
    fn live_handle_cannot_escape_commit_boundary() {
        let mut state = test_state();
        let _handle = state
            .create(TestDoc {
                id: "a".into(),
                value: 1,
            })
            .expect("create should succeed");

        let (_, result) = state.take_entries();
        match result {
            Ok(_) => panic!("live handle should fail"),
            Err(err) => assert!(err.to_string().contains("live doc handle escaped trx")),
        }
    }
}
