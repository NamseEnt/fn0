use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use dodb_core::{Error, Result};

use super::paged_manifest::PagedManifestStore;
use super::resource::{
    BlinkResourceManager, DiskCategory, DiskReservation, Reservation, ReservationClass,
    ResourceCategory,
};
use super::source_catalog::{SourceCatalogRecord, SourceCatalogStore, encode_source_record};
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
    reserved_roots: usize,
    page_reuse_root: Option<VersionIndexRoot>,
    durable_catalog_root: Option<super::version_index::NodeRef>,
    pending_catalog_root: Option<super::version_index::NodeRef>,
    catalog_publication_active: bool,
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
    source_catalog: SourceCatalogStore,
    checkpoint_disk_key: String,
    wal_disk_key: String,
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

pub struct SourceCatalogPublicationLease {
    manager: SourceManager,
    root: super::version_index::NodeRef,
    active: bool,
}

pub struct SourceCatalogBuildLease {
    manager: SourceManager,
    active: bool,
}

pub struct SourceRootReservation {
    manager: SourceManager,
    root_id: u64,
    index_reservation: Option<Reservation>,
    root_reservation: Option<Reservation>,
    slot_reserved: bool,
}

pub struct SourcePageReuseReservation {
    manager: SourceManager,
    root: VersionIndexRoot,
}

pub struct SourceLease {
    manager: SourceManager,
    identity: SourceIdentity,
    file: File,
    _reservation: Reservation,
}

