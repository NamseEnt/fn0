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
            root = index.insert(
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
            )?;
            Ok(())
        },
    )?;
    Ok((index.promote_root_to_durable(root)?, source))
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
    use super::{PagedRecoveryIndexBuilder, build_checkpoint_index};
    use crate::blink::durable_bases::{BasePackJob, pack_path};
    use crate::blink::paged_manifest::PagedManifest;
    use crate::blink::resource::{BlinkResourceManager, ResourceLimits};
    use crate::blink::version_chain::DeltaChainEntry;
    use crate::blink::version_index::{
        SourceFileKind, SourceIdentity, StoreIdentity, VersionDescriptor, VersionIndex,
        VersionLocator, VersionToken, WalFrameLocator,
    };
    use crate::blink::{BlinkPage, encode_blink_page};
    use crate::durable_file::ProductionFile;
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
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

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
        let index = VersionIndex::open(
            &directory.path().join("index"),
            identity,
            resources(),
            76,
            2,
        )
        .unwrap();
        let page_id = PageId::new(2);
        let checkpoint_image = encode_blink_page(
            page_id,
            &BlinkPage::Free {
                lsn: Lsn::new(10),
                next: None,
            },
        )
        .unwrap();
        let second_page_id = PageId::new(5);
        let second_checkpoint_image = encode_blink_page(
            second_page_id,
            &BlinkPage::Free {
                lsn: Lsn::new(9),
                next: None,
            },
        )
        .unwrap();
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
            &BTreeMap::from([
                (page_id, Arc::new(checkpoint_image)),
                (second_page_id, Arc::new(second_checkpoint_image)),
            ]),
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
        let checkpoint_crc32c = crc32c::crc32c(&checkpoint_image);
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
            free_list_root: None,
            high_water_page_id: second_page_id,
            durable_index_root: checkpoint_root.node_reference().unwrap(),
            source_catalog_root: None,
            recovery_wal_file_number: 1,
            recovery_offset: 0,
            committed_prefix_lsn: Lsn::ZERO,
            previous_generation: 0,
        };
        let wal_source_identity = wal_identity(identity);
        let wal_path = directory.path().join("wal");
        let wal_file = ProductionFile::open(&wal_path).unwrap();
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
            std::collections::BTreeMap::from([(
                page_id,
                crate::wal::RecoveredWalPage {
                    commit_lsn: Lsn::new(10),
                    image: Box::new(checkpoint_image),
                },
            )]),
            None,
        )
        .unwrap();
        let image = encode_blink_page(
            page_id,
            &BlinkPage::Free {
                lsn: Lsn::new(12),
                next: None,
            },
        )
        .unwrap();
        let eligible_commits = [true];
        let mut base_source = |requested_page_id| {
            (requested_page_id == page_id).then(|| Cow::Borrowed(&checkpoint_image))
        };
        let mut delta_request = WalDeltaRequest {
            eligible_commits: &eligible_commits,
            base_versions: None,
            base_source: &mut base_source,
        };
        wal.append_group_with_page_deltas(
            &[WalCommit {
                batch_id: 1,
                commit_lsn: Lsn::new(12),
                pages: vec![WalPageImage { page_id, image }],
            }],
            &mut delta_request,
            None,
        )
        .unwrap();
        drop(wal);
        let wal_bytes = fs::read(&wal_path).unwrap();
        let init_frame_length = u32::from_le_bytes(wal_bytes[8..12].try_into().unwrap()) as usize;
        let wal_header_identity = *blake3::hash(&wal_bytes[..init_frame_length]).as_bytes();
        let mut wal_file = ProductionFile::open(&wal_path).unwrap();
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
        assert_eq!(recovered.image_lsn, Lsn::new(12));
        assert_eq!(
            recovered.token,
            VersionToken::Wal {
                source_file_number: 1,
                record_lsn: Lsn::new(11),
            }
        );
        assert_eq!(summary.last_commit_lsn, Some(Lsn::new(12)));
        assert_eq!(summary.torn_tail_bytes, 0);
        assert_eq!(summary.source_header_identity, wal_header_identity);
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
    }
}
