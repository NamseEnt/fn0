use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use dodb_core::{Error, Lsn, PageId, Result};

use super::resource::{
    BlinkResourceManager, Reservation, ReservationClass, ResourceCategory, ResourceError,
};
use super::source_manager::{SourceLease, SourceManager};
use super::version_chain::{DeltaChain, DeltaChainEntry};
use super::version_index::{
    SourceIdentity, StoreIdentity, VersionDescriptor, VersionIndex, VersionIndexRoot,
    VersionLocator, VersionToken, WalFrameLocator,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PageVersionKey {
    pub store: StoreIdentity,
    pub page_id: PageId,
    pub token_kind: u8,
    pub token_file_number: u64,
    pub token_lsn: Lsn,
    pub image_lsn: Lsn,
    pub image_crc32c: u32,
    pub locator_kind: u8,
    pub source_file_number: u64,
    pub source_identity: [u8; 32],
    pub locator_offset: u64,
    pub chain_file_number: u64,
    pub chain_checksum: [u8; 32],
}

impl PageVersionKey {
    pub fn from_descriptor(store: StoreIdentity, descriptor: VersionDescriptor) -> Self {
        let (token_kind, token_file_number, token_lsn) = match descriptor.token {
            VersionToken::Bootstrap { page_id } => (0, page_id.get(), Lsn::new(0)),
            VersionToken::Wal {
                source_file_number,
                record_lsn,
            } => (1, source_file_number, record_lsn),
        };
        let (
            locator_kind,
            source_file_number,
            source_identity,
            locator_offset,
            chain_file_number,
            chain_checksum,
        ) = match descriptor.locator {
            VersionLocator::CheckpointImage { source, offset, .. } => (
                0,
                source.file_number,
                source.header_identity,
                offset,
                0,
                [0; 32],
            ),
            VersionLocator::WalImage { frame } => (
                1,
                frame.source.file_number,
                frame.source.header_identity,
                frame.frame_offset,
                0,
                [0; 32],
            ),
            VersionLocator::WalDelta { frame, chain, .. } => (
                2,
                frame.source.file_number,
                frame.source.header_identity,
                frame.frame_offset,
                chain.file_number,
                chain.checksum,
            ),
        };
        Self {
            store,
            page_id: descriptor.page_id,
            token_kind,
            token_file_number,
            token_lsn,
            image_lsn: descriptor.image_lsn,
            image_crc32c: descriptor.image_crc32c,
            locator_kind,
            source_file_number,
            source_identity,
            locator_offset,
            chain_file_number,
            chain_checksum,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageLoaderConfig {
    pub maximum_cache_bytes: u64,
    pub minimum_decoded_page_bytes: u64,
    pub maximum_load_reconstruction_bytes: u64,
    pub maximum_concurrent_loads: usize,
    pub maximum_followers: usize,
}

impl PageLoaderConfig {
    pub fn validate(self) -> Result<Self> {
        if self.maximum_cache_bytes < self.minimum_decoded_page_bytes
            || self.minimum_decoded_page_bytes == 0
            || self.maximum_load_reconstruction_bytes == 0
            || self.maximum_concurrent_loads == 0
            || self.maximum_followers == 0
        {
            return Err(Error::invalid_input(
                "paged cache configuration cannot fit one legal page load",
            ));
        }
        Ok(self)
    }
}

struct CachedValue<V> {
    value: V,
    bytes: u64,
    _reservation: Reservation,
}

pub struct PageGuard<V>(Arc<CachedValue<V>>);

impl<V> Clone for PageGuard<V> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<V> AsRef<V> for PageGuard<V> {
    fn as_ref(&self) -> &V {
        &self.0.value
    }
}

impl<V> PageGuard<V> {
    pub fn bytes(&self) -> u64 {
        self.0.bytes
    }
}

struct LoadTicket<V> {
    outcome: Mutex<Option<std::result::Result<Arc<CachedValue<V>>, LoadFailure>>>,
    completed: Condvar,
    followers: AtomicUsize,
}

impl<V> LoadTicket<V> {
    fn new() -> Self {
        Self {
            outcome: Mutex::new(None),
            completed: Condvar::new(),
            followers: AtomicUsize::new(0),
        }
    }

    fn complete(&self, outcome: std::result::Result<Arc<CachedValue<V>>, LoadFailure>) {
        if let Ok(mut stored) = self.outcome.lock()
            && stored.is_none()
        {
            *stored = Some(outcome);
            self.completed.notify_all();
        }
    }

    fn wait(&self) -> Result<Arc<CachedValue<V>>> {
        let mut stored = self
            .outcome
            .lock()
            .map_err(|_| Error::invariant("page loader ticket lock poisoned"))?;
        while stored.is_none() {
            stored = self
                .completed
                .wait(stored)
                .map_err(|_| Error::invariant("page loader ticket lock poisoned"))?;
        }
        match stored.as_ref().unwrap() {
            Ok(value) => Ok(Arc::clone(value)),
            Err(failure) => Err(failure.to_error()),
        }
    }
}

struct CacheState<K, V> {
    entries: HashMap<K, Arc<CachedValue<V>>>,
    lru: VecDeque<K>,
    in_flight: HashMap<K, Arc<LoadTicket<V>>>,
    cache_bytes: u64,
    reserved_cache_bytes: u64,
    active_loads: usize,
    active_followers: usize,
    closed: bool,
}

pub struct PageLoader<K, V> {
    tenant_id: u64,
    resources: BlinkResourceManager,
    config: PageLoaderConfig,
    state: Mutex<CacheState<K, V>>,
    source_reads: AtomicU64,
}

impl<K, V> PageLoader<K, V>
where
    K: Clone + Eq + Hash,
    V: Send + Sync + 'static,
{
    pub fn new(
        tenant_id: u64,
        resources: BlinkResourceManager,
        config: PageLoaderConfig,
    ) -> Result<Self> {
        let config = config.validate()?;
        Ok(Self {
            tenant_id,
            resources,
            config,
            state: Mutex::new(CacheState {
                entries: HashMap::new(),
                lru: VecDeque::new(),
                in_flight: HashMap::new(),
                cache_bytes: 0,
                reserved_cache_bytes: 0,
                active_loads: 0,
                active_followers: 0,
                closed: false,
            }),
            source_reads: AtomicU64::new(0),
        })
    }

    pub fn get_or_load<F>(
        &self,
        key: K,
        encoded_bytes: u64,
        decoded_bytes: u64,
        load: F,
    ) -> Result<PageGuard<V>>
    where
        F: FnOnce() -> Result<V>,
    {
        if decoded_bytes < self.config.minimum_decoded_page_bytes
            || decoded_bytes > self.config.maximum_cache_bytes
            || encoded_bytes > self.config.maximum_load_reconstruction_bytes
        {
            return Err(Error::overloaded(
                "paged load exceeds its configured cache or reconstruction envelope",
            ));
        }
        let (ticket, is_leader) = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| Error::invariant("page cache lock poisoned"))?;
            if state.closed {
                return Err(Error::overloaded("paged loader is shut down"));
            }
            if let Some(value) = state.entries.get(&key).cloned() {
                touch_lru(&mut state.lru, key);
                return Ok(PageGuard(value));
            }
            if let Some(ticket) = state.in_flight.get(&key).cloned() {
                if state.active_followers == self.config.maximum_followers {
                    return Err(Error::overloaded("paged loader follower limit exceeded"));
                }
                state.active_followers += 1;
                ticket.followers.fetch_add(1, Ordering::AcqRel);
                (ticket, false)
            } else {
                if state.active_loads == self.config.maximum_concurrent_loads {
                    return Err(Error::overloaded("paged loader job limit exceeded"));
                }
                evict_to_fit(&mut state, self.config.maximum_cache_bytes, decoded_bytes)?;
                state.reserved_cache_bytes = state
                    .reserved_cache_bytes
                    .checked_add(decoded_bytes)
                    .ok_or_else(|| Error::invariant("page cache reservation overflow"))?;
                let ticket = Arc::new(LoadTicket::new());
                state.in_flight.insert(key.clone(), Arc::clone(&ticket));
                state.active_loads += 1;
                (ticket, true)
            }
        };
        if !is_leader {
            let result = ticket.wait().map(PageGuard);
            if let Ok(mut state) = self.state.lock() {
                state.active_followers = state.active_followers.saturating_sub(1);
            }
            ticket.followers.fetch_sub(1, Ordering::AcqRel);
            return result;
        }

        let mut leader = LoadLeader {
            loader: self,
            key: key.clone(),
            ticket: Arc::clone(&ticket),
            reserved_cache_bytes: decoded_bytes,
            finished: false,
        };
        let outcome = (|| {
            let reconstruction_reservation = self
                .resources
                .reserve(
                    self.tenant_id,
                    ResourceCategory::LoadReconstruction,
                    ReservationClass::Foreground,
                    encoded_bytes,
                )
                .map_err(map_resource_error)?;
            let decoded_reservation = self
                .resources
                .reserve(
                    self.tenant_id,
                    ResourceCategory::DecodedPayload,
                    ReservationClass::Foreground,
                    decoded_bytes,
                )
                .map_err(map_resource_error)?;
            let value = load()?;
            drop(reconstruction_reservation);
            let candidate = Arc::new(CachedValue {
                value,
                bytes: decoded_bytes,
                _reservation: decoded_reservation,
            });
            let mut state = self
                .state
                .lock()
                .map_err(|_| Error::invariant("page cache lock poisoned"))?;
            if state.closed {
                return Err(Error::overloaded("paged loader shut down during a load"));
            }
            state.reserved_cache_bytes = state
                .reserved_cache_bytes
                .checked_sub(decoded_bytes)
                .ok_or_else(|| Error::invariant("page cache reservation underflow"))?;
            leader.reserved_cache_bytes = 0;
            state.cache_bytes = state
                .cache_bytes
                .checked_add(decoded_bytes)
                .ok_or_else(|| Error::invariant("page cache byte count overflow"))?;
            state.entries.insert(key.clone(), Arc::clone(&candidate));
            touch_lru(&mut state.lru, key.clone());
            drop(state);
            Ok(candidate)
        })();
        match outcome {
            Ok(candidate) => {
                leader.finish(Ok(Arc::clone(&candidate)));
                Ok(PageGuard(candidate))
            }
            Err(error) => {
                leader.finish(Err(LoadFailure::from_error(&error)));
                Err(error)
            }
        }
    }

    pub fn evict(&self, key: &K) -> Result<bool> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| Error::invariant("page cache lock poisoned"))?;
        let Some(value) = state.entries.remove(key) else {
            return Ok(false);
        };
        state.lru.retain(|candidate| candidate != key);
        state.cache_bytes = state.cache_bytes.saturating_sub(value.bytes);
        drop(state);
        drop(value);
        Ok(true)
    }

    pub fn source_read_count(&self) -> u64 {
        self.source_reads.load(Ordering::Relaxed)
    }

    pub fn close(&self) -> Result<()> {
        let tickets = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| Error::invariant("page cache lock poisoned"))?;
            state.closed = true;
            state.entries.clear();
            state.lru.clear();
            state.cache_bytes = 0;
            state.in_flight.values().cloned().collect::<Vec<_>>()
        };
        for ticket in tickets {
            ticket.complete(Err(LoadFailure::Shutdown));
        }
        Ok(())
    }

    #[cfg(test)]
    fn snapshot(&self) -> (u64, usize, usize, usize) {
        let state = self.state.lock().unwrap();
        (
            state.cache_bytes,
            state.active_loads,
            state.active_followers,
            state.entries.len(),
        )
    }
}