pub(crate) struct ValidationScratch {
    file: File,
    path: PathBuf,
    _reservation: DiskReservation,
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
    pub unlinked_catalog_pages: u64,
    pub unlinked_catalog_bytes: u64,
    pub unlinked_index_nodes: u64,
    pub unlinked_index_bytes: u64,
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
    _reservation: Option<DiskReservation>,
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
        fs::create_dir_all(&config.wal_directory)?;
        let source_catalog =
            SourceCatalogStore::open(&config.checkpoint_directory, config.store_identity)?;
        let checkpoint_disk_bytes = directory_file_bytes(&config.checkpoint_directory)?;
        let wal_disk_bytes = directory_file_bytes(&config.wal_directory)?;
        let checkpoint_disk_key = fs::canonicalize(&config.checkpoint_directory)?
            .display()
            .to_string();
        let wal_disk_key = fs::canonicalize(&config.wal_directory)?
            .display()
            .to_string();
        resources
            .disk()
            .reconcile_persistent_usage(
                config.tenant_id,
                DiskCategory::RetainedSource,
                &checkpoint_disk_key,
                checkpoint_disk_bytes,
            )
            .map_err(|_| Error::overloaded("paged source disk budget is exhausted"))?;
        resources
            .disk()
            .reconcile_persistent_usage(
                config.tenant_id,
                DiskCategory::Wal,
                &wal_disk_key,
                wal_disk_bytes,
            )
            .map_err(|_| Error::overloaded("paged WAL disk budget is exhausted"))?;
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
                source_catalog,
                checkpoint_disk_key,
                wal_disk_key,
                state: Mutex::new(SourceState {
                    roots: HashMap::new(),
                    active_handles: HashMap::new(),
                    next_root_id: 0,
                    reserved_roots: 0,
                    page_reuse_root: None,
                    durable_catalog_root: None,
                    pending_catalog_root: None,
                    catalog_publication_active: false,
                    garbage_collection_active: false,
                }),
            }),
        })
    }

    pub fn pin_root(
        &self,
        index: &VersionIndex,
        root: VersionIndexRoot,
    ) -> Result<SourceRootLease> {
        let mut reservation = self.reserve_root_slot(index)?;
        let index_reservation = reservation
            .index_reservation
            .take()
            .ok_or_else(|| Error::invariant("paged source index root reservation is missing"))?;
        let pin = index.pin_root_reserved(root, index_reservation)?;
        self.register_root(root, pin, reservation)
    }

    #[cfg(test)]
    pub(crate) fn registered_root_count(&self) -> Result<usize> {
        self.inner
            .state
            .lock()
            .map(|state| state.roots.len())
            .map_err(|_| Error::invariant("paged source manager lock poisoned"))
    }

    pub fn reserve_root_slot(&self, index: &VersionIndex) -> Result<SourceRootReservation> {
        self.reserve_root_slot_inner(index, None)
    }

    pub fn reserve_page_reuse(
        &self,
        index: &VersionIndex,
        root: VersionIndexRoot,
    ) -> Result<Option<SourcePageReuseReservation>> {
        if index.store_identity() != self.inner.store_identity {
            return Err(Error::corruption(
                "paged source root belongs to a different store identity",
            ));
        }
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
        if state.garbage_collection_active
            || state.reserved_roots != 0
            || state.page_reuse_root.is_some()
            || state.roots.len() != 1
            || !state.roots.values().any(|entry| entry.root == root)
        {
            return Ok(None);
        }
        state.page_reuse_root = Some(root);
        Ok(Some(SourcePageReuseReservation {
            manager: self.clone(),
            root,
        }))
    }

    fn reserve_root_slot_inner(
        &self,
        index: &VersionIndex,
        page_reuse: Option<&SourcePageReuseReservation>,
    ) -> Result<SourceRootReservation> {
        if index.store_identity() != self.inner.store_identity {
            return Err(Error::corruption(
                "paged source root belongs to a different store identity",
            ));
        }
        if page_reuse
            .is_some_and(|reservation| !Arc::ptr_eq(&self.inner, &reservation.manager.inner))
        {
            return Err(Error::corruption(
                "paged page-reuse reservation belongs to another source manager",
            ));
        }
        let index_reservation = index.reserve_root_pin()?;
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
        match (state.page_reuse_root, page_reuse) {
            (None, None) => {}
            (Some(root), Some(reservation)) if root == reservation.root => {}
            (Some(_), _) => {
                return Err(Error::overloaded(
                    "paged source root admission is paused during zero-reader page reuse",
                ));
            }
            (None, Some(_)) => {
                return Err(Error::invariant(
                    "paged page-reuse reservation lost its root fence",
                ));
            }
        }
        if state
            .roots
            .len()
            .checked_add(state.reserved_roots)
            .is_none_or(|root_count| root_count >= self.inner.maximum_roots)
        {
            return Err(Error::overloaded(
                "paged source root pin limit is exhausted",
            ));
        }
        let reserved_roots = state
            .reserved_roots
            .checked_add(1)
            .ok_or_else(|| Error::overloaded("paged source root reservation count overflows"))?;
        state
            .roots
            .try_reserve(reserved_roots)
            .map_err(|_| Error::overloaded("paged source root registry capacity is exhausted"))?;
        let root_id = state.next_root_id;
        let next_root_id = state
            .next_root_id
            .checked_add(1)
            .ok_or_else(|| Error::overloaded("paged source root id reservation overflows"))?;
        state.reserved_roots = reserved_roots;
        state.next_root_id = next_root_id;
        drop(state);
        Ok(SourceRootReservation {
            manager: self.clone(),
            root_id,
            index_reservation: Some(index_reservation),
            root_reservation: Some(root_reservation),
            slot_reserved: true,
        })
    }

    pub fn pin_root_reserved(
        &self,
        index: &VersionIndex,
        root: VersionIndexRoot,
        mut reservation: SourceRootReservation,
    ) -> Result<SourceRootLease> {
        if !Arc::ptr_eq(&self.inner, &reservation.manager.inner)
            || index.store_identity() != self.inner.store_identity
        {
            return Err(Error::corruption(
                "paged source root reservation belongs to another manager",
            ));
        }
        let index_reservation = reservation
            .index_reservation
            .take()
            .ok_or_else(|| Error::invariant("paged source index root reservation is missing"))?;
        let pin = index.pin_inserted_root(root, index_reservation)?;
        self.register_root(root, pin, reservation)
    }

    fn register_root(
        &self,
        root: VersionIndexRoot,
        pin: VersionIndexRootPin,
        mut reservation: SourceRootReservation,
    ) -> Result<SourceRootLease> {
        let root_reservation = reservation
            .root_reservation
            .take()
            .ok_or_else(|| Error::invariant("paged source root reservation is missing"))?;
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
        if state.reserved_roots == 0 {
            return Err(Error::invariant(
                "paged source root reservation accounting underflow",
            ));
        }
        state.reserved_roots -= 1;
        let root_id = reservation.root_id;
        state.roots.insert(
            root_id,
            RootEntry {
                root,
                _pin: pin,
                _reservation: root_reservation,
            },
        );
        reservation.slot_reserved = false;
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
        if let Some(catalog_root) = manifest.source_catalog_root {
            self.validate_manifest_source_catalog(index, manifest, catalog_root)?;
        }
        let lease = self.pin_root(index, root)?;
        self.inner
            .state
            .lock()
            .map_err(|_| Error::invariant("paged source manager lock poisoned"))?
            .durable_catalog_root = manifest.source_catalog_root;
        Ok(lease)
    }

    pub(super) fn write_source_catalog(
        &self,
        index: &VersionIndex,
        root: VersionIndexRoot,
        recovery_source: SourceIdentity,
    ) -> Result<super::version_index::NodeRef> {
        if index.store_identity() != self.inner.store_identity
            || recovery_source.kind != SourceFileKind::WalSegment
            || recovery_source.file_number == 0
        {
            return Err(Error::invalid_input(
                "source catalog target identity is invalid",
            ));
        }
        let run_id = GC_RUN_SEQUENCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map_err(|_| Error::invalid_input("source catalog run id exhausted"))?;
        let mut marker_count = 1u64;
        index.visit_sources(root, &mut |_| {
            marker_count = marker_count
                .checked_add(1)
                .ok_or_else(|| Error::overloaded("source catalog marker count overflows"))?;
            Ok(())
        })?;
        let scratch_reservation = self.reserve_marker_scratch(marker_count)?;
        let marker_directory = self.inner.scratch_directory.join(format!(
            "source-catalog-{}-{run_id:020}",
            std::process::id()
        ));
        fs::create_dir(&marker_directory)?;
        let _scratch = GcScratch {
            path: marker_directory.clone(),
            _reservation: Some(scratch_reservation),
        };
        index.visit_sources(root, &mut |source| match source {
            SourceReachability::Exact(identity) => write_source_marker(&marker_directory, identity),
            SourceReachability::AllWalSegments => {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(marker_directory.join("retain-all-wal-segments"))?;
                Ok(())
            }
        })?;
        write_source_marker(&marker_directory, recovery_source)?;
        let reserved_bytes = self
            .inner
            .source_catalog
            .estimate_marker_run_bytes(&marker_directory)?;
        let checkpoint_bytes_before = directory_file_bytes(&self.inner.checkpoint_directory)?;
        let mut disk_reservation = self
            .inner
            .resources
            .disk()
            .reserve(
                self.inner.tenant_id,
                DiskCategory::RetainedSource,
                reserved_bytes,
            )
            .map_err(|_| Error::overloaded("source catalog disk budget is exhausted"))?;
        let catalog_build = self
            .inner
            .source_catalog
            .build_from_marker_run(&marker_directory);
        let checkpoint_bytes_after = directory_file_bytes(&self.inner.checkpoint_directory)?;
        let written_bytes = checkpoint_bytes_after
            .checked_sub(checkpoint_bytes_before)
            .ok_or_else(|| Error::invariant("source catalog directory shrank during build"))?;
        if written_bytes > disk_reservation.bytes() {
            disk_reservation
                .grow(written_bytes - disk_reservation.bytes())
                .map_err(|_| Error::overloaded("source catalog disk usage exceeded reservation"))?;
        } else {
            let unused_bytes = disk_reservation.bytes() - written_bytes;
            disk_reservation
                .shrink(unused_bytes)
                .map_err(|_| Error::invariant("source catalog disk reservation shrink failed"))?;
        }
        let actual_bytes = directory_file_bytes(&self.inner.checkpoint_directory)?;
        self.inner
            .resources
            .disk()
            .replace_persistent_usage_from_reservation(
                disk_reservation,
                self.inner.checkpoint_disk_key.clone(),
                actual_bytes,
            )
            .map_err(|_| Error::overloaded("paged source disk budget is exhausted"))?;
        let catalog_root = catalog_build?;
        self.inner
            .source_catalog
            .visit(catalog_root, &mut |_| Ok(()))?;
        Ok(catalog_root)
    }

    pub(super) fn begin_source_catalog_publication(&self) -> Result<SourceCatalogBuildLease> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
        if state.garbage_collection_active
            || state.pending_catalog_root.is_some()
            || state.catalog_publication_active
        {
            return Err(Error::overloaded(
                "paged source catalog publication is already active",
            ));
        }
        state.catalog_publication_active = true;
        Ok(SourceCatalogBuildLease {
            manager: self.clone(),
            active: true,
        })
    }

    pub(super) fn retain_disk_reservation(&self, reservation: DiskReservation) -> Result<()> {
        let (directory, key) = match reservation.category() {
            DiskCategory::RetainedSource => (
                &self.inner.checkpoint_directory,
                self.inner.checkpoint_disk_key.clone(),
            ),
            DiskCategory::Wal => (&self.inner.wal_directory, self.inner.wal_disk_key.clone()),
            _ => {
                return Err(Error::invariant(
                    "paged source disk reservation has the wrong category",
                ));
            }
        };
        let actual_bytes = directory_file_bytes(directory)?;
        self.inner
            .resources
            .disk()
            .replace_persistent_usage_from_reservation(reservation, key, actual_bytes)
            .map_err(|_| Error::overloaded("paged source disk budget is exhausted"))?;
        Ok(())
    }

    pub(crate) fn reserve_wal_growth(&self, maximum_bytes: u64) -> Result<DiskReservation> {
        self.inner
            .resources
            .disk()
            .reserve(self.inner.tenant_id, DiskCategory::Wal, maximum_bytes)
            .map_err(|_| Error::overloaded("paged WAL disk budget is exhausted"))
    }

    pub(crate) fn create_validation_scratch(&self, bytes: u64) -> Result<ValidationScratch> {
        let reservation = self
            .inner
            .resources
            .disk()
            .reserve(self.inner.tenant_id, DiskCategory::Scratch, bytes)
            .map_err(|_| Error::overloaded("paged validation scratch disk budget is exhausted"))?;
        let run_id = GC_RUN_SEQUENCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map_err(|_| Error::invalid_input("paged validation scratch id exhausted"))?;
        let path = self.inner.scratch_directory.join(format!(
            "blink-validation-{}-{run_id:020}.scratch",
            std::process::id()
        ));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        if let Err(error) = file.set_len(bytes) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(error.into());
        }
        Ok(ValidationScratch {
            file,
            path,
            _reservation: reservation,
        })
    }

    fn reserve_marker_scratch(&self, marker_count: u64) -> Result<DiskReservation> {
        let bytes = marker_count
            .checked_mul(48)
            .ok_or_else(|| Error::overloaded("paged source scratch reservation overflows"))?;
        self.inner
            .resources
            .disk()
            .reserve(self.inner.tenant_id, DiskCategory::Scratch, bytes)
            .map_err(|_| Error::overloaded("paged source scratch disk budget is exhausted"))
    }

    fn count_gc_markers(
        &self,
        index: &VersionIndex,
        roots: &[VersionIndexRoot],
        catalog_roots: [Option<super::version_index::NodeRef>; 2],
    ) -> Result<u64> {
        let mut marker_count = 0u64;
        for root in roots {
            index.visit_sources(*root, &mut |_| {
                marker_count = marker_count
                    .checked_add(1)
                    .ok_or_else(|| Error::overloaded("paged source GC marker count overflows"))?;
                Ok(())
            })?;
        }
        for catalog_root in catalog_roots.into_iter().flatten() {
            self.inner.source_catalog.visit(catalog_root, &mut |_| {
                marker_count = marker_count
                    .checked_add(1)
                    .ok_or_else(|| Error::overloaded("paged source GC marker count overflows"))?;
                Ok(())
            })?;
        }
        Ok(marker_count)
    }

    pub(super) fn install_source_catalog_root(
        &self,
        mut publication: SourceCatalogPublicationLease,
    ) -> Result<()> {
        if !Arc::ptr_eq(&self.inner, &publication.manager.inner) {
            return Err(Error::corruption(
                "paged source catalog publication belongs to another manager",
            ));
        }
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
        if state.pending_catalog_root != Some(publication.root) {
            return Err(Error::invariant(
                "paged source catalog publication lost its pending root",
            ));
        }
        state.durable_catalog_root = Some(publication.root);
        state.pending_catalog_root = None;
        publication.active = false;
        Ok(())
    }

    fn validate_manifest_source_catalog(
        &self,
        index: &VersionIndex,
        manifest: super::paged_manifest::PagedManifest,
        catalog_root: super::version_index::NodeRef,
    ) -> Result<()> {
        let root = index.root_from_node_reference(manifest.durable_index_root)?;
        let recovery_wal = self.source_identity_from_file(
            SourceFileKind::WalSegment,
            manifest.recovery_wal_file_number,
        )?;
        let mut marker_count = 1u64;
        index.visit_sources(root, &mut |_| {
            marker_count = marker_count.checked_add(1).ok_or_else(|| {
                Error::overloaded("source catalog validation marker count overflows")
            })?;
            Ok(())
        })?;
        self.inner.source_catalog.visit(catalog_root, &mut |_| {
            marker_count = marker_count.checked_add(1).ok_or_else(|| {
                Error::overloaded("source catalog validation marker count overflows")
            })?;
            Ok(())
        })?;
        let scratch_reservation = self.reserve_marker_scratch(marker_count)?;
        let run_id = GC_RUN_SEQUENCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map_err(|_| Error::invalid_input("source catalog validation id exhausted"))?;
        let scratch_path = self.inner.scratch_directory.join(format!(
            "source-catalog-validation-{}-{run_id:020}",
            std::process::id()
        ));
        let expected_path = scratch_path.join("expected");
        let actual_path = scratch_path.join("actual");
        fs::create_dir_all(&expected_path)?;
        fs::create_dir(&actual_path)?;
        let _scratch = GcScratch {
            path: scratch_path,
            _reservation: Some(scratch_reservation),
        };
        index.visit_sources(root, &mut |source| match source {
            SourceReachability::Exact(identity) => write_source_marker(&expected_path, identity),
            SourceReachability::AllWalSegments => write_all_wal_marker(&expected_path),
        })?;
        write_source_marker(&expected_path, recovery_wal)?;
        self.inner
            .source_catalog
            .visit(catalog_root, &mut |record| match record {
                SourceCatalogRecord::Exact(identity) => {
                    let _source = open_verified_source(
                        &self.source_path(identity),
                        identity,
                        source_header_bytes(identity.kind),
                    )?;
                    write_source_marker(&actual_path, identity)
                }
                SourceCatalogRecord::AllWalSegments => write_all_wal_marker(&actual_path),
            })?;
        compare_source_marker_sets(&expected_path, &actual_path)
    }

    fn source_identity_from_file(
        &self,
        kind: SourceFileKind,
        file_number: u64,
    ) -> Result<SourceIdentity> {
        if file_number == 0 {
            return Err(Error::corruption("manifest recovery WAL number is zero"));
        }
        let path = self.source_path(SourceIdentity {
            kind,
            file_number,
            header_identity: [0; 32],
        });
        let mut file = File::open(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::corruption("manifest recovery WAL source is missing")
            } else {
                Error::from(error)
            }
        })?;
        let header_bytes = source_header_bytes(kind);
        if file.metadata()?.len() < header_bytes as u64 {
            return Err(Error::corruption(
                "manifest recovery WAL header is truncated",
            ));
        }
        let mut header = vec![0u8; header_bytes];
        file.read_exact(&mut header)?;
        Ok(SourceIdentity {
            kind,
            file_number,
            header_identity: *blake3::hash(&header).as_bytes(),
        })
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
        self.reconcile_source_disk_usage()?;
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
        let (roots, durable_catalog_root, pending_catalog_root) = {
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
            if state.catalog_publication_active {
                return Err(Error::overloaded(
                    "paged source collection is deferred during catalog publication",
                ));
            }
            if state.reserved_roots != 0 || state.page_reuse_root.is_some() {
                return Err(Error::overloaded(
                    "paged source collection is deferred by a reader generation reservation",
                ));
            }
            state.garbage_collection_active = true;
            (
                state
                    .roots
                    .values()
                    .map(|entry| entry.root)
                    .collect::<Vec<_>>(),
                state.durable_catalog_root,
                state.pending_catalog_root,
            )
        };
        let _gate = GcGate {
            manager: self.clone(),
        };
        let marker_count =
            self.count_gc_markers(index, &roots, [durable_catalog_root, pending_catalog_root])?;
        let scratch_reservation = self.reserve_marker_scratch(marker_count)?;
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
            _reservation: Some(scratch_reservation),
        };
        for root in &roots {
            index.visit_sources(*root, &mut |source| match source {
                SourceReachability::Exact(identity) => write_source_marker(&scratch_path, identity),
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
        for catalog_root in [durable_catalog_root, pending_catalog_root]
            .into_iter()
            .flatten()
        {
            self.inner
                .source_catalog
                .visit(catalog_root, &mut |record| match record {
                    SourceCatalogRecord::Exact(identity) => {
                        write_source_marker(&scratch_path, identity)
                    }
                    SourceCatalogRecord::AllWalSegments => {
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(false)
                            .open(scratch_path.join("retain-all-wal-segments"))?;
                        Ok(())
                    }
                })?;
        }

        let (unlinked_index_nodes, unlinked_index_bytes) =
            index.reclaim_unreachable_nodes(&roots, &scratch_path)?;
        let mut report = SourceGcReport {
            unlinked_index_nodes,
            unlinked_index_bytes,
            ..SourceGcReport::default()
        };
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
        let (unlinked_catalog_pages, unlinked_catalog_bytes) =
            self.inner.source_catalog.reclaim_unreachable(
                [durable_catalog_root, pending_catalog_root]
                    .into_iter()
                    .flatten(),
            )?;
        report.unlinked_catalog_pages = unlinked_catalog_pages;
        report.unlinked_catalog_bytes = unlinked_catalog_bytes;
        self.reconcile_source_disk_usage()?;
        Ok(report)
    }

    pub(crate) fn sync_wal_directory(&self) -> Result<()> {
        File::open(&self.inner.wal_directory)?.sync_all()?;
        Ok(())
    }

    pub(crate) fn checkpoint_directory(&self) -> &Path {
        &self.inner.checkpoint_directory
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

    fn reconcile_source_disk_usage(&self) -> Result<()> {
        let checkpoint_bytes = directory_file_bytes(&self.inner.checkpoint_directory)?;
        self.inner
            .resources
            .disk()
            .reconcile_persistent_usage(
                self.inner.tenant_id,
                DiskCategory::RetainedSource,
                &self.inner.checkpoint_disk_key,
                checkpoint_bytes,
            )
            .map_err(|_| Error::overloaded("paged source disk budget is exhausted"))?;
        let wal_bytes = directory_file_bytes(&self.inner.wal_directory)?;
        self.inner
            .resources
            .disk()
            .reconcile_persistent_usage(
                self.inner.tenant_id,
                DiskCategory::Wal,
                &self.inner.wal_disk_key,
                wal_bytes,
            )
            .map_err(|_| Error::overloaded("paged WAL disk budget is exhausted"))?;
        Ok(())
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

impl ValidationScratch {
    pub(crate) fn read_marker(&mut self, offset: u64) -> Result<u8> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut marker = [0u8; 1];
        self.file.read_exact(&mut marker)?;
        Ok(marker[0])
    }

    pub(crate) fn write_marker(&mut self, offset: u64, marker: u8) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&[marker])?;
        Ok(())
    }
}

impl Drop for ValidationScratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
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

impl SourcePageReuseReservation {
    pub fn reserve_root_slot(&self, index: &VersionIndex) -> Result<SourceRootReservation> {
        self.manager.reserve_root_slot_inner(index, Some(self))
    }
}

impl Drop for SourceRootLease {
    fn drop(&mut self) {
        if let Ok(mut state) = self.manager.inner.state.lock() {
            state.roots.remove(&self.root_id);
        }
    }
}

impl Drop for SourceCatalogPublicationLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Ok(mut state) = self.manager.inner.state.lock()
            && state.pending_catalog_root == Some(self.root)
        {
            state.pending_catalog_root = None;
        }
    }
}

