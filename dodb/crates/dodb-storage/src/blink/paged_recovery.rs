use std::fs::File;
use std::io::Read;
use std::path::Path;

use dodb_core::{Error, Lsn, Result};

use crate::durable_file::DurableFile;
use crate::wal::{
    WalIdentity, WalLocationRecoverySink, WalLocationScanSummary, WalLog, WalPageImage,
    WalPageImageFormat, WalPageLocation, WalPageRecoveryBase, WalRecordType,
};

use super::decode_blink_page;
use super::durable_bases::{BasePackEntry, scan_pack};
use super::paged_manifest::PagedManifest;
use super::version_chain::DeltaChain;
use super::version_chain::DeltaChainEntry;
use super::version_index::{
    SourceFileKind, SourceIdentity, VersionDescriptor, VersionIndex, VersionIndexRoot,
    VersionLocator, VersionToken, WalFrameLocator,
};
#[cfg(feature = "blink-demand-paging")]
use super::{PagedReadConfig, PagedReadHandle, page_cache, resource, source_manager};
#[cfg(feature = "blink-demand-paging")]
use std::sync::Arc;

#[cfg(feature = "blink-demand-paging")]
pub struct PagedRecoveryValidationConfig<F> {
    pub tenant_id: u64,
    pub root_page_id: dodb_core::PageId,
    pub free_list_head: Option<dodb_core::PageId>,
    pub high_water_page_id: dodb_core::PageId,
    pub resources: resource::BlinkResourceManager,
    pub sources: source_manager::SourceManager,
    pub loader: page_cache::PageLoaderConfig,
    pub header_length: F,
}

pub fn build_checkpoint_index(
    index: &VersionIndex,
    base_root: VersionIndexRoot,
    pack_path: &Path,
    pack_entry: &BasePackEntry,
    complete: bool,
    wal_identity: &WalIdentity,
    high_water_page_id: dodb_core::PageId,
) -> Result<(VersionIndexRoot, SourceIdentity)> {
    if !complete {
        return Err(Error::invalid_input(
            "paged checkpoint index requires a complete pack",
        ));
    }
    let checkpoint_insert_reservation = index.reserve_checkpoint_insert_work()?;
    let mut file = File::open(pack_path)?;
    let mut header = [0u8; crate::page::PAGE_SIZE];
    file.read_exact(&mut header)?;
    let source = SourceIdentity {
        kind: SourceFileKind::CheckpointPack,
        file_number: pack_entry.number,
        header_identity: *blake3::hash(&header).as_bytes(),
    };
    let mut root = base_root;
    scan_pack(
        &mut file,
        pack_entry,
        wal_identity,
        high_water_page_id,
        |page_id, image_lsn, offset, image| {
            let image_crc32c = crc32c::crc32c(image);
            let token = match index.lookup(root, page_id)? {
                Some(previous)
                    if previous.image_lsn == image_lsn && previous.image_crc32c == image_crc32c =>
                {
                    previous.token
                }
                Some(_) => {
                    return Err(Error::corruption(
                        "checkpoint page differs from its indexed source version",
                    ));
                }
                None => VersionToken::Bootstrap { page_id },
            };
            root = index.insert_reserved(
                root,
                page_id,
                VersionDescriptor {
                    page_id,
                    token,
                    image_lsn,
                    image_crc32c,
                    locator: VersionLocator::CheckpointImage {
                        source,
                        offset,
                        source_length: pack_entry.bytes,
                        image_lsn,
                        image_crc32c,
                    },
                },
                &checkpoint_insert_reservation,
            )?;
            Ok(())
        },
    )?;
    drop(checkpoint_insert_reservation);
    Ok((index.promote_root_to_durable(root)?, source))
}

pub fn build_incremental_checkpoint_index(
    index: &VersionIndex,
    target_root: VersionIndexRoot,
    pack_path: &Path,
    pack_entry: &BasePackEntry,
    previous_checkpoint_lsn: Lsn,
    wal_identity: &WalIdentity,
    high_water_page_id: dodb_core::PageId,
) -> Result<(VersionIndexRoot, SourceIdentity)> {
    if pack_entry.checkpoint_lsn <= previous_checkpoint_lsn || pack_entry.page_count == 0 {
        return Err(Error::invalid_input(
            "incremental checkpoint pack target or page count is invalid",
        ));
    }
    let checkpoint_insert_reservation = index.reserve_checkpoint_insert_work()?;
    let mut expected_page_count = 0u64;
    index.visit_descriptors(target_root, &mut |descriptor| {
        if descriptor.image_lsn > pack_entry.checkpoint_lsn {
            return Err(Error::corruption(
                "incremental checkpoint target precedes an indexed page image",
            ));
        }
        if descriptor.image_lsn > previous_checkpoint_lsn {
            expected_page_count = expected_page_count
                .checked_add(1)
                .ok_or_else(|| Error::corruption("incremental checkpoint count overflows"))?;
        }
        Ok(())
    })?;
    if expected_page_count != pack_entry.page_count {
        return Err(Error::corruption(
            "incremental checkpoint pack omits or adds target-generation pages",
        ));
    }
    let mut file = File::open(pack_path)?;
    let mut header = [0u8; crate::page::PAGE_SIZE];
    file.read_exact(&mut header)?;
    let source = SourceIdentity {
        kind: SourceFileKind::CheckpointPack,
        file_number: pack_entry.number,
        header_identity: *blake3::hash(&header).as_bytes(),
    };
    let mut root = target_root;
    let mut indexed_page_count = 0u64;
    scan_pack(
        &mut file,
        pack_entry,
        wal_identity,
        high_water_page_id,
        |page_id, image_lsn, offset, image| {
            let image_crc32c = crc32c::crc32c(image);
            let previous = index
                .lookup(root, page_id)?
                .ok_or_else(|| Error::corruption("incremental pack page is absent from target"))?;
            if image_lsn <= previous_checkpoint_lsn
                || previous.image_lsn != image_lsn
                || previous.image_crc32c != image_crc32c
            {
                return Err(Error::corruption(
                    "incremental checkpoint image differs from its target descriptor",
                ));
            }
            root = index.insert_reserved(
                root,
                page_id,
                VersionDescriptor {
                    page_id,
                    token: previous.token,
                    image_lsn,
                    image_crc32c,
                    locator: VersionLocator::CheckpointImage {
                        source,
                        offset,
                        source_length: pack_entry.bytes,
                        image_lsn,
                        image_crc32c,
                    },
                },
                &checkpoint_insert_reservation,
            )?;
            indexed_page_count = indexed_page_count
                .checked_add(1)
                .ok_or_else(|| Error::corruption("incremental checkpoint count overflows"))?;
            Ok(())
        },
    )?;
    if indexed_page_count != expected_page_count {
        return Err(Error::corruption(
            "incremental checkpoint scan count differs from the target",
        ));
    }
    drop(checkpoint_insert_reservation);
    Ok((index.promote_root_to_durable(root)?, source))
}

pub struct PagedCheckpointGeneration<'generation> {
    pub target_root: VersionIndexRoot,
    pub latest_root: VersionIndexRoot,
    pub pack_path: &'generation Path,
    pub pack_entry: &'generation BasePackEntry,
    pub complete: bool,
    pub previous_checkpoint_lsn: Option<Lsn>,
    pub wal_identity: &'generation WalIdentity,
    pub high_water_page_id: dodb_core::PageId,
}

