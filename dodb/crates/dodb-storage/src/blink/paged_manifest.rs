use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dodb_core::{Error, Lsn, PageId, Result};

#[cfg(feature = "blink-demand-paging")]
use crate::durable_file::{DurableFile, ProductionFile};
#[cfg(feature = "blink-demand-paging")]
use crate::wal::{WalIdentity, WalPageImageFormat};

use super::version_index::{NodeRef, StoreIdentity, VersionIndex};
use super::{PagedReadConfig, PagedReadHandle, page_cache, resource, source_manager};

const MARKER_MAGIC: [u8; 8] = *b"DBLKPAG1";
const POINTER_MAGIC: [u8; 8] = *b"DBLKCUR1";
const MANIFEST_MAGIC: [u8; 8] = *b"DBLKMNF1";
const FORMAT_VERSION: u32 = 1;
const MARKER_BYTES: usize = 80;
const POINTER_BYTES: usize = 88;
const MANIFEST_HEADER_BYTES: usize = 88;
const MANIFEST_PAYLOAD_BYTES: usize = 192;
const MANIFEST_BYTES: usize = MANIFEST_HEADER_BYTES + MANIFEST_PAYLOAD_BYTES;
const MANIFEST_FILE_PREFIX: &str = "blink-paged-manifest-";
const MANIFEST_FILE_SUFFIX: &str = ".bin";
const FORMAT_MARKER_SUFFIX: &str = "blink-paged.format";
const SIDECAR_DIRECTORY_SUFFIX: &str = "blink-paged";
const CURRENT_POINTER_NAME: &str = "blink-paged.current";
static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PagedManifest {
    pub identity: StoreIdentity,
    pub generation: u64,
    pub tree_root: PageId,
    pub free_list_root: Option<PageId>,
    pub high_water_page_id: PageId,
    pub durable_index_root: NodeRef,
    pub source_catalog_root: Option<NodeRef>,
    pub recovery_wal_file_number: u64,
    pub recovery_offset: u64,
    pub committed_prefix_lsn: Lsn,
    pub previous_generation: u64,
}

impl PagedManifest {
    pub fn validate(&self) -> Result<()> {
        if self.generation == 0
            || self.tree_root.get() < 2
            || self.high_water_page_id.get() < self.tree_root.get()
            || self
                .free_list_root
                .is_some_and(|page_id| page_id.get() < 2 || page_id > self.high_water_page_id)
            || self.durable_index_root.file_number == 0
            || self.durable_index_root.offset != 0
            || self
                .source_catalog_root
                .is_some_and(|node| node.file_number == 0 || node.offset != 0)
            || self.recovery_wal_file_number == 0
            || self.previous_generation >= self.generation
        {
            return Err(Error::invalid_input("paged manifest fields are invalid"));
        }
        Ok(())
    }
}

pub struct PagedManifestStore {
    database_path: PathBuf,
    directory: PathBuf,
    marker_path: PathBuf,
    pointer_path: PathBuf,
    identity: StoreIdentity,
    manifest_file_number: u64,
    current: PagedManifest,
    next_file_number: u64,
    broken: bool,
}

pub struct PagedValidatedOpenConfig<F> {
    pub index: Arc<VersionIndex>,
    pub tenant_id: u64,
    pub resources: resource::BlinkResourceManager,
    pub sources: source_manager::SourceManager,
    pub loader: page_cache::PageLoaderConfig,
    pub header_length: F,
}

#[cfg(feature = "blink-demand-paging")]
pub struct PagedRecoveryOpenConfig<F> {
    pub index: Arc<VersionIndex>,
    pub tenant_id: u64,
    pub resources: resource::BlinkResourceManager,
    pub sources: source_manager::SourceManager,
    pub loader: page_cache::PageLoaderConfig,
    pub header_length: F,
    pub recovery_wal_path: PathBuf,
    pub recovery_source: super::version_index::SourceIdentity,
    pub wal_identity: WalIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PagedManifestPointerStage {
    BeforeTemporaryCreate,
    AfterTemporarySync,
    AfterRename,
    AfterDirectorySync,
}

impl PagedManifestStore {
    pub fn validate_format_marker(database_path: &Path, identity: StoreIdentity) -> Result<()> {
        validate_marker(&format_marker_path(database_path), identity)
    }

    pub fn has_format_marker(database_path: &Path) -> bool {
        format_marker_path(database_path).is_file()
    }

    pub fn sidecar_directory(database_path: &Path) -> PathBuf {
        sidecar_directory(database_path)
    }

    pub fn prepare_format(database_path: &Path, identity: StoreIdentity) -> Result<()> {
        let marker_path = format_marker_path(database_path);
        let directory = sidecar_directory(database_path);
        if marker_path.exists() {
            validate_marker(&marker_path, identity)?;
            if directory.join(CURRENT_POINTER_NAME).exists() {
                return Err(Error::invalid_input(
                    "paged Blink manifest is already initialized",
                ));
            }
            fs::create_dir_all(&directory)?;
            return Ok(());
        }
        if directory.exists() {
            return Err(Error::unsupported_format(
                "paged Blink sidecar directory has no compatibility marker",
            ));
        }
        if database_path.is_file() && fs::metadata(database_path)?.len() != 0 {
            return Err(Error::invalid_input(
                "paged Blink format requires a fresh database path",
            ));
        }
        let parent = marker_path
            .parent()
            .ok_or_else(|| Error::invalid_input("paged format marker has no parent"))?;
        fs::create_dir_all(parent)?;
        write_marker(&marker_path, identity)?;
        sync_directory(parent)?;
        fs::create_dir(&directory)?;
        sync_directory(parent)?;
        Ok(())
    }

    pub fn create_with_index(
        database_path: &Path,
        manifest: PagedManifest,
        index: &VersionIndex,
    ) -> Result<Self> {
        validate_index_root(index, manifest)?;
        Self::create_unchecked(database_path, manifest)
    }

    fn create_unchecked(database_path: &Path, manifest: PagedManifest) -> Result<Self> {
        manifest.validate()?;
        let marker_path = format_marker_path(database_path);
        if !marker_path.exists() {
            Self::prepare_format(database_path, manifest.identity)?;
        } else {
            validate_marker(&marker_path, manifest.identity)?;
        }
        let directory = sidecar_directory(database_path);
        let pointer_path = directory.join(CURRENT_POINTER_NAME);
        if pointer_path.exists() {
            return Err(Error::invalid_input(
                "paged Blink manifest is already initialized",
            ));
        }
        let mut store = Self {
            database_path: database_path.to_path_buf(),
            directory,
            marker_path,
            pointer_path,
            identity: manifest.identity,
            manifest_file_number: 0,
            current: manifest,
            next_file_number: 1,
            broken: false,
        };
        let manifest_file_number = store.write_manifest(&manifest)?;
        store.write_pointer(manifest_file_number, manifest.generation, &mut |_| Ok(()))?;
        store.manifest_file_number = manifest_file_number;
        store.next_file_number = manifest_file_number
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("paged manifest file number exhausted"))?;
        Ok(store)
    }

