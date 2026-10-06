//! Versioned redo-only write-ahead logging.
//!
//! The WAL is deliberately independent from the data file.  It contains
//! encoded after-images, byte deltas against an earlier after-image of the
//! same WAL history, and an explicit commit marker.  A redo record is never
//! replayed unless the matching commit marker is complete and its digest
//! matches the record sequence.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::io::ErrorKind;
use std::time::Instant;

use dodb_core::{Error, Lsn, PageId, Result, Revision, ShardEpoch, ShardId, TenantId};

use crate::durable_file::DurableFile;
use crate::fault::FaultInjector;
use crate::page::{PAGE_SIZE, decode_page_at};
use crate::superblock::decode_superblock;

pub const WAL_FORMAT_VERSION: u16 = 3;
pub const WAL_LOGICAL_FORMAT_VERSION: u16 = 4;
pub const WAL_PAGE_IMAGE_FORMAT_VERSION: u16 = 2;
pub const WAL_MAGIC: [u8; 4] = *b"DWAL";
pub const WAL_HEADER_SIZE: usize = 48;
pub const WAL_TRAILER_SIZE: usize = 4;
pub const WAL_MIN_FRAME_SIZE: usize = WAL_HEADER_SIZE + WAL_TRAILER_SIZE;
pub const WAL_MAX_PAYLOAD_SIZE: usize = 64 * 1024 * 1024;

const HEADER_CHECKSUM_OFFSET: usize = 40;
const PAYLOAD_CHECKSUM_OFFSET: usize = 44;
const LEGACY_WAL_FORMAT_VERSION: u16 = 1;
const LEGACY_INIT_PAYLOAD_SIZE: usize = 44;
const INIT_PAYLOAD_SIZE: usize = 52;
const LOGICAL_INIT_PAYLOAD_SIZE: usize = 60;
const INIT_FRAME_SIZE: usize = WAL_HEADER_SIZE + INIT_PAYLOAD_SIZE + WAL_TRAILER_SIZE;
pub(crate) const LOGICAL_INIT_FRAME_SIZE: usize =
    WAL_HEADER_SIZE + LOGICAL_INIT_PAYLOAD_SIZE + WAL_TRAILER_SIZE;
pub(crate) const MAX_LOGICAL_WAL_BYTES: u64 = 256 * 1024 * 1024;
pub(crate) const PAGE_IMAGE_PAYLOAD_SIZE: usize = 8 + PAGE_SIZE;
const COMMIT_PAYLOAD_SIZE: usize = 16;
pub const PAGE_DELTA_HEADER_SIZE: usize = 18;
pub const PAGE_DELTA_SPAN_HEADER_SIZE: usize = 4;
pub const PAGE_DELTA_SPAN_MERGE_GAP: usize = PAGE_DELTA_SPAN_HEADER_SIZE;
pub const PAGE_DELTA_MAX_SPANS: usize = PAGE_SIZE.div_ceil(PAGE_DELTA_SPAN_MERGE_GAP + 1);
const PAGE_LSN_RANGE: std::ops::Range<usize> = 16..24;

const SUPERBLOCK_A_PAGE: PageId = PageId::ZERO;
const SUPERBLOCK_B_PAGE: PageId = PageId::new(1);

