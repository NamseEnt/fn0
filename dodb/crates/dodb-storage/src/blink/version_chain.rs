use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use dodb_core::{Error, Lsn, PageId, Result};

use super::resource::{BlinkResourceManager, Reservation, ReservationClass, ResourceCategory};
use super::version_index::{SourceFileKind, SourceIdentity, StoreIdentity, WalFrameLocator};

const CHAIN_MAGIC: [u8; 8] = *b"DDBVCHN1";
const CHAIN_FORMAT_VERSION: u16 = 1;
pub const DELTA_CHAIN_FORMAT_ID: &str = "DDBVCHN1/v1";
const CHAIN_HEADER_BYTES: usize = 128;
const CHAIN_ENTRY_BYTES: usize = 136;
const CHAIN_CHECKSUM_BYTES: usize = 32;
const MAX_CHAIN_ENTRIES: usize = 9;
const MAX_CHAIN_FILE_BYTES: usize =
    CHAIN_HEADER_BYTES + CHAIN_ENTRY_BYTES * MAX_CHAIN_ENTRIES + CHAIN_CHECKSUM_BYTES;
const CHAIN_FILE_PREFIX: &str = "version-chain-";
const CHAIN_FILE_SUFFIX: &str = ".bin";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeltaChainRef {
    pub file_number: u64,
    pub offset: u64,
    pub checksum: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeltaChainEntry {
    CheckpointImage {
        source: SourceIdentity,
        offset: u64,
        source_length: u64,
        image_lsn: Lsn,
        image_crc32c: u32,
    },
    WalImage {
        frame: WalFrameLocator,
        record_lsn: Lsn,
        image_lsn: Lsn,
        image_crc32c: u32,
    },
    WalDelta {
        frame: WalFrameLocator,
        record_lsn: Lsn,
        base_lsn: Lsn,
        base_crc32c: u32,
        image_lsn: Lsn,
        image_crc32c: u32,
    },
}

impl DeltaChainEntry {
    pub fn image_lsn(self) -> Lsn {
        match self {
            Self::CheckpointImage { image_lsn, .. }
            | Self::WalImage { image_lsn, .. }
            | Self::WalDelta { image_lsn, .. } => image_lsn,
        }
    }

    pub fn image_crc32c(self) -> u32 {
        match self {
            Self::CheckpointImage { image_crc32c, .. }
            | Self::WalImage { image_crc32c, .. }
            | Self::WalDelta { image_crc32c, .. } => image_crc32c,
        }
    }

    pub fn source_identity(self) -> SourceIdentity {
        match self {
            Self::CheckpointImage { source, .. } => source,
            Self::WalImage { frame, .. } | Self::WalDelta { frame, .. } => frame.source,
        }
    }

    fn encode(self, page_id: PageId, output: &mut [u8]) -> Result<()> {
        if output.len() != CHAIN_ENTRY_BYTES {
            return Err(Error::invariant(
                "delta-chain entry output has wrong length",
            ));
        }
        output.fill(0);
        let (
            entry_kind,
            source,
            offset,
            frame_length,
            header_crc32c,
            payload_offset,
            payload_length,
            payload_crc32c,
            source_length,
            commit_frame_offset,
            commit_lsn,
            record_lsn,
            image_lsn,
            image_crc32c,
            base_lsn,
            base_crc32c,
        ) = match self {
            Self::CheckpointImage {
                source,
                offset,
                source_length,
                image_lsn,
                image_crc32c,
            } => (
                1,
                source,
                offset,
                u32::try_from(crate::page::PAGE_SIZE)
                    .map_err(|_| Error::invariant("page size does not fit chain entry"))?,
                0,
                offset,
                crate::page::PAGE_SIZE as u32,
                image_crc32c,
                source_length,
                0,
                Lsn::ZERO,
                Lsn::ZERO,
                image_lsn,
                image_crc32c,
                Lsn::ZERO,
                0,
            ),
            Self::WalImage {
                frame,
                record_lsn,
                image_lsn,
                image_crc32c,
            } => (
                2,
                frame.source,
                frame.frame_offset,
                frame.frame_length,
                frame.header_crc32c,
                frame.payload_offset,
                frame.payload_length,
                frame.payload_crc32c,
                frame.committed_prefix_length,
                frame.commit_frame_offset,
                frame.commit_lsn,
                record_lsn,
                image_lsn,
                image_crc32c,
                Lsn::ZERO,
                0,
            ),
            Self::WalDelta {
                frame,
                record_lsn,
                base_lsn,
                base_crc32c,
                image_lsn,
                image_crc32c,
            } => (
                3,
                frame.source,
                frame.frame_offset,
                frame.frame_length,
                frame.header_crc32c,
                frame.payload_offset,
                frame.payload_length,
                frame.payload_crc32c,
                frame.committed_prefix_length,
                frame.commit_frame_offset,
                frame.commit_lsn,
                record_lsn,
                image_lsn,
                image_crc32c,
                base_lsn,
                base_crc32c,
            ),
        };
        if page_id.get() < 2 || source.file_number == 0 {
            return Err(Error::invalid_input(
                "delta-chain entry identity is invalid",
            ));
        }
        output[0] = entry_kind;
        output[1] = source.kind as u8;
        output[4..12].copy_from_slice(&source.file_number.to_le_bytes());
        output[12..44].copy_from_slice(&source.header_identity);
        output[44..52].copy_from_slice(&offset.to_le_bytes());
        output[52..56].copy_from_slice(&frame_length.to_le_bytes());
        output[56..60].copy_from_slice(&header_crc32c.to_le_bytes());
        output[60..68].copy_from_slice(&payload_offset.to_le_bytes());
        output[68..72].copy_from_slice(&payload_length.to_le_bytes());
        output[72..76].copy_from_slice(&payload_crc32c.to_le_bytes());
        output[76..84].copy_from_slice(&source_length.to_le_bytes());
        output[84..92].copy_from_slice(&commit_frame_offset.to_le_bytes());
        output[92..100].copy_from_slice(&commit_lsn.get().to_le_bytes());
        output[100..108].copy_from_slice(&record_lsn.get().to_le_bytes());
        output[108..116].copy_from_slice(&image_lsn.get().to_le_bytes());
        output[116..120].copy_from_slice(&image_crc32c.to_le_bytes());
        output[120..128].copy_from_slice(&base_lsn.get().to_le_bytes());
        output[128..132].copy_from_slice(&base_crc32c.to_le_bytes());
        Ok(())
    }

    fn decode(page_id: PageId, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != CHAIN_ENTRY_BYTES || bytes[2..4] != [0; 2] || bytes[132..136] != [0; 4] {
            return Err(Error::corruption(
                "delta-chain entry length or reserved bytes are invalid",
            ));
        }
        let kind = bytes[0];
        let source_kind = match bytes[1] {
            1 => SourceFileKind::CheckpointPack,
            2 => SourceFileKind::WalSegment,
            _ => return Err(Error::corruption("delta-chain source kind is invalid")),
        };
        let source = SourceIdentity {
            kind: source_kind,
            file_number: read_u64(bytes, 4)?,
            header_identity: bytes[12..44]
                .try_into()
                .map_err(|_| Error::corruption("delta-chain source identity is truncated"))?,
        };
        let offset = read_u64(bytes, 44)?;
        let frame_length = read_u32(bytes, 52)?;
        let header_crc32c = read_u32(bytes, 56)?;
        let payload_offset = read_u64(bytes, 60)?;
        let payload_length = read_u32(bytes, 68)?;
        let payload_crc32c = read_u32(bytes, 72)?;
        let source_length = read_u64(bytes, 76)?;
        let commit_frame_offset = read_u64(bytes, 84)?;
        let commit_lsn = Lsn::new(read_u64(bytes, 92)?);
        let record_lsn = Lsn::new(read_u64(bytes, 100)?);
        let image_lsn = Lsn::new(read_u64(bytes, 108)?);
        let image_crc32c = read_u32(bytes, 116)?;
        let base_lsn = Lsn::new(read_u64(bytes, 120)?);
        let base_crc32c = read_u32(bytes, 128)?;
        if page_id.get() < 2 || source.file_number == 0 {
            return Err(Error::corruption("delta-chain entry identity is invalid"));
        }
        match kind {
            1 => {
                if source.kind != SourceFileKind::CheckpointPack
                    || frame_length as usize != crate::page::PAGE_SIZE
                    || header_crc32c != 0
                    || payload_offset != offset
                    || payload_length as usize != crate::page::PAGE_SIZE
                    || payload_crc32c != image_crc32c
                    || source_length < offset.saturating_add(crate::page::PAGE_SIZE as u64)
                    || commit_frame_offset != 0
                    || commit_lsn != Lsn::ZERO
                    || record_lsn != Lsn::ZERO
                    || base_lsn != Lsn::ZERO
                    || base_crc32c != 0
                {
                    return Err(Error::corruption("checkpoint delta-chain entry is invalid"));
                }
                Ok(Self::CheckpointImage {
                    source,
                    offset,
                    source_length,
                    image_lsn,
                    image_crc32c,
                })
            }
            2 | 3 => {
                let frame_end = offset
                    .checked_add(u64::from(frame_length))
                    .ok_or_else(|| Error::corruption("delta-chain frame length overflows"))?;
                let payload_end = payload_offset
                    .checked_add(u64::from(payload_length))
                    .ok_or_else(|| Error::corruption("delta-chain payload length overflows"))?;
                if source.kind != SourceFileKind::WalSegment
                    || record_lsn == Lsn::ZERO
                    || commit_lsn == Lsn::ZERO
                    || commit_frame_offset < frame_end
                    || commit_frame_offset >= source_length
                    || frame_end > source_length
                    || payload_offset < offset
                    || payload_end > frame_end
                    || payload_end > source_length
                    || (kind == 2 && (base_lsn != Lsn::ZERO || base_crc32c != 0))
                    || (kind == 3 && base_lsn == Lsn::ZERO)
                {
                    return Err(Error::corruption("WAL delta-chain entry is invalid"));
                }
                let frame = WalFrameLocator {
                    source,
                    frame_offset: offset,
                    frame_length,
                    header_crc32c,
                    payload_offset,
                    payload_length,
                    payload_crc32c,
                    committed_prefix_length: source_length,
                    commit_frame_offset,
                    commit_lsn,
                };
                if kind == 2 {
                    Ok(Self::WalImage {
                        frame,
                        record_lsn,
                        image_lsn,
                        image_crc32c,
                    })
                } else {
                    Ok(Self::WalDelta {
                        frame,
                        record_lsn,
                        base_lsn,
                        base_crc32c,
                        image_lsn,
                        image_crc32c,
                    })
                }
            }
            _ => Err(Error::corruption("delta-chain entry tag is unknown")),
        }
    }
}

pub struct DeltaChain {
    pub page_id: PageId,
    pub entries: Vec<DeltaChainEntry>,
    _reservation: Option<Reservation>,
}

impl DeltaChain {
    pub fn new(page_id: PageId, entries: Vec<DeltaChainEntry>) -> Result<Self> {
        let chain = Self {
            page_id,
            entries,
            _reservation: None,
        };
        chain.validate()?;
        Ok(chain)
    }

    pub fn validate(&self) -> Result<()> {
        if self.page_id.get() < 2
            || self.entries.is_empty()
            || self.entries.len() > MAX_CHAIN_ENTRIES
        {
            return Err(Error::corruption(
                "delta-chain entry count or page id is invalid",
            ));
        }
        if !matches!(
            self.entries.first(),
            Some(DeltaChainEntry::CheckpointImage { .. } | DeltaChainEntry::WalImage { .. })
        ) {
            return Err(Error::corruption(
                "delta chain does not start at a full image",
            ));
        }
        let mut previous_version = None;
        let mut delta_count = 0usize;
        for entry in &self.entries {
            let source = entry.source_identity();
            if source.file_number == 0 {
                return Err(Error::corruption("delta-chain source number is zero"));
            }
            match entry {
                DeltaChainEntry::CheckpointImage {
                    source,
                    offset,
                    source_length,
                    ..
                } if source.kind != SourceFileKind::CheckpointPack
                    || *source_length < offset.saturating_add(crate::page::PAGE_SIZE as u64) =>
                {
                    return Err(Error::corruption(
                        "delta-chain checkpoint source range is invalid",
                    ));
                }
                DeltaChainEntry::WalImage {
                    frame, record_lsn, ..
                }
                | DeltaChainEntry::WalDelta {
                    frame, record_lsn, ..
                } if frame.source.kind != SourceFileKind::WalSegment
                    || *record_lsn == Lsn::ZERO
                    || frame.commit_lsn == Lsn::ZERO
                    || frame
                        .frame_offset
                        .checked_add(u64::from(frame.frame_length))
                        .is_none_or(|frame_end| {
                            frame_end > frame.committed_prefix_length
                                || frame.commit_frame_offset < frame_end
                                || frame.commit_frame_offset >= frame.committed_prefix_length
                                || frame.payload_offset < frame.frame_offset
                                || frame
                                    .payload_offset
                                    .checked_add(u64::from(frame.payload_length))
                                    .is_none_or(|payload_end| payload_end > frame_end)
                        }) =>
                {
                    return Err(Error::corruption("delta-chain WAL source range is invalid"));
                }
                _ => {}
            }
            if let Some((previous_lsn, previous_crc32c)) = previous_version {
                match entry {
                    DeltaChainEntry::WalDelta {
                        base_lsn,
                        base_crc32c,
                        ..
                    } if *base_lsn == previous_lsn && *base_crc32c == previous_crc32c => {
                        delta_count += 1;
                    }
                    _ => {
                        return Err(Error::corruption(
                            "delta-chain predecessor does not match the preceding image",
                        ));
                    }
                }
            } else if matches!(entry, DeltaChainEntry::WalDelta { .. }) {
                return Err(Error::corruption("delta chain starts with a delta"));
            }
            previous_version = Some((entry.image_lsn(), entry.image_crc32c()));
        }
        if delta_count > 8 {
            return Err(Error::corruption("delta chain exceeds eight updates"));
        }
        Ok(())
    }

    pub fn reconstruct<F>(&self, mut read_payload: F) -> Result<[u8; crate::page::PAGE_SIZE]>
    where
        F: FnMut(DeltaChainEntry) -> Result<Vec<u8>>,
    {
        self.validate()?;
        let mut reconstructed: Option<[u8; crate::page::PAGE_SIZE]> = None;
        for entry in self.entries.iter().copied() {
            let payload = read_payload(entry)?;
            let (image_lsn, image_crc32c) = match entry {
                DeltaChainEntry::CheckpointImage {
                    image_lsn,
                    image_crc32c,
                    ..
                } => {
                    if payload.len() != crate::page::PAGE_SIZE {
                        return Err(Error::corruption(
                            "checkpoint page image has an invalid length",
                        ));
                    }
                    reconstructed =
                        Some(payload.as_slice().try_into().map_err(|_| {
                            Error::corruption("checkpoint page image is truncated")
                        })?);
                    (image_lsn, image_crc32c)
                }
                DeltaChainEntry::WalImage {
                    record_lsn,
                    image_lsn,
                    image_crc32c,
                    ..
                } => {
                    if payload.len() != crate::wal::PAGE_IMAGE_PAYLOAD_SIZE
                        || u64::from_le_bytes(payload[..8].try_into().unwrap())
                            != self.page_id.get()
                    {
                        return Err(Error::corruption(
                            "WAL page image identity or length is invalid",
                        ));
                    }
                    if record_lsn == Lsn::ZERO {
                        return Err(Error::corruption("WAL page image record LSN is zero"));
                    }
                    reconstructed = Some(
                        payload[8..]
                            .try_into()
                            .map_err(|_| Error::corruption("WAL page image is truncated"))?,
                    );
                    (image_lsn, image_crc32c)
                }
                DeltaChainEntry::WalDelta {
                    record_lsn,
                    base_lsn,
                    base_crc32c,
                    image_lsn,
                    image_crc32c,
                    ..
                } => {
                    if record_lsn == Lsn::ZERO {
                        return Err(Error::corruption("WAL page delta record LSN is zero"));
                    }
                    let base = reconstructed.take().ok_or_else(|| {
                        Error::corruption("WAL page delta has no reconstructed base")
                    })?;
                    if crc32c::crc32c(&base) != base_crc32c {
                        return Err(Error::corruption(
                            "WAL page delta base checksum differs from its ancestry",
                        ));
                    }
                    let delta = crate::wal::decode_page_delta(&payload)?;
                    if delta.page_id != self.page_id || delta.base_page_lsn != base_lsn {
                        return Err(Error::corruption(
                            "WAL page delta identity or base LSN differs from its ancestry",
                        ));
                    }
                    reconstructed = Some(crate::wal::apply_page_delta(&base, &delta)?);
                    (image_lsn, image_crc32c)
                }
            };
            let image = reconstructed
                .as_ref()
                .ok_or_else(|| Error::invariant("delta-chain reconstruction produced no image"))?;
            if crc32c::crc32c(image) != image_crc32c {
                return Err(Error::corruption(
                    "reconstructed page checksum differs from its ancestry",
                ));
            }
            let decoded = crate::blink::decode_blink_page(image, self.page_id)?;
            if decoded.lsn() != image_lsn {
                return Err(Error::corruption(
                    "reconstructed page LSN differs from its ancestry",
                ));
            }
        }
        reconstructed.ok_or_else(|| Error::corruption("delta chain produced no page image"))
    }
}

struct ChainStoreInner {
    directory: PathBuf,
    identity: StoreIdentity,
    next_file_number: AtomicU64,
    resources: BlinkResourceManager,
    tenant_id: u64,
}

pub struct DeltaChainStore {
    inner: ChainStoreInner,
}

impl DeltaChainStore {
    pub fn open(
        directory: &Path,
        identity: StoreIdentity,
        resources: BlinkResourceManager,
        tenant_id: u64,
    ) -> Result<Self> {
        fs::create_dir_all(directory)?;
        let mut maximum_file_number = 0u64;
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            if let Some(file_number) = parse_chain_file_number(file_name)? {
                maximum_file_number = maximum_file_number.max(file_number);
            }
        }
        let next_file_number = maximum_file_number
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("delta-chain file number exhausted"))?
            .max(1);
        Ok(Self {
            inner: ChainStoreInner {
                directory: directory.to_path_buf(),
                identity,
                next_file_number: AtomicU64::new(next_file_number),
                resources,
                tenant_id,
            },
        })
    }

    pub fn write(&self, generation: u64, chain: &DeltaChain) -> Result<DeltaChainRef> {
        let reservation = self.reserve_chain_bytes()?;
        self.write_reserved(generation, chain, reservation)
    }

    #[cfg(test)]
    pub(crate) fn write_with_stage<F>(
        &self,
        generation: u64,
        chain: &DeltaChain,
        stage: F,
    ) -> Result<DeltaChainRef>
    where
        F: FnMut(&str) -> Result<()>,
    {
        let reservation = self.reserve_chain_bytes()?;
        self.write_reserved_with_stage(generation, chain, reservation, stage)
    }

    pub(crate) fn reserve_write_work(&self, chain_count: usize) -> Result<Vec<Reservation>> {
        let mut reservations = Vec::new();
        reservations
            .try_reserve(chain_count)
            .map_err(|_| Error::overloaded("delta-chain reservation list is exhausted"))?;
        for _chain_number in 0..chain_count {
            reservations.push(self.reserve_chain_bytes()?);
        }
        Ok(reservations)
    }

    pub(crate) fn write_reserved(
        &self,
        generation: u64,
        chain: &DeltaChain,
        output_charge: Reservation,
    ) -> Result<DeltaChainRef> {
        self.write_reserved_with_stage(generation, chain, output_charge, |_| Ok(()))
    }

    fn write_reserved_with_stage<F>(
        &self,
        generation: u64,
        chain: &DeltaChain,
        output_charge: Reservation,
        mut stage: F,
    ) -> Result<DeltaChainRef>
    where
        F: FnMut(&str) -> Result<()>,
    {
        chain.validate()?;
        if generation == 0 {
            return Err(Error::invalid_input("delta-chain generation is zero"));
        }
        if output_charge.category() != ResourceCategory::Metadata
            || output_charge.bytes() < MAX_CHAIN_FILE_BYTES as u64
        {
            return Err(Error::invariant(
                "delta-chain output reservation is insufficient",
            ));
        }
        loop {
            let file_number = self
                .inner
                .next_file_number
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    current.checked_add(1)
                })
                .map_err(|_| Error::invalid_input("delta-chain file number exhausted"))?;
            if file_number == 0 {
                return Err(Error::invariant("delta-chain allocated file number zero"));
            }
            let path = self.path(file_number);
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            };
            stage("after_delta_chain_create")?;
            let bytes = encode_chain(self.inner.identity, generation, chain, file_number)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            stage("after_delta_chain_file_sync")?;
            File::open(&self.inner.directory)?.sync_all()?;
            stage("after_delta_chain_directory_sync")?;
            let payload_checksum = bytes[bytes.len() - CHAIN_CHECKSUM_BYTES..]
                .try_into()
                .map_err(|_| Error::invariant("delta-chain payload checksum length mismatch"))?;
            drop(output_charge);
            return Ok(DeltaChainRef {
                file_number,
                offset: 0,
                checksum: payload_checksum,
            });
        }
    }

    pub fn read(&self, reference: DeltaChainRef, expected_page_id: PageId) -> Result<DeltaChain> {
        if reference.file_number == 0 || reference.offset != 0 {
            return Err(Error::corruption("delta-chain reference is invalid"));
        }
        let charge = self.reserve_chain_bytes()?;
        let path = self.path(reference.file_number);
        let mut file = File::open(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::corruption("reachable delta-chain sidecar is missing")
            } else {
                Error::from(error)
            }
        })?;
        let file_length = file.metadata()?.len();
        if file_length < (CHAIN_HEADER_BYTES + CHAIN_ENTRY_BYTES + CHAIN_CHECKSUM_BYTES) as u64
            || file_length > MAX_CHAIN_FILE_BYTES as u64
        {
            return Err(Error::corruption("delta-chain sidecar length is invalid"));
        }
        let mut bytes = vec![0; file_length as usize];
        file.read_exact(&mut bytes)?;
        let mut trailing_byte = [0; 1];
        if file.read(&mut trailing_byte)? != 0 {
            return Err(Error::corruption("delta-chain sidecar grew during read"));
        }
        let chain = decode_chain(
            self.inner.identity,
            reference.file_number,
            reference.checksum,
            expected_page_id,
            &bytes,
        )?;
        Ok(DeltaChain {
            page_id: chain.page_id,
            entries: chain.entries,
            _reservation: Some(charge),
        })
    }

    fn path(&self, file_number: u64) -> PathBuf {
        self.inner.directory.join(format!(
            "{CHAIN_FILE_PREFIX}{file_number:020}{CHAIN_FILE_SUFFIX}"
        ))
    }

    fn reserve_chain_bytes(&self) -> Result<Reservation> {
        self.inner
            .resources
            .reserve(
                self.inner.tenant_id,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                MAX_CHAIN_FILE_BYTES as u64,
            )
            .map_err(|_| Error::overloaded("delta-chain metadata budget is exhausted"))
    }
}

