use dodb_core::{DocumentKey, Error, Lsn, PageId, Result, Revision};

use crate::page::{DecodedPage, PAGE_SIZE, PageHeader, PageType};

pub(crate) const BODY_SIZE: usize = PAGE_SIZE - crate::page::PAGE_HEADER_SIZE;
pub(crate) const NULL_PAGE_ID: u64 = u64::MAX;
pub(crate) const LAYOUT_VERSION: u16 = 1;
pub(crate) const INLINE_VALUE_LIMIT: usize = 512;
pub(crate) const MAX_VALUE_SIZE: usize = 64 * 1024 * 1024;
pub(crate) const OVERFLOW_DATA_OFFSET: usize = 32;
pub(crate) const OVERFLOW_DATA_SIZE: usize = BODY_SIZE - OVERFLOW_DATA_OFFSET;
pub(crate) const MAX_OVERFLOW_PAGES: usize = MAX_VALUE_SIZE.div_ceil(OVERFLOW_DATA_SIZE);

const LEAF_MAGIC: [u8; 4] = *b"LEAF";
const INTERNAL_MAGIC: [u8; 4] = *b"INTN";
const OVERFLOW_MAGIC: [u8; 4] = *b"OVFL";
const FREE_MAGIC: [u8; 4] = *b"FREE";
const LEAF_HEADER_SIZE: usize = 32;
const INTERNAL_HEADER_SIZE: usize = 32;
const SLOT_SIZE: usize = 8;
const LEAF_RECORD_HEADER_SIZE: usize = 32;
const INTERNAL_RECORD_HEADER_SIZE: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ValueRef {
    Inline(Vec<u8>),
    Overflow { head: PageId, length: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LeafEntry {
    pub(crate) key: Vec<u8>,
    pub(crate) revision: Revision,
    pub(crate) value: Option<ValueRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InternalEntry {
    pub(crate) key: Vec<u8>,
    pub(crate) right_child: PageId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PageData {
    Leaf {
        lsn: Lsn,
        next_leaf: Option<PageId>,
        entries: Vec<LeafEntry>,
    },
    Internal {
        lsn: Lsn,
        leftmost_child: PageId,
        entries: Vec<InternalEntry>,
    },
    Overflow {
        lsn: Lsn,
        next: Option<PageId>,
        total_length: u64,
        chunk: Vec<u8>,
    },
    Free {
        lsn: Lsn,
        next: Option<PageId>,
    },
}

impl PageData {
    pub(crate) fn page_type(&self) -> PageType {
        match self {
            Self::Leaf { .. } => PageType::Leaf,
            Self::Internal { .. } => PageType::Internal,
            Self::Overflow { .. } => PageType::Overflow,
            Self::Free { .. } => PageType::Free,
        }
    }

    pub(crate) fn lsn(&self) -> Lsn {
        match self {
            Self::Leaf { lsn, .. }
            | Self::Internal { lsn, .. }
            | Self::Overflow { lsn, .. }
            | Self::Free { lsn, .. } => *lsn,
        }
    }

    pub(crate) fn restamp(&mut self, provisional: Revision, committed: Lsn) {
        match self {
            Self::Leaf { lsn, entries, .. } => {
                *lsn = committed;
                for entry in entries {
                    if entry.revision == provisional {
                        entry.revision = Revision::from(committed);
                    }
                }
            }
            Self::Internal { lsn, .. } | Self::Overflow { lsn, .. } | Self::Free { lsn, .. } => {
                *lsn = committed
            }
        }
    }

    pub(crate) fn encode(&self, page_id: PageId) -> Result<[u8; PAGE_SIZE]> {
        let body = self.encode_body()?;
        crate::page::encode_page(
            PageHeader::new(self.page_type(), page_id, self.lsn()),
            &body,
        )
    }

    pub(crate) fn decode(page: DecodedPage) -> Result<Self> {
        if page.header.flags != 0 {
            return Err(Error::corruption(format!(
                "page {} has unsupported flags {:#x}",
                page.header.page_id.get(),
                page.header.flags
            )));
        }
        match page.header.page_type {
            PageType::Leaf => decode_leaf(page.header.page_lsn, &page.body),
            PageType::Internal => decode_internal(page.header.page_lsn, &page.body),
            PageType::Overflow => decode_overflow(page.header.page_lsn, &page.body),
            PageType::Free => decode_free(page.header.page_lsn, &page.body),
            PageType::Test => Err(Error::corruption("test pages are not valid database pages")),
        }
    }

    fn encode_body(&self) -> Result<Vec<u8>> {
        match self {
            Self::Leaf {
                next_leaf, entries, ..
            } => encode_leaf(*next_leaf, entries),
            Self::Internal {
                leftmost_child,
                entries,
                ..
            } => encode_internal(*leftmost_child, entries),
            Self::Overflow {
                next,
                total_length,
                chunk,
                ..
            } => encode_overflow(*next, *total_length, chunk),
            Self::Free { next, .. } => encode_free(*next),
        }
    }
}

pub(crate) fn leaf_fits(entries: &[LeafEntry]) -> bool {
    encode_leaf(None, entries).is_ok()
}

pub(crate) fn internal_fits(leftmost_child: PageId, entries: &[InternalEntry]) -> bool {
    encode_internal(leftmost_child, entries).is_ok()
}

fn encode_leaf(next_leaf: Option<PageId>, entries: &[LeafEntry]) -> Result<Vec<u8>> {
    let slot_bytes = LEAF_HEADER_SIZE
        .checked_add(
            entries
                .len()
                .checked_mul(SLOT_SIZE)
                .ok_or_else(|| Error::invalid_input("leaf slot array size overflow"))?,
        )
        .ok_or_else(|| Error::invalid_input("leaf slot array size overflow"))?;
    if entries.len() > u16::MAX as usize || slot_bytes > BODY_SIZE {
        return Err(Error::invalid_input("too many entries for one leaf page"));
    }
    ensure_sorted_leaf(entries)?;

    let mut body = vec![0; BODY_SIZE];
    body[0..4].copy_from_slice(&LEAF_MAGIC);
    body[4..6].copy_from_slice(&LAYOUT_VERSION.to_le_bytes());
    body[6..8].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    body[8..10].copy_from_slice(&(slot_bytes as u16).to_le_bytes());
    body[12..20].copy_from_slice(&encode_page_id(next_leaf).to_le_bytes());

    let mut upper = BODY_SIZE;
    let mut slots = Vec::with_capacity(entries.len());
    for entry in entries.iter().rev() {
        let record = encode_leaf_record(entry)?;
        upper = upper
            .checked_sub(record.len())
            .ok_or_else(|| Error::invalid_input("leaf records exceed page capacity"))?;
        if upper < slot_bytes {
            return Err(Error::invalid_input("leaf records exceed page capacity"));
        }
        body[upper..upper + record.len()].copy_from_slice(&record);
        slots.push((upper, record.len(), entry.key.len()));
    }
    slots.reverse();
    body[10..12].copy_from_slice(&(upper as u16).to_le_bytes());
    for (index, (offset, length, key_length)) in slots.into_iter().enumerate() {
        let slot = LEAF_HEADER_SIZE + index * SLOT_SIZE;
        body[slot..slot + 2].copy_from_slice(&(offset as u16).to_le_bytes());
        body[slot + 2..slot + 4].copy_from_slice(&(length as u16).to_le_bytes());
        body[slot + 4..slot + 6].copy_from_slice(&(key_length as u16).to_le_bytes());
    }
    Ok(body)
}

fn encode_leaf_record(entry: &LeafEntry) -> Result<Vec<u8>> {
    if entry.key.len() > u16::MAX as usize {
        return Err(Error::invalid_input("encoded document key is too large"));
    }
    DocumentKey::decode(&entry.key)
        .map_err(|error| Error::invalid_input(format!("document key is not canonical: {error}")))?;

    let (flags, value_length, aux, inline) = match &entry.value {
        None => (0u8, 0u64, NULL_PAGE_ID, &[][..]),
        Some(ValueRef::Inline(value)) => (1u8, value.len() as u64, 0, value.as_slice()),
        Some(ValueRef::Overflow { head, length }) => (2u8, *length, head.get(), &[][..]),
    };
    let record_length = LEAF_RECORD_HEADER_SIZE
        .checked_add(entry.key.len())
        .and_then(|length| length.checked_add(inline.len()))
        .ok_or_else(|| Error::invalid_input("leaf record size overflow"))?;
    let mut record = vec![0; record_length];
    record[0..8].copy_from_slice(&entry.revision.get().to_le_bytes());
    record[8..16].copy_from_slice(&value_length.to_le_bytes());
    record[16..24].copy_from_slice(&aux.to_le_bytes());
    record[24] = flags;
    record[26..28].copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
    record[LEAF_RECORD_HEADER_SIZE..LEAF_RECORD_HEADER_SIZE + entry.key.len()]
        .copy_from_slice(&entry.key);
    record[LEAF_RECORD_HEADER_SIZE + entry.key.len()..].copy_from_slice(inline);
    Ok(record)
}

fn decode_leaf(lsn: Lsn, body: &[u8]) -> Result<PageData> {
    check_body_size(body)?;
    if body[0..4] != LEAF_MAGIC {
        return Err(Error::corruption("leaf layout magic mismatch"));
    }
    check_version(&body[4..6])?;
    let count = read_u16(body, 6)? as usize;
    let slot_end = read_u16(body, 8)? as usize;
    let records_start = read_u16(body, 10)? as usize;
    let next_leaf = decode_page_id(read_u64(body, 12)?);
    if body[20..32].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption("leaf reserved bytes are non-zero"));
    }
    validate_slot_region(count, slot_end, records_start, LEAF_HEADER_SIZE, body.len())?;

    let mut entries = Vec::with_capacity(count);
    let mut ranges = Vec::with_capacity(count);
    for index in 0..count {
        let slot = LEAF_HEADER_SIZE + index * SLOT_SIZE;
        let offset = read_u16(body, slot)? as usize;
        let length = read_u16(body, slot + 2)? as usize;
        let key_length = read_u16(body, slot + 4)? as usize;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| Error::corruption("leaf record range overflows"))?;
        if offset < records_start || end > body.len() || length < LEAF_RECORD_HEADER_SIZE {
            return Err(Error::corruption("leaf record range is outside free space"));
        }
        ranges.push((offset, end));
        entries.push(decode_leaf_record(&body[offset..end], key_length)?);
    }
    ensure_non_overlapping(&mut ranges, "leaf records")?;
    ensure_sorted_leaf(&entries)?;
    Ok(PageData::Leaf {
        lsn,
        next_leaf,
        entries,
    })
}

fn decode_leaf_record(bytes: &[u8], slot_key_length: usize) -> Result<LeafEntry> {
    let revision = Revision::new(read_u64(bytes, 0)?);
    let value_length = read_u64(bytes, 8)?;
    if value_length > MAX_VALUE_SIZE as u64 {
        return Err(Error::corruption("leaf value exceeds supported maximum"));
    }
    let aux = read_u64(bytes, 16)?;
    let flags = bytes[24];
    if bytes[25] != 0 || bytes[28..32].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption("leaf record reserved bytes are non-zero"));
    }
    let key_length = read_u16(bytes, 26)? as usize;
    if key_length != slot_key_length {
        return Err(Error::corruption(
            "leaf slot key length does not match record",
        ));
    }
    let key_end = LEAF_RECORD_HEADER_SIZE
        .checked_add(key_length)
        .ok_or_else(|| Error::corruption("leaf key length overflows"))?;
    if key_end > bytes.len() {
        return Err(Error::corruption("leaf key exceeds record"));
    }
    let key = bytes[LEAF_RECORD_HEADER_SIZE..key_end].to_vec();
    DocumentKey::decode(&key)
        .map_err(|error| Error::corruption(format!("leaf key is not canonical: {error}")))?;

    let value = match flags {
        0 => {
            if value_length != 0 || aux != NULL_PAGE_ID || bytes.len() != key_end {
                return Err(Error::corruption("invalid tombstone leaf record"));
            }
            None
        }
        1 => {
            let value_length = usize::try_from(value_length)
                .map_err(|_| Error::corruption("inline value length does not fit usize"))?;
            let value_end = key_end
                .checked_add(value_length)
                .ok_or_else(|| Error::corruption("inline value length overflows"))?;
            if value_end != bytes.len() || aux != 0 {
                return Err(Error::corruption("invalid inline leaf record"));
            }
            Some(ValueRef::Inline(bytes[key_end..value_end].to_vec()))
        }
        2 => {
            if bytes.len() != key_end || aux == NULL_PAGE_ID || value_length == 0 {
                return Err(Error::corruption("invalid overflow leaf record"));
            }
            Some(ValueRef::Overflow {
                head: PageId::new(aux),
                length: value_length,
            })
        }
        other => {
            return Err(Error::corruption(format!(
                "unknown leaf value state {other}"
            )));
        }
    };
    Ok(LeafEntry {
        key,
        revision,
        value,
    })
}