    pub fn open_with_index(
        database_path: &Path,
        identity: StoreIdentity,
        index: &VersionIndex,
    ) -> Result<Self> {
        let store = Self::open_unchecked(database_path, identity)?;
        validate_index_root(index, store.current)?;
        Ok(store)
    }

    pub fn open_with_validated_pages<F>(
        database_path: &Path,
        identity: StoreIdentity,
        config: PagedValidatedOpenConfig<F>,
    ) -> Result<(Self, PagedReadHandle)>
    where
        F: Fn(super::version_index::SourceIdentity) -> Result<usize> + Send + Sync + 'static,
    {
        let store = Self::open_with_index(database_path, identity, &config.index)?;
        let manifest = store.current;
        let root = config
            .index
            .root_from_node_reference(manifest.durable_index_root)?;
        let handle = PagedReadHandle::open_with_indexed_page_validation(
            PagedReadConfig {
                tenant_id: config.tenant_id,
                root,
                root_page_id: manifest.tree_root,
                free_list_head: manifest.free_list_root,
                high_water_page_id: manifest.high_water_page_id,
                loader: config.loader,
            },
            config.index,
            config.resources,
            config.sources,
            config.header_length,
        )?;
        Ok((store, handle))
    }

    #[cfg(feature = "blink-demand-paging")]
    pub fn open_with_recovered_pages<F>(
        database_path: &Path,
        identity: StoreIdentity,
        config: PagedRecoveryOpenConfig<F>,
    ) -> Result<(Self, PagedReadHandle)>
    where
        F: Fn(super::version_index::SourceIdentity) -> Result<usize> + Send + Sync + 'static,
    {
        let store = Self::open_with_index(database_path, identity, &config.index)?;
        let manifest = store.current;
        if config.recovery_source.kind != super::version_index::SourceFileKind::WalSegment
            || config.recovery_source.file_number != manifest.recovery_wal_file_number
            || config.wal_identity.database_uuid != identity.database_uuid
            || config.wal_identity.shard_epoch.get() != identity.wal_epoch
        {
            return Err(Error::invalid_input(
                "paged recovery source differs from the manifest identity or boundary",
            ));
        }
        let root = config
            .index
            .root_from_node_reference(manifest.durable_index_root)?;
        let _checkpoint_root_lease = config.sources.pin_manifest_root(&config.index, &store)?;
        let header_length = Arc::new(config.header_length);
        let checkpoint_header_length = Arc::clone(&header_length);
        let checkpoint_handle = PagedReadHandle::new(
            PagedReadConfig {
                tenant_id: config.tenant_id,
                root,
                root_page_id: manifest.tree_root,
                free_list_head: manifest.free_list_root,
                high_water_page_id: manifest.high_water_page_id,
                loader: config.loader,
            },
            Arc::clone(&config.index),
            config.resources.clone(),
            config.sources.clone(),
            move |source| checkpoint_header_length(source),
        )?;
        if !config.recovery_wal_path.is_file() {
            return Err(Error::corruption("paged recovery WAL segment is missing"));
        }
        let mut wal = ProductionFile::open(&config.recovery_wal_path)?;
        let committed_prefix_length = wal.len()?;
        let mut builder = super::paged_recovery::PagedRecoveryIndexBuilder::new(
            &config.index,
            manifest,
            config.recovery_source,
            committed_prefix_length,
        )?;
        let mut resolve_page =
            |descriptor| checkpoint_handle.load_indexed_version_image(descriptor);
        let summary = builder.replay_wal(
            &mut wal,
            &config.wal_identity,
            WalPageImageFormat::ExperimentalBlink,
            &mut resolve_page,
        )?;
        if summary.source_header_identity != config.recovery_source.header_identity {
            return Err(Error::corruption(
                "paged recovery WAL header identity differs from its source identity",
            ));
        }
        let handle = PagedReadHandle::open_with_indexed_page_validation(
            PagedReadConfig {
                tenant_id: config.tenant_id,
                root: builder.current_root(),
                root_page_id: manifest.tree_root,
                free_list_head: manifest.free_list_root,
                high_water_page_id: manifest.high_water_page_id,
                loader: config.loader,
            },
            config.index,
            config.resources,
            config.sources,
            move |source| header_length(source),
        )?;
        Ok((store, handle))
    }

    fn open_unchecked(database_path: &Path, identity: StoreIdentity) -> Result<Self> {
        let marker_path = format_marker_path(database_path);
        validate_marker(&marker_path, identity)?;
        let directory = sidecar_directory(database_path);
        let pointer_path = directory.join(CURRENT_POINTER_NAME);
        let (manifest_file_number, generation) = read_pointer(&pointer_path, identity)?;
        let manifest_path = manifest_path(&directory, manifest_file_number);
        let current = read_manifest(&manifest_path, identity)?;
        if current.generation != generation {
            return Err(Error::corruption(
                "paged manifest pointer generation differs from its manifest",
            ));
        }
        let next_file_number = scan_next_manifest_number(&directory)?;
        Ok(Self {
            database_path: database_path.to_path_buf(),
            directory,
            marker_path,
            pointer_path,
            identity,
            manifest_file_number,
            current,
            next_file_number,
            broken: false,
        })
    }

    pub fn current(&self) -> PagedManifest {
        self.current
    }

    pub fn current_file_number(&self) -> u64 {
        self.manifest_file_number
    }

    pub fn publish_with_index(
        &mut self,
        manifest: PagedManifest,
        index: &VersionIndex,
    ) -> Result<()> {
        self.publish_with_index_and_hook(manifest, index, |_| Ok(()))
    }

    pub(crate) fn publish_with_index_and_hook<F>(
        &mut self,
        manifest: PagedManifest,
        index: &VersionIndex,
        mut pointer_fault: F,
    ) -> Result<()>
    where
        F: FnMut(PagedManifestPointerStage) -> Result<()>,
    {
        validate_index_root(index, manifest)?;
        self.publish_with_pointer_hook(manifest, || Ok(()), &mut pointer_fault)
    }

    #[cfg(test)]
    fn publish_with_hook<F>(&mut self, manifest: PagedManifest, before_pointer: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        self.publish_with_pointer_hook(manifest, before_pointer, &mut |_| Ok(()))
    }

