use crate::{Database, Transaction, Value};
use anyhow::{Result, anyhow, bail};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    any::type_name,
    cell::{Cell, RefCell, UnsafeCell},
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

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

pub trait Document: Serialize + DeserializeOwned + 'static {
    fn key(&self) -> DocKey;
}

pub trait DocGet {
    type Doc: Document;
    fn key(&self) -> DocKey;
}

#[allow(async_fn_in_trait)]
pub trait TrxRead: Sized {
    type Output;
    async fn read(self, tx: &Trx) -> Result<Self::Output>;
}

impl<R> TrxRead for R
where
    R: DocGet,
{
    type Output = Option<DocHandle<R::Doc>>;

    async fn read(self, tx: &Trx) -> Result<Self::Output> {
        tx.get_one(self).await
    }
}

macro_rules! impl_trx_read_tuple {
    ($($T:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($T: TrxRead),+> TrxRead for ($($T,)+) {
            type Output = ($($T::Output,)+);

            async fn read(self, tx: &Trx) -> Result<Self::Output> {
                let ($($T,)+) = self;
                Ok(($($T.read(tx).await?,)+))
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
    data: Rc<UnsafeCell<T>>,
    dirty: Rc<Cell<bool>>,
    deleted: Rc<Cell<bool>>,
    _alive: Rc<()>,
    _marker: PhantomData<Rc<T>>,
}

impl<T> DocHandle<T> {
    pub fn delete(&self) {
        self.deleted.set(true);
    }
}

impl<T> Deref for DocHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // A handle is unique per key within a trx and is not cloneable.
        unsafe { &*self.data.get() }
    }
}

impl<T> DerefMut for DocHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty.set(true);
        // Dirty tracking happens at first mutable access; the doc lives until trx end.
        unsafe { &mut *self.data.get() }
    }
}

pub struct Trx {
    inner: Rc<RefCell<TrxState>>,
}

impl Trx {
    pub async fn get<R>(&self, request: R) -> Result<R::Output>
    where
        R: TrxRead,
    {
        request.read(self).await
    }

    pub fn create<T>(&self, doc: T) -> Result<DocHandle<T>>
    where
        T: Document,
    {
        self.inner.borrow_mut().create(doc)
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

    async fn get_one<R>(&self, request: R) -> Result<Option<DocHandle<R::Doc>>>
    where
        R: DocGet,
    {
        let key = request.key();

        {
            let state = self.inner.borrow();
            if state.index.contains_key(&key) {
                bail!("duplicate trx key access: {}/{}", key.pk, key.sk);
            }
        }

        let db = self.inner.borrow().db.clone();
        let stored = db.get_with_version(&key.pk, &key.sk).await?;
        self.inner
            .borrow_mut()
            .register_loaded::<R::Doc>(key, stored)
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
    pub expected_version: Option<i64>,
    pub actual_version: Option<i64>,
}

const MAX_ATTEMPTS: u32 = 5;
const BACKOFF_BASE_MS: u64 = 50;
const BACKOFF_CAP_MS: u64 = 1000;

pub(crate) async fn run<F, Fut, Out, Cancel, E>(
    db: Database,
    mut f: F,
) -> TrxResult<Out, Cancel, E>
where
    F: FnMut(Trx) -> Fut,
    Fut: std::future::Future<Output = Result<TrxControl<Out, Cancel>, E>>,
    E: From<anyhow::Error>,
{
    let mut attempt: u32 = 0;
    loop {
        let state = Rc::new(RefCell::new(TrxState::new(db.clone())));
        let tx = Trx {
            inner: state.clone(),
        };

        let control = match f(tx).await {
            Ok(control) => control,
            Err(err) => return TrxResult::Err(err),
        };

        match control.inner {
            TrxControlInner::Commit(out) => {
                let (commit_db, entries) = match take_entries(state) {
                    Ok(value) => value,
                    Err(err) => return TrxResult::Err(E::from(err)),
                };

                match commit_entries(commit_db, entries).await {
                    Ok(()) => return TrxResult::Committed(out),
                    Err(CommitFailure::Conflict(details)) => {
                        if attempt + 1 >= MAX_ATTEMPTS {
                            return TrxResult::Conflict(details);
                        }
                        let backoff = conflict_backoff(attempt).await;
                        forte_sdk::time_wasi::sleep(backoff).await;
                        attempt += 1;
                    }
                    Err(CommitFailure::Err(err)) => return TrxResult::Err(E::from(err)),
                }
            }
            TrxControlInner::Cancel(reason) => return TrxResult::Cancelled(reason),
        }
    }
}

async fn conflict_backoff(attempt: u32) -> forte_sdk::time_wasi::Duration {
    let ceiling = BACKOFF_BASE_MS
        .checked_shl(attempt)
        .unwrap_or(BACKOFF_CAP_MS)
        .min(BACKOFF_CAP_MS);
    let mut buf = [0u8; 8];
    forte_sdk::rand::get_insecure_random_bytes(&mut buf).await;
    let raw = u64::from_le_bytes(buf);
    let delay_ms = raw % (ceiling + 1);
    forte_sdk::time_wasi::Duration::from_millis(delay_ms)
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
        stored: Option<crate::turso::StoredDoc>,
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
                });
                Ok(Some(handle))
            }
            None => {
                self.entries.push(TrackedEntry {
                    key,
                    expected_version: None,
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
                    expected_version: None,
                    state: TrackedState::Managed {
                        shared,
                        created: true,
                    },
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
                    Ok(handle)
                }
                _ => bail!("duplicate trx key access: {}/{}", key.pk, key.sk),
            },
        }
    }

    fn take_entries(&mut self) -> Result<(Database, Vec<TrackedEntry>)> {
        for entry in &self.entries {
            if let TrackedState::Managed { shared, .. } = &entry.state
                && shared.handle_alive.upgrade().is_some()
            {
                bail!(
                    "live doc handle escaped trx for key {}/{}; commit outputs must not contain DocHandle values",
                    entry.key.pk,
                    entry.key.sk
                );
            }
        }

        Ok((self.db.clone(), std::mem::take(&mut self.entries)))
    }
}

