use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

use dodb_core::{Error, Lsn, PageId, Result};

use super::decode_blink_page;
use super::durable_bases::{BasePackEntry, pack_path, scan_pack};
use crate::page::PAGE_SIZE;
use crate::wal::WalIdentity;

#[derive(Debug)]
struct PackSource {
    file: Mutex<File>,
    entry: BasePackEntry,
    identity: WalIdentity,
}

#[derive(Clone, Copy, Debug)]
struct ImageLocation {
    page_id: PageId,
    image_lsn: Lsn,
    offset: u64,
    checksum: u32,
}

#[derive(Debug)]
pub struct ValidatedCheckpointPack {
    source: Arc<PackSource>,
    locations: BTreeMap<PageId, ImageLocation>,
}

impl ValidatedCheckpointPack {
    pub fn open(
        directory: &Path,
        entry: &BasePackEntry,
        identity: &WalIdentity,
        high_water_page_id: PageId,
    ) -> Result<Self> {
        let mut file = File::open(pack_path(directory, entry.number))?;
        let mut locations = BTreeMap::new();
        scan_pack(
            &mut file,
            entry,
            identity,
            high_water_page_id,
            |page_id, image_lsn, offset, image| {
                locations.insert(
                    page_id,
                    ImageLocation {
                        page_id,
                        image_lsn,
                        offset,
                        checksum: crc32c::crc32c(image),
                    },
                );
                Ok(())
            },
        )?;
        Ok(Self {
            source: Arc::new(PackSource {
                file: Mutex::new(file),
                entry: entry.clone(),
                identity: identity.clone(),
            }),
            locations,
        })
    }

    pub fn page_count(&self) -> usize {
        self.locations.len()
    }

    pub fn locate(&self, page_id: PageId) -> Option<CheckpointPageLocator> {
        self.locations
            .get(&page_id)
            .map(|location| CheckpointPageLocator {
                source: Arc::clone(&self.source),
                location: *location,
            })
    }
}

#[derive(Clone, Debug)]
pub struct CheckpointPageLocator {
    source: Arc<PackSource>,
    location: ImageLocation,
}

impl CheckpointPageLocator {
    pub fn identity(&self) -> &WalIdentity {
        &self.source.identity
    }

    pub fn pack_number(&self) -> u64 {
        self.source.entry.number
    }

    pub fn checkpoint_lsn(&self) -> Lsn {
        self.source.entry.checkpoint_lsn
    }

    pub fn page_id(&self) -> PageId {
        self.location.page_id
    }

    pub fn image_lsn(&self) -> Lsn {
        self.location.image_lsn
    }