impl SourceCatalogBuildLease {
    pub(super) fn reserve_root(
        mut self,
        root: super::version_index::NodeRef,
    ) -> Result<SourceCatalogPublicationLease> {
        let mut state = self
            .manager
            .inner
            .state
            .lock()
            .map_err(|_| Error::invariant("paged source manager lock poisoned"))?;
        if !state.catalog_publication_active || state.pending_catalog_root.is_some() {
            return Err(Error::invariant(
                "paged source catalog build lease lost its publication fence",
            ));
        }
        state.catalog_publication_active = false;
        state.pending_catalog_root = Some(root);
        self.active = false;
        Ok(SourceCatalogPublicationLease {
            manager: self.manager.clone(),
            root,
            active: true,
        })
    }
}

impl Drop for SourceCatalogBuildLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Ok(mut state) = self.manager.inner.state.lock() {
            state.catalog_publication_active = false;
        }
    }
}

impl Drop for SourceRootReservation {
    fn drop(&mut self) {
        if !self.slot_reserved {
            return;
        }
        if let Ok(mut state) = self.manager.inner.state.lock() {
            state.reserved_roots = state.reserved_roots.saturating_sub(1);
        }
    }
}

impl Drop for SourcePageReuseReservation {
    fn drop(&mut self) {
        if let Ok(mut state) = self.manager.inner.state.lock()
            && state.page_reuse_root == Some(self.root)
        {
            state.page_reuse_root = None;
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
    let mut bytes = [0u8; 41];
    bytes[0] = identity.kind as u8;
    bytes[1..9].copy_from_slice(&identity.file_number.to_le_bytes());
    bytes[9..41].copy_from_slice(&identity.header_identity);
    format!("source-{}", blake3::hash(&bytes).to_hex())
}

fn source_header_bytes(kind: SourceFileKind) -> usize {
    match kind {
        SourceFileKind::CheckpointPack => super::PAGE_SIZE,
        SourceFileKind::WalSegment => crate::wal::INIT_FRAME_SIZE,
    }
}

fn directory_file_bytes(directory: &Path) -> Result<u64> {
    let mut total_bytes = 0u64;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| Error::invalid_input("paged source disk byte count overflows"))?;
        }
    }
    Ok(total_bytes)
}

