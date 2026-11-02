use super::*;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

const MAGIC: [u8; 8] = *b"DBLKBASE";
const HEADER_SIZE: usize = PAGE_SIZE;
const RECORD_SIZE: usize = PAGE_SIZE;
pub(super) const MAX_PACKS: usize = 8;
const SYNC_BYTES: usize = 8 * 1024 * 1024;

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

#[cfg(feature = "blink-demand-paging")]
pub(super) struct PagedBasePackJob {
    pub directory: PathBuf,
    pub identity: WalIdentity,
    pub number: u64,
    pub checkpoint_lsn: Lsn,
    pub page_count: u64,
    pub complete: bool,
    pub coalesce: bool,
}

#[cfg(feature = "blink-demand-paging")]
pub(super) struct PagedBasePackWriter {
    directory: PathBuf,
    file: fs::File,
    entry: BasePackEntry,
    complete: bool,
    expected_page_count: u64,
    written_page_count: u64,
    previous_page_id: Option<PageId>,
    checksum: u32,
    unsynced_bytes: usize,
    write_calls: u64,
    sync_nanos: u64,
    buffer: Vec<u8>,
    buffer_capacity: usize,
    started: Instant,
    injector: Option<Box<dyn FaultInjector + Send>>,
}

#[cfg(feature = "blink-demand-paging")]
pub struct PagedBasePackOutput {
    pub entry: BasePackEntry,
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
        let mut unsynced_bytes = HEADER_SIZE;
        let mut sync_nanos = 0u64;
        let capacity = RECORD_SIZE * if self.coalesce { 64 } else { 1 };
        let mut buffer = Vec::with_capacity(capacity);
        for image in pages.values() {
            hit(injector, "during_checkpoint_page_write")?;
            hit(injector, "during_base_pack_page_write")?;
            buffer.extend_from_slice(&image[..]);
            if buffer.len() == capacity {
                checksum = crc32c::crc32c_append(checksum, &buffer);
                file.write_all(&buffer)?;
                unsynced_bytes += buffer.len();
                buffer.clear();
                write_calls += 1;
                hit(injector, "after_checkpoint_page_write")?;
                if unsynced_bytes >= SYNC_BYTES {
                    hit(injector, "before_base_pack_chunk_sync")?;
                    let sync_started = Instant::now();
                    file.sync_data()?;
                    sync_nanos = sync_nanos.saturating_add(elapsed_nanos(sync_started));
                    unsynced_bytes = 0;
                    hit(injector, "after_base_pack_chunk_sync")?;
                }
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
        let page_nanos = elapsed_nanos(page_started).saturating_sub(sync_nanos);
        hit(injector, "before_checkpoint_data_sync")?;
        hit(injector, "before_base_pack_sync")?;
        let sync_started = Instant::now();
        file.sync_all()?;
        sync_nanos = sync_nanos.saturating_add(elapsed_nanos(sync_started));
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

#[cfg(feature = "blink-demand-paging")]
impl PagedBasePackJob {
    pub fn begin(
        self,
        injector: Option<Box<dyn FaultInjector + Send>>,
    ) -> Result<PagedBasePackWriter> {
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
        header.extend_from_slice(&self.page_count.to_le_bytes());
        header.resize(HEADER_SIZE - 4, 0);
        let header_checksum = crc32c::crc32c(&header);
        header.extend_from_slice(&header_checksum.to_le_bytes());
        let mut injector = injector;
        hit(&mut injector, "before_base_pack_create")?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(pack_path(&self.directory, self.number))?;
        hit(&mut injector, "before_checkpoint_data_flush")?;
        file.write_all(&header)?;
        let buffer_capacity = RECORD_SIZE * if self.coalesce { 4 } else { 1 };
        Ok(PagedBasePackWriter {
            directory: self.directory,
            file,
            entry: BasePackEntry {
                number: self.number,
                checkpoint_lsn: self.checkpoint_lsn,
                page_count: self.page_count,
                bytes: encoded_size(self.page_count),
                checksum: crc32c::crc32c(&header),
            },
            complete: self.complete,
            expected_page_count: self.page_count,
            written_page_count: 0,
            previous_page_id: None,
            checksum: crc32c::crc32c(&header),
            unsynced_bytes: HEADER_SIZE,
            write_calls: 1,
            sync_nanos: 0,
            buffer: Vec::with_capacity(buffer_capacity),
            buffer_capacity,
            started: Instant::now(),
            injector,
        })
    }
}

#[cfg(feature = "blink-demand-paging")]
impl PagedBasePackWriter {
    pub fn append_page(&mut self, page_id: PageId, image: &[u8; PAGE_SIZE]) -> Result<()> {
        if page_id.get() < FIRST_DATA_PAGE
            || self
                .previous_page_id
                .is_some_and(|previous_page_id| page_id <= previous_page_id)
            || self.written_page_count >= self.expected_page_count
        {
            return Err(Error::invariant(
                "paged checkpoint pages are duplicated, unordered, or exceed the target count",
            ));
        }
        let decoded = decode_blink_page(image, page_id)?;
        if decoded.lsn() > self.entry.checkpoint_lsn {
            return Err(Error::invariant(
                "paged checkpoint image exceeds its captured target LSN",
            ));
        }
        hit(&mut self.injector, "during_checkpoint_page_write")?;
        hit(&mut self.injector, "during_base_pack_page_write")?;
        self.buffer.extend_from_slice(image);
        self.written_page_count += 1;
        self.previous_page_id = Some(page_id);
        if self.buffer.len() == self.buffer_capacity {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        self.checksum = crc32c::crc32c_append(self.checksum, &self.buffer);
        self.file.write_all(&self.buffer)?;
        self.unsynced_bytes = self.unsynced_bytes.saturating_add(self.buffer.len());
        self.buffer.clear();
        self.write_calls += 1;
        hit(&mut self.injector, "after_checkpoint_page_write")?;
        if self.unsynced_bytes >= SYNC_BYTES {
            hit(&mut self.injector, "before_base_pack_chunk_sync")?;
            let sync_started = Instant::now();
            self.file.sync_data()?;
            self.sync_nanos = self.sync_nanos.saturating_add(elapsed_nanos(sync_started));
            self.unsynced_bytes = 0;
            hit(&mut self.injector, "after_base_pack_chunk_sync")?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<PagedBasePackOutput> {
        if self.written_page_count != self.expected_page_count {
            return Err(Error::invariant(
                "paged checkpoint page count differs from its captured index root",
            ));
        }
        self.flush_buffer()?;
        self.file.write_all(&self.checksum.to_le_bytes())?;
        self.write_calls += 1;
        hit(&mut self.injector, "before_checkpoint_data_sync")?;
        hit(&mut self.injector, "before_base_pack_sync")?;
        let sync_started = Instant::now();
        self.file.sync_all()?;
        self.sync_nanos = self.sync_nanos.saturating_add(elapsed_nanos(sync_started));
        hit(&mut self.injector, "after_base_pack_sync")?;
        hit(&mut self.injector, "after_checkpoint_data_sync")?;
        let directory_started = Instant::now();
        hit(&mut self.injector, "before_base_pack_directory_sync")?;
        fs::File::open(&self.directory)?.sync_all()?;
        hit(&mut self.injector, "after_base_pack_directory_sync")?;
        self.entry.checksum = self.checksum;
        let page_nanos = elapsed_nanos(self.started).saturating_sub(self.sync_nanos);
        Ok(PagedBasePackOutput {
            entry: self.entry,
            complete: self.complete,
            write_calls: self.write_calls,
            phase_timings: [
                page_nanos,
                self.sync_nanos,
                elapsed_nanos(directory_started),
            ],
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
    scan_pack(
        &mut file,
        entry,
        identity,
        high_water_page_id,
        |page_id, _image_lsn, _record_offset, image| {
            pages.insert(
                page_id,
                RecoveredWalPage {
                    commit_lsn: entry.checkpoint_lsn,
                    image: Box::new(*image),
                },
            );
            owners.insert(page_id, entry.number);
            Ok(())
        },
    )
}

pub(super) fn scan_pack(
    file: &mut fs::File,
    entry: &BasePackEntry,
    identity: &WalIdentity,
    high_water_page_id: PageId,
    mut visit: impl FnMut(PageId, Lsn, u64, &[u8; PAGE_SIZE]) -> Result<()>,
) -> Result<()> {
    if entry.bytes != encoded_size(entry.page_count) || file.metadata()?.len() != entry.bytes {
        return Err(Error::corruption("page-base file length mismatch"));
    }
    file.rewind()?;
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
    for page_index in 0..entry.page_count {
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
        let decoded = decode_blink_page(&record, page_id)?;
        if decoded.lsn() > entry.checkpoint_lsn {
            return Err(Error::corruption(
                "page-base image exceeds its checkpoint target",
            ));
        }
        let record_offset = page_index
            .checked_mul(RECORD_SIZE as u64)
            .and_then(|offset| offset.checked_add(HEADER_SIZE as u64))
            .ok_or_else(|| Error::corruption("page-base record offset overflow"))?;
        visit(page_id, decoded.lsn(), record_offset, &record)?;
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
