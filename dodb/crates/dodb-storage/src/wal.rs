//! Versioned redo-only write-ahead logging.
//!
//! The WAL is deliberately independent from the data file.  It contains
//! complete encoded after-images and an explicit commit marker.  A page image
//! is never replayed unless the matching commit marker is complete and its
//! metadata digest matches the image sequence.

use std::io::ErrorKind;
use std::time::Instant;

use dodb_core::{Error, Lsn, PageId, Result, ShardEpoch, ShardId, TenantId};

use crate::durable_file::DurableFile;
use crate::fault::FaultInjector;
use crate::page::{PAGE_SIZE, decode_page_at};
use crate::superblock::decode_superblock;

pub const WAL_FORMAT_VERSION: u16 = 2;
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
const INIT_FRAME_SIZE: usize = WAL_HEADER_SIZE + INIT_PAYLOAD_SIZE + WAL_TRAILER_SIZE;
const PAGE_IMAGE_PAYLOAD_SIZE: usize = 8 + PAGE_SIZE;
const COMMIT_PAYLOAD_SIZE: usize = 16;

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
}

impl WalRecordType {
    fn decode(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Init),
            2 => Ok(Self::PageImage),
            3 => Ok(Self::Commit),
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
pub struct CommittedWalBatch {
    pub batch_id: u64,
    pub commit_lsn: Lsn,
    pub pages: Vec<WalPageImage>,
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
struct Frame {
    format_version: u16,
    record_type: WalRecordType,
    record_lsn: Lsn,
    batch_id: u64,
    record_index: u32,
    payload: Vec<u8>,
}

#[derive(Clone, Debug)]
struct PendingBatch {
    batch_id: u64,
    first_record_lsn: Lsn,
    pages: Vec<WalPageImage>,
    digest_input: Vec<u8>,
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

/// A WAL file with a validated in-memory index of complete commits.
pub struct WalLog<F: DurableFile> {
    file: F,
    identity: WalIdentity,
    format_version: u16,
    next_lsn: Lsn,
    next_batch_id: u64,
    history_start_lsn: Lsn,
    committed: Vec<CommittedWalBatch>,
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
            && is_torn_initialization_prefix(&mut file, &identity, start_after_lsn, length)?
        {
            file.set_len(0)
                .map_err(|error| Error::recovery(format!("WAL torn INIT reset failed: {error}")))?;
            file.sync_data().map_err(|error| {
                Error::durability(format!("WAL torn INIT reset sync failed: {error}"))
            })?;
            return Self::initialize_empty(file, identity, start_after_lsn, page_image_format);
        }

