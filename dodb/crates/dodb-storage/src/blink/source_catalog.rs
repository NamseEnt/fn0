use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use dodb_core::{Error, Result};

use super::version_index::{NodeRef, SourceFileKind, SourceIdentity, StoreIdentity};

const PAGE_BYTES: usize = 4096;
const HEADER_BYTES: usize = 144;
const CHECKSUM_BYTES: usize = 32;
const RECORD_BYTES: usize = 48;
const RECORDS_PER_PAGE: usize = (PAGE_BYTES - HEADER_BYTES - CHECKSUM_BYTES) / RECORD_BYTES;
const PAYLOAD_END: usize = PAGE_BYTES - CHECKSUM_BYTES;
const MAGIC: [u8; 8] = *b"DDBSCAT1";
const FORMAT_VERSION: u16 = 1;
const FILE_PREFIX: &str = "source-catalog-";
const FILE_SUFFIX: &str = ".node";
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) struct SourceCatalogStore {
    directory: PathBuf,
    identity: StoreIdentity,
    next_file_number: AtomicU64,
}

impl SourceCatalogStore {
    pub(super) fn open(directory: &Path, identity: StoreIdentity) -> Result<Self> {
        fs::create_dir_all(directory)?;
        let mut maximum_file_number = 0u64;
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if let Some(file_number) = parse_catalog_file_number(&file_name)? {
                maximum_file_number = maximum_file_number.max(file_number);
            }
        }
        let next_file_number = maximum_file_number
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("source catalog file number exhausted"))?;
        Ok(Self {
            directory: directory.to_path_buf(),
            identity,
            next_file_number: AtomicU64::new(next_file_number.max(1)),
        })
    }

    pub(super) fn build_from_marker_run(&self, marker_directory: &Path) -> Result<NodeRef> {
        let temporary_file_number = TEMPORARY_SEQUENCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map_err(|_| Error::invalid_input("source catalog run number exhausted"))?;
        let run_path = marker_directory.join(format!(
            "source-catalog-run-{}-{temporary_file_number:020}.tmp",
            std::process::id()
        ));
        let mut run_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&run_path)?;
        let mut record_count = 0u64;
        for entry in fs::read_dir(marker_directory)? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if name == "retain-all-wal-segments" {
                run_file.write_all(&encode_retain_all_wal_record())?;
                record_count = record_count
                    .checked_add(1)
                    .ok_or_else(|| Error::overloaded("source catalog record count overflows"))?;
                continue;
            }
            let Some(source_hash) = name.strip_prefix("source-") else {
                continue;
            };
            if source_hash.len() != 64 || !source_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                continue;
            }
            let mut bytes = [0u8; RECORD_BYTES];
            let mut marker = File::open(entry.path())?;
            marker.read_exact(&mut bytes)?;
            if marker.metadata()?.len() != RECORD_BYTES as u64 {
                return Err(Error::corruption("source catalog marker length is invalid"));
            }
            decode_source_record(&bytes)?;
            run_file.write_all(&bytes)?;
            record_count = record_count
                .checked_add(1)
                .ok_or_else(|| Error::overloaded("source catalog record count overflows"))?;
        }
        run_file.flush()?;
        if record_count == 0 {
            return Err(Error::corruption("source catalog cannot be empty"));
        }
        let page_count = record_count
            .checked_add(RECORDS_PER_PAGE as u64 - 1)
            .ok_or_else(|| Error::overloaded("source catalog page count overflows"))?
            / RECORDS_PER_PAGE as u64;
        let first_file_number = self
            .next_file_number
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(page_count)
            })
            .map_err(|_| Error::invalid_input("source catalog file number exhausted"))?;
        let mut next_reference = None;
        for page_index in (0..page_count).rev() {
            let first_record = page_index
                .checked_mul(RECORDS_PER_PAGE as u64)
                .ok_or_else(|| Error::overloaded("source catalog record offset overflows"))?;
            let remaining_records = record_count - first_record;
            let page_record_count = remaining_records.min(RECORDS_PER_PAGE as u64) as usize;
            let record_bytes = page_record_count
                .checked_mul(RECORD_BYTES)
                .ok_or_else(|| Error::overloaded("source catalog page size overflows"))?;
            let run_offset = first_record
                .checked_mul(RECORD_BYTES as u64)
                .ok_or_else(|| Error::overloaded("source catalog run offset overflows"))?;
            run_file.seek(SeekFrom::Start(run_offset))?;
            let mut page = [0u8; PAGE_BYTES];
            page[0..8].copy_from_slice(&MAGIC);
            page[8..10].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
            page[10..12].copy_from_slice(&(page_record_count as u16).to_le_bytes());
            page[16..32].copy_from_slice(&self.identity.database_uuid);
            page[32..40].copy_from_slice(&self.identity.wal_epoch.to_le_bytes());
            let file_number = first_file_number
                .checked_add(page_index)
                .ok_or_else(|| Error::invalid_input("source catalog file number exhausted"))?;
            page[40..48].copy_from_slice(&file_number.to_le_bytes());
            if let Some(next_reference) = next_reference {
                encode_node_reference(next_reference, &mut page[48..96]);
            }
            let header_checksum = *blake3::hash(&page[..104]).as_bytes();
            page[104..136].copy_from_slice(&header_checksum);
            run_file.read_exact(&mut page[HEADER_BYTES..HEADER_BYTES + record_bytes])?;
            let body_checksum = *blake3::hash(&page[..PAYLOAD_END]).as_bytes();
            page[PAYLOAD_END..].copy_from_slice(&body_checksum);
            let path = catalog_path(&self.directory, file_number);
            let mut output = OpenOptions::new().write(true).create_new(true).open(path)?;
            output.write_all(&page)?;
            output.sync_all()?;
            next_reference = Some(NodeRef {
                file_number,
                offset: 0,
                checksum: body_checksum,
            });
        }
        File::open(&self.directory)?.sync_all()?;
        next_reference.ok_or_else(|| Error::invariant("source catalog root was not written"))
    }

    pub(super) fn estimate_marker_run_bytes(&self, marker_directory: &Path) -> Result<u64> {
        let mut record_count = 0u64;
        for entry in fs::read_dir(marker_directory)? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if name == "retain-all-wal-segments" {
                record_count = record_count
                    .checked_add(1)
                    .ok_or_else(|| Error::overloaded("source catalog record count overflows"))?;
                continue;
            }
            let Some(source_hash) = name.strip_prefix("source-") else {
                continue;
            };
            if source_hash.len() != 64 || !source_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                continue;
            }
            let metadata = entry.metadata()?;
            if !metadata.is_file() || metadata.len() != RECORD_BYTES as u64 {
                return Err(Error::corruption("source catalog marker length is invalid"));
            }
            let mut bytes = [0u8; RECORD_BYTES];
            File::open(entry.path())?.read_exact(&mut bytes)?;
            decode_source_record(&bytes)?;
            record_count = record_count
                .checked_add(1)
                .ok_or_else(|| Error::overloaded("source catalog record count overflows"))?;
        }
        if record_count == 0 {
            return Err(Error::corruption("source catalog cannot be empty"));
        }
        let page_count = record_count
            .checked_add(RECORDS_PER_PAGE as u64 - 1)
            .ok_or_else(|| Error::overloaded("source catalog page count overflows"))?
            / RECORDS_PER_PAGE as u64;
        page_count
            .checked_mul(PAGE_BYTES as u64)
            .ok_or_else(|| Error::overloaded("source catalog byte count overflows"))
    }

    pub(super) fn visit<F>(&self, root: NodeRef, visitor: &mut F) -> Result<()>
    where
        F: FnMut(SourceCatalogRecord) -> Result<()>,
    {
        if root.file_number == 0 || root.offset != 0 {
            return Err(Error::corruption(
                "source catalog root reference is invalid",
            ));
        }
        let mut current_reference = Some(root);
        while let Some(reference) = current_reference {
            let path = catalog_path(&self.directory, reference.file_number);
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Err(Error::corruption(
                        "reachable source catalog page is missing",
                    ));
                }
                Err(error) => return Err(error.into()),
            };
            if file.metadata()?.len() != PAGE_BYTES as u64 {
                return Err(Error::corruption("source catalog page length is invalid"));
            }
            let mut page = [0u8; PAGE_BYTES];
            file.read_exact(&mut page)?;
            let body_checksum = *blake3::hash(&page[..PAYLOAD_END]).as_bytes();
            if body_checksum != reference.checksum {
                return Err(Error::corruption("source catalog page checksum mismatch"));
            }
            if page[0..8] != MAGIC
                || u16::from_le_bytes(page[8..10].try_into().unwrap()) != FORMAT_VERSION
                || page[12..16].iter().any(|byte| *byte != 0)
                || page[16..32] != self.identity.database_uuid
                || u64::from_le_bytes(page[32..40].try_into().unwrap()) != self.identity.wal_epoch
                || u64::from_le_bytes(page[40..48].try_into().unwrap()) != reference.file_number
                || page[96..104].iter().any(|byte| *byte != 0)
                || blake3::hash(&page[..104]).as_bytes() != &page[104..136]
                || page[136..144].iter().any(|byte| *byte != 0)
            {
                return Err(Error::corruption("source catalog page header is invalid"));
            }
            let record_count = usize::from(u16::from_le_bytes(page[10..12].try_into().unwrap()));
            if record_count == 0 || record_count > RECORDS_PER_PAGE {
                return Err(Error::corruption(
                    "source catalog page record count is invalid",
                ));
            }
            let next_reference = decode_optional_node_reference(&page[48..96])?;
            let records_end = HEADER_BYTES + record_count * RECORD_BYTES;
            if page[records_end..PAYLOAD_END].iter().any(|byte| *byte != 0) {
                return Err(Error::corruption("source catalog page padding is not zero"));
            }
            for record_index in 0..record_count {
                let record_offset = HEADER_BYTES + record_index * RECORD_BYTES;
                let record = decode_source_record(
                    &page[record_offset..record_offset + RECORD_BYTES]
                        .try_into()
                        .unwrap(),
                )?;
                visitor(record)?;
            }
            if let Some(next_reference) = next_reference
                && (next_reference.file_number
                    != reference.file_number.checked_add(1).ok_or_else(|| {
                        Error::corruption("source catalog page sequence overflows")
                    })?
                    || next_reference.offset != 0)
            {
                return Err(Error::corruption("source catalog page link is invalid"));
            }
            if next_reference.is_some() && record_count != RECORDS_PER_PAGE {
                return Err(Error::corruption(
                    "nonterminal source catalog page is not full",
                ));
            }
            current_reference = next_reference;
        }
        Ok(())
    }

    pub(super) fn reclaim_unreachable(
        &self,
        roots: impl IntoIterator<Item = NodeRef>,
    ) -> Result<(u64, u64)> {
        let mut retained_ranges = Vec::new();
        for root in roots {
            let mut record_count = 0u64;
            self.visit(root, &mut |_| {
                record_count = record_count
                    .checked_add(1)
                    .ok_or_else(|| Error::corruption("source catalog record count overflows"))?;
                Ok(())
            })?;
            let page_count = record_count
                .checked_add(RECORDS_PER_PAGE as u64 - 1)
                .ok_or_else(|| Error::corruption("source catalog page count overflows"))?
                / RECORDS_PER_PAGE as u64;
            let last_file_number = root
                .file_number
                .checked_add(page_count - 1)
                .ok_or_else(|| Error::corruption("source catalog file range overflows"))?;
            retained_ranges.push((root.file_number, last_file_number));
        }
        retained_ranges.sort_unstable();
        retained_ranges.dedup();
        let mut unlinked_pages = 0u64;
        let mut unlinked_bytes = 0u64;
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some(file_number) = parse_catalog_file_number(&file_name)? else {
                continue;
            };
            if retained_ranges
                .iter()
                .any(|(first, last)| (*first..=*last).contains(&file_number))
            {
                continue;
            }
            let metadata = entry.metadata()?;
            if !metadata.is_file() {
                return Err(Error::corruption(
                    "unreachable source catalog path is not a file",
                ));
            }
            fs::remove_file(entry.path())?;
            unlinked_pages = unlinked_pages
                .checked_add(1)
                .ok_or_else(|| Error::invalid_input("source catalog GC page count overflows"))?;
            unlinked_bytes = unlinked_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| Error::invalid_input("source catalog GC byte count overflows"))?;
        }
        if unlinked_pages > 0 {
            File::open(&self.directory)?.sync_all()?;
        }
        Ok((unlinked_pages, unlinked_bytes))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SourceCatalogRecord {
    Exact(SourceIdentity),
    AllWalSegments,
}