fn write_all_wal_marker(directory: &Path) -> Result<()> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("retain-all-wal-segments"))?;
    Ok(())
}

fn compare_source_marker_sets(expected: &Path, actual: &Path) -> Result<()> {
    for (source_directory, other_directory) in [(expected, actual), (actual, expected)] {
        for entry in fs::read_dir(source_directory)? {
            let entry = entry?;
            let file_name = entry.file_name();
            if !other_directory.join(file_name).try_exists()? {
                return Err(Error::corruption(
                    "source catalog does not match the durable version index",
                ));
            }
        }
    }
    Ok(())
}

fn write_source_marker(directory: &Path, identity: SourceIdentity) -> Result<()> {
    if identity.file_number == 0 {
        return Err(Error::corruption(
            "paged source identity has file number zero",
        ));
    }
    let path = directory.join(source_mark_name(identity));
    let encoded_identity = encode_source_record(identity);
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut marker) => {
            marker.write_all(&encoded_identity)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = fs::read(path)?;
            if existing.as_slice() == encoded_identity {
                Ok(())
            } else {
                Err(Error::corruption(
                    "paged source marker identity collides with existing marker",
                ))
            }
        }
        Err(error) => Err(error.into()),
    }
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
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

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

    #[test]
    fn retained_disk_usage_survives_manager_drop_and_reopen_without_double_counting() {
        let directory = TestDirectory::new();
        let resources = test_resources();
        let config = || SourceManagerConfig {
            store_identity: StoreIdentity {
                database_uuid: [19; 16],
                wal_epoch: 3,
            },
            checkpoint_directory: directory.path().join("packs"),
            wal_directory: directory.path().join("segments"),
            scratch_directory: directory.path().join("scratch"),
            tenant_id: 9,
            maximum_open_handles: 2,
            maximum_roots: 4,
            handle_charge_bytes: 128,
            gc_reservation_bytes: 8192,
        };
        let first = SourceManager::new(config(), resources.clone()).unwrap();
        let wal_identity = source_identity(SourceFileKind::WalSegment, 1, b"active-segment");
        let wal_path = write_source(&first, wal_identity, b"active-segment");
        let pack_identity = source_identity(SourceFileKind::CheckpointPack, 2, b"retained-pack");
        let pack_path = write_source(&first, pack_identity, b"retained-pack");
        first.reconcile_source_disk_usage().unwrap();
        drop(first);
        let disk = resources.disk().snapshot().unwrap();
        let wal_bytes = fs::metadata(wal_path).unwrap().len();
        let pack_bytes = fs::metadata(pack_path).unwrap().len();
        assert_eq!(disk.current_total, wal_bytes + pack_bytes);
        assert_eq!(
            disk.current_by_category[DiskCategory::Wal as usize],
            wal_bytes
        );
        assert_eq!(
            disk.current_by_category[DiskCategory::RetainedSource as usize],
            pack_bytes
        );
        let reopened = SourceManager::new(config(), resources.clone()).unwrap();
        assert_eq!(
            resources.disk().snapshot().unwrap().current_total,
            wal_bytes + pack_bytes
        );
        drop(reopened);
        assert_eq!(
            resources.disk().snapshot().unwrap().current_total,
            wal_bytes + pack_bytes
        );
    }

    #[test]
    fn source_gc_rejects_scratch_quota_before_creating_markers() {
        let directory = TestDirectory::new();
        let resources = BlinkResourceManager::new_with_disk_limits(
            super::super::resource::ResourceLimits {
                global_bytes: 1 << 20,
                per_tenant_bytes: 1 << 20,
                progress_pool_bytes: 1 << 17,
                max_tenants: 4,
                max_pending_waiters: 4,
                global_category_bytes: [1 << 20; 7],
                per_tenant_category_bytes: [1 << 20; 7],
            },
            super::super::resource::DiskResourceLimits {
                global_bytes: 1 << 20,
                per_tenant_bytes: 1 << 20,
                max_tenants: 4,
                global_category_bytes: [1 << 20, 1 << 20, 1 << 20, 47, 1 << 20],
                per_tenant_category_bytes: [1 << 20, 1 << 20, 1 << 20, 47, 1 << 20],
            },
        )
        .unwrap();
        let store_identity = StoreIdentity {
            database_uuid: [19; 16],
            wal_epoch: 3,
        };
        let index = VersionIndex::open(
            &directory.path().join("index"),
            store_identity,
            resources.clone(),
            9,
            2,
        )
        .unwrap();
        let manager = SourceManager::new(
            SourceManagerConfig {
                store_identity,
                checkpoint_directory: directory.path().join("packs"),
                wal_directory: directory.path().join("segments"),
                scratch_directory: directory.path().join("scratch"),
                tenant_id: 9,
                maximum_open_handles: 2,
                maximum_roots: 4,
                handle_charge_bytes: 128,
                gc_reservation_bytes: 8192,
            },
            resources,
        )
        .unwrap();
        let source = source_identity(SourceFileKind::CheckpointPack, 1, b"missing-but-indexed");
        let root = insert_checkpoint_page(&index, index.empty_root(), source, 8192, 1, 3);
        let _root_lease = manager.pin_root(&index, root).unwrap();

        assert!(manager.collect_garbage(&index, [], |_| Ok(())).is_err());
        assert_eq!(
            fs::read_dir(directory.path().join("scratch"))
                .unwrap()
                .count(),
            0
        );
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
        let chain = crate::blink::version_chain::DeltaChain::new(
            PageId::new(2),
            vec![
                crate::blink::version_chain::DeltaChainEntry::CheckpointImage {
                    source: checkpoint,
                    offset: 64,
                    source_length: 8192,
                    image_lsn: Lsn::new(8),
                    image_crc32c: 3,
                },
                crate::blink::version_chain::DeltaChainEntry::WalDelta {
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
                    record_lsn: Lsn::new(10),
                    base_lsn: Lsn::new(8),
                    base_crc32c: 3,
                    image_lsn: Lsn::new(10),
                    image_crc32c: 4,
                },
            ],
        )
        .unwrap();
        let chain_reference = index.delta_chains().write(1, &chain).unwrap();
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
                        chain: chain_reference,
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
    fn delta_roots_retain_only_exact_ancestry_sources() {
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
        let chain = crate::blink::version_chain::DeltaChain::new(
            PageId::new(2),
            vec![
                crate::blink::version_chain::DeltaChainEntry::WalImage {
                    frame: WalFrameLocator {
                        source: base,
                        frame_offset: 32,
                        frame_length: 96,
                        header_crc32c: 5,
                        payload_offset: 48,
                        payload_length: 64,
                        payload_crc32c: 4,
                        committed_prefix_length: 256,
                        commit_frame_offset: 128,
                        commit_lsn: Lsn::new(22),
                    },
                    record_lsn: Lsn::new(20),
                    image_lsn: Lsn::new(20),
                    image_crc32c: 4,
                },
                crate::blink::version_chain::DeltaChainEntry::WalDelta {
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
                    record_lsn: Lsn::new(30),
                    base_lsn: Lsn::new(20),
                    base_crc32c: 4,
                    image_lsn: Lsn::new(30),
                    image_crc32c: 3,
                },
            ],
        )
        .unwrap();
        let chain_reference = index.delta_chains().write(1, &chain).unwrap();
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
                        chain: chain_reference,
                        depth: 1,
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
        assert_eq!(report.protected_sources, 2);
        assert_eq!(report.unlinked_sources, 1);
        drop(root_lease);
        let report = manager
            .collect_garbage(&index, candidates, |_| Ok(()))
            .unwrap();
        assert_eq!(report.unlinked_sources, 2);
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
    fn source_gc_survives_sigkill_before_and_after_unlink() {
        const CHILD_DIRECTORY: &str = "DODB_SOURCE_GC_SIGKILL_DIRECTORY";
        const CHILD_STAGE: &str = "DODB_SOURCE_GC_SIGKILL_STAGE";
        const CHILD_READY: &str = "DODB_SOURCE_GC_SIGKILL_READY";
        let header = b"sigkill-wal-prefix";
        if let Ok(directory_path) = std::env::var(CHILD_DIRECTORY) {
            let stage_number = std::env::var(CHILD_STAGE).unwrap().parse::<u8>().unwrap();
            let target_point = match stage_number {
                0 => SourceGcFaultPoint::BeforeUnlink,
                1 => SourceGcFaultPoint::AfterUnlink,
                _ => panic!("invalid source GC stage number"),
            };
            let directory_path = PathBuf::from(directory_path);
            let index = test_index(&directory_path);
            let source_manager = manager(&directory_path);
            let identity = source_identity(SourceFileKind::WalSegment, 5, header);
            let _ = source_manager.collect_garbage(
                &index,
                [SourceCandidate {
                    identity,
                    header_bytes: header.len(),
                }],
                |point| {
                    if point == target_point {
                        fs::write(std::env::var(CHILD_READY).unwrap(), b"ready").unwrap();
                        loop {
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                    Ok(())
                },
            );
            panic!("child collection did not stop at the requested boundary");
        }

        for (stage_number, target_point) in [
            SourceGcFaultPoint::BeforeUnlink,
            SourceGcFaultPoint::AfterUnlink,
        ]
        .into_iter()
        .enumerate()
        {
            let directory = TestDirectory::new();
            let index = test_index(directory.path());
            let source_manager = manager(directory.path());
            let identity = source_identity(SourceFileKind::WalSegment, 5, header);
            let source_path = write_source(&source_manager, identity, header);
            drop(source_manager);
            drop(index);
            let ready_path = directory.path().join(format!("ready-{stage_number}"));
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "blink::source_manager::tests::source_gc_survives_sigkill_before_and_after_unlink",
                    "--nocapture",
                ])
                .env(CHILD_DIRECTORY, directory.path())
                .env(CHILD_STAGE, stage_number.to_string())
                .env(CHILD_READY, &ready_path)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ready_path.exists() && Instant::now() < deadline {
                if let Some(status) = child.try_wait().unwrap() {
                    panic!("SIGKILL child exited before {target_point:?}: {status}");
                }
                thread::sleep(Duration::from_millis(1));
            }
            assert!(ready_path.exists(), "child did not reach {target_point:?}");
            assert_eq!(
                unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
                0
            );
            let status = child.wait().unwrap();
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(status.signal(), Some(libc::SIGKILL));
            assert_eq!(
                source_path.exists(),
                target_point == SourceGcFaultPoint::BeforeUnlink
            );

            let reopened_index = test_index(directory.path());
            let reopened_manager = manager(directory.path());
            let report = reopened_manager
                .collect_garbage(
                    &reopened_index,
                    [SourceCandidate {
                        identity,
                        header_bytes: header.len(),
                    }],
                    |_| Ok(()),
                )
                .unwrap();
            if target_point == SourceGcFaultPoint::BeforeUnlink {
                assert_eq!(report.unlinked_sources, 1);
            } else {
                assert_eq!(report.unlinked_sources, 0);
            }
            assert!(!source_path.exists());
        }
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
        let mut reservations = Vec::new();
        for _reservation_number in 0..4 {
            reservations.push(manager.reserve_root_slot(&index).unwrap());
        }
        assert!(manager.reserve_root_slot(&index).is_err());
        let root_leases = reservations
            .into_iter()
            .map(|reservation| {
                manager
                    .pin_root_reserved(&index, root, reservation)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(root_leases.len(), 4);
        drop(root_leases);
    }

    #[test]
    fn zero_reader_page_reuse_reservation_fences_new_roots_until_publication_slot_is_reserved() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let root = index.empty_root();
        let current_lease = manager.pin_root(&index, root).unwrap();
        let older_reader = manager.pin_root(&index, root).unwrap();
        assert!(manager.reserve_page_reuse(&index, root).unwrap().is_none());
        drop(older_reader);

        let reuse_reservation = manager.reserve_page_reuse(&index, root).unwrap().unwrap();
        assert!(manager.pin_root(&index, root).is_err());
        let publication_reservation = reuse_reservation.reserve_root_slot(&index).unwrap();
        drop(publication_reservation);
        drop(reuse_reservation);

        let new_reader = manager.pin_root(&index, root).unwrap();
        drop(new_reader);
        drop(current_lease);
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

    #[test]
    fn pending_catalog_publication_protects_recovery_wal_from_gc() {
        let directory = TestDirectory::new();
        let index = test_index(directory.path());
        let manager = manager(directory.path());
        let (checkpoint_source, checkpoint_header_bytes) =
            write_checkpoint_page(&manager, 1, 31, 41);
        let root = insert_checkpoint_page(
            &index,
            index.empty_root(),
            checkpoint_source,
            (checkpoint_header_bytes + 4096) as u64,
            1,
            2,
        );
        let _root_lease = manager.pin_root(&index, root).unwrap();
        let recovery_header = b"recovery-wal-header";
        let recovery_source = source_identity(SourceFileKind::WalSegment, 2, recovery_header);
        let recovery_path = write_source(&manager, recovery_source, recovery_header);
        let catalog_build = manager.begin_source_catalog_publication().unwrap();
        assert!(
            manager
                .collect_garbage(
                    &index,
                    [SourceCandidate {
                        identity: recovery_source,
                        header_bytes: recovery_header.len(),
                    }],
                    |_| Ok(()),
                )
                .is_err()
        );
        let catalog_root = manager
            .write_source_catalog(&index, root, recovery_source)
            .unwrap();
        let catalog_publication = catalog_build.reserve_root(catalog_root).unwrap();

        let protected = manager
            .collect_garbage(
                &index,
                [SourceCandidate {
                    identity: recovery_source,
                    header_bytes: recovery_header.len(),
                }],
                |_| Ok(()),
            )
            .unwrap();
        assert_eq!(protected.protected_sources, 1);
        assert!(recovery_path.is_file());

        drop(catalog_publication);
        let orphaned = manager
            .collect_garbage(
                &index,
                [SourceCandidate {
                    identity: recovery_source,
                    header_bytes: recovery_header.len(),
                }],
                |_| Ok(()),
            )
            .unwrap();
        assert_eq!(orphaned.unlinked_sources, 1);
        assert!(!recovery_path.exists());
    }
}