fn encode_internal(leftmost_child: PageId, entries: &[InternalEntry]) -> Result<Vec<u8>> {
    if leftmost_child.get() < 2 {
        return Err(Error::invalid_input("internal leftmost child is invalid"));
    }
    let slot_bytes = INTERNAL_HEADER_SIZE
        .checked_add(
            entries
                .len()
                .checked_mul(SLOT_SIZE)
                .ok_or_else(|| Error::invalid_input("internal slot array size overflow"))?,
        )
        .ok_or_else(|| Error::invalid_input("internal slot array size overflow"))?;
    if entries.len() > u16::MAX as usize || slot_bytes > BODY_SIZE {
        return Err(Error::invalid_input(
            "too many entries for one internal page",
        ));
    }
    ensure_sorted_internal(entries)?;
    let mut body = vec![0; BODY_SIZE];
    body[0..4].copy_from_slice(&INTERNAL_MAGIC);
    body[4..6].copy_from_slice(&LAYOUT_VERSION.to_le_bytes());
    body[6..8].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    body[8..10].copy_from_slice(&(slot_bytes as u16).to_le_bytes());
    body[12..20].copy_from_slice(&leftmost_child.get().to_le_bytes());

    let mut upper = BODY_SIZE;
    let mut slots = Vec::with_capacity(entries.len());
    for entry in entries.iter().rev() {
        if entry.key.len() > u16::MAX as usize || entry.right_child.get() < 2 {
            return Err(Error::invalid_input("invalid internal entry"));
        }
        DocumentKey::decode(&entry.key).map_err(|error| {
            Error::invalid_input(format!("internal separator is not canonical: {error}"))
        })?;
        let record_length = INTERNAL_RECORD_HEADER_SIZE
            .checked_add(entry.key.len())
            .ok_or_else(|| Error::invalid_input("internal record size overflow"))?;
        let mut record = vec![0; record_length];
        record[0..2].copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
        record[4..12].copy_from_slice(&entry.right_child.get().to_le_bytes());
        record[INTERNAL_RECORD_HEADER_SIZE..].copy_from_slice(&entry.key);
        upper = upper
            .checked_sub(record.len())
            .ok_or_else(|| Error::invalid_input("internal records exceed page capacity"))?;
        if upper < slot_bytes {
            return Err(Error::invalid_input(
                "internal records exceed page capacity",
            ));
        }
        body[upper..upper + record.len()].copy_from_slice(&record);
        slots.push((upper, record.len(), entry.key.len()));
    }
    slots.reverse();
    body[10..12].copy_from_slice(&(upper as u16).to_le_bytes());
    for (index, (offset, length, key_length)) in slots.into_iter().enumerate() {
        let slot = INTERNAL_HEADER_SIZE + index * SLOT_SIZE;
        body[slot..slot + 2].copy_from_slice(&(offset as u16).to_le_bytes());
        body[slot + 2..slot + 4].copy_from_slice(&(length as u16).to_le_bytes());
        body[slot + 4..slot + 6].copy_from_slice(&(key_length as u16).to_le_bytes());
    }
    Ok(body)
}