struct TrackedEntry {
    key: DocKey,
    expected_version: Option<i64>,
    state: TrackedState,
}

impl TrackedEntry {
    fn write(&self) -> Result<PendingWrite> {
        match &self.state {
            TrackedState::Missing => Ok(PendingWrite::None),
            TrackedState::Managed { shared, created } => {
                if shared.deleted.get() {
                    if *created {
                        return Ok(PendingWrite::None);
                    }
                    let expected_version = self.expected_version.ok_or_else(|| {
                        anyhow!("existing tracked doc missing expected version for delete")
                    })?;
                    return Ok(PendingWrite::Delete(expected_version));
                }

                if *created {
                    return Ok(PendingWrite::Insert((shared.serialize)()?));
                }

                if shared.dirty.get() {
                    let expected_version = self.expected_version.ok_or_else(|| {
                        anyhow!("existing tracked doc missing expected version for update")
                    })?;
                    return Ok(PendingWrite::Update {
                        expected_version,
                        data: (shared.serialize)()?,
                    });
                }

                Ok(PendingWrite::None)
            }
        }
    }
}

enum TrackedState {
    Missing,
    Managed { shared: SharedDoc, created: bool },
}

struct SharedDoc {
    dirty: Rc<Cell<bool>>,
    deleted: Rc<Cell<bool>>,
    handle_alive: Weak<()>,
    serialize: Box<dyn Fn() -> Result<Vec<u8>>>,
}