fn encode_retain_all_wal_record() -> [u8; RECORD_BYTES] {
    let mut bytes = [0u8; RECORD_BYTES];
    bytes[0] = 3;
    bytes
}

fn decode_source_record(bytes: &[u8; RECORD_BYTES]) -> Result<SourceCatalogRecord> {
    if bytes[1..8].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption(
            "source catalog record padding is not zero",
        ));
    }
    let file_number = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let header_identity: [u8; 32] = bytes[16..48].try_into().unwrap();
    match bytes[0] {
        1 | 2 => {
            let kind = if bytes[0] == 1 {
                SourceFileKind::CheckpointPack
            } else {
                SourceFileKind::WalSegment
            };
            if file_number == 0 {
                return Err(Error::corruption("source catalog file number is zero"));
            }
            Ok(SourceCatalogRecord::Exact(SourceIdentity {
                kind,
                file_number,
                header_identity,
            }))
        }
        3 if file_number == 0 && header_identity.iter().all(|byte| *byte == 0) => {
            Ok(SourceCatalogRecord::AllWalSegments)
        }
        _ => Err(Error::corruption("source catalog record kind is invalid")),
    }
}

pub(super) fn encode_source_record(identity: SourceIdentity) -> [u8; RECORD_BYTES] {
    let mut bytes = [0u8; RECORD_BYTES];
    bytes[0] = identity.kind as u8;
    bytes[8..16].copy_from_slice(&identity.file_number.to_le_bytes());
    bytes[16..48].copy_from_slice(&identity.header_identity);
    bytes
}