fn decode_internal(lsn: Lsn, body: &[u8]) -> Result<PageData> {
    check_body_size(body)?;
    if body[0..4] != INTERNAL_MAGIC {
        return Err(Error::corruption("internal layout magic mismatch"));
    }
    check_version(&body[4..6])?;
    let count = read_u16(body, 6)? as usize;
    let slot_end = read_u16(body, 8)? as usize;
    let records_start = read_u16(body, 10)? as usize;
    let leftmost_child = PageId::new(read_u64(body, 12)?);
    if leftmost_child.get() < 2 || body[20..32].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption("invalid internal header"));
    }
    validate_slot_region(
        count,
        slot_end,
        records_start,
        INTERNAL_HEADER_SIZE,
        body.len(),
    )?;

    let mut entries = Vec::with_capacity(count);
    let mut ranges = Vec::with_capacity(count);
    for index in 0..count {
        let slot = INTERNAL_HEADER_SIZE + index * SLOT_SIZE;
        let offset = read_u16(body, slot)? as usize;
        let length = read_u16(body, slot + 2)? as usize;
        let key_length = read_u16(body, slot + 4)? as usize;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| Error::corruption("internal record range overflows"))?;
        if offset < records_start || end > body.len() || length < INTERNAL_RECORD_HEADER_SIZE {
            return Err(Error::corruption(
                "internal record range is outside free space",
            ));
        }
        ranges.push((offset, end));
        let record = &body[offset..end];
        let record_key_length = read_u16(record, 0)? as usize;
        if record[2..4].iter().any(|byte| *byte != 0) || record_key_length != key_length {
            return Err(Error::corruption("internal record key length is invalid"));
        }
        if record.len() != INTERNAL_RECORD_HEADER_SIZE + key_length {
            return Err(Error::corruption("internal record length is invalid"));
        }
        let right_child = PageId::new(read_u64(record, 4)?);
        if right_child.get() < 2 {
            return Err(Error::corruption("internal child page id is invalid"));
        }
        let key = record[INTERNAL_RECORD_HEADER_SIZE..].to_vec();
        DocumentKey::decode(&key).map_err(|error| {
            Error::corruption(format!("internal separator is not canonical: {error}"))
        })?;
        entries.push(InternalEntry { key, right_child });
    }
    ensure_non_overlapping(&mut ranges, "internal records")?;
    ensure_sorted_internal(&entries)?;
    Ok(PageData::Internal {
        lsn,
        leftmost_child,
        entries,
    })
}

