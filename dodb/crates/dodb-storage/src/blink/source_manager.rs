use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use dodb_core::{Error, Result};

use super::paged_manifest::PagedManifestStore;
use super::resource::{BlinkResourceManager, Reservation, ReservationClass, ResourceCategory};
use super::version_index::{
    SourceFileKind, SourceIdentity, SourceReachability, StoreIdentity, VersionIndex,
    VersionIndexRoot, VersionIndexRootPin,
};

static GC_RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const ROOT_REGISTRATION_CHARGE_BYTES: u64 = 256;
const GC_BASE_RESERVATION_BYTES: u64 = 4096;

struct RootEntry {
    root: VersionIndexRoot,
    _pin: VersionIndexRootPin,
    _reservation: Reservation,
}

#[derive(Default)]
struct SourceState {
    roots: HashMap<u64, RootEntry>,
    active_handles: HashMap<SourceIdentity, usize>,
    next_root_id: u64,
    garbage_collection_active: bool,
}

struct SourceManagerInner {
    store_identity: StoreIdentity,
    checkpoint_directory: PathBuf,
    wal_directory: PathBuf,
    scratch_directory: PathBuf,
    resources: BlinkResourceManager,
    tenant_id: u64,
    maximum_open_handles: usize,
    maximum_roots: usize,
    handle_charge_bytes: u64,
    gc_reservation_bytes: u64,
    state: Mutex<SourceState>,
}

#[derive(Clone)]
pub struct SourceManager {
    inner: Arc<SourceManagerInner>,
}

pub struct SourceManagerConfig {
    pub store_identity: StoreIdentity,
    pub checkpoint_directory: PathBuf,
    pub wal_directory: PathBuf,
    pub scratch_directory: PathBuf,
    pub tenant_id: u64,
    pub maximum_open_handles: usize,
    pub maximum_roots: usize,
    pub handle_charge_bytes: u64,
    pub gc_reservation_bytes: u64,
}

pub struct SourceRootLease {
    manager: SourceManager,
    root_id: u64,
}