pub fn build_checkpoint_index_for_generation(
    index: &VersionIndex,
    generation: PagedCheckpointGeneration<'_>,
) -> Result<VersionIndexRoot> {
    let PagedCheckpointGeneration {
        target_root,
        latest_root,
        pack_path,
        pack_entry,
        complete,
        previous_checkpoint_lsn,
        wal_identity,
        high_water_page_id,
    } = generation;
    if pack_entry.page_count == 0
        || complete != previous_checkpoint_lsn.is_none()
        || previous_checkpoint_lsn
            .is_some_and(|previous_lsn| pack_entry.checkpoint_lsn <= previous_lsn)
    {
        return Err(Error::invalid_input(
            "checkpoint generation mode, target, or page count is invalid",
        ));
    }
    let checkpoint_insert_reservation = index.reserve_checkpoint_insert_work()?;
    let mut expected_page_count = 0u64;
    index.visit_descriptors(target_root, &mut |descriptor| {
        if descriptor.image_lsn > pack_entry.checkpoint_lsn {
            return Err(Error::corruption(
                "checkpoint pack target precedes an indexed page image",
            ));
        }
        if complete
            || previous_checkpoint_lsn
                .is_some_and(|previous_lsn| descriptor.image_lsn > previous_lsn)
        {
            expected_page_count = expected_page_count
                .checked_add(1)
                .ok_or_else(|| Error::corruption("checkpoint page count overflows"))?;
        }
        Ok(())
    })?;
    if expected_page_count != pack_entry.page_count {
        return Err(Error::corruption(
            "checkpoint pack omits or adds target-generation pages",
        ));
    }
    let mut file = File::open(pack_path)?;
    let mut header = [0u8; crate::page::PAGE_SIZE];
    file.read_exact(&mut header)?;
    let source = SourceIdentity {
        kind: SourceFileKind::CheckpointPack,
        file_number: pack_entry.number,
        header_identity: *blake3::hash(&header).as_bytes(),
    };
    let mut root = latest_root;
    let mut packed_page_count = 0u64;
    scan_pack(
        &mut file,
        pack_entry,
        wal_identity,
        high_water_page_id,
        |page_id, image_lsn, offset, image| {
            let image_crc32c = crc32c::crc32c(image);
            let target = index
                .lookup(target_root, page_id)?
                .ok_or_else(|| Error::corruption("checkpoint page is absent from target"))?;
            if target.image_lsn != image_lsn || target.image_crc32c != image_crc32c {
                return Err(Error::corruption(
                    "checkpoint image differs from its captured target version",
                ));
            }
            if previous_checkpoint_lsn.is_some_and(|previous_lsn| image_lsn <= previous_lsn) {
                return Err(Error::corruption(
                    "incremental checkpoint pack contains an old page version",
                ));
            }
            let latest = index
                .lookup(root, page_id)?
                .ok_or_else(|| Error::corruption("checkpoint page is absent from latest root"))?;
            match latest.image_lsn.cmp(&image_lsn) {
                std::cmp::Ordering::Greater => {}
                std::cmp::Ordering::Equal => {
                    if latest.token != target.token || latest.image_crc32c != image_crc32c {
                        return Err(Error::corruption(
                            "latest page version differs from the captured target token",
                        ));
                    }
                    root = index.insert_reserved(
                        root,
                        page_id,
                        VersionDescriptor {
                            page_id,
                            token: target.token,
                            image_lsn,
                            image_crc32c,
                            locator: VersionLocator::CheckpointImage {
                                source,
                                offset,
                                source_length: pack_entry.bytes,
                                image_lsn,
                                image_crc32c,
                            },
                        },
                        &checkpoint_insert_reservation,
                    )?;
                }
                std::cmp::Ordering::Less => {
                    return Err(Error::corruption(
                        "latest page version precedes the captured checkpoint image",
                    ));
                }
            }
            packed_page_count = packed_page_count
                .checked_add(1)
                .ok_or_else(|| Error::corruption("checkpoint scan count overflows"))?;
            Ok(())
        },
    )?;
    if packed_page_count != expected_page_count {
        return Err(Error::corruption(
            "checkpoint scan count differs from the captured target",
        ));
    }
    drop(checkpoint_insert_reservation);
    index.promote_root_to_durable(root)
}

pub struct PagedRecoveryIndexBuilder<'index> {
    index: &'index VersionIndex,
    root: VersionIndexRoot,
    manifest: PagedManifest,
    source: SourceIdentity,
    committed_prefix_length: u64,
    last_record_lsn: Option<Lsn>,
    last_commit_lsn: Option<Lsn>,
}

impl<'index> PagedRecoveryIndexBuilder<'index> {
    pub fn new(
        index: &'index VersionIndex,
        manifest: PagedManifest,
        source: SourceIdentity,
        committed_prefix_length: u64,
    ) -> Result<Self> {
        manifest.validate()?;
        if source.kind != SourceFileKind::WalSegment
            || source.file_number < manifest.recovery_wal_file_number
        {
            return Err(Error::invalid_input(
                "paged recovery WAL source precedes the manifest boundary",
            ));
        }
        let root = index.root_from_node_reference(manifest.durable_index_root)?;
        index.validate_durable_root(root)?;
        Ok(Self {
            index,
            root,
            manifest,
            source,
            committed_prefix_length,
            last_record_lsn: None,
            last_commit_lsn: None,
        })
    }

    pub fn current_root(&self) -> VersionIndexRoot {
        self.root
    }

    #[cfg(feature = "blink-demand-paging")]
    pub fn validate_recovered_pages<F>(
        &self,
        index: Arc<VersionIndex>,
        config: PagedRecoveryValidationConfig<F>,
    ) -> Result<u64>
    where
        F: Fn(SourceIdentity) -> Result<usize> + Send + Sync + 'static,
    {
        if !std::ptr::eq(Arc::as_ptr(&index), self.index) {
            return Err(Error::invalid_input(
                "paged recovery validation must use the builder index",
            ));
        }
        let handle = PagedReadHandle::new(
            PagedReadConfig {
                tenant_id: config.tenant_id,
                root: self.root,
                root_page_id: config.root_page_id,
                free_list_head: config.free_list_head,
                high_water_page_id: config.high_water_page_id,
                loader: config.loader,
            },
            index,
            config.resources,
            config.sources,
            config.header_length,
        )?;
        handle.validate_all_indexed_pages()
    }

    pub fn apply_committed_locations<I>(&mut self, locations: I) -> Result<usize>
    where
        I: IntoIterator<Item = WalPageLocation>,
    {
        let starting_state = self.staging_state();
        let result = (|| {
            let mut indexed_pages = 0usize;
            for location in locations {
                self.validate_committed_location(&location)?;
                if location.page_id.get() < 2 {
                    continue;
                }
                let descriptor = self.descriptor_for_location(&location)?;
                self.root = self.index.insert(self.root, location.page_id, descriptor)?;
                indexed_pages = indexed_pages
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("paged recovery page count overflow"))?;
            }
            Ok(indexed_pages)
        })();
        if result.is_err() {
            self.restore_staging_state(starting_state);
        }
        result
    }

    pub fn replay_wal<F, R>(
        &mut self,
        file: &mut F,
        identity: &WalIdentity,
        page_image_format: WalPageImageFormat,
        resolve_page: &mut R,
    ) -> Result<WalLocationScanSummary>
    where
        F: DurableFile,
        R: FnMut(VersionDescriptor) -> Result<WalPageImage>,
    {
        let starting_state = self.staging_state();
        let result = {
            let mut sink = PagedRecoverySink {
                builder: self,
                resolve_page,
            };
            WalLog::<F>::scan_committed_page_locations_streaming(
                file,
                identity,
                page_image_format,
                None,
                &mut sink,
            )
        };
        if result.is_err() {
            self.restore_staging_state(starting_state);
        }
        result
    }

    fn staging_state(&self) -> (VersionIndexRoot, u64, Option<Lsn>, Option<Lsn>) {
        (
            self.root,
            self.committed_prefix_length,
            self.last_record_lsn,
            self.last_commit_lsn,
        )
    }

    fn restore_staging_state(&mut self, state: (VersionIndexRoot, u64, Option<Lsn>, Option<Lsn>)) {
        self.root = state.0;
        self.committed_prefix_length = state.1;
        self.last_record_lsn = state.2;
        self.last_commit_lsn = state.3;
    }

    fn validate_committed_location(&mut self, location: &WalPageLocation) -> Result<()> {
        if location.source_identity.database_uuid != self.manifest.identity.database_uuid
            || location.source_identity.shard_epoch.get() != self.manifest.identity.wal_epoch
            || location.record_lsn <= self.manifest.committed_prefix_lsn
            || location.record_lsn == Lsn::ZERO
            || location.commit_lsn < location.record_lsn
            || location.frame_offset < self.manifest.recovery_offset
                && self.source.file_number == self.manifest.recovery_wal_file_number
        {
            return Err(Error::corruption(
                "paged recovery location is outside the manifest WAL boundary",
            ));
        }
        if let Some(previous_record_lsn) = self.last_record_lsn
            && location.record_lsn <= previous_record_lsn
        {
            return Err(Error::corruption(
                "paged recovery WAL record LSNs are not strictly increasing",
            ));
        }
        if let Some(previous_commit_lsn) = self.last_commit_lsn
            && location.commit_lsn < previous_commit_lsn
        {
            return Err(Error::corruption(
                "paged recovery commit LSNs are not monotonic",
            ));
        }
        let frame_end = location
            .frame_offset
            .checked_add(u64::from(location.frame_length))
            .ok_or_else(|| Error::corruption("paged recovery frame range overflows"))?;
        let payload_end = location
            .payload_offset
            .checked_add(u64::from(location.payload_length))
            .ok_or_else(|| Error::corruption("paged recovery payload range overflows"))?;
        if location.frame_length == 0
            || location.payload_length == 0
            || location.payload_offset < location.frame_offset
            || payload_end > frame_end
            || frame_end > self.committed_prefix_length
            || location.commit_frame_offset < frame_end
            || location.commit_frame_offset >= self.committed_prefix_length
            || location.commit_lsn == Lsn::ZERO
        {
            return Err(Error::corruption(
                "paged recovery frame is outside the committed prefix",
            ));
        }
        self.last_record_lsn = Some(location.record_lsn);
        self.last_commit_lsn = Some(location.commit_lsn);
        Ok(())
    }

    fn descriptor_for_location(&self, location: &WalPageLocation) -> Result<VersionDescriptor> {
        let frame = WalFrameLocator {
            source: self.source,
            frame_offset: location.frame_offset,
            frame_length: location.frame_length,
            header_crc32c: location.header_crc32c,
            payload_offset: location.payload_offset,
            payload_length: location.payload_length,
            payload_crc32c: location.payload_crc32c,
            committed_prefix_length: self.committed_prefix_length,
            commit_frame_offset: location.commit_frame_offset,
            commit_lsn: location.commit_lsn,
        };
        let image_lsn = location
            .image_lsn
            .ok_or_else(|| Error::corruption("committed WAL page has no image LSN"))?;
        let image_crc32c = location
            .image_crc32c
            .ok_or_else(|| Error::corruption("committed WAL page has no image checksum"))?;
        let locator = match location.record_type {
            WalRecordType::PageImage => {
                if location.delta_depth != 0 {
                    return Err(Error::corruption(
                        "WAL image location has a nonzero delta depth",
                    ));
                }
                VersionLocator::WalImage { frame }
            }
            WalRecordType::PageDelta => {
                let previous = self
                    .index
                    .lookup(self.root, location.page_id)?
                    .ok_or_else(|| Error::corruption("WAL delta has no indexed predecessor"))?;
                let base_lsn = location
                    .base_page_lsn
                    .ok_or_else(|| Error::corruption("WAL delta has no base LSN"))?;
                let base_crc32c = location
                    .base_image_crc32c
                    .ok_or_else(|| Error::corruption("WAL delta has no base checksum"))?;
                if previous.image_lsn != base_lsn || previous.image_crc32c != base_crc32c {
                    return Err(Error::corruption(
                        "WAL delta base differs from its indexed predecessor",
                    ));
                }
                let mut ancestry = match previous.locator {
                    VersionLocator::CheckpointImage {
                        source,
                        offset,
                        source_length,
                        image_lsn,
                        image_crc32c,
                    } => vec![DeltaChainEntry::CheckpointImage {
                        source,
                        offset,
                        source_length,
                        image_lsn,
                        image_crc32c,
                    }],
                    VersionLocator::WalImage { frame } => {
                        let VersionToken::Wal { record_lsn, .. } = previous.token else {
                            return Err(Error::corruption("WAL image version has a non-WAL token"));
                        };
                        vec![DeltaChainEntry::WalImage {
                            frame,
                            record_lsn,
                            image_lsn: previous.image_lsn,
                            image_crc32c: previous.image_crc32c,
                        }]
                    }
                    VersionLocator::WalDelta { chain, .. } => {
                        self.index
                            .delta_chains()
                            .read(chain, location.page_id)?
                            .entries
                    }
                };
                ancestry.push(DeltaChainEntry::WalDelta {
                    frame,
                    record_lsn: location.record_lsn,
                    base_lsn,
                    base_crc32c,
                    image_lsn,
                    image_crc32c,
                });
                let chain = DeltaChain::new(location.page_id, ancestry)?;
                let next_index_generation = self
                    .root
                    .generation()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("version index generation exhausted"))?;
                let chain = self
                    .index
                    .delta_chains()
                    .write(next_index_generation, &chain)?;
                VersionLocator::WalDelta {
                    frame,
                    chain,
                    depth: location.delta_depth,
                }
            }
            _ => {
                return Err(Error::corruption(
                    "paged recovery location is not a page image or delta",
                ));
            }
        };
        Ok(VersionDescriptor {
            page_id: location.page_id,
            token: VersionToken::Wal {
                source_file_number: self.source.file_number,
                record_lsn: location.record_lsn,
            },
            image_lsn,
            image_crc32c,
            locator,
        })
    }
}