fn encode_overflow(next: Option<PageId>, total_length: u64, chunk: &[u8]) -> Result<Vec<u8>> {
    if total_length == 0 || total_length > MAX_VALUE_SIZE as u64 {
        return Err(Error::invalid_input(
            "overflow value length is outside limits",
        ));
    }
    if chunk.len() > OVERFLOW_DATA_SIZE || chunk.len() > u16::MAX as usize || chunk.is_empty() {
        return Err(Error::invalid_input("overflow chunk length is invalid"));
    }
    let mut body = vec![0; BODY_SIZE];
    body[0..4].copy_from_slice(&OVERFLOW_MAGIC);
    body[4..6].copy_from_slice(&LAYOUT_VERSION.to_le_bytes());
    body[8..16].copy_from_slice(&encode_page_id(next).to_le_bytes());
    body[16..24].copy_from_slice(&total_length.to_le_bytes());
    body[24..26].copy_from_slice(&(chunk.len() as u16).to_le_bytes());
    body[OVERFLOW_DATA_OFFSET..OVERFLOW_DATA_OFFSET + chunk.len()].copy_from_slice(chunk);
    Ok(body)
}

fn decode_overflow(lsn: Lsn, body: &[u8]) -> Result<PageData> {
    check_body_size(body)?;
    if body[0..4] != OVERFLOW_MAGIC {
        return Err(Error::corruption("overflow layout magic mismatch"));
    }
    check_version(&body[4..6])?;
    if body[6..8].iter().any(|byte| *byte != 0) || body[26..32].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption("overflow reserved bytes are non-zero"));
    }
    let next = decode_page_id(read_u64(body, 8)?);
    let total_length = read_u64(body, 16)?;
    let chunk_length = read_u16(body, 24)? as usize;
    if total_length == 0 || total_length > MAX_VALUE_SIZE as u64 || chunk_length == 0 {
        return Err(Error::corruption("overflow length is invalid"));
    }
    if OVERFLOW_DATA_OFFSET + chunk_length > body.len() {
        return Err(Error::corruption("overflow chunk exceeds page"));
    }
    Ok(PageData::Overflow {
        lsn,
        next,
        total_length,
        chunk: body[OVERFLOW_DATA_OFFSET..OVERFLOW_DATA_OFFSET + chunk_length].to_vec(),
    })
}