fn encode_node_reference(reference: NodeRef, bytes: &mut [u8]) {
    bytes[0..8].copy_from_slice(&reference.file_number.to_le_bytes());
    bytes[8..16].copy_from_slice(&reference.offset.to_le_bytes());
    bytes[16..48].copy_from_slice(&reference.checksum);
}

fn decode_optional_node_reference(bytes: &[u8]) -> Result<Option<NodeRef>> {
    let file_number = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
    let offset = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let checksum = <[u8; 32]>::try_from(&bytes[16..48]).unwrap();
    if file_number == 0 && offset == 0 && checksum.iter().all(|byte| *byte == 0) {
        return Ok(None);
    }
    if file_number == 0 || offset != 0 || checksum.iter().all(|byte| *byte == 0) {
        return Err(Error::corruption(
            "source catalog next-page reference is invalid",
        ));
    }
    Ok(Some(NodeRef {
        file_number,
        offset,
        checksum,
    }))
}

fn catalog_path(directory: &Path, file_number: u64) -> PathBuf {
    directory.join(format!("{FILE_PREFIX}{file_number:020}{FILE_SUFFIX}"))
}

fn parse_catalog_file_number(file_name: &str) -> Result<Option<u64>> {
    let Some(number) = file_name
        .strip_prefix(FILE_PREFIX)
        .and_then(|name| name.strip_suffix(FILE_SUFFIX))
    else {
        return Ok(None);
    };
    if number.len() != 20 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::corruption("source catalog filename is invalid"));
    }
    let file_number = number
        .parse::<u64>()
        .map_err(|_| Error::corruption("source catalog filename is invalid"))?;
    if file_number == 0 {
        return Err(Error::corruption("source catalog filename is invalid"));
    }
    Ok(Some(file_number))
}