    pub fn read_image(&self) -> Result<Box<[u8; PAGE_SIZE]>> {
        let mut image = Box::new([0u8; PAGE_SIZE]);
        {
            let mut file = self
                .source
                .file
                .lock()
                .map_err(|_| Error::invariant("checkpoint locator file lock poisoned"))?;
            if file.metadata()?.len() != self.source.entry.bytes {
                return Err(Error::corruption("checkpoint locator file length changed"));
            }
            file.seek(SeekFrom::Start(self.location.offset))?;
            file.read_exact(&mut image[..])?;
        }
        if crc32c::crc32c(&image[..]) != self.location.checksum {
            return Err(Error::corruption("checkpoint locator image changed"));
        }
        let decoded = decode_blink_page(&image[..], self.location.page_id)?;
        if decoded.lsn() != self.location.image_lsn {
            return Err(Error::corruption("checkpoint locator image LSN changed"));
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    use dodb_core::{ShardEpoch, ShardId, TenantId};

    use super::*;
    use crate::blink::durable_bases::{BasePackJob, read_pack};
    use crate::blink::leaf::LeafEntries;
    use crate::blink::{BlinkPage, encode_blink_page};

    struct Fixture {
        directory: PathBuf,
        identity: WalIdentity,
        entry: BasePackEntry,
        images: BTreeMap<PageId, Arc<[u8; PAGE_SIZE]>>,
    }

    impl Fixture {
        fn new(coalesce: bool) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let directory = std::env::temp_dir().join(format!(
                "dodb-checkpoint-locator-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&directory).unwrap();
            let identity = WalIdentity::new(
                [7; 16],
                TenantId::new(2),
                ShardId::new(3),
                ShardEpoch::new(4),
            );
            let pages = [
                (
                    PageId::new(2),
                    BlinkPage::Leaf {
                        lsn: Lsn::new(7),
                        high_key: None,
                        right_sibling: None,
                        entries: LeafEntries::default(),
                    },
                ),
                (
                    PageId::new(9),
                    BlinkPage::Overflow {
                        lsn: Lsn::new(8),
                        next: None,
                        total_length: 16,
                        chunk: vec![8; 16],
                    },
                ),
                (
                    PageId::new(17),
                    BlinkPage::Free {
                        lsn: Lsn::new(9),
                        next: None,
                    },
                ),
            ];
            let images = pages
                .into_iter()
                .map(|(page_id, page)| {
                    (
                        page_id,
                        Arc::new(encode_blink_page(page_id, &page).unwrap()),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let output = BasePackJob {
                directory: directory.clone(),
                identity: identity.clone(),
                number: 3,
                checkpoint_lsn: Lsn::new(10),
                complete: true,
                coalesce,
            }
            .run(&images, &mut None)
            .unwrap();
            Self {
                directory,
                identity,
                entry: output.entry,
                images,
            }
        }

        fn path(&self) -> PathBuf {
            pack_path(&self.directory, self.entry.number)
        }

        fn open(&self) -> Result<ValidatedCheckpointPack> {
            ValidatedCheckpointPack::open(
                &self.directory,
                &self.entry,
                &self.identity,
                PageId::new(17),
            )
        }

        fn reseal(&mut self, mut bytes: Vec<u8>) {
            let header_checksum = crc32c::crc32c(&bytes[..PAGE_SIZE - 4]);
            bytes[PAGE_SIZE - 4..PAGE_SIZE].copy_from_slice(&header_checksum.to_le_bytes());
            let trailer_offset = bytes.len() - 4;
            self.entry.checksum = crc32c::crc32c(&bytes[..trailer_offset]);
            bytes[trailer_offset..].copy_from_slice(&self.entry.checksum.to_le_bytes());
            fs::write(self.path(), bytes).unwrap();
        }

        fn assert_both_reject(&self) {
            assert!(self.open().is_err());
            assert!(
                read_pack(
                    &self.directory,
                    &self.entry,
                    &self.identity,
                    PageId::new(17),
                    &mut BTreeMap::new(),
                    &mut BTreeMap::new(),
                )
                .is_err()
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if !thread::panicking() {
                fs::remove_dir_all(&self.directory).unwrap();
            }
        }
    }

    #[test]
    fn indexed_sparse_images_match_recovery_and_original_bytes() {
        for coalesce in [false, true] {
            let fixture = Fixture::new(coalesce);
            let pack = fixture.open().unwrap();
            let mut recovered = BTreeMap::new();
            let mut owners = BTreeMap::new();
            read_pack(
                &fixture.directory,
                &fixture.entry,
                &fixture.identity,
                PageId::new(17),
                &mut recovered,
                &mut owners,
            )
            .unwrap();
            assert_eq!(pack.page_count(), 3);
            assert!(pack.locate(PageId::new(3)).is_none());
            for (page_id, image) in &fixture.images {
                let locator = pack.locate(*page_id).unwrap();
                assert_eq!(locator.identity(), &fixture.identity);
                assert_eq!(locator.pack_number(), fixture.entry.number);
                assert_eq!(locator.checkpoint_lsn(), fixture.entry.checkpoint_lsn);
                assert_eq!(locator.page_id(), *page_id);
                assert_eq!(locator.read_image().unwrap().as_slice(), image.as_slice());
                assert_eq!(
                    locator.read_image().unwrap(),
                    recovered.get(page_id).unwrap().image
                );
                assert_eq!(owners.get(page_id), Some(&fixture.entry.number));
                assert_eq!(
                    locator.image_lsn(),
                    decode_blink_page(&image[..], *page_id).unwrap().lsn()
                );
            }
        }
    }

    #[test]
    fn full_scan_rejects_unrequested_corrupt_page_and_invalid_trailer() {
        for corrupt_offset in [PAGE_SIZE * 3 + 70, PAGE_SIZE * 4] {
            let fixture = Fixture::new(true);
            let mut bytes = fs::read(fixture.path()).unwrap();
            bytes[corrupt_offset] ^= 1;
            fs::write(fixture.path(), bytes).unwrap();
            fixture.assert_both_reject();
        }
    }

    #[test]
    fn full_scan_rejects_bad_header_crc_and_resealed_reserved_padding() {
        let mut fixture = Fixture::new(true);
        let original = fs::read(fixture.path()).unwrap();
        let mut bytes = original.clone();
        bytes[PAGE_SIZE - 1] ^= 1;
        fs::write(fixture.path(), bytes).unwrap();
        fixture.assert_both_reject();
        let mut bytes = original;
        bytes[80] = 1;
        fixture.reseal(bytes);
        fixture.assert_both_reject();
    }

    #[test]
    fn full_scan_rejects_wrong_database_tenant_shard_and_epoch() {
        let fixture = Fixture::new(true);
        for wrong_field in 0..4 {
            let mut identity = fixture.identity.clone();
            match wrong_field {
                0 => identity.database_uuid[0] ^= 1,
                1 => identity.tenant_id = TenantId::new(99),
                2 => identity.shard_id = ShardId::new(99),
                3 => identity.shard_epoch = ShardEpoch::new(99),
                _ => unreachable!(),
            }
            assert!(
                ValidatedCheckpointPack::open(
                    &fixture.directory,
                    &fixture.entry,
                    &identity,
                    PageId::new(17),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn full_scan_rejects_resealed_pack_and_checkpoint_header_mismatch() {
        for offset in [56, 64] {
            let mut fixture = Fixture::new(true);
            let mut bytes = fs::read(fixture.path()).unwrap();
            bytes[offset..offset + 8].copy_from_slice(&99u64.to_le_bytes());
            fixture.reseal(bytes);
            fixture.assert_both_reject();
        }
    }

    #[test]
    fn full_scan_rejects_duplicate_unsorted_and_out_of_range_pages() {
        let mut fixture = Fixture::new(true);
        let original = fs::read(fixture.path()).unwrap();
        let mut duplicate = original.clone();
        duplicate[PAGE_SIZE * 2..PAGE_SIZE * 3]
            .copy_from_slice(&original[PAGE_SIZE..PAGE_SIZE * 2]);
        fixture.reseal(duplicate);
        fixture.assert_both_reject();
        let mut reversed = original.clone();
        reversed[PAGE_SIZE..PAGE_SIZE * 2].copy_from_slice(&original[PAGE_SIZE * 2..PAGE_SIZE * 3]);
        reversed[PAGE_SIZE * 2..PAGE_SIZE * 3].copy_from_slice(&original[PAGE_SIZE..PAGE_SIZE * 2]);
        fixture.reseal(reversed);
        fixture.assert_both_reject();
        fixture.reseal(original);
        assert!(
            ValidatedCheckpointPack::open(
                &fixture.directory,
                &fixture.entry,
                &fixture.identity,
                PageId::new(16),
            )
            .is_err()
        );
    }

    #[test]
    fn full_scan_rejects_valid_page_with_lsn_above_checkpoint() {
        let mut fixture = Fixture::new(true);
        let mut bytes = fs::read(fixture.path()).unwrap();
        let newer = encode_blink_page(
            PageId::new(17),
            &BlinkPage::Free {
                lsn: Lsn::new(11),
                next: None,
            },
        )
        .unwrap();
        bytes[PAGE_SIZE * 3..PAGE_SIZE * 4].copy_from_slice(&newer);
        fixture.reseal(bytes);
        fixture.assert_both_reject();
    }

    #[test]
    fn full_scan_rejects_truncation_extra_bytes_and_wrong_record_count() {
        for length_delta in [-1isize, 1] {
            let fixture = Fixture::new(true);
            let mut bytes = fs::read(fixture.path()).unwrap();
            bytes.resize(bytes.len().checked_add_signed(length_delta).unwrap(), 0);
            fs::write(fixture.path(), bytes).unwrap();
            fixture.assert_both_reject();
        }
        let mut fixture = Fixture::new(true);
        fixture.entry.page_count += 1;
        fixture.assert_both_reject();
    }

    #[test]
    fn reload_rejects_changed_valid_image_and_truncated_source() {
        let fixture = Fixture::new(true);
        let pack = fixture.open().unwrap();
        let locator = pack.locate(PageId::new(17)).unwrap();
        let changed = encode_blink_page(
            PageId::new(17),
            &BlinkPage::Free {
                lsn: locator.image_lsn(),
                next: Some(PageId::new(2)),
            },
        )
        .unwrap();
        let mut file = OpenOptions::new().write(true).open(fixture.path()).unwrap();
        file.seek(SeekFrom::Start((PAGE_SIZE * 3) as u64)).unwrap();
        file.write_all(&changed).unwrap();
        file.sync_all().unwrap();
        assert!(locator.read_image().is_err());
        file.set_len(1).unwrap();
        assert!(locator.read_image().is_err());
    }

    #[test]
    fn concurrent_locators_do_not_share_loaded_payloads_or_mix_seek_positions() {
        let fixture = Fixture::new(true);
        let pack = fixture.open().unwrap();
        thread::scope(|scope| {
            for page_id in fixture.images.keys() {
                let locator = pack.locate(*page_id).unwrap();
                let expected = Arc::clone(fixture.images.get(page_id).unwrap());
                scope.spawn(move || {
                    for _load_attempt in 0..64 {
                        let mut first = locator.read_image().unwrap();
                        first[0] ^= 1;
                        assert_eq!(
                            locator.read_image().unwrap().as_slice(),
                            expected.as_slice()
                        );
                    }
                });
            }
        });
    }

    #[cfg(unix)]
    #[test]
    fn old_locator_survives_index_drop_unlink_and_pathname_replacement() {
        let fixture = Fixture::new(true);
        let pack = fixture.open().unwrap();
        let locator = pack.locate(PageId::new(9)).unwrap();
        let weak_source = Arc::downgrade(&locator.source);
        drop(pack);
        fs::remove_file(fixture.path()).unwrap();
        fs::write(fixture.path(), vec![99; fixture.entry.bytes as usize]).unwrap();
        assert!(fixture.open().is_err());
        assert_eq!(
            locator.read_image().unwrap().as_slice(),
            fixture.images.get(&PageId::new(9)).unwrap().as_slice()
        );
        drop(locator);
        assert!(weak_source.upgrade().is_none());
    }
}
