use dodb_core::{Error, Lsn, PageId, Result, ShardEpoch, ShardId, TenantId};

use crate::page::PAGE_SIZE;

pub const SUPERBLOCK_FORMAT_VERSION: u16 = 1;
pub const SUPERBLOCK_MAGIC: [u8; 4] = *b"DSBK";

const ROOT_OFFSET: usize = 58;
const FREE_LIST_OFFSET: usize = 66;
const HIGH_WATER_OFFSET: usize = 74;
const CHECKPOINT_OFFSET: usize = 82;
const CHECKSUM_OFFSET: usize = 90;
const NULL_PAGE_ID: u64 = u64::MAX;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Superblock {
    pub format_version: u16,
    pub generation: u64,
    pub database_uuid: [u8; 16],
    pub tenant_id: TenantId,
    pub shard_id: ShardId,
    pub shard_epoch: ShardEpoch,
    pub page_size: u32,
    pub root_page_id: Option<PageId>,
    pub free_list_head: Option<PageId>,
    pub high_water_page_id: Option<PageId>,
    pub checkpoint_lsn: Lsn,
    pub checksum: u32,
}

impl Superblock {
    pub fn new(
        generation: u64,
        database_uuid: [u8; 16],
        tenant_id: TenantId,
        shard_id: ShardId,
        shard_epoch: ShardEpoch,
    ) -> Self {
        Self {
            format_version: SUPERBLOCK_FORMAT_VERSION,
            generation,
            database_uuid,
            tenant_id,
            shard_id,
            shard_epoch,
            page_size: PAGE_SIZE as u32,
            root_page_id: None,
            free_list_head: None,
            high_water_page_id: None,
            checkpoint_lsn: Lsn::ZERO,
            checksum: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuperblockSlot {
    A,
    B,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSuperblock {
    pub slot: SuperblockSlot,
    pub superblock: Superblock,
}

/// Superblock copies occupy full pages 0 and 1. Fields use an explicit
/// little-endian codec; `None` page references are encoded as `u64::MAX`.
///
/// Generation comparison intentionally uses ordinary integer ordering. The
/// Phase 0 contract assumes generation wraparound is operationally
/// impossible for a database lifetime.
pub fn encode_superblock(superblock: &Superblock) -> Result<[u8; PAGE_SIZE]> {
    if superblock.format_version != SUPERBLOCK_FORMAT_VERSION {
        return Err(Error::unsupported_format(format!(
            "cannot encode superblock version {}",
            superblock.format_version
        )));
    }
    if superblock.page_size != PAGE_SIZE as u32 {
        return Err(Error::invalid_input(format!(
            "superblock page size is {}, expected {PAGE_SIZE}",
            superblock.page_size
        )));
    }

    let mut page = [0u8; PAGE_SIZE];
    page[0..4].copy_from_slice(&SUPERBLOCK_MAGIC);
    page[4..6].copy_from_slice(&superblock.format_version.to_le_bytes());
    page[6..14].copy_from_slice(&superblock.generation.to_le_bytes());
    page[14..30].copy_from_slice(&superblock.database_uuid);
    page[30..38].copy_from_slice(&superblock.tenant_id.get().to_le_bytes());
    page[38..46].copy_from_slice(&superblock.shard_id.get().to_le_bytes());
    page[46..54].copy_from_slice(&superblock.shard_epoch.get().to_le_bytes());
    page[54..58].copy_from_slice(&superblock.page_size.to_le_bytes());
    page[ROOT_OFFSET..ROOT_OFFSET + 8]
        .copy_from_slice(&encode_page_id(superblock.root_page_id).to_le_bytes());
    page[FREE_LIST_OFFSET..FREE_LIST_OFFSET + 8]
        .copy_from_slice(&encode_page_id(superblock.free_list_head).to_le_bytes());
    page[HIGH_WATER_OFFSET..HIGH_WATER_OFFSET + 8]
        .copy_from_slice(&encode_page_id(superblock.high_water_page_id).to_le_bytes());
    page[CHECKPOINT_OFFSET..CHECKPOINT_OFFSET + 8]
        .copy_from_slice(&superblock.checkpoint_lsn.get().to_le_bytes());

    let checksum = superblock_checksum(&page);
    page[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].copy_from_slice(&checksum.to_le_bytes());
    Ok(page)
}

pub fn decode_superblock(bytes: &[u8]) -> Result<Superblock> {
    if bytes.len() != PAGE_SIZE {
        return Err(Error::corruption(format!(
            "superblock has {} bytes, expected {PAGE_SIZE}",
            bytes.len()
        )));
    }
    if bytes[0..4] != SUPERBLOCK_MAGIC {
        return Err(Error::corruption("superblock magic mismatch"));
    }
    let format_version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    if format_version != SUPERBLOCK_FORMAT_VERSION {
        return Err(Error::unsupported_format(format!(
            "superblock version {format_version}, supported {SUPERBLOCK_FORMAT_VERSION}"
        )));
    }

    let stored_checksum = u32::from_le_bytes(
        bytes[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4]
            .try_into()
            .unwrap(),
    );
    let calculated_checksum = superblock_checksum(bytes);
    if stored_checksum != calculated_checksum {
        return Err(Error::corruption(format!(
            "superblock checksum mismatch: stored {stored_checksum:#010x}, calculated {calculated_checksum:#010x}"
        )));
    }

    let page_size = u32::from_le_bytes(bytes[54..58].try_into().unwrap());
    if page_size != PAGE_SIZE as u32 {
        return Err(Error::unsupported_format(format!(
            "superblock page size {page_size}, supported {PAGE_SIZE}"
        )));
    }

    Ok(Superblock {
        format_version,
        generation: u64::from_le_bytes(bytes[6..14].try_into().unwrap()),
        database_uuid: bytes[14..30].try_into().unwrap(),
        tenant_id: TenantId::new(u64::from_le_bytes(bytes[30..38].try_into().unwrap())),
        shard_id: ShardId::new(u64::from_le_bytes(bytes[38..46].try_into().unwrap())),
        shard_epoch: ShardEpoch::new(u64::from_le_bytes(bytes[46..54].try_into().unwrap())),
        page_size,
        root_page_id: decode_page_id(u64::from_le_bytes(
            bytes[ROOT_OFFSET..ROOT_OFFSET + 8].try_into().unwrap(),
        )),
        free_list_head: decode_page_id(u64::from_le_bytes(
            bytes[FREE_LIST_OFFSET..FREE_LIST_OFFSET + 8]
                .try_into()
                .unwrap(),
        )),
        high_water_page_id: decode_page_id(u64::from_le_bytes(
            bytes[HIGH_WATER_OFFSET..HIGH_WATER_OFFSET + 8]
                .try_into()
                .unwrap(),
        )),
        checkpoint_lsn: Lsn::new(u64::from_le_bytes(
            bytes[CHECKPOINT_OFFSET..CHECKPOINT_OFFSET + 8]
                .try_into()
                .unwrap(),
        )),
        checksum: stored_checksum,
    })
}

/// Selects the valid copy with the highest generation. An unsupported format
/// is never silently treated as corruption or skipped.
pub fn choose_superblock(slot_a: &[u8], slot_b: &[u8]) -> Result<SelectedSuperblock> {
    let a = decode_superblock(slot_a);
    let b = decode_superblock(slot_b);

    if let Err(Error::UnsupportedFormat(message)) = &a {
        return Err(Error::UnsupportedFormat(message.clone()));
    }
    if let Err(Error::UnsupportedFormat(message)) = &b {
        return Err(Error::UnsupportedFormat(message.clone()));
    }

    match (a, b) {
        (Ok(superblock), Err(_)) => Ok(SelectedSuperblock {
            slot: SuperblockSlot::A,
            superblock,
        }),
        (Err(_), Ok(superblock)) => Ok(SelectedSuperblock {
            slot: SuperblockSlot::B,
            superblock,
        }),
        (Ok(a), Ok(b)) if a.generation >= b.generation => Ok(SelectedSuperblock {
            slot: SuperblockSlot::A,
            superblock: a,
        }),
        (Ok(_), Ok(b)) => Ok(SelectedSuperblock {
            slot: SuperblockSlot::B,
            superblock: b,
        }),
        (Err(_), Err(_)) => Err(Error::corruption("both superblock copies are invalid")),
    }
}

fn encode_page_id(page_id: Option<PageId>) -> u64 {
    page_id.map_or(NULL_PAGE_ID, PageId::get)
}

fn decode_page_id(value: u64) -> Option<PageId> {
    (value != NULL_PAGE_ID).then(|| PageId::new(value))
}

fn superblock_checksum(page: &[u8]) -> u32 {
    debug_assert_eq!(page.len(), PAGE_SIZE);
    let mut checksum_input = [0u8; PAGE_SIZE];
    checksum_input.copy_from_slice(page);
    checksum_input[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].fill(0);
    crc32c::crc32c(&checksum_input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(generation: u64) -> [u8; PAGE_SIZE] {
        let mut block = Superblock::new(
            generation,
            [0x11; 16],
            TenantId::new(3),
            ShardId::new(4),
            ShardEpoch::new(5),
        );
        block.root_page_id = Some(PageId::new(8));
        block.free_list_head = Some(PageId::new(9));
        block.high_water_page_id = Some(PageId::new(10));
        block.checkpoint_lsn = Lsn::new(11);
        encode_superblock(&block).unwrap()
    }

    #[test]
    fn round_trip_and_nullable_pages() {
        let encoded = block(7);
        let decoded = decode_superblock(&encoded).unwrap();
        assert_eq!(decoded.generation, 7);
        assert_eq!(decoded.root_page_id, Some(PageId::new(8)));
        assert_eq!(decoded.checkpoint_lsn, Lsn::new(11));

        let mut empty =
            Superblock::new(1, [0; 16], TenantId::ZERO, ShardId::ZERO, ShardEpoch::ZERO);
        empty.root_page_id = None;
        assert_eq!(
            decode_superblock(&encode_superblock(&empty).unwrap())
                .unwrap()
                .root_page_id,
            None
        );
    }

    #[test]
    fn selects_valid_highest_generation() {
        let a = block(1);
        let b = block(2);
        assert_eq!(choose_superblock(&a, &b).unwrap().slot, SuperblockSlot::B);
        assert_eq!(choose_superblock(&b, &a).unwrap().slot, SuperblockSlot::A);

        let mut invalid = block(3);
        invalid[123] ^= 1;
        assert_eq!(
            choose_superblock(&invalid, &b).unwrap().slot,
            SuperblockSlot::B
        );
        assert!(matches!(
            choose_superblock(&invalid, &invalid),
            Err(Error::Corruption(_))
        ));
    }

    #[test]
    fn unsupported_format_is_explicit() {
        let mut encoded = block(1);
        encoded[4..6].copy_from_slice(&99u16.to_le_bytes());
        assert!(matches!(
            decode_superblock(&encoded),
            Err(Error::UnsupportedFormat(_))
        ));
        assert!(matches!(
            choose_superblock(&encoded, &block(1)),
            Err(Error::UnsupportedFormat(_))
        ));
    }
}