#[cfg(test)]
mod tests {
    use super::{RECORDS_PER_PAGE, SourceCatalogRecord, SourceCatalogStore, encode_source_record};
    use crate::blink::version_index::{SourceFileKind, SourceIdentity, StoreIdentity};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "dodb-source-catalog-{}-{sequence:020}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn source_catalog_streams_multi_page_marker_runs_and_rejects_corruption() {
        let directory = TestDirectory::new();
        let checkpoint_directory = directory.0.join("checkpoint");
        let marker_directory = directory.0.join("markers");
        fs::create_dir_all(&marker_directory).unwrap();
        let identity = StoreIdentity {
            database_uuid: [19; 16],
            wal_epoch: 23,
        };
        let source_identities = (1..=(RECORDS_PER_PAGE as u64 * 2 + 7))
            .map(|file_number| SourceIdentity {
                kind: if file_number % 2 == 0 {
                    SourceFileKind::CheckpointPack
                } else {
                    SourceFileKind::WalSegment
                },
                file_number,
                header_identity: *blake3::hash(&file_number.to_le_bytes()).as_bytes(),
            })
            .collect::<Vec<_>>();
        for (marker_number, source) in source_identities.iter().copied().enumerate() {
            let mut marker =
                File::create(marker_directory.join(format!("source-{marker_number:064x}")))
                    .unwrap();
            marker.write_all(&encode_source_record(source)).unwrap();
        }
        let store = SourceCatalogStore::open(&checkpoint_directory, identity).unwrap();
        let root = store.build_from_marker_run(&marker_directory).unwrap();
        assert_eq!(root.file_number, 1);
        let mut actual = Vec::new();
        store
            .visit(root, &mut |record| {
                if let SourceCatalogRecord::Exact(source) = record {
                    actual.push(source);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(actual.len(), source_identities.len());
        assert!(
            source_identities
                .iter()
                .all(|source| actual.contains(source))
        );

        let middle_page_path =
            checkpoint_directory.join(format!("source-catalog-{:020}.node", root.file_number + 1));
        let mut middle_page = fs::read(&middle_page_path).unwrap();
        middle_page[RECORDS_PER_PAGE + 200] ^= 1;
        fs::write(middle_page_path, middle_page).unwrap();
        assert!(store.visit(root, &mut |_| Ok(())).is_err());
    }

    #[test]
    fn source_catalog_reclaims_only_pages_outside_protected_roots() {
        let directory = TestDirectory::new();
        let checkpoint_directory = directory.0.join("checkpoint");
        let marker_directory = directory.0.join("markers");
        fs::create_dir_all(&marker_directory).unwrap();
        let identity = StoreIdentity {
            database_uuid: [29; 16],
            wal_epoch: 31,
        };
        for marker_number in 0..=(RECORDS_PER_PAGE as u64) {
            let source = SourceIdentity {
                kind: SourceFileKind::WalSegment,
                file_number: marker_number + 1,
                header_identity: *blake3::hash(&marker_number.to_le_bytes()).as_bytes(),
            };
            let mut marker =
                File::create(marker_directory.join(format!("source-{marker_number:064x}")))
                    .unwrap();
            marker.write_all(&encode_source_record(source)).unwrap();
        }
        let store = SourceCatalogStore::open(&checkpoint_directory, identity).unwrap();
        let obsolete_root = store.build_from_marker_run(&marker_directory).unwrap();
        for entry in fs::read_dir(&marker_directory).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        let current_source = SourceIdentity {
            kind: SourceFileKind::CheckpointPack,
            file_number: 500,
            header_identity: *blake3::hash(b"current source").as_bytes(),
        };
        let mut current_marker =
            File::create(marker_directory.join(format!("source-{:064x}", 500))).unwrap();
        current_marker
            .write_all(&encode_source_record(current_source))
            .unwrap();
        let current_root = store.build_from_marker_run(&marker_directory).unwrap();

        let (unlinked_pages, unlinked_bytes) = store.reclaim_unreachable([current_root]).unwrap();

        assert_eq!(unlinked_pages, 2);
        assert_eq!(unlinked_bytes, (2 * super::PAGE_BYTES) as u64);
        assert!(
            !checkpoint_directory
                .join(format!(
                    "source-catalog-{:020}.node",
                    obsolete_root.file_number
                ))
                .exists()
        );
        assert!(
            checkpoint_directory
                .join(format!(
                    "source-catalog-{:020}.node",
                    current_root.file_number
                ))
                .is_file()
        );
    }
}
