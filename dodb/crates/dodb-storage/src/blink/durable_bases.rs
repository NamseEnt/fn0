use super::*;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAGIC: [u8; 8] = *b"DBLKBASE";
const HEADER_SIZE: usize = PAGE_SIZE;
const RECORD_SIZE: usize = PAGE_SIZE;
pub(super) const MAX_PACKS: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasePackEntry {
    pub number: u64,
    pub checkpoint_lsn: Lsn,
    pub page_count: u64,
    pub bytes: u64,
    pub checksum: u32,
}

pub(super) struct BasePackJob {
    pub directory: PathBuf,
    pub identity: WalIdentity,
    pub number: u64,
    pub checkpoint_lsn: Lsn,
    pub complete: bool,
    pub coalesce: bool,
}

pub(super) struct BasePackOutput {
    pub entry: BasePackEntry,
    pub page_ids: BTreeSet<PageId>,
    pub complete: bool,
    pub write_calls: u64,
    pub phase_timings: [u64; 3],
}

pub(super) fn encoded_size(page_count: u64) -> u64 {
    page_count
        .saturating_mul(RECORD_SIZE as u64)
        .saturating_add(HEADER_SIZE as u64 + 4)
}

pub(super) fn pack_path(directory: &Path, number: u64) -> PathBuf {
    directory.join(format!("base-{number:020}.pages"))
}

fn hit(injector: &mut Option<Box<dyn FaultInjector + Send>>, point: &str) -> Result<()> {
    if let Some(injector) = injector.as_deref_mut() {
        injector.hit(point)?;
    }
    Ok(())
}

impl BasePackJob {
    pub fn run(
        self,
        pages: &BTreeMap<PageId, Arc<[u8; PAGE_SIZE]>>,
        injector: &mut Option<Box<dyn FaultInjector + Send>>,
    ) -> Result<BasePackOutput> {
        let page_started = Instant::now();
        let mut header = Vec::with_capacity(HEADER_SIZE);
        header.extend_from_slice(&MAGIC);
        header.extend_from_slice(&1u32.to_le_bytes());
        header.extend_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
        header.extend_from_slice(&self.identity.database_uuid);
        header.extend_from_slice(&self.identity.tenant_id.get().to_le_bytes());
        header.extend_from_slice(&self.identity.shard_id.get().to_le_bytes());
        header.extend_from_slice(&self.identity.shard_epoch.get().to_le_bytes());
        header.extend_from_slice(&self.number.to_le_bytes());
        header.extend_from_slice(&self.checkpoint_lsn.get().to_le_bytes());
        header.extend_from_slice(&(pages.len() as u64).to_le_bytes());
        header.resize(HEADER_SIZE - 4, 0);
        let header_checksum = crc32c::crc32c(&header);
        header.extend_from_slice(&header_checksum.to_le_bytes());
        hit(injector, "before_base_pack_create")?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(pack_path(&self.directory, self.number))?;
        hit(injector, "before_checkpoint_data_flush")?;
        file.write_all(&header)?;
        let mut checksum = crc32c::crc32c(&header);
        let mut write_calls = 1;
        let capacity = RECORD_SIZE * if self.coalesce { 64 } else { 1 };
        let mut buffer = Vec::with_capacity(capacity);
        for image in pages.values() {
            hit(injector, "during_checkpoint_page_write")?;
            hit(injector, "during_base_pack_page_write")?;
            buffer.extend_from_slice(&image[..]);
            if buffer.len() == capacity {
                checksum = crc32c::crc32c_append(checksum, &buffer);
                file.write_all(&buffer)?;
                buffer.clear();
                write_calls += 1;
                hit(injector, "after_checkpoint_page_write")?;
            }
        }
        if !buffer.is_empty() {
            checksum = crc32c::crc32c_append(checksum, &buffer);
            file.write_all(&buffer)?;
            write_calls += 1;
            hit(injector, "after_checkpoint_page_write")?;
        }
        file.write_all(&checksum.to_le_bytes())?;
        write_calls += 1;
        let page_nanos = elapsed_nanos(page_started);
        hit(injector, "before_checkpoint_data_sync")?;
        hit(injector, "before_base_pack_sync")?;
        let sync_started = Instant::now();
        file.sync_all()?;
        let sync_nanos = elapsed_nanos(sync_started);
        hit(injector, "after_base_pack_sync")?;
        hit(injector, "after_checkpoint_data_sync")?;
        let directory_started = Instant::now();
        hit(injector, "before_base_pack_directory_sync")?;
        fs::File::open(&self.directory)?.sync_all()?;
        hit(injector, "after_base_pack_directory_sync")?;
        Ok(BasePackOutput {
            entry: BasePackEntry {
                number: self.number,
                checkpoint_lsn: self.checkpoint_lsn,
                page_count: pages.len() as u64,
                bytes: encoded_size(pages.len() as u64),
                checksum,
            },
            page_ids: pages.keys().copied().collect(),
            complete: self.complete,
            write_calls,
            phase_timings: [page_nanos, sync_nanos, elapsed_nanos(directory_started)],
        })
    }
}