fn new_shared_doc<T>(doc: T) -> (SharedDoc, DocHandle<T>)
where
    T: Document,
{
    let data = Rc::new(UnsafeCell::new(doc));
    let dirty = Rc::new(Cell::new(false));
    let deleted = Rc::new(Cell::new(false));
    let alive = Rc::new(());

    let serialize_data = data.clone();
    let shared = SharedDoc {
        dirty: dirty.clone(),
        deleted: deleted.clone(),
        handle_alive: Rc::downgrade(&alive),
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

fn take_entries(state: Rc<RefCell<TrxState>>) -> Result<(Database, Vec<TrackedEntry>)> {
    state.borrow_mut().take_entries()
}

enum PendingWrite {
    None,
    Insert(Vec<u8>),
    Update {
        expected_version: i64,
        data: Vec<u8>,
    },
    Delete(i64),
}

enum CommitFailure {
    Conflict(ConflictDetails),
    Err(anyhow::Error),
}

async fn commit_entries(
    db: Database,
    entries: Vec<TrackedEntry>,
) -> std::result::Result<(), CommitFailure> {
    let mut tx = db.transaction().await.map_err(CommitFailure::Err)?;

    let mut conflicts = Vec::new();
    for entry in &entries {
        let current = tx
            .get_with_version(&entry.key.pk, &entry.key.sk)
            .await
            .map_err(CommitFailure::Err)?;
        let actual_version = current.as_ref().map(|doc| doc.version);
        if actual_version != entry.expected_version {
            conflicts.push(ConflictKey {
                key: entry.key.clone(),
                expected_version: entry.expected_version,
                actual_version,
            });
        }
    }

    if !conflicts.is_empty() {
        rollback_silent(tx).await;
        return Err(CommitFailure::Conflict(ConflictDetails { keys: conflicts }));
    }

    for entry in &entries {
        match entry.write().map_err(CommitFailure::Err)? {
            PendingWrite::None => {}
            PendingWrite::Insert(data) => {
                let result = tx
                    .execute_stmt(
                        "INSERT INTO docs (pk, sk, data, version) SELECT ?, ?, ?, 0 WHERE NOT EXISTS (SELECT 1 FROM docs WHERE pk = ? AND sk = ?)",
                        vec![
                            Value::Text {
                                value: entry.key.pk.clone().into(),
                            },
                            Value::Text {
                                value: entry.key.sk.clone().into(),
                            },
                            Value::Blob { value: data.into() },
                            Value::Text {
                                value: entry.key.pk.clone().into(),
                            },
                            Value::Text {
                                value: entry.key.sk.clone().into(),
                            },
                        ],
                        false,
                    )
                    .await
                    .map_err(CommitFailure::Err)?;

                if result.affected_row_count != 1 {
                    let details = refresh_conflict(&db, &entry.key, entry.expected_version).await;
                    rollback_silent(tx).await;
                    return Err(CommitFailure::Conflict(details));
                }
            }
            PendingWrite::Update {
                expected_version,
                data,
            } => {
                let result = tx
                    .execute_stmt(
                        "UPDATE docs SET data = ?, version = version + 1 WHERE pk = ? AND sk = ? AND version = ?",
                        vec![
                            Value::Blob { value: data.into() },
                            Value::Text {
                                value: entry.key.pk.clone().into(),
                            },
                            Value::Text {
                                value: entry.key.sk.clone().into(),
                            },
                            Value::Integer {
                                value: expected_version,
                            },
                        ],
                        false,
                    )
                    .await
                    .map_err(CommitFailure::Err)?;

                if result.affected_row_count != 1 {
                    let details = refresh_conflict(&db, &entry.key, entry.expected_version).await;
                    rollback_silent(tx).await;
                    return Err(CommitFailure::Conflict(details));
                }
            }
            PendingWrite::Delete(expected_version) => {
                let result = tx
                    .execute_stmt(
                        "DELETE FROM docs WHERE pk = ? AND sk = ? AND version = ?",
                        vec![
                            Value::Text {
                                value: entry.key.pk.clone().into(),
                            },
                            Value::Text {
                                value: entry.key.sk.clone().into(),
                            },
                            Value::Integer {
                                value: expected_version,
                            },
                        ],
                        false,
                    )
                    .await
                    .map_err(CommitFailure::Err)?;

                if result.affected_row_count != 1 {
                    let details = refresh_conflict(&db, &entry.key, entry.expected_version).await;
                    rollback_silent(tx).await;
                    return Err(CommitFailure::Conflict(details));
                }
            }
        }
    }

    tx.commit().await.map_err(CommitFailure::Err)
}

async fn refresh_conflict(
    db: &Database,
    key: &DocKey,
    expected_version: Option<i64>,
) -> ConflictDetails {
    let actual_version = match db.get_with_version(&key.pk, &key.sk).await {
        Ok(Some(doc)) => Some(doc.version),
        Ok(None) => None,
        Err(_) => None,
    };

    ConflictDetails {
        keys: vec![ConflictKey {
            key: key.clone(),
            expected_version,
            actual_version,
        }],
    }
}

async fn rollback_silent(tx: Transaction<'_>) {
    let _ = tx.rollback().await;
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

        let write = state.entries[0].write().expect("write plan");
        match write {
            PendingWrite::Insert(data) => {
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
                Some(crate::turso::StoredDoc {
                    data: serde_json::to_vec(&doc).expect("serialize").into(),
                    version: 7,
                }),
            )
            .expect("load should succeed")
            .expect("doc should exist");

        handle.value = 5;

        match state.entries[0].write().expect("write plan") {
            PendingWrite::Update {
                expected_version,
                data,
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
                Some(crate::turso::StoredDoc {
                    data: serde_json::to_vec(&doc).expect("serialize").into(),
                    version: 7,
                }),
            )
            .expect("load should succeed")
            .expect("doc should exist");

        handle.delete();

        match state.entries[0].write().expect("write plan") {
            PendingWrite::Delete(expected_version) => assert_eq!(expected_version, 7),
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
            state.entries[0].write().expect("write plan"),
            PendingWrite::Insert(_)
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

        match state.take_entries() {
            Ok(_) => panic!("live handle should fail"),
            Err(err) => assert!(err.to_string().contains("live doc handle escaped trx")),
        }
    }
}