fn encode_chain(
    identity: StoreIdentity,
    generation: u64,
    chain: &DeltaChain,
    file_number: u64,
) -> Result<Vec<u8>> {
    chain.validate()?;
    let payload_length = chain
        .entries
        .len()
        .checked_mul(CHAIN_ENTRY_BYTES)
        .ok_or_else(|| Error::invalid_input("delta-chain payload length overflows"))?;
    let total_length = CHAIN_HEADER_BYTES
        .checked_add(payload_length)
        .and_then(|length| length.checked_add(CHAIN_CHECKSUM_BYTES))
        .ok_or_else(|| Error::invalid_input("delta-chain length overflows"))?;
    let mut bytes = vec![0; total_length];
    bytes[..8].copy_from_slice(&CHAIN_MAGIC);
    bytes[8..10].copy_from_slice(&CHAIN_FORMAT_VERSION.to_le_bytes());
    bytes[12..16].copy_from_slice(&(CHAIN_HEADER_BYTES as u32).to_le_bytes());
    bytes[16..32].copy_from_slice(&identity.database_uuid);
    bytes[32..40].copy_from_slice(&identity.wal_epoch.to_le_bytes());
    bytes[40..48].copy_from_slice(&file_number.to_le_bytes());
    bytes[48..56].copy_from_slice(&generation.to_le_bytes());
    bytes[56..64].copy_from_slice(&chain.page_id.get().to_le_bytes());
    bytes[64] = chain.entries.len() as u8;
    bytes[72..76].copy_from_slice(&(payload_length as u32).to_le_bytes());
    let header_checksum = *blake3::hash(&bytes[..80]).as_bytes();
    bytes[80..112].copy_from_slice(&header_checksum);
    for (entry_index, entry) in chain.entries.iter().copied().enumerate() {
        let start = CHAIN_HEADER_BYTES + entry_index * CHAIN_ENTRY_BYTES;
        entry.encode(chain.page_id, &mut bytes[start..start + CHAIN_ENTRY_BYTES])?;
    }
    let payload_checksum =
        *blake3::hash(&bytes[CHAIN_HEADER_BYTES..CHAIN_HEADER_BYTES + payload_length]).as_bytes();
    bytes[CHAIN_HEADER_BYTES + payload_length..].copy_from_slice(&payload_checksum);
    Ok(bytes)
}