pub(super) fn read_pack(
    directory: &Path,
    entry: &BasePackEntry,
    identity: &WalIdentity,
    high_water_page_id: PageId,
    pages: &mut BTreeMap<PageId, RecoveredWalPage>,
    owners: &mut BTreeMap<PageId, u64>,
) -> Result<()> {
    let mut file = fs::File::open(pack_path(directory, entry.number))?;
    if entry.bytes != encoded_size(entry.page_count) || file.metadata()?.len() != entry.bytes {
        return Err(Error::corruption("page-base file length mismatch"));
    }
    let mut header = [0u8; HEADER_SIZE];
    file.read_exact(&mut header)?;
    let word = |offset| u64::from_le_bytes(header[offset..offset + 8].try_into().unwrap());
    if header[..8] != MAGIC
        || header[8..12] != 1u32.to_le_bytes()
        || header[12..16] != (PAGE_SIZE as u32).to_le_bytes()
        || header[16..32] != identity.database_uuid
        || word(32) != identity.tenant_id.get()
        || word(40) != identity.shard_id.get()
        || word(48) != identity.shard_epoch.get()
        || word(56) != entry.number
        || word(64) != entry.checkpoint_lsn.get()
        || word(72) != entry.page_count
        || header[80..HEADER_SIZE - 4].iter().any(|byte| *byte != 0)
        || crc32c::crc32c(&header[..HEADER_SIZE - 4])
            != u32::from_le_bytes(header[HEADER_SIZE - 4..].try_into().unwrap())
    {
        return Err(Error::corruption("page-base identity or header mismatch"));
    }
    let mut checksum = crc32c::crc32c(&header);
    let mut previous_id = None;
    let mut record = [0u8; RECORD_SIZE];
    for _page_index in 0..entry.page_count {
        file.read_exact(&mut record)?;
        checksum = crc32c::crc32c_append(checksum, &record);
        let page_id = PageId::new(u64::from_le_bytes(record[8..16].try_into().unwrap()));
        if page_id.get() < FIRST_DATA_PAGE
            || page_id > high_water_page_id
            || previous_id.is_some_and(|previous| page_id <= previous)
        {
            return Err(Error::corruption(
                "page-base page range or ordering mismatch",
            ));
        }
        let image = record;
        let decoded = decode_blink_page(&image, page_id)?;
        if decoded.lsn() > entry.checkpoint_lsn {
            return Err(Error::corruption(
                "page-base image exceeds its checkpoint target",
            ));
        }
        pages.insert(
            page_id,
            RecoveredWalPage {
                commit_lsn: entry.checkpoint_lsn,
                image: Box::new(image),
            },
        );
        owners.insert(page_id, entry.number);
        previous_id = Some(page_id);
    }
    let mut trailer = [0u8; 4];
    file.read_exact(&mut trailer)?;
    if checksum != entry.checksum || checksum != u32::from_le_bytes(trailer) {
        return Err(Error::corruption("page-base checksum mismatch"));
    }
    Ok(())
}

pub(super) fn inspect_retired_pack(
    directory: &Path,
    number: u64,
    identity: &WalIdentity,
    checkpoint_lsn: Lsn,
    high_water_page_id: PageId,
) -> Result<BasePackEntry> {
    let mut file = fs::File::open(pack_path(directory, number))?;
    let mut header = [0u8; HEADER_SIZE];
    file.read_exact(&mut header)?;
    let word = |offset| u64::from_le_bytes(header[offset..offset + 8].try_into().unwrap());
    let entry = BasePackEntry {
        number,
        checkpoint_lsn: Lsn::new(word(64)),
        page_count: word(72),
        bytes: file.metadata()?.len(),
        checksum: 0,
    };
    if entry.page_count == 0
        || entry.checkpoint_lsn > checkpoint_lsn
        || entry.page_count > high_water_page_id.get().saturating_sub(1)
        || entry.bytes != encoded_size(entry.page_count)
    {
        return Err(Error::corruption(
            "unlisted page-base file is not covered by the checkpoint",
        ));
    }
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::End(-4))?;
    let mut trailer = [0u8; 4];
    file.read_exact(&mut trailer)?;
    let entry = BasePackEntry {
        checksum: u32::from_le_bytes(trailer),
        ..entry
    };
    read_pack(
        directory,
        &entry,
        identity,
        high_water_page_id,
        &mut BTreeMap::new(),
        &mut BTreeMap::new(),
    )?;
    Ok(entry)
}