/// Identifies the data-page decoder used to validate WAL page images.
///
/// The WAL frame protocol is intentionally shared by the baseline and the
/// experimental engine.  Their page/superblock bodies are not shared, so the
/// validator must be selected explicitly at open time instead of guessing
/// from an image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalPageImageFormat {
    Baseline,
    ExperimentalBlink,
    ExperimentalBtree,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PageImageValidationMode {
    Strict,
    TrustedInternal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WalRecordType {
    Init = 1,
    PageImage = 2,
    Commit = 3,
    PageDelta = 4,
    LogicalPut = 5,
    LogicalDelete = 6,
}

impl WalRecordType {
    fn decode(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Init),
            2 => Ok(Self::PageImage),
            3 => Ok(Self::Commit),
            4 => Ok(Self::PageDelta),
            5 => Ok(Self::LogicalPut),
            6 => Ok(Self::LogicalDelete),
            other => Err(Error::unsupported_format(format!(
                "unknown WAL record type {other}"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalIdentity {
    pub database_uuid: [u8; 16],
    pub tenant_id: TenantId,
    pub shard_id: ShardId,
    pub shard_epoch: ShardEpoch,
}

impl WalIdentity {
    pub fn new(
        database_uuid: [u8; 16],
        tenant_id: TenantId,
        shard_id: ShardId,
        shard_epoch: ShardEpoch,
    ) -> Self {
        Self {
            database_uuid,
            tenant_id,
            shard_id,
            shard_epoch,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalPageImage {
    pub page_id: PageId,
    pub image: [u8; PAGE_SIZE],
}

/// One logical commit in a physical WAL group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalCommit {
    pub batch_id: u64,
    pub commit_lsn: Lsn,
    pub pages: Vec<WalPageImage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalWalMutation {
    pub encoded_key: Vec<u8>,
    pub value: Option<Vec<u8>>,
    pub revision: Revision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalWalTransaction {
    pub commit_sequence: u64,
    pub mutations: Vec<LogicalWalMutation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedLogicalWalTransaction {
    pub batch_id: u64,
    pub physical_commit_lsn: Lsn,
    pub commit_sequence: u64,
    pub mutations: Vec<LogicalWalMutation>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LogicalWalAppendReport {
    pub first_record_lsn: Lsn,
    pub physical_commit_lsn: Lsn,
    pub commit_sequence: u64,
    pub record_count: usize,
    pub bytes_written: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalRedoKind {
    PageImage,
    PageDelta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedWalBatch {
    pub batch_id: u64,
    pub commit_lsn: Lsn,
    pub pages: Vec<WalPageImage>,
    pub redo_kinds: Vec<WalRedoKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveredWalPage {
    pub commit_lsn: Lsn,
    pub image: Box<[u8; PAGE_SIZE]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PageChainEntry {
    page_lsn: Lsn,
    image_crc: u32,
}

pub(crate) type WalDeltaBaseSource<'source, 'base> =
    dyn FnMut(PageId) -> Option<Cow<'base, [u8; PAGE_SIZE]>> + 'source;

pub(crate) struct WalDeltaRequest<'request, 'source, 'base> {
    pub eligible_commits: &'request [bool],
    pub base_source: &'request mut WalDeltaBaseSource<'source, 'base>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WalRedoStats {
    pub page_image_records: u64,
    pub page_delta_records: u64,
    pub page_delta_payload_bytes: u64,
    pub page_delta_spans: u64,
    pub page_delta_changed_bytes: u64,
    pub image_superblock: u64,
    pub image_not_requested: u64,
    pub image_page_image_format: u64,
    pub image_ineligible_commit: u64,
    pub image_first_touch: u64,
    pub image_no_base: u64,
    pub image_not_smaller: u64,
}

impl WalRedoStats {
    fn add(&mut self, other: &Self) {
        self.page_image_records += other.page_image_records;
        self.page_delta_records += other.page_delta_records;
        self.page_delta_payload_bytes += other.page_delta_payload_bytes;
        self.page_delta_spans += other.page_delta_spans;
        self.page_delta_changed_bytes += other.page_delta_changed_bytes;
        self.image_superblock += other.image_superblock;
        self.image_not_requested += other.image_not_requested;
        self.image_page_image_format += other.image_page_image_format;
        self.image_ineligible_commit += other.image_ineligible_commit;
        self.image_first_touch += other.image_first_touch;
        self.image_no_base += other.image_no_base;
        self.image_not_smaller += other.image_not_smaller;
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WalScanReport {
    pub records_scanned: usize,
    pub committed_batches: usize,
    pub replayable_pages: usize,
    pub torn_tail_bytes: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WalAppendReport {
    pub first_record_lsn: Lsn,
    pub commit_lsn: Lsn,
    pub page_count: usize,
    pub bytes_written: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WalMetrics {
    pub wal_bytes: u64,
    pub wal_syncs: u64,
    pub committed_batches: usize,
    pub page_images: usize,
    pub retained_recovery_batches: usize,
    pub retained_recovery_page_images: usize,
    pub tracked_chain_pages: usize,
    pub redo: WalRedoStats,
    pub redo_plan_nanos: u64,
    pub append_nanos: u64,
    pub sync_nanos: u64,
    pub group_encode_nanos: u64,
    pub group_page_lsn_validate_nanos: u64,
    pub group_page_image_materialize_nanos: u64,
    pub group_page_image_validate_nanos: u64,
    pub group_digest_copy_nanos: u64,
    pub group_page_payload_crc_nanos: u64,
    pub group_page_header_crc_nanos: u64,
    pub group_page_frame_materialize_nanos: u64,
    pub group_page_frame_append_nanos: u64,
    pub group_page_direct_encode_nanos: u64,
    pub group_commit_digest_crc_nanos: u64,
    pub group_commit_payload_crc_nanos: u64,
    pub group_commit_header_crc_nanos: u64,
    pub group_commit_frame_materialize_nanos: u64,
    pub group_commit_frame_append_nanos: u64,
    pub group_commit_direct_encode_nanos: u64,
    pub group_page_frames: u64,
    pub group_commit_frames: u64,
    pub group_page_validations: u64,
    pub group_write_nanos: u64,
    pub physical_write_calls: u64,
}

#[derive(Clone, Debug)]
enum PendingRedo {
    Image(Box<WalPageImage>),
    Delta(Vec<u8>),
}

#[derive(Clone, Debug)]
struct PendingBatch {
    batch_id: u64,
    first_record_lsn: Lsn,
    records: Vec<PendingRedo>,
    digest: u32,
}

#[derive(Clone, Debug)]
enum PlannedRedo {
    Image,
    Delta(Vec<u8>),
}

struct RedoPlan {
    records: Vec<Vec<PlannedRedo>>,
    chain_updates: Vec<(PageId, PageChainEntry)>,
    stats: WalRedoStats,
}

struct GroupAppendOutcome {
    last_commit_lsn: Lsn,
    next_lsn: Lsn,
    next_batch_id: u64,
    committed_batches: usize,
    replayable_pages: usize,
    stats: WalRedoStats,
    chain_updates: Vec<(PageId, PageChainEntry)>,
}

/// One logical commit whose redo records were already planned and encoded by
/// this engine's leaf workers. The WAL still checks the page chain, the LSNs
/// and the batch identity before it writes anything.
pub(crate) struct PreparedWalCommit<'record> {
    pub batch_id: u64,
    pub commit_lsn: Lsn,
    pub records: Vec<PreparedWalRecord<'record>>,
}

pub(crate) struct PreparedWalRecord<'record> {
    pub page_id: PageId,
    pub page_lsn: Lsn,
    pub image_crc: u32,
    pub redo: PreparedWalRedo<'record>,
}

pub(crate) enum PreparedWalRedo<'record> {
    Image(&'record [u8; PAGE_SIZE]),
    Delta {
        payload: &'record [u8],
        base_lsn: Lsn,
        base_crc: u32,
        spans: u64,
        changed_bytes: u64,
    },
}

struct WalScanOutput {
    next_lsn: Lsn,
    next_batch_id: u64,
    format_version: u16,
    history_start_lsn: Lsn,
    last_commit_lsn: Option<Lsn>,
    batches: Vec<CommittedWalBatch>,
    pages: BTreeMap<PageId, RecoveredWalPage>,
    redo: WalRedoStats,
    report: WalScanReport,
    valid_length: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScanMaterialization {
    Recovery,
    Batches,
}

struct EncodedWalGroup {
    bytes: Vec<u8>,
    reports: Vec<WalAppendReport>,
    next_lsn: Lsn,
    next_batch_id: u64,
    attribution: WalEncodeAttribution,
}

#[derive(Clone, Debug, Default)]
struct WalEncodeAttribution {
    group_page_lsn_validate_nanos: u64,
    group_page_image_materialize_nanos: u64,
    group_page_image_validate_nanos: u64,
    group_digest_copy_nanos: u64,
    group_page_payload_crc_nanos: u64,
    group_page_header_crc_nanos: u64,
    group_page_frame_materialize_nanos: u64,
    group_page_frame_append_nanos: u64,
    group_page_direct_encode_nanos: u64,
    group_commit_digest_crc_nanos: u64,
    group_commit_payload_crc_nanos: u64,
    group_commit_header_crc_nanos: u64,
    group_commit_frame_materialize_nanos: u64,
    group_commit_frame_append_nanos: u64,
    group_commit_direct_encode_nanos: u64,
    group_page_frames: u64,
    group_commit_frames: u64,
    group_page_validations: u64,
}

/// A WAL file with the metadata of its complete commits.
pub struct WalLog<F: DurableFile> {
    file: F,
    identity: WalIdentity,
    format_version: u16,
    next_lsn: Lsn,
    next_batch_id: u64,
    history_start_lsn: Lsn,
    last_commit_lsn: Option<Lsn>,
    recovery_batches: Vec<CommittedWalBatch>,
    recovery_pages: BTreeMap<PageId, RecoveredWalPage>,
    page_chain: HashMap<PageId, PageChainEntry>,
    redo_stats: WalRedoStats,
    redo_plan_nanos: u64,
    scan_report: WalScanReport,
    sync_count: u64,
    page_images: usize,
    append_nanos: u64,
    sync_nanos: u64,
    group_encode_nanos: u64,
    attribution: WalEncodeAttribution,
    group_write_nanos: u64,
    physical_write_calls: u64,
    page_image_format: WalPageImageFormat,
}

impl<F: DurableFile> WalLog<F> {
    pub fn open(file: F, identity: WalIdentity) -> Result<Self> {
        Self::open_with_fault_injector(file, identity, None)
    }

    pub fn open_with_fault_injector(
        file: F,
        identity: WalIdentity,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Self> {
        Self::open_with_fault_injector_and_start_lsn(file, identity, Lsn::ZERO, injector.take())
    }

    pub fn open_with_fault_injector_and_start_lsn(
        file: F,
        identity: WalIdentity,
        start_after_lsn: Lsn,
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Self> {
        Self::open_with_page_image_format_and_fault_injector_and_start_lsn(
            file,
            identity,
            WalPageImageFormat::Baseline,
            start_after_lsn,
            injector,
        )
    }

    pub fn open_with_page_image_format(
        file: F,
        identity: WalIdentity,
        page_image_format: WalPageImageFormat,
    ) -> Result<Self> {
        Self::open_with_page_image_format_and_fault_injector_and_start_lsn(
            file,
            identity,
            page_image_format,
            Lsn::ZERO,
            None,
        )
    }

    pub fn open_with_page_image_format_and_fault_injector_and_start_lsn(
        mut file: F,
        identity: WalIdentity,
        page_image_format: WalPageImageFormat,
        start_after_lsn: Lsn,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Self> {
        let length = file.len()?;
        if length == 0 {
            return Self::initialize_empty(file, identity, start_after_lsn, page_image_format);
        }
        if usize::try_from(length)
            .ok()
            .is_some_and(|length| length < INIT_FRAME_SIZE)
            && is_torn_initialization_prefix(
                &mut file,
                &identity,
                start_after_lsn,
                length,
                page_image_format,
            )?
        {
            file.set_len(0)
                .map_err(|error| Error::recovery(format!("WAL torn INIT reset failed: {error}")))?;
            file.sync_data().map_err(|error| {
                Error::durability(format!("WAL torn INIT reset sync failed: {error}"))
            })?;
            return Self::initialize_empty(file, identity, start_after_lsn, page_image_format);
        }

        let WalScanOutput {
            next_lsn,
            next_batch_id,
            format_version,
            history_start_lsn,
            last_commit_lsn,
            batches,
            pages,
            redo,
            mut report,
            valid_length,
        } = scan_wal(
            &mut file,
            &identity,
            page_image_format,
            ScanMaterialization::Recovery,
        )?;
        if format_version == WAL_LOGICAL_FORMAT_VERSION {
            return Err(Error::unsupported_format(
                "logical WAL v4 requires the logical WAL reader",
            ));
        }
        if valid_length < length {
            hit(&mut injector, "before_wal_tail_truncate")?;
            file.set_len(valid_length)?;
            file.sync_data().map_err(|error| {
                Error::durability(format!("WAL torn-tail truncation sync failed: {error}"))
            })?;
            hit(&mut injector, "after_wal_tail_truncate")?;
        }
        report.torn_tail_bytes = usize::try_from(length - valid_length)
            .map_err(|_| Error::invariant("WAL tail length does not fit usize"))?;
        let page_images = usize::try_from(redo.page_image_records)
            .map_err(|_| Error::invariant("WAL page-image count does not fit usize"))?;
        let page_chain = pages
            .iter()
            .filter(|(page_id, _)| !is_superblock_page(**page_id))
            .map(|(page_id, page)| {
                (
                    *page_id,
                    PageChainEntry {
                        page_lsn: page_lsn_of(&page.image),
                        image_crc: crc32c::crc32c(&page.image[..]),
                    },
                )
            })
            .collect();
        Ok(Self {
            file,
            identity,
            format_version,
            next_lsn,
            next_batch_id,
            history_start_lsn,
            last_commit_lsn,
            recovery_batches: batches,
            recovery_pages: pages,
            page_chain,
            redo_stats: redo,
            redo_plan_nanos: 0,
            scan_report: report,
            sync_count: 0,
            page_images,
            append_nanos: 0,
            sync_nanos: 0,
            group_encode_nanos: 0,
            attribution: WalEncodeAttribution::default(),
            group_write_nanos: 0,
            physical_write_calls: 0,
            page_image_format,
        })
    }

    fn initialize_empty(
        file: F,
        identity: WalIdentity,
        start_after_lsn: Lsn,
        page_image_format: WalPageImageFormat,
    ) -> Result<Self> {
        let next_lsn = start_after_lsn
            .get()
            .checked_add(1)
            .map(Lsn::new)
            .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
        let mut wal = Self {
            file,
            identity,
            format_version: write_format_version(page_image_format),
            next_lsn,
            next_batch_id: 1,
            history_start_lsn: start_after_lsn,
            last_commit_lsn: None,
            recovery_batches: Vec::new(),
            recovery_pages: BTreeMap::new(),
            page_chain: HashMap::new(),
            redo_stats: WalRedoStats::default(),
            redo_plan_nanos: 0,
            scan_report: WalScanReport::default(),
            sync_count: 0,
            page_images: 0,
            append_nanos: 0,
            sync_nanos: 0,
            group_encode_nanos: 0,
            attribution: WalEncodeAttribution::default(),
            group_write_nanos: 0,
            physical_write_calls: 0,
            page_image_format,
        };
        let payload = wal.identity_payload(start_after_lsn);
        wal.append_frame(WalRecordType::Init, Lsn::ZERO, 0, 0, &payload, &mut None)?;
        wal.file
            .sync_data()
            .map_err(|error| Error::durability(format!("initial WAL sync failed: {error}")))?;
        wal.sync_count = 1;
        wal.scan_report.records_scanned = 1;
        Ok(wal)
    }

    pub fn into_file(self) -> F {
        self.file
    }

    pub fn recovery_batches(&self) -> &[CommittedWalBatch] {
        &self.recovery_batches
    }

    pub fn take_recovery_batches(&mut self) -> Vec<CommittedWalBatch> {
        std::mem::take(&mut self.recovery_batches)
    }

    pub fn recovery_pages(&self) -> &BTreeMap<PageId, RecoveredWalPage> {
        &self.recovery_pages
    }

    pub fn take_recovery_pages(&mut self) -> BTreeMap<PageId, RecoveredWalPage> {
        std::mem::take(&mut self.recovery_pages)
    }

    pub fn format_version(&self) -> u16 {
        self.format_version
    }

    pub fn page_delta_enabled(&self) -> bool {
        self.format_version >= WAL_FORMAT_VERSION
            && self.page_image_format != WalPageImageFormat::Baseline
    }

    pub fn last_commit_lsn(&self) -> Option<Lsn> {
        self.last_commit_lsn
    }

    pub fn committed_batch_count(&self) -> usize {
        self.scan_report.committed_batches
    }

    #[cfg(test)]
    pub(crate) fn committed_batches_on_disk(&mut self) -> Vec<CommittedWalBatch> {
        scan_wal(
            &mut self.file,
            &self.identity,
            self.page_image_format,
            ScanMaterialization::Batches,
        )
        .expect("WAL on disk scans")
        .batches
    }

    #[cfg(test)]
    pub(crate) fn frame_summaries_from(
        &mut self,
        start_offset: u64,
    ) -> Vec<(WalRecordType, u64, usize)> {
        let length = self.file.len().expect("WAL length");
        let mut offset = start_offset;
        let mut summaries = Vec::new();
        while offset + WAL_HEADER_SIZE as u64 <= length {
            let header =
                read_exact_at(&mut self.file, offset, WAL_HEADER_SIZE).expect("WAL header");
            let (_, record_type, frame_length, _, _, batch_id, _) =
                decode_header(&header).expect("WAL header decodes");
            summaries.push((record_type, batch_id, frame_length));
            offset += frame_length as u64;
        }
        summaries
    }

    pub fn next_lsn(&self) -> Lsn {
        self.next_lsn
    }

    pub fn next_batch_id(&self) -> u64 {
        self.next_batch_id
    }

    pub fn history_start_lsn(&self) -> Lsn {
        self.history_start_lsn
    }

    pub fn resume_after(&mut self, checkpoint_lsn: Lsn) -> Result<()> {
        let next_lsn = checkpoint_lsn
            .get()
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
        if self.next_lsn < Lsn::new(next_lsn) {
            self.next_lsn = Lsn::new(next_lsn);
        }
        Ok(())
    }

    pub fn reset(
        &mut self,
        checkpoint_lsn: Lsn,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        if checkpoint_lsn < self.history_start_lsn {
            return Err(Error::checkpoint(
                "WAL reset would move its history start backwards",
            ));
        }
        hit(&mut injector, "before_wal_reset")?;
        hit(&mut injector, "during_wal_truncate")?;
        self.file
            .set_len(0)
            .map_err(|error| Error::checkpoint(format!("WAL reset truncation failed: {error}")))?;
        hit(&mut injector, "after_wal_truncate")?;
        hit(&mut injector, "before_wal_reset_truncate_sync")?;
        hit(&mut injector, "during_wal_reset_truncate_sync")?;
        self.file.sync_data().map_err(|error| {
            Error::checkpoint(format!("WAL reset truncation sync failed: {error}"))
        })?;
        hit(&mut injector, "after_wal_reset_truncate_sync")?;

        self.history_start_lsn = checkpoint_lsn;
        self.format_version = write_format_version(self.page_image_format);
        self.next_lsn = checkpoint_lsn
            .get()
            .checked_add(1)
            .map(Lsn::new)
            .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
        self.next_batch_id = 1;
        self.last_commit_lsn = None;
        self.recovery_batches = Vec::new();
        self.recovery_pages = BTreeMap::new();
        self.page_chain = HashMap::new();
        self.scan_report = WalScanReport {
            records_scanned: 1,
            ..WalScanReport::default()
        };
        self.page_images = 0;
        hit(&mut injector, "before_wal_reinitialization")?;
        let payload = self.identity_payload(checkpoint_lsn);
        hit(&mut injector, "during_wal_reinitialization")?;
        self.append_frame(
            WalRecordType::Init,
            Lsn::ZERO,
            0,
            0,
            &payload,
            &mut injector,
        )?;
        hit(&mut injector, "after_wal_reset_write")?;
        hit(&mut injector, "before_wal_reset_sync")?;
        hit(&mut injector, "during_wal_reset_sync")?;
        self.file
            .sync_data()
            .map_err(|error| Error::checkpoint(format!("WAL reset sync failed: {error}")))?;
        self.sync_count = self
            .sync_count
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL sync count overflow"))?;
        hit(&mut injector, "after_wal_reset_sync")?;
        Ok(())
    }

    pub fn scan_report(&self) -> &WalScanReport {
        &self.scan_report
    }

    pub fn metrics(&self) -> Result<WalMetrics> {
        Ok(WalMetrics {
            wal_bytes: self.file.len()?,
            wal_syncs: self.sync_count,
            committed_batches: self.scan_report.committed_batches,
            page_images: self.page_images,
            retained_recovery_batches: self.recovery_batches.len(),
            retained_recovery_page_images: self
                .recovery_batches
                .iter()
                .map(|batch| batch.pages.len())
                .sum::<usize>()
                + self.recovery_pages.len(),
            tracked_chain_pages: self.page_chain.len(),
            redo: self.redo_stats,
            redo_plan_nanos: self.redo_plan_nanos,
            append_nanos: self.append_nanos,
            sync_nanos: self.sync_nanos,
            group_encode_nanos: self.group_encode_nanos,
            group_page_lsn_validate_nanos: self.attribution.group_page_lsn_validate_nanos,
            group_page_image_materialize_nanos: self.attribution.group_page_image_materialize_nanos,
            group_page_image_validate_nanos: self.attribution.group_page_image_validate_nanos,
            group_digest_copy_nanos: self.attribution.group_digest_copy_nanos,
            group_page_payload_crc_nanos: self.attribution.group_page_payload_crc_nanos,
            group_page_header_crc_nanos: self.attribution.group_page_header_crc_nanos,
            group_page_frame_materialize_nanos: self.attribution.group_page_frame_materialize_nanos,
            group_page_frame_append_nanos: self.attribution.group_page_frame_append_nanos,
            group_page_direct_encode_nanos: self.attribution.group_page_direct_encode_nanos,
            group_commit_digest_crc_nanos: self.attribution.group_commit_digest_crc_nanos,
            group_commit_payload_crc_nanos: self.attribution.group_commit_payload_crc_nanos,
            group_commit_header_crc_nanos: self.attribution.group_commit_header_crc_nanos,
            group_commit_frame_materialize_nanos: self
                .attribution
                .group_commit_frame_materialize_nanos,
            group_commit_frame_append_nanos: self.attribution.group_commit_frame_append_nanos,
            group_commit_direct_encode_nanos: self.attribution.group_commit_direct_encode_nanos,
            group_page_frames: self.attribution.group_page_frames,
            group_commit_frames: self.attribution.group_commit_frames,
            group_page_validations: self.attribution.group_page_validations,
            group_write_nanos: self.group_write_nanos,
            physical_write_calls: self.physical_write_calls,
        })
    }

    fn add_attribution(&mut self, attribution: &WalEncodeAttribution) -> Result<()> {
        macro_rules! add_metric {
            ($field:ident) => {
                self.attribution.$field =
                    self.attribution
                        .$field
                        .checked_add(attribution.$field)
                        .ok_or_else(|| Error::invariant("WAL attribution metric overflow"))?;
            };
        }
        add_metric!(group_page_lsn_validate_nanos);
        add_metric!(group_page_image_materialize_nanos);
        add_metric!(group_page_image_validate_nanos);
        add_metric!(group_digest_copy_nanos);
        add_metric!(group_page_payload_crc_nanos);
        add_metric!(group_page_header_crc_nanos);
        add_metric!(group_page_frame_materialize_nanos);
        add_metric!(group_page_frame_append_nanos);
        add_metric!(group_page_direct_encode_nanos);
        add_metric!(group_commit_digest_crc_nanos);
        add_metric!(group_commit_payload_crc_nanos);
        add_metric!(group_commit_header_crc_nanos);
        add_metric!(group_commit_frame_materialize_nanos);
        add_metric!(group_commit_frame_append_nanos);
        add_metric!(group_commit_direct_encode_nanos);
        add_metric!(group_page_frames);
        add_metric!(group_commit_frames);
        add_metric!(group_page_validations);
        Ok(())
    }

    /// Appends one logical commit and syncs the WAL.
    pub fn append_commit(
        &mut self,
        batch_id: u64,
        commit_lsn: Lsn,
        pages: &[WalPageImage],
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<WalAppendReport> {
        let mut reports = self.append_group(
            &[WalCommit {
                batch_id,
                commit_lsn,
                pages: pages.to_vec(),
            }],
            injector.take(),
        )?;
        Ok(reports.remove(0))
    }

    /// Appends several logical commits in serialization order and performs
    /// exactly one WAL sync for the group. Each commit retains its own batch
    /// identity, page images, commit marker, and commit LSN.
    pub fn append_group(
        &mut self,
        commits: &[WalCommit],
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        self.append_group_inner(commits, injector, PageImageValidationMode::Strict, None)
    }

    pub(crate) fn append_group_with_page_deltas(
        &mut self,
        commits: &[WalCommit],
        delta: &mut WalDeltaRequest<'_, '_, '_>,
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        self.append_group_inner(
            commits,
            injector,
            PageImageValidationMode::Strict,
            Some(delta),
        )
    }

    pub(crate) fn append_group_trusted_internal_with_page_deltas(
        &mut self,
        commits: &[WalCommit],
        delta: &mut WalDeltaRequest<'_, '_, '_>,
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        if injector.is_some() {
            return self.append_group_with_page_deltas(commits, delta, injector);
        }
        self.append_group_inner(
            commits,
            None,
            PageImageValidationMode::TrustedInternal,
            Some(delta),
        )
    }

    /// Appends page images produced moments earlier by this storage engine's
    /// internal Blink page and superblock encoders in the same process. Never
    /// use this path for caller-provided bytes. Fault-injected calls delegate
    /// to the strict public path.
    #[cfg(test)]
    pub(crate) fn append_group_trusted_internal(
        &mut self,
        commits: &[WalCommit],
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        if injector.is_some() {
            return self.append_group(commits, injector);
        }
        self.append_group_inner(
            commits,
            None,
            PageImageValidationMode::TrustedInternal,
            None,
        )
    }

    fn append_group_inner(
        &mut self,
        commits: &[WalCommit],
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
        validation_mode: PageImageValidationMode,
        delta: Option<&mut WalDeltaRequest<'_, '_, '_>>,
    ) -> Result<Vec<WalAppendReport>> {
        if commits.is_empty() {
            return Err(Error::invalid_input("a WAL group must contain a commit"));
        }

        let append_started = Instant::now();
        hit(&mut injector, "before_wal_append")?;
        let plan_started = Instant::now();
        let plan = self.plan_redo(commits, delta)?;
        let plan_nanos = elapsed_nanos(plan_started)?;
        self.redo_plan_nanos = self.redo_plan_nanos.saturating_add(plan_nanos);
        self.group_encode_nanos = self
            .group_encode_nanos
            .checked_add(plan_nanos)
            .ok_or_else(|| Error::invariant("WAL group-encode timing overflow"))?;
        let (reports, next_lsn, next_batch_id) = if injector.is_some() {
            self.append_group_fault_injectable(commits, &plan, &mut injector)?
        } else {
            let encode_started = Instant::now();
            let encoded = self.encode_group(commits, validation_mode, &plan)?;
            self.group_encode_nanos = self
                .group_encode_nanos
                .checked_add(elapsed_nanos(encode_started)?)
                .ok_or_else(|| Error::invariant("WAL group-encode timing overflow"))?;
            self.add_attribution(&encoded.attribution)?;

            let write_started = Instant::now();
            let write_result = (|| {
                let offset = self.file.len()?;
                write_all_at_counted(
                    &mut self.file,
                    offset,
                    &encoded.bytes,
                    &mut self.physical_write_calls,
                )
            })();
            self.group_write_nanos = self
                .group_write_nanos
                .checked_add(elapsed_nanos(write_started)?)
                .ok_or_else(|| Error::invariant("WAL group-write timing overflow"))?;
            write_result?;
            (encoded.reports, encoded.next_lsn, encoded.next_batch_id)
        };

        let replayable_pages = commits
            .iter()
            .map(|commit| commit.pages.len())
            .try_fold(0usize, |count, pages| count.checked_add(pages))
            .ok_or_else(|| Error::invariant("WAL record count overflow"))?;
        self.finish_group_append(
            append_started,
            &mut injector,
            GroupAppendOutcome {
                last_commit_lsn: commits.last().expect("non-empty WAL group").commit_lsn,
                next_lsn,
                next_batch_id,
                committed_batches: commits.len(),
                replayable_pages,
                stats: plan.stats,
                chain_updates: plan.chain_updates,
            },
        )?;
        Ok(reports)
    }

    fn finish_group_append(
        &mut self,
        append_started: Instant,
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
        outcome: GroupAppendOutcome,
    ) -> Result<()> {
        self.append_nanos = self
            .append_nanos
            .checked_add(elapsed_nanos(append_started)?)
            .ok_or_else(|| Error::invariant("WAL append timing overflow"))?;
        hit(injector, "after_group_records_written")?;
        hit(injector, "before_wal_sync")?;
        hit(injector, "during_wal_sync")?;
        let sync_started = Instant::now();
        self.file.sync_data().map_err(|error| {
            Error::durability(format!(
                "WAL group sync failed at commit LSN {}: {error}",
                outcome.last_commit_lsn
            ))
        })?;
        self.sync_nanos = self
            .sync_nanos
            .checked_add(elapsed_nanos(sync_started)?)
            .ok_or_else(|| Error::invariant("WAL sync timing overflow"))?;
        self.sync_count = self
            .sync_count
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL sync count overflow"))?;
        hit(injector, "after_wal_sync")?;

        self.last_commit_lsn = Some(outcome.last_commit_lsn);
        self.next_lsn = outcome.next_lsn;
        self.next_batch_id = outcome.next_batch_id;
        let record_count = outcome
            .replayable_pages
            .checked_add(outcome.committed_batches)
            .ok_or_else(|| Error::invariant("WAL record count overflow"))?;
        self.scan_report.records_scanned = self
            .scan_report
            .records_scanned
            .checked_add(record_count)
            .ok_or_else(|| Error::invariant("WAL record count overflow"))?;
        self.scan_report.committed_batches += outcome.committed_batches;
        self.scan_report.replayable_pages += outcome.replayable_pages;
        self.page_images = self
            .page_images
            .checked_add(
                usize::try_from(outcome.stats.page_image_records)
                    .map_err(|_| Error::invariant("WAL page-image count overflow"))?,
            )
            .ok_or_else(|| Error::invariant("WAL page-image count overflow"))?;
        for (page_id, entry) in outcome.chain_updates {
            self.page_chain.insert(page_id, entry);
        }
        self.redo_stats.add(&outcome.stats);
        Ok(())
    }

    /// Appends commits whose page redo records were prepared outside the WAL
    /// (image or page delta, with the image CRC of every after-image). The
    /// chain check below uses the same rule as `plan_redo`: a delta must name
    /// the LSN and CRC of the latest committed image of its page.
    pub(crate) fn append_group_prepared(
        &mut self,
        commits: &[PreparedWalCommit<'_>],
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        if commits.is_empty() {
            return Err(Error::invalid_input("a WAL group must contain a commit"));
        }
        if !self.page_delta_enabled() {
            return Err(Error::invariant(
                "prepared page redo needs a page-delta WAL",
            ));
        }
        let append_started = Instant::now();
        hit(&mut injector, "before_wal_append")?;
        let plan_started = Instant::now();
        let strict = injector.is_some() || cfg!(debug_assertions);
        let (chain_updates, stats) = self.check_prepared_chain(commits, strict)?;
        let plan_nanos = elapsed_nanos(plan_started)?;
        self.redo_plan_nanos = self.redo_plan_nanos.saturating_add(plan_nanos);
        self.group_encode_nanos = self
            .group_encode_nanos
            .checked_add(plan_nanos)
            .ok_or_else(|| Error::invariant("WAL group-encode timing overflow"))?;
        let (reports, next_lsn, next_batch_id) = if injector.is_some() {
            self.append_prepared_fault_injectable(commits, &mut injector)?
        } else {
            let encode_started = Instant::now();
            let encoded = self.encode_prepared_group(commits)?;
            self.group_encode_nanos = self
                .group_encode_nanos
                .checked_add(elapsed_nanos(encode_started)?)
                .ok_or_else(|| Error::invariant("WAL group-encode timing overflow"))?;
            self.add_attribution(&encoded.attribution)?;
            let write_started = Instant::now();
            let write_result = (|| {
                let offset = self.file.len()?;
                write_all_at_counted(
                    &mut self.file,
                    offset,
                    &encoded.bytes,
                    &mut self.physical_write_calls,
                )
            })();
            self.group_write_nanos = self
                .group_write_nanos
                .checked_add(elapsed_nanos(write_started)?)
                .ok_or_else(|| Error::invariant("WAL group-write timing overflow"))?;
            write_result?;
            (encoded.reports, encoded.next_lsn, encoded.next_batch_id)
        };
        let replayable_pages = commits
            .iter()
            .map(|commit| commit.records.len())
            .try_fold(0usize, |count, records| count.checked_add(records))
            .ok_or_else(|| Error::invariant("WAL record count overflow"))?;
        self.finish_group_append(
            append_started,
            &mut injector,
            GroupAppendOutcome {
                last_commit_lsn: commits.last().expect("non-empty WAL group").commit_lsn,
                next_lsn,
                next_batch_id,
                committed_batches: commits.len(),
                replayable_pages,
                stats,
                chain_updates,
            },
        )?;
        Ok(reports)
    }

    fn check_prepared_chain(
        &self,
        commits: &[PreparedWalCommit<'_>],
        strict: bool,
    ) -> Result<(Vec<(PageId, PageChainEntry)>, WalRedoStats)> {
        let mut group_chain: HashMap<PageId, PageChainEntry> = HashMap::new();
        let mut chain_updates = Vec::new();
        let mut stats = WalRedoStats::default();
        let mut next_lsn = self.next_lsn;
        let mut next_batch_id = self.next_batch_id;
        for commit in commits {
            if commit.records.is_empty() {
                return Err(Error::invalid_input(
                    "a WAL commit must contain at least one page image",
                ));
            }
            if commit.batch_id != next_batch_id {
                return Err(Error::invariant(format!(
                    "WAL batch id {}, expected {}",
                    commit.batch_id, next_batch_id
                )));
            }
            let expected_commit_lsn = next_lsn
                .get()
                .checked_add(commit.records.len() as u64)
                .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
            if commit.commit_lsn.get() != expected_commit_lsn {
                return Err(Error::invariant(format!(
                    "WAL commit LSN {}, expected {}",
                    commit.commit_lsn.get(),
                    expected_commit_lsn
                )));
            }
            let mut previous_page_id = None;
            for record in &commit.records {
                if is_superblock_page(record.page_id) {
                    return Err(Error::invariant(
                        "prepared page redo cannot carry a superblock",
                    ));
                }
                if previous_page_id.is_some_and(|previous| previous >= record.page_id) {
                    return Err(Error::invariant(
                        "prepared page redo records are not in page order",
                    ));
                }
                previous_page_id = Some(record.page_id);
                if record.page_lsn != commit.commit_lsn {
                    return Err(Error::corruption(format!(
                        "WAL page {} has page LSN {}, commit LSN {}",
                        record.page_id, record.page_lsn, commit.commit_lsn
                    )));
                }
                let known = group_chain
                    .get(&record.page_id)
                    .or_else(|| self.page_chain.get(&record.page_id))
                    .copied();
                match &record.redo {
                    PreparedWalRedo::Image(image) => {
                        if page_lsn_of(image) != record.page_lsn {
                            return Err(Error::corruption(format!(
                                "WAL page {} image LSN does not match its record",
                                record.page_id
                            )));
                        }
                        if strict {
                            crate::blink::validate_blink_page_image(&image[..], record.page_id)?;
                            if crc32c::crc32c(&image[..]) != record.image_crc {
                                return Err(Error::invariant(format!(
                                    "WAL page {} image CRC does not match its record",
                                    record.page_id
                                )));
                            }
                        }
                        if known.is_some() {
                            stats.image_not_smaller += 1;
                        } else {
                            stats.image_first_touch += 1;
                        }
                        stats.page_image_records += 1;
                    }
                    PreparedWalRedo::Delta {
                        payload,
                        base_lsn,
                        base_crc,
                        spans,
                        changed_bytes,
                    } => {
                        if payload.len() < PAGE_DELTA_HEADER_SIZE
                            || payload.len() >= PAGE_IMAGE_PAYLOAD_SIZE
                        {
                            return Err(Error::invariant(format!(
                                "page {} prepared delta has {} bytes",
                                record.page_id,
                                payload.len()
                            )));
                        }
                        let payload_page_id =
                            PageId::new(u64::from_le_bytes(payload[0..8].try_into().unwrap()));
                        let payload_base_lsn =
                            Lsn::new(u64::from_le_bytes(payload[8..16].try_into().unwrap()));
                        if payload_page_id != record.page_id || payload_base_lsn != *base_lsn {
                            return Err(Error::invariant(format!(
                                "page {} prepared delta header does not match its record",
                                record.page_id
                            )));
                        }
                        match known {
                            Some(entry)
                                if entry.page_lsn == *base_lsn && entry.image_crc == *base_crc => {}
                            Some(entry) => {
                                return Err(Error::invariant(format!(
                                    "page {} delta base does not match the WAL page chain (chain LSN {}, base LSN {})",
                                    record.page_id, entry.page_lsn, base_lsn
                                )));
                            }
                            None => {
                                return Err(Error::invariant(format!(
                                    "page {} delta has no WAL page-chain base",
                                    record.page_id
                                )));
                            }
                        }
                        if strict {
                            let view = decode_page_delta(payload)?;
                            if view.spans.len() as u64 != *spans
                                || view.changed_bytes() as u64 != *changed_bytes
                            {
                                return Err(Error::invariant(format!(
                                    "page {} prepared delta counters do not match its payload",
                                    record.page_id
                                )));
                            }
                        }
                        stats.page_delta_records += 1;
                        stats.page_delta_payload_bytes += payload.len() as u64;
                        stats.page_delta_spans += *spans;
                        stats.page_delta_changed_bytes += *changed_bytes;
                    }
                }
                let entry = PageChainEntry {
                    page_lsn: record.page_lsn,
                    image_crc: record.image_crc,
                };
                group_chain.insert(record.page_id, entry);
                chain_updates.push((record.page_id, entry));
            }
            next_lsn = Lsn::new(
                expected_commit_lsn
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            next_batch_id = next_batch_id
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL batch id exhausted"))?;
        }
        Ok((chain_updates, stats))
    }

    fn encode_prepared_group(&self, commits: &[PreparedWalCommit<'_>]) -> Result<EncodedWalGroup> {
        let mut attribution = WalEncodeAttribution::default();
        let mut capacity = 0usize;
        for commit in commits {
            for record in &commit.records {
                let payload_length = match &record.redo {
                    PreparedWalRedo::Image(_) => PAGE_IMAGE_PAYLOAD_SIZE,
                    PreparedWalRedo::Delta { payload, .. } => payload.len(),
                };
                capacity = capacity
                    .checked_add(payload_length + WAL_HEADER_SIZE + WAL_TRAILER_SIZE)
                    .ok_or_else(|| Error::invalid_input("WAL group size overflows"))?;
            }
            capacity = capacity
                .checked_add(WAL_HEADER_SIZE + COMMIT_PAYLOAD_SIZE + WAL_TRAILER_SIZE)
                .ok_or_else(|| Error::invalid_input("WAL group size overflows"))?;
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve(capacity)
            .map_err(|_| Error::invalid_input("WAL group buffer is too large"))?;
        let mut reports = Vec::with_capacity(commits.len());
        let mut next_lsn = self.next_lsn;
        let mut next_batch_id = self.next_batch_id;
        for commit in commits {
            let first_record_lsn = next_lsn;
            let mut bytes_written = 0usize;
            let mut digest = 0u32;
            for (record_position, record) in commit.records.iter().enumerate() {
                let page_id_bytes = record.page_id.get().to_le_bytes();
                let record_index = u32::try_from(record_position)
                    .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
                let (record_type, payload_parts): (WalRecordType, [&[u8]; 2]) = match &record.redo {
                    PreparedWalRedo::Image(image) => {
                        (WalRecordType::PageImage, [&page_id_bytes, &image[..]])
                    }
                    PreparedWalRedo::Delta { payload, .. } => {
                        (WalRecordType::PageDelta, [payload, &[]])
                    }
                };
                let payload_crc_started = Instant::now();
                let payload_checksum = crc32c::crc32c(payload_parts[0]);
                let payload_checksum = crc32c::crc32c_append(payload_checksum, payload_parts[1]);
                attribution.group_page_payload_crc_nanos += elapsed_nanos(payload_crc_started)?;
                let digest_started = Instant::now();
                digest = update_record_digest(
                    digest,
                    self.format_version,
                    record_type,
                    record_index,
                    &payload_parts,
                );
                attribution.group_commit_digest_crc_nanos += elapsed_nanos(digest_started)?;
                let record_lsn = first_record_lsn
                    .get()
                    .checked_add(record_position as u64)
                    .map(Lsn::new)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
                let frame_length = append_frame_parts_direct(
                    &mut bytes,
                    self.format_version,
                    record_type,
                    record_lsn,
                    commit.batch_id,
                    record_index,
                    payload_parts[0].len() + payload_parts[1].len(),
                    payload_checksum,
                    &payload_parts,
                    &mut attribution,
                )?;
                bytes_written = bytes_written
                    .checked_add(frame_length)
                    .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
                attribution.group_page_frames += 1;
            }
            let commit_payload =
                prepared_commit_payload(first_record_lsn, commit.records.len(), digest)?;
            let commit_record_index = u32::try_from(commit.records.len())
                .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
            let commit_frame_length = append_frame_direct(
                &mut bytes,
                self.format_version,
                WalRecordType::Commit,
                commit.commit_lsn,
                commit.batch_id,
                commit_record_index,
                &commit_payload,
                &mut attribution,
            )?;
            bytes_written = bytes_written
                .checked_add(commit_frame_length)
                .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
            attribution.group_commit_frames += 1;
            reports.push(WalAppendReport {
                first_record_lsn,
                commit_lsn: commit.commit_lsn,
                page_count: commit.records.len(),
                bytes_written,
            });
            next_lsn = Lsn::new(
                commit
                    .commit_lsn
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            next_batch_id = next_batch_id
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL batch id exhausted"))?;
        }
        Ok(EncodedWalGroup {
            bytes,
            reports,
            next_lsn,
            next_batch_id,
            attribution,
        })
    }

    fn append_prepared_fault_injectable(
        &mut self,
        commits: &[PreparedWalCommit<'_>],
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<(Vec<WalAppendReport>, Lsn, u64)> {
        let mut next_lsn = self.next_lsn;
        let mut next_batch_id = self.next_batch_id;
        let mut reports = Vec::with_capacity(commits.len());
        for commit in commits {
            let first_record_lsn = next_lsn;
            let mut digest = 0u32;
            let mut bytes_written = 0usize;
            for (record_position, record) in commit.records.iter().enumerate() {
                let (record_type, payload) = match &record.redo {
                    PreparedWalRedo::Image(image) => {
                        let mut payload = Vec::with_capacity(PAGE_IMAGE_PAYLOAD_SIZE);
                        payload.extend_from_slice(&record.page_id.get().to_le_bytes());
                        payload.extend_from_slice(&image[..]);
                        (WalRecordType::PageImage, payload)
                    }
                    PreparedWalRedo::Delta { payload, .. } => {
                        (WalRecordType::PageDelta, payload.to_vec())
                    }
                };
                let record_index = u32::try_from(record_position)
                    .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
                digest = update_record_digest(
                    digest,
                    self.format_version,
                    record_type,
                    record_index,
                    &[&payload],
                );
                let record_lsn = Lsn::new(
                    first_record_lsn
                        .get()
                        .checked_add(record_position as u64)
                        .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
                );
                bytes_written = bytes_written
                    .checked_add(self.append_frame(
                        record_type,
                        record_lsn,
                        commit.batch_id,
                        record_index,
                        &payload,
                        &mut *injector,
                    )?)
                    .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
                if record_type == WalRecordType::PageDelta {
                    hit(&mut *injector, "after_page_delta_record")?;
                }
            }
            hit(&mut *injector, "after_page_images_written")?;
            let commit_payload =
                prepared_commit_payload(first_record_lsn, commit.records.len(), digest)?;
            hit(&mut *injector, "before_commit_record")?;
            bytes_written = bytes_written
                .checked_add(
                    self.append_frame(
                        WalRecordType::Commit,
                        commit.commit_lsn,
                        commit.batch_id,
                        u32::try_from(commit.records.len())
                            .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?,
                        &commit_payload,
                        &mut *injector,
                    )?,
                )
                .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
            hit(&mut *injector, "after_commit_record_write")?;
            reports.push(WalAppendReport {
                first_record_lsn,
                commit_lsn: commit.commit_lsn,
                page_count: commit.records.len(),
                bytes_written,
            });
            next_lsn = Lsn::new(
                commit
                    .commit_lsn
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            next_batch_id = next_batch_id
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL batch id exhausted"))?;
        }
        Ok((reports, next_lsn, next_batch_id))
    }

    pub(crate) fn page_chain_entry(&self, page_id: PageId) -> Option<(Lsn, u32)> {
        self.page_chain
            .get(&page_id)
            .map(|entry| (entry.page_lsn, entry.image_crc))
    }

    fn plan_redo(
        &self,
        commits: &[WalCommit],
        mut delta: Option<&mut WalDeltaRequest<'_, '_, '_>>,
    ) -> Result<RedoPlan> {
        if let Some(request) = delta.as_deref()
            && request.eligible_commits.len() != commits.len()
        {
            return Err(Error::invalid_input(
                "WAL page-delta eligibility must name every commit",
            ));
        }
        let track_chain = self.page_image_format != WalPageImageFormat::Baseline;
        let delta_format = self.page_delta_enabled();
        let mut group_chain: HashMap<PageId, (PageChainEntry, &[u8; PAGE_SIZE])> = HashMap::new();
        let mut records = Vec::with_capacity(commits.len());
        let mut chain_updates = Vec::new();
        let mut stats = WalRedoStats::default();
        for (commit_index, commit) in commits.iter().enumerate() {
            let mut commit_records = Vec::with_capacity(commit.pages.len());
            for page in &commit.pages {
                let superblock = is_superblock_page(page.page_id);
                let planned = if superblock {
                    stats.image_superblock += 1;
                    PlannedRedo::Image
                } else if let Some(request) = delta.as_deref_mut() {
                    if !delta_format {
                        stats.image_page_image_format += 1;
                        PlannedRedo::Image
                    } else if !request.eligible_commits[commit_index] {
                        stats.image_ineligible_commit += 1;
                        PlannedRedo::Image
                    } else {
                        let known = match group_chain.get(&page.page_id) {
                            Some((entry, image)) => Some((*entry, Some(*image))),
                            None => self
                                .page_chain
                                .get(&page.page_id)
                                .map(|entry| (*entry, None)),
                        };
                        match known {
                            None => {
                                stats.image_first_touch += 1;
                                PlannedRedo::Image
                            }
                            Some((entry, group_base)) => {
                                let source_base;
                                let base = match group_base {
                                    Some(image) => Some(image),
                                    None => {
                                        source_base = (request.base_source)(page.page_id);
                                        source_base.as_deref()
                                    }
                                };
                                match base {
                                    None => {
                                        stats.image_no_base += 1;
                                        PlannedRedo::Image
                                    }
                                    Some(base) => {
                                        self.plan_page_delta(page, base, entry, &mut stats)?
                                    }
                                }
                            }
                        }
                    }
                } else {
                    stats.image_not_requested += 1;
                    PlannedRedo::Image
                };
                if matches!(planned, PlannedRedo::Image) {
                    stats.page_image_records += 1;
                }
                if track_chain && !superblock {
                    let entry = PageChainEntry {
                        page_lsn: page_lsn_of(&page.image),
                        image_crc: crc32c::crc32c(&page.image[..]),
                    };
                    group_chain.insert(page.page_id, (entry, &page.image));
                    chain_updates.push((page.page_id, entry));
                }
                commit_records.push(planned);
            }
            records.push(commit_records);
        }
        Ok(RedoPlan {
            records,
            chain_updates,
            stats,
        })
    }

    fn plan_page_delta(
        &self,
        page: &WalPageImage,
        base: &[u8; PAGE_SIZE],
        entry: PageChainEntry,
        stats: &mut WalRedoStats,
    ) -> Result<PlannedRedo> {
        if page_lsn_of(base) != entry.page_lsn || crc32c::crc32c(&base[..]) != entry.image_crc {
            return Err(Error::invariant(format!(
                "page {} delta base does not match the WAL page chain (chain LSN {}, base LSN {})",
                page.page_id,
                entry.page_lsn,
                page_lsn_of(base)
            )));
        }
        let payload = encode_page_delta(page.page_id, base, &page.image)?;
        if payload.len() >= PAGE_IMAGE_PAYLOAD_SIZE {
            stats.image_not_smaller += 1;
            return Ok(PlannedRedo::Image);
        }
        let view = decode_page_delta(&payload)?;
        if apply_page_delta(base, &view)? != page.image {
            return Err(Error::invariant(format!(
                "page {} delta does not rebuild its after-image",
                page.page_id
            )));
        }
        stats.page_delta_records += 1;
        stats.page_delta_payload_bytes += payload.len() as u64;
        stats.page_delta_spans += view.spans.len() as u64;
        stats.page_delta_changed_bytes += view.changed_bytes() as u64;
        drop(view);
        Ok(PlannedRedo::Delta(payload))
    }

    fn append_group_fault_injectable(
        &mut self,
        commits: &[WalCommit],
        plan: &RedoPlan,
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<(Vec<WalAppendReport>, Lsn, u64)> {
        let mut next_lsn = self.next_lsn;
        let mut next_batch_id = self.next_batch_id;
        let mut reports = Vec::with_capacity(commits.len());
        for (commit, records) in commits.iter().zip(&plan.records) {
            let report =
                self.append_group_commit(commit, records, next_lsn, next_batch_id, injector)?;
            next_lsn = Lsn::new(
                report
                    .commit_lsn
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            next_batch_id = next_batch_id
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL batch id exhausted"))?;
            reports.push(report);
        }
        Ok((reports, next_lsn, next_batch_id))
    }

    fn encode_group(
        &self,
        commits: &[WalCommit],
        validation_mode: PageImageValidationMode,
        plan: &RedoPlan,
    ) -> Result<EncodedWalGroup> {
        let mut attribution = WalEncodeAttribution::default();
        let mut capacity = 0usize;
        for (commit, records) in commits.iter().zip(&plan.records) {
            if commit.pages.is_empty() {
                return Err(Error::invalid_input(
                    "a WAL commit must contain at least one page image",
                ));
            }
            let page_frames = records
                .iter()
                .map(|record| match record {
                    PlannedRedo::Image => PAGE_IMAGE_PAYLOAD_SIZE,
                    PlannedRedo::Delta(payload) => payload.len(),
                } + WAL_HEADER_SIZE
                    + WAL_TRAILER_SIZE)
                .try_fold(0usize, |total, size| total.checked_add(size))
                .ok_or_else(|| Error::invalid_input("WAL group size overflows"))?;
            let commit_frame = WAL_HEADER_SIZE
                .checked_add(COMMIT_PAYLOAD_SIZE)
                .and_then(|size| size.checked_add(WAL_TRAILER_SIZE))
                .ok_or_else(|| Error::invalid_input("WAL group size overflows"))?;
            capacity = capacity
                .checked_add(page_frames)
                .and_then(|size| size.checked_add(commit_frame))
                .ok_or_else(|| Error::invalid_input("WAL group size overflows"))?;
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve(capacity)
            .map_err(|_| Error::invalid_input("WAL group buffer is too large"))?;
        let mut reports = Vec::with_capacity(commits.len());
        let mut next_lsn = self.next_lsn;
        let mut next_batch_id = self.next_batch_id;

        for (commit, records) in commits.iter().zip(&plan.records) {
            let first_record_lsn = next_lsn;
            if commit.batch_id != next_batch_id {
                return Err(Error::invariant(format!(
                    "WAL batch id {}, expected {}",
                    commit.batch_id, next_batch_id
                )));
            }
            let page_count = u64::try_from(commit.pages.len())
                .map_err(|_| Error::invalid_input("WAL page count does not fit u64"))?;
            let expected_commit_lsn = first_record_lsn
                .get()
                .checked_add(page_count)
                .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
            if commit.commit_lsn.get() != expected_commit_lsn {
                return Err(Error::invariant(format!(
                    "WAL commit LSN {}, expected {}",
                    commit.commit_lsn.get(),
                    expected_commit_lsn
                )));
            }

            let mut bytes_written = 0usize;
            let mut digest = 0u32;
            for (page_index, (page, record)) in commit.pages.iter().zip(records).enumerate() {
                let page_lsn_validation_started = Instant::now();
                validate_page_image_lsn(page, commit.commit_lsn)?;
                attribution.group_page_lsn_validate_nanos +=
                    elapsed_nanos(page_lsn_validation_started)?;
                if validation_mode == PageImageValidationMode::Strict || cfg!(debug_assertions) {
                    let validation_started = Instant::now();
                    validate_page_image(page, self.page_image_format)?;
                    attribution.group_page_image_validate_nanos +=
                        elapsed_nanos(validation_started)?;
                    attribution.group_page_validations += 1;
                }
                let page_id_bytes = page.page_id.get().to_le_bytes();
                let record_index = u32::try_from(page_index)
                    .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
                let (record_type, payload_parts): (WalRecordType, [&[u8]; 2]) = match record {
                    PlannedRedo::Image => (WalRecordType::PageImage, [&page_id_bytes, &page.image]),
                    PlannedRedo::Delta(payload) => (WalRecordType::PageDelta, [payload, &[]]),
                };
                let payload_crc_started = Instant::now();
                let payload_checksum = crc32c::crc32c(payload_parts[0]);
                let payload_checksum = crc32c::crc32c_append(payload_checksum, payload_parts[1]);
                attribution.group_page_payload_crc_nanos += elapsed_nanos(payload_crc_started)?;
                let digest_started = Instant::now();
                digest = update_record_digest(
                    digest,
                    self.format_version,
                    record_type,
                    record_index,
                    &payload_parts,
                );
                attribution.group_commit_digest_crc_nanos += elapsed_nanos(digest_started)?;
                let record_lsn = first_record_lsn
                    .get()
                    .checked_add(
                        u64::try_from(page_index)
                            .map_err(|_| Error::invalid_input("WAL record index overflows"))?,
                    )
                    .map(Lsn::new)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
                let frame_length = append_frame_parts_direct(
                    &mut bytes,
                    self.format_version,
                    record_type,
                    record_lsn,
                    commit.batch_id,
                    record_index,
                    payload_parts[0].len() + payload_parts[1].len(),
                    payload_checksum,
                    &payload_parts,
                    &mut attribution,
                )?;
                bytes_written = bytes_written
                    .checked_add(frame_length)
                    .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
                attribution.group_page_frames += 1;
            }

            let mut commit_payload = [0u8; COMMIT_PAYLOAD_SIZE];
            commit_payload[0..8].copy_from_slice(&first_record_lsn.get().to_le_bytes());
            commit_payload[8..12].copy_from_slice(
                &u32::try_from(commit.pages.len())
                    .map_err(|_| Error::invalid_input("WAL page count does not fit u32"))?
                    .to_le_bytes(),
            );
            commit_payload[12..16].copy_from_slice(&digest.to_le_bytes());
            let commit_record_index = u32::try_from(commit.pages.len())
                .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
            let commit_frame_length = append_frame_direct(
                &mut bytes,
                self.format_version,
                WalRecordType::Commit,
                commit.commit_lsn,
                commit.batch_id,
                commit_record_index,
                &commit_payload,
                &mut attribution,
            )?;
            bytes_written = bytes_written
                .checked_add(commit_frame_length)
                .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
            attribution.group_commit_frames += 1;
            reports.push(WalAppendReport {
                first_record_lsn,
                commit_lsn: commit.commit_lsn,
                page_count: commit.pages.len(),
                bytes_written,
            });
            next_lsn = Lsn::new(
                commit
                    .commit_lsn
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            next_batch_id = next_batch_id
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL batch id exhausted"))?;
        }

        Ok(EncodedWalGroup {
            bytes,
            reports,
            next_lsn,
            next_batch_id,
            attribution,
        })
    }

    fn append_group_commit(
        &mut self,
        commit: &WalCommit,
        records: &[PlannedRedo],
        first_record_lsn: Lsn,
        expected_batch_id: u64,
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<WalAppendReport> {
        let pages = &commit.pages;
        if pages.is_empty() {
            return Err(Error::invalid_input(
                "a WAL commit must contain at least one page image",
            ));
        }
        if commit.batch_id != expected_batch_id {
            return Err(Error::invariant(format!(
                "WAL batch id {}, expected {}",
                commit.batch_id, expected_batch_id
            )));
        }
        let page_count = u64::try_from(pages.len())
            .map_err(|_| Error::invalid_input("WAL page count does not fit u64"))?;
        let expected_commit = first_record_lsn
            .get()
            .checked_add(page_count)
            .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
        if commit.commit_lsn.get() != expected_commit {
            return Err(Error::invariant(format!(
                "WAL commit LSN {}, expected {}",
                commit.commit_lsn.get(),
                expected_commit
            )));
        }

        let mut digest = 0u32;
        let mut bytes_written = 0usize;
        for (index, (page, record)) in pages.iter().zip(records).enumerate() {
            validate_page_image_lsn(page, commit.commit_lsn)?;
            let (record_type, payload) = match record {
                PlannedRedo::Image => (
                    WalRecordType::PageImage,
                    encode_page_image(page, self.page_image_format)?,
                ),
                PlannedRedo::Delta(payload) => {
                    validate_page_image(page, self.page_image_format)?;
                    (WalRecordType::PageDelta, payload.clone())
                }
            };
            let record_index = u32::try_from(index)
                .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
            digest = update_record_digest(
                digest,
                self.format_version,
                record_type,
                record_index,
                &[&payload],
            );
            let record_lsn = Lsn::new(
                first_record_lsn
                    .get()
                    .checked_add(index as u64)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            bytes_written = bytes_written
                .checked_add(self.append_frame(
                    record_type,
                    record_lsn,
                    commit.batch_id,
                    record_index,
                    &payload,
                    &mut *injector,
                )?)
                .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
            if record_type == WalRecordType::PageDelta {
                hit(&mut *injector, "after_page_delta_record")?;
            }
        }
        hit(&mut *injector, "after_page_images_written")?;

        let mut commit_payload = [0u8; COMMIT_PAYLOAD_SIZE];
        commit_payload[0..8].copy_from_slice(&first_record_lsn.get().to_le_bytes());
        commit_payload[8..12].copy_from_slice(
            &u32::try_from(pages.len())
                .map_err(|_| Error::invalid_input("WAL page count does not fit u32"))?
                .to_le_bytes(),
        );
        commit_payload[12..16].copy_from_slice(&u32::to_le_bytes(digest));
        hit(&mut *injector, "before_commit_record")?;
        bytes_written = bytes_written
            .checked_add(
                self.append_frame(
                    WalRecordType::Commit,
                    commit.commit_lsn,
                    commit.batch_id,
                    u32::try_from(pages.len())
                        .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?,
                    &commit_payload,
                    &mut *injector,
                )?,
            )
            .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
        hit(&mut *injector, "after_commit_record_write")?;

        Ok(WalAppendReport {
            first_record_lsn,
            commit_lsn: commit.commit_lsn,
            page_count: pages.len(),
            bytes_written,
        })
    }

    fn append_frame(
        &mut self,
        record_type: WalRecordType,
        record_lsn: Lsn,
        batch_id: u64,
        record_index: u32,
        payload: &[u8],
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<usize> {
        let frame = encode_frame(
            self.format_version,
            record_type,
            record_lsn,
            batch_id,
            record_index,
            payload,
        )?;
        let frame_length = frame.len();
        let payload_end = WAL_HEADER_SIZE + payload.len();

        let offset = self.file.len()?;
        hit(injector, "during_wal_header_write")?;
        if record_type == WalRecordType::PageDelta {
            hit(injector, "during_page_delta_header_write")?;
        }
        write_all_at_counted(
            &mut self.file,
            offset,
            &frame[..WAL_HEADER_SIZE],
            &mut self.physical_write_calls,
        )?;
        hit(injector, "during_wal_payload_write")?;
        if record_type == WalRecordType::PageDelta {
            hit(injector, "during_page_delta_payload_write")?;
        }
        write_all_at_counted(
            &mut self.file,
            offset
                .checked_add(WAL_HEADER_SIZE as u64)
                .ok_or_else(|| Error::invalid_input("WAL offset overflows"))?,
            &frame[WAL_HEADER_SIZE..payload_end],
            &mut self.physical_write_calls,
        )?;
        hit(injector, "during_wal_trailer_write")?;
        write_all_at_counted(
            &mut self.file,
            offset
                .checked_add(payload_end as u64)
                .ok_or_else(|| Error::invalid_input("WAL offset overflows"))?,
            &frame[payload_end..],
            &mut self.physical_write_calls,
        )?;
        Ok(frame_length)
    }

    fn identity_payload(&self, start_after_lsn: Lsn) -> [u8; INIT_PAYLOAD_SIZE] {
        identity_payload(&self.identity, start_after_lsn)
    }
}

struct LogicalWalScanOutput {
    next_lsn: Lsn,
    next_batch_id: u64,
    history_start_lsn: Lsn,
    last_commit_sequence: Option<u64>,
    history_start_sequence: u64,
    last_commit_lsn: Option<Lsn>,
    transactions: Vec<CommittedLogicalWalTransaction>,
    report: WalScanReport,
    valid_length: u64,
}

struct PendingLogicalTransaction {
    batch_id: u64,
    first_record_lsn: Lsn,
    start_offset: u64,
    mutations: Vec<LogicalWalMutation>,
    digest: u32,
}

pub struct LogicalWalLog<F: DurableFile> {
    file: F,
    identity: WalIdentity,
    next_lsn: Lsn,
    next_batch_id: u64,
    history_start_lsn: Lsn,
    history_start_sequence: u64,
    last_commit_sequence: Option<u64>,
    last_commit_lsn: Option<Lsn>,
    recovery_transactions: Vec<CommittedLogicalWalTransaction>,
    scan_report: WalScanReport,
    sync_count: u64,
    appended_transactions: u64,
}

impl<F: DurableFile> LogicalWalLog<F> {
    pub fn open(
        file: F,
        identity: WalIdentity,
        start_after_lsn: Lsn,
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Self> {
        Self::open_with_checkpoint(
            file,
            identity,
            start_after_lsn,
            start_after_lsn.get(),
            injector,
        )
    }

    pub fn open_with_checkpoint(
        mut file: F,
        identity: WalIdentity,
        start_after_lsn: Lsn,
        start_after_sequence: u64,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Self> {
        let length = file.len()?;
        if length == 0 {
            return Self::initialize_empty(file, identity, start_after_lsn, start_after_sequence);
        }
        if length < LOGICAL_INIT_FRAME_SIZE as u64 {
            let expected_init = encode_frame(
                WAL_LOGICAL_FORMAT_VERSION,
                WalRecordType::Init,
                Lsn::ZERO,
                0,
                0,
                &logical_identity_payload(&identity, start_after_lsn, start_after_sequence),
            )?;
            let previous_init = encode_frame(
                WAL_LOGICAL_FORMAT_VERSION,
                WalRecordType::Init,
                Lsn::ZERO,
                0,
                0,
                &identity_payload(&identity, start_after_lsn),
            )?;
            let prefix = read_exact_at(&mut file, 0, length as usize)?;
            let matches_new_init = prefix == expected_init[..length as usize];
            let matches_previous_init =
                length <= INIT_FRAME_SIZE as u64 && prefix == previous_init[..length as usize];
            let has_complete_previous_init = length >= INIT_FRAME_SIZE as u64
                && prefix[..INIT_FRAME_SIZE] == previous_init[..INIT_FRAME_SIZE];
            if !matches_new_init && !matches_previous_init && !has_complete_previous_init {
                return Err(Error::corruption(
                    "truncated WAL prefix does not match logical INIT",
                ));
            }
            if matches_new_init || matches_previous_init {
                file.set_len(0).map_err(|error| {
                    Error::recovery(format!("WAL torn INIT reset failed: {error}"))
                })?;
                file.sync_data().map_err(|error| {
                    Error::durability(format!("WAL torn INIT reset sync failed: {error}"))
                })?;
                return Self::initialize_empty(
                    file,
                    identity,
                    start_after_lsn,
                    start_after_sequence,
                );
            }
        }
        let LogicalWalScanOutput {
            next_lsn,
            next_batch_id,
            history_start_lsn,
            history_start_sequence,
            last_commit_sequence,
            last_commit_lsn,
            transactions,
            mut report,
            valid_length,
        } = scan_logical_wal(&mut file, &identity)?;
        if history_start_lsn > start_after_lsn
            || history_start_lsn == start_after_lsn
                && history_start_sequence != start_after_sequence
        {
            return Err(Error::corruption(
                "logical WAL checkpoint boundary does not match the durable superblock",
            ));
        }
        if valid_length < length {
            hit(&mut injector, "before_wal_tail_truncate")?;
            file.set_len(valid_length)?;
            file.sync_data().map_err(|error| {
                Error::durability(format!("WAL torn-tail truncation sync failed: {error}"))
            })?;
            hit(&mut injector, "after_wal_tail_truncate")?;
        }
        report.torn_tail_bytes = usize::try_from(length - valid_length)
            .map_err(|_| Error::invariant("WAL tail length does not fit usize"))?;
        Ok(Self {
            file,
            identity,
            next_lsn,
            next_batch_id,
            history_start_lsn,
            history_start_sequence,
            last_commit_sequence,
            last_commit_lsn,
            recovery_transactions: transactions,
            scan_report: report,
            sync_count: 0,
            appended_transactions: 0,
        })
    }

    fn initialize_empty(
        file: F,
        identity: WalIdentity,
        start_after_lsn: Lsn,
        start_after_sequence: u64,
    ) -> Result<Self> {
        let next_lsn = start_after_lsn
            .get()
            .checked_add(1)
            .map(Lsn::new)
            .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
        let payload = logical_identity_payload(&identity, start_after_lsn, start_after_sequence);
        let frame = encode_frame(
            WAL_LOGICAL_FORMAT_VERSION,
            WalRecordType::Init,
            Lsn::ZERO,
            0,
            0,
            &payload,
        )?;
        let mut wal = Self {
            file,
            identity,
            next_lsn,
            next_batch_id: 1,
            history_start_lsn: start_after_lsn,
            history_start_sequence: start_after_sequence,
            last_commit_sequence: None,
            last_commit_lsn: None,
            recovery_transactions: Vec::new(),
            scan_report: WalScanReport::default(),
            sync_count: 0,
            appended_transactions: 0,
        };
        write_all_at_logical(&mut wal.file, 0, &frame)?;
        wal.file
            .sync_data()
            .map_err(|error| Error::durability(format!("initial WAL sync failed: {error}")))?;
        wal.sync_count = 1;
        wal.scan_report.records_scanned = 1;
        Ok(wal)
    }

    pub fn append_group(
        &mut self,
        transactions: &[LogicalWalTransaction],
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<LogicalWalAppendReport>> {
        if transactions.is_empty() {
            return Err(Error::invalid_input(
                "a WAL group must contain a transaction",
            ));
        }
        hit(&mut injector, "before_wal_append")?;
        let mut output = Vec::new();
        let existing_length = self.file.len()?;
        let mut reports = Vec::with_capacity(transactions.len());
        let mut next_lsn = self.next_lsn;
        let mut next_batch_id = self.next_batch_id;
        let mut expected_sequence = self
            .last_commit_sequence
            .unwrap_or(self.history_start_sequence)
            .checked_add(1)
            .ok_or_else(|| Error::invariant("logical WAL sequence exhausted"))?;
        for transaction in transactions {
            if transaction.mutations.is_empty() {
                return Err(Error::invalid_input(
                    "a logical WAL transaction has no mutations",
                ));
            }
            if transaction.commit_sequence != expected_sequence {
                return Err(Error::invariant(
                    "logical WAL commit sequence is not contiguous",
                ));
            }
            let first_record_lsn = next_lsn;
            let batch_id = next_batch_id;
            let mut digest = 0u32;
            let mut bytes_written = 0usize;
            for (record_position, mutation) in transaction.mutations.iter().enumerate() {
                let (record_type, payload) = encode_logical_mutation(mutation)?;
                let record_index = u32::try_from(record_position)
                    .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
                digest = update_record_digest(
                    digest,
                    WAL_LOGICAL_FORMAT_VERSION,
                    record_type,
                    record_index,
                    &[&payload],
                );
                let frame = encode_frame(
                    WAL_LOGICAL_FORMAT_VERSION,
                    record_type,
                    next_lsn,
                    batch_id,
                    record_index,
                    &payload,
                )?;
                ensure_logical_wal_capacity(existing_length, output.len(), frame.len())?;
                output.extend_from_slice(&frame);
                bytes_written = bytes_written
                    .checked_add(frame.len())
                    .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
                next_lsn = Lsn::new(
                    next_lsn
                        .get()
                        .checked_add(1)
                        .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
                );
            }
            let physical_commit_lsn = next_lsn;
            let mut payload = [0u8; 24];
            payload[0..8].copy_from_slice(&first_record_lsn.get().to_le_bytes());
            payload[8..12].copy_from_slice(
                &u32::try_from(transaction.mutations.len())
                    .map_err(|_| Error::invalid_input("WAL mutation count overflows u32"))?
                    .to_le_bytes(),
            );
            payload[12..16].copy_from_slice(&digest.to_le_bytes());
            payload[16..24].copy_from_slice(&transaction.commit_sequence.to_le_bytes());
            hit(&mut injector, "before_commit_record")?;
            let frame = encode_frame(
                WAL_LOGICAL_FORMAT_VERSION,
                WalRecordType::Commit,
                physical_commit_lsn,
                batch_id,
                u32::try_from(transaction.mutations.len())
                    .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?,
                &payload,
            )?;
            ensure_logical_wal_capacity(existing_length, output.len(), frame.len())?;
            output.extend_from_slice(&frame);
            bytes_written = bytes_written
                .checked_add(frame.len())
                .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
            next_lsn = Lsn::new(
                physical_commit_lsn
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            reports.push(LogicalWalAppendReport {
                first_record_lsn,
                physical_commit_lsn,
                commit_sequence: transaction.commit_sequence,
                record_count: transaction.mutations.len(),
                bytes_written,
            });
            next_batch_id = next_batch_id
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL batch ID exhausted"))?;
            expected_sequence = expected_sequence
                .checked_add(1)
                .ok_or_else(|| Error::invariant("logical WAL sequence exhausted"))?;
        }
        let offset = existing_length;
        hit(&mut injector, "before_group_records_write")?;
        write_all_at_logical(&mut self.file, offset, &output)
            .map_err(|error| Error::durability(format!("logical WAL append failed: {error}")))?;
        hit(&mut injector, "after_group_records_written")?;
        hit(&mut injector, "before_wal_sync")?;
        hit(&mut injector, "during_wal_sync")?;
        self.file
            .sync_data()
            .map_err(|error| Error::durability(format!("logical WAL sync failed: {error}")))?;
        hit(&mut injector, "after_wal_sync")?;
        self.sync_count = self
            .sync_count
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL sync count overflow"))?;
        self.appended_transactions = self
            .appended_transactions
            .saturating_add(transactions.len() as u64);
        self.scan_report.records_scanned = self
            .scan_report
            .records_scanned
            .saturating_add(output_frame_count(transactions));
        self.scan_report.committed_batches = self
            .scan_report
            .committed_batches
            .saturating_add(transactions.len());
        self.scan_report.replayable_pages = self.scan_report.replayable_pages.saturating_add(
            transactions
                .iter()
                .map(|transaction| transaction.mutations.len())
                .sum::<usize>(),
        );
        self.last_commit_sequence = transactions
            .last()
            .map(|transaction| transaction.commit_sequence);
        self.last_commit_lsn = reports.last().map(|report| report.physical_commit_lsn);
        self.next_lsn = next_lsn;
        self.next_batch_id = next_batch_id;
        Ok(reports)
    }

    pub fn take_recovery_transactions(&mut self) -> Vec<CommittedLogicalWalTransaction> {
        std::mem::take(&mut self.recovery_transactions)
    }

    pub fn last_commit_sequence(&self) -> Option<u64> {
        self.last_commit_sequence
    }

    pub fn next_commit_sequence(&self) -> u64 {
        self.last_commit_sequence
            .unwrap_or(self.history_start_sequence)
            .saturating_add(1)
    }

    pub fn checkpoint_sequence(&self) -> u64 {
        self.last_commit_sequence
            .unwrap_or(self.history_start_sequence)
    }

    pub fn last_commit_lsn(&self) -> Option<Lsn> {
        self.last_commit_lsn
    }

    pub fn next_lsn(&self) -> Lsn {
        self.next_lsn
    }

    pub fn next_batch_id(&self) -> u64 {
        self.next_batch_id
    }

    pub fn sync_count(&self) -> u64 {
        self.sync_count
    }

    pub fn appended_transaction_count(&self) -> u64 {
        self.appended_transactions
    }

    pub fn scan_report(&self) -> &WalScanReport {
        &self.scan_report
    }

    pub fn wal_bytes(&self) -> Result<u64> {
        self.file.len()
    }

    pub fn reset(
        &mut self,
        checkpoint_lsn: Lsn,
        checkpoint_sequence: u64,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        if checkpoint_lsn < self.history_start_lsn
            || checkpoint_sequence < self.history_start_sequence
        {
            return Err(Error::checkpoint(
                "logical WAL reset would move its history start backwards",
            ));
        }
        hit(&mut injector, "before_wal_reset")?;
        hit(&mut injector, "during_wal_truncate")?;
        self.file.set_len(0).map_err(|error| {
            Error::checkpoint(format!("logical WAL truncation failed: {error}"))
        })?;
        hit(&mut injector, "after_wal_truncate")?;
        hit(&mut injector, "before_wal_reset_truncate_sync")?;
        hit(&mut injector, "during_wal_reset_truncate_sync")?;
        self.file.sync_data().map_err(|error| {
            Error::checkpoint(format!("logical WAL truncation sync failed: {error}"))
        })?;
        hit(&mut injector, "after_wal_reset_truncate_sync")?;
        self.history_start_lsn = checkpoint_lsn;
        self.history_start_sequence = checkpoint_sequence;
        self.next_lsn = Lsn::new(
            checkpoint_lsn
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("logical WAL LSN exhausted"))?,
        );
        self.next_batch_id = 1;
        self.last_commit_sequence = None;
        self.last_commit_lsn = None;
        self.recovery_transactions.clear();
        self.scan_report = WalScanReport {
            records_scanned: 1,
            ..WalScanReport::default()
        };
        self.appended_transactions = 0;
        let payload = logical_identity_payload(&self.identity, checkpoint_lsn, checkpoint_sequence);
        let frame = encode_frame(
            WAL_LOGICAL_FORMAT_VERSION,
            WalRecordType::Init,
            Lsn::ZERO,
            0,
            0,
            &payload,
        )?;
        hit(&mut injector, "before_wal_reinitialization")?;
        write_all_at_logical(&mut self.file, 0, &frame)?;
        hit(&mut injector, "after_wal_reset_write")?;
        hit(&mut injector, "before_wal_reset_sync")?;
        hit(&mut injector, "during_wal_reset_sync")?;
        self.file.sync_data().map_err(|error| {
            Error::checkpoint(format!("logical WAL reset sync failed: {error}"))
        })?;
        self.sync_count = self.sync_count.saturating_add(1);
        hit(&mut injector, "after_wal_reset_sync")?;
        Ok(())
    }

    pub fn into_file(self) -> F {
        self.file
    }
}

fn ensure_logical_wal_capacity(
    existing_length: u64,
    pending_bytes: usize,
    frame_bytes: usize,
) -> Result<()> {
    let total_bytes = existing_length
        .checked_add(pending_bytes as u64)
        .and_then(|length| length.checked_add(frame_bytes as u64))
        .ok_or_else(|| {
            Error::overloaded("logical WAL byte limit reached; checkpoint is required")
        })?;
    if total_bytes > MAX_LOGICAL_WAL_BYTES {
        return Err(Error::overloaded(
            "logical WAL byte limit reached; checkpoint is required",
        ));
    }
    Ok(())
}

fn logical_identity_payload(
    identity: &WalIdentity,
    start_after_lsn: Lsn,
    start_after_sequence: u64,
) -> [u8; LOGICAL_INIT_PAYLOAD_SIZE] {
    let mut payload = [0u8; LOGICAL_INIT_PAYLOAD_SIZE];
    payload[..INIT_PAYLOAD_SIZE].copy_from_slice(&identity_payload(identity, start_after_lsn));
    payload[52..60].copy_from_slice(&start_after_sequence.to_le_bytes());
    payload
}

fn output_frame_count(transactions: &[LogicalWalTransaction]) -> usize {
    transactions
        .iter()
        .map(|transaction| transaction.mutations.len().saturating_add(1))
        .sum()
}

fn write_all_at_logical<F: DurableFile>(file: &mut F, offset: u64, bytes: &[u8]) -> Result<()> {
    let mut position = 0usize;
    while position < bytes.len() {
        let count = file.write_at(
            offset
                .checked_add(position as u64)
                .ok_or_else(|| Error::invalid_input("WAL write offset overflows"))?,
            &bytes[position..],
        )?;
        if count == 0 {
            return Err(Error::Io(std::io::Error::new(
                ErrorKind::WriteZero,
                "WAL returned a zero-byte write",
            )));
        }
        position += count;
    }
    Ok(())
}

fn encode_logical_mutation(mutation: &LogicalWalMutation) -> Result<(WalRecordType, Vec<u8>)> {
    let key_length = u32::try_from(mutation.encoded_key.len())
        .map_err(|_| Error::invalid_input("logical WAL key exceeds 4 GiB"))?;
    if key_length == 0 {
        return Err(Error::invalid_input("logical WAL key is empty"));
    }
    let mut payload = Vec::with_capacity(
        16usize
            .saturating_add(mutation.encoded_key.len())
            .saturating_add(mutation.value.as_ref().map_or(0, Vec::len)),
    );
    payload.extend_from_slice(&mutation.revision.get().to_le_bytes());
    payload.extend_from_slice(&key_length.to_le_bytes());
    let value_length = mutation
        .value
        .as_ref()
        .map_or(Ok(u32::MAX), |value| u32::try_from(value.len()))
        .map_err(|_| Error::invalid_input("logical WAL value exceeds 4 GiB"))?;
    payload.extend_from_slice(&value_length.to_le_bytes());
    payload.extend_from_slice(&mutation.encoded_key);
    if let Some(value) = &mutation.value {
        payload.extend_from_slice(value);
    }
    if payload.len() > WAL_MAX_PAYLOAD_SIZE {
        return Err(Error::invalid_input(
            "logical WAL mutation exceeds frame limit",
        ));
    }
    Ok((
        if mutation.value.is_some() {
            WalRecordType::LogicalPut
        } else {
            WalRecordType::LogicalDelete
        },
        payload,
    ))
}

fn decode_logical_mutation(
    record_type: WalRecordType,
    payload: &[u8],
) -> Result<LogicalWalMutation> {
    if payload.len() < 16 {
        return Err(Error::corruption(
            "logical WAL mutation header is truncated",
        ));
    }
    let revision = Revision::new(u64::from_le_bytes(payload[0..8].try_into().unwrap()));
    let key_length = u32::from_le_bytes(payload[8..12].try_into().unwrap()) as usize;
    let value_length = u32::from_le_bytes(payload[12..16].try_into().unwrap());
    let expected_length = 16usize
        .checked_add(key_length)
        .and_then(|length| {
            length.checked_add(if value_length == u32::MAX {
                0
            } else {
                value_length as usize
            })
        })
        .ok_or_else(|| Error::corruption("logical WAL mutation length overflows"))?;
    if key_length == 0 || payload.len() != expected_length {
        return Err(Error::corruption("logical WAL mutation length is invalid"));
    }
    let value = match (record_type, value_length) {
        (WalRecordType::LogicalPut, length) if length != u32::MAX => {
            Some(payload[16 + key_length..].to_vec())
        }
        (WalRecordType::LogicalDelete, u32::MAX) => None,
        _ => {
            return Err(Error::corruption(
                "logical WAL operation and payload disagree",
            ));
        }
    };
    Ok(LogicalWalMutation {
        encoded_key: payload[16..16 + key_length].to_vec(),
        value,
        revision,
    })
}

fn scan_logical_wal<F: DurableFile>(
    file: &mut F,
    identity: &WalIdentity,
) -> Result<LogicalWalScanOutput> {
    let length = file.len()?;
    let mut offset = 0u64;
    let mut previous_lsn = None;
    let mut report = WalScanReport::default();
    let mut history_start_lsn = None;
    let mut history_start_sequence = None;
    let mut pending: Option<PendingLogicalTransaction> = None;
    let mut transactions = Vec::new();
    let mut highest_batch_id = 0u64;
    let mut last_commit_sequence: Option<u64> = None;
    let mut last_commit_lsn = Lsn::ZERO;
    while offset < length {
        let remaining = length - offset;
        if remaining < WAL_HEADER_SIZE as u64 {
            break;
        }
        let header = read_exact_at(file, offset, WAL_HEADER_SIZE)?;
        let (version, record_type, frame_length, payload_length, raw_lsn, batch_id, record_index) =
            decode_header(&header)?;
        if version != WAL_LOGICAL_FORMAT_VERSION {
            return Err(Error::unsupported_format(format!(
                "WAL v{} is not a logical v4 stream",
                version
            )));
        }
        if frame_length < WAL_MIN_FRAME_SIZE
            || frame_length != WAL_HEADER_SIZE + payload_length + WAL_TRAILER_SIZE
            || payload_length > WAL_MAX_PAYLOAD_SIZE
        {
            return Err(Error::corruption("invalid logical WAL frame length"));
        }
        let frame_length_u64 = frame_length as u64;
        if frame_length_u64 > remaining {
            break;
        }
        let payload = read_exact_at(file, offset + WAL_HEADER_SIZE as u64, payload_length)?;
        let trailer = read_exact_at(
            file,
            offset + (WAL_HEADER_SIZE + payload_length) as u64,
            WAL_TRAILER_SIZE,
        )?;
        if u32::from_le_bytes(trailer.try_into().unwrap()) != frame_length as u32 {
            return Err(Error::corruption("logical WAL trailing length mismatch"));
        }
        if previous_lsn.is_some_and(|previous| raw_lsn <= previous) {
            return Err(Error::corruption("logical WAL frame LSN is not increasing"));
        }
        verify_payload_checksum(&header, &payload, offset)?;
        report.records_scanned = report.records_scanned.saturating_add(1);
        offset += frame_length_u64;
        let record_lsn = Lsn::new(raw_lsn);
        if history_start_lsn.is_none() {
            if record_type != WalRecordType::Init
                || record_lsn != Lsn::ZERO
                || batch_id != 0
                || record_index != 0
            {
                return Err(Error::corruption("logical WAL has no valid INIT frame"));
            }
            history_start_lsn = Some(verify_identity(version, &payload, identity)?);
            history_start_sequence = Some(if payload.len() == LOGICAL_INIT_PAYLOAD_SIZE {
                u64::from_le_bytes(payload[52..60].try_into().unwrap())
            } else {
                u64::from_le_bytes(payload[44..52].try_into().unwrap())
            });
            previous_lsn = Some(raw_lsn);
            continue;
        }
        let start_after = history_start_lsn.unwrap();
        let start_sequence = history_start_sequence.unwrap();
        if record_lsn <= start_after {
            return Err(Error::corruption(
                "logical WAL frame precedes its history boundary",
            ));
        }
        previous_lsn = Some(raw_lsn);
        match record_type {
            WalRecordType::LogicalPut | WalRecordType::LogicalDelete => {
                let mutation = decode_logical_mutation(record_type, &payload)?;
                if let Some(pending_transaction) = pending.as_ref() {
                    if pending_transaction.batch_id != batch_id {
                        return Err(Error::corruption(
                            "logical WAL transactions are interleaved",
                        ));
                    }
                } else {
                    if batch_id <= highest_batch_id {
                        return Err(Error::corruption(
                            "logical WAL batch IDs are not increasing",
                        ));
                    }
                    highest_batch_id = batch_id;
                    pending = Some(PendingLogicalTransaction {
                        batch_id,
                        first_record_lsn: record_lsn,
                        start_offset: offset - frame_length_u64,
                        mutations: Vec::new(),
                        digest: 0,
                    });
                }
                let pending_transaction = pending.as_mut().unwrap();
                if record_index != pending_transaction.mutations.len() as u32 {
                    return Err(Error::corruption(
                        "logical WAL record index is not contiguous",
                    ));
                }
                pending_transaction.digest = update_record_digest(
                    pending_transaction.digest,
                    version,
                    record_type,
                    record_index,
                    &[&payload],
                );
                pending_transaction.mutations.push(mutation);
            }
            WalRecordType::Commit => {
                let pending_transaction = pending.take().ok_or_else(|| {
                    Error::corruption("logical WAL commit has no mutation records")
                })?;
                if payload.len() != 24
                    || batch_id != pending_transaction.batch_id
                    || record_index != pending_transaction.mutations.len() as u32
                {
                    return Err(Error::corruption(
                        "logical WAL commit metadata is malformed",
                    ));
                }
                let first_record_lsn =
                    Lsn::new(u64::from_le_bytes(payload[0..8].try_into().unwrap()));
                let mutation_count =
                    u32::from_le_bytes(payload[8..12].try_into().unwrap()) as usize;
                let digest = u32::from_le_bytes(payload[12..16].try_into().unwrap());
                let commit_sequence = u64::from_le_bytes(payload[16..24].try_into().unwrap());
                if first_record_lsn != pending_transaction.first_record_lsn
                    || mutation_count != pending_transaction.mutations.len()
                    || digest != pending_transaction.digest
                    || pending_transaction
                        .mutations
                        .iter()
                        .any(|mutation| mutation.revision.get() != commit_sequence)
                    || last_commit_sequence
                        .is_some_and(|previous| previous.checked_add(1) != Some(commit_sequence))
                    || last_commit_sequence.is_none()
                        && commit_sequence != start_sequence.saturating_add(1)
                {
                    return Err(Error::corruption(
                        "logical WAL commit does not match its mutation records",
                    ));
                }
                transactions.push(CommittedLogicalWalTransaction {
                    batch_id,
                    physical_commit_lsn: record_lsn,
                    commit_sequence,
                    mutations: pending_transaction.mutations,
                });
                last_commit_sequence = Some(commit_sequence);
                last_commit_lsn = record_lsn;
                report.committed_batches = report.committed_batches.saturating_add(1);
                report.replayable_pages = report.replayable_pages.saturating_add(mutation_count);
            }
            WalRecordType::Init => {
                return Err(Error::corruption(
                    "logical WAL contains a second INIT frame",
                ));
            }
            WalRecordType::PageImage | WalRecordType::PageDelta => {
                return Err(Error::corruption(
                    "logical WAL contains a physical redo record",
                ));
            }
        }
    }
    let history_start_lsn = history_start_lsn
        .ok_or_else(|| Error::corruption("logical WAL has no complete INIT frame"))?;
    let history_start_sequence = history_start_sequence
        .ok_or_else(|| Error::corruption("logical WAL has no complete sequence boundary"))?;
    let valid_length = pending.as_ref().map_or(offset, |pending_transaction| {
        pending_transaction.start_offset
    });
    let next_lsn = last_commit_lsn
        .get()
        .max(history_start_lsn.get())
        .checked_add(1)
        .map(Lsn::new)
        .ok_or_else(|| Error::invariant("logical WAL LSN exhausted"))?;
    let next_batch_id = transactions
        .last()
        .map_or(1, |transaction| transaction.batch_id.saturating_add(1));
    Ok(LogicalWalScanOutput {
        next_lsn,
        next_batch_id,
        history_start_lsn,
        history_start_sequence,
        last_commit_lsn: (last_commit_sequence.is_some()).then_some(last_commit_lsn),
        last_commit_sequence,
        transactions,
        report,
        valid_length,
    })
}

fn hit(injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>, point: &str) -> Result<()> {
    if let Some(injector) = injector.as_deref_mut() {
        injector.hit(point)?;
    }
    Ok(())
}

fn is_torn_initialization_prefix<F: DurableFile>(
    file: &mut F,
    identity: &WalIdentity,
    start_after_lsn: Lsn,
    length: u64,
    page_image_format: WalPageImageFormat,
) -> Result<bool> {
    let length = usize::try_from(length)
        .map_err(|_| Error::recovery("WAL torn INIT length does not fit usize"))?;
    let expected = encode_frame(
        write_format_version(page_image_format),
        WalRecordType::Init,
        Lsn::ZERO,
        0,
        0,
        &identity_payload(identity, start_after_lsn),
    )?;
    if length >= expected.len() {
        return Ok(false);
    }
    let prefix = read_exact_at(file, 0, length)?;
    Ok(prefix == expected[..length])
}

fn identity_payload(identity: &WalIdentity, start_after_lsn: Lsn) -> [u8; INIT_PAYLOAD_SIZE] {
    let mut payload = [0u8; INIT_PAYLOAD_SIZE];
    payload[0..16].copy_from_slice(&identity.database_uuid);
    payload[16..24].copy_from_slice(&identity.tenant_id.get().to_le_bytes());
    payload[24..32].copy_from_slice(&identity.shard_id.get().to_le_bytes());
    payload[32..40].copy_from_slice(&identity.shard_epoch.get().to_le_bytes());
    payload[40..44].copy_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
    payload[44..52].copy_from_slice(&start_after_lsn.get().to_le_bytes());
    payload
}

fn encode_frame(
    format_version: u16,
    record_type: WalRecordType,
    record_lsn: Lsn,
    batch_id: u64,
    record_index: u32,
    payload: &[u8],
) -> Result<Vec<u8>> {
    encode_frame_impl(
        format_version,
        record_type,
        record_lsn,
        batch_id,
        record_index,
        payload,
    )
}

fn encode_frame_impl(
    format_version: u16,
    record_type: WalRecordType,
    record_lsn: Lsn,
    batch_id: u64,
    record_index: u32,
    payload: &[u8],
) -> Result<Vec<u8>> {
    if payload.len() > WAL_MAX_PAYLOAD_SIZE {
        return Err(Error::invalid_input("WAL payload exceeds maximum size"));
    }
    let frame_length = WAL_HEADER_SIZE
        .checked_add(payload.len())
        .and_then(|length| length.checked_add(WAL_TRAILER_SIZE))
        .ok_or_else(|| Error::invalid_input("WAL frame length overflows"))?;
    let mut header = [0u8; WAL_HEADER_SIZE];
    header[0..4].copy_from_slice(&WAL_MAGIC);
    header[4..6].copy_from_slice(&format_version.to_le_bytes());
    header[6] = record_type as u8;
    header[8..12].copy_from_slice(
        &u32::try_from(frame_length)
            .map_err(|_| Error::invalid_input("WAL frame length does not fit u32"))?
            .to_le_bytes(),
    );
    header[12..16].copy_from_slice(
        &u32::try_from(payload.len())
            .map_err(|_| Error::invalid_input("WAL payload length does not fit u32"))?
            .to_le_bytes(),
    );
    header[16..24].copy_from_slice(&record_lsn.get().to_le_bytes());
    header[24..32].copy_from_slice(&batch_id.to_le_bytes());
    header[32..36].copy_from_slice(&record_index.to_le_bytes());
    let payload_checksum = crc32c::crc32c(payload);
    header[PAYLOAD_CHECKSUM_OFFSET..PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum = header_checksum(&header);
    header[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&header_checksum.to_le_bytes());

    let mut frame = Vec::with_capacity(frame_length);
    frame.extend_from_slice(&header);
    frame.extend_from_slice(payload);
    frame.extend_from_slice(
        &u32::try_from(frame_length)
            .map_err(|_| Error::invalid_input("WAL frame length does not fit u32"))?
            .to_le_bytes(),
    );
    Ok(frame)
}

fn append_frame_direct(
    output: &mut Vec<u8>,
    format_version: u16,
    record_type: WalRecordType,
    record_lsn: Lsn,
    batch_id: u64,
    record_index: u32,
    payload: &[u8],
    attribution: &mut WalEncodeAttribution,
) -> Result<usize> {
    if payload.len() > WAL_MAX_PAYLOAD_SIZE {
        return Err(Error::invalid_input("WAL payload exceeds maximum size"));
    }
    let payload_crc_started = Instant::now();
    let payload_checksum = crc32c::crc32c(payload);
    attribution.group_commit_payload_crc_nanos += elapsed_nanos(payload_crc_started)?;
    append_frame_parts_direct(
        output,
        format_version,
        record_type,
        record_lsn,
        batch_id,
        record_index,
        payload.len(),
        payload_checksum,
        &[payload],
        attribution,
    )
}

fn append_frame_parts_direct(
    output: &mut Vec<u8>,
    format_version: u16,
    record_type: WalRecordType,
    record_lsn: Lsn,
    batch_id: u64,
    record_index: u32,
    payload_length: usize,
    payload_checksum: u32,
    payload_parts: &[&[u8]],
    attribution: &mut WalEncodeAttribution,
) -> Result<usize> {
    let frame_length = WAL_HEADER_SIZE
        .checked_add(payload_length)
        .and_then(|length| length.checked_add(WAL_TRAILER_SIZE))
        .ok_or_else(|| Error::invalid_input("WAL frame length overflows"))?;
    let mut header = [0u8; WAL_HEADER_SIZE];
    header[0..4].copy_from_slice(&WAL_MAGIC);
    header[4..6].copy_from_slice(&format_version.to_le_bytes());
    header[6] = record_type as u8;
    header[8..12].copy_from_slice(
        &u32::try_from(frame_length)
            .map_err(|_| Error::invalid_input("WAL frame length does not fit u32"))?
            .to_le_bytes(),
    );
    header[12..16].copy_from_slice(
        &u32::try_from(payload_length)
            .map_err(|_| Error::invalid_input("WAL payload length does not fit u32"))?
            .to_le_bytes(),
    );
    header[16..24].copy_from_slice(&record_lsn.get().to_le_bytes());
    header[24..32].copy_from_slice(&batch_id.to_le_bytes());
    header[32..36].copy_from_slice(&record_index.to_le_bytes());
    header[PAYLOAD_CHECKSUM_OFFSET..PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    let header_crc_started = Instant::now();
    let checksum = header_checksum(&header);
    let header_crc_nanos = elapsed_nanos(header_crc_started)?;
    header[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&checksum.to_le_bytes());
    match record_type {
        WalRecordType::PageImage | WalRecordType::PageDelta => {
            attribution.group_page_header_crc_nanos += header_crc_nanos;
        }
        WalRecordType::Commit => {
            attribution.group_commit_header_crc_nanos += header_crc_nanos;
        }
        WalRecordType::Init | WalRecordType::LogicalPut | WalRecordType::LogicalDelete => {}
    }
    let direct_started = Instant::now();
    output.extend_from_slice(&header);
    for part in payload_parts {
        output.extend_from_slice(part);
    }
    output.extend_from_slice(
        &u32::try_from(frame_length)
            .map_err(|_| Error::invalid_input("WAL frame length does not fit u32"))?
            .to_le_bytes(),
    );
    let direct_nanos = elapsed_nanos(direct_started)?;
    match record_type {
        WalRecordType::PageImage | WalRecordType::PageDelta => {
            attribution.group_page_direct_encode_nanos += direct_nanos;
        }
        WalRecordType::Commit => {
            attribution.group_commit_direct_encode_nanos += direct_nanos;
        }
        WalRecordType::Init | WalRecordType::LogicalPut | WalRecordType::LogicalDelete => {}
    }
    Ok(frame_length)
}

fn elapsed_nanos(started: Instant) -> Result<u64> {
    u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::invariant("WAL timing does not fit u64"))
}

fn scan_wal<F: DurableFile>(
    file: &mut F,
    identity: &WalIdentity,
    page_image_format: WalPageImageFormat,
    materialization: ScanMaterialization,
) -> Result<WalScanOutput> {
    let length = file.len()?;
    let mut offset = 0u64;
    let mut previous_lsn = None;
    let mut report = WalScanReport::default();
    let mut header_state: Option<(u16, Lsn)> = None;
    let mut batches = Vec::new();
    let mut pages: BTreeMap<PageId, RecoveredWalPage> = BTreeMap::new();
    let mut redo = WalRedoStats::default();
    let mut pending: Option<PendingBatch> = None;
    let mut highest_batch_id_seen = 0u64;
    let mut max_batch_id = 0u64;
    let mut last_commit_lsn = None;
    let blink = page_image_format == WalPageImageFormat::ExperimentalBlink;
    let compact = page_image_format != WalPageImageFormat::Baseline;
    let keep_batches = !blink || materialization == ScanMaterialization::Batches;
    while offset < length {
        let remaining = length - offset;
        if remaining < WAL_HEADER_SIZE as u64 {
            break;
        }
        let header_bytes = read_exact_at(file, offset, WAL_HEADER_SIZE)?;
        let (
            format_version,
            record_type,
            frame_length,
            payload_length,
            record_lsn,
            batch_id,
            record_index,
        ) = decode_header(&header_bytes)?;
        if header_state.is_some_and(|(first_version, _)| first_version != format_version) {
            return Err(Error::corruption(
                "WAL frames use inconsistent format versions",
            ));
        }
        if frame_length < WAL_MIN_FRAME_SIZE
            || frame_length != WAL_HEADER_SIZE + payload_length + WAL_TRAILER_SIZE
            || payload_length > WAL_MAX_PAYLOAD_SIZE
        {
            return Err(Error::corruption(format!(
                "invalid WAL frame length {frame_length} at offset {offset}"
            )));
        }
        let frame_length_u64 = u64::try_from(frame_length)
            .map_err(|_| Error::corruption("WAL frame length does not fit u64"))?;
        if frame_length_u64 > remaining {
            break;
        }
        let payload = read_exact_at(
            file,
            offset
                .checked_add(WAL_HEADER_SIZE as u64)
                .ok_or_else(|| Error::corruption("WAL payload offset overflows"))?,
            payload_length,
        )?;
        let trailer = read_exact_at(
            file,
            offset
                .checked_add((WAL_HEADER_SIZE + payload_length) as u64)
                .ok_or_else(|| Error::corruption("WAL trailer offset overflows"))?,
            WAL_TRAILER_SIZE,
        )?;
        if u32::from_le_bytes(trailer.try_into().unwrap()) != frame_length as u32 {
            return Err(Error::corruption(format!(
                "WAL trailing length mismatch at offset {offset}"
            )));
        }
        if previous_lsn.is_some_and(|previous| record_lsn <= previous) {
            return Err(Error::corruption(format!(
                "WAL LSN {} is not strictly greater than the previous LSN",
                record_lsn
            )));
        }
        previous_lsn = Some(record_lsn);
        verify_payload_checksum(&header_bytes, &payload, offset)?;
        report.records_scanned += 1;
        offset += frame_length_u64;
        let record_lsn = Lsn::new(record_lsn);

        let Some((_, history_start_lsn)) = header_state else {
            if record_type != WalRecordType::Init
                || record_lsn != Lsn::ZERO
                || batch_id != 0
                || record_index != 0
            {
                return Err(Error::corruption(
                    "WAL does not begin with the required initialization record",
                ));
            }
            header_state = Some((
                format_version,
                verify_identity(format_version, &payload, identity)?,
            ));
            continue;
        };
        if record_lsn <= history_start_lsn {
            return Err(Error::corruption(
                "WAL record LSN is not after the initialization history boundary",
            ));
        }
        max_batch_id = max_batch_id.max(batch_id);
        match record_type {
            WalRecordType::Init => {
                return Err(Error::corruption(
                    "WAL contains a second initialization record",
                ));
            }
            WalRecordType::PageImage | WalRecordType::PageDelta => {
                let image = if record_type == WalRecordType::PageImage {
                    let page = decode_page_image(&payload)?;
                    validate_page_image(&page, page_image_format)?;
                    Some(page)
                } else {
                    if format_version < WAL_FORMAT_VERSION || !compact {
                        return Err(Error::corruption(
                            "WAL page-delta record in a page-image-only WAL",
                        ));
                    }
                    if is_superblock_page(decode_page_delta(&payload)?.page_id) {
                        return Err(Error::corruption("WAL page delta targets a superblock"));
                    }
                    None
                };
                if pending
                    .as_ref()
                    .is_none_or(|pending| pending.batch_id != batch_id)
                {
                    if batch_id <= highest_batch_id_seen {
                        return Err(Error::corruption(
                            "WAL batch IDs are not strictly increasing",
                        ));
                    }
                    highest_batch_id_seen = batch_id;
                    pending = Some(PendingBatch {
                        batch_id,
                        first_record_lsn: record_lsn,
                        records: Vec::new(),
                        digest: 0,
                    });
                }
                let current = pending.as_mut().expect("pending batch was created");
                if record_index != current.records.len() as u32 {
                    return Err(Error::corruption(
                        "WAL page-image record index is not contiguous",
                    ));
                }
                current.digest = update_record_digest(
                    current.digest,
                    format_version,
                    record_type,
                    record_index,
                    &[&payload],
                );
                current.records.push(match image {
                    Some(page) => PendingRedo::Image(Box::new(page)),
                    None => PendingRedo::Delta(payload),
                });
            }
            WalRecordType::Commit => {
                let current = pending.take().ok_or_else(|| {
                    Error::corruption("WAL commit has no matching page-image records")
                })?;
                if current.batch_id != batch_id || record_index != current.records.len() as u32 {
                    return Err(Error::corruption(
                        "WAL commit does not match its page-image sequence",
                    ));
                }
                let (first_lsn, page_count, digest) = decode_commit_payload(&payload)?;
                if first_lsn != current.first_record_lsn
                    || page_count != current.records.len()
                    || current.digest != digest
                {
                    return Err(Error::corruption(
                        "WAL commit metadata does not match page images",
                    ));
                }
                let mut batch_pages = Vec::new();
                let mut batch_kinds = Vec::new();
                for record in current.records {
                    let (page, kind) = match record {
                        PendingRedo::Image(page) => {
                            validate_page_image_lsn(&page, record_lsn)?;
                            redo.page_image_records += 1;
                            (*page, WalRedoKind::PageImage)
                        }
                        PendingRedo::Delta(delta_payload) => {
                            let view = decode_page_delta(&delta_payload)?;
                            let base = pages.get(&view.page_id).ok_or_else(|| {
                                Error::corruption(format!(
                                    "WAL page delta for page {} has no full-image base in this WAL history",
                                    view.page_id
                                ))
                            })?;
                            let page = WalPageImage {
                                page_id: view.page_id,
                                image: apply_page_delta(&base.image, &view)?,
                            };
                            validate_page_image(&page, page_image_format)?;
                            validate_page_image_lsn(&page, record_lsn)?;
                            redo.page_delta_records += 1;
                            redo.page_delta_payload_bytes += delta_payload.len() as u64;
                            redo.page_delta_spans += view.spans.len() as u64;
                            redo.page_delta_changed_bytes += view.changed_bytes() as u64;
                            (page, WalRedoKind::PageDelta)
                        }
                    };
                    report.replayable_pages += 1;
                    if compact {
                        match pages.get_mut(&page.page_id) {
                            Some(recovered) => {
                                recovered.commit_lsn = record_lsn;
                                *recovered.image = page.image;
                            }
                            None => {
                                pages.insert(
                                    page.page_id,
                                    RecoveredWalPage {
                                        commit_lsn: record_lsn,
                                        image: Box::new(page.image),
                                    },
                                );
                            }
                        }
                    }
                    if keep_batches {
                        batch_pages.push(page);
                        batch_kinds.push(kind);
                    }
                }
                if keep_batches {
                    batches.push(CommittedWalBatch {
                        batch_id,
                        commit_lsn: record_lsn,
                        pages: batch_pages,
                        redo_kinds: batch_kinds,
                    });
                }
                report.committed_batches += 1;
                last_commit_lsn = Some(record_lsn);
            }
            WalRecordType::LogicalPut | WalRecordType::LogicalDelete => {
                return Err(Error::unsupported_format(
                    "logical records require the WAL v4 reader",
                ));
            }
        }
    }

    let (format_version, history_start_lsn) = header_state
        .ok_or_else(|| Error::corruption("WAL has no complete initialization record"))?;
    let next_lsn = Lsn::new(
        previous_lsn
            .unwrap_or(Lsn::ZERO.get())
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
    );
    let next_batch_id = max_batch_id
        .checked_add(1)
        .ok_or_else(|| Error::invariant("WAL batch ID exhausted"))?
        .max(1);
    Ok(WalScanOutput {
        next_lsn,
        next_batch_id,
        format_version,
        history_start_lsn,
        last_commit_lsn,
        batches,
        pages,
        redo,
        report,
        valid_length: offset,
    })
}

fn decode_header(bytes: &[u8]) -> Result<(u16, WalRecordType, usize, usize, u64, u64, u32)> {
    if bytes.len() != WAL_HEADER_SIZE {
        return Err(Error::corruption("WAL header has an invalid length"));
    }
    if bytes[0..4] != WAL_MAGIC {
        return Err(Error::corruption("WAL magic mismatch"));
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    if version != WAL_FORMAT_VERSION
        && version != WAL_LOGICAL_FORMAT_VERSION
        && version != WAL_PAGE_IMAGE_FORMAT_VERSION
        && version != LEGACY_WAL_FORMAT_VERSION
    {
        return Err(Error::unsupported_format(format!(
            "WAL version {version}, supported {WAL_FORMAT_VERSION}"
        )));
    }
    if bytes[7] != 0 || bytes[36..40].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption("WAL reserved header bytes are non-zero"));
    }
    let expected_checksum =
        u32::from_le_bytes(bytes[HEADER_CHECKSUM_OFFSET..44].try_into().unwrap());
    if expected_checksum != header_checksum(bytes) {
        return Err(Error::corruption("WAL header checksum mismatch"));
    }
    Ok((
        version,
        WalRecordType::decode(bytes[6])?,
        u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
        u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize,
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
        u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
        u32::from_le_bytes(bytes[32..36].try_into().unwrap()),
    ))
}

fn verify_payload_checksum(header: &[u8], payload: &[u8], offset: u64) -> Result<()> {
    let expected = u32::from_le_bytes(header[PAYLOAD_CHECKSUM_OFFSET..48].try_into().unwrap());
    let actual = crc32c::crc32c(payload);
    if expected != actual {
        return Err(Error::corruption(format!(
            "WAL payload checksum mismatch at offset {offset}: expected {expected:#010x}, calculated {actual:#010x}"
        )));
    }
    Ok(())
}

fn encode_page_image(
    page: &WalPageImage,
    page_image_format: WalPageImageFormat,
) -> Result<Vec<u8>> {
    let mut payload = Vec::with_capacity(PAGE_IMAGE_PAYLOAD_SIZE);
    payload.extend_from_slice(&page.page_id.get().to_le_bytes());
    payload.extend_from_slice(&page.image);
    validate_page_image(page, page_image_format)?;
    Ok(payload)
}

fn decode_page_image(payload: &[u8]) -> Result<WalPageImage> {
    if payload.len() != PAGE_IMAGE_PAYLOAD_SIZE {
        return Err(Error::corruption(format!(
            "WAL page-image payload has {} bytes, expected {PAGE_IMAGE_PAYLOAD_SIZE}",
            payload.len()
        )));
    }
    Ok(WalPageImage {
        page_id: PageId::new(u64::from_le_bytes(payload[0..8].try_into().unwrap())),
        image: payload[8..].try_into().unwrap(),
    })
}

fn validate_page_image(page: &WalPageImage, page_image_format: WalPageImageFormat) -> Result<()> {
    if page.page_id == SUPERBLOCK_A_PAGE || page.page_id == SUPERBLOCK_B_PAGE {
        match page_image_format {
            WalPageImageFormat::Baseline | WalPageImageFormat::ExperimentalBtree => {
                decode_superblock(&page.image)?;
            }
            WalPageImageFormat::ExperimentalBlink => {
                if page.page_id == SUPERBLOCK_A_PAGE || page.page_id == SUPERBLOCK_B_PAGE {
                    crate::blink::decode_blink_superblock_image(&page.image)?;
                } else {
                    crate::blink::validate_blink_page_image(&page.image, page.page_id)?;
                }
            }
        }
    } else {
        match page_image_format {
            WalPageImageFormat::Baseline | WalPageImageFormat::ExperimentalBtree => {
                decode_page_at(&page.image, Some(page.page_id))?;
            }
            WalPageImageFormat::ExperimentalBlink => {
                crate::blink::validate_blink_page_image(&page.image, page.page_id)?;
            }
        }
    }
    Ok(())
}

fn validate_page_image_lsn(page: &WalPageImage, commit_lsn: Lsn) -> Result<()> {
    if page.page_id == SUPERBLOCK_A_PAGE || page.page_id == SUPERBLOCK_B_PAGE {
        return Ok(());
    }
    let page_lsn = Lsn::new(u64::from_le_bytes(page.image[16..24].try_into().unwrap()));
    if page_lsn != commit_lsn {
        return Err(Error::corruption(format!(
            "WAL page {} has page LSN {}, commit LSN {}",
            page.page_id, page_lsn, commit_lsn
        )));
    }
    Ok(())
}

fn is_superblock_page(page_id: PageId) -> bool {
    page_id == SUPERBLOCK_A_PAGE || page_id == SUPERBLOCK_B_PAGE
}

fn page_lsn_of(image: &[u8; PAGE_SIZE]) -> Lsn {
    Lsn::new(u64::from_le_bytes(
        image[PAGE_LSN_RANGE].try_into().unwrap(),
    ))
}

fn write_format_version(page_image_format: WalPageImageFormat) -> u16 {
    match page_image_format {
        WalPageImageFormat::Baseline => WAL_PAGE_IMAGE_FORMAT_VERSION,
        WalPageImageFormat::ExperimentalBlink | WalPageImageFormat::ExperimentalBtree => {
            WAL_FORMAT_VERSION
        }
    }
}

fn canonical_delta_spans(base: &[u8; PAGE_SIZE], target: &[u8; PAGE_SIZE]) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut position = 0usize;
    while position < PAGE_SIZE {
        if position.is_multiple_of(8)
            && position + 8 <= PAGE_SIZE
            && base[position..position + 8] == target[position..position + 8]
        {
            position += 8;
            continue;
        }
        if base[position] == target[position] {
            position += 1;
            continue;
        }
        let run_start = position;
        while position < PAGE_SIZE && base[position] != target[position] {
            position += 1;
        }
        match spans.last_mut() {
            Some(last) if run_start - last.1 <= PAGE_DELTA_SPAN_MERGE_GAP => last.1 = position,
            _ => spans.push((run_start, position)),
        }
    }
    spans
}

pub(crate) fn encode_page_delta(
    page_id: PageId,
    base: &[u8; PAGE_SIZE],
    target: &[u8; PAGE_SIZE],
) -> Result<Vec<u8>> {
    let _delta_site = crate::churn::enter(crate::churn::ChurnSite::PageDeltaGeneration);
    let spans = canonical_delta_spans(base, target);
    if spans.is_empty() {
        return Err(Error::invalid_input(
            "a page delta needs at least one changed byte",
        ));
    }
    let payload_length = spans
        .iter()
        .fold(PAGE_DELTA_HEADER_SIZE, |length, (start, end)| {
            length + PAGE_DELTA_SPAN_HEADER_SIZE + (end - start)
        });
    if crate::churn::ENABLED {
        crate::churn::add(crate::churn::ChurnCounter::DeltaPayloadBuffers, 2);
        crate::churn::add(
            crate::churn::ChurnCounter::DeltaPayloadBytes,
            payload_length as u64,
        );
    }
    let mut payload = Vec::with_capacity(payload_length);
    payload.extend_from_slice(&page_id.get().to_le_bytes());
    payload.extend_from_slice(&page_lsn_of(base).get().to_le_bytes());
    payload.extend_from_slice(&(spans.len() as u16).to_le_bytes());
    for (start, end) in spans {
        payload.extend_from_slice(&(start as u16).to_le_bytes());
        payload.extend_from_slice(&((end - start) as u16).to_le_bytes());
        payload.extend_from_slice(&target[start..end]);
    }
    Ok(payload)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PageDeltaView<'payload> {
    pub page_id: PageId,
    pub base_page_lsn: Lsn,
    pub spans: Vec<(usize, &'payload [u8])>,
}

impl PageDeltaView<'_> {
    pub(crate) fn changed_bytes(&self) -> usize {
        self.spans.iter().map(|(_, bytes)| bytes.len()).sum()
    }
}

pub(crate) fn decode_page_delta(payload: &[u8]) -> Result<PageDeltaView<'_>> {
    if payload.len() < PAGE_DELTA_HEADER_SIZE {
        return Err(Error::corruption("WAL page-delta payload is truncated"));
    }
    if payload.len() >= PAGE_IMAGE_PAYLOAD_SIZE {
        return Err(Error::corruption(
            "WAL page-delta payload is not smaller than a page image",
        ));
    }
    let page_id = PageId::new(u64::from_le_bytes(payload[0..8].try_into().unwrap()));
    let base_page_lsn = Lsn::new(u64::from_le_bytes(payload[8..16].try_into().unwrap()));
    let span_count = usize::from(u16::from_le_bytes(payload[16..18].try_into().unwrap()));
    if span_count == 0 || span_count > PAGE_DELTA_MAX_SPANS {
        return Err(Error::corruption(format!(
            "WAL page-delta span count {span_count} is out of range"
        )));
    }
    let mut spans = Vec::with_capacity(span_count);
    let mut cursor = PAGE_DELTA_HEADER_SIZE;
    let mut previous_end: Option<usize> = None;
    for _ in 0..span_count {
        let span_header_end = cursor + PAGE_DELTA_SPAN_HEADER_SIZE;
        if span_header_end > payload.len() {
            return Err(Error::corruption("WAL page-delta span header is truncated"));
        }
        let offset = usize::from(u16::from_le_bytes(
            payload[cursor..cursor + 2].try_into().unwrap(),
        ));
        let length = usize::from(u16::from_le_bytes(
            payload[cursor + 2..cursor + 4].try_into().unwrap(),
        ));
        if length == 0 {
            return Err(Error::corruption("WAL page-delta span has zero length"));
        }
        if offset >= PAGE_SIZE || offset + length > PAGE_SIZE {
            return Err(Error::corruption("WAL page-delta span is outside the page"));
        }
        if previous_end.is_some_and(|end| offset <= end + PAGE_DELTA_SPAN_MERGE_GAP) {
            return Err(Error::corruption(
                "WAL page-delta spans overlap, are unsorted, or are not canonically merged",
            ));
        }
        let span_end = span_header_end + length;
        if span_end > payload.len() {
            return Err(Error::corruption("WAL page-delta span bytes are truncated"));
        }
        spans.push((offset, &payload[span_header_end..span_end]));
        previous_end = Some(offset + length);
        cursor = span_end;
    }
    if cursor != payload.len() {
        return Err(Error::corruption(
            "WAL page-delta payload has trailing bytes",
        ));
    }
    Ok(PageDeltaView {
        page_id,
        base_page_lsn,
        spans,
    })
}

fn check_page_delta_against_base(base: &[u8; PAGE_SIZE], delta: &PageDeltaView<'_>) -> Result<()> {
    if page_lsn_of(base) != delta.base_page_lsn {
        return Err(Error::corruption(format!(
            "WAL page delta for page {} expects base LSN {}, found {}",
            delta.page_id,
            delta.base_page_lsn,
            page_lsn_of(base)
        )));
    }
    for (offset, bytes) in &delta.spans {
        let original = &base[*offset..*offset + bytes.len()];
        if original[0] == bytes[0] || original[bytes.len() - 1] == bytes[bytes.len() - 1] {
            return Err(Error::corruption(
                "WAL page-delta span does not start and end on a changed byte",
            ));
        }
        let mut unchanged_run = 0usize;
        for (original_byte, new_byte) in original.iter().zip(bytes.iter()) {
            if original_byte == new_byte {
                unchanged_run += 1;
                if unchanged_run > PAGE_DELTA_SPAN_MERGE_GAP {
                    return Err(Error::corruption(
                        "WAL page-delta span contains an unchanged gap that should split it",
                    ));
                }
            } else {
                unchanged_run = 0;
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_page_delta(
    base: &[u8; PAGE_SIZE],
    delta: &PageDeltaView<'_>,
) -> Result<[u8; PAGE_SIZE]> {
    check_page_delta_against_base(base, delta)?;
    if crate::churn::ENABLED {
        crate::churn::add(crate::churn::ChurnCounter::DeltaVerifyImages, 1);
        crate::churn::add(crate::churn::ChurnCounter::PageImageCopies, 1);
        crate::churn::add(
            crate::churn::ChurnCounter::PageImageBytesCopied,
            PAGE_SIZE as u64,
        );
    }
    let mut rebuilt = *base;
    for (offset, bytes) in &delta.spans {
        rebuilt[*offset..*offset + bytes.len()].copy_from_slice(bytes);
    }
    Ok(rebuilt)
}

pub(crate) fn page_delta_rebuilds(
    base: &[u8; PAGE_SIZE],
    delta: &PageDeltaView<'_>,
    target: &[u8; PAGE_SIZE],
) -> Result<bool> {
    let ordered = delta
        .spans
        .windows(2)
        .all(|pair| pair[0].0 + pair[0].1.len() <= pair[1].0);
    if !ordered {
        return Ok(apply_page_delta(base, delta)? == *target);
    }
    check_page_delta_against_base(base, delta)?;
    let mut cursor = 0usize;
    for (offset, bytes) in &delta.spans {
        let span_end = *offset + bytes.len();
        if base[cursor..*offset] != target[cursor..*offset] || **bytes != target[*offset..span_end]
        {
            return Ok(false);
        }
        cursor = span_end;
    }
    Ok(base[cursor..] == target[cursor..])
}

fn prepared_commit_payload(
    first_record_lsn: Lsn,
    record_count: usize,
    digest: u32,
) -> Result<[u8; COMMIT_PAYLOAD_SIZE]> {
    let mut commit_payload = [0u8; COMMIT_PAYLOAD_SIZE];
    commit_payload[0..8].copy_from_slice(&first_record_lsn.get().to_le_bytes());
    commit_payload[8..12].copy_from_slice(
        &u32::try_from(record_count)
            .map_err(|_| Error::invalid_input("WAL page count does not fit u32"))?
            .to_le_bytes(),
    );
    commit_payload[12..16].copy_from_slice(&digest.to_le_bytes());
    Ok(commit_payload)
}

fn update_record_digest(
    digest: u32,
    format_version: u16,
    record_type: WalRecordType,
    record_index: u32,
    payload_parts: &[&[u8]],
) -> u32 {
    let mut digest = digest;
    if format_version >= WAL_FORMAT_VERSION {
        let payload_length: usize = payload_parts.iter().map(|part| part.len()).sum();
        digest = crc32c::crc32c_append(digest, &[record_type as u8]);
        digest = crc32c::crc32c_append(digest, &record_index.to_le_bytes());
        digest = crc32c::crc32c_append(digest, &(payload_length as u32).to_le_bytes());
    }
    for part in payload_parts {
        digest = crc32c::crc32c_append(digest, part);
    }
    digest
}

fn decode_commit_payload(payload: &[u8]) -> Result<(Lsn, usize, u32)> {
    if payload.len() != COMMIT_PAYLOAD_SIZE {
        return Err(Error::corruption(format!(
            "WAL commit payload has {} bytes, expected {COMMIT_PAYLOAD_SIZE}",
            payload.len()
        )));
    }
    let page_count = u32::from_le_bytes(payload[8..12].try_into().unwrap());
    Ok((
        Lsn::new(u64::from_le_bytes(payload[0..8].try_into().unwrap())),
        usize::try_from(page_count).map_err(|_| Error::corruption("WAL page count overflows"))?,
        u32::from_le_bytes(payload[12..16].try_into().unwrap()),
    ))
}

fn verify_identity(version: u16, payload: &[u8], expected: &WalIdentity) -> Result<Lsn> {
    let valid_length = if version == LEGACY_WAL_FORMAT_VERSION {
        payload.len() == LEGACY_INIT_PAYLOAD_SIZE
    } else if version == WAL_LOGICAL_FORMAT_VERSION {
        payload.len() == INIT_PAYLOAD_SIZE || payload.len() == LOGICAL_INIT_PAYLOAD_SIZE
    } else {
        payload.len() == INIT_PAYLOAD_SIZE
    };
    if !valid_length {
        return Err(Error::corruption(
            "WAL initialization payload has invalid length",
        ));
    }
    let actual = WalIdentity {
        database_uuid: payload[0..16].try_into().unwrap(),
        tenant_id: TenantId::new(u64::from_le_bytes(payload[16..24].try_into().unwrap())),
        shard_id: ShardId::new(u64::from_le_bytes(payload[24..32].try_into().unwrap())),
        shard_epoch: ShardEpoch::new(u64::from_le_bytes(payload[32..40].try_into().unwrap())),
    };
    let page_size = u32::from_le_bytes(payload[40..44].try_into().unwrap());
    if page_size as usize != PAGE_SIZE {
        return Err(Error::unsupported_format("WAL page size is unsupported"));
    }
    if &actual != expected {
        return Err(Error::corruption(
            "WAL identity does not match the requested database shard",
        ));
    }
    let start_after_lsn = if version == LEGACY_WAL_FORMAT_VERSION {
        Lsn::ZERO
    } else {
        Lsn::new(u64::from_le_bytes(payload[44..52].try_into().unwrap()))
    };
    Ok(start_after_lsn)
}

fn header_checksum(header: &[u8]) -> u32 {
    let mut input = [0u8; WAL_HEADER_SIZE];
    input.copy_from_slice(header);
    input[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 4].fill(0);
    crc32c::crc32c(&input)
}

fn read_exact_at<F: DurableFile>(file: &mut F, offset: u64, length: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; length];
    let mut position = 0usize;
    while position < length {
        let count = file.read_at(
            offset
                .checked_add(position as u64)
                .ok_or_else(|| Error::corruption("WAL read offset overflows"))?,
            &mut bytes[position..],
        )?;
        if count == 0 {
            return Err(Error::Io(std::io::Error::new(
                ErrorKind::UnexpectedEof,
                "unexpected end of WAL",
            )));
        }
        position += count;
    }
    Ok(bytes)
}

fn write_all_at_counted<F: DurableFile>(
    file: &mut F,
    offset: u64,
    bytes: &[u8],
    physical_write_calls: &mut u64,
) -> Result<()> {
    let mut position = 0usize;
    while position < bytes.len() {
        let write_result = file.write_at(
            offset
                .checked_add(position as u64)
                .ok_or_else(|| Error::invalid_input("WAL write offset overflows"))?,
            &bytes[position..],
        );
        *physical_write_calls = physical_write_calls
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL physical write count overflow"))?;
        let count = write_result?;
        if count == 0 {
            return Err(Error::Io(std::io::Error::new(
                ErrorKind::WriteZero,
                "WAL returned a zero-byte write",
            )));
        }
        position += count;
    }
    Ok(())
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct TestWalFrame {
    pub version: u16,
    pub record_type: WalRecordType,
    pub record_lsn: u64,
    pub batch_id: u64,
    pub record_index: u32,
    pub payload: Vec<u8>,
}

#[cfg(test)]
pub(crate) fn parse_wal_frames_for_test(bytes: &[u8]) -> Vec<TestWalFrame> {
    let mut frames = Vec::new();
    let mut offset = 0usize;
    while offset + WAL_HEADER_SIZE <= bytes.len() {
        let (
            version,
            record_type,
            frame_length,
            payload_length,
            record_lsn,
            batch_id,
            record_index,
        ) = decode_header(&bytes[offset..offset + WAL_HEADER_SIZE]).expect("test WAL header");
        frames.push(TestWalFrame {
            version,
            record_type,
            record_lsn,
            batch_id,
            record_index,
            payload: bytes[offset + WAL_HEADER_SIZE..offset + WAL_HEADER_SIZE + payload_length]
                .to_vec(),
        });
        offset += frame_length;
    }
    frames
}

#[cfg(test)]
pub(crate) fn serialize_wal_frames_for_test(frames: &[TestWalFrame]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for frame in frames {
        bytes.extend_from_slice(
            &encode_frame(
                frame.version,
                frame.record_type,
                Lsn::new(frame.record_lsn),
                frame.batch_id,
                frame.record_index,
                &frame.payload,
            )
            .expect("test WAL frame encodes"),
        );
    }
    bytes
}

#[cfg(test)]
pub(crate) fn recompute_commit_digests_for_test(frames: &mut [TestWalFrame]) {
    let mut digest = 0u32;
    let mut count = 0u32;
    let mut first_lsn = None;
    for frame in frames.iter_mut() {
        match frame.record_type {
            WalRecordType::Init | WalRecordType::LogicalPut | WalRecordType::LogicalDelete => {}
            WalRecordType::PageImage | WalRecordType::PageDelta => {
                first_lsn.get_or_insert(frame.record_lsn);
                digest = update_record_digest(
                    digest,
                    frame.version,
                    frame.record_type,
                    frame.record_index,
                    &[&frame.payload],
                );
                count += 1;
            }
            WalRecordType::Commit => {
                frame.payload[0..8].copy_from_slice(&first_lsn.take().unwrap_or(0).to_le_bytes());
                frame.payload[8..12].copy_from_slice(&count.to_le_bytes());
                frame.payload[12..16].copy_from_slice(&digest.to_le_bytes());
                digest = 0;
                count = 0;
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn record_digest_for_test(version: u16, records: &[(WalRecordType, u32, &[u8])]) -> u32 {
    records
        .iter()
        .fold(0u32, |digest, (record_type, record_index, payload)| {
            update_record_digest(digest, version, *record_type, *record_index, &[payload])
        })
}

#[cfg(test)]
pub(crate) fn init_frame_for_test(
    version: u16,
    identity: &WalIdentity,
    start_after_lsn: Lsn,
) -> Vec<u8> {
    encode_frame(
        version,
        WalRecordType::Init,
        Lsn::ZERO,
        0,
        0,
        &identity_payload(identity, start_after_lsn),
    )
    .expect("test INIT frame encodes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{PageHeader, PageType, encode_page};
    use std::cell::Cell;

    #[derive(Default)]
    struct MemoryFile(Vec<u8>);

    struct CountingFile {
        inner: MemoryFile,
        write_calls: u64,
        len_calls: Cell<u64>,
        sync_calls: u64,
        max_write: Option<usize>,
    }

    impl CountingFile {
        fn new(max_write: Option<usize>) -> Self {
            Self {
                inner: MemoryFile::default(),
                write_calls: 0,
                len_calls: Cell::new(0),
                sync_calls: 0,
                max_write,
            }
        }

        fn reset_counts(&mut self) {
            self.write_calls = 0;
            self.len_calls.set(0);
            self.sync_calls = 0;
        }
    }

    impl DurableFile for CountingFile {
        fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
            self.inner.read_at(offset, buffer)
        }

        fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
            let write_length = self
                .max_write
                .map_or(bytes.len(), |limit| bytes.len().min(limit));
            self.write_calls += 1;
            self.inner.write_at(offset, &bytes[..write_length])
        }

        fn len(&self) -> Result<u64> {
            self.len_calls.set(self.len_calls.get() + 1);
            self.inner.len()
        }

        fn set_len(&mut self, length: u64) -> Result<()> {
            self.inner.set_len(length)
        }

        fn sync_data(&mut self) -> Result<()> {
            self.sync_calls += 1;
            Ok(())
        }

        fn sync_all(&mut self) -> Result<()> {
            Ok(())
        }
    }

    struct NoopInjector;

    impl FaultInjector for NoopInjector {
        fn hit(&mut self, _point: &str) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingInjector(Vec<String>);

    impl FaultInjector for RecordingInjector {
        fn hit(&mut self, point: &str) -> Result<()> {
            self.0.push(point.to_owned());
            Ok(())
        }
    }

    struct FailAtPoint(&'static str);

    impl FaultInjector for FailAtPoint {
        fn hit(&mut self, point: &str) -> Result<()> {
            if point == self.0 {
                return Err(Error::recovery(format!("injected failure at {point}")));
            }
            Ok(())
        }
    }

    impl DurableFile for MemoryFile {
        fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
            let offset = usize::try_from(offset).unwrap();
            if offset >= self.0.len() {
                return Ok(0);
            }
            let count = buffer.len().min(self.0.len() - offset);
            buffer[..count].copy_from_slice(&self.0[offset..offset + count]);
            Ok(count)
        }

        fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
            let offset = usize::try_from(offset).unwrap();
            let end = offset + bytes.len();
            self.0.resize(self.0.len().max(end), 0);
            self.0[offset..end].copy_from_slice(bytes);
            Ok(bytes.len())
        }

        fn len(&self) -> Result<u64> {
            Ok(self.0.len() as u64)
        }

        fn set_len(&mut self, length: u64) -> Result<()> {
            self.0.resize(length as usize, 0);
            Ok(())
        }

        fn sync_data(&mut self) -> Result<()> {
            Ok(())
        }

        fn sync_all(&mut self) -> Result<()> {
            Ok(())
        }
    }

    fn identity() -> WalIdentity {
        WalIdentity::new(
            [7; 16],
            TenantId::new(1),
            ShardId::new(2),
            ShardEpoch::new(3),
        )
    }

    fn page(page_id: u64) -> WalPageImage {
        page_at_lsn(page_id, Lsn::new(2))
    }

    fn page_at_lsn(page_id: u64, lsn: Lsn) -> WalPageImage {
        WalPageImage {
            page_id: PageId::new(page_id),
            image: encode_page(
                PageHeader::new(PageType::Leaf, PageId::new(page_id), lsn),
                &[],
            )
            .unwrap(),
        }
    }

    #[test]
    fn combined_payload_digest_matches_direct_digest_for_randomized_records() {
        let mut random_state = 0x8f3d_9b71_c2a5_640d_u64;
        let mut tested_records = 0usize;
        let mut tested_page_deltas = 0usize;
        let mut tested_page_images = 0usize;
        for case_index in 0..4_096u32 {
            random_state ^= random_state << 13;
            random_state ^= random_state >> 7;
            random_state ^= random_state << 17;
            let record_count = (random_state as usize % 8) + 1;
            let format_version = if case_index % 2 == 0 {
                WAL_FORMAT_VERSION
            } else {
                LEGACY_WAL_FORMAT_VERSION
            };
            let mut direct_digest = case_index.wrapping_mul(0x9e37_79b9);
            let mut combined_digest = direct_digest;
            for record_position in 0..record_count {
                random_state ^= random_state << 13;
                random_state ^= random_state >> 7;
                random_state ^= random_state << 17;
                let record_type = if (case_index + record_position as u32) % 3 == 0 {
                    tested_page_images += 1;
                    WalRecordType::PageImage
                } else {
                    tested_page_deltas += 1;
                    WalRecordType::PageDelta
                };
                let payload_length = match record_type {
                    WalRecordType::PageImage => PAGE_SIZE + 8,
                    WalRecordType::PageDelta => match random_state % 4 {
                        0 => 24,
                        1 => 159,
                        2 => 2_048,
                        _ => PAGE_SIZE - 17,
                    },
                    _ => unreachable!(),
                };
                let mut payload = Vec::with_capacity(payload_length);
                while payload.len() < payload_length {
                    random_state ^= random_state << 13;
                    random_state ^= random_state >> 7;
                    random_state ^= random_state << 17;
                    payload.push(random_state as u8);
                }
                let split_point = payload.len() / 3;
                let payload_parts = [&payload[..split_point], &payload[split_point..]];
                let record_index = match record_position % 4 {
                    0 => 0,
                    1 => u32::MAX,
                    _ => case_index
                        .wrapping_mul(31)
                        .wrapping_add(record_position as u32),
                };
                direct_digest = update_record_digest(
                    direct_digest,
                    format_version,
                    record_type,
                    record_index,
                    &payload_parts,
                );

                let mut metadata_digest = combined_digest;
                let combined_payload_length =
                    payload_parts.iter().map(|part| part.len()).sum::<usize>();
                if format_version >= WAL_FORMAT_VERSION {
                    metadata_digest = crc32c::crc32c_append(metadata_digest, &[record_type as u8]);
                    metadata_digest =
                        crc32c::crc32c_append(metadata_digest, &record_index.to_le_bytes());
                    metadata_digest = crc32c::crc32c_append(
                        metadata_digest,
                        &(combined_payload_length as u32).to_le_bytes(),
                    );
                }
                let payload_crc = payload_parts
                    .iter()
                    .fold(0u32, |checksum, part| crc32c::crc32c_append(checksum, part));
                combined_digest =
                    crc32c::crc32c_combine(metadata_digest, payload_crc, combined_payload_length);
                assert_eq!(
                    combined_digest, direct_digest,
                    "case {case_index}, record {record_position}"
                );
                tested_records += 1;
            }
        }
        assert!(tested_records >= 4_096);
        assert!(tested_page_deltas > 0);
        assert!(tested_page_images > 0);

        for (record_type, payload_length) in [
            (WalRecordType::PageDelta, 159usize),
            (WalRecordType::PageDelta, PAGE_SIZE - 17),
            (WalRecordType::PageImage, PAGE_SIZE + 8),
        ] {
            let payload = vec![0x5a; payload_length];
            let record_index = 17u32;
            let iteration_count = 50_000u32;
            let mut direct_digest = 0x1357_9bdf;
            let direct_started = std::time::Instant::now();
            for iteration in 0..iteration_count {
                let payload_crc = crc32c::crc32c(std::hint::black_box(&payload));
                direct_digest = update_record_digest(
                    direct_digest,
                    WAL_FORMAT_VERSION,
                    record_type,
                    record_index.wrapping_add(iteration),
                    &[&payload],
                );
                std::hint::black_box(payload_crc);
            }
            let direct_elapsed = direct_started.elapsed();

            let mut combined_digest = 0x1357_9bdf;
            let combine_started = std::time::Instant::now();
            for iteration in 0..iteration_count {
                let payload_crc = crc32c::crc32c(std::hint::black_box(&payload));
                let current_record_index = record_index.wrapping_add(iteration);
                let mut metadata_digest =
                    crc32c::crc32c_append(combined_digest, &[record_type as u8]);
                metadata_digest =
                    crc32c::crc32c_append(metadata_digest, &current_record_index.to_le_bytes());
                metadata_digest =
                    crc32c::crc32c_append(metadata_digest, &(payload.len() as u32).to_le_bytes());
                combined_digest =
                    crc32c::crc32c_combine(metadata_digest, payload_crc, payload.len());
            }
            let combine_elapsed = combine_started.elapsed();
            assert_eq!(direct_digest, combined_digest);
            println!(
                "H2 length={payload_length} records={iteration_count} direct_ns_per_record={} combine_ns_per_record={}",
                direct_elapsed.as_nanos() / u128::from(iteration_count),
                combine_elapsed.as_nanos() / u128::from(iteration_count)
            );
        }
    }

    fn wal_commits(commit_count: usize, pages_per_commit: usize) -> Vec<WalCommit> {
        let mut first_record_lsn = 1u64;
        (0..commit_count)
            .map(|commit_index| {
                let commit_lsn = first_record_lsn + pages_per_commit as u64;
                let pages = (0..pages_per_commit)
                    .map(|page_index| {
                        page_at_lsn(
                            2 + (commit_index * pages_per_commit + page_index) as u64,
                            Lsn::new(commit_lsn),
                        )
                    })
                    .collect();
                let commit = WalCommit {
                    batch_id: commit_index as u64 + 1,
                    commit_lsn: Lsn::new(commit_lsn),
                    pages,
                };
                first_record_lsn = commit_lsn + 1;
                commit
            })
            .collect()
    }

    #[test]
    fn framed_commit_round_trips_and_assigns_monotonic_lsns() {
        let mut wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let commit_lsn = Lsn::new(2);
        wal.append_commit(1, commit_lsn, &[page(2)], None).unwrap();
        assert_eq!(wal.last_commit_lsn(), Some(commit_lsn));
        assert_eq!(wal.committed_batches_on_disk()[0].commit_lsn, commit_lsn);
        assert_eq!(wal.next_lsn(), Lsn::new(3));
    }

    #[test]
    fn group_fast_path_is_byte_identical_to_fault_injectable_path() {
        let commits = wal_commits(4, 2);
        let mut fast_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let fast_reports = fast_wal.append_group(&commits, None).unwrap();
        let fast_metrics = fast_wal.metrics().unwrap();
        let fast_next_lsn = fast_wal.next_lsn();
        let fast_next_batch_id = fast_wal.next_batch_id;
        assert_eq!(fast_metrics.group_page_frames, 8);
        assert_eq!(fast_metrics.group_commit_frames, 4);
        assert_eq!(fast_metrics.group_page_validations, 8);
        assert!(fast_metrics.group_page_lsn_validate_nanos > 0);
        assert_eq!(fast_metrics.group_page_image_materialize_nanos, 0);
        assert!(fast_metrics.group_page_image_validate_nanos > 0);
        assert_eq!(fast_metrics.group_digest_copy_nanos, 0);
        assert!(fast_metrics.group_page_direct_encode_nanos > 0);
        assert!(fast_metrics.group_commit_direct_encode_nanos > 0);
        let fast_bytes = fast_wal.into_file().0;

        let mut injected_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let mut injector = NoopInjector;
        let injected_reports = injected_wal
            .append_group(&commits, Some(&mut injector))
            .unwrap();
        assert_eq!(fast_reports, injected_reports);
        assert_eq!(fast_next_lsn, injected_wal.next_lsn());
        assert_eq!(fast_next_batch_id, injected_wal.next_batch_id);
        let injected_bytes = injected_wal.into_file().0;
        assert_eq!(fast_bytes, injected_bytes);
    }

    #[test]
    fn split_page_payload_crc_and_incremental_digest_match_contiguous_crc() {
        let mut state = 0x7f4a_7c15_d1ce_4e5b_u64;
        for case_index in 0..512 {
            let page_count = case_index % 19 + 1;
            let mut concatenated = Vec::with_capacity(page_count * PAGE_IMAGE_PAYLOAD_SIZE);
            let mut digest = 0u32;
            for page_index in 0..page_count {
                let mut image = [0u8; PAGE_SIZE];
                for byte in &mut image {
                    state = state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    *byte = (state >> 32) as u8;
                }
                let page_id = state.rotate_left(17).wrapping_add(page_index as u64 + 2);
                let page_id_bytes = page_id.to_le_bytes();
                let mut payload = Vec::with_capacity(PAGE_IMAGE_PAYLOAD_SIZE);
                payload.extend_from_slice(&page_id_bytes);
                payload.extend_from_slice(&image);
                let split_crc = crc32c::crc32c(&page_id_bytes);
                let split_crc = crc32c::crc32c_append(split_crc, &image);
                assert_eq!(split_crc, crc32c::crc32c(&payload));
                digest = crc32c::crc32c_append(digest, &page_id_bytes);
                digest = crc32c::crc32c_append(digest, &image);
                concatenated.extend_from_slice(&payload);
            }
            assert_eq!(digest, crc32c::crc32c(&concatenated));
        }
    }

    #[test]
    fn direct_group_encoding_matches_reference_for_randomized_valid_groups() {
        let mut state = 0x2d35_8dcc_aa6c_78a5_u64;
        for case_index in 0..300 {
            let commit_count = case_index % 5 + 1;
            let mut commits = Vec::with_capacity(commit_count);
            let mut first_record_lsn = 1u64;
            let mut page_id = 2u64;
            for commit_index in 0..commit_count {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let page_count = ((state >> 32) as usize % 7) + 1;
                let commit_lsn = first_record_lsn + page_count as u64;
                let mut pages = Vec::with_capacity(page_count);
                for _ in 0..page_count {
                    pages.push(page_at_lsn(page_id, Lsn::new(commit_lsn)));
                    page_id += 1;
                }
                commits.push(WalCommit {
                    batch_id: commit_index as u64 + 1,
                    commit_lsn: Lsn::new(commit_lsn),
                    pages,
                });
                first_record_lsn = commit_lsn + 1;
            }

            let mut direct_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
            let direct_reports = direct_wal.append_group(&commits, None).unwrap();
            let direct_next_lsn = direct_wal.next_lsn();
            let direct_next_batch_id = direct_wal.next_batch_id();
            let direct_bytes = direct_wal.into_file().0;

            let mut reference_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
            let mut injector = NoopInjector;
            let reference_reports = reference_wal
                .append_group(&commits, Some(&mut injector))
                .unwrap();
            assert_eq!(direct_reports, reference_reports);
            assert_eq!(direct_next_lsn, reference_wal.next_lsn());
            assert_eq!(direct_next_batch_id, reference_wal.next_batch_id());
            assert_eq!(direct_bytes, reference_wal.into_file().0);
        }
    }

    #[test]
    fn trusted_internal_valid_images_are_byte_identical_to_strict_images() {
        let commits = wal_commits(4, 2);
        let mut strict_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let strict_reports = strict_wal.append_group(&commits, None).unwrap();
        let strict_next_lsn = strict_wal.next_lsn();
        let strict_next_batch_id = strict_wal.next_batch_id();
        let strict_bytes = strict_wal.into_file().0;

        let mut trusted_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let trusted_reports = trusted_wal
            .append_group_trusted_internal(&commits, None)
            .unwrap();
        assert_eq!(trusted_reports, strict_reports);
        assert_eq!(trusted_wal.next_lsn(), strict_next_lsn);
        assert_eq!(trusted_wal.next_batch_id(), strict_next_batch_id);
        assert_eq!(trusted_wal.into_file().0, strict_bytes);
    }

    #[test]
    fn trusted_internal_fault_injection_delegates_to_the_strict_path() {
        let commits = wal_commits(2, 1);
        let mut strict_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let mut strict_injector = RecordingInjector::default();
        let strict_reports = strict_wal
            .append_group(&commits, Some(&mut strict_injector))
            .unwrap();

        let mut trusted_wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let mut trusted_injector = RecordingInjector::default();
        let trusted_reports = trusted_wal
            .append_group_trusted_internal(&commits, Some(&mut trusted_injector))
            .unwrap();

        assert_eq!(trusted_reports, strict_reports);
        assert_eq!(trusted_injector.0, strict_injector.0);
        assert!(
            trusted_injector
                .0
                .iter()
                .any(|point| point == "before_wal_append")
        );
        assert!(
            trusted_injector
                .0
                .iter()
                .any(|point| point == "after_group_records_written")
        );
        assert!(
            trusted_injector
                .0
                .iter()
                .any(|point| point == "before_wal_sync")
        );
        assert!(
            trusted_injector
                .0
                .iter()
                .any(|point| point == "during_wal_sync")
        );
        assert!(
            trusted_injector
                .0
                .iter()
                .any(|point| point == "after_wal_sync")
        );
        assert_eq!(trusted_wal.into_file().0, strict_wal.into_file().0);
    }

    #[test]
    fn trusted_internal_fault_injection_still_rejects_invalid_images() {
        let mut malformed_commit = wal_commits(1, 1);
        malformed_commit[0].pages[0].image[100] ^= 1;
        let mut wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let mut injector = NoopInjector;
        assert!(
            wal.append_group_trusted_internal(&malformed_commit, Some(&mut injector))
                .is_err()
        );
        assert_eq!(wal.committed_batch_count(), 0);
        assert_eq!(wal.last_commit_lsn(), None);
    }

    #[test]
    fn appended_commits_keep_only_metadata_and_reopen_hands_payload_once() {
        let mut wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let commits = wal_commits(3_000, 1);
        for commit_group in commits.chunks(7) {
            wal.append_group(commit_group, None).unwrap();
        }
        let metrics = wal.metrics().unwrap();
        assert_eq!(metrics.committed_batches, 3_000);
        assert_eq!(metrics.page_images, 3_000);
        assert_eq!(metrics.retained_recovery_batches, 0);
        assert_eq!(metrics.retained_recovery_page_images, 0);
        assert!(wal.recovery_batches().is_empty());
        let last_commit_lsn = commits.last().unwrap().commit_lsn;
        assert_eq!(wal.last_commit_lsn(), Some(last_commit_lsn));

        let mut reopened = WalLog::open(wal.into_file(), identity()).unwrap();
        assert_eq!(reopened.last_commit_lsn(), Some(last_commit_lsn));
        assert_eq!(reopened.recovery_batches().len(), 3_000);
        assert_eq!(
            reopened.metrics().unwrap().retained_recovery_page_images,
            3_000
        );
        let recovery = reopened.take_recovery_batches();
        assert_eq!(
            recovery
                .iter()
                .map(|batch| (batch.batch_id, batch.commit_lsn, batch.pages.clone()))
                .collect::<Vec<_>>(),
            commits
                .iter()
                .map(|commit| (commit.batch_id, commit.commit_lsn, commit.pages.clone()))
                .collect::<Vec<_>>()
        );
        let metrics = reopened.metrics().unwrap();
        assert_eq!(metrics.retained_recovery_batches, 0);
        assert_eq!(metrics.retained_recovery_page_images, 0);
        assert_eq!(metrics.committed_batches, 3_000);
        assert_eq!(reopened.last_commit_lsn(), Some(last_commit_lsn));
        assert!(reopened.take_recovery_batches().is_empty());

        reopened.reset(last_commit_lsn, None).unwrap();
        assert_eq!(reopened.last_commit_lsn(), None);
        assert_eq!(reopened.committed_batch_count(), 0);
    }

    #[test]
    fn group_fast_path_uses_one_physical_write() {
        let mut wal = WalLog::open(CountingFile::new(None), identity()).unwrap();
        let metrics_before = wal.metrics().unwrap();
        wal.file.reset_counts();
        wal.append_group(&wal_commits(10, 2), None).unwrap();
        let metrics_after = wal.metrics().unwrap();
        assert_eq!(wal.file.write_calls, 1);
        assert_eq!(wal.file.len_calls.get(), 2);
        assert_eq!(wal.file.sync_calls, 1);
        assert_eq!(
            metrics_after.physical_write_calls - metrics_before.physical_write_calls,
            1
        );
        let group_encode_nanos =
            metrics_after.group_encode_nanos - metrics_before.group_encode_nanos;
        let group_write_nanos = metrics_after.group_write_nanos - metrics_before.group_write_nanos;
        let append_nanos = metrics_after.append_nanos - metrics_before.append_nanos;
        assert!(group_encode_nanos > 0);
        assert!(group_write_nanos > 0);
        assert!(group_encode_nanos + group_write_nanos <= append_nanos * 105 / 100 + 50_000);
    }

    #[test]
    fn group_fast_path_handles_short_writes() {
        let mut wal = WalLog::open(CountingFile::new(Some(137)), identity()).unwrap();
        wal.file.reset_counts();
        wal.append_group(&wal_commits(10, 2), None).unwrap();
        assert!(wal.file.write_calls > 1);
        let file = wal.into_file();
        let reopened = WalLog::open(file, identity()).unwrap();
        assert_eq!(reopened.recovery_batches().len(), 10);
        assert_eq!(reopened.scan_report().replayable_pages, 20);
    }

    #[test]
    fn fast_group_torn_tail_keeps_only_complete_commits() {
        let mut wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        wal.append_group(&wal_commits(2, 1), None).unwrap();
        let mut file = wal.into_file();
        let original_length = file.len().unwrap();
        file.set_len(original_length - 10).unwrap();
        let reopened = WalLog::open(file, identity()).unwrap();
        assert_eq!(reopened.recovery_batches().len(), 1);
        assert_eq!(reopened.scan_report().replayable_pages, 1);
        assert_eq!(reopened.scan_report().torn_tail_bytes, 58);
    }

    #[test]
    fn incomplete_final_frame_is_truncated() {
        let mut wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        wal.append_commit(1, Lsn::new(2), &[page(2)], None).unwrap();
        let mut file = wal.into_file();
        let length = file.len().unwrap();
        file.write_at(length, &[1, 2, 3]).unwrap();
        let wal = WalLog::open(file, identity()).unwrap();
        assert_eq!(wal.scan_report().torn_tail_bytes, 3);
    }

    #[test]
    fn payload_corruption_is_not_treated_as_a_torn_tail() {
        let mut wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        wal.append_commit(1, Lsn::new(2), &[page(2)], None).unwrap();
        let mut file = wal.into_file();
        let offset = WAL_HEADER_SIZE as u64 + 8;
        let mut byte = [0u8; 1];
        file.read_at(offset, &mut byte).unwrap();
        byte[0] ^= 1;
        file.write_at(offset, &byte).unwrap();
        assert!(matches!(
            WalLog::open(file, identity()),
            Err(Error::Corruption(_))
        ));
    }

    #[test]
    fn wrong_magic_is_rejected() {
        let wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let mut file = wal.into_file();
        file.write_at(0, b"NOPE").unwrap();
        assert!(matches!(
            WalLog::open(file, identity()),
            Err(Error::Corruption(_))
        ));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let mut file = wal.into_file();
        let mut header = [0u8; WAL_HEADER_SIZE];
        file.read_at(0, &mut header).unwrap();
        header[4..6].copy_from_slice(&(WAL_FORMAT_VERSION + 1).to_le_bytes());
        let checksum = header_checksum(&header);
        header[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&checksum.to_le_bytes());
        file.write_at(0, &header).unwrap();
        assert!(matches!(
            WalLog::open(file, identity()),
            Err(Error::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn impossible_frame_length_is_rejected() {
        let wal = WalLog::open(MemoryFile::default(), identity()).unwrap();
        let mut file = wal.into_file();
        let mut header = [0u8; WAL_HEADER_SIZE];
        file.read_at(0, &mut header).unwrap();
        header[8..12].copy_from_slice(&(WAL_MIN_FRAME_SIZE as u32 - 1).to_le_bytes());
        let checksum = header_checksum(&header);
        header[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&checksum.to_le_bytes());
        file.write_at(0, &header).unwrap();
        assert!(matches!(
            WalLog::open(file, identity()),
            Err(Error::Corruption(_))
        ));
    }

    #[test]
    fn logical_wal_group_round_trips_and_separates_commit_sequence_from_frame_lsn() {
        let mut wal =
            LogicalWalLog::open(MemoryFile::default(), identity(), Lsn::ZERO, None).unwrap();
        let syncs_before = wal.sync_count();
        let reports = wal
            .append_group(
                &[
                    LogicalWalTransaction {
                        commit_sequence: 1,
                        mutations: vec![
                            LogicalWalMutation {
                                encoded_key: b"a".to_vec(),
                                value: Some(b"one".to_vec()),
                                revision: Revision::new(1),
                            },
                            LogicalWalMutation {
                                encoded_key: b"b".to_vec(),
                                value: None,
                                revision: Revision::new(1),
                            },
                        ],
                    },
                    LogicalWalTransaction {
                        commit_sequence: 2,
                        mutations: vec![LogicalWalMutation {
                            encoded_key: b"a".to_vec(),
                            value: Some(b"two".to_vec()),
                            revision: Revision::new(2),
                        }],
                    },
                ],
                None,
            )
            .unwrap();
        assert_eq!(wal.sync_count(), syncs_before + 1);
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].commit_sequence, 1);
        assert_eq!(reports[1].commit_sequence, 2);
        assert!(reports[1].physical_commit_lsn.get() > 2);
        assert_ne!(
            reports[1].physical_commit_lsn.get(),
            reports[1].commit_sequence
        );
        let bytes = reports
            .iter()
            .map(|report| report.bytes_written)
            .sum::<usize>();
        let mut reopened =
            LogicalWalLog::open(wal.into_file(), identity(), Lsn::ZERO, None).unwrap();
        let recovered = reopened.take_recovery_transactions();
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].commit_sequence, 1);
        assert_eq!(recovered[0].mutations[1].value, None);
        assert_eq!(
            recovered[1].mutations[0].value.as_deref(),
            Some(&b"two"[..])
        );
        assert_eq!(reopened.last_commit_sequence(), Some(2));
        assert!(bytes > 0);
    }

    #[test]
    fn logical_wal_retries_short_writes_and_fails_before_commit_without_appending() {
        let mut wal =
            LogicalWalLog::open(CountingFile::new(Some(7)), identity(), Lsn::ZERO, None).unwrap();
        let initial_bytes = wal.wal_bytes().unwrap();
        let mut injector = FailAtPoint("before_commit_record");
        assert!(
            wal.append_group(
                &[LogicalWalTransaction {
                    commit_sequence: 1,
                    mutations: vec![LogicalWalMutation {
                        encoded_key: b"key".to_vec(),
                        value: Some(b"value".to_vec()),
                        revision: Revision::new(1),
                    }],
                }],
                Some(&mut injector),
            )
            .is_err()
        );
        assert_eq!(wal.wal_bytes().unwrap(), initial_bytes);
        assert_eq!(wal.next_commit_sequence(), 1);
        wal.append_group(
            &[LogicalWalTransaction {
                commit_sequence: 1,
                mutations: vec![LogicalWalMutation {
                    encoded_key: b"key".to_vec(),
                    value: Some(b"value".to_vec()),
                    revision: Revision::new(1),
                }],
            }],
            None,
        )
        .unwrap();
        let counted_file = wal.into_file();
        assert!(counted_file.write_calls > 1);
        assert_eq!(counted_file.sync_calls, 2);
        let mut reopened = LogicalWalLog::open(counted_file, identity(), Lsn::ZERO, None).unwrap();
        assert_eq!(reopened.take_recovery_transactions().len(), 1);
    }

    #[test]
    fn logical_wal_discards_an_incomplete_transaction_and_reuses_its_sequence() {
        let mut wal =
            LogicalWalLog::open(MemoryFile::default(), identity(), Lsn::ZERO, None).unwrap();
        wal.append_group(
            &[LogicalWalTransaction {
                commit_sequence: 1,
                mutations: vec![LogicalWalMutation {
                    encoded_key: b"key".to_vec(),
                    value: Some(b"value".to_vec()),
                    revision: Revision::new(1),
                }],
            }],
            None,
        )
        .unwrap();
        let mut file = wal.into_file();
        let original_length = file.len().unwrap();
        file.set_len(original_length - 10).unwrap();
        let mut reopened = LogicalWalLog::open(file, identity(), Lsn::ZERO, None).unwrap();
        assert!(reopened.take_recovery_transactions().is_empty());
        assert_eq!(reopened.next_commit_sequence(), 1);
        assert_eq!(reopened.scan_report().torn_tail_bytes, 142);
        reopened
            .append_group(
                &[LogicalWalTransaction {
                    commit_sequence: 1,
                    mutations: vec![LogicalWalMutation {
                        encoded_key: b"key".to_vec(),
                        value: Some(b"replacement".to_vec()),
                        revision: Revision::new(1),
                    }],
                }],
                None,
            )
            .unwrap();
        assert_eq!(reopened.take_recovery_transactions().len(), 0);
        let bytes = reopened.into_file();
        let mut twice = LogicalWalLog::open(bytes, identity(), Lsn::ZERO, None).unwrap();
        assert_eq!(twice.take_recovery_transactions().len(), 1);
    }

    #[test]
    fn logical_wal_discards_torn_put_and_delete_records_and_keeps_complete_group_prefix() {
        for value in [Some(b"value".to_vec()), None] {
            let mut wal =
                LogicalWalLog::open(MemoryFile::default(), identity(), Lsn::ZERO, None).unwrap();
            wal.append_group(
                &[LogicalWalTransaction {
                    commit_sequence: 1,
                    mutations: vec![LogicalWalMutation {
                        encoded_key: b"key".to_vec(),
                        value: value.clone(),
                        revision: Revision::new(1),
                    }],
                }],
                None,
            )
            .unwrap();
            let mut file = wal.into_file();
            let payload_length = 16 + 3 + value.as_ref().map_or(0, Vec::len);
            let mutation_frame_length = WAL_HEADER_SIZE + payload_length + WAL_TRAILER_SIZE;
            file.set_len((INIT_FRAME_SIZE + mutation_frame_length - 5) as u64)
                .unwrap();
            let mut reopened = LogicalWalLog::open(file, identity(), Lsn::ZERO, None).unwrap();
            assert!(reopened.take_recovery_transactions().is_empty());
            assert_eq!(reopened.next_commit_sequence(), 1);
        }

        let mut wal =
            LogicalWalLog::open(MemoryFile::default(), identity(), Lsn::ZERO, None).unwrap();
        wal.append_group(
            &[
                LogicalWalTransaction {
                    commit_sequence: 1,
                    mutations: vec![LogicalWalMutation {
                        encoded_key: b"first".to_vec(),
                        value: Some(b"complete".to_vec()),
                        revision: Revision::new(1),
                    }],
                },
                LogicalWalTransaction {
                    commit_sequence: 2,
                    mutations: vec![LogicalWalMutation {
                        encoded_key: b"second".to_vec(),
                        value: None,
                        revision: Revision::new(2),
                    }],
                },
            ],
            None,
        )
        .unwrap();
        let mut file = wal.into_file();
        let length = file.len().unwrap();
        file.set_len(length - 10).unwrap();
        let mut reopened = LogicalWalLog::open(file, identity(), Lsn::ZERO, None).unwrap();
        let recovered = reopened.take_recovery_transactions();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].commit_sequence, 1);
        assert_eq!(reopened.next_commit_sequence(), 2);
    }

    #[test]
    fn logical_wal_rejects_corrupt_commit_digest_and_mixed_v3_dispatch() {
        let mut wal =
            LogicalWalLog::open(MemoryFile::default(), identity(), Lsn::ZERO, None).unwrap();
        wal.append_group(
            &[LogicalWalTransaction {
                commit_sequence: 1,
                mutations: vec![LogicalWalMutation {
                    encoded_key: b"key".to_vec(),
                    value: Some(b"value".to_vec()),
                    revision: Revision::new(1),
                }],
            }],
            None,
        )
        .unwrap();
        let mut bytes = wal.into_file();
        let frame_offset = INIT_FRAME_SIZE as u64 + WAL_HEADER_SIZE as u64 + 16;
        let mut byte = [0u8; 1];
        bytes.read_at(frame_offset, &mut byte).unwrap();
        byte[0] ^= 1;
        bytes.write_at(frame_offset, &byte).unwrap();
        assert!(matches!(
            LogicalWalLog::open(bytes, identity(), Lsn::ZERO, None),
            Err(Error::Corruption(_))
        ));

        let mut wal =
            LogicalWalLog::open(MemoryFile::default(), identity(), Lsn::ZERO, None).unwrap();
        wal.append_group(
            &[LogicalWalTransaction {
                commit_sequence: 1,
                mutations: vec![LogicalWalMutation {
                    encoded_key: b"key".to_vec(),
                    value: Some(b"value".to_vec()),
                    revision: Revision::new(1),
                }],
            }],
            None,
        )
        .unwrap();
        let mut frames = parse_wal_frames_for_test(&wal.into_file().0);
        let commit_frame = frames
            .iter_mut()
            .find(|frame| frame.record_type == WalRecordType::Commit)
            .unwrap();
        commit_frame.payload[12] ^= 1;
        let corrupted_digest = MemoryFile(serialize_wal_frames_for_test(&frames));
        assert!(matches!(
            LogicalWalLog::open(corrupted_digest, identity(), Lsn::ZERO, None),
            Err(Error::Corruption(_))
        ));

        let v4 = LogicalWalLog::open(MemoryFile::default(), identity(), Lsn::ZERO, None).unwrap();
        assert!(matches!(
            WalLog::open_with_page_image_format(
                v4.into_file(),
                identity(),
                WalPageImageFormat::ExperimentalBlink,
            ),
            Err(Error::UnsupportedFormat(_))
        ));
    }
}
