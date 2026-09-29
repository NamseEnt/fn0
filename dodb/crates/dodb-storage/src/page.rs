use dodb_core::{Error, Lsn, PageId, Result};

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_HEADER_SIZE: usize = 32;
pub const PAGE_FORMAT_VERSION: u16 = 1;
pub const PAGE_MAGIC: [u8; 4] = *b"DBPG";

const CHECKSUM_OFFSET: usize = 28;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PageType {
    Internal = 1,
    Leaf = 2,
    Overflow = 3,
    Free = 4,
    Test = 255,
}

impl PageType {
    fn decode(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Internal),
            2 => Ok(Self::Leaf),
            3 => Ok(Self::Overflow),
            4 => Ok(Self::Free),
            255 => Ok(Self::Test),
            other => Err(Error::unsupported_format(format!(
                "unknown page type {other}"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageHeader {
    pub format_version: u16,
    pub page_type: PageType,
    pub page_id: PageId,
    pub page_lsn: Lsn,
    pub flags: u32,
    /// Stored checksum. Encoding replaces this value with the calculated one.
    pub checksum: u32,
}

impl PageHeader {
    pub fn new(page_type: PageType, page_id: PageId, page_lsn: Lsn) -> Self {
        Self {
            format_version: PAGE_FORMAT_VERSION,
            page_type,
            page_id,
            page_lsn,
            flags: 0,
            checksum: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedPage {
    pub header: PageHeader,
    pub body: Vec<u8>,
}

/// Encodes a full fixed-size page using explicit little-endian fields.
///
/// The checksum covers all 4096 bytes. The checksum field at bytes 28..32 is
/// zeroed while calculating CRC32C, then filled with the result.
pub fn encode_page(header: PageHeader, body: &[u8]) -> Result<[u8; PAGE_SIZE]> {
    validate_encoded_page_header(header)?;
    if body.len() > PAGE_SIZE - PAGE_HEADER_SIZE {
        return Err(Error::invalid_input(format!(
            "page body is {} bytes, maximum is {}",
            body.len(),
            PAGE_SIZE - PAGE_HEADER_SIZE
        )));
    }

    let mut page = [0u8; PAGE_SIZE];
    page[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + body.len()].copy_from_slice(body);
    finalize_encoded_page(header, &mut page)?;
    Ok(page)
}

pub(crate) fn finalize_encoded_page(header: PageHeader, page: &mut [u8; PAGE_SIZE]) -> Result<()> {
    validate_encoded_page_header(header)?;
    page[0..4].copy_from_slice(&PAGE_MAGIC);
    page[4..6].copy_from_slice(&header.format_version.to_le_bytes());
    page[6] = header.page_type as u8;
    page[7] = 0;
    page[8..16].copy_from_slice(&header.page_id.get().to_le_bytes());
    page[16..24].copy_from_slice(&header.page_lsn.get().to_le_bytes());
    page[24..28].copy_from_slice(&header.flags.to_le_bytes());
    page[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].fill(0);
    let checksum = crc32c::crc32c(page);
    page[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].copy_from_slice(&checksum.to_le_bytes());
    Ok(())
}

fn validate_encoded_page_header(header: PageHeader) -> Result<()> {
    if header.format_version != PAGE_FORMAT_VERSION {
        return Err(Error::unsupported_format(format!(
            "cannot encode page version {}",
            header.format_version
        )));
    }
    Ok(())
}

pub fn decode_page(bytes: &[u8]) -> Result<DecodedPage> {
    decode_page_at(bytes, None)
}

/// Decodes a page and optionally checks that the physical slot agrees with its
/// encoded page id.
pub fn decode_page_at(bytes: &[u8], expected_page_id: Option<PageId>) -> Result<DecodedPage> {
    if bytes.len() != PAGE_SIZE {
        return Err(Error::corruption(format!(
            "page has {} bytes, expected {PAGE_SIZE}",
            bytes.len()
        )));
    }
    if bytes[0..4] != PAGE_MAGIC {
        return Err(Error::corruption("page magic mismatch"));
    }

    let format_version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if format_version != PAGE_FORMAT_VERSION {
        return Err(Error::unsupported_format(format!(
            "page version {format_version}, supported {PAGE_FORMAT_VERSION}"
        )));
    }
    if bytes[7] != 0 {
        return Err(Error::corruption("page reserved byte is non-zero"));
    }

    let page_type = PageType::decode(bytes[6])?;
    let page_id = PageId::new(u64::from_le_bytes(bytes[8..16].try_into().unwrap()));
    if let Some(expected) = expected_page_id
        && page_id != expected
    {
        return Err(Error::corruption(format!(
            "page id mismatch: encoded {}, expected {}",
            page_id.get(),
            expected.get()
        )));
    }
    let page_lsn = Lsn::new(u64::from_le_bytes(bytes[16..24].try_into().unwrap()));
    let flags = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
    let stored_checksum = u32::from_le_bytes(
        bytes[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4]
            .try_into()
            .unwrap(),
    );
    let calculated_checksum = page_checksum(bytes);
    if stored_checksum != calculated_checksum {
        return Err(Error::corruption(format!(
            "page checksum mismatch: stored {stored_checksum:#010x}, calculated {calculated_checksum:#010x}"
        )));
    }

    Ok(DecodedPage {
        header: PageHeader {
            format_version,
            page_type,
            page_id,
            page_lsn,
            flags,
            checksum: stored_checksum,
        },
        body: bytes[PAGE_HEADER_SIZE..].to_vec(),
    })
}

fn page_checksum(page: &[u8]) -> u32 {
    debug_assert_eq!(page.len(), PAGE_SIZE);
    let mut checksum_input = [0u8; PAGE_SIZE];
    checksum_input.copy_from_slice(page);
    checksum_input[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].fill(0);
    crc32c::crc32c(&checksum_input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_page() -> [u8; PAGE_SIZE] {
        encode_page(
            PageHeader::new(PageType::Leaf, PageId::new(7), Lsn::new(42)),
            b"body",
        )
        .unwrap()
    }

    fn legacy_encode_page(header: PageHeader, body: &[u8]) -> [u8; PAGE_SIZE] {
        let mut page = [0u8; PAGE_SIZE];
        page[0..4].copy_from_slice(&PAGE_MAGIC);
        page[4..6].copy_from_slice(&header.format_version.to_le_bytes());
        page[6] = header.page_type as u8;
        page[8..16].copy_from_slice(&header.page_id.get().to_le_bytes());
        page[16..24].copy_from_slice(&header.page_lsn.get().to_le_bytes());
        page[24..28].copy_from_slice(&header.flags.to_le_bytes());
        page[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + body.len()].copy_from_slice(body);
        let mut checksum_input = page;
        checksum_input[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].fill(0);
        let checksum = crc32c::crc32c(&checksum_input);
        page[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].copy_from_slice(&checksum.to_le_bytes());
        page
    }

    #[test]
    fn generic_encode_page_matches_legacy_bytes() {
        let header = PageHeader {
            format_version: PAGE_FORMAT_VERSION,
            page_type: PageType::Internal,
            page_id: PageId::new(19),
            page_lsn: Lsn::new(0x1020_3040_5060_7080),
            flags: 0x1234_5678,
            checksum: 0xfeed_beef,
        };
        let body = (0..257)
            .map(|offset| (offset * 37) as u8)
            .collect::<Vec<_>>();
        assert_eq!(
            encode_page(header, &body).unwrap(),
            legacy_encode_page(header, &body)
        );
    }

    #[test]
    fn generic_encode_page_preserves_header_validation_order() {
        let mut header = PageHeader::new(PageType::Leaf, PageId::new(7), Lsn::new(42));
        header.format_version = PAGE_FORMAT_VERSION + 1;
        let oversized_body = vec![0; PAGE_SIZE];
        assert!(matches!(
            encode_page(header, &oversized_body),
            Err(Error::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn page_round_trip() {
        let encoded = test_page();
        let decoded = decode_page_at(&encoded, Some(PageId::new(7))).unwrap();
        assert_eq!(decoded.header.page_type, PageType::Leaf);
        assert_eq!(decoded.header.page_lsn, Lsn::new(42));
        assert_eq!(&decoded.body[..4], b"body");
        assert!(decoded.body[4..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn one_bit_corruption_is_detected() {
        let mut encoded = test_page();
        encoded[100] ^= 1;
        assert!(matches!(decode_page(&encoded), Err(Error::Corruption(_))));
    }

    #[test]
    fn malformed_headers_are_rejected() {
        let mut bad_magic = test_page();
        bad_magic[0] ^= 1;
        assert!(matches!(decode_page(&bad_magic), Err(Error::Corruption(_))));

        let mut bad_version = test_page();
        bad_version[4..6].copy_from_slice(&99u16.to_le_bytes());
        assert!(matches!(
            decode_page(&bad_version),
            Err(Error::UnsupportedFormat(_))
        ));

        let encoded = test_page();
        assert!(matches!(
            decode_page_at(&encoded, Some(PageId::new(8))),
            Err(Error::Corruption(_))
        ));
        assert!(matches!(
            decode_page(&encoded[..100]),
            Err(Error::Corruption(_))
        ));
    }

    #[test]
    fn random_garbage_does_not_panic() {
        for length in [0, 1, 31, 32, 100, PAGE_SIZE - 1, PAGE_SIZE, PAGE_SIZE + 1] {
            let bytes = vec![0xa5; length];
            let result = std::panic::catch_unwind(|| decode_page(&bytes));
            assert!(result.is_ok());
        }
    }
}