pub struct SourceLease {
    manager: SourceManager,
    identity: SourceIdentity,
    file: File,
    _reservation: Reservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceCandidate {
    pub identity: SourceIdentity,
    pub header_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SourceGcReport {
    pub candidates_examined: u64,
    pub candidate_bytes: u64,
    pub protected_sources: u64,
    pub protected_bytes: u64,
    pub leased_sources: u64,
    pub leased_bytes: u64,
    pub unlinked_sources: u64,
    pub unlinked_bytes: u64,
    pub unlinked_open_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceGcFaultPoint {
    BeforeUnlink,
    AfterUnlink,
}

struct GcGate {
    manager: SourceManager,
}

struct GcScratch {
    path: PathBuf,
}

impl SourceManager {
    pub fn new(config: SourceManagerConfig, resources: BlinkResourceManager) -> Result<Self> {
        if config.maximum_open_handles == 0
            || config.maximum_roots == 0
            || config.handle_charge_bytes == 0
            || config.gc_reservation_bytes == 0
        {
            return Err(Error::invalid_input(
                "paged source manager limits must be nonzero",
            ));
        }
        let root_snapshot_bytes = u64::try_from(config.maximum_roots)
            .ok()
            .and_then(|root_count| {
                root_count.checked_mul(std::mem::size_of::<VersionIndexRoot>() as u64)
            })
            .and_then(|root_bytes| GC_BASE_RESERVATION_BYTES.checked_add(root_bytes))
            .ok_or_else(|| Error::invalid_input("paged source GC reservation overflows"))?;
        if config.gc_reservation_bytes < root_snapshot_bytes {
            return Err(Error::invalid_input(
                "paged source GC reservation cannot fit its root snapshot",
            ));
        }
        fs::create_dir_all(&config.scratch_directory)?;
        Ok(Self {
            inner: Arc::new(SourceManagerInner {
                store_identity: config.store_identity,
                checkpoint_directory: config.checkpoint_directory,
                wal_directory: config.wal_directory,
                scratch_directory: config.scratch_directory,
                resources,
                tenant_id: config.tenant_id,
                maximum_open_handles: config.maximum_open_handles,
                maximum_roots: config.maximum_roots,
                handle_charge_bytes: config.handle_charge_bytes,
                gc_reservation_bytes: config.gc_reservation_bytes,
                state: Mutex::new(SourceState::default()),
            }),
        })
    }

    pub fn pin_root(
        &self,
        index: &VersionIndex,
        root: VersionIndexRoot,
    ) -> Result<SourceRootLease> {
        if index.store_identity() != self.inner.store_identity {
            return Err(Error::corruption(
                "paged source root belongs to a different store identity",
            ));
        }
        let pin = index.pin_root(root)?;
        let root_reservation = self
            .inner
            .resources
            .reserve(
                self.inner.tenant_id,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                ROOT_REGISTRATION_CHARGE_BYTES,
            )
            .map_err(|_| Error::overloaded("paged source root budget is exhausted"))?;
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
        if state.garbage_collection_active {
            return Err(Error::overloaded(
                "paged source root admission is paused during source collection",
            ));
        }
        if state.roots.len() >= self.inner.maximum_roots {
            return Err(Error::overloaded(
                "paged source root pin limit is exhausted",
            ));
        }
        let root_id = state.next_root_id;
        state.next_root_id = state
            .next_root_id
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("paged source root id exhausted"))?;
        state.roots.insert(
            root_id,
            RootEntry {
                root,
                _pin: pin,
                _reservation: root_reservation,
            },
        );
        Ok(SourceRootLease {
            manager: self.clone(),
            root_id,
        })
    }

    pub fn pin_manifest_root(
        &self,
        index: &VersionIndex,
        manifest_store: &PagedManifestStore,
    ) -> Result<SourceRootLease> {
        let manifest = manifest_store.current();
        let root = index.root_from_node_reference(manifest.durable_index_root)?;
        self.pin_root(index, root)
    }

    pub fn acquire(&self, identity: SourceIdentity, header_bytes: usize) -> Result<SourceLease> {
        if identity.file_number == 0 || header_bytes == 0 || header_bytes > 4096 {
            return Err(Error::invalid_input("paged source identity is invalid"));
        }
        let reservation = self
            .inner
            .resources
            .reserve(
                self.inner.tenant_id,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                self.inner.handle_charge_bytes,
            )
            .map_err(|_| Error::overloaded("paged source handle budget is exhausted"))?;
        {
            let mut state = self
                .inner
                .state
                .lock()
                .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
            if state.garbage_collection_active {
                return Err(Error::overloaded(
                    "paged source reads are paused during source collection",
                ));
            }
            let active_handles = state.active_handles.values().sum::<usize>();
            if active_handles >= self.inner.maximum_open_handles {
                return Err(Error::overloaded("paged source handle limit is exhausted"));
            }
            *state.active_handles.entry(identity).or_default() += 1;
        }
        let path = self.source_path(identity);
        let open_result = open_verified_source(&path, identity, header_bytes);
        let file = match open_result {
            Ok(file) => file,
            Err(error) => {
                self.release_handle(identity);
                return Err(error);
            }
        };
        Ok(SourceLease {
            manager: self.clone(),
            identity,
            file,
            _reservation: reservation,
        })
    }

    pub fn collect_garbage<I, F>(
        &self,
        index: &VersionIndex,
        candidates: I,
        mut fault: F,
    ) -> Result<SourceGcReport>
    where
        I: IntoIterator<Item = SourceCandidate>,
        F: FnMut(SourceGcFaultPoint) -> Result<()>,
    {
        let _gc_reservation = self
            .inner
            .resources
            .reserve(
                self.inner.tenant_id,
                ResourceCategory::CheckpointValidation,
                ReservationClass::Progress,
                self.inner.gc_reservation_bytes,
            )
            .map_err(|_| Error::overloaded("paged source GC progress budget is exhausted"))?;
        let roots = {
            let mut state = self
                .inner
                .state
                .lock()
                .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
            if state.garbage_collection_active {
                return Err(Error::overloaded(
                    "paged source collection is already active",
                ));
            }
            state.garbage_collection_active = true;
            state
                .roots
                .values()
                .map(|entry| entry.root)
                .collect::<Vec<_>>()
        };
        let _gate = GcGate {
            manager: self.clone(),
        };
        let run_id = GC_RUN_SEQUENCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map_err(|_| Error::invalid_input("paged source collection id exhausted"))?;
        let scratch_path = self
            .inner
            .scratch_directory
            .join(format!("source-gc-{}-{run_id:020}", std::process::id()));
        fs::create_dir(&scratch_path)?;
        let _scratch = GcScratch {
            path: scratch_path.clone(),
        };
        for root in roots {
            index.visit_sources(root, &mut |source| match source {
                SourceReachability::Exact(identity) => {
                    let marker_path = scratch_path.join(source_mark_name(identity));
                    match OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(marker_path)
                    {
                        Ok(mut marker) => {
                            marker.write_all(&identity.file_number.to_le_bytes())?;
                            marker.write_all(&identity.header_identity)?;
                            Ok(())
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                        Err(error) => Err(error.into()),
                    }
                }
                SourceReachability::AllWalSegments => {
                    OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(scratch_path.join("retain-all-wal-segments"))?;
                    Ok(())
                }
            })?;
        }

        let mut report = SourceGcReport::default();
        for candidate in candidates {
            report.candidates_examined = report
                .candidates_examined
                .checked_add(1)
                .ok_or_else(|| Error::invalid_input("paged source GC counter overflow"))?;
            validate_candidate(candidate)?;
            let path = self.source_path(candidate.identity);
            let source_is_marked = scratch_path
                .join(source_mark_name(candidate.identity))
                .exists()
                || (candidate.identity.kind == SourceFileKind::WalSegment
                    && scratch_path.join("retain-all-wal-segments").exists());
            let metadata = match fs::metadata(&path) {
                Ok(metadata) if metadata.is_file() => metadata,
                Ok(_) => continue,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound && source_is_marked => {
                    return Err(Error::corruption("reachable paged source is missing"));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            report.candidate_bytes = report
                .candidate_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| Error::invalid_input("paged source GC byte count overflow"))?;
            if source_is_marked {
                report.protected_sources = report
                    .protected_sources
                    .checked_add(1)
                    .ok_or_else(|| Error::invalid_input("paged source GC counter overflow"))?;
                report.protected_bytes = report
                    .protected_bytes
                    .checked_add(metadata.len())
                    .ok_or_else(|| Error::invalid_input("paged source GC byte count overflow"))?;
                continue;
            }
            {
                let state = self
                    .inner
                    .state
                    .lock()
                    .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
                if state
                    .active_handles
                    .get(&candidate.identity)
                    .copied()
                    .unwrap_or(0)
                    > 0
                {
                    report.leased_sources = report
                        .leased_sources
                        .checked_add(1)
                        .ok_or_else(|| Error::invalid_input("paged source GC counter overflow"))?;
                    report.leased_bytes = report
                        .leased_bytes
                        .checked_add(metadata.len())
                        .ok_or_else(|| {
                            Error::invalid_input("paged source GC byte count overflow")
                        })?;
                    continue;
                }
            }
            let _verified =
                open_verified_source(&path, candidate.identity, candidate.header_bytes)?;
            drop(_verified);
            fault(SourceGcFaultPoint::BeforeUnlink)?;
            fs::remove_file(&path)?;
            if let Some(parent) = path.parent() {
                File::open(parent)?.sync_all()?;
            }
            report.unlinked_sources = report
                .unlinked_sources
                .checked_add(1)
                .ok_or_else(|| Error::invalid_input("paged source GC counter overflow"))?;
            report.unlinked_bytes = report
                .unlinked_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| Error::invalid_input("paged source GC byte count overflow"))?;
            fault(SourceGcFaultPoint::AfterUnlink)?;
        }
        Ok(report)
    }

    fn source_path(&self, identity: SourceIdentity) -> PathBuf {
        match identity.kind {
            SourceFileKind::CheckpointPack => self
                .inner
                .checkpoint_directory
                .join(format!("base-{:020}.pages", identity.file_number)),
            SourceFileKind::WalSegment => self
                .inner
                .wal_directory
                .join(format!("segment-{:020}.wal", identity.file_number)),
        }
    }

    fn release_handle(&self, identity: SourceIdentity) {
        if let Ok(mut state) = self.inner.state.lock()
            && let Some(active) = state.active_handles.get_mut(&identity)
        {
            *active -= 1;
            if *active == 0 {
                state.active_handles.remove(&identity);
            }
        }
    }
}

impl SourceLease {
    pub fn identity(&self) -> SourceIdentity {
        self.identity
    }

    pub fn read_exact_at(&mut self, offset: u64, output: &mut [u8]) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(output)?;
        Ok(())
    }
}

impl Drop for SourceLease {
    fn drop(&mut self) {
        self.manager.release_handle(self.identity);
    }
}

impl Drop for SourceRootLease {
    fn drop(&mut self) {
        if let Ok(mut state) = self.manager.inner.state.lock() {
            state.roots.remove(&self.root_id);
        }
    }
}

impl Drop for GcGate {
    fn drop(&mut self) {
        if let Ok(mut state) = self.manager.inner.state.lock() {
            state.garbage_collection_active = false;
        }
    }
}

impl Drop for GcScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn validate_candidate(candidate: SourceCandidate) -> Result<()> {
    if candidate.identity.file_number == 0
        || candidate.header_bytes == 0
        || candidate.header_bytes > 4096
    {
        return Err(Error::invalid_input("paged source candidate is invalid"));
    }
    Ok(())
}

fn open_verified_source(
    path: &Path,
    identity: SourceIdentity,
    header_bytes: usize,
) -> Result<File> {
    let mut file = File::open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Error::corruption("reachable paged source is missing")
        } else {
            Error::from(error)
        }
    })?;
    if file.metadata()?.len() < header_bytes as u64 {
        return Err(Error::corruption("paged source header is truncated"));
    }
    let mut header = vec![0u8; header_bytes];
    file.read_exact(&mut header)?;
    if blake3::hash(&header).as_bytes() != &identity.header_identity {
        return Err(Error::corruption("paged source header identity mismatch"));
    }
    Ok(file)
}