struct PagedRecoverySink<'builder, 'index, 'resolver, R> {
    builder: &'builder mut PagedRecoveryIndexBuilder<'index>,
    resolve_page: &'resolver mut R,
}

impl<R> WalLocationRecoverySink for PagedRecoverySink<'_, '_, '_, R>
where
    R: FnMut(VersionDescriptor) -> Result<WalPageImage>,
{
    fn begin_scan(
        &mut self,
        committed_prefix_length: u64,
        source_header_identity: [u8; 32],
    ) -> Result<()> {
        if self.builder.source.header_identity != source_header_identity {
            return Err(Error::corruption(
                "paged recovery WAL source header identity differs",
            ));
        }
        self.builder.committed_prefix_length = committed_prefix_length;
        Ok(())
    }

    fn current_page(&mut self, page_id: dodb_core::PageId) -> Result<Option<WalPageRecoveryBase>> {
        let Some(descriptor) = self.builder.index.lookup(self.builder.root, page_id)? else {
            return Ok(None);
        };
        let page = (self.resolve_page)(descriptor)?;
        let decoded = decode_blink_page(&page.image, page_id)?;
        let image_crc32c = crc32c::crc32c(&page.image[..]);
        if page.page_id != page_id
            || decoded.lsn() != descriptor.image_lsn
            || image_crc32c != descriptor.image_crc32c
        {
            return Err(Error::corruption(
                "paged recovery resolver returned bytes that differ from the indexed version",
            ));
        }
        let (predecessor_frame_offset, delta_depth) = match descriptor.locator {
            VersionLocator::CheckpointImage { .. } => (None, 0),
            VersionLocator::WalImage { frame } => (Some(frame.frame_offset), 0),
            VersionLocator::WalDelta { frame, depth, .. } => (Some(frame.frame_offset), depth),
        };
        Ok(Some(WalPageRecoveryBase {
            page,
            image_crc32c,
            predecessor_frame_offset,
            delta_depth,
        }))
    }

    fn accept_committed_page(
        &mut self,
        location: WalPageLocation,
        page: WalPageImage,
    ) -> Result<()> {
        if page.page_id != location.page_id
            || Some(crc32c::crc32c(&page.image[..])) != location.image_crc32c
            || (!is_superblock_page(location.page_id)
                && Some(decode_blink_page(&page.image, page.page_id)?.lsn()) != location.image_lsn)
        {
            return Err(Error::corruption(
                "streamed WAL page differs from its committed location metadata",
            ));
        }
        self.builder.validate_committed_location(&location)?;
        if location.page_id.get() >= 2 {
            let descriptor = self.builder.descriptor_for_location(&location)?;
            self.builder.root =
                self.builder
                    .index
                    .insert(self.builder.root, location.page_id, descriptor)?;
        }
        Ok(())
    }
}

fn is_superblock_page(page_id: dodb_core::PageId) -> bool {
    page_id.get() < 2
}

#[cfg(test)]
mod tests {
    use super::{
        PagedRecoveryIndexBuilder, build_checkpoint_index, build_incremental_checkpoint_index,
    };
    use crate::blink::durable_bases::{BasePackJob, pack_path};
    #[cfg(feature = "blink-demand-paging")]
    use crate::blink::page_cache::PageLoaderConfig;
    use crate::blink::paged_manifest::{PagedManifest, PagedManifestStore};
    use crate::blink::resource::{BlinkResourceManager, ResourceLimits};
    #[cfg(feature = "blink-demand-paging")]
    use crate::blink::source_manager::{SourceManager, SourceManagerConfig};
    use crate::blink::version_chain::DeltaChainEntry;
    use crate::blink::version_index::{
        SourceFileKind, SourceIdentity, StoreIdentity, VersionDescriptor, VersionIndex,
        VersionLocator, VersionToken, WalFrameLocator,
    };
    use crate::blink::{BlinkPage, encode_blink_page};
    #[cfg(feature = "blink-demand-paging")]
    use crate::blink::{PagedReadConfig, PagedReadHandle};
    use crate::durable_file::ProductionFile;
    use crate::fault::FaultInjector;
    use crate::wal::{
        WalCommit, WalDeltaRequest, WalIdentity, WalLog, WalPageImage, WalPageImageFormat,
        WalPageLocation, WalRecordType, WalSegmentBoundary,
    };
    use dodb_core::{Error, Lsn, PageId, ShardEpoch, ShardId, TenantId};
    use std::borrow::Cow;
    use std::collections::BTreeMap;
    use std::fs;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "dodb-paged-recovery-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).unwrap();
            Self(directory)
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

    struct PauseBasePackAtPageWrite {
        ready_path: PathBuf,
    }