impl PageLoader<PageVersionKey, super::BlinkPage> {
    pub(crate) fn load_version<F>(
        &self,
        index: &VersionIndex,
        root: VersionIndexRoot,
        page_id: PageId,
        sources: &SourceManager,
        header_length: F,
    ) -> Result<PageGuard<super::BlinkPage>>
    where
        F: FnMut(SourceIdentity) -> Result<usize>,
    {
        let descriptor = index
            .lookup(root, page_id)?
            .ok_or_else(|| Error::corruption("version index has no page descriptor"))?;
        self.load_descriptor(index, descriptor, sources, header_length)
    }

    pub(crate) fn load_descriptor<F>(
        &self,
        index: &VersionIndex,
        descriptor: VersionDescriptor,
        sources: &SourceManager,
        mut header_length: F,
    ) -> Result<PageGuard<super::BlinkPage>>
    where
        F: FnMut(SourceIdentity) -> Result<usize>,
    {
        let page_id = descriptor.page_id;
        let chain = match descriptor.locator {
            VersionLocator::CheckpointImage {
                source,
                offset,
                source_length,
                image_lsn,
                image_crc32c,
            } => DeltaChain::new(
                page_id,
                vec![DeltaChainEntry::CheckpointImage {
                    source,
                    offset,
                    source_length,
                    image_lsn,
                    image_crc32c,
                }],
            )?,
            VersionLocator::WalImage { frame } => {
                let VersionToken::Wal { record_lsn, .. } = descriptor.token else {
                    return Err(Error::corruption(
                        "WAL page image descriptor has a non-WAL token",
                    ));
                };
                DeltaChain::new(
                    page_id,
                    vec![DeltaChainEntry::WalImage {
                        frame,
                        record_lsn,
                        image_lsn: descriptor.image_lsn,
                        image_crc32c: descriptor.image_crc32c,
                    }],
                )?
            }
            VersionLocator::WalDelta { chain, .. } => index.delta_chains().read(chain, page_id)?,
        };
        let final_entry = chain
            .entries
            .last()
            .copied()
            .ok_or_else(|| Error::corruption("page version chain is empty"))?;
        if final_entry.image_lsn() != descriptor.image_lsn
            || final_entry.image_crc32c() != descriptor.image_crc32c
        {
            return Err(Error::corruption(
                "page version chain does not match its index descriptor",
            ));
        }
        let key = PageVersionKey::from_descriptor(index.store_identity(), descriptor);
        let encoded_bytes = chain.entries.iter().try_fold(0u64, |total, entry| {
            let bytes = match entry {
                DeltaChainEntry::CheckpointImage { .. } => crate::page::PAGE_SIZE as u64,
                DeltaChainEntry::WalImage { frame, .. }
                | DeltaChainEntry::WalDelta { frame, .. } => u64::from(frame.frame_length),
            };
            total
                .checked_add(bytes)
                .ok_or_else(|| Error::overloaded("page reconstruction size overflows"))
        })?;
        self.get_or_load(key, encoded_bytes, crate::page::PAGE_SIZE as u64, || {
            let image = chain.reconstruct(|entry| match entry {
                DeltaChainEntry::CheckpointImage { source, offset, .. } => {
                    let mut lease = sources.acquire(source, header_length(source)?)?;
                    let mut image = vec![0; crate::page::PAGE_SIZE];
                    lease.read_exact_at(offset, &mut image)?;
                    self.source_reads.fetch_add(1, Ordering::Relaxed);
                    Ok(image)
                }
                DeltaChainEntry::WalImage {
                    frame, record_lsn, ..
                } => read_wal_payload(
                    sources,
                    &mut header_length,
                    &self.source_reads,
                    frame,
                    page_id,
                    record_lsn,
                    false,
                ),
                DeltaChainEntry::WalDelta {
                    frame, record_lsn, ..
                } => read_wal_payload(
                    sources,
                    &mut header_length,
                    &self.source_reads,
                    frame,
                    page_id,
                    record_lsn,
                    true,
                ),
            })?;
            super::decode_blink_page(&image, page_id)
        })
    }
}

