use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DB, Direction, IteratorMode, Options, WriteBatch,
    WriteOptions,
};
use uuid::Uuid;

use crate::{
    CommitMutation, CommitRecord, DibiError, Result, commit_log::COMMIT_RECORD_FORMAT_VERSION,
    decode_document_key, decode_document_value, encode_document_key, encode_document_value,
};

const DATABASE_FORMAT_VERSION: u32 = 1;
const DOCS_CF: &str = "docs";
const META_CF: &str = "meta";
const BACKUP_OUTBOX_CF: &str = "backup_outbox";
const FORMAT_VERSION_KEY: &[u8] = b"format_version";
const DB_UUID_KEY: &[u8] = b"db_uuid";
const LAST_COMMIT_ID_KEY: &[u8] = b"last_commit_id";

#[derive(Clone, Debug)]
pub struct DibiConfig {
    pub data_dir: PathBuf,
}

#[derive(Clone)]
pub struct DibiEngine {
    inner: Arc<EngineInner>,
}

struct EngineInner {
    db: DB,
    write_gate: Mutex<()>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredDocument {
    pub data: Bytes,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub pk: String,
    pub sk: String,
    pub data: Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminDocument {
    pub pk: String,
    pub sk: String,
    pub data: Bytes,
    pub version: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdminScanRequest {
    pub after: Option<(String, String)>,
    pub limit: usize,
    pub pk_prefix: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminScanPage {
    pub documents: Vec<AdminDocument>,
    pub next: Option<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicationWrite {
    Put { pk: String, sk: String, data: Bytes },
    Delete { pk: String, sk: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionalWrite {
    Create {
        pk: String,
        sk: String,
        data: Bytes,
    },
    Put {
        pk: String,
        sk: String,
        expected_version: i64,
        data: Bytes,
    },
    Delete {
        pk: String,
        sk: String,
        expected_version: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub pk: String,
    pub sk: String,
    pub expected_version: Option<i64>,
    pub actual_version: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionalWriteOutcome {
    Applied(WriteResult),
    Conflict(Vec<Conflict>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteResult {
    pub commit_id: Option<u64>,
}

impl DibiEngine {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_config(DibiConfig {
            data_dir: path.as_ref().to_path_buf(),
        })
    }

    pub fn open_config(config: DibiConfig) -> Result<Self> {
        let is_new = !config.data_dir.join("CURRENT").exists();
        if !is_new {
            validate_column_families(&config.data_dir)?;
        }

        let mut database_options = Options::default();
        database_options.create_if_missing(is_new);
        database_options.create_missing_column_families(is_new);
        let descriptors = [DOCS_CF, META_CF, BACKUP_OUTBOX_CF]
            .into_iter()
            .map(|name| ColumnFamilyDescriptor::new(name, Options::default()));
        let db = DB::open_cf_descriptors(&database_options, &config.data_dir, descriptors)?;
        let engine = Self {
            inner: Arc::new(EngineInner {
                db,
                write_gate: Mutex::new(()),
            }),
        };
        engine.initialize_or_validate_metadata(is_new)?;
        Ok(engine)
    }

    pub fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        Ok(self.get_with_version(pk, sk)?.map(|document| document.data))
    }

    pub fn get_with_version(&self, pk: &str, sk: &str) -> Result<Option<StoredDocument>> {
        let key = encode_document_key(pk, sk);
        self.read_document(&key)
    }

    pub fn put(&self, pk: &str, sk: &str, data: Bytes) -> Result<u64> {
        let _guard = self.lock_writes()?;
        let key = encode_document_key(pk, sk);
        let version = match self.read_document(&key)? {
            Some(document) => increment_version(document.version, pk, sk)?,
            None => 0,
        };
        let value = encode_document_value(&StoredDocument { data, version });
        let mutations = BTreeMap::from([(key, Some(value))]);
        self.commit_mutations(mutations)
    }

    pub fn delete(&self, pk: &str, sk: &str) -> Result<u64> {
        let _guard = self.lock_writes()?;
        let mutations = BTreeMap::from([(encode_document_key(pk, sk), None)]);
        self.commit_mutations(mutations)
    }

    pub fn query(&self, pk: &str, after_sk: Option<&str>, limit: usize) -> Result<Vec<Document>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let prefix = encode_pk_prefix(pk);
        let start = after_sk
            .map(|sk| encode_document_key(pk, sk))
            .unwrap_or_else(|| prefix.clone());
        let snapshot = self.inner.db.snapshot();
        let docs_cf = self.cf(DOCS_CF)?;
        let mut documents = Vec::with_capacity(limit);
        for item in snapshot.iterator_cf(docs_cf, IteratorMode::From(&start, Direction::Forward)) {
            let (encoded_key, encoded_value) = item?;
            if !encoded_key.starts_with(&prefix) {
                break;
            }
            if after_sk.is_some() && encoded_key.as_ref() == start.as_slice() {
                continue;
            }
            let (found_pk, sk) = decode_document_key(&encoded_key)?;
            if found_pk != pk {
                break;
            }
            let stored = decode_document_value(&encoded_value)?;
            documents.push(Document {
                pk: found_pk,
                sk,
                data: stored.data,
            });
            if documents.len() == limit {
                break;
            }
        }
        Ok(documents)
    }

    pub fn scan(&self, after: Option<(&str, &str)>, limit: usize) -> Result<Vec<Document>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let start = after.map(|(pk, sk)| encode_document_key(pk, sk));
        let mode = start
            .as_deref()
            .map(|key| IteratorMode::From(key, Direction::Forward))
            .unwrap_or(IteratorMode::Start);
        let snapshot = self.inner.db.snapshot();
        let docs_cf = self.cf(DOCS_CF)?;
        let mut documents = Vec::with_capacity(limit);
        for item in snapshot.iterator_cf(docs_cf, mode) {
            let (encoded_key, encoded_value) = item?;
            if start
                .as_deref()
                .is_some_and(|cursor| encoded_key.as_ref() == cursor)
            {
                continue;
            }
            let (pk, sk) = decode_document_key(&encoded_key)?;
            let stored = decode_document_value(&encoded_value)?;
            documents.push(Document {
                pk,
                sk,
                data: stored.data,
            });
            if documents.len() == limit {
                break;
            }
        }
        Ok(documents)
    }

    pub fn admin_scan(&self, request: AdminScanRequest) -> Result<AdminScanPage> {
        if request.limit == 0 {
            return Ok(AdminScanPage {
                documents: Vec::new(),
                next: None,
            });
        }
        let start = request
            .after
            .as_ref()
            .map(|(pk, sk)| encode_document_key(pk, sk));
        let mode = start
            .as_deref()
            .map(|key| IteratorMode::From(key, Direction::Forward))
            .unwrap_or(IteratorMode::Start);
        let snapshot = self.inner.db.snapshot();
        let docs_cf = self.cf(DOCS_CF)?;
        let mut documents = Vec::with_capacity(request.limit);
        for item in snapshot.iterator_cf(docs_cf, mode) {
            let (encoded_key, encoded_value) = item?;
            if start
                .as_deref()
                .is_some_and(|cursor| encoded_key.as_ref() == cursor)
            {
                continue;
            }
            let (pk, sk) = decode_document_key(&encoded_key)?;
            if request
                .pk_prefix
                .as_ref()
                .is_some_and(|prefix| !pk.starts_with(prefix))
            {
                continue;
            }
            let stored = decode_document_value(&encoded_value)?;
            documents.push(AdminDocument {
                pk,
                sk,
                data: stored.data,
                version: stored.version,
            });
            if documents.len() == request.limit {
                break;
            }
        }
        let next = (documents.len() == request.limit)
            .then(|| {
                documents
                    .last()
                    .map(|document| (document.pk.clone(), document.sk.clone()))
            })
            .flatten();
        Ok(AdminScanPage { documents, next })
    }

    pub fn application_write_batch(&self, operations: &[ApplicationWrite]) -> Result<WriteResult> {
        if operations.is_empty() {
            return Ok(WriteResult { commit_id: None });
        }
        let _guard = self.lock_writes()?;
        let mut states = BTreeMap::new();
        for operation in operations {
            let (pk, sk) = application_key(operation);
            let key = encode_document_key(pk, sk);
            if !states.contains_key(&key) {
                states.insert(key.clone(), self.read_encoded_document(&key)?);
            }
            match operation {
                ApplicationWrite::Put { data, .. } => {
                    let version = match states.get(&key).and_then(Option::as_deref) {
                        Some(value) => {
                            let document = decode_document_value(value)?;
                            increment_version(document.version, pk, sk)?
                        }
                        None => 0,
                    };
                    states.insert(
                        key,
                        Some(encode_document_value(&StoredDocument {
                            data: data.clone(),
                            version,
                        })),
                    );
                }
                ApplicationWrite::Delete { .. } => {
                    states.insert(key, None);
                }
            }
        }
        let commit_id = self.commit_mutations(states)?;
        Ok(WriteResult {
            commit_id: Some(commit_id),
        })
    }

    pub fn conditional_write_batch(
        &self,
        operations: &[ConditionalWrite],
    ) -> Result<ConditionalWriteOutcome> {
        validate_conditional_uniqueness(operations)?;
        if operations.is_empty() {
            return Ok(ConditionalWriteOutcome::Applied(WriteResult {
                commit_id: None,
            }));
        }
        let _guard = self.lock_writes()?;
        let mut current = Vec::with_capacity(operations.len());
        let mut conflicts = Vec::new();
        for operation in operations {
            let (pk, sk, expected_version) = conditional_key(operation);
            let key = encode_document_key(pk, sk);
            let value = self.read_encoded_document(&key)?;
            let actual_version = value
                .as_deref()
                .map(decode_document_value)
                .transpose()?
                .map(|document| document.version);
            if actual_version != expected_version {
                conflicts.push(Conflict {
                    pk: pk.to_owned(),
                    sk: sk.to_owned(),
                    expected_version,
                    actual_version,
                });
            }
            current.push((key, value));
        }
        if !conflicts.is_empty() {
            return Ok(ConditionalWriteOutcome::Conflict(conflicts));
        }

        let mut mutations = BTreeMap::new();
        for (operation, (key, value)) in operations.iter().zip(current) {
            match operation {
                ConditionalWrite::Create { data, .. } => {
                    mutations.insert(
                        key,
                        Some(encode_document_value(&StoredDocument {
                            data: data.clone(),
                            version: 0,
                        })),
                    );
                }
                ConditionalWrite::Put { pk, sk, data, .. } => {
                    let document = decode_document_value(value.as_deref().ok_or_else(|| {
                        DibiError::CorruptValue("validated conditional value is missing".to_owned())
                    })?)?;
                    mutations.insert(
                        key,
                        Some(encode_document_value(&StoredDocument {
                            data: data.clone(),
                            version: increment_version(document.version, pk, sk)?,
                        })),
                    );
                }
                ConditionalWrite::Delete { .. } => {
                    mutations.insert(key, None);
                }
            }
        }
        let commit_id = self.commit_mutations(mutations)?;
        Ok(ConditionalWriteOutcome::Applied(WriteResult {
            commit_id: Some(commit_id),
        }))
    }

    pub fn read_outbox(
        &self,
        after_commit_id: Option<u64>,
        limit: usize,
    ) -> Result<Vec<CommitRecord>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let start = after_commit_id.map(|commit_id| commit_id.to_be_bytes());
        let mode = start
            .as_ref()
            .map(|key| IteratorMode::From(key.as_slice(), Direction::Forward))
            .unwrap_or(IteratorMode::Start);
        let snapshot = self.inner.db.snapshot();
        let outbox_cf = self.cf(BACKUP_OUTBOX_CF)?;
        let mut records = Vec::with_capacity(limit);
        for item in snapshot.iterator_cf(outbox_cf, mode) {
            let (encoded_key, encoded_value) = item?;
            if start
                .as_ref()
                .is_some_and(|cursor| encoded_key.as_ref() == cursor.as_slice())
            {
                continue;
            }
            let commit_id = decode_commit_id(&encoded_key)?;
            let record = CommitRecord::decode(&encoded_value)?;
            if record.commit_id != commit_id {
                return Err(DibiError::CorruptCommitRecord(format!(
                    "outbox key {commit_id} does not match record {}",
                    record.commit_id
                )));
            }
            records.push(record);
            if records.len() == limit {
                break;
            }
        }
        Ok(records)
    }

    pub fn last_commit_id(&self) -> Result<u64> {
        let meta_cf = self.cf(META_CF)?;
        let value = self
            .inner
            .db
            .get_cf(meta_cf, LAST_COMMIT_ID_KEY)?
            .ok_or_else(|| DibiError::CorruptMetadata("last_commit_id is missing".to_owned()))?;
        decode_u64_metadata("last_commit_id", &value)
    }

    pub fn db_uuid(&self) -> Result<Uuid> {
        let meta_cf = self.cf(META_CF)?;
        let value = self
            .inner
            .db
            .get_cf(meta_cf, DB_UUID_KEY)?
            .ok_or_else(|| DibiError::CorruptMetadata("db_uuid is missing".to_owned()))?;
        Uuid::from_slice(&value)
            .map_err(|error| DibiError::CorruptMetadata(format!("invalid db_uuid: {error}")))
    }

    fn initialize_or_validate_metadata(&self, is_new: bool) -> Result<()> {
        let meta_cf = self.cf(META_CF)?;
        let format_version = self.inner.db.get_cf(meta_cf, FORMAT_VERSION_KEY)?;
        let db_uuid = self.inner.db.get_cf(meta_cf, DB_UUID_KEY)?;
        let last_commit_id = self.inner.db.get_cf(meta_cf, LAST_COMMIT_ID_KEY)?;
        let present_count = [
            format_version.is_some(),
            db_uuid.is_some(),
            last_commit_id.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();

        if present_count == 0 {
            if !is_new {
                return Err(DibiError::CorruptMetadata(
                    "all required metadata is missing from an existing database".to_owned(),
                ));
            }
            let mut batch = WriteBatch::default();
            batch.put_cf(
                meta_cf,
                FORMAT_VERSION_KEY,
                DATABASE_FORMAT_VERSION.to_be_bytes(),
            );
            batch.put_cf(meta_cf, DB_UUID_KEY, Uuid::new_v4().as_bytes());
            batch.put_cf(meta_cf, LAST_COMMIT_ID_KEY, 0_u64.to_be_bytes());
            return self.write_sync(batch);
        }
        if present_count != 3 {
            return Err(DibiError::CorruptMetadata(
                "required metadata is only partially present".to_owned(),
            ));
        }

        let format_version = decode_u32_metadata(
            "format_version",
            format_version.as_deref().unwrap_or_default(),
        )?;
        if format_version != DATABASE_FORMAT_VERSION {
            return Err(DibiError::UnsupportedFormatVersion(format_version));
        }
        Uuid::from_slice(db_uuid.as_deref().unwrap_or_default())
            .map_err(|error| DibiError::CorruptMetadata(format!("invalid db_uuid: {error}")))?;
        decode_u64_metadata(
            "last_commit_id",
            last_commit_id.as_deref().unwrap_or_default(),
        )?;
        Ok(())
    }

    fn commit_mutations(&self, mutations: BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> Result<u64> {
        let commit_id = self
            .last_commit_id()?
            .checked_add(1)
            .ok_or(DibiError::CommitIdOverflow)?;
        let docs_cf = self.cf(DOCS_CF)?;
        let meta_cf = self.cf(META_CF)?;
        let outbox_cf = self.cf(BACKUP_OUTBOX_CF)?;
        let record = CommitRecord {
            format_version: COMMIT_RECORD_FORMAT_VERSION,
            commit_id,
            mutations: mutations
                .iter()
                .map(|(encoded_key, encoded_value)| match encoded_value {
                    Some(encoded_value) => CommitMutation::Set {
                        encoded_key: encoded_key.clone(),
                        encoded_value: encoded_value.clone(),
                    },
                    None => CommitMutation::Delete {
                        encoded_key: encoded_key.clone(),
                    },
                })
                .collect(),
        };
        let mut batch = WriteBatch::default();
        for (encoded_key, encoded_value) in mutations {
            match encoded_value {
                Some(encoded_value) => batch.put_cf(docs_cf, encoded_key, encoded_value),
                None => batch.delete_cf(docs_cf, encoded_key),
            }
        }
        batch.put_cf(meta_cf, LAST_COMMIT_ID_KEY, commit_id.to_be_bytes());
        batch.put_cf(outbox_cf, commit_id.to_be_bytes(), record.encode());
        self.write_sync(batch)?;
        Ok(commit_id)
    }

    fn read_document(&self, encoded_key: &[u8]) -> Result<Option<StoredDocument>> {
        self.read_encoded_document(encoded_key)?
            .as_deref()
            .map(decode_document_value)
            .transpose()
    }

    fn read_encoded_document(&self, encoded_key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.inner.db.get_cf(self.cf(DOCS_CF)?, encoded_key)?)
    }

    fn write_sync(&self, batch: WriteBatch) -> Result<()> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(true);
        write_options.disable_wal(false);
        self.inner.db.write_opt(batch, &write_options)?;
        Ok(())
    }

    fn lock_writes(&self) -> Result<std::sync::MutexGuard<'_, ()>> {
        self.inner
            .write_gate
            .lock()
            .map_err(|_| DibiError::WriteGatePoisoned)
    }

    fn cf(&self, name: &str) -> Result<&ColumnFamily> {
        self.inner.db.cf_handle(name).ok_or_else(|| {
            DibiError::CorruptMetadata(format!("required column family {name:?} is missing"))
        })
    }
}

fn validate_column_families(path: &Path) -> Result<()> {
    let actual = DB::list_cf(&Options::default(), path)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let expected = ["default", DOCS_CF, META_CF, BACKUP_OUTBOX_CF]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(DibiError::CorruptMetadata(format!(
            "column families do not match: expected {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

fn encode_pk_prefix(pk: &str) -> Vec<u8> {
    let encoded = encode_document_key(pk, "");
    encoded[..encoded.len() - 2].to_vec()
}

fn application_key(operation: &ApplicationWrite) -> (&str, &str) {
    match operation {
        ApplicationWrite::Put { pk, sk, .. } | ApplicationWrite::Delete { pk, sk } => (pk, sk),
    }
}

fn conditional_key(operation: &ConditionalWrite) -> (&str, &str, Option<i64>) {
    match operation {
        ConditionalWrite::Create { pk, sk, .. } => (pk, sk, None),
        ConditionalWrite::Put {
            pk,
            sk,
            expected_version,
            ..
        }
        | ConditionalWrite::Delete {
            pk,
            sk,
            expected_version,
        } => (pk, sk, Some(*expected_version)),
    }
}

fn validate_conditional_uniqueness(operations: &[ConditionalWrite]) -> Result<()> {
    let mut keys = BTreeSet::new();
    for operation in operations {
        let (pk, sk, _) = conditional_key(operation);
        if !keys.insert((pk, sk)) {
            return Err(DibiError::DuplicateConditionalKey {
                pk: pk.to_owned(),
                sk: sk.to_owned(),
            });
        }
    }
    Ok(())
}

fn increment_version(version: i64, pk: &str, sk: &str) -> Result<i64> {
    version
        .checked_add(1)
        .ok_or_else(|| DibiError::VersionOverflow {
            pk: pk.to_owned(),
            sk: sk.to_owned(),
        })
}

fn decode_u32_metadata(name: &str, encoded: &[u8]) -> Result<u32> {
    let bytes: [u8; 4] = encoded
        .try_into()
        .map_err(|_| DibiError::CorruptMetadata(format!("{name} must contain exactly 4 bytes")))?;
    Ok(u32::from_be_bytes(bytes))
}

fn decode_u64_metadata(name: &str, encoded: &[u8]) -> Result<u64> {
    let bytes: [u8; 8] = encoded
        .try_into()
        .map_err(|_| DibiError::CorruptMetadata(format!("{name} must contain exactly 8 bytes")))?;
    Ok(u64::from_be_bytes(bytes))
}

fn decode_commit_id(encoded: &[u8]) -> Result<u64> {
    let bytes: [u8; 8] = encoded.try_into().map_err(|_| {
        DibiError::CorruptCommitRecord("outbox key must contain exactly 8 bytes".to_owned())
    })?;
    Ok(u64::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use rocksdb::{ColumnFamilyDescriptor, DB, Options, WriteBatch};
    use tempfile::tempdir;

    use super::*;

    fn raw_database(path: &Path) -> DB {
        let options = Options::default();
        DB::open_cf_descriptors(
            &options,
            path,
            [DOCS_CF, META_CF, BACKUP_OUTBOX_CF]
                .into_iter()
                .map(|name| ColumnFamilyDescriptor::new(name, Options::default())),
        )
        .unwrap()
    }

    #[test]
    fn malformed_storage_values_are_reported() {
        let directory = tempdir().unwrap();
        let engine = DibiEngine::open(directory.path()).unwrap();
        let docs_cf = engine.cf(DOCS_CF).unwrap();
        engine
            .inner
            .db
            .put_cf(docs_cf, b"malformed", [1, 2, 3])
            .unwrap();
        assert!(engine.scan(None, 10).is_err());
        let outbox_cf = engine.cf(BACKUP_OUTBOX_CF).unwrap();
        engine.inner.db.put_cf(outbox_cf, [0; 8], [0xff]).unwrap();
        assert!(engine.read_outbox(None, 10).is_err());
    }

    #[test]
    fn partial_and_unsupported_metadata_are_rejected() {
        let directory = tempdir().unwrap();
        {
            let engine = DibiEngine::open(directory.path()).unwrap();
            let meta_cf = engine.cf(META_CF).unwrap();
            engine.inner.db.delete_cf(meta_cf, DB_UUID_KEY).unwrap();
        }
        assert!(matches!(
            DibiEngine::open(directory.path()),
            Err(DibiError::CorruptMetadata(_))
        ));

        let unsupported_directory = tempdir().unwrap();
        {
            let engine = DibiEngine::open(unsupported_directory.path()).unwrap();
            let meta_cf = engine.cf(META_CF).unwrap();
            engine
                .inner
                .db
                .put_cf(meta_cf, FORMAT_VERSION_KEY, 2_u32.to_be_bytes())
                .unwrap();
        }
        assert!(matches!(
            DibiEngine::open(unsupported_directory.path()),
            Err(DibiError::UnsupportedFormatVersion(2))
        ));
    }

    #[test]
    fn externally_rewritten_metadata_is_checked_on_reopen() {
        let directory = tempdir().unwrap();
        let engine = DibiEngine::open(directory.path()).unwrap();
        let uuid = engine.db_uuid().unwrap();
        drop(engine);
        let database = raw_database(directory.path());
        let meta_cf = database.cf_handle(META_CF).unwrap();
        let mut batch = WriteBatch::default();
        batch.put_cf(meta_cf, DB_UUID_KEY, uuid.as_bytes());
        database.write(batch).unwrap();
        drop(database);
        assert_eq!(
            DibiEngine::open(directory.path())
                .unwrap()
                .db_uuid()
                .unwrap(),
            uuid
        );
    }
}
