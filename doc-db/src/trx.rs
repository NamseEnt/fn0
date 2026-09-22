use crate::{
    Database, DocDbRevision, ObservedDocument, TransactCondition, TransactMutation, TransactRequest,
};
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
        results: &mut std::vec::IntoIter<ObservedDocument>,
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
        results: &mut std::vec::IntoIter<ObservedDocument>,
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
                results: &mut std::vec::IntoIter<ObservedDocument>,
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

    async fn batch_load(&self, keys: &[(String, String)]) -> Result<Vec<ObservedDocument>> {
        let db = self.inner.lock().unwrap().db.clone();
        db.batch_get_observed(keys).await
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

#[derive(Clone, Debug, Default)]
pub struct ConflictDetails {
    pub keys: Vec<ConflictKey>,
}

#[derive(Clone, Debug)]
pub struct ConflictKey {
    pub key: DocKey,
    pub expected_revision: Option<DocDbRevision>,
    pub actual_revision: Option<DocDbRevision>,
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
        observed: ObservedDocument,
    ) -> Result<Option<DocHandle<T>>>
    where
        T: Document,
    {
        if self.index.contains_key(&key) {
            bail!("duplicate trx key access: {}/{}", key.pk, key.sk);
        }

        let idx = self.entries.len();
        self.index.insert(key.clone(), idx);

        match observed {
            ObservedDocument::Present { data, revision } => {
                let doc = serde_json::from_slice::<T>(&data).map_err(|err| {
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
                    expected_revision: Some(revision),
                    observed: true,
                    state: TrackedState::Managed {
                        shared,
                        created: false,
                    },
                });
                Ok(Some(handle))
            }
            ObservedDocument::Missing { revision } => {
                self.entries.push(TrackedEntry {
                    key,
                    expected_revision: revision,
                    observed: true,
                    state: TrackedState::Missing,
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
                    expected_revision: None,
                    observed: false,
                    state: TrackedState::Managed {
                        shared,
                        created: true,
                    },
                });
                Ok(handle)
            }
            Some(idx) => match self.entries.get_mut(idx) {
                Some(TrackedEntry {
                    expected_revision: _,
                    state: TrackedState::Missing,
                    ..
                }) => {
                    self.entries[idx].state = TrackedState::Managed {
                        shared,
                        created: true,
                    };
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
    expected_revision: Option<DocDbRevision>,
    observed: bool,
    state: TrackedState,
}

impl TrackedEntry {
    fn condition(&self) -> Option<TransactCondition> {
        if !self.observed {
            let is_live_created = matches!(
                &self.state,
                TrackedState::Managed {
                    shared,
                    created: true,
                } if !shared.deleted.load(Ordering::Acquire)
            );
            if !is_live_created {
                return None;
            }
        }
        match self.expected_revision {
            Some(expected_revision) => Some(TransactCondition::RevisionEquals {
                pk: self.key.pk.clone(),
                sk: self.key.sk.clone(),
                expected_revision,
            }),
            None => Some(TransactCondition::NotExists {
                pk: self.key.pk.clone(),
                sk: self.key.sk.clone(),
            }),
        }
    }

    fn mutation(&self) -> Result<Option<TransactMutation>> {
        let TrackedState::Managed { shared, created } = &self.state else {
            return Ok(None);
        };
        if shared.deleted.load(Ordering::Acquire) {
            if *created {
                return Ok(None);
            }
            return Ok(Some(TransactMutation::Delete {
                pk: self.key.pk.clone(),
                sk: self.key.sk.clone(),
            }));
        }
        if *created || shared.dirty.load(Ordering::Acquire) {
            return Ok(Some(TransactMutation::Put {
                pk: self.key.pk.clone(),
                sk: self.key.sk.clone(),
                data: (shared.serialize)()?,
            }));
        }
        Ok(None)
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
    let mut conditions = Vec::new();
    let mut mutations = Vec::new();
    for entry in &entries {
        if let Some(condition) = entry.condition() {
            conditions.push(condition);
        }
        if let Some(mutation) = entry.mutation().map_err(CommitFailure::Err)? {
            mutations.push(mutation);
        }
    }

    if conditions.is_empty() && mutations.is_empty() {
        return Ok(());
    }

    let request = TransactRequest {
        conditions,
        mutations,
    };
    let outcome = db.transact(&request).await.map_err(CommitFailure::Err)?;

    let mut conflicts = Vec::new();

    if let Some(info) = outcome.conflict {
        let condition = request.conditions.get(info.condition_index).ok_or_else(|| {
            CommitFailure::Err(anyhow!(
                "backend returned invalid transaction conflict condition_index {} for {} conditions",
                info.condition_index,
                request.conditions.len()
            ))
        })?;
        let (pk, sk, expected_revision) = condition_key_and_expected(condition);
        conflicts.push(ConflictKey {
            key: DocKey { pk, sk },
            expected_revision,
            actual_revision: None,
        });
    }

    if !conflicts.is_empty() {
        let key_pairs: Vec<(String, String)> = conflicts
            .iter()
            .map(|c| (c.key.pk.clone(), c.key.sk.clone()))
            .collect();
        if let Ok(observed) = db.batch_get_observed(&key_pairs).await {
            for (conflict, observation) in conflicts.iter_mut().zip(observed.into_iter()) {
                conflict.actual_revision = match observation {
                    ObservedDocument::Present { revision, .. } => Some(revision),
                    ObservedDocument::Missing { revision } => revision,
                };
            }
        }
        return Err(CommitFailure::Conflict(ConflictDetails { keys: conflicts }));
    }

    Ok(())
}

fn condition_key_and_expected(
    condition: &TransactCondition,
) -> (String, String, Option<DocDbRevision>) {
    match condition {
        TransactCondition::Exists { pk, sk } | TransactCondition::NotExists { pk, sk } => {
            (pk.clone(), sk.clone(), None)
        }
        TransactCondition::RevisionEquals {
            pk,
            sk,
            expected_revision,
        } => (pk.clone(), sk.clone(), Some(*expected_revision)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        TrxState::new(crate::memory())
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
            .mutation()
            .expect("transaction mutation")
            .expect("transaction mutation should exist");
        match write {
            TransactMutation::Put { data, .. } => {
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
                ObservedDocument::Present {
                    data: serde_json::to_vec(&doc).expect("serialize").into(),
                    revision: DocDbRevision::new(7),
                },
            )
            .expect("load should succeed")
            .expect("doc should exist");

        handle.value = 5;

        match state.entries[0]
            .mutation()
            .expect("transaction mutation")
            .expect("transaction mutation should exist")
        {
            TransactMutation::Put { data, .. } => {
                let doc: TestDoc = serde_json::from_slice(&data).expect("deserialize update");
                assert_eq!(doc.value, 5);
            }
            _ => panic!("expected update"),
        }
        assert!(matches!(
            state.entries[0].condition(),
            Some(TransactCondition::RevisionEquals {
                expected_revision,
                ..
            }) if expected_revision == DocDbRevision::new(7)
        ));
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
                ObservedDocument::Present {
                    data: serde_json::to_vec(&doc).expect("serialize").into(),
                    revision: DocDbRevision::new(7),
                },
            )
            .expect("load should succeed")
            .expect("doc should exist");

        handle.delete();

        match state.entries[0]
            .mutation()
            .expect("transaction mutation")
            .expect("transaction mutation should exist")
        {
            TransactMutation::Delete { .. } => {}
            _ => panic!("expected delete"),
        }
        assert!(matches!(
            state.entries[0].condition(),
            Some(TransactCondition::RevisionEquals {
                expected_revision,
                ..
            }) if expected_revision == DocDbRevision::new(7)
        ));
    }

    #[test]
    fn missing_read_can_be_promoted_to_create() {
        let mut state = test_state();
        let key = TestDocGet { id: "a".into() }.key();
        let loaded = state
            .register_loaded::<TestDoc>(key, ObservedDocument::Missing { revision: None })
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
            state.entries[0].mutation().expect("transaction mutation"),
            Some(TransactMutation::Put { .. })
        ));
        assert!(matches!(
            state.entries[0].condition(),
            Some(TransactCondition::NotExists { .. })
        ));
    }

    #[test]
    fn exact_missing_revision_becomes_revision_condition() {
        let mut state = test_state();
        let key = TestDocGet { id: "a".into() }.key();
        assert!(
            state
                .register_loaded::<TestDoc>(
                    key,
                    ObservedDocument::Missing {
                        revision: Some(DocDbRevision::new(11)),
                    },
                )
                .expect("register missing should succeed")
                .is_none()
        );

        assert!(matches!(
            state.entries[0].condition(),
            Some(TransactCondition::RevisionEquals {
                expected_revision,
                ..
            }) if expected_revision == DocDbRevision::new(11)
        ));
        assert!(state.entries[0].mutation().unwrap().is_none());
    }

    #[test]
    fn missing_read_then_create_then_delete_keeps_missing_dependency() {
        let mut state = test_state();
        let key = TestDocGet { id: "a".into() }.key();
        assert!(
            state
                .register_loaded::<TestDoc>(key, ObservedDocument::Missing { revision: None })
                .expect("register missing should succeed")
                .is_none()
        );

        let handle = state
            .create(TestDoc {
                id: "a".into(),
                value: 3,
            })
            .expect("create after missing get should succeed");
        handle.delete();
        drop(handle);

        assert!(matches!(
            state.entries[0].condition(),
            Some(TransactCondition::NotExists { .. })
        ));
        assert!(
            state.entries[0]
                .mutation()
                .expect("transaction mutation")
                .is_none()
        );
    }

    #[test]
    fn create_then_delete_without_read_is_a_noop() {
        let mut state = test_state();
        let handle = state
            .create(TestDoc {
                id: "a".into(),
                value: 3,
            })
            .expect("create should succeed");
        handle.delete();
        drop(handle);

        assert!(
            state.entries[0].condition().is_none()
                && state.entries[0]
                    .mutation()
                    .expect("transaction mutation")
                    .is_none()
        );
    }

    #[test]
    fn loaded_doc_without_mutation_produces_version_check() {
        let mut state = test_state();
        let key = TestDocGet { id: "a".into() }.key();
        let handle = state
            .register_loaded::<TestDoc>(
                key,
                ObservedDocument::Present {
                    data: serde_json::to_vec(&TestDoc {
                        id: "a".into(),
                        value: 1,
                    })
                    .expect("serialize")
                    .into(),
                    revision: DocDbRevision::new(7),
                },
            )
            .expect("load should succeed")
            .expect("doc should exist");
        drop(handle);

        assert!(matches!(
            state.entries[0].condition(),
            Some(TransactCondition::RevisionEquals {
                expected_revision,
                ..
            }) if expected_revision == DocDbRevision::new(7)
        ));
    }

    #[test]
    fn duplicate_key_access_is_rejected() {
        let mut state = test_state();
        let first = state.register_loaded::<TestDoc>(
            TestDocGet { id: "a".into() }.key(),
            ObservedDocument::Missing { revision: None },
        );
        assert!(first.is_ok());

        let second = state.register_loaded::<TestDoc>(
            TestDocGet { id: "a".into() }.key(),
            ObservedDocument::Missing { revision: None },
        );
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