fn read_wal_payload<F>(
    sources: &SourceManager,
    header_length: &mut F,
    source_reads: &AtomicU64,
    frame: WalFrameLocator,
    page_id: PageId,
    record_lsn: Lsn,
    is_delta: bool,
) -> Result<Vec<u8>>
where
    F: FnMut(SourceIdentity) -> Result<usize>,
{
    let frame_length = usize::try_from(frame.frame_length)
        .map_err(|_| Error::corruption("indexed WAL frame length is invalid"))?;
    let mut lease: SourceLease = sources.acquire(frame.source, header_length(frame.source)?)?;
    let mut bytes = vec![0; frame_length];
    lease.read_exact_at(frame.frame_offset, &mut bytes)?;
    source_reads.fetch_add(1, Ordering::Relaxed);
    crate::wal::validate_paged_frame_bytes(&bytes, frame, page_id, record_lsn, is_delta)
}

struct LoadLeader<'a, K, V>
where
    K: Clone + Eq + Hash,
    V: Send + Sync + 'static,
{
    loader: &'a PageLoader<K, V>,
    key: K,
    ticket: Arc<LoadTicket<V>>,
    reserved_cache_bytes: u64,
    finished: bool,
}

impl<K, V> LoadLeader<'_, K, V>
where
    K: Clone + Eq + Hash,
    V: Send + Sync + 'static,
{
    fn finish(&mut self, outcome: std::result::Result<Arc<CachedValue<V>>, LoadFailure>) {
        self.ticket.complete(outcome);
        if let Ok(mut state) = self.loader.state.lock() {
            state.in_flight.remove(&self.key);
            state.active_loads = state.active_loads.saturating_sub(1);
            state.reserved_cache_bytes = state
                .reserved_cache_bytes
                .saturating_sub(self.reserved_cache_bytes);
        }
        self.finished = true;
    }
}