fn decode_chain(
    expected_identity: StoreIdentity,
    expected_file_number: u64,
    expected_checksum: [u8; 32],
    expected_page_id: PageId,
    bytes: &[u8],
) -> Result<DeltaChain> {
    if bytes.len() < CHAIN_HEADER_BYTES + CHAIN_ENTRY_BYTES + CHAIN_CHECKSUM_BYTES
        || bytes.len() > MAX_CHAIN_FILE_BYTES
        || bytes[..8] != CHAIN_MAGIC
        || read_u16(bytes, 8)? != CHAIN_FORMAT_VERSION
        || read_u16(bytes, 10)? != 0
        || read_u32(bytes, 12)? as usize != CHAIN_HEADER_BYTES
        || bytes[65..72] != [0; 7]
        || bytes[76..80] != [0; 4]
        || bytes[112..128] != [0; 16]
    {
        return Err(Error::corruption("delta-chain header is invalid"));
    }
    if bytes[16..32] != expected_identity.database_uuid
        || read_u64(bytes, 32)? != expected_identity.wal_epoch
        || read_u64(bytes, 40)? != expected_file_number
        || read_u64(bytes, 48)? == 0
        || read_u64(bytes, 56)? != expected_page_id.get()
    {
        return Err(Error::corruption("delta-chain identity or page id differs"));
    }
    let entry_count = usize::from(bytes[64]);
    if !(1..=MAX_CHAIN_ENTRIES).contains(&entry_count) {
        return Err(Error::corruption("delta-chain entry count is invalid"));
    }
    let payload_length = entry_count
        .checked_mul(CHAIN_ENTRY_BYTES)
        .ok_or_else(|| Error::corruption("delta-chain payload length overflows"))?;
    if read_u32(bytes, 72)? as usize != payload_length
        || bytes.len() != CHAIN_HEADER_BYTES + payload_length + CHAIN_CHECKSUM_BYTES
        || bytes[80..112] != *blake3::hash(&bytes[..80]).as_bytes()
    {
        return Err(Error::corruption(
            "delta-chain header checksum or length differs",
        ));
    }
    let payload = &bytes[CHAIN_HEADER_BYTES..CHAIN_HEADER_BYTES + payload_length];
    let payload_checksum = *blake3::hash(payload).as_bytes();
    if payload_checksum != expected_checksum
        || bytes[CHAIN_HEADER_BYTES + payload_length..] != payload_checksum
    {
        return Err(Error::corruption("delta-chain payload checksum differs"));
    }
    let mut entries = Vec::with_capacity(entry_count);
    for entry_bytes in payload.chunks_exact(CHAIN_ENTRY_BYTES) {
        entries.push(DeltaChainEntry::decode(expected_page_id, entry_bytes)?);
    }
    let chain = DeltaChain {
        page_id: expected_page_id,
        entries,
        _reservation: None,
    };
    chain.validate()?;
    Ok(chain)
}