fn source_mark_name(identity: SourceIdentity) -> String {
    let mut bytes = [0u8; 9];
    bytes[0] = identity.kind as u8;
    bytes[1..9].copy_from_slice(&identity.file_number.to_le_bytes());
    format!("source-{}", blake3::hash(&bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blink::paged_manifest::{PagedManifest, PagedManifestStore};
    use crate::blink::version_index::{
        SourceFileKind, StoreIdentity, VersionDescriptor, VersionLocator, VersionToken,
        WalFrameLocator,
    };
    use dodb_core::{Lsn, PageId};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "dodb-source-manager-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn test_resources() -> BlinkResourceManager {
        BlinkResourceManager::new(super::super::resource::ResourceLimits {
            global_bytes: 1 << 20,
            per_tenant_bytes: 1 << 20,
            progress_pool_bytes: 1 << 17,
            max_tenants: 4,
            max_pending_waiters: 4,
            global_category_bytes: [1 << 20; 7],
            per_tenant_category_bytes: [1 << 20; 7],
        })
        .unwrap()
    }

    fn test_index(directory: &Path) -> VersionIndex {
        let index_directory = directory.join("index");
        VersionIndex::open(
            &index_directory,
            StoreIdentity {
                database_uuid: [19; 16],
                wal_epoch: 3,
            },
            test_resources(),
            9,
            2,
        )
        .unwrap()
    }

    fn source_identity(kind: SourceFileKind, file_number: u64, header: &[u8]) -> SourceIdentity {
        SourceIdentity {
            kind,
            file_number,
            header_identity: *blake3::hash(header).as_bytes(),
        }
    }

    fn manager(directory: &Path) -> SourceManager {
        SourceManager::new(
            SourceManagerConfig {
                store_identity: StoreIdentity {
                    database_uuid: [19; 16],
                    wal_epoch: 3,
                },
                checkpoint_directory: directory.join("packs"),
                wal_directory: directory.join("segments"),
                scratch_directory: directory.join("scratch"),
                tenant_id: 9,
                maximum_open_handles: 2,
                maximum_roots: 4,
                handle_charge_bytes: 128,
                gc_reservation_bytes: 8192,
            },
            test_resources(),
        )
        .unwrap()
    }

    fn insert_checkpoint_page(
        index: &VersionIndex,
        root: VersionIndexRoot,
        source: SourceIdentity,
        source_length: u64,
        image_lsn: u64,
        image_crc32c: u32,
    ) -> VersionIndexRoot {
        index
            .insert(
                root,
                PageId::new(2),
                VersionDescriptor {
                    page_id: PageId::new(2),
                    token: VersionToken::Bootstrap {
                        page_id: PageId::new(2),
                    },
                    image_lsn: Lsn::new(image_lsn),
                    image_crc32c,
                    locator: VersionLocator::CheckpointImage {
                        source,
                        offset: 4096,
                        source_length,
                        image_lsn: Lsn::new(image_lsn),
                        image_crc32c,
                    },
                },
            )
            .unwrap()
    }

    fn write_checkpoint_page(
        manager: &SourceManager,
        file_number: u64,
        header_byte: u8,
        image_byte: u8,
    ) -> (SourceIdentity, usize) {
        let header = vec![header_byte; 4096];
        let identity = source_identity(SourceFileKind::CheckpointPack, file_number, &header);
        let path = manager.source_path(identity);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(&header).unwrap();
        file.write_all(&vec![image_byte; 4096]).unwrap();
        (identity, header.len())
    }

    fn write_source(manager: &SourceManager, identity: SourceIdentity, header: &[u8]) -> PathBuf {
        let path = manager.source_path(identity);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(&path).unwrap();
        file.write_all(header).unwrap();
        file.write_all(b"payload").unwrap();
        path
    }

    #[test]
    fn indexed_and_pinned_sources_are_retained_until_every_root_drops() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let checkpoint_header = b"checkpoint-header";
        let wal_header = b"wal-header";
        let checkpoint = source_identity(SourceFileKind::CheckpointPack, 1, checkpoint_header);
        let wal = source_identity(SourceFileKind::WalSegment, 2, wal_header);
        write_source(&manager, checkpoint, checkpoint_header);
        write_source(&manager, wal, wal_header);
        let mut root = index.empty_root();
        root = index
            .insert(
                root,
                PageId::new(2),
                VersionDescriptor {
                    page_id: PageId::new(2),
                    token: VersionToken::Wal {
                        source_file_number: wal.file_number,
                        record_lsn: Lsn::new(10),
                    },
                    image_lsn: Lsn::new(10),
                    image_crc32c: 4,
                    locator: VersionLocator::WalDelta {
                        frame: WalFrameLocator {
                            source: wal,
                            frame_offset: 32,
                            frame_length: 128,
                            header_crc32c: 1,
                            payload_offset: 48,
                            payload_length: 64,
                            payload_crc32c: 2,
                            committed_prefix_length: 256,
                            commit_frame_offset: 200,
                            commit_lsn: Lsn::new(12),
                        },
                        base_lsn: Lsn::new(8),
                        base_crc32c: 3,
                        base: crate::blink::version_index::BaseFrameLocator {
                            source: checkpoint,
                            frame_offset: 64,
                            frame_length: 4096,
                            header_crc32c: 5,
                            image_lsn: Lsn::new(8),
                            image_crc32c: 3,
                        },
                        depth: 1,
                    },
                },
            )
            .unwrap();
        let durable_root_lease = manager.pin_root(&index, root).unwrap();
        let checkpoint_candidate = SourceCandidate {
            identity: checkpoint,
            header_bytes: checkpoint_header.len(),
        };
        let wal_candidate = SourceCandidate {
            identity: wal,
            header_bytes: wal_header.len(),
        };
        let report = manager
            .collect_garbage(&index, [checkpoint_candidate, wal_candidate], |_| Ok(()))
            .unwrap();
        assert_eq!(report.protected_sources, 2);
        assert_eq!(report.unlinked_sources, 0);
        assert_eq!(report.unlinked_open_bytes, 0);
        drop(durable_root_lease);
        let report = manager
            .collect_garbage(&index, [checkpoint_candidate, wal_candidate], |_| Ok(()))
            .unwrap();
        assert_eq!(report.unlinked_sources, 2);
        assert!(!manager.source_path(checkpoint).exists());
        assert!(!manager.source_path(wal).exists());
    }

    #[test]
    fn delta_roots_retain_earlier_wal_segments_as_a_conservative_dependency_run() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let oldest_header = b"oldest-wal";
        let base_header = b"base-wal";
        let delta_header = b"delta-wal";
        let oldest = source_identity(SourceFileKind::WalSegment, 1, oldest_header);
        let base = source_identity(SourceFileKind::WalSegment, 2, base_header);
        let delta = source_identity(SourceFileKind::WalSegment, 3, delta_header);
        write_source(&manager, oldest, oldest_header);
        write_source(&manager, base, base_header);
        write_source(&manager, delta, delta_header);
        let root = index
            .insert(
                index.empty_root(),
                PageId::new(2),
                VersionDescriptor {
                    page_id: PageId::new(2),
                    token: VersionToken::Wal {
                        source_file_number: delta.file_number,
                        record_lsn: Lsn::new(30),
                    },
                    image_lsn: Lsn::new(30),
                    image_crc32c: 3,
                    locator: VersionLocator::WalDelta {
                        frame: WalFrameLocator {
                            source: delta,
                            frame_offset: 64,
                            frame_length: 96,
                            header_crc32c: 1,
                            payload_offset: 80,
                            payload_length: 64,
                            payload_crc32c: 2,
                            committed_prefix_length: 256,
                            commit_frame_offset: 192,
                            commit_lsn: Lsn::new(32),
                        },
                        base_lsn: Lsn::new(20),
                        base_crc32c: 4,
                        base: crate::blink::version_index::BaseFrameLocator {
                            source: base,
                            frame_offset: 32,
                            frame_length: 96,
                            header_crc32c: 5,
                            image_lsn: Lsn::new(20),
                            image_crc32c: 4,
                        },
                        depth: 2,
                    },
                },
            )
            .unwrap();
        let root_lease = manager.pin_root(&index, root).unwrap();
        let candidates = [oldest, base, delta].map(|identity| SourceCandidate {
            identity,
            header_bytes: match identity.file_number {
                1 => oldest_header.len(),
                2 => base_header.len(),
                _ => delta_header.len(),
            },
        });
        let report = manager
            .collect_garbage(&index, candidates, |_| Ok(()))
            .unwrap();
        assert_eq!(report.protected_sources, 3);
        drop(root_lease);
        let report = manager
            .collect_garbage(&index, candidates, |_| Ok(()))
            .unwrap();
        assert_eq!(report.unlinked_sources, 3);
    }

    #[test]
    fn manifest_roots_keep_old_reader_sources_across_checkpoint_publication() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let identity = index.store_identity();
        let (old_source, header_bytes) = write_checkpoint_page(&manager, 1, 11, 71);
        let old_root = insert_checkpoint_page(&index, index.empty_root(), old_source, 8192, 1, 7);
        let durable_old_root = index.promote_root_to_durable(old_root).unwrap();
        let manifest_path = directory.path().join("tenant.db");
        let mut manifest_store = PagedManifestStore::create_with_index(
            &manifest_path,
            PagedManifest {
                identity,
                generation: 1,
                tree_root: PageId::new(2),
                free_list_root: None,
                high_water_page_id: PageId::new(2),
                durable_index_root: durable_old_root.node_reference().unwrap(),
                source_catalog_root: None,
                recovery_wal_file_number: 1,
                recovery_offset: 4096,
                committed_prefix_lsn: Lsn::new(1),
                previous_generation: 0,
            },
            &index,
        )
        .unwrap();
        let old_reader = manager.pin_manifest_root(&index, &manifest_store).unwrap();
        let mut old_source_lease = manager.acquire(old_source, header_bytes).unwrap();
        let mut old_image = vec![0; 4096];
        old_source_lease
            .read_exact_at(4096, &mut old_image)
            .unwrap();
        assert!(old_image.iter().all(|byte| *byte == 71));
        drop(old_source_lease);
        let mut reopened_old_source = manager.acquire(old_source, header_bytes).unwrap();
        reopened_old_source
            .read_exact_at(4096, &mut old_image)
            .unwrap();
        assert!(old_image.iter().all(|byte| *byte == 71));
        drop(reopened_old_source);

        let (new_source, _) = write_checkpoint_page(&manager, 2, 12, 72);
        let new_root = insert_checkpoint_page(&index, old_root, new_source, 8192, 2, 8);
        let durable_new_root = index.promote_root_to_durable(new_root).unwrap();
        manifest_store
            .publish_with_index(
                PagedManifest {
                    identity,
                    generation: 2,
                    tree_root: PageId::new(2),
                    free_list_root: None,
                    high_water_page_id: PageId::new(2),
                    durable_index_root: durable_new_root.node_reference().unwrap(),
                    source_catalog_root: None,
                    recovery_wal_file_number: 1,
                    recovery_offset: 4096,
                    committed_prefix_lsn: Lsn::new(2),
                    previous_generation: 1,
                },
                &index,
            )
            .unwrap();
        drop(manifest_store);
        let manifest_store =
            PagedManifestStore::open_with_index(&manifest_path, identity, &index).unwrap();
        let current_reader = manager.pin_manifest_root(&index, &manifest_store).unwrap();
        let candidates = [old_source, new_source].map(|source| SourceCandidate {
            identity: source,
            header_bytes,
        });
        let report = manager
            .collect_garbage(&index, candidates, |_| Ok(()))
            .unwrap();
        assert_eq!(report.protected_sources, 2);
        drop(old_reader);
        let report = manager
            .collect_garbage(&index, candidates, |_| Ok(()))
            .unwrap();
        assert_eq!(report.protected_sources, 1);
        assert_eq!(report.unlinked_sources, 1);
        assert!(!manager.source_path(old_source).exists());
        assert!(manager.source_path(new_source).exists());
        drop(current_reader);
    }

    #[test]
    fn active_read_lease_blocks_retirement_and_reopen_checks_exact_header() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let header = b"stable-header";
        let identity = source_identity(SourceFileKind::WalSegment, 4, header);
        write_source(&manager, identity, header);
        let lease = manager.acquire(identity, header.len()).unwrap();
        let report = manager
            .collect_garbage(
                &index,
                [SourceCandidate {
                    identity,
                    header_bytes: header.len(),
                }],
                |_| Ok(()),
            )
            .unwrap();
        assert_eq!(report.leased_sources, 1);
        assert!(report.leased_bytes > 0);
        assert_eq!(report.unlinked_open_bytes, 0);
        drop(lease);
        let altered_header = b"changed-header";
        let wrong_identity = SourceIdentity {
            header_identity: *blake3::hash(altered_header).as_bytes(),
            ..identity
        };
        assert!(
            manager
                .acquire(wrong_identity, altered_header.len())
                .is_err()
        );
        assert!(manager.source_path(identity).exists());
        assert!(
            manager
                .collect_garbage(
                    &index,
                    [SourceCandidate {
                        identity: wrong_identity,
                        header_bytes: altered_header.len(),
                    }],
                    |_| Ok(()),
                )
                .is_err()
        );
        assert!(manager.source_path(identity).exists());
    }

    #[test]
    fn load_acquisition_racing_retirement_never_reads_an_unlinked_source() {
        let directory = TestDirectory::new();
        let index = Arc::new(test_index(directory.path()));
        let manager = manager(directory.path());
        let header = b"retirement-race";
        let identity = source_identity(SourceFileKind::WalSegment, 9, header);
        write_source(&manager, identity, header);
        let candidate = SourceCandidate {
            identity,
            header_bytes: header.len(),
        };
        let start_barrier = Arc::new(std::sync::Barrier::new(3));
        let loader_manager = manager.clone();
        let loader_barrier = Arc::clone(&start_barrier);
        let loader = std::thread::spawn(move || {
            loader_barrier.wait();
            loader_manager.acquire(identity, header.len())
        });
        let collector_manager = manager.clone();
        let collector_barrier = Arc::clone(&start_barrier);
        let collector_index = Arc::clone(&index);
        let collector = std::thread::spawn(move || {
            collector_barrier.wait();
            collector_manager.collect_garbage(&collector_index, [candidate], |_| Ok(()))
        });
        start_barrier.wait();
        let lease_result = loader.join().unwrap();
        let report = collector.join().unwrap().unwrap();
        match lease_result {
            Ok(lease) => {
                assert_eq!(report.leased_sources, 1);
                assert!(manager.source_path(identity).exists());
                let mut loaded_header = vec![0; header.len()];
                let mut lease = lease;
                lease.read_exact_at(0, &mut loaded_header).unwrap();
                assert_eq!(loaded_header, header);
                drop(lease);
                let followup = manager
                    .collect_garbage(&index, [candidate], |_| Ok(()))
                    .unwrap();
                assert_eq!(followup.unlinked_sources, 1);
            }
            Err(_) => {
                assert_eq!(report.unlinked_sources, 1);
                assert!(!manager.source_path(identity).exists());
            }
        }
    }

    #[test]
    fn source_gc_preserves_reachable_files_across_unlink_fault_boundaries() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let header = b"wal-prefix";
        let identity = source_identity(SourceFileKind::WalSegment, 5, header);
        write_source(&manager, identity, header);
        let candidate = SourceCandidate {
            identity,
            header_bytes: header.len(),
        };
        let before_fault = manager.collect_garbage(&index, [candidate], |point| {
            if point == SourceGcFaultPoint::BeforeUnlink {
                return Err(Error::recovery("injected pre-unlink failure"));
            }
            Ok(())
        });
        assert!(before_fault.is_err());
        assert!(manager.source_path(identity).exists());
        let after_fault = manager.collect_garbage(&index, [candidate], |point| {
            if point == SourceGcFaultPoint::AfterUnlink {
                return Err(Error::recovery("injected post-unlink failure"));
            }
            Ok(())
        });
        assert!(after_fault.is_err());
        assert!(!manager.source_path(identity).exists());
        let foreign_path = directory.path().join("segments/foreign-retained.wal");
        fs::write(&foreign_path, b"unrecognized source artifact").unwrap();
        assert!(foreign_path.exists());
    }

    #[test]
    fn source_handles_and_roots_have_explicit_limits() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let header = b"wal-header";
        let first = source_identity(SourceFileKind::WalSegment, 6, header);
        let second = source_identity(SourceFileKind::WalSegment, 7, header);
        write_source(&manager, first, header);
        write_source(&manager, second, header);
        let first_lease = manager.acquire(first, header.len()).unwrap();
        let second_lease = manager.acquire(second, header.len()).unwrap();
        let third = source_identity(SourceFileKind::WalSegment, 8, header);
        write_source(&manager, third, header);
        assert!(manager.acquire(third, header.len()).is_err());
        drop(first_lease);
        drop(second_lease);
        let root = index
            .insert(
                index.empty_root(),
                PageId::new(2),
                VersionDescriptor {
                    page_id: PageId::new(2),
                    token: VersionToken::Wal {
                        source_file_number: first.file_number,
                        record_lsn: Lsn::new(1),
                    },
                    image_lsn: Lsn::new(1),
                    image_crc32c: 1,
                    locator: VersionLocator::WalImage {
                        frame: WalFrameLocator {
                            source: first,
                            frame_offset: 0,
                            frame_length: 64,
                            header_crc32c: 1,
                            payload_offset: 16,
                            payload_length: 32,
                            payload_crc32c: 1,
                            committed_prefix_length: 128,
                            commit_frame_offset: 64,
                            commit_lsn: Lsn::new(2),
                        },
                    },
                },
            )
            .unwrap();
        let first_root = manager.pin_root(&index, root).unwrap();
        let second_root = manager.pin_root(&index, root).unwrap();
        let third_root = manager.pin_root(&index, root).unwrap();
        let fourth_root = manager.pin_root(&index, root).unwrap();
        assert!(manager.pin_root(&index, root).is_err());
        drop(first_root);
        drop((second_root, third_root, fourth_root));
    }

    #[test]
    fn missing_index_reachable_source_is_corruption() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let header = b"missing-reachable-source";
        let identity = source_identity(SourceFileKind::WalSegment, 12, header);
        let root = index
            .insert(
                index.empty_root(),
                PageId::new(2),
                VersionDescriptor {
                    page_id: PageId::new(2),
                    token: VersionToken::Wal {
                        source_file_number: identity.file_number,
                        record_lsn: Lsn::new(1),
                    },
                    image_lsn: Lsn::new(1),
                    image_crc32c: 3,
                    locator: VersionLocator::WalImage {
                        frame: WalFrameLocator {
                            source: identity,
                            frame_offset: 32,
                            frame_length: 48,
                            header_crc32c: 4,
                            payload_offset: 48,
                            payload_length: 32,
                            payload_crc32c: 5,
                            committed_prefix_length: 128,
                            commit_frame_offset: 96,
                            commit_lsn: Lsn::new(2),
                        },
                    },
                },
            )
            .unwrap();
        let _root_lease = manager.pin_root(&index, root).unwrap();
        assert!(manager.acquire(identity, header.len()).is_err());
        assert!(
            manager
                .collect_garbage(
                    &index,
                    [SourceCandidate {
                        identity,
                        header_bytes: header.len(),
                    }],
                    |_| Ok(()),
                )
                .is_err()
        );
    }
}