impl<K, V> Drop for LoadLeader<'_, K, V>
where
    K: Clone + Eq + Hash,
    V: Send + Sync + 'static,
{
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.ticket.complete(Err(LoadFailure::Cancelled));
        if let Ok(mut state) = self.loader.state.lock() {
            state.in_flight.remove(&self.key);
            state.active_loads = state.active_loads.saturating_sub(1);
            state.reserved_cache_bytes = state
                .reserved_cache_bytes
                .saturating_sub(self.reserved_cache_bytes);
        }
    }
}

fn evict_to_fit<K, V>(
    state: &mut CacheState<K, V>,
    maximum_bytes: u64,
    needed_bytes: u64,
) -> Result<()>
where
    K: Clone + Eq + Hash,
{
    while state
        .cache_bytes
        .saturating_add(state.reserved_cache_bytes)
        .saturating_add(needed_bytes)
        > maximum_bytes
    {
        let Some(key) = state.lru.pop_front() else {
            return Err(Error::overloaded(
                "paged cache capacity is held by active page guards",
            ));
        };
        let Some(value) = state.entries.get(&key) else {
            continue;
        };
        if Arc::strong_count(value) != 1 {
            state.lru.push_back(key);
            if state.lru.iter().all(|candidate| {
                state
                    .entries
                    .get(candidate)
                    .is_none_or(|entry| Arc::strong_count(entry) != 1)
            }) {
                return Err(Error::overloaded(
                    "paged cache capacity is held by active page guards",
                ));
            }
            continue;
        }
        let value = state.entries.remove(&key).unwrap();
        state.cache_bytes = state.cache_bytes.saturating_sub(value.bytes);
        drop(value);
    }
    Ok(())
}

fn touch_lru<K: Clone + Eq>(lru: &mut VecDeque<K>, key: K) {
    lru.retain(|candidate| candidate != &key);
    lru.push_back(key);
}

enum LoadFailure {
    Io(ErrorKind, String),
    Corruption(String),
    Overloaded(String),
    Cancelled,
    Shutdown,
    Other(String),
}

impl LoadFailure {
    fn from_error(error: &Error) -> Self {
        match error {
            Error::Io(error) => Self::Io(error.kind(), error.to_string()),
            Error::Corruption(message) => Self::Corruption(message.clone()),
            Error::Overloaded(message) => Self::Overloaded(message.clone()),
            _ => Self::Other(error.to_string()),
        }
    }

    fn to_error(&self) -> Error {
        match self {
            Self::Io(kind, message) => Error::Io(std::io::Error::new(*kind, message.clone())),
            Self::Corruption(message) => Error::corruption(message.clone()),
            Self::Overloaded(message) => Error::overloaded(message.clone()),
            Self::Cancelled => Error::recovery("coalesced page load leader was cancelled"),
            Self::Shutdown => Error::overloaded("paged loader shut down during a load"),
            Self::Other(message) => {
                Error::recovery(format!("coalesced page load failed: {message}"))
            }
        }
    }
}