    fn publish_with_pointer_hook<F, G>(
        &mut self,
        manifest: PagedManifest,
        before_pointer: F,
        pointer_fault: &mut G,
    ) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
        G: FnMut(PagedManifestPointerStage) -> Result<()>,
    {
        if self.broken {
            return Err(Error::invariant(
                "paged manifest store cannot publish after an uncertain pointer failure",
            ));
        }
        manifest.validate()?;
        let expected_generation = self
            .current
            .generation
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("paged manifest generation exhausted"))?;
        if manifest.identity != self.identity
            || manifest.generation != expected_generation
            || manifest.previous_generation != self.current.generation
        {
            return Err(Error::invalid_input(
                "paged manifest publication does not extend the selected generation",
            ));
        }
        let file_number = self.write_manifest(&manifest)?;
        before_pointer()?;
        if let Err(error) = self.write_pointer(file_number, manifest.generation, pointer_fault) {
            self.broken = true;
            return Err(error);
        }
        self.manifest_file_number = file_number;
        self.current = manifest;
        self.next_file_number = file_number
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("paged manifest file number exhausted"))?;
        Ok(())
    }

    fn write_manifest(&mut self, manifest: &PagedManifest) -> Result<u64> {
        let bytes = encode_manifest(manifest)?;
        loop {
            let file_number = self.next_file_number;
            self.next_file_number = file_number
                .checked_add(1)
                .ok_or_else(|| Error::invalid_input("paged manifest file number exhausted"))?;
            if file_number == 0 {
                return Err(Error::invariant(
                    "paged manifest allocated file number zero",
                ));
            }
            let path = manifest_path(&self.directory, file_number);
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(mut file) => {
                    file.write_all(&bytes)?;
                    file.sync_all()?;
                    sync_directory(&self.directory)?;
                    return Ok(file_number);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn write_pointer<G>(
        &self,
        file_number: u64,
        generation: u64,
        pointer_fault: &mut G,
    ) -> Result<()>
    where
        G: FnMut(PagedManifestPointerStage) -> Result<()>,
    {
        let bytes = encode_pointer(self.identity, file_number, generation)?;
        pointer_fault(PagedManifestPointerStage::BeforeTemporaryCreate)?;
        loop {
            let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::AcqRel);
            let temporary_path = self.directory.join(format!(
                "{CURRENT_POINTER_NAME}.tmp-{}-{sequence}",
                std::process::id()
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary_path)
            {
                Ok(mut file) => {
                    file.write_all(&bytes)?;
                    file.sync_all()?;
                    pointer_fault(PagedManifestPointerStage::AfterTemporarySync)?;
                    fs::rename(&temporary_path, &self.pointer_path)?;
                    pointer_fault(PagedManifestPointerStage::AfterRename)?;
                    sync_directory(&self.directory)?;
                    pointer_fault(PagedManifestPointerStage::AfterDirectorySync)?;
                    return Ok(());
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn marker_path(&self) -> &Path {
        &self.marker_path
    }
}

pub fn version_index_directory(database_path: &Path) -> PathBuf {
    sidecar_directory(database_path).join("index")
}

fn validate_index_root(index: &VersionIndex, manifest: PagedManifest) -> Result<()> {
    let root = index.root_from_node_reference(manifest.durable_index_root)?;
    if root.generation() == 0 {
        return Err(Error::corruption("paged manifest index root is empty"));
    }
    index.validate_durable_root(root)
}

fn format_marker_path(database_path: &Path) -> PathBuf {
    sidecar_path(database_path, FORMAT_MARKER_SUFFIX)
}

fn sidecar_directory(database_path: &Path) -> PathBuf {
    sidecar_path(database_path, SIDECAR_DIRECTORY_SUFFIX)
}

fn sidecar_path(database_path: &Path, extension: &str) -> PathBuf {
    let mut name = database_path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(extension);
    database_path.with_file_name(name)
}

fn manifest_path(directory: &Path, file_number: u64) -> PathBuf {
    directory.join(format!(
        "{MANIFEST_FILE_PREFIX}{file_number:020}{MANIFEST_FILE_SUFFIX}"
    ))
}

fn write_marker(path: &Path, identity: StoreIdentity) -> Result<()> {
    let bytes = encode_marker(identity);
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn encode_marker(identity: StoreIdentity) -> [u8; MARKER_BYTES] {
    let mut bytes = [0u8; MARKER_BYTES];
    bytes[0..8].copy_from_slice(&MARKER_MAGIC);
    bytes[8..10].copy_from_slice(&(FORMAT_VERSION as u16).to_le_bytes());
    bytes[16..32].copy_from_slice(&identity.database_uuid);
    bytes[32..40].copy_from_slice(&identity.wal_epoch.to_le_bytes());
    let checksum = blake3::hash(&bytes[..48]);
    bytes[48..80].copy_from_slice(checksum.as_bytes());
    bytes
}

fn validate_marker(path: &Path, identity: StoreIdentity) -> Result<()> {
    let bytes = read_exact_file::<MARKER_BYTES>(path, "paged format marker")?;
    if bytes[0..8] != MARKER_MAGIC
        || u16::from_le_bytes(bytes[8..10].try_into().unwrap()) != FORMAT_VERSION as u16
        || bytes[10..16].iter().any(|byte| *byte != 0)
        || bytes[40..48].iter().any(|byte| *byte != 0)
        || bytes[16..32] != identity.database_uuid
        || u64::from_le_bytes(bytes[32..40].try_into().unwrap()) != identity.wal_epoch
        || blake3::hash(&bytes[..48]).as_bytes() != &bytes[48..80]
    {
        return Err(Error::corruption(
            "paged format marker is invalid or belongs to another store",
        ));
    }
    Ok(())
}

fn encode_pointer(
    identity: StoreIdentity,
    file_number: u64,
    generation: u64,
) -> Result<[u8; POINTER_BYTES]> {
    if file_number == 0 || generation == 0 {
        return Err(Error::invalid_input(
            "paged manifest pointer fields are invalid",
        ));
    }
    let mut bytes = [0u8; POINTER_BYTES];
    bytes[0..8].copy_from_slice(&POINTER_MAGIC);
    bytes[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes[16..32].copy_from_slice(&identity.database_uuid);
    bytes[32..40].copy_from_slice(&identity.wal_epoch.to_le_bytes());
    bytes[40..48].copy_from_slice(&file_number.to_le_bytes());
    bytes[48..56].copy_from_slice(&generation.to_le_bytes());
    let checksum = blake3::hash(&bytes[..56]);
    bytes[56..88].copy_from_slice(checksum.as_bytes());
    Ok(bytes)
}

fn read_pointer(path: &Path, identity: StoreIdentity) -> Result<(u64, u64)> {
    let bytes = read_exact_file::<POINTER_BYTES>(path, "paged manifest pointer")?;
    if bytes[0..8] != POINTER_MAGIC
        || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) != FORMAT_VERSION
        || bytes[12..16].iter().any(|byte| *byte != 0)
        || bytes[16..32] != identity.database_uuid
        || u64::from_le_bytes(bytes[32..40].try_into().unwrap()) != identity.wal_epoch
        || blake3::hash(&bytes[..56]).as_bytes() != &bytes[56..88]
    {
        return Err(Error::corruption("paged manifest pointer is invalid"));
    }
    let file_number = u64::from_le_bytes(bytes[40..48].try_into().unwrap());
    let generation = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
    if file_number == 0 || generation == 0 {
        return Err(Error::corruption("paged manifest pointer is incomplete"));
    }
    Ok((file_number, generation))
}

fn encode_manifest(manifest: &PagedManifest) -> Result<[u8; MANIFEST_BYTES]> {
    manifest.validate()?;
    let mut bytes = [0u8; MANIFEST_BYTES];
    bytes[0..8].copy_from_slice(&MANIFEST_MAGIC);
    bytes[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes[12..16].copy_from_slice(&(MANIFEST_HEADER_BYTES as u32).to_le_bytes());
    bytes[16..32].copy_from_slice(&manifest.identity.database_uuid);
    bytes[32..40].copy_from_slice(&manifest.identity.wal_epoch.to_le_bytes());
    bytes[40..48].copy_from_slice(&manifest.generation.to_le_bytes());
    bytes[48..52].copy_from_slice(&(MANIFEST_PAYLOAD_BYTES as u32).to_le_bytes());
    bytes[MANIFEST_HEADER_BYTES..MANIFEST_HEADER_BYTES + 8]
        .copy_from_slice(&manifest.generation.to_le_bytes());
    bytes[MANIFEST_HEADER_BYTES + 8..MANIFEST_HEADER_BYTES + 16]
        .copy_from_slice(&manifest.tree_root.get().to_le_bytes());
    bytes[MANIFEST_HEADER_BYTES + 16..MANIFEST_HEADER_BYTES + 24].copy_from_slice(
        &manifest
            .free_list_root
            .map_or(u64::MAX, PageId::get)
            .to_le_bytes(),
    );
    bytes[MANIFEST_HEADER_BYTES + 24..MANIFEST_HEADER_BYTES + 32]
        .copy_from_slice(&manifest.high_water_page_id.get().to_le_bytes());
    encode_node_reference(
        manifest.durable_index_root,
        &mut bytes[MANIFEST_HEADER_BYTES + 32..MANIFEST_HEADER_BYTES + 80],
    );
    if let Some(source_catalog_root) = manifest.source_catalog_root {
        encode_node_reference(
            source_catalog_root,
            &mut bytes[MANIFEST_HEADER_BYTES + 80..MANIFEST_HEADER_BYTES + 128],
        );
    }
    bytes[MANIFEST_HEADER_BYTES + 128..MANIFEST_HEADER_BYTES + 136]
        .copy_from_slice(&manifest.recovery_wal_file_number.to_le_bytes());
    bytes[MANIFEST_HEADER_BYTES + 136..MANIFEST_HEADER_BYTES + 144]
        .copy_from_slice(&manifest.recovery_offset.to_le_bytes());
    bytes[MANIFEST_HEADER_BYTES + 144..MANIFEST_HEADER_BYTES + 152]
        .copy_from_slice(&manifest.committed_prefix_lsn.get().to_le_bytes());
    bytes[MANIFEST_HEADER_BYTES + 152..MANIFEST_HEADER_BYTES + 160]
        .copy_from_slice(&manifest.previous_generation.to_le_bytes());
    let payload_checksum = blake3::hash(&bytes[MANIFEST_HEADER_BYTES..MANIFEST_BYTES - 32]);
    bytes[MANIFEST_BYTES - 32..].copy_from_slice(payload_checksum.as_bytes());
    let header_checksum = blake3::hash(&bytes[..56]);
    bytes[56..88].copy_from_slice(header_checksum.as_bytes());
    Ok(bytes)
}

fn decode_manifest(
    bytes: &[u8; MANIFEST_BYTES],
    expected_identity: StoreIdentity,
) -> Result<PagedManifest> {
    if bytes[0..8] != MANIFEST_MAGIC
        || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) != FORMAT_VERSION
        || u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize != MANIFEST_HEADER_BYTES
        || bytes[16..32] != expected_identity.database_uuid
        || u64::from_le_bytes(bytes[32..40].try_into().unwrap()) != expected_identity.wal_epoch
        || u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize != MANIFEST_PAYLOAD_BYTES
        || bytes[52..56].iter().any(|byte| *byte != 0)
        || blake3::hash(&bytes[..56]).as_bytes() != &bytes[56..88]
        || blake3::hash(&bytes[MANIFEST_HEADER_BYTES..MANIFEST_BYTES - 32]).as_bytes()
            != &bytes[MANIFEST_BYTES - 32..]
    {
        return Err(Error::corruption(
            "paged manifest header or checksum is invalid",
        ));
    }
    let payload = &bytes[MANIFEST_HEADER_BYTES..];
    let generation = read_u64(payload, 0)?;
    if u64::from_le_bytes(bytes[40..48].try_into().unwrap()) != generation {
        return Err(Error::corruption("paged manifest generations disagree"));
    }
    let free_list_root = read_u64(payload, 16)?;
    let source_catalog_root = decode_optional_node_reference(&payload[80..128])?;
    let durable_index_root = decode_node_reference(&payload[32..80])?;
    let manifest = PagedManifest {
        identity: expected_identity,
        generation,
        tree_root: PageId::new(read_u64(payload, 8)?),
        free_list_root: (free_list_root != u64::MAX).then(|| PageId::new(free_list_root)),
        high_water_page_id: PageId::new(read_u64(payload, 24)?),
        durable_index_root,
        source_catalog_root,
        recovery_wal_file_number: read_u64(payload, 128)?,
        recovery_offset: read_u64(payload, 136)?,
        committed_prefix_lsn: Lsn::new(read_u64(payload, 144)?),
        previous_generation: read_u64(payload, 152)?,
    };
    manifest
        .validate()
        .map_err(|_| Error::corruption("paged manifest fields are invalid"))?;
    Ok(manifest)
}

fn encode_node_reference(node: NodeRef, bytes: &mut [u8]) {
    bytes[0..8].copy_from_slice(&node.file_number.to_le_bytes());
    bytes[8..16].copy_from_slice(&node.offset.to_le_bytes());
    bytes[16..48].copy_from_slice(&node.checksum);
}

fn decode_node_reference(bytes: &[u8]) -> Result<NodeRef> {
    if bytes.len() != 48 {
        return Err(Error::invariant(
            "paged manifest node reference size is invalid",
        ));
    }
    Ok(NodeRef {
        file_number: read_u64(bytes, 0)?,
        offset: read_u64(bytes, 8)?,
        checksum: bytes[16..48].try_into().unwrap(),
    })
}

fn decode_optional_node_reference(bytes: &[u8]) -> Result<Option<NodeRef>> {
    let node = decode_node_reference(bytes)?;
    if node.file_number == 0 && node.offset == 0 && node.checksum.iter().all(|byte| *byte == 0) {
        return Ok(None);
    }
    if node.file_number == 0 || node.offset != 0 {
        return Err(Error::corruption(
            "paged manifest source-catalog reference is invalid",
        ));
    }
    Ok(Some(node))
}

fn read_manifest(path: &Path, identity: StoreIdentity) -> Result<PagedManifest> {
    let bytes = read_exact_file::<MANIFEST_BYTES>(path, "paged manifest")?;
    decode_manifest(&bytes, identity)
}

fn read_exact_file<const SIZE: usize>(path: &Path, label: &str) -> Result<[u8; SIZE]> {
    let mut file = File::open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Error::corruption(format!("{label} is missing"))
        } else {
            error.into()
        }
    })?;
    if file.metadata()?.len() != SIZE as u64 {
        return Err(Error::corruption(format!("{label} length is invalid")));
    }
    let mut bytes = [0u8; SIZE];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn scan_next_manifest_number(directory: &Path) -> Result<u64> {
    let mut maximum_file_number = 0u64;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let Some(number) = file_name
            .strip_prefix(MANIFEST_FILE_PREFIX)
            .and_then(|suffix| suffix.strip_suffix(MANIFEST_FILE_SUFFIX))
        else {
            continue;
        };
        if number.len() != 20 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(Error::corruption("paged manifest filename is malformed"));
        }
        let file_number = number
            .parse::<u64>()
            .map_err(|_| Error::corruption("paged manifest filename number is invalid"))?;
        maximum_file_number = maximum_file_number.max(file_number);
    }
    maximum_file_number
        .checked_add(1)
        .ok_or_else(|| Error::invalid_input("paged manifest file number exhausted"))
        .map(|file_number| file_number.max(1))
}

fn sync_directory(directory: &Path) -> Result<()> {
    File::open(directory)?.sync_all()?;
    Ok(())
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    let word = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| Error::corruption("paged manifest field is truncated"))?;
    Ok(u64::from_le_bytes(word.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::{
        CURRENT_POINTER_NAME, MANIFEST_BYTES, PagedManifest, PagedManifestPointerStage,
        PagedManifestStore, encode_manifest, sidecar_directory, version_index_directory,
    };
    use crate::blink::BlinkStore;
    use crate::blink::resource::{BlinkResourceManager, ResourceLimits};
    use crate::blink::version_chain::{DeltaChain, DeltaChainEntry};
    use crate::blink::version_index::{
        NodeRef, SourceFileKind, SourceIdentity, StoreIdentity, VersionDescriptor, VersionIndex,
        VersionLocator, VersionToken, WalFrameLocator,
    };
    use crate::btree::DatabaseConfig;
    use crate::durable_file::ProductionFile;
    use dodb_core::{Error, Lsn, PageId};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "dodb-paged-manifest-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).unwrap();
            Self(directory)
        }

        fn database_path(&self) -> PathBuf {
            self.0.join("tenant.db")
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn manifest(
        identity: StoreIdentity,
        generation: u64,
        previous_generation: u64,
    ) -> PagedManifest {
        PagedManifest {
            identity,
            generation,
            tree_root: PageId::new(2),
            free_list_root: None,
            high_water_page_id: PageId::new(2),
            durable_index_root: NodeRef {
                file_number: generation,
                offset: 0,
                checksum: [generation as u8; 32],
            },
            source_catalog_root: None,
            recovery_wal_file_number: 1,
            recovery_offset: 64 + generation,
            committed_prefix_lsn: Lsn::new(generation * 10),
            previous_generation,
        }
    }

    fn write_manifest_store(directory: &TestDirectory) -> (StoreIdentity, PagedManifestStore) {
        let identity = StoreIdentity {
            database_uuid: [0x35; 16],
            wal_epoch: 21,
        };
        let store = PagedManifestStore::create_unchecked(
            &directory.database_path(),
            manifest(identity, 1, 0),
        )
        .unwrap();
        (identity, store)
    }

    fn resource_manager() -> BlinkResourceManager {
        BlinkResourceManager::new(ResourceLimits {
            global_bytes: 2 * 1024 * 1024,
            per_tenant_bytes: 2 * 1024 * 1024,
            progress_pool_bytes: 256 * 1024,
            max_tenants: 4,
            max_pending_waiters: 4,
            global_category_bytes: [2 * 1024 * 1024; 7],
            per_tenant_category_bytes: [2 * 1024 * 1024; 7],
        })
        .unwrap()
    }

    fn build_index(
        directory: &TestDirectory,
        identity: StoreIdentity,
    ) -> (VersionIndex, NodeRef, NodeRef) {
        let database_path = directory.database_path();
        PagedManifestStore::prepare_format(&database_path, identity).unwrap();
        let index = VersionIndex::open(
            &version_index_directory(&database_path),
            identity,
            resource_manager(),
            71,
            4,
        )
        .unwrap();
        let page_id = PageId::new(2);
        let root = index
            .insert(
                index.empty_root(),
                page_id,
                VersionDescriptor {
                    page_id,
                    token: VersionToken::Bootstrap { page_id },
                    image_lsn: Lsn::ZERO,
                    image_crc32c: 99,
                    locator: VersionLocator::CheckpointImage {
                        source: SourceIdentity {
                            kind: SourceFileKind::CheckpointPack,
                            file_number: 1,
                            header_identity: [0x44; 32],
                        },
                        offset: 4096,
                        source_length: 8192,
                        image_lsn: Lsn::ZERO,
                        image_crc32c: 99,
                    },
                },
            )
            .unwrap();
        let spill_reference = root.node_reference().unwrap();
        let durable_root = index.promote_root_to_durable(root).unwrap();
        (
            index,
            spill_reference,
            durable_root.node_reference().unwrap(),
        )
    }

    fn snapshot_files(directory: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                for child in fs::read_dir(&path).unwrap() {
                    let child = child.unwrap();
                    files.push((child.path(), fs::read(child.path()).unwrap()));
                }
            } else {
                files.push((path.clone(), fs::read(path).unwrap()));
            }
        }
        files.sort();
        files
    }

    #[test]
    fn create_publish_reopen_preserves_immutable_generations() {
        let directory = TestDirectory::new("publish");
        let (identity, mut store) = write_manifest_store(&directory);
        let first_file_number = store.current_file_number();
        let second_manifest = manifest(identity, 2, 1);
        store.publish_with_hook(second_manifest, || Ok(())).unwrap();
        let second_file_number = store.current_file_number();
        assert!(second_file_number > first_file_number);
        assert!(
            sidecar_directory(&directory.database_path())
                .join(format!("blink-paged-manifest-{first_file_number:020}.bin"))
                .is_file()
        );

        let reopened =
            PagedManifestStore::open_unchecked(&directory.database_path(), identity).unwrap();
        assert_eq!(reopened.current(), second_manifest);
        assert_eq!(reopened.current_file_number(), second_file_number);
        assert_eq!(fs::metadata(&directory.database_path()).is_ok(), false);
    }

    #[test]
    fn failed_pointer_publication_keeps_selected_generation_and_orphan() {
        let directory = TestDirectory::new("failed-publish");
        let (identity, mut store) = write_manifest_store(&directory);
        let failed_manifest = manifest(identity, 2, 1);
        assert!(
            store
                .publish_with_hook(failed_manifest, || {
                    Err(Error::invalid_input("injected pointer publication failure"))
                })
                .is_err()
        );
        assert_eq!(store.current(), manifest(identity, 1, 0));
        assert_eq!(
            fs::read_dir(sidecar_directory(&directory.database_path()))
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().ends_with(".bin"))
                .count(),
            2,
        );
        let reopened =
            PagedManifestStore::open_unchecked(&directory.database_path(), identity).unwrap();
        assert_eq!(reopened.current(), manifest(identity, 1, 0));
        store.publish_with_hook(failed_manifest, || Ok(())).unwrap();
        assert_eq!(store.current().generation, 2);
        assert_eq!(store.current_file_number(), 3);
    }

    #[test]
    fn pointer_publication_faults_cover_every_ordering_boundary() {
        for stage in [
            PagedManifestPointerStage::BeforeTemporaryCreate,
            PagedManifestPointerStage::AfterTemporarySync,
            PagedManifestPointerStage::AfterRename,
            PagedManifestPointerStage::AfterDirectorySync,
        ] {
            let directory = TestDirectory::new("pointer-boundary");
            let (identity, mut store) = write_manifest_store(&directory);
            let next_manifest = manifest(identity, 2, 1);
            let result =
                store.publish_with_pointer_hook(next_manifest, || Ok(()), &mut |current_stage| {
                    if current_stage == stage {
                        Err(Error::durability("injected pointer ordering failure"))
                    } else {
                        Ok(())
                    }
                });
            assert!(result.is_err(), "fault stage {stage:?} was not reached");
            assert_eq!(store.current(), manifest(identity, 1, 0));
            let reopened =
                PagedManifestStore::open_unchecked(&directory.database_path(), identity).unwrap();
            let expected_generation = match stage {
                PagedManifestPointerStage::BeforeTemporaryCreate
                | PagedManifestPointerStage::AfterTemporarySync => 1,
                PagedManifestPointerStage::AfterRename
                | PagedManifestPointerStage::AfterDirectorySync => 2,
            };
            assert_eq!(reopened.current().generation, expected_generation);
        }
    }

    #[test]
    fn paged_manifest_pointer_publication_survives_sigkill() {
        const CHILD_DIRECTORY: &str = "DODB_PAGED_MANIFEST_SIGKILL_DIRECTORY";
        const CHILD_STAGE: &str = "DODB_PAGED_MANIFEST_SIGKILL_STAGE";
        const CHILD_READY: &str = "DODB_PAGED_MANIFEST_SIGKILL_READY";
        if let Ok(directory_path) = std::env::var(CHILD_DIRECTORY) {
            let stage_number = std::env::var(CHILD_STAGE).unwrap().parse::<u8>().unwrap();
            let target_stage = match stage_number {
                0 => PagedManifestPointerStage::BeforeTemporaryCreate,
                1 => PagedManifestPointerStage::AfterTemporarySync,
                2 => PagedManifestPointerStage::AfterRename,
                3 => PagedManifestPointerStage::AfterDirectorySync,
                _ => panic!("invalid pointer stage number"),
            };
            let database_path = PathBuf::from(directory_path).join("tenant.db");
            let identity = StoreIdentity {
                database_uuid: [0x35; 16],
                wal_epoch: 21,
            };
            let mut store = PagedManifestStore::open_unchecked(&database_path, identity).unwrap();
            let _ = store.publish_with_pointer_hook(
                manifest(identity, 2, 1),
                || Ok(()),
                &mut |stage| {
                    if stage == target_stage {
                        fs::write(std::env::var(CHILD_READY).unwrap(), b"ready").unwrap();
                        loop {
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                    Ok(())
                },
            );
            panic!("child publication did not stop at the requested boundary");
        }

        for (stage_number, stage) in [
            PagedManifestPointerStage::BeforeTemporaryCreate,
            PagedManifestPointerStage::AfterTemporarySync,
            PagedManifestPointerStage::AfterRename,
            PagedManifestPointerStage::AfterDirectorySync,
        ]
        .into_iter()
        .enumerate()
        {
            let directory = TestDirectory::new("pointer-sigkill");
            let (identity, _store) = write_manifest_store(&directory);
            let ready_path = directory.0.join(format!("ready-{stage_number}"));
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "blink::paged_manifest::tests::paged_manifest_pointer_publication_survives_sigkill",
                    "--nocapture",
                ])
                .env(CHILD_DIRECTORY, &directory.0)
                .env(CHILD_STAGE, stage_number.to_string())
                .env(CHILD_READY, &ready_path)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ready_path.exists() && Instant::now() < deadline {
                if let Some(status) = child.try_wait().unwrap() {
                    panic!("SIGKILL child exited before {stage:?}: {status}");
                }
                thread::sleep(Duration::from_millis(1));
            }
            assert!(ready_path.exists(), "child did not reach {stage:?}");
            assert_eq!(
                unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
                0
            );
            let status = child.wait().unwrap();
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(status.signal(), Some(libc::SIGKILL));
            let expected_generation = match stage {
                PagedManifestPointerStage::BeforeTemporaryCreate
                | PagedManifestPointerStage::AfterTemporarySync => 1,
                PagedManifestPointerStage::AfterRename
                | PagedManifestPointerStage::AfterDirectorySync => 2,
            };
            let reopened =
                PagedManifestStore::open_unchecked(&directory.database_path(), identity).unwrap();
            assert_eq!(reopened.current().generation, expected_generation);
        }
    }

    #[test]
    fn durable_index_nodes_before_manifest_pointer_survive_sigkill() {
        const CHILD_DIRECTORY: &str = "DODB_INDEX_BEFORE_MANIFEST_SIGKILL_DIRECTORY";
        const CHILD_READY: &str = "DODB_INDEX_BEFORE_MANIFEST_SIGKILL_READY";
        let identity = StoreIdentity {
            database_uuid: [0x35; 16],
            wal_epoch: 21,
        };
        if let Ok(directory_path) = std::env::var(CHILD_DIRECTORY) {
            let directory = TestDirectory(PathBuf::from(directory_path));
            let index = VersionIndex::open(
                &version_index_directory(&directory.database_path()),
                identity,
                resource_manager(),
                71,
                4,
            )
            .unwrap();
            let mut manifest_store =
                PagedManifestStore::open_with_index(&directory.database_path(), identity, &index)
                    .unwrap();
            let initial_node = manifest_store.current().durable_index_root;
            let initial_root = index.root_from_node_reference(initial_node).unwrap();
            let pinned_checkpoint_target = index.pin_root(initial_root).unwrap();
            let page_id = PageId::new(2);
            let mut next_descriptor = index.lookup(initial_root, page_id).unwrap().unwrap();
            next_descriptor.image_lsn = Lsn::new(11);
            next_descriptor.image_crc32c = 100;
            if let VersionLocator::CheckpointImage {
                source,
                offset,
                source_length,
                ..
            } = next_descriptor.locator
            {
                next_descriptor.locator = VersionLocator::CheckpointImage {
                    source,
                    offset,
                    source_length,
                    image_lsn: Lsn::new(11),
                    image_crc32c: 100,
                };
            } else {
                panic!("initial descriptor does not reference a checkpoint image");
            }
            let next_root = index
                .insert(initial_root, page_id, next_descriptor)
                .unwrap();
            assert_eq!(
                index
                    .lookup(pinned_checkpoint_target.root(), page_id)
                    .unwrap()
                    .unwrap()
                    .image_lsn,
                Lsn::ZERO
            );
            let next_durable_root = index.promote_root_to_durable(next_root).unwrap();
            let mut next_manifest = manifest(identity, 2, 1);
            next_manifest.durable_index_root = next_durable_root.node_reference().unwrap();
            let ready_path = PathBuf::from(std::env::var(CHILD_READY).unwrap());
            let _ = manifest_store.publish_with_index_and_hook(next_manifest, &index, |stage| {
                if stage == PagedManifestPointerStage::AfterTemporarySync {
                    fs::write(&ready_path, b"ready")?;
                    loop {
                        thread::sleep(Duration::from_secs(1));
                    }
                }
                Ok(())
            });
            panic!("child publication passed the pre-rename manifest boundary");
        }

        let directory = TestDirectory::new("index-before-manifest-sigkill");
        let (index, _, initial_node) = build_index(&directory, identity);
        let mut initial_manifest = manifest(identity, 1, 0);
        initial_manifest.durable_index_root = initial_node;
        PagedManifestStore::create_with_index(&directory.database_path(), initial_manifest, &index)
            .unwrap();
        drop(index);
        let ready_path = directory.0.join("ready");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "blink::paged_manifest::tests::durable_index_nodes_before_manifest_pointer_survive_sigkill",
                "--nocapture",
            ])
            .env(CHILD_DIRECTORY, &directory.0)
            .env(CHILD_READY, &ready_path)
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready_path.exists() && Instant::now() < deadline {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("SIGKILL child exited before pre-rename pointer boundary: {status}");
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(
            ready_path.exists(),
            "child did not reach pre-rename pointer boundary"
        );
        assert_eq!(
            unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
            0
        );
        let status = child.wait().unwrap();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(libc::SIGKILL));

        let index = VersionIndex::open(
            &version_index_directory(&directory.database_path()),
            identity,
            resource_manager(),
            71,
            4,
        )
        .unwrap();
        let reopened =
            PagedManifestStore::open_with_index(&directory.database_path(), identity, &index)
                .unwrap();
        assert_eq!(reopened.current().generation, 1);
        assert_eq!(reopened.current().durable_index_root, initial_node);
        let selected_root = index
            .root_from_node_reference(reopened.current().durable_index_root)
            .unwrap();
        assert_eq!(
            index
                .lookup(selected_root, PageId::new(2))
                .unwrap()
                .unwrap()
                .image_lsn,
            Lsn::ZERO
        );
        assert!(
            fs::read_dir(version_index_directory(&directory.database_path()))
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().ends_with(".node"))
                .count()
                > 1
        );
    }

    #[test]
    fn synced_delta_chain_before_index_publication_survives_sigkill_as_orphan() {
        const CHILD_DIRECTORY: &str = "DODB_CHAIN_BEFORE_INDEX_SIGKILL_DIRECTORY";
        const CHILD_READY: &str = "DODB_CHAIN_BEFORE_INDEX_SIGKILL_READY";
        let identity = StoreIdentity {
            database_uuid: [0x35; 16],
            wal_epoch: 21,
        };
        if let Ok(directory_path) = std::env::var(CHILD_DIRECTORY) {
            let directory = TestDirectory(PathBuf::from(directory_path));
            let index = VersionIndex::open(
                &version_index_directory(&directory.database_path()),
                identity,
                resource_manager(),
                71,
                4,
            )
            .unwrap();
            let manifest_store =
                PagedManifestStore::open_with_index(&directory.database_path(), identity, &index)
                    .unwrap();
            let initial_root = index
                .root_from_node_reference(manifest_store.current().durable_index_root)
                .unwrap();
            let page_id = PageId::new(2);
            let mut descriptor = index.lookup(initial_root, page_id).unwrap().unwrap();
            let (checkpoint_source, checkpoint_offset, checkpoint_source_length) =
                match descriptor.locator {
                    VersionLocator::CheckpointImage {
                        source,
                        offset,
                        source_length,
                        ..
                    } => (source, offset, source_length),
                    _ => panic!("initial page is not backed by a checkpoint image"),
                };
            let wal_source = SourceIdentity {
                kind: SourceFileKind::WalSegment,
                file_number: 1,
                header_identity: [0x22; 32],
            };
            let frame = WalFrameLocator {
                source: wal_source,
                frame_offset: 200,
                frame_length: 100,
                header_crc32c: 11,
                payload_offset: 232,
                payload_length: 50,
                payload_crc32c: 11,
                committed_prefix_length: 4096,
                commit_frame_offset: 300,
                commit_lsn: Lsn::new(12),
            };
            let chain = DeltaChain::new(
                page_id,
                vec![
                    DeltaChainEntry::CheckpointImage {
                        source: checkpoint_source,
                        offset: checkpoint_offset,
                        source_length: checkpoint_source_length,
                        image_lsn: Lsn::new(10),
                        image_crc32c: 99,
                    },
                    DeltaChainEntry::WalDelta {
                        frame,
                        record_lsn: Lsn::new(11),
                        base_lsn: Lsn::new(10),
                        base_crc32c: 99,
                        image_lsn: Lsn::new(12),
                        image_crc32c: 100,
                    },
                ],
            )
            .unwrap();
            let ready_path = PathBuf::from(std::env::var(CHILD_READY).unwrap());
            let chain_reference = index.delta_chains().write_with_stage(2, &chain, |stage| {
                if stage == "after_delta_chain_directory_sync" {
                    fs::write(&ready_path, b"ready")?;
                    loop {
                        thread::sleep(Duration::from_secs(1));
                    }
                }
                Ok(())
            });
            let chain_reference = chain_reference.unwrap();
            descriptor.token = VersionToken::Wal {
                source_file_number: 1,
                record_lsn: Lsn::new(11),
            };
            descriptor.image_lsn = Lsn::new(12);
            descriptor.image_crc32c = 100;
            descriptor.locator = VersionLocator::WalDelta {
                frame,
                chain: chain_reference,
                depth: 1,
            };
            let staged_root = index.insert(initial_root, page_id, descriptor).unwrap();
            let _staged_durable_root = index.promote_root_to_durable(staged_root).unwrap();
            panic!("child published the index after the sidecar crash boundary");
        }

        let directory = TestDirectory::new("chain-before-index-sigkill");
        let (index, _, initial_node) = build_index(&directory, identity);
        let mut initial_manifest = manifest(identity, 1, 0);
        initial_manifest.durable_index_root = initial_node;
        PagedManifestStore::create_with_index(&directory.database_path(), initial_manifest, &index)
            .unwrap();
        drop(index);
        let ready_path = directory.0.join("ready");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "blink::paged_manifest::tests::synced_delta_chain_before_index_publication_survives_sigkill_as_orphan",
                "--nocapture",
            ])
            .env(CHILD_DIRECTORY, &directory.0)
            .env(CHILD_READY, &ready_path)
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready_path.exists() && Instant::now() < deadline {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("SIGKILL child exited before delta-chain directory sync: {status}");
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(ready_path.exists(), "child did not reach delta-chain sync");
        assert_eq!(
            unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
            0
        );
        let status = child.wait().unwrap();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(libc::SIGKILL));

        let index = VersionIndex::open(
            &version_index_directory(&directory.database_path()),
            identity,
            resource_manager(),
            71,
            4,
        )
        .unwrap();
        let reopened =
            PagedManifestStore::open_with_index(&directory.database_path(), identity, &index)
                .unwrap();
        assert_eq!(reopened.current().generation, 1);
        assert_eq!(reopened.current().durable_index_root, initial_node);
        let selected_root = index
            .root_from_node_reference(reopened.current().durable_index_root)
            .unwrap();
        let selected_descriptor = index
            .lookup(selected_root, PageId::new(2))
            .unwrap()
            .unwrap();
        assert_eq!(selected_descriptor.image_lsn, Lsn::ZERO);
        assert!(matches!(
            selected_descriptor.locator,
            VersionLocator::CheckpointImage { .. }
        ));
        let chain_path = version_index_directory(&directory.database_path())
            .join("version-chain-00000000000000000001.bin");
        let chain_bytes = fs::read(&chain_path).unwrap();
        let checksum = chain_bytes[chain_bytes.len() - 32..].try_into().unwrap();
        assert_eq!(
            index
                .delta_chains()
                .read(
                    crate::blink::version_chain::DeltaChainRef {
                        file_number: 1,
                        offset: 0,
                        checksum,
                    },
                    PageId::new(2),
                )
                .unwrap()
                .entries
                .len(),
            2
        );
    }

    #[test]
    fn marker_pointer_manifest_and_identity_corruption_fail_closed() {
        let directory = TestDirectory::new("corruption");
        let (identity, store) = write_manifest_store(&directory);
        assert!(
            PagedManifestStore::open_unchecked(
                &directory.database_path(),
                StoreIdentity {
                    database_uuid: [0x36; 16],
                    wal_epoch: 21,
                },
            )
            .is_err()
        );
        let manifest_file = sidecar_directory(&directory.database_path()).join(format!(
            "blink-paged-manifest-{:020}.bin",
            store.current_file_number()
        ));
        let mut manifest_bytes = fs::read(&manifest_file).unwrap();
        assert_eq!(manifest_bytes.len(), MANIFEST_BYTES);
        manifest_bytes[120] ^= 1;
        fs::write(&manifest_file, manifest_bytes).unwrap();
        assert!(PagedManifestStore::open_unchecked(&directory.database_path(), identity).is_err());
    }

    #[test]
    fn truncated_marker_pointer_and_manifest_fail_closed() {
        let marker_directory = TestDirectory::new("truncated-marker");
        let (marker_identity, marker_store) = write_manifest_store(&marker_directory);
        fs::write(marker_store.marker_path(), [0u8; 17]).unwrap();
        assert!(
            PagedManifestStore::open_unchecked(&marker_directory.database_path(), marker_identity)
                .is_err()
        );

        let pointer_directory = TestDirectory::new("truncated-pointer");
        let (pointer_identity, pointer_store) = write_manifest_store(&pointer_directory);
        fs::write(&pointer_store.pointer_path, [0u8; 17]).unwrap();
        assert!(
            PagedManifestStore::open_unchecked(
                &pointer_directory.database_path(),
                pointer_identity
            )
            .is_err()
        );

        let manifest_directory = TestDirectory::new("truncated-manifest");
        let (manifest_identity, manifest_store) = write_manifest_store(&manifest_directory);
        let manifest_file = sidecar_directory(&manifest_directory.database_path()).join(format!(
            "blink-paged-manifest-{:020}.bin",
            manifest_store.current_file_number()
        ));
        fs::write(&manifest_file, [0u8; MANIFEST_BYTES - 1]).unwrap();
        assert!(
            PagedManifestStore::open_unchecked(
                &manifest_directory.database_path(),
                manifest_identity
            )
            .is_err()
        );
    }

    #[test]
    fn unknown_pointer_version_fails_closed() {
        let directory = TestDirectory::new("unknown-version");
        let (identity, store) = write_manifest_store(&directory);
        let pointer_path = sidecar_directory(&directory.database_path()).join(CURRENT_POINTER_NAME);
        let mut pointer_bytes = fs::read(&pointer_path).unwrap();
        pointer_bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
        let checksum = blake3::hash(&pointer_bytes[..56]);
        pointer_bytes[56..88].copy_from_slice(checksum.as_bytes());
        fs::write(&pointer_path, pointer_bytes).unwrap();
        assert!(PagedManifestStore::open_unchecked(&directory.database_path(), identity).is_err());
        assert_eq!(store.current().generation, 1);
    }

    #[test]
    fn compatibility_marker_blocks_resident_path_open_before_file_creation() {
        let directory = TestDirectory::new("compatibility");
        let _ = write_manifest_store(&directory);
        let database_path = directory.database_path();
        let files_before_open = snapshot_files(&directory.0);
        let config = DatabaseConfig::default();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_path(&database_path, config.clone())
                .is_err()
        );
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &database_path,
                config,
                1024 * 1024
            )
            .is_err()
        );
        assert!(!database_path.exists());
        assert_eq!(snapshot_files(&directory.0), files_before_open);
    }

    #[test]
    fn manifest_rejects_invalid_fields_before_writing() {
        let directory = TestDirectory::new("invalid");
        let (identity, store) = write_manifest_store(&directory);
        let mut invalid_manifest = manifest(identity, 2, 0);
        invalid_manifest.previous_generation = 2;
        assert!(encode_manifest(&invalid_manifest).is_err());
        assert_eq!(store.current(), manifest(identity, 1, 0));
    }

    #[test]
    fn durable_manifest_rejects_spill_nodes_and_reopens_synced_roots() {
        let directory = TestDirectory::new("durable-index");
        let identity = StoreIdentity {
            database_uuid: [0x37; 16],
            wal_epoch: 23,
        };
        let (index, spill_reference, durable_reference) = build_index(&directory, identity);
        let mut candidate = manifest(identity, 1, 0);
        candidate.durable_index_root = spill_reference;
        assert!(
            PagedManifestStore::create_with_index(&directory.database_path(), candidate, &index)
                .is_err()
        );

        candidate.durable_index_root = durable_reference;
        let store =
            PagedManifestStore::create_with_index(&directory.database_path(), candidate, &index)
                .unwrap();
        let reopened =
            PagedManifestStore::open_with_index(&directory.database_path(), identity, &index)
                .unwrap();
        assert_eq!(reopened.current(), candidate);
        assert_eq!(reopened.current_file_number(), store.current_file_number());
    }
}