        let (
            next_lsn,
            next_batch_id,
            format_version,
            history_start_lsn,
            committed,
            mut report,
            valid_length,
        ) = scan_wal(&mut file, &identity, page_image_format)?;
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
        let page_images = report.replayable_pages;
        Ok(Self {
            file,
            identity,
            format_version,
            next_lsn,
            next_batch_id,
            history_start_lsn,
            committed,
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
            format_version: WAL_FORMAT_VERSION,
            next_lsn,
            next_batch_id: 1,
            history_start_lsn: start_after_lsn,
            committed: Vec::new(),
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

    pub fn committed_batches(&self) -> &[CommittedWalBatch] {
        &self.committed
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
        self.format_version = WAL_FORMAT_VERSION;
        self.next_lsn = checkpoint_lsn
            .get()
            .checked_add(1)
            .map(Lsn::new)
            .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
        self.next_batch_id = 1;
        self.committed.clear();
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
            committed_batches: self.committed.len(),
            page_images: self.page_images,
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
        self.append_group_inner(commits, injector, PageImageValidationMode::Strict)
    }

    /// Appends page images produced moments earlier by this storage engine's
    /// internal Blink page and superblock encoders in the same process. Never
    /// use this path for caller-provided bytes. Fault-injected calls delegate
    /// to the strict public path.
    pub(crate) fn append_group_trusted_internal(
        &mut self,
        commits: &[WalCommit],
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        if injector.is_some() {
            return self.append_group(commits, injector);
        }
        self.append_group_inner(commits, None, PageImageValidationMode::TrustedInternal)
    }

    fn append_group_inner(
        &mut self,
        commits: &[WalCommit],
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
        validation_mode: PageImageValidationMode,
    ) -> Result<Vec<WalAppendReport>> {
        if commits.is_empty() {
            return Err(Error::invalid_input("a WAL group must contain a commit"));
        }

        let append_started = Instant::now();
        hit(&mut injector, "before_wal_append")?;
        let (reports, next_lsn, next_batch_id) = if injector.is_some() {
            self.append_group_fault_injectable(commits, &mut injector)?
        } else {
            let encode_started = Instant::now();
            let encoded = self.encode_group(commits, validation_mode)?;
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

        let committed = commits
            .iter()
            .map(|commit| CommittedWalBatch {
                batch_id: commit.batch_id,
                commit_lsn: commit.commit_lsn,
                pages: commit.pages.clone(),
            })
            .collect::<Vec<_>>();

        self.append_nanos = self
            .append_nanos
            .checked_add(elapsed_nanos(append_started)?)
            .ok_or_else(|| Error::invariant("WAL append timing overflow"))?;
        hit(&mut injector, "after_group_records_written")?;
        hit(&mut injector, "before_wal_sync")?;
        hit(&mut injector, "during_wal_sync")?;
        let sync_started = Instant::now();
        self.file.sync_data().map_err(|error| {
            Error::durability(format!(
                "WAL group sync failed at commit LSN {}: {error}",
                commits.last().expect("non-empty WAL group").commit_lsn
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
        hit(&mut injector, "after_wal_sync")?;

        self.committed.extend(committed);
        self.next_lsn = next_lsn;
        self.next_batch_id = next_batch_id;
        let record_count = commits.iter().try_fold(0usize, |count, commit| {
            count
                .checked_add(commit.pages.len() + 1)
                .ok_or_else(|| Error::invariant("WAL record count overflow"))
        })?;
        self.scan_report.records_scanned = self
            .scan_report
            .records_scanned
            .checked_add(record_count)
            .ok_or_else(|| Error::invariant("WAL record count overflow"))?;
        self.scan_report.committed_batches += commits.len();
        self.scan_report.replayable_pages += commits
            .iter()
            .map(|commit| commit.pages.len())
            .sum::<usize>();
        self.page_images = self
            .page_images
            .checked_add(commits.iter().map(|commit| commit.pages.len()).sum())
            .ok_or_else(|| Error::invariant("WAL page-image count overflow"))?;

        Ok(reports)
    }

    fn append_group_fault_injectable(
        &mut self,
        commits: &[WalCommit],
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<(Vec<WalAppendReport>, Lsn, u64)> {
        let mut next_lsn = self.next_lsn;
        let mut next_batch_id = self.next_batch_id;
        let mut reports = Vec::with_capacity(commits.len());
        for commit in commits {
            let report = self.append_group_commit(commit, next_lsn, next_batch_id, injector)?;
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
    ) -> Result<EncodedWalGroup> {
        let mut attribution = WalEncodeAttribution::default();
        let mut capacity = 0usize;
        for commit in commits {
            if commit.pages.is_empty() {
                return Err(Error::invalid_input(
                    "a WAL commit must contain at least one page image",
                ));
            }
            let page_frames = commit
                .pages
                .len()
                .checked_mul(WAL_HEADER_SIZE + PAGE_IMAGE_PAYLOAD_SIZE + WAL_TRAILER_SIZE)
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

        for commit in commits {
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
            for (page_index, page) in commit.pages.iter().enumerate() {
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
                let payload_crc_started = Instant::now();
                let payload_checksum = crc32c::crc32c(&page_id_bytes);
                let payload_checksum = crc32c::crc32c_append(payload_checksum, &page.image);
                attribution.group_page_payload_crc_nanos += elapsed_nanos(payload_crc_started)?;
                let digest_started = Instant::now();
                digest = crc32c::crc32c_append(digest, &page_id_bytes);
                digest = crc32c::crc32c_append(digest, &page.image);
                attribution.group_commit_digest_crc_nanos += elapsed_nanos(digest_started)?;
                let record_lsn = first_record_lsn
                    .get()
                    .checked_add(
                        u64::try_from(page_index)
                            .map_err(|_| Error::invalid_input("WAL record index overflows"))?,
                    )
                    .map(Lsn::new)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?;
                let record_index = u32::try_from(page_index)
                    .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?;
                let frame_length = append_page_frame_direct(
                    &mut bytes,
                    self.format_version,
                    record_lsn,
                    commit.batch_id,
                    record_index,
                    &page_id_bytes,
                    &page.image,
                    payload_checksum,
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

        let mut digest_input = Vec::with_capacity(
            pages
                .len()
                .checked_mul(PAGE_IMAGE_PAYLOAD_SIZE)
                .ok_or_else(|| Error::invalid_input("WAL digest input is too large"))?,
        );
        let mut bytes_written = 0usize;
        for (index, page) in pages.iter().enumerate() {
            validate_page_image_lsn(page, commit.commit_lsn)?;
            let payload = encode_page_image(page, self.page_image_format)?;
            digest_input.extend_from_slice(&payload);
            let record_lsn = Lsn::new(
                first_record_lsn
                    .get()
                    .checked_add(index as u64)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            );
            bytes_written = bytes_written
                .checked_add(
                    self.append_frame(
                        WalRecordType::PageImage,
                        record_lsn,
                        commit.batch_id,
                        u32::try_from(index)
                            .map_err(|_| Error::invalid_input("WAL record index overflows u32"))?,
                        &payload,
                        &mut *injector,
                    )?,
                )
                .ok_or_else(|| Error::invariant("WAL byte count overflow"))?;
        }
        hit(&mut *injector, "after_page_images_written")?;

        let digest = crc32c::crc32c(&digest_input);
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
        write_all_at_counted(
            &mut self.file,
            offset,
            &frame[..WAL_HEADER_SIZE],
            &mut self.physical_write_calls,
        )?;
        hit(injector, "during_wal_payload_write")?;
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
) -> Result<bool> {
    let length = usize::try_from(length)
        .map_err(|_| Error::recovery("WAL torn INIT length does not fit usize"))?;
    let expected = encode_frame(
        WAL_FORMAT_VERSION,
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

fn append_page_frame_direct(
    output: &mut Vec<u8>,
    format_version: u16,
    record_lsn: Lsn,
    batch_id: u64,
    record_index: u32,
    page_id: &[u8; 8],
    page_image: &[u8; PAGE_SIZE],
    payload_checksum: u32,
    attribution: &mut WalEncodeAttribution,
) -> Result<usize> {
    append_frame_parts_direct(
        output,
        format_version,
        WalRecordType::PageImage,
        record_lsn,
        batch_id,
        record_index,
        PAGE_IMAGE_PAYLOAD_SIZE,
        payload_checksum,
        &[page_id, page_image],
        attribution,
    )
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
        WalRecordType::PageImage => {
            attribution.group_page_header_crc_nanos += header_crc_nanos;
        }
        WalRecordType::Commit => {
            attribution.group_commit_header_crc_nanos += header_crc_nanos;
        }
        WalRecordType::Init => {}
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
        WalRecordType::PageImage => {
            attribution.group_page_direct_encode_nanos += direct_nanos;
        }
        WalRecordType::Commit => {
            attribution.group_commit_direct_encode_nanos += direct_nanos;
        }
        WalRecordType::Init => {}
    }
    Ok(frame_length)
}

fn elapsed_nanos(started: Instant) -> Result<u64> {
    u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::invariant("WAL timing does not fit u64"))
}

type WalScanResult = (
    Lsn,
    u64,
    u16,
    Lsn,
    Vec<CommittedWalBatch>,
    WalScanReport,
    u64,
);

fn scan_wal<F: DurableFile>(
    file: &mut F,
    identity: &WalIdentity,
    page_image_format: WalPageImageFormat,
) -> Result<WalScanResult> {
    let length = file.len()?;
    let mut offset = 0u64;
    let mut previous_lsn = None;
    let mut frames = Vec::new();
    let mut report = WalScanReport::default();
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
        if frames
            .first()
            .is_some_and(|frame: &Frame| frame.format_version != format_version)
        {
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
        frames.push(Frame {
            format_version,
            record_type,
            record_lsn: Lsn::new(record_lsn),
            batch_id,
            record_index,
            payload,
        });
        report.records_scanned += 1;
        offset += frame_length_u64;
    }

    let valid_length = offset;
    let first = frames
        .first()
        .ok_or_else(|| Error::corruption("WAL has no complete initialization record"))?;
    if first.record_type != WalRecordType::Init
        || first.record_lsn != Lsn::ZERO
        || first.batch_id != 0
        || first.record_index != 0
    {
        return Err(Error::corruption(
            "WAL does not begin with the required initialization record",
        ));
    }
    let history_start_lsn = verify_identity(first.format_version, &first.payload, identity)?;

    let mut committed = Vec::new();
    let mut pending: Option<PendingBatch> = None;
    let mut highest_batch_id_seen = 0u64;
    let mut max_batch_id = 0u64;
    for frame in frames.iter().skip(1) {
        if frame.record_lsn <= history_start_lsn {
            return Err(Error::corruption(
                "WAL record LSN is not after the initialization history boundary",
            ));
        }
        max_batch_id = max_batch_id.max(frame.batch_id);
        match frame.record_type {
            WalRecordType::Init => {
                return Err(Error::corruption(
                    "WAL contains a second initialization record",
                ));
            }
            WalRecordType::PageImage => {
                let page = decode_page_image(&frame.payload)?;
                validate_page_image(&page, page_image_format)?;
                if pending
                    .as_ref()
                    .is_none_or(|pending| pending.batch_id != frame.batch_id)
                {
                    if frame.batch_id <= highest_batch_id_seen {
                        return Err(Error::corruption(
                            "WAL batch IDs are not strictly increasing",
                        ));
                    }
                    highest_batch_id_seen = frame.batch_id;
                    pending = Some(PendingBatch {
                        batch_id: frame.batch_id,
                        first_record_lsn: frame.record_lsn,
                        pages: Vec::new(),
                        digest_input: Vec::new(),
                    });
                }
                let current = pending.as_mut().expect("pending batch was created");
                if frame.record_index != current.pages.len() as u32 {
                    return Err(Error::corruption(
                        "WAL page-image record index is not contiguous",
                    ));
                }
                current.digest_input.extend_from_slice(&frame.payload);
                current.pages.push(page);
            }
            WalRecordType::Commit => {
                let current = pending.take().ok_or_else(|| {
                    Error::corruption("WAL commit has no matching page-image records")
                })?;
                if current.batch_id != frame.batch_id
                    || frame.record_index != current.pages.len() as u32
                {
                    return Err(Error::corruption(
                        "WAL commit does not match its page-image sequence",
                    ));
                }
                let (first_lsn, page_count, digest) = decode_commit_payload(&frame.payload)?;
                if first_lsn != current.first_record_lsn
                    || page_count != current.pages.len()
                    || crc32c::crc32c(&current.digest_input) != digest
                {
                    return Err(Error::corruption(
                        "WAL commit metadata does not match page images",
                    ));
                }
                for page in &current.pages {
                    validate_page_image_lsn(page, frame.record_lsn)?;
                }
                committed.push(CommittedWalBatch {
                    batch_id: frame.batch_id,
                    commit_lsn: frame.record_lsn,
                    pages: current.pages,
                });
            }
        }
    }
    report.committed_batches = committed.len();
    report.replayable_pages = committed.iter().map(|batch| batch.pages.len()).sum();
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
    Ok((
        next_lsn,
        next_batch_id,
        first.format_version,
        history_start_lsn,
        committed,
        report,
        valid_length,
    ))
}

fn decode_header(bytes: &[u8]) -> Result<(u16, WalRecordType, usize, usize, u64, u64, u32)> {
    if bytes.len() != WAL_HEADER_SIZE {
        return Err(Error::corruption("WAL header has an invalid length"));
    }
    if bytes[0..4] != WAL_MAGIC {
        return Err(Error::corruption("WAL magic mismatch"));
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    if version != WAL_FORMAT_VERSION && version != LEGACY_WAL_FORMAT_VERSION {
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
            WalPageImageFormat::Baseline => {
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
            WalPageImageFormat::Baseline => {
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
    let expected_length = if version == LEGACY_WAL_FORMAT_VERSION {
        LEGACY_INIT_PAYLOAD_SIZE
    } else {
        INIT_PAYLOAD_SIZE
    };
    if payload.len() != expected_length {
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
        assert_eq!(wal.committed_batches()[0].commit_lsn, commit_lsn);
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
        assert!(wal.committed_batches().is_empty());
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
        assert_eq!(reopened.committed_batches().len(), 10);
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
        assert_eq!(reopened.committed_batches().len(), 1);
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
}
