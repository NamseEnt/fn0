use dodb_core::{Error, Lsn, Result};

use crate::wal::{WalPageLocation, WalRecordType};

use super::paged_manifest::PagedManifest;
use super::version_index::{
    BaseFrameLocator, SourceFileKind, SourceIdentity, VersionDescriptor, VersionIndex,
    VersionIndexRoot, VersionLocator, VersionToken, WalFrameLocator,
};

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
                VersionLocator::WalDelta {
                    frame,
                    base_lsn,
                    base_crc32c,
                    base: base_locator(previous)?,
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

fn base_locator(descriptor: VersionDescriptor) -> Result<BaseFrameLocator> {
    match descriptor.locator {
        VersionLocator::CheckpointImage {
            source,
            offset,
            image_lsn,
            image_crc32c,
            ..
        } => Ok(BaseFrameLocator {
            source,
            frame_offset: offset,
            frame_length: super::version_index::VERSION_INDEX_NODE_BYTES as u32,
            header_crc32c: 0,
            image_lsn,
            image_crc32c,
        }),
        VersionLocator::WalImage { frame } | VersionLocator::WalDelta { frame, .. } => {
            Ok(BaseFrameLocator {
                source: frame.source,
                frame_offset: frame.frame_offset,
                frame_length: frame.frame_length,
                header_crc32c: frame.header_crc32c,
                image_lsn: descriptor.image_lsn,
                image_crc32c: descriptor.image_crc32c,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PagedRecoveryIndexBuilder;
    use crate::blink::paged_manifest::PagedManifest;
    use crate::blink::resource::{BlinkResourceManager, ResourceLimits};
    use crate::blink::version_index::{
        SourceFileKind, SourceIdentity, StoreIdentity, VersionDescriptor, VersionIndex,
        VersionLocator, VersionToken,
    };
    use crate::durable_file::ProductionFile;
    use crate::page::{PageHeader, PageType, encode_page};
    use crate::wal::{
        WalIdentity, WalLog, WalPageImage, WalPageImageFormat, WalPageLocation, WalRecordType,
    };
    use dodb_core::{Lsn, PageId, ShardEpoch, ShardId, TenantId};
    use std::fs;
    use std::path::{Path, PathBuf};
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
            base_lsn,
            base_crc32c,
            base,
            depth,
        } = recovered.locator
        else {
            panic!("recovered delta did not retain its locator");
        };
        assert_eq!(frame.source, source);
        assert_eq!(base_lsn, Lsn::new(11));
        assert_eq!(base_crc32c, 0x2222_3333);
        assert_eq!(base.frame_offset, 100);
        assert_eq!(base.image_lsn, Lsn::new(11));
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
            recovery_offset: 0,
            committed_prefix_lsn: Lsn::ZERO,
            previous_generation: 0,
        };
        let wal_source_identity = wal_identity(identity);
        let wal_path = directory.path().join("wal");
        let wal_file = ProductionFile::open(&wal_path).unwrap();
        let mut wal = WalLog::open_with_page_image_format(
            wal_file,
            wal_source_identity,
            WalPageImageFormat::ExperimentalBtree,
        )
        .unwrap();
        let image =
            encode_page(PageHeader::new(PageType::Leaf, page_id, Lsn::new(2)), &[]).unwrap();
        wal.append_commit(1, Lsn::new(2), &[WalPageImage { page_id, image }], None)
            .unwrap();
        drop(wal);
        let mut wal_file = ProductionFile::open(&wal_path).unwrap();
        let scan = WalLog::scan_committed_page_locations(
            &mut wal_file,
            &wal_identity(identity),
            WalPageImageFormat::ExperimentalBtree,
            None,
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let mut builder = PagedRecoveryIndexBuilder::new(
            &index,
            manifest,
            SourceIdentity {
                kind: SourceFileKind::WalSegment,
                file_number: 1,
                header_identity: [0x93; 32],
            },
            scan.committed_prefix_length,
        )
        .unwrap();
        let record_lsn = scan.locations[0].record_lsn;
        assert_eq!(
            builder.apply_committed_locations(scan.locations).unwrap(),
            1
        );
        let recovered = index
            .lookup(builder.current_root(), page_id)
            .unwrap()
            .unwrap();
        assert_eq!(recovered.image_lsn, Lsn::new(2));
        assert_eq!(
            recovered.token,
            VersionToken::Wal {
                source_file_number: 1,
                record_lsn,
            }
        );
    }
}