pub(super) fn map_resource_error(error: ResourceError) -> Error {
    match error {
        ResourceError::CapacityExceeded | ResourceError::TooManyTenants => {
            Error::overloaded("paged load resource capacity exceeded")
        }
        ResourceError::TooManyWaiters => Error::overloaded("paged load waiter limit exceeded"),
        ResourceError::WaitTimedOut => Error::overloaded("paged load resource wait timed out"),
        ResourceError::InvalidLimits => Error::invalid_input("paged load resource limits invalid"),
        ResourceError::ArithmeticOverflow => Error::invariant("paged load byte count overflow"),
        ResourceError::SharedOwnership => Error::invariant("paged load reservation is shared"),
        ResourceError::Poisoned => Error::invariant("paged load resource manager lock poisoned"),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    use super::{PageLoader, PageLoaderConfig, PageVersionKey};
    use crate::blink::resource::{BlinkResourceManager, ResourceCategory, ResourceLimits};
    use crate::blink::source_manager::{SourceManager, SourceManagerConfig};
    use crate::blink::version_index::{
        SourceFileKind, SourceIdentity, StoreIdentity, VersionDescriptor, VersionIndex,
        VersionLocator, VersionToken, WalFrameLocator,
    };
    use crate::blink::{BlinkPage, OwnedBlinkPageGuard};
    use dodb_core::{Lsn, PageId};

    fn resources(decoded_limit: u64, load_limit: u64) -> BlinkResourceManager {
        let mut global_category_bytes = [1024; 7];
        let mut per_tenant_category_bytes = [1024; 7];
        global_category_bytes[ResourceCategory::DecodedPayload as usize] = decoded_limit;
        global_category_bytes[ResourceCategory::LoadReconstruction as usize] = load_limit;
        per_tenant_category_bytes[ResourceCategory::DecodedPayload as usize] = decoded_limit;
        per_tenant_category_bytes[ResourceCategory::LoadReconstruction as usize] = load_limit;
        BlinkResourceManager::new(ResourceLimits {
            global_bytes: 4096,
            per_tenant_bytes: 4096,
            progress_pool_bytes: 512,
            max_tenants: 4,
            max_pending_waiters: 64,
            global_category_bytes,
            per_tenant_category_bytes,
        })
        .unwrap()
    }

    fn loader(maximum_cache_bytes: u64) -> PageLoader<u64, Vec<u8>> {
        PageLoader::new(
            1,
            resources(1024, 1024),
            PageLoaderConfig {
                maximum_cache_bytes,
                minimum_decoded_page_bytes: 8,
                maximum_load_reconstruction_bytes: 64,
                maximum_concurrent_loads: 2,
                maximum_followers: 16,
            },
        )
        .unwrap()
    }

    fn integration_resources() -> BlinkResourceManager {
        BlinkResourceManager::new(ResourceLimits {
            global_bytes: 1_048_576,
            per_tenant_bytes: 1_048_576,
            progress_pool_bytes: 65_536,
            max_tenants: 2,
            max_pending_waiters: 8,
            global_category_bytes: [1_048_576; 7],
            per_tenant_category_bytes: [1_048_576; 7],
        })
        .unwrap()
    }

    fn write_checkpoint_source(
        directory: &Path,
        file_number: u64,
        image: &[u8; crate::page::PAGE_SIZE],
    ) -> (SourceIdentity, usize, u64) {
        let header = vec![file_number as u8; crate::page::PAGE_SIZE];
        let identity = SourceIdentity {
            kind: SourceFileKind::CheckpointPack,
            file_number,
            header_identity: *blake3::hash(&header).as_bytes(),
        };
        let source_path = directory.join(format!("base-{file_number:020}.pages"));
        let mut file = fs::File::create(source_path).unwrap();
        use std::io::Write;
        file.write_all(&header).unwrap();
        file.write_all(image).unwrap();
        (identity, header.len(), (header.len() + image.len()) as u64)
    }

    #[test]
    fn current_and_pinned_old_versions_read_exact_bytes_after_cache_eviction() {
        let root_directory = std::env::temp_dir().join(format!(
            "dodb-page-cache-source-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let checkpoint_directory = root_directory.join("packs");
        let wal_directory = root_directory.join("segments");
        let scratch_directory = root_directory.join("scratch");
        let index_directory = root_directory.join("index");
        fs::create_dir_all(&checkpoint_directory).unwrap();
        fs::create_dir_all(&wal_directory).unwrap();
        let store = StoreIdentity {
            database_uuid: [41; 16],
            wal_epoch: 6,
        };
        let page_id = PageId::new(5);
        let old_image = crate::blink::test_blink_page_image(page_id, Lsn::new(10)).unwrap();
        let current_image = crate::blink::test_blink_page_image(page_id, Lsn::new(12)).unwrap();
        let (old_source, old_header_length, old_source_length) =
            write_checkpoint_source(&checkpoint_directory, 1, &old_image);
        let (current_source, current_header_length, current_source_length) =
            write_checkpoint_source(&checkpoint_directory, 2, &current_image);
        let resources = integration_resources();
        let index = VersionIndex::open(&index_directory, store, resources.clone(), 1, 2).unwrap();
        let old_root = index
            .insert(
                index.empty_root(),
                page_id,
                VersionDescriptor {
                    page_id,
                    token: VersionToken::Bootstrap { page_id },
                    image_lsn: Lsn::new(10),
                    image_crc32c: crc32c::crc32c(&old_image),
                    locator: VersionLocator::CheckpointImage {
                        source: old_source,
                        offset: crate::page::PAGE_SIZE as u64,
                        source_length: old_source_length,
                        image_lsn: Lsn::new(10),
                        image_crc32c: crc32c::crc32c(&old_image),
                    },
                },
            )
            .unwrap();
        let current_root = index
            .insert(
                old_root,
                page_id,
                VersionDescriptor {
                    page_id,
                    token: VersionToken::Bootstrap { page_id },
                    image_lsn: Lsn::new(12),
                    image_crc32c: crc32c::crc32c(&current_image),
                    locator: VersionLocator::CheckpointImage {
                        source: current_source,
                        offset: crate::page::PAGE_SIZE as u64,
                        source_length: current_source_length,
                        image_lsn: Lsn::new(12),
                        image_crc32c: crc32c::crc32c(&current_image),
                    },
                },
            )
            .unwrap();
        let sources = SourceManager::new(
            SourceManagerConfig {
                store_identity: store,
                checkpoint_directory,
                wal_directory,
                scratch_directory,
                tenant_id: 1,
                maximum_open_handles: 2,
                maximum_roots: 4,
                handle_charge_bytes: 128,
                gc_reservation_bytes: 8192,
            },
            resources.clone(),
        )
        .unwrap();
        let page_loader = PageLoader::new(
            1,
            resources,
            PageLoaderConfig {
                maximum_cache_bytes: crate::page::PAGE_SIZE as u64,
                minimum_decoded_page_bytes: crate::page::PAGE_SIZE as u64,
                maximum_load_reconstruction_bytes: crate::page::PAGE_SIZE as u64,
                maximum_concurrent_loads: 1,
                maximum_followers: 4,
            },
        )
        .unwrap();
        let old_guard = page_loader
            .load_version(&index, old_root, page_id, &sources, |source| {
                Ok(if source == old_source {
                    old_header_length
                } else {
                    current_header_length
                })
            })
            .unwrap();
        assert!(matches!(
            old_guard.as_ref(),
            BlinkPage::Leaf { lsn, entries, .. } if *lsn == Lsn::new(10) && entries.len() == 0
        ));
        let old_key = PageVersionKey::from_descriptor(
            store,
            index.lookup(old_root, page_id).unwrap().unwrap(),
        );
        page_loader.evict(&old_key).unwrap();
        drop(old_guard);
        let current_guard = page_loader
            .load_version(&index, current_root, page_id, &sources, |source| {
                Ok(if source == old_source {
                    old_header_length
                } else {
                    current_header_length
                })
            })
            .unwrap();
        assert!(matches!(
            current_guard.as_ref(),
            BlinkPage::Leaf { lsn, entries, .. } if *lsn == Lsn::new(12) && entries.len() == 0
        ));
        page_loader
            .evict(&PageVersionKey::from_descriptor(
                store,
                index.lookup(current_root, page_id).unwrap().unwrap(),
            ))
            .unwrap();
        drop(current_guard);
        let reloaded_old_guard = page_loader
            .load_version(&index, old_root, page_id, &sources, |source| {
                Ok(if source == old_source {
                    old_header_length
                } else {
                    current_header_length
                })
            })
            .unwrap();
        assert!(matches!(
            reloaded_old_guard.as_ref(),
            BlinkPage::Leaf { lsn, entries, .. } if *lsn == Lsn::new(10) && entries.len() == 0
        ));
        fs::remove_dir_all(root_directory).unwrap();
    }

    #[test]
    fn same_version_concurrent_misses_coalesce_one_load() {
        let loader = Arc::new(loader(32));
        let barrier = Arc::new(Barrier::new(8));
        let load_count = Arc::new(AtomicUsize::new(0));
        let threads = (0..8)
            .map(|_| {
                let loader = Arc::clone(&loader);
                let barrier = Arc::clone(&barrier);
                let load_count = Arc::clone(&load_count);
                thread::spawn(move || {
                    barrier.wait();
                    loader
                        .get_or_load(7, 4, 8, || {
                            load_count.fetch_add(1, Ordering::SeqCst);
                            thread::sleep(Duration::from_millis(20));
                            Ok(vec![9; 8])
                        })
                        .unwrap()
                })
            })
            .collect::<Vec<_>>();
        let pages = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(load_count.load(Ordering::SeqCst), 1);
        assert!(pages.iter().all(|page| page.as_ref() == &vec![9; 8]));
    }

    #[test]
    fn distinct_versions_do_not_coalesce() {
        let loader = loader(32);
        let load_count = AtomicUsize::new(0);
        for version_key in [11, 12] {
            loader
                .get_or_load(version_key, 4, 8, || {
                    load_count.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![version_key as u8; 8])
                })
                .unwrap();
        }
        assert_eq!(load_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn cache_eviction_keeps_guard_charge_until_last_owner_drops() {
        let resource_manager = resources(64, 64);
        let page_loader = PageLoader::new(
            1,
            resource_manager.clone(),
            PageLoaderConfig {
                maximum_cache_bytes: 8,
                minimum_decoded_page_bytes: 8,
                maximum_load_reconstruction_bytes: 64,
                maximum_concurrent_loads: 1,
                maximum_followers: 1,
            },
        )
        .unwrap();
        let guard: OwnedBlinkPageGuard = page_loader
            .get_or_load(1, 4, 8, || {
                Ok(BlinkPage::Free {
                    lsn: Lsn::new(1),
                    next: None,
                })
            })
            .unwrap()
            .into();
        page_loader.evict(&1).unwrap();
        let snapshot = resource_manager.snapshot().unwrap();
        assert_eq!(
            snapshot.current_by_category[ResourceCategory::DecodedPayload as usize],
            8
        );
        assert!(matches!(guard.as_ref(), BlinkPage::Free { .. }));
        drop(guard);
        assert_eq!(
            resource_manager.snapshot().unwrap().current_by_category
                [ResourceCategory::DecodedPayload as usize],
            0
        );
    }

    #[test]
    fn all_pinned_cache_capacity_returns_overload() {
        let loader = loader(8);
        let guard = loader.get_or_load(1, 4, 8, || Ok(vec![1; 8])).unwrap();
        let result = loader.get_or_load(2, 4, 8, || Ok(vec![2; 8]));
        assert!(matches!(result, Err(dodb_core::Error::Overloaded(_))));
        assert_eq!(guard.as_ref(), &vec![1; 8]);
    }

    #[test]
    fn concurrent_loads_reserve_cache_bytes_before_decoding() {
        let loader = Arc::new(loader(8));
        let load_entered = Arc::new(Barrier::new(2));
        let load_release = Arc::new(Barrier::new(2));
        let leader_loader = Arc::clone(&loader);
        let leader_entered = Arc::clone(&load_entered);
        let leader_release = Arc::clone(&load_release);
        let leader = thread::spawn(move || {
            leader_loader.get_or_load(1, 4, 8, || {
                leader_entered.wait();
                leader_release.wait();
                Ok(vec![1; 8])
            })
        });
        load_entered.wait();
        let second_load_started = AtomicUsize::new(0);
        let second_load = loader.get_or_load(2, 4, 8, || {
            second_load_started.fetch_add(1, Ordering::SeqCst);
            Ok(vec![2; 8])
        });
        assert!(matches!(second_load, Err(dodb_core::Error::Overloaded(_))));
        assert_eq!(second_load_started.load(Ordering::SeqCst), 0);
        load_release.wait();
        assert!(leader.join().unwrap().is_ok());
        assert_eq!(loader.snapshot().1, 0);
    }

    #[test]
    fn load_and_decoded_payload_charges_overlap_until_source_reconstruction_finishes() {
        let resource_manager = resources(64, 64);
        let loader = PageLoader::new(
            1,
            resource_manager.clone(),
            PageLoaderConfig {
                maximum_cache_bytes: 16,
                minimum_decoded_page_bytes: 8,
                maximum_load_reconstruction_bytes: 64,
                maximum_concurrent_loads: 1,
                maximum_followers: 1,
            },
        )
        .unwrap();
        let page = loader
            .get_or_load(1, 4, 8, || {
                let snapshot = resource_manager.snapshot().unwrap();
                assert_eq!(
                    snapshot.current_by_category[ResourceCategory::LoadReconstruction as usize],
                    4
                );
                assert_eq!(
                    snapshot.current_by_category[ResourceCategory::DecodedPayload as usize],
                    8
                );
                Ok(vec![1; 8])
            })
            .unwrap();
        let snapshot = resource_manager.snapshot().unwrap();
        assert_eq!(
            snapshot.current_by_category[ResourceCategory::LoadReconstruction as usize],
            0
        );
        assert_eq!(
            snapshot.current_by_category[ResourceCategory::DecodedPayload as usize],
            8
        );
        drop(page);
    }

    #[test]
    fn load_failures_are_shared_and_permits_are_released() {
        let loader = loader(16);
        let result = loader.get_or_load(1, 4, 8, || Err(dodb_core::Error::corruption("bad page")));
        assert!(matches!(result, Err(dodb_core::Error::Corruption(_))));
        assert_eq!(loader.snapshot().1, 0);
        let recovered = loader.get_or_load(1, 4, 8, || Ok(vec![3; 8])).unwrap();
        assert_eq!(recovered.as_ref(), &vec![3; 8]);
    }

    #[test]
    fn coalesced_followers_receive_source_failure_and_loader_slot_recovers() {
        let loader = Arc::new(loader(16));
        let load_entered = Arc::new(Barrier::new(2));
        let load_release = Arc::new(Barrier::new(2));
        let leader_loader = Arc::clone(&loader);
        let leader_entered = Arc::clone(&load_entered);
        let leader_release = Arc::clone(&load_release);
        let leader = thread::spawn(move || {
            leader_loader.get_or_load(1, 4, 8, || {
                leader_entered.wait();
                leader_release.wait();
                Err(dodb_core::Error::Io(std::io::Error::other(
                    "source read failed",
                )))
            })
        });
        load_entered.wait();
        let follower_loader = Arc::clone(&loader);
        let follower =
            thread::spawn(move || follower_loader.get_or_load(1, 4, 8, || Ok(vec![1; 8])));
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while loader.snapshot().2 == 0 && std::time::Instant::now() < deadline {
            thread::yield_now();
        }
        assert_eq!(loader.snapshot().2, 1);
        load_release.wait();
        assert!(matches!(
            leader.join().unwrap(),
            Err(dodb_core::Error::Io(_))
        ));
        assert!(matches!(
            follower.join().unwrap(),
            Err(dodb_core::Error::Io(_))
        ));
        assert_eq!(loader.snapshot().1, 0);
        assert_eq!(loader.snapshot().2, 0);
        assert!(loader.get_or_load(1, 4, 8, || Ok(vec![2; 8])).is_ok());
    }

    #[test]
    fn cancelled_load_wakes_followers_and_releases_job_and_byte_reservations() {
        let resource_manager = resources(64, 64);
        let loader = Arc::new(
            PageLoader::new(
                1,
                resource_manager.clone(),
                PageLoaderConfig {
                    maximum_cache_bytes: 16,
                    minimum_decoded_page_bytes: 8,
                    maximum_load_reconstruction_bytes: 64,
                    maximum_concurrent_loads: 1,
                    maximum_followers: 2,
                },
            )
            .unwrap(),
        );
        let load_entered = Arc::new(Barrier::new(2));
        let load_release = Arc::new(Barrier::new(2));
        let leader_loader = Arc::clone(&loader);
        let leader_entered = Arc::clone(&load_entered);
        let leader_release = Arc::clone(&load_release);
        let leader = thread::spawn(move || {
            leader_loader.get_or_load(1, 4, 8, || {
                leader_entered.wait();
                leader_release.wait();
                panic!("injected loader cancellation")
            })
        });
        load_entered.wait();
        let follower_loader = Arc::clone(&loader);
        let follower =
            thread::spawn(move || follower_loader.get_or_load(1, 4, 8, || Ok(vec![1; 8])));
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while loader.snapshot().2 == 0 && std::time::Instant::now() < deadline {
            thread::yield_now();
        }
        assert_eq!(loader.snapshot().2, 1);
        load_release.wait();
        assert!(leader.join().is_err());
        assert!(matches!(
            follower.join().unwrap(),
            Err(dodb_core::Error::RecoveryFailure(_))
        ));
        assert_eq!(loader.snapshot().1, 0);
        assert_eq!(loader.snapshot().2, 0);
        assert_eq!(resource_manager.snapshot().unwrap().current_total, 0);
        assert!(loader.get_or_load(1, 4, 8, || Ok(vec![3; 8])).is_ok());
    }

    #[test]
    fn shutdown_wakes_followers_and_rejects_new_loads() {
        let loader = Arc::new(loader(16));
        let load_entered = Arc::new(Barrier::new(2));
        let load_release = Arc::new(Barrier::new(2));
        let leader_loader = Arc::clone(&loader);
        let leader_entered = Arc::clone(&load_entered);
        let leader_release = Arc::clone(&load_release);
        let leader = thread::spawn(move || {
            leader_loader.get_or_load(1, 4, 8, || {
                leader_entered.wait();
                leader_release.wait();
                Ok(vec![1; 8])
            })
        });
        load_entered.wait();
        let follower_loader = Arc::clone(&loader);
        let follower =
            thread::spawn(move || follower_loader.get_or_load(1, 4, 8, || Ok(vec![1; 8])));
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while loader.snapshot().2 == 0 && std::time::Instant::now() < deadline {
            thread::yield_now();
        }
        assert_eq!(loader.snapshot().2, 1);
        loader.close().unwrap();
        assert!(matches!(
            follower.join().unwrap(),
            Err(dodb_core::Error::Overloaded(_))
        ));
        load_release.wait();
        assert!(matches!(
            leader.join().unwrap(),
            Err(dodb_core::Error::Overloaded(_))
        ));
        assert!(matches!(
            loader.get_or_load(2, 4, 8, || Ok(vec![2; 8])),
            Err(dodb_core::Error::Overloaded(_))
        ));
    }

    #[test]
    fn configuration_rejects_capacity_below_one_legal_load() {
        let result = PageLoader::<u64, Vec<u8>>::new(
            1,
            resources(64, 64),
            PageLoaderConfig {
                maximum_cache_bytes: 7,
                minimum_decoded_page_bytes: 8,
                maximum_load_reconstruction_bytes: 64,
                maximum_concurrent_loads: 1,
                maximum_followers: 1,
            },
        );
        assert!(matches!(result, Err(dodb_core::Error::InvalidInput(_))));
    }

    #[test]
    fn version_key_binds_store_and_source_version() {
        let descriptor = VersionDescriptor {
            page_id: PageId::new(5),
            token: VersionToken::Wal {
                source_file_number: 7,
                record_lsn: Lsn::new(11),
            },
            image_lsn: Lsn::new(11),
            image_crc32c: 13,
            locator: VersionLocator::WalImage {
                frame: WalFrameLocator {
                    source: SourceIdentity {
                        kind: SourceFileKind::WalSegment,
                        file_number: 7,
                        header_identity: [4; 32],
                    },
                    frame_offset: 128,
                    frame_length: 64,
                    header_crc32c: 2,
                    payload_offset: 160,
                    payload_length: 32,
                    payload_crc32c: 13,
                    committed_prefix_length: 1024,
                    commit_frame_offset: 900,
                    commit_lsn: Lsn::new(11),
                },
            },
        };
        let store = StoreIdentity {
            database_uuid: [1; 16],
            wal_epoch: 3,
        };
        let first_key = PageVersionKey::from_descriptor(store, descriptor);
        let foreign_key = PageVersionKey::from_descriptor(
            StoreIdentity {
                database_uuid: [2; 16],
                wal_epoch: 3,
            },
            descriptor,
        );
        assert_ne!(first_key, foreign_key);
    }
}
