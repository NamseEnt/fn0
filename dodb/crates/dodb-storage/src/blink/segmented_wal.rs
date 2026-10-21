use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dodb_core::{Error, Lsn, PageId, Result, ShardEpoch, ShardId, TenantId};

use crate::page::PAGE_SIZE;
use crate::wal::WalIdentity;

const MANIFEST_MAGIC: [u8; 8] = *b"DBLKSWM1";
const MANIFEST_VERSION: u32 = 1;
const CHECKSUM_SIZE: usize = 4;
const FIXED_SIZE: usize = 120;
const SEGMENT_SIZE: usize = 32;
const MAX_SEGMENTS: usize = 65_536;
static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalSegmentEntry {
    pub number: u64,
    pub start_lsn: Lsn,
    pub end_lsn: Lsn,
    pub closed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentedWalManifest {
    pub generation: u64,
    pub identity: WalIdentity,
    pub checkpoint_lsn: Lsn,
    pub root_page_id: PageId,
    pub free_list_head: Option<PageId>,
    pub high_water_page_id: PageId,
    pub format_version: u32,
    pub segments: Vec<WalSegmentEntry>,
    pub active_segment: u64,
    pub next_segment: u64,
}

impl SegmentedWalManifest {
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let segment_count = u32::try_from(self.segments.len())
            .map_err(|_| Error::invalid_input("too many WAL segments"))?;
        let mut bytes = Vec::with_capacity(
            FIXED_SIZE
                .saturating_add(self.segments.len().saturating_mul(SEGMENT_SIZE))
                .saturating_add(CHECKSUM_SIZE),
        );
        bytes.extend_from_slice(&MANIFEST_MAGIC);
        bytes.extend_from_slice(&MANIFEST_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.identity.database_uuid);
        bytes.extend_from_slice(&self.identity.tenant_id.get().to_le_bytes());
        bytes.extend_from_slice(&self.identity.shard_id.get().to_le_bytes());
        bytes.extend_from_slice(&self.identity.shard_epoch.get().to_le_bytes());
        bytes.extend_from_slice(&self.checkpoint_lsn.get().to_le_bytes());
        bytes.extend_from_slice(&self.root_page_id.get().to_le_bytes());
        bytes.extend_from_slice(
            &self
                .free_list_head
                .map_or(u64::MAX, PageId::get)
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&self.high_water_page_id.get().to_le_bytes());
        bytes.extend_from_slice(&self.format_version.to_le_bytes());
        bytes.extend_from_slice(&segment_count.to_le_bytes());
        bytes.extend_from_slice(&self.active_segment.to_le_bytes());
        bytes.extend_from_slice(&self.next_segment.to_le_bytes());
        bytes.resize(FIXED_SIZE, 0);
        for segment in &self.segments {
            bytes.extend_from_slice(&segment.number.to_le_bytes());
            bytes.extend_from_slice(&segment.start_lsn.get().to_le_bytes());
            bytes.extend_from_slice(&segment.end_lsn.get().to_le_bytes());
            bytes.push(u8::from(segment.closed));
            bytes.extend_from_slice(&[0; 7]);
        }
        let checksum = crc32c::crc32c(&bytes);
        bytes.extend_from_slice(&checksum.to_le_bytes());
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < FIXED_SIZE + CHECKSUM_SIZE || bytes[..8] != MANIFEST_MAGIC {
            return Err(Error::corruption(
                "segmented WAL manifest header is invalid",
            ));
        }
        let version = read_u32(bytes, 8)?;
        if version != MANIFEST_VERSION {
            return Err(Error::unsupported_format(format!(
                "segmented WAL manifest version {version} is unsupported"
            )));
        }
        if read_u32(bytes, 12)? as usize != PAGE_SIZE {
            return Err(Error::unsupported_format(
                "segmented WAL manifest page size is incompatible",
            ));
        }
        let segment_count = read_u32(bytes, 100)? as usize;
        if segment_count > MAX_SEGMENTS {
            return Err(Error::corruption(
                "segmented WAL manifest has too many entries",
            ));
        }
        let expected_length = FIXED_SIZE
            .checked_add(
                segment_count
                    .checked_mul(SEGMENT_SIZE)
                    .ok_or_else(|| Error::corruption("segmented WAL manifest length overflow"))?,
            )
            .and_then(|length| length.checked_add(CHECKSUM_SIZE))
            .ok_or_else(|| Error::corruption("segmented WAL manifest length overflow"))?;
        if bytes.len() != expected_length {
            return Err(Error::corruption(
                "segmented WAL manifest length is invalid",
            ));
        }
        let checksum_offset = bytes.len() - CHECKSUM_SIZE;
        let expected_checksum = u32::from_le_bytes(
            bytes[checksum_offset..]
                .try_into()
                .map_err(|_| Error::corruption("segmented WAL manifest checksum is truncated"))?,
        );
        if crc32c::crc32c(&bytes[..checksum_offset]) != expected_checksum {
            return Err(Error::corruption(
                "segmented WAL manifest checksum mismatch",
            ));
        }
        let identity = WalIdentity::new(
            bytes[24..40]
                .try_into()
                .map_err(|_| Error::corruption("segmented WAL database identity is truncated"))?,
            TenantId::new(read_u64(bytes, 40)?),
            ShardId::new(read_u64(bytes, 48)?),
            ShardEpoch::new(read_u64(bytes, 56)?),
        );
        let free_list_value = read_u64(bytes, 80)?;
        let mut segments = Vec::with_capacity(segment_count);
        for segment_index in 0..segment_count {
            let offset = FIXED_SIZE + segment_index * SEGMENT_SIZE;
            let closed = match bytes[offset + 24] {
                0 => false,
                1 => true,
                _ => return Err(Error::corruption("WAL segment closed flag is invalid")),
            };
            if bytes[offset + 25..offset + SEGMENT_SIZE]
                .iter()
                .any(|value| *value != 0)
            {
                return Err(Error::corruption("WAL segment reserved bytes are nonzero"));
            }
            segments.push(WalSegmentEntry {
                number: read_u64(bytes, offset)?,
                start_lsn: Lsn::new(read_u64(bytes, offset + 8)?),
                end_lsn: Lsn::new(read_u64(bytes, offset + 16)?),
                closed,
            });
        }
        let manifest = Self {
            generation: read_u64(bytes, 16)?,
            identity,
            checkpoint_lsn: Lsn::new(read_u64(bytes, 64)?),
            root_page_id: PageId::new(read_u64(bytes, 72)?),
            free_list_head: (free_list_value != u64::MAX).then(|| PageId::new(free_list_value)),
            high_water_page_id: PageId::new(read_u64(bytes, 88)?),
            format_version: read_u32(bytes, 96)?,
            segments,
            active_segment: read_u64(bytes, 104)?,
            next_segment: read_u64(bytes, 112)?,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<()> {
        if self.generation == 0 || self.format_version == 0 {
            return Err(Error::corruption(
                "segmented WAL manifest generation or format is zero",
            ));
        }
        if self.segments.is_empty() || self.segments.len() > MAX_SEGMENTS {
            return Err(Error::corruption(
                "segmented WAL manifest segment list is invalid",
            ));
        }
        if self.next_segment <= self.active_segment {
            return Err(Error::corruption(
                "segmented WAL next segment number is not ahead",
            ));
        }
        let mut previous_number: Option<u64> = None;
        let mut previous_end: Option<Lsn> = None;
        let mut active_count = 0usize;
        for (segment_index, segment) in self.segments.iter().enumerate() {
            if segment.start_lsn > segment.end_lsn {
                return Err(Error::corruption("WAL segment LSN range is reversed"));
            }
            if segment.start_lsn <= self.checkpoint_lsn && segment.end_lsn > self.checkpoint_lsn {
                return Err(Error::corruption(
                    "WAL segment range straddles the durable checkpoint boundary",
                ));
            }
            if let Some(number) = previous_number {
                if segment.number != number.saturating_add(1) {
                    return Err(Error::corruption("WAL segment numbers are not contiguous"));
                }
            }
            if let Some(end) = previous_end {
                if segment.start_lsn.get() != end.get().saturating_add(1) {
                    return Err(Error::corruption(
                        "WAL segment LSN ranges are not contiguous",
                    ));
                }
            }
            if segment.number == self.active_segment {
                if segment.closed || segment_index + 1 != self.segments.len() {
                    return Err(Error::corruption("active WAL segment is marked closed"));
                }
                active_count += 1;
            } else if !segment.closed {
                return Err(Error::corruption("non-active WAL segment is not closed"));
            }
            previous_number = Some(segment.number);
            previous_end = Some(segment.end_lsn);
        }
        if active_count != 1 {
            return Err(Error::corruption(
                "segmented WAL manifest has no unique active file",
            ));
        }
        if self.next_segment
            != self
                .segments
                .last()
                .map_or(0, |segment| segment.number.saturating_add(1))
        {
            return Err(Error::corruption(
                "next WAL segment number does not follow the active file",
            ));
        }
        Ok(())
    }

    pub fn write_atomic(&self, path: &Path) -> Result<()> {
        let bytes = self.encode()?;
        let parent = path
            .parent()
            .ok_or_else(|| Error::invalid_input("manifest path has no parent directory"))?;
        fs::create_dir_all(parent)?;
        let temporary_path = temporary_path(path, self.generation);
        let write_result = write_synced(&temporary_path, &bytes);
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }
        fs::rename(&temporary_path, path)?;
        sync_directory(parent)?;
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        let length = usize::try_from(length)
            .map_err(|_| Error::corruption("segmented WAL manifest is too large"))?;
        if length > FIXED_SIZE + MAX_SEGMENTS * SEGMENT_SIZE + CHECKSUM_SIZE {
            return Err(Error::corruption(
                "segmented WAL manifest exceeds its size limit",
            ));
        }
        let mut bytes = Vec::with_capacity(length);
        file.read_to_end(&mut bytes)?;
        Self::decode(&bytes)
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| Error::corruption("manifest offset overflow"))?;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| Error::corruption("manifest field is truncated"))?;
    Ok(u32::from_le_bytes(value.try_into().map_err(|_| {
        Error::corruption("manifest field is truncated")
    })?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    let end = offset
        .checked_add(8)
        .ok_or_else(|| Error::corruption("manifest offset overflow"))?;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| Error::corruption("manifest field is truncated"))?;
    Ok(u64::from_le_bytes(value.try_into().map_err(|_| {
        Error::corruption("manifest field is truncated")
    })?))
}

fn temporary_path(path: &Path, generation: u64) -> PathBuf {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut temporary_name = path.file_name().unwrap_or_default().to_os_string();
    temporary_name.push(format!(
        ".tmp-{generation}-{}-{timestamp_nanos}-{sequence}",
        std::process::id()
    ));
    path.with_file_name(temporary_name)
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> SegmentedWalManifest {
        SegmentedWalManifest {
            generation: 1,
            identity: WalIdentity::new(
                [7; 16],
                TenantId::new(2),
                ShardId::new(3),
                ShardEpoch::new(4),
            ),
            checkpoint_lsn: Lsn::new(10),
            root_page_id: PageId::new(2),
            free_list_head: Some(PageId::new(5)),
            high_water_page_id: PageId::new(9),
            format_version: 1,
            segments: vec![
                WalSegmentEntry {
                    number: 1,
                    start_lsn: Lsn::new(11),
                    end_lsn: Lsn::new(20),
                    closed: true,
                },
                WalSegmentEntry {
                    number: 2,
                    start_lsn: Lsn::new(21),
                    end_lsn: Lsn::new(30),
                    closed: false,
                },
            ],
            active_segment: 2,
            next_segment: 3,
        }
    }

    #[test]
    fn manifest_round_trip_preserves_identity_and_ranges() {
        let expected = manifest();
        assert_eq!(
            SegmentedWalManifest::decode(&expected.encode().unwrap()).unwrap(),
            expected
        );
    }

    #[test]
    fn manifest_rejects_corruption_and_invalid_ranges() {
        let mut bytes = manifest().encode().unwrap();
        bytes[45] ^= 1;
        assert!(SegmentedWalManifest::decode(&bytes).is_err());

        let mut invalid = manifest();
        invalid.segments[1].start_lsn = Lsn::new(20);
        assert!(invalid.encode().is_err());
    }

    #[test]
    fn manifest_atomic_write_round_trips() {
        let root = std::env::temp_dir().join(format!(
            "dodb-segmented-manifest-{}-{}",
            std::process::id(),
            manifest().generation
        ));
        let path = root.join("manifest");
        let expected = manifest();
        expected.write_atomic(&path).unwrap();
        assert_eq!(SegmentedWalManifest::read(&path).unwrap(), expected);
        fs::remove_dir_all(root).unwrap();
    }
}