fn encode_free(next: Option<PageId>) -> Result<Vec<u8>> {
    let mut body = vec![0; BODY_SIZE];
    body[0..4].copy_from_slice(&FREE_MAGIC);
    body[4..6].copy_from_slice(&LAYOUT_VERSION.to_le_bytes());
    body[8..16].copy_from_slice(&encode_page_id(next).to_le_bytes());
    Ok(body)
}

fn decode_free(lsn: Lsn, body: &[u8]) -> Result<PageData> {
    check_body_size(body)?;
    if body[0..4] != FREE_MAGIC {
        return Err(Error::corruption("free-page layout magic mismatch"));
    }
    check_version(&body[4..6])?;
    if body[6..8].iter().any(|byte| *byte != 0) || body[16..].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption("free-page reserved bytes are non-zero"));
    }
    Ok(PageData::Free {
        lsn,
        next: decode_page_id(read_u64(body, 8)?),
    })
}

fn check_body_size(body: &[u8]) -> Result<()> {
    if body.len() != BODY_SIZE {
        return Err(Error::corruption(format!(
            "page body has {} bytes, expected {BODY_SIZE}",
            body.len()
        )));
    }
    Ok(())
}

fn check_version(bytes: &[u8]) -> Result<()> {
    let version = read_u16(bytes, 0)?;
    if version != LAYOUT_VERSION {
        return Err(Error::unsupported_format(format!(
            "page layout version {version}, supported {LAYOUT_VERSION}"
        )));
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(read_array(bytes, offset)?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(read_array(bytes, offset)?))
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    let end = offset
        .checked_add(N)
        .ok_or_else(|| Error::corruption("fixed-field offset overflows"))?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| Error::corruption("fixed field is truncated"))?;
    slice
        .try_into()
        .map_err(|_| Error::corruption("fixed field has an invalid length"))
}