fn parse_chain_file_number(file_name: &str) -> Result<Option<u64>> {
    let Some(number_text) = file_name
        .strip_prefix(CHAIN_FILE_PREFIX)
        .and_then(|name| name.strip_suffix(CHAIN_FILE_SUFFIX))
    else {
        return Ok(None);
    };
    if number_text.len() != 20 || !number_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::corruption("delta-chain filename is malformed"));
    }
    let file_number = number_text
        .parse::<u64>()
        .map_err(|_| Error::corruption("delta-chain file number is invalid"))?;
    if file_number == 0 {
        return Err(Error::corruption("delta-chain file number is zero"));
    }
    Ok(Some(file_number))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or_else(|| Error::corruption("delta-chain integer is truncated"))?
            .try_into()
            .map_err(|_| Error::corruption("delta-chain integer has wrong length"))?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| Error::corruption("delta-chain integer is truncated"))?
            .try_into()
            .map_err(|_| Error::corruption("delta-chain integer has wrong length"))?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(|| Error::corruption("delta-chain integer is truncated"))?
            .try_into()
            .map_err(|_| Error::corruption("delta-chain integer has wrong length"))?,
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{DeltaChain, DeltaChainEntry, DeltaChainRef, DeltaChainStore};
    use crate::blink::resource::{BlinkResourceManager, ResourceLimits};
    use crate::blink::version_index::{
        SourceFileKind, SourceIdentity, StoreIdentity, WalFrameLocator,
    };
    use dodb_core::{Lsn, PageId};
    use std::collections::HashMap;

    fn resources() -> BlinkResourceManager {
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

    fn temporary_directory() -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "dodb-version-chain-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    fn source(kind: SourceFileKind, file_number: u64) -> SourceIdentity {
        SourceIdentity {
            kind,
            file_number,
            header_identity: [file_number as u8; 32],
        }
    }

    fn frame(file_number: u64, offset: u64, record_lsn: u64) -> WalFrameLocator {
        WalFrameLocator {
            source: source(SourceFileKind::WalSegment, file_number),
            frame_offset: offset,
            frame_length: 100,
            header_crc32c: 11,
            payload_offset: offset + 32,
            payload_length: 50,
            payload_crc32c: record_lsn as u32,
            committed_prefix_length: 4096,
            commit_frame_offset: offset + 100,
            commit_lsn: Lsn::new(record_lsn + 1),
        }
    }

    fn sample_chain() -> DeltaChain {
        DeltaChain::new(
            PageId::new(5),
            vec![
                DeltaChainEntry::CheckpointImage {
                    source: source(SourceFileKind::CheckpointPack, 2),
                    offset: 4096,
                    source_length: 8192,
                    image_lsn: Lsn::new(10),
                    image_crc32c: 20,
                },
                DeltaChainEntry::WalDelta {
                    frame: frame(3, 200, 11),
                    record_lsn: Lsn::new(11),
                    base_lsn: Lsn::new(10),
                    base_crc32c: 20,
                    image_lsn: Lsn::new(12),
                    image_crc32c: 21,
                },
                DeltaChainEntry::WalDelta {
                    frame: frame(4, 300, 13),
                    record_lsn: Lsn::new(13),
                    base_lsn: Lsn::new(12),
                    base_crc32c: 21,
                    image_lsn: Lsn::new(14),
                    image_crc32c: 22,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn immutable_chain_sidecar_reopens_exact_delta_ancestry() {
        let directory = temporary_directory();
        let identity = StoreIdentity {
            database_uuid: [1; 16],
            wal_epoch: 7,
        };
        let store = DeltaChainStore::open(&directory, identity, resources(), 1).unwrap();
        let chain = sample_chain();
        let reference = store.write(8, &chain).unwrap();
        let reopened = store.read(reference, PageId::new(5)).unwrap();
        assert_eq!(reopened.page_id, chain.page_id);
        assert_eq!(reopened.entries, chain.entries);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reconstructs_eight_deltas_from_exact_sources_after_reopen() {
        let page_id = PageId::new(5);
        let mut entries = Vec::new();
        let mut payloads = HashMap::new();
        let mut previous_image =
            crate::blink::test_blink_page_image(page_id, Lsn::new(10)).unwrap();
        let mut previous_lsn = Lsn::new(10);
        let mut previous_crc32c = crc32c::crc32c(&previous_image);
        let initial_source = source(SourceFileKind::WalSegment, 1);
        let initial_record_lsn = Lsn::new(11);
        let mut initial_payload = page_id.get().to_le_bytes().to_vec();
        initial_payload.extend_from_slice(&previous_image);
        entries.push(DeltaChainEntry::WalImage {
            frame: frame(1, 100, 11),
            record_lsn: initial_record_lsn,
            image_lsn: previous_lsn,
            image_crc32c: previous_crc32c,
        });
        payloads.insert(
            (initial_source.file_number, initial_record_lsn),
            initial_payload,
        );
        for delta_index in 0..8u64 {
            let image_lsn = Lsn::new(12 + delta_index * 2);
            let record_lsn = Lsn::new(11 + delta_index * 2);
            let image = crate::blink::test_blink_page_image(page_id, image_lsn).unwrap();
            let payload = crate::wal::encode_page_delta(page_id, &previous_image, &image).unwrap();
            let source_file_number = 2 + delta_index;
            let image_crc32c = crc32c::crc32c(&image);
            entries.push(DeltaChainEntry::WalDelta {
                frame: frame(source_file_number, 100, record_lsn.get()),
                record_lsn,
                base_lsn: previous_lsn,
                base_crc32c: previous_crc32c,
                image_lsn,
                image_crc32c,
            });
            payloads.insert((source_file_number, record_lsn), payload);
            previous_image = image;
            previous_lsn = image_lsn;
            previous_crc32c = image_crc32c;
        }
        let chain = DeltaChain::new(page_id, entries).unwrap();
        let reopened_image = chain
            .reconstruct(|entry| match entry {
                DeltaChainEntry::CheckpointImage { .. } => unreachable!(),
                DeltaChainEntry::WalImage {
                    frame, record_lsn, ..
                }
                | DeltaChainEntry::WalDelta {
                    frame, record_lsn, ..
                } => payloads
                    .get(&(frame.source.file_number, record_lsn))
                    .cloned()
                    .ok_or_else(|| dodb_core::Error::corruption("test source is missing")),
            })
            .unwrap();
        assert_eq!(reopened_image, previous_image);
    }

    #[test]
    fn chain_read_rejects_foreign_identity_corruption_and_truncation() {
        let directory = temporary_directory();
        let identity = StoreIdentity {
            database_uuid: [1; 16],
            wal_epoch: 7,
        };
        let store = DeltaChainStore::open(&directory, identity, resources(), 1).unwrap();
        let reference = store.write(8, &sample_chain()).unwrap();
        assert!(store.read(reference, PageId::new(6)).is_err());
        let path = directory.join("version-chain-00000000000000000001.bin");
        let original_bytes = fs::read(&path).unwrap();
        let mut wrong_identity_bytes = original_bytes.clone();
        wrong_identity_bytes[16] ^= 1;
        let header_checksum = *blake3::hash(&wrong_identity_bytes[..80]).as_bytes();
        wrong_identity_bytes[80..112].copy_from_slice(&header_checksum);
        fs::write(&path, &wrong_identity_bytes).unwrap();
        assert!(store.read(reference, PageId::new(5)).is_err());
        let mut corrupted_bytes = original_bytes.clone();
        corrupted_bytes[140] ^= 1;
        fs::write(&path, &corrupted_bytes).unwrap();
        assert!(store.read(reference, PageId::new(5)).is_err());
        fs::write(&path, &original_bytes[..original_bytes.len() - 1]).unwrap();
        assert!(store.read(reference, PageId::new(5)).is_err());
        fs::write(&path, &original_bytes).unwrap();
        let reopened_chain = store.read(reference, PageId::new(5)).unwrap();
        assert_eq!(reopened_chain.page_id, sample_chain().page_id);
        assert_eq!(reopened_chain.entries, sample_chain().entries);
        fs::remove_file(&path).unwrap();
        assert!(store.read(reference, PageId::new(5)).is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn chain_validation_rejects_base_mismatch_and_more_than_eight_deltas() {
        let mut entries = sample_chain().entries;
        if let DeltaChainEntry::WalDelta { base_lsn, .. } = &mut entries[2] {
            *base_lsn = Lsn::new(99);
        }
        assert!(DeltaChain::new(PageId::new(5), entries).is_err());
        let mut entries = vec![sample_chain().entries[0]; 1];
        for delta_index in 0..9 {
            let previous = entries.last().unwrap();
            let previous_lsn = match previous {
                DeltaChainEntry::CheckpointImage { image_lsn, .. }
                | DeltaChainEntry::WalImage { image_lsn, .. }
                | DeltaChainEntry::WalDelta { image_lsn, .. } => *image_lsn,
            };
            let previous_crc32c = match previous {
                DeltaChainEntry::CheckpointImage { image_crc32c, .. }
                | DeltaChainEntry::WalImage { image_crc32c, .. }
                | DeltaChainEntry::WalDelta { image_crc32c, .. } => *image_crc32c,
            };
            let record_lsn = 20 + delta_index * 2;
            entries.push(DeltaChainEntry::WalDelta {
                frame: frame(10 + delta_index as u64, 300, record_lsn),
                record_lsn: Lsn::new(record_lsn),
                base_lsn: previous_lsn,
                base_crc32c: previous_crc32c,
                image_lsn: Lsn::new(record_lsn + 1),
                image_crc32c: previous_crc32c + 1,
            });
        }
        assert!(DeltaChain::new(PageId::new(5), entries).is_err());
    }

    #[test]
    fn chain_reference_requires_zero_offset_and_nonzero_file_number() {
        assert_eq!(
            DeltaChainRef {
                file_number: 1,
                offset: 0,
                checksum: [4; 32],
            }
            .offset,
            0
        );
    }

    #[test]
    fn delta_chain_sidecar_sigkill_boundaries_leave_only_unpublished_files() {
        const CHILD_DIRECTORY: &str = "DODB_DELTA_CHAIN_SIGKILL_DIRECTORY";
        const CHILD_STAGE: &str = "DODB_DELTA_CHAIN_SIGKILL_STAGE";
        const CHILD_READY: &str = "DODB_DELTA_CHAIN_SIGKILL_READY";
        let identity = StoreIdentity {
            database_uuid: [1; 16],
            wal_epoch: 7,
        };
        if let Ok(directory) = std::env::var(CHILD_DIRECTORY) {
            let store =
                DeltaChainStore::open(std::path::Path::new(&directory), identity, resources(), 1)
                    .unwrap();
            let target_stage = std::env::var(CHILD_STAGE).unwrap();
            let ready_path = std::path::PathBuf::from(std::env::var(CHILD_READY).unwrap());
            let _ = store.write_with_stage(8, &sample_chain(), |stage_name| {
                if stage_name == target_stage {
                    fs::write(&ready_path, b"ready")?;
                    loop {
                        thread::sleep(Duration::from_secs(1));
                    }
                }
                Ok(())
            });
            panic!("child sidecar write passed the selected crash boundary");
        }

        for stage_name in [
            "after_delta_chain_create",
            "after_delta_chain_file_sync",
            "after_delta_chain_directory_sync",
        ] {
            let directory = temporary_directory();
            let ready_path = directory.join("ready");
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "blink::version_chain::tests::delta_chain_sidecar_sigkill_boundaries_leave_only_unpublished_files",
                    "--nocapture",
                ])
                .env(CHILD_DIRECTORY, &directory)
                .env(CHILD_STAGE, stage_name)
                .env(CHILD_READY, &ready_path)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ready_path.exists() && Instant::now() < deadline {
                if let Some(status) = child.try_wait().unwrap() {
                    panic!("SIGKILL child exited before {stage_name}: {status}");
                }
                thread::sleep(Duration::from_millis(1));
            }
            assert!(ready_path.exists(), "child did not reach {stage_name}");
            assert_eq!(
                unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
                0
            );
            let status = child.wait().unwrap();
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(status.signal(), Some(libc::SIGKILL));

            let reopened = DeltaChainStore::open(&directory, identity, resources(), 1).unwrap();
            let sidecar_path = directory.join("version-chain-00000000000000000001.bin");
            let sidecar_bytes = fs::read(&sidecar_path).unwrap();
            if stage_name == "after_delta_chain_create" {
                assert!(sidecar_bytes.is_empty());
                assert!(
                    reopened
                        .read(
                            DeltaChainRef {
                                file_number: 1,
                                offset: 0,
                                checksum: [0; 32],
                            },
                            PageId::new(5),
                        )
                        .is_err()
                );
            } else {
                let checksum_start = sidecar_bytes.len() - 32;
                let checksum = sidecar_bytes[checksum_start..].try_into().unwrap();
                let reference = DeltaChainRef {
                    file_number: 1,
                    offset: 0,
                    checksum,
                };
                assert_eq!(
                    reopened.read(reference, PageId::new(5)).unwrap().entries,
                    sample_chain().entries
                );
            }
            assert_eq!(
                fs::read_dir(&directory)
                    .unwrap()
                    .filter_map(std::result::Result::ok)
                    .filter(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .starts_with("version-chain-")
                    })
                    .count(),
                1
            );
            fs::remove_dir_all(directory).unwrap();
        }
    }
}