    impl FaultInjector for PauseBasePackAtPageWrite {
        fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
            if point == "during_checkpoint_page_write" {
                fs::write(&self.ready_path, b"ready")?;
                loop {
                    thread::sleep(Duration::from_secs(1));
                }
            }
            Ok(())
        }
    }

    struct PauseWalBeforeCommit {
        ready_path: PathBuf,
    }

    impl FaultInjector for PauseWalBeforeCommit {
        fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
            if point == "after_page_images_written" {
                fs::write(&self.ready_path, b"ready")?;
                loop {
                    thread::sleep(Duration::from_secs(1));
                }
            }
            Ok(())
        }
    }

    #[test]
    fn active_wal_torn_tail_after_sigkill_keeps_only_acknowledged_commit() {
        const CHILD_WAL_PATH: &str = "DODB_TORN_WAL_SIGKILL_PATH";
        const CHILD_READY: &str = "DODB_TORN_WAL_SIGKILL_READY";
        const CHILD_BATCH_ID: &str = "DODB_TORN_WAL_SIGKILL_BATCH_ID";
        const CHILD_COMMIT_LSN: &str = "DODB_TORN_WAL_SIGKILL_COMMIT_LSN";
        let page_id = PageId::new(2);
        let identity = StoreIdentity {
            database_uuid: [0x78; 16],
            wal_epoch: 14,
        };
        let wal_identity = wal_identity(identity);
        let checkpoint_image = encode_blink_page(
            page_id,
            &BlinkPage::Leaf {
                lsn: Lsn::new(10),
                high_key: None,
                right_sibling: None,
                entries: super::super::LeafEntries::default(),
            },
        )
        .unwrap();
        if let Ok(wal_path) = std::env::var(CHILD_WAL_PATH) {
            let mut wal = WalLog::open_with_segment_bases(
                ProductionFile::open(&wal_path).unwrap(),
                wal_identity.clone(),
                WalPageImageFormat::ExperimentalBlink,
                WalSegmentBoundary {
                    start_after_lsn: Lsn::new(10),
                    start_batch_id: None,
                    truncate_torn_tail: true,
                    defer_initial_sync: false,
                },
                BTreeMap::from([(
                    page_id,
                    crate::wal::RecoveredWalPage {
                        commit_lsn: Lsn::new(10),
                        image: Box::new(checkpoint_image),
                    },
                )]),
                None,
            )
            .unwrap();
            let incomplete_batch_id = std::env::var(CHILD_BATCH_ID)
                .map(|value| value.parse::<u64>().unwrap())
                .unwrap_or(2);
            let incomplete_commit_lsn = std::env::var(CHILD_COMMIT_LSN)
                .map(|value| value.parse::<u64>().unwrap())
                .unwrap_or(14);
            let incomplete_image = encode_blink_page(
                page_id,
                &BlinkPage::Leaf {
                    lsn: Lsn::new(incomplete_commit_lsn),
                    high_key: None,
                    right_sibling: None,
                    entries: super::super::LeafEntries::default(),
                },
            )
            .unwrap();
            let _ = wal.append_group(
                &[WalCommit {
                    batch_id: incomplete_batch_id,
                    commit_lsn: Lsn::new(incomplete_commit_lsn),
                    pages: vec![WalPageImage {
                        page_id,
                        image: incomplete_image,
                    }],
                }],
                Some(&mut PauseWalBeforeCommit {
                    ready_path: PathBuf::from(std::env::var(CHILD_READY).unwrap()),
                }),
            );
            panic!("child WAL append did not stop before its commit frame");
        }

        let directory = TestDirectory::new();
        let wal_path = directory.path().join("active.wal");
        let acknowledged_image = encode_blink_page(
            page_id,
            &BlinkPage::Leaf {
                lsn: Lsn::new(12),
                high_key: None,
                right_sibling: None,
                entries: super::super::LeafEntries::default(),
            },
        )
        .unwrap();
        let mut wal = WalLog::open_with_segment_bases(
            ProductionFile::open(&wal_path).unwrap(),
            wal_identity.clone(),
            WalPageImageFormat::ExperimentalBlink,
            WalSegmentBoundary {
                start_after_lsn: Lsn::new(10),
                start_batch_id: None,
                truncate_torn_tail: true,
                defer_initial_sync: false,
            },
            BTreeMap::from([(
                page_id,
                crate::wal::RecoveredWalPage {
                    commit_lsn: Lsn::new(10),
                    image: Box::new(checkpoint_image),
                },
            )]),
            None,
        )
        .unwrap();
        wal.append_group(
            &[WalCommit {
                batch_id: 1,
                commit_lsn: Lsn::new(12),
                pages: vec![WalPageImage {
                    page_id,
                    image: acknowledged_image,
                }],
            }],
            None,
        )
        .unwrap();
        drop(wal);
        let committed_length = fs::metadata(&wal_path).unwrap().len();
        let ready_path = directory.path().join("ready");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "blink::paged_recovery::tests::active_wal_torn_tail_after_sigkill_keeps_only_acknowledged_commit",
                "--nocapture",
            ])
            .env(CHILD_WAL_PATH, &wal_path)
            .env(CHILD_READY, &ready_path)
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready_path.exists() && Instant::now() < deadline {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("SIGKILL child exited before incomplete page append: {status}");
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(
            ready_path.exists(),
            "child did not reach incomplete WAL append"
        );
        assert_eq!(
            unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
            0
        );
        let status = child.wait().unwrap();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(libc::SIGKILL));

        let mut reopened = WalLog::open_with_segment_bases(
            ProductionFile::open(&wal_path).unwrap(),
            wal_identity.clone(),
            WalPageImageFormat::ExperimentalBlink,
            WalSegmentBoundary {
                start_after_lsn: Lsn::new(10),
                start_batch_id: None,
                truncate_torn_tail: true,
                defer_initial_sync: false,
            },
            BTreeMap::from([(
                page_id,
                crate::wal::RecoveredWalPage {
                    commit_lsn: Lsn::new(10),
                    image: Box::new(checkpoint_image),
                },
            )]),
            None,
        )
        .unwrap();
        assert_eq!(reopened.last_commit_lsn(), Some(Lsn::new(12)));
        assert_eq!(
            reopened
                .recovery_pages()
                .get(&page_id)
                .unwrap()
                .image
                .as_ref(),
            &acknowledged_image
        );
        assert_eq!(reopened.committed_batch_count(), 1);
        assert!(fs::metadata(&wal_path).unwrap().len() >= committed_length);

        let recovered_write_lsn = Lsn::new(reopened.next_lsn().get() + 1);
        let recovered_write_image = encode_blink_page(
            page_id,
            &BlinkPage::Leaf {
                lsn: recovered_write_lsn,
                high_key: None,
                right_sibling: None,
                entries: super::super::LeafEntries::default(),
            },
        )
        .unwrap();
        let recovered_batch_id = reopened.next_batch_id();
        reopened
            .append_group(
                &[WalCommit {
                    batch_id: recovered_batch_id,
                    commit_lsn: recovered_write_lsn,
                    pages: vec![WalPageImage {
                        page_id,
                        image: recovered_write_image,
                    }],
                }],
                None,
            )
            .unwrap();
        let second_incomplete_batch_id = reopened.next_batch_id();
        let second_incomplete_commit_lsn = Lsn::new(reopened.next_lsn().get() + 1);
        drop(reopened);

        let second_ready_path = directory.path().join("ready-second-crash");
        let mut second_child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "blink::paged_recovery::tests::active_wal_torn_tail_after_sigkill_keeps_only_acknowledged_commit",
                "--nocapture",
            ])
            .env(CHILD_WAL_PATH, &wal_path)
            .env(CHILD_READY, &second_ready_path)
            .env(CHILD_BATCH_ID, second_incomplete_batch_id.to_string())
            .env(CHILD_COMMIT_LSN, second_incomplete_commit_lsn.get().to_string())
            .spawn()
            .unwrap();
        let second_deadline = Instant::now() + Duration::from_secs(10);
        while !second_ready_path.exists() && Instant::now() < second_deadline {
            if let Some(status) = second_child.try_wait().unwrap() {
                panic!("second-crash child exited before incomplete WAL append: {status}");
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(
            second_ready_path.exists(),
            "second-crash child did not reach incomplete WAL append"
        );
        assert_eq!(
            unsafe { libc::kill(second_child.id() as libc::pid_t, libc::SIGKILL) },
            0
        );
        let second_status = second_child.wait().unwrap();
        assert_eq!(second_status.signal(), Some(libc::SIGKILL));

        let reopened_after_second_crash = WalLog::open_with_segment_bases(
            ProductionFile::open(&wal_path).unwrap(),
            wal_identity,
            WalPageImageFormat::ExperimentalBlink,
            WalSegmentBoundary {
                start_after_lsn: Lsn::new(10),
                start_batch_id: None,
                truncate_torn_tail: true,
                defer_initial_sync: false,
            },
            BTreeMap::from([(
                page_id,
                crate::wal::RecoveredWalPage {
                    commit_lsn: Lsn::new(10),
                    image: Box::new(checkpoint_image),
                },
            )]),
            None,
        )
        .unwrap();
        assert_eq!(
            reopened_after_second_crash.last_commit_lsn(),
            Some(recovered_write_lsn)
        );
        assert_eq!(reopened_after_second_crash.committed_batch_count(), 2);
        assert_eq!(
            reopened_after_second_crash
                .recovery_pages()
                .get(&page_id)
                .unwrap()
                .image
                .as_ref(),
            &recovered_write_image
        );
    }

    #[test]
    fn paged_base_pack_construction_survives_sigkill() {
        const CHILD_DIRECTORY: &str = "DODB_PAGED_BASE_PACK_SIGKILL_DIRECTORY";
        const CHILD_READY: &str = "DODB_PAGED_BASE_PACK_SIGKILL_READY";
        if let Ok(directory_path) = std::env::var(CHILD_DIRECTORY) {
            let directory = PathBuf::from(directory_path);
            let identity = StoreIdentity {
                database_uuid: [0x77; 16],
                wal_epoch: 13,
            };
            let page_id = PageId::new(2);
            let image = encode_blink_page(
                page_id,
                &BlinkPage::Free {
                    lsn: Lsn::new(10),
                    next: None,
                },
            )
            .unwrap();
            let mut writer = super::super::durable_bases::PagedBasePackJob {
                directory,
                identity: wal_identity(identity),
                number: 5,
                checkpoint_lsn: Lsn::new(10),
                page_count: 1,
                complete: true,
                coalesce: true,
            }
            .begin(Some(Box::new(PauseBasePackAtPageWrite {
                ready_path: PathBuf::from(std::env::var(CHILD_READY).unwrap()),
            })))
            .unwrap();
            let _ = writer.append_page(page_id, &image);
            panic!("child pack writer did not stop at the page-write boundary");
        }

        let directory = TestDirectory::new();
        let identity = StoreIdentity {
            database_uuid: [0x77; 16],
            wal_epoch: 13,
        };
        let ready_path = directory.path().join("ready");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "blink::paged_recovery::tests::paged_base_pack_construction_survives_sigkill",
                "--nocapture",
            ])
            .env(CHILD_DIRECTORY, directory.path())
            .env(CHILD_READY, &ready_path)
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready_path.exists() && Instant::now() < deadline {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("SIGKILL child exited before page write: {status}");
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(ready_path.exists(), "child did not reach page write");
        assert_eq!(
            unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
            0
        );
        let status = child.wait().unwrap();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        let pack_path = pack_path(directory.path(), 5);
        assert!(pack_path.exists());
        let entry = super::super::durable_bases::BasePackEntry {
            number: 5,
            checkpoint_lsn: Lsn::new(10),
            page_count: 1,
            bytes: super::super::durable_bases::encoded_size(1),
            checksum: 0,
        };
        let mut pack_file = fs::File::open(pack_path).unwrap();
        assert!(
            super::super::durable_bases::scan_pack(
                &mut pack_file,
                &entry,
                &wal_identity(identity),
                PageId::new(2),
                |_, _, _, _| Ok(()),
            )
            .is_err()
        );
    }

    fn resources() -> BlinkResourceManager {
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

    fn wal_identity(identity: StoreIdentity) -> WalIdentity {
        WalIdentity::new(
            identity.database_uuid,
            TenantId::new(7),
            ShardId::new(3),
            ShardEpoch::new(identity.wal_epoch),
        )
    }

    fn base_descriptor(page_id: PageId) -> VersionDescriptor {
        VersionDescriptor {
            page_id,
            token: VersionToken::Bootstrap { page_id },
            image_lsn: Lsn::new(10),
            image_crc32c: 0x1111_2222,
            locator: VersionLocator::CheckpointImage {
                source: SourceIdentity {
                    kind: SourceFileKind::CheckpointPack,
                    file_number: 5,
                    header_identity: [0x55; 32],
                },
                offset: 4096,
                source_length: 8192,
                image_lsn: Lsn::new(10),
                image_crc32c: 0x1111_2222,
            },
        }
    }

    fn committed_image_location(identity: &WalIdentity, page_id: PageId) -> WalPageLocation {
        WalPageLocation {
            source_identity: identity.clone(),
            page_id,
            record_lsn: Lsn::new(11),
            batch_id: 1,
            record_index: 0,
            record_type: WalRecordType::PageImage,
            frame_offset: 100,
            frame_length: 100,
            header_crc32c: 0x1010_1010,
            payload_offset: 132,
            payload_length: 50,
            payload_crc32c: 0x2020_2020,
            base_page_lsn: None,
            base_image_crc32c: None,
            predecessor_frame_offset: None,
            image_lsn: Some(Lsn::new(11)),
            image_crc32c: Some(0x2222_3333),
            delta_depth: 0,
            commit_lsn: Lsn::new(12),
            commit_frame_offset: 200,
        }
    }

    fn committed_delta_location(identity: &WalIdentity, page_id: PageId) -> WalPageLocation {
        WalPageLocation {
            source_identity: identity.clone(),
            page_id,
            record_lsn: Lsn::new(13),
            batch_id: 2,
            record_index: 0,
            record_type: WalRecordType::PageDelta,
            frame_offset: 220,
            frame_length: 100,
            header_crc32c: 0x3030_3030,
            payload_offset: 252,
            payload_length: 20,
            payload_crc32c: 0x4040_4040,
            base_page_lsn: Some(Lsn::new(11)),
            base_image_crc32c: Some(0x2222_3333),
            predecessor_frame_offset: Some(100),
            image_lsn: Some(Lsn::new(13)),
            image_crc32c: Some(0x3333_4444),
            delta_depth: 1,
            commit_lsn: Lsn::new(14),
            commit_frame_offset: 320,
        }
    }

    #[test]
    fn committed_images_and_deltas_stream_into_the_disk_index() {
        let directory = TestDirectory::new();
        let identity = StoreIdentity {
            database_uuid: [0x71; 16],
            wal_epoch: 9,
        };
        let index_directory = directory.path().join("index");
        let index = VersionIndex::open(&index_directory, identity, resources(), 74, 2).unwrap();
        let page_id = PageId::new(2);
        let checkpoint_root = index
            .insert(index.empty_root(), page_id, base_descriptor(page_id))
            .unwrap();
        let checkpoint_root = index.promote_root_to_durable(checkpoint_root).unwrap();
        let manifest = PagedManifest {
            identity,
            generation: 1,
            tree_root: page_id,
            free_list_root: None,
            high_water_page_id: page_id,
            durable_index_root: checkpoint_root.node_reference().unwrap(),
            source_catalog_root: None,
            recovery_wal_file_number: 1,
            recovery_offset: 64,
            committed_prefix_lsn: Lsn::new(10),
            previous_generation: 0,
        };
        let wal_source_identity = wal_identity(identity);
        let source = SourceIdentity {
            kind: SourceFileKind::WalSegment,
            file_number: 1,
            header_identity: [0x91; 32],
        };
        let mut builder = PagedRecoveryIndexBuilder::new(&index, manifest, source, 512).unwrap();
        assert_eq!(
            builder
                .apply_committed_locations([
                    committed_image_location(&wal_source_identity, page_id),
                    committed_delta_location(&wal_source_identity, page_id),
                ])
                .unwrap(),
            2
        );
        let recovered_root = builder.current_root();
        let recovered = index.lookup(recovered_root, page_id).unwrap().unwrap();
        assert_eq!(recovered.image_lsn, Lsn::new(13));
        assert_eq!(recovered.image_crc32c, 0x3333_4444);
        assert_eq!(
            recovered.token,
            VersionToken::Wal {
                source_file_number: 1,
                record_lsn: Lsn::new(13),
            }
        );
        let VersionLocator::WalDelta {
            frame,
            chain,
            depth,
        } = recovered.locator
        else {
            panic!("recovered delta did not retain its locator");
        };
        assert_eq!(frame.source, source);
        let chain = index.delta_chains().read(chain, page_id).unwrap();
        let Some(DeltaChainEntry::WalDelta {
            base_lsn,
            base_crc32c,
            frame: latest_frame,
            ..
        }) = chain.entries.last()
        else {
            panic!("latest delta ancestry entry is missing");
        };
        assert_eq!(*base_lsn, Lsn::new(11));
        assert_eq!(*base_crc32c, 0x2222_3333);
        assert_eq!(latest_frame.frame_offset, 220);
        assert_eq!(depth, 1);
        assert!(index.root_node_is_rebuildable(recovered_root).unwrap());
    }

    #[test]
    fn recovery_rejects_locations_outside_the_manifest_or_committed_prefix() {
        let directory = TestDirectory::new();
        let identity = StoreIdentity {
            database_uuid: [0x72; 16],
            wal_epoch: 10,
        };
        let index = VersionIndex::open(
            &directory.path().join("index"),
            identity,
            resources(),
            75,
            2,
        )
        .unwrap();
        let page_id = PageId::new(2);
        let root = index
            .insert(index.empty_root(), page_id, base_descriptor(page_id))
            .unwrap();
        let root = index.promote_root_to_durable(root).unwrap();
        let manifest = PagedManifest {
            identity,
            generation: 1,
            tree_root: page_id,
            free_list_root: None,
            high_water_page_id: page_id,
            durable_index_root: root.node_reference().unwrap(),
            source_catalog_root: None,
            recovery_wal_file_number: 1,
            recovery_offset: 150,
            committed_prefix_lsn: Lsn::new(10),
            previous_generation: 0,
        };
        let mut builder = PagedRecoveryIndexBuilder::new(
            &index,
            manifest,
            SourceIdentity {
                kind: SourceFileKind::WalSegment,
                file_number: 1,
                header_identity: [0x92; 32],
            },
            512,
        )
        .unwrap();
        let invalid_location = committed_image_location(&wal_identity(identity), page_id);
        assert!(
            builder
                .apply_committed_locations([invalid_location.clone()])
                .is_err()
        );
        let mut wrong_epoch_location = invalid_location;
        wrong_epoch_location.source_identity.shard_epoch = ShardEpoch::new(99);
        assert!(
            builder
                .apply_committed_locations([wrong_epoch_location])
                .is_err()
        );
        assert_eq!(builder.current_root(), root);

        let mut outside_prefix = committed_image_location(&wal_identity(identity), page_id);
        outside_prefix.frame_offset = 450;
        outside_prefix.payload_offset = 482;
        outside_prefix.commit_frame_offset = 550;
        assert!(builder.apply_committed_locations([outside_prefix]).is_err());
        assert_eq!(builder.current_root(), root);

        let mut valid_location = committed_image_location(&wal_identity(identity), page_id);
        valid_location.frame_offset = 200;
        valid_location.payload_offset = 232;
        valid_location.commit_frame_offset = 400;
        let mut invalid_after_valid =
            committed_image_location(&wal_identity(identity), PageId::new(3));
        invalid_after_valid.frame_offset = 200;
        invalid_after_valid.payload_offset = 232;
        invalid_after_valid.commit_frame_offset = 400;
        invalid_after_valid.source_identity.database_uuid = [0xff; 16];
        assert!(
            builder
                .apply_committed_locations([valid_location, invalid_after_valid])
                .is_err()
        );
        assert_eq!(builder.current_root(), root);
    }

    #[test]
    fn recovery_indexes_locations_from_an_actual_committed_wal() {
        let directory = TestDirectory::new();
        let identity = StoreIdentity {
            database_uuid: [0x73; 16],
            wal_epoch: 11,
        };
        let index_directory = directory.path().join("index");
        let index =
            Arc::new(VersionIndex::open(&index_directory, identity, resources(), 76, 2).unwrap());
        let page_id = PageId::new(2);
        let checkpoint_image = encode_blink_page(
            page_id,
            &BlinkPage::Leaf {
                lsn: Lsn::new(10),
                high_key: None,
                right_sibling: None,
                entries: super::super::LeafEntries::default(),
            },
        )
        .unwrap();
        let first_free_page_id = PageId::new(3);
        let second_page_id = PageId::new(25);
        let mut checkpoint_images = BTreeMap::from([(page_id, checkpoint_image)]);
        for page_number in first_free_page_id.get()..=second_page_id.get() {
            let free_page_id = PageId::new(page_number);
            let next = (free_page_id < second_page_id).then(|| PageId::new(page_number + 1));
            let free_image = encode_blink_page(
                free_page_id,
                &BlinkPage::Free {
                    lsn: Lsn::new(9),
                    next,
                },
            )
            .unwrap();
            checkpoint_images.insert(free_page_id, free_image);
        }
        let pack_identity = wal_identity(identity);
        let mut pack_injector = None;
        let pack_output = BasePackJob {
            directory: directory.path().to_path_buf(),
            identity: pack_identity.clone(),
            number: 5,
            checkpoint_lsn: Lsn::new(10),
            complete: true,
            coalesce: true,
        }
        .run(
            &checkpoint_images
                .iter()
                .map(|(page_id, image)| (*page_id, Arc::new(*image)))
                .collect(),
            &mut pack_injector,
        )
        .unwrap();
        let checkpoint_pack_path = pack_path(directory.path(), pack_output.entry.number);
        assert!(
            build_checkpoint_index(
                &index,
                index.empty_root(),
                &checkpoint_pack_path,
                &pack_output.entry,
                false,
                &pack_identity,
                second_page_id,
            )
            .is_err()
        );
        let checkpoint_wal_source = SourceIdentity {
            kind: SourceFileKind::WalSegment,
            file_number: 4,
            header_identity: [0x54; 32],
        };
        let checkpoint_crc32c = crc32c::crc32c(&checkpoint_images[&page_id]);
        let base_root = index
            .insert(
                index.empty_root(),
                page_id,
                VersionDescriptor {
                    page_id,
                    token: VersionToken::Wal {
                        source_file_number: checkpoint_wal_source.file_number,
                        record_lsn: Lsn::new(9),
                    },
                    image_lsn: Lsn::new(10),
                    image_crc32c: checkpoint_crc32c,
                    locator: VersionLocator::WalImage {
                        frame: WalFrameLocator {
                            source: checkpoint_wal_source,
                            frame_offset: 100,
                            frame_length: 4156,
                            header_crc32c: 0x1010_1010,
                            payload_offset: 148,
                            payload_length: 4104,
                            payload_crc32c: 0x2020_2020,
                            committed_prefix_length: 4324,
                            commit_frame_offset: 4256,
                            commit_lsn: Lsn::new(10),
                        },
                    },
                },
            )
            .unwrap();
        let (checkpoint_root, checkpoint_source) = build_checkpoint_index(
            &index,
            base_root,
            &checkpoint_pack_path,
            &pack_output.entry,
            pack_output.complete,
            &pack_identity,
            second_page_id,
        )
        .unwrap();
        let checkpoint_descriptor = index.lookup(checkpoint_root, page_id).unwrap().unwrap();
        assert_eq!(
            checkpoint_descriptor.token,
            VersionToken::Wal {
                source_file_number: checkpoint_wal_source.file_number,
                record_lsn: Lsn::new(9),
            }
        );
        assert_eq!(
            index
                .lookup(checkpoint_root, second_page_id)
                .unwrap()
                .unwrap()
                .token,
            VersionToken::Bootstrap {
                page_id: second_page_id
            }
        );
        index.validate_durable_root(checkpoint_root).unwrap();
        let manifest = PagedManifest {
            identity,
            generation: 1,
            tree_root: page_id,
            free_list_root: Some(first_free_page_id),
            high_water_page_id: second_page_id,
            durable_index_root: checkpoint_root.node_reference().unwrap(),
            source_catalog_root: None,
            recovery_wal_file_number: 1,
            recovery_offset: 0,
            committed_prefix_lsn: Lsn::ZERO,
            previous_generation: 0,
        };
        let manifest_database_path = directory.path().join("database");
        let manifest_store =
            PagedManifestStore::create_with_index(&manifest_database_path, manifest, &index)
                .unwrap();
        let wal_source_identity = wal_identity(identity);
        let wal_path = directory.path().join("wal");
        let wal_file = ProductionFile::open(&wal_path).unwrap();
        let recovered_checkpoint_pages = checkpoint_images
            .iter()
            .map(|(checkpoint_page_id, checkpoint_page_image)| {
                (
                    *checkpoint_page_id,
                    crate::wal::RecoveredWalPage {
                        commit_lsn: Lsn::new(10),
                        image: Box::new(*checkpoint_page_image),
                    },
                )
            })
            .collect();
        let mut wal = WalLog::open_with_segment_bases(
            wal_file,
            wal_source_identity,
            WalPageImageFormat::ExperimentalBlink,
            WalSegmentBoundary {
                start_after_lsn: Lsn::new(10),
                start_batch_id: None,
                truncate_torn_tail: true,
                defer_initial_sync: false,
            },
            recovered_checkpoint_pages,
            None,
        )
        .unwrap();
        let recovered_commit_lsn = Lsn::new(12);
        let free_page_commit_lsn = Lsn::new(12 + (checkpoint_images.len() as u64 - 1) + 1);
        let expected_image = encode_blink_page(
            page_id,
            &BlinkPage::Leaf {
                lsn: recovered_commit_lsn,
                high_key: None,
                right_sibling: None,
                entries: super::super::LeafEntries::default(),
            },
        )
        .unwrap();
        let mut free_page_wal_images = Vec::with_capacity(checkpoint_images.len() - 1);
        for (updated_page_id, checkpoint_page_image) in &checkpoint_images {
            if *updated_page_id == page_id {
                continue;
            }
            let decoded =
                crate::blink::decode_blink_page(checkpoint_page_image, *updated_page_id).unwrap();
            let BlinkPage::Free { next, .. } = decoded else {
                panic!("unexpected paged recovery fixture page type");
            };
            let image = encode_blink_page(
                *updated_page_id,
                &BlinkPage::Free {
                    lsn: free_page_commit_lsn,
                    next,
                },
            )
            .unwrap();
            free_page_wal_images.push(WalPageImage {
                page_id: *updated_page_id,
                image,
            });
        }
        let eligible_commits = [true, false];
        let mut base_source =
            |requested_page_id| checkpoint_images.get(&requested_page_id).map(Cow::Borrowed);
        let mut delta_request = WalDeltaRequest {
            eligible_commits: &eligible_commits,
            base_versions: None,
            base_source: &mut base_source,
        };
        wal.append_group_with_page_deltas(
            &[
                WalCommit {
                    batch_id: 1,
                    commit_lsn: recovered_commit_lsn,
                    pages: vec![WalPageImage {
                        page_id,
                        image: expected_image,
                    }],
                },
                WalCommit {
                    batch_id: 2,
                    commit_lsn: free_page_commit_lsn,
                    pages: free_page_wal_images,
                },
            ],
            &mut delta_request,
            None,
        )
        .unwrap();
        drop(wal);
        let wal_bytes = fs::read(&wal_path).unwrap();
        assert!(wal_bytes.len() > crate::page::PAGE_SIZE * 2);
        let init_frame_length = u32::from_le_bytes(wal_bytes[8..12].try_into().unwrap()) as usize;
        let wal_header_identity = *blake3::hash(&wal_bytes[..init_frame_length]).as_bytes();
        let mut wal_file = ProductionFile::open(&wal_path).unwrap();
        drop(manifest_store);
        drop(index);
        let index =
            Arc::new(VersionIndex::open(&index_directory, identity, resources(), 76, 2).unwrap());
        let mut manifest_store =
            PagedManifestStore::open_with_index(&manifest_database_path, identity, &index).unwrap();
        let manifest = manifest_store.current();
        let mut wrong_source_builder = PagedRecoveryIndexBuilder::new(
            &index,
            manifest,
            SourceIdentity {
                kind: SourceFileKind::WalSegment,
                file_number: 1,
                header_identity: [0x93; 32],
            },
            0,
        )
        .unwrap();
        let mut unused_resolver =
            |_: VersionDescriptor| Err(Error::corruption("unexpected paged recovery source"));
        assert!(
            wrong_source_builder
                .replay_wal(
                    &mut wal_file,
                    &wal_identity(identity),
                    WalPageImageFormat::ExperimentalBlink,
                    &mut unused_resolver,
                )
                .is_err()
        );
        assert_eq!(wrong_source_builder.current_root(), checkpoint_root);
        let mut builder = PagedRecoveryIndexBuilder::new(
            &index,
            manifest,
            SourceIdentity {
                kind: SourceFileKind::WalSegment,
                file_number: 1,
                header_identity: wal_header_identity,
            },
            0,
        )
        .unwrap();
        let mut checkpoint_file = fs::File::open(&checkpoint_pack_path).unwrap();
        let mut resolver = |descriptor: VersionDescriptor| {
            let VersionLocator::CheckpointImage {
                source,
                offset,
                source_length,
                image_lsn,
                image_crc32c,
            } = descriptor.locator
            else {
                return Err(Error::corruption("unexpected paged recovery source"));
            };
            if source != checkpoint_source
                || source_length != pack_output.entry.bytes
                || image_lsn != descriptor.image_lsn
                || image_crc32c != descriptor.image_crc32c
            {
                return Err(Error::corruption("checkpoint source identity differs"));
            }
            let mut image = [0; crate::page::PAGE_SIZE];
            checkpoint_file.seek(SeekFrom::Start(offset))?;
            checkpoint_file.read_exact(&mut image)?;
            Ok(WalPageImage {
                page_id: descriptor.page_id,
                image,
            })
        };
        let summary = builder
            .replay_wal(
                &mut wal_file,
                &wal_identity(identity),
                WalPageImageFormat::ExperimentalBlink,
                &mut resolver,
            )
            .unwrap();
        let recovered = index
            .lookup(builder.current_root(), page_id)
            .unwrap()
            .unwrap();
        assert_eq!(recovered.image_lsn, recovered_commit_lsn);
        assert_eq!(recovered.image_crc32c, crc32c::crc32c(&expected_image));
        assert_eq!(
            recovered.token,
            VersionToken::Wal {
                source_file_number: 1,
                record_lsn: Lsn::new(11),
            }
        );
        assert_eq!(summary.last_commit_lsn, Some(free_page_commit_lsn));
        assert_eq!(summary.torn_tail_bytes, 0);
        assert_eq!(summary.source_header_identity, wal_header_identity);
        let wal_source_path = directory
            .path()
            .join("segments/segment-00000000000000000001.wal");
        fs::create_dir_all(wal_source_path.parent().unwrap()).unwrap();
        fs::copy(&wal_path, &wal_source_path).unwrap();
        let sources = SourceManager::new(
            SourceManagerConfig {
                store_identity: identity,
                checkpoint_directory: directory.path().to_path_buf(),
                wal_directory: wal_source_path.parent().unwrap().to_path_buf(),
                scratch_directory: directory.path().join("validation-scratch"),
                tenant_id: 76,
                maximum_open_handles: 4,
                maximum_roots: 4,
                handle_charge_bytes: 128,
                gc_reservation_bytes: 8192,
            },
            resources(),
        )
        .unwrap();
        assert_eq!(
            builder
                .validate_recovered_pages(
                    Arc::clone(&index),
                    super::PagedRecoveryValidationConfig {
                        tenant_id: 76,
                        root_page_id: page_id,
                        free_list_head: Some(first_free_page_id),
                        high_water_page_id: second_page_id,
                        resources: resources(),
                        sources,
                        loader: PageLoaderConfig {
                            maximum_cache_bytes: (crate::page::PAGE_SIZE * 2) as u64,
                            minimum_decoded_page_bytes: crate::page::PAGE_SIZE as u64,
                            maximum_load_reconstruction_bytes: (crate::page::PAGE_SIZE * 4) as u64,
                            maximum_concurrent_loads: 2,
                            maximum_followers: 8,
                        },
                        header_length: |source: SourceIdentity| {
                            Ok(if source.kind == SourceFileKind::WalSegment {
                                crate::wal::INIT_FRAME_SIZE
                            } else {
                                crate::page::PAGE_SIZE
                            })
                        },
                    },
                )
                .unwrap(),
            24
        );
        let restarted_wal = WalLog::open_with_segment_bases(
            ProductionFile::open(&wal_path).unwrap(),
            wal_identity(identity),
            WalPageImageFormat::ExperimentalBlink,
            WalSegmentBoundary {
                start_after_lsn: Lsn::new(10),
                start_batch_id: None,
                truncate_torn_tail: true,
                defer_initial_sync: false,
            },
            std::collections::BTreeMap::from([(
                page_id,
                crate::wal::RecoveredWalPage {
                    commit_lsn: Lsn::new(10),
                    image: Box::new(checkpoint_images[&page_id]),
                },
            )]),
            None,
        )
        .unwrap();
        assert_eq!(
            restarted_wal
                .recovery_pages()
                .get(&page_id)
                .unwrap()
                .image
                .as_ref(),
            &expected_image
        );
        assert_eq!(
            builder.committed_prefix_length,
            summary.committed_prefix_length
        );
        assert_eq!(
            index.lookup(checkpoint_root, page_id).unwrap().unwrap(),
            checkpoint_descriptor
        );
        let VersionLocator::WalDelta { chain, depth, .. } = recovered.locator else {
            panic!("committed WAL delta was not retained as a delta locator");
        };
        let chain = index.delta_chains().read(chain, page_id).unwrap();
        assert!(matches!(
            chain.entries.first(),
            Some(DeltaChainEntry::CheckpointImage {
                source,
                image_lsn,
                image_crc32c,
                ..
            }) if *source == checkpoint_source
                && *image_lsn == Lsn::new(10)
                && *image_crc32c == checkpoint_descriptor.image_crc32c
        ));
        assert_eq!(depth, 1);
        let recovered_root = index
            .promote_root_to_durable(builder.current_root())
            .unwrap();
        manifest_store
            .publish_with_index(
                PagedManifest {
                    identity,
                    generation: 2,
                    tree_root: page_id,
                    free_list_root: Some(first_free_page_id),
                    high_water_page_id: second_page_id,
                    durable_index_root: recovered_root.node_reference().unwrap(),
                    source_catalog_root: None,
                    recovery_wal_file_number: 1,
                    recovery_offset: summary.committed_prefix_length,
                    committed_prefix_lsn: summary.last_commit_lsn.unwrap(),
                    previous_generation: 1,
                },
                &index,
            )
            .unwrap();
        drop(manifest_store);
        drop(index);
        let reopened_index =
            VersionIndex::open(&index_directory, identity, resources(), 76, 2).unwrap();
        let reopened_manifest =
            PagedManifestStore::open_with_index(&manifest_database_path, identity, &reopened_index)
                .unwrap();
        let reopened_root = reopened_index
            .root_from_node_reference(reopened_manifest.current().durable_index_root)
            .unwrap();
        let reopened_descriptor = reopened_index
            .lookup(reopened_root, page_id)
            .unwrap()
            .unwrap();
        assert_eq!(reopened_descriptor.image_lsn, Lsn::new(12));
        assert_eq!(
            reopened_descriptor.image_crc32c,
            crc32c::crc32c(&expected_image)
        );
    }

    #[test]
    #[cfg(feature = "blink-demand-paging")]
    fn incremental_checkpoint_replaces_only_pages_newer_than_the_previous_checkpoint() {
        let directory = TestDirectory::new();
        let identity = StoreIdentity {
            database_uuid: [0x7a; 16],
            wal_epoch: 21,
        };
        let resources = resources();
        let index = VersionIndex::open(
            &directory.path().join("index"),
            identity,
            resources.clone(),
            79,
            2,
        )
        .unwrap();
        let unchanged_page_id = PageId::new(2);
        let changed_page_id = PageId::new(3);
        let previous_image = encode_blink_page(
            unchanged_page_id,
            &BlinkPage::Free {
                lsn: Lsn::new(10),
                next: None,
            },
        )
        .unwrap();
        let pack_identity = wal_identity(identity);
        let mut initial_pack_injector = None;
        let initial_pack_output = BasePackJob {
            directory: directory.path().to_path_buf(),
            identity: pack_identity.clone(),
            number: 1,
            checkpoint_lsn: Lsn::new(10),
            complete: true,
            coalesce: true,
        }
        .run(
            &BTreeMap::from([(unchanged_page_id, Arc::new(previous_image))]),
            &mut initial_pack_injector,
        )
        .unwrap();
        let mut initial_pack_file = std::fs::File::open(pack_path(
            directory.path(),
            initial_pack_output.entry.number,
        ))
        .unwrap();
        let mut initial_pack_header = [0u8; crate::page::PAGE_SIZE];
        initial_pack_file
            .read_exact(&mut initial_pack_header)
            .unwrap();
        let previous_source = SourceIdentity {
            kind: SourceFileKind::CheckpointPack,
            file_number: initial_pack_output.entry.number,
            header_identity: *blake3::hash(&initial_pack_header).as_bytes(),
        };
        let previous_crc = crc32c::crc32c(&previous_image);
        let unchanged_descriptor = VersionDescriptor {
            page_id: unchanged_page_id,
            token: VersionToken::Bootstrap {
                page_id: unchanged_page_id,
            },
            image_lsn: Lsn::new(10),
            image_crc32c: previous_crc,
            locator: VersionLocator::CheckpointImage {
                source: previous_source,
                offset: 4096,
                source_length: initial_pack_output.entry.bytes,
                image_lsn: Lsn::new(10),
                image_crc32c: previous_crc,
            },
        };
        let previous_root = index
            .insert(index.empty_root(), unchanged_page_id, unchanged_descriptor)
            .unwrap();
        let previous_root = index.promote_root_to_durable(previous_root).unwrap();
        let database_path = directory.path().join("database");
        let mut manifest_store = PagedManifestStore::create_with_index(
            &database_path,
            PagedManifest {
                identity,
                generation: 1,
                tree_root: unchanged_page_id,
                free_list_root: None,
                high_water_page_id: unchanged_page_id,
                durable_index_root: previous_root.node_reference().unwrap(),
                source_catalog_root: None,
                recovery_wal_file_number: 1,
                recovery_offset: 0,
                committed_prefix_lsn: Lsn::new(10),
                previous_generation: 0,
            },
            &index,
        )
        .unwrap();
        let changed_image = encode_blink_page(
            changed_page_id,
            &BlinkPage::Free {
                lsn: Lsn::new(12),
                next: None,
            },
        )
        .unwrap();
        let previous_changed_descriptor = VersionDescriptor {
            page_id: changed_page_id,
            token: VersionToken::Wal {
                source_file_number: 4,
                record_lsn: Lsn::new(12),
            },
            image_lsn: Lsn::new(12),
            image_crc32c: crc32c::crc32c(&changed_image),
            locator: VersionLocator::WalImage {
                frame: WalFrameLocator {
                    source: SourceIdentity {
                        kind: SourceFileKind::WalSegment,
                        file_number: 4,
                        header_identity: [0x24; 32],
                    },
                    frame_offset: 100,
                    frame_length: 4156,
                    header_crc32c: 0x1010_1010,
                    payload_offset: 148,
                    payload_length: 4104,
                    payload_crc32c: 0x2020_2020,
                    committed_prefix_length: 4324,
                    commit_frame_offset: 4256,
                    commit_lsn: Lsn::new(12),
                },
            },
        };
        let target_root = index
            .insert(previous_root, changed_page_id, previous_changed_descriptor)
            .unwrap();
        let mut writer = super::super::durable_bases::PagedBasePackJob {
            directory: directory.path().to_path_buf(),
            identity: pack_identity.clone(),
            number: 2,
            checkpoint_lsn: Lsn::new(12),
            page_count: 1,
            complete: false,
            coalesce: true,
        }
        .begin(None)
        .unwrap();
        writer.append_page(changed_page_id, &changed_image).unwrap();
        let output = writer.finish().unwrap();
        assert!(!output.complete);
        let (checkpoint_root, checkpoint_source) = build_incremental_checkpoint_index(
            &index,
            target_root,
            &pack_path(directory.path(), output.entry.number),
            &output.entry,
            Lsn::new(10),
            &pack_identity,
            changed_page_id,
        )
        .unwrap();
        let unchanged_after = index
            .lookup(checkpoint_root, unchanged_page_id)
            .unwrap()
            .unwrap();
        assert_eq!(unchanged_after, unchanged_descriptor);
        let changed_after = index
            .lookup(checkpoint_root, changed_page_id)
            .unwrap()
            .unwrap();
        assert_eq!(changed_after.token, previous_changed_descriptor.token);
        assert_eq!(changed_after.image_lsn, Lsn::new(12));
        assert!(matches!(
            changed_after.locator,
            VersionLocator::CheckpointImage { source, .. } if source == checkpoint_source
        ));
        index.validate_durable_root(checkpoint_root).unwrap();
        let wal_directory = directory.path().join("wal");
        fs::create_dir_all(&wal_directory).unwrap();
        let next_wal_path = wal_directory.join("segment-00000000000000000002.wal");
        let mut next_wal = WalLog::open_with_segment_boundary(
            ProductionFile::open(&next_wal_path).unwrap(),
            pack_identity,
            WalPageImageFormat::ExperimentalBlink,
            Lsn::new(12),
            Some(1),
            true,
            false,
            None,
        )
        .unwrap();
        next_wal.sync_all().unwrap();
        let recovery_offset = next_wal.file_length().unwrap();
        drop(next_wal);
        fs::File::open(&wal_directory).unwrap().sync_all().unwrap();
        manifest_store
            .publish_with_index(
                PagedManifest {
                    identity,
                    generation: 2,
                    tree_root: unchanged_page_id,
                    free_list_root: None,
                    high_water_page_id: changed_page_id,
                    durable_index_root: checkpoint_root.node_reference().unwrap(),
                    source_catalog_root: None,
                    recovery_wal_file_number: 2,
                    recovery_offset,
                    committed_prefix_lsn: Lsn::new(12),
                    previous_generation: 1,
                },
                &index,
            )
            .unwrap();
        drop(manifest_store);
        let reopened_manifest =
            PagedManifestStore::open_with_index(&database_path, identity, &index).unwrap();
        assert_eq!(reopened_manifest.current().generation, 2);
        assert_eq!(
            reopened_manifest.current().committed_prefix_lsn,
            Lsn::new(12)
        );
        let sources = SourceManager::new(
            SourceManagerConfig {
                store_identity: identity,
                checkpoint_directory: directory.path().to_path_buf(),
                wal_directory,
                scratch_directory: directory.path().join("scratch"),
                tenant_id: 7,
                maximum_open_handles: 4,
                maximum_roots: 4,
                handle_charge_bytes: 256,
                gc_reservation_bytes: 8192,
            },
            resources.clone(),
        )
        .unwrap();
        let reopened_root = index
            .root_from_node_reference(reopened_manifest.current().durable_index_root)
            .unwrap();
        let reader = PagedReadHandle::new(
            PagedReadConfig {
                tenant_id: 7,
                root: reopened_root,
                root_page_id: unchanged_page_id,
                free_list_head: None,
                high_water_page_id: changed_page_id,
                loader: PageLoaderConfig {
                    maximum_cache_bytes: (2 * crate::page::PAGE_SIZE) as u64,
                    minimum_decoded_page_bytes: crate::page::PAGE_SIZE as u64,
                    maximum_load_reconstruction_bytes: crate::page::PAGE_SIZE as u64,
                    maximum_concurrent_loads: 1,
                    maximum_followers: 2,
                },
            },
            Arc::new(index),
            resources,
            sources,
            |_| Ok(crate::page::PAGE_SIZE),
        )
        .unwrap();
        let reopened_unchanged = reader
            .load_mutation_base_page(unchanged_page_id)
            .unwrap()
            .unwrap();
        let reopened_changed = reader
            .load_mutation_base_page(changed_page_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            encode_blink_page(unchanged_page_id, reopened_unchanged.as_ref()).unwrap(),
            previous_image
        );
        assert_eq!(
            encode_blink_page(changed_page_id, reopened_changed.as_ref()).unwrap(),
            changed_image
        );
    }
}