fn validate_slot_region(
    count: usize,
    slot_end: usize,
    records_start: usize,
    header_size: usize,
    body_len: usize,
) -> Result<()> {
    let expected_slot_end = header_size
        .checked_add(
            count
                .checked_mul(SLOT_SIZE)
                .ok_or_else(|| Error::corruption("slot count overflows"))?,
        )
        .ok_or_else(|| Error::corruption("slot count overflows"))?;
    if slot_end != expected_slot_end
        || records_start < slot_end
        || records_start > body_len
        || slot_end > body_len
    {
        return Err(Error::corruption("invalid slotted-page boundaries"));
    }
    Ok(())
}

fn ensure_non_overlapping(ranges: &mut [(usize, usize)], kind: &str) -> Result<()> {
    ranges.sort_unstable();
    for pair in ranges.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(Error::corruption(format!("overlapping {kind}")));
        }
    }
    Ok(())
}

fn ensure_sorted_leaf(entries: &[LeafEntry]) -> Result<()> {
    for pair in entries.windows(2) {
        if pair[0].key >= pair[1].key {
            return Err(Error::corruption("leaf keys are not strictly ordered"));
        }
    }
    Ok(())
}

fn ensure_sorted_internal(entries: &[InternalEntry]) -> Result<()> {
    for pair in entries.windows(2) {
        if pair[0].key >= pair[1].key {
            return Err(Error::corruption(
                "internal separators are not strictly ordered",
            ));
        }
    }
    Ok(())
}

fn encode_page_id(page_id: Option<PageId>) -> u64 {
    page_id.map_or(NULL_PAGE_ID, PageId::get)
}

fn decode_page_id(value: u64) -> Option<PageId> {
    (value != NULL_PAGE_ID).then(|| PageId::new(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_leaf() -> PageData {
        PageData::Leaf {
            lsn: Lsn::new(1),
            next_leaf: None,
            entries: vec![LeafEntry {
                key: DocumentKey::new(vec![1], vec![2]).encode(),
                revision: Revision::new(1),
                value: Some(ValueRef::Inline(b"value".to_vec())),
            }],
        }
    }

    #[test]
    fn malformed_slotted_offsets_return_corruption() {
        let page = sample_leaf().encode(PageId::new(2)).unwrap();
        let decoded = crate::decode_page_at(&page, Some(PageId::new(2))).unwrap();
        let mut body = decoded.body;
        let slot_offset = LEAF_HEADER_SIZE;
        body[slot_offset..slot_offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let result = PageData::decode(DecodedPage {
            header: decoded.header,
            body,
        });
        assert!(matches!(result, Err(Error::Corruption(_))));
    }

    #[test]
    fn random_leaf_body_is_rejected_without_panic() {
        let result = PageData::decode(DecodedPage {
            header: PageHeader::new(PageType::Leaf, PageId::new(2), Lsn::ZERO),
            body: vec![0xa5; BODY_SIZE],
        });
        assert!(result.is_err());
    }
}
