use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dodb_core::{Error, Lsn, PageId, Result, ShardEpoch, ShardId, TenantId};

use crate::durable_file::ProductionFile;
use crate::fault::FaultInjector;
use crate::page::PAGE_SIZE;
use crate::wal::{
    RecoveredWalPage, WalAppendReport, WalCommit, WalDeltaRequest, WalIdentity, WalLog, WalMetrics,
    WalPageImageFormat,
};

const MANIFEST_MAGIC: [u8; 8] = *b"DBLKSWM1";
const MANIFEST_VERSION: u32 = 2;
const CHECKSUM_SIZE: usize = 4;
const FIXED_SIZE: usize = 120;
const SEGMENT_SIZE: usize = 48;
const MAX_SEGMENTS: usize = 65_536;
static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const SEGMENT_DIRECTORY_SUFFIX: &str = "segments";
const SEGMENT_MANIFEST_SUFFIX: &str = "segments.manifest";

pub(crate) fn manifest_exists(database_path: &Path) -> bool {
    sidecar_path(database_path, SEGMENT_MANIFEST_SUFFIX).is_file()
}

pub(crate) fn artifacts_exist(database_path: &Path) -> bool {
    manifest_exists(database_path) || sidecar_path(database_path, SEGMENT_DIRECTORY_SUFFIX).exists()
}

pub(crate) struct SegmentedWal {
    manifest_path: PathBuf,
    segment_directory: PathBuf,
    manifest: SegmentedWalManifest,
    active: WalLog<ProductionFile>,
    max_segment_bytes: u64,
    closed_metrics: WalMetrics,
    closed_wal_bytes: u64,
}

impl SegmentedWal {
    pub(crate) fn create(
        database_path: &Path,
        identity: WalIdentity,
        root_page_id: PageId,
        free_list_head: Option<PageId>,
        high_water_page_id: PageId,
        format_version: u32,
        max_segment_bytes: u64,
    ) -> Result<Self> {
        let manifest_path = sidecar_path(database_path, SEGMENT_MANIFEST_SUFFIX);
        let segment_directory = sidecar_path(database_path, SEGMENT_DIRECTORY_SUFFIX);
        if manifest_path.exists() || segment_directory.exists() {
            return Err(Error::invalid_input(
                "segmented WAL already exists without an openable manifest",
            ));
        }
        fs::create_dir_all(&segment_directory)?;
        let first_path = segment_path(&segment_directory, 1);
        let first_file = create_new_file(&first_path)?;
        let mut active = WalLog::open_with_segment_boundary(
            first_file,
            identity.clone(),
            WalPageImageFormat::ExperimentalBlink,
            Lsn::ZERO,
            Some(1),
            true,
            true,
            None,
        )?;
        active.sync_all()?;
        sync_directory(&segment_directory)?;
        sync_directory(parent_directory(&segment_directory)?)?;
        let manifest = SegmentedWalManifest {
            generation: 1,
            identity,
            checkpoint_lsn: Lsn::ZERO,
            root_page_id,
            free_list_head,
            high_water_page_id,
            format_version,
            segments: vec![WalSegmentEntry {
                number: 1,
                start_lsn: Lsn::new(1),
                end_lsn: None,
                start_batch_id: 1,
                end_batch_id: None,
                closed: false,
            }],
            active_segment: 1,
            next_segment: 2,
        };
        manifest.write_atomic(&manifest_path)?;
        Ok(Self {
            manifest_path,
            segment_directory,
            manifest,
            active,
            max_segment_bytes,
            closed_metrics: WalMetrics::default(),
            closed_wal_bytes: 0,
        })
    }

    pub(crate) fn open(
        database_path: &Path,
        identity: WalIdentity,
        max_segment_bytes: u64,
    ) -> Result<(Self, BTreeMap<PageId, RecoveredWalPage>)> {
        let manifest_path = sidecar_path(database_path, SEGMENT_MANIFEST_SUFFIX);
        let segment_directory = sidecar_path(database_path, SEGMENT_DIRECTORY_SUFFIX);
        let manifest = SegmentedWalManifest::read(&manifest_path)?;
        if manifest.identity != identity {
            return Err(Error::corruption("segmented WAL identity mismatch"));
        }
        if manifest.format_version != u32::from(super::BLINK_SUPERBLOCK_VERSION) {
            return Err(Error::unsupported_format(
                "segmented WAL tree format is unsupported",
            ));
        }
        let mut recovered_pages = BTreeMap::new();
        let mut active = None;
        let mut closed_metrics = WalMetrics::default();
        let mut closed_wal_bytes = 0u64;
        for segment in &manifest.segments {
            let path = segment_path(&segment_directory, segment.number);
            if !path.is_file() {
                return Err(Error::corruption(format!(
                    "required WAL segment {} is missing",
                    segment.number
                )));
            }
            if fs::metadata(&path)?.len() == 0 {
                return Err(Error::corruption(format!(
                    "WAL segment {} has no initialization record",
                    segment.number
                )));
            }
            let start_after_lsn = segment
                .start_lsn
                .get()
                .checked_sub(1)
                .map(Lsn::new)
                .ok_or_else(|| Error::corruption("WAL segment starts at LSN zero"))?;
            let mut wal = WalLog::open_with_segment_boundary(
                ProductionFile::open(&path)?,
                identity.clone(),
                WalPageImageFormat::ExperimentalBlink,
                start_after_lsn,
                Some(segment.start_batch_id),
                !segment.closed,
                false,
                None,
            )?;
            let scan = wal.scan_report();
            if segment.closed && scan.torn_tail_bytes != 0 {
                return Err(Error::corruption(format!(
                    "closed WAL segment {} has a torn tail",
                    segment.number
                )));
            }
            if wal.history_start_lsn() != start_after_lsn {
                return Err(Error::corruption(format!(
                    "WAL segment {} starts at the wrong LSN",
                    segment.number
                )));
            }
            if wal.segment_start_batch_id() != Some(segment.start_batch_id) {
                return Err(Error::corruption(format!(
                    "WAL segment {} starts at the wrong transaction identifier",
                    segment.number
                )));
            }
            if segment.closed {
                let end_lsn = segment
                    .end_lsn
                    .ok_or_else(|| Error::corruption("closed WAL segment has no end LSN"))?;
                let end_batch_id = segment.end_batch_id.ok_or_else(|| {
                    Error::corruption("closed WAL segment has no end transaction identifier")
                })?;
                if wal.last_commit_lsn() != Some(end_lsn)
                    || wal.next_batch_id().checked_sub(1) != Some(end_batch_id)
                {
                    return Err(Error::corruption(format!(
                        "closed WAL segment {} does not match its manifest boundary",
                        segment.number
                    )));
                }
            } else if segment.end_lsn.is_some() || segment.end_batch_id.is_some() {
                return Err(Error::corruption(
                    "active WAL segment has a finalized boundary",
                ));
            }
            for (page_id, page) in wal.take_recovery_pages() {
                recovered_pages.insert(page_id, page);
            }
            if segment.number == manifest.active_segment {
                active = Some(wal);
            } else {
                let metrics = wal.metrics()?;
                closed_wal_bytes = closed_wal_bytes
                    .checked_add(metrics.wal_bytes)
                    .ok_or_else(|| Error::invariant("retained WAL byte count overflow"))?;
                closed_metrics.accumulate_counters(&metrics);
            }
        }
        let active = active.ok_or_else(|| Error::corruption("active WAL segment is missing"))?;
        let wal = Self {
            manifest_path,
            segment_directory,
            manifest,
            active,
            max_segment_bytes,
            closed_metrics,
            closed_wal_bytes,
        };
        Ok((wal, recovered_pages))
    }

    pub(crate) fn next_lsn(&self) -> Lsn {
        self.active.next_lsn()
    }

    pub(crate) fn next_batch_id(&self) -> u64 {
        self.active.next_batch_id()
    }

    pub(crate) fn metrics(&self) -> Result<WalMetrics> {
        let mut metrics = self.active.metrics()?;
        metrics.accumulate_counters(&self.closed_metrics);
        metrics.wal_bytes = metrics
            .wal_bytes
            .checked_add(self.closed_wal_bytes)
            .ok_or_else(|| Error::invariant("retained WAL byte count overflow"))?;
        Ok(metrics)
    }

    pub(crate) fn append_group(
        &mut self,
        commits: &[WalCommit],
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        self.rotate_if_needed(injector.as_deref_mut())?;
        self.active.append_group(commits, injector)
    }

    pub(crate) fn append_group_trusted_internal_with_page_deltas(
        &mut self,
        commits: &[WalCommit],
        delta: &mut WalDeltaRequest<'_, '_, '_>,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<Vec<WalAppendReport>> {
        self.rotate_if_needed(injector.as_deref_mut())?;
        self.active
            .append_group_trusted_internal_with_page_deltas(commits, delta, injector)
    }

    pub(crate) fn manifest(&self) -> &SegmentedWalManifest {
        &self.manifest
    }

    pub(crate) fn last_commit_lsn(&self) -> Lsn {
        self.active
            .last_commit_lsn()
            .or_else(|| {
                self.manifest
                    .segments
                    .iter()
                    .rev()
                    .find_map(|segment| segment.end_lsn)
            })
            .unwrap_or(self.manifest.checkpoint_lsn)
    }

    fn rotate_if_needed(
        &mut self,
        injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        if self.active.metrics()?.wal_bytes < self.max_segment_bytes
            || self.active.last_commit_lsn().is_none()
        {
            return Ok(());
        }
        self.rotate(injector)
    }

    pub(crate) fn rotate(
        &mut self,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        if self.active.last_commit_lsn().is_none() {
            return Ok(());
        }
        hit(&mut injector, "before_wal_segment_close")?;
        let end_lsn = self
            .active
            .last_commit_lsn()
            .ok_or_else(|| Error::invariant("nonempty WAL segment has no commit"))?;
        let end_batch_id = self
            .active
            .next_batch_id()
            .checked_sub(1)
            .ok_or_else(|| Error::invariant("WAL transaction identifier underflow"))?;
        let start_lsn = self.active.next_lsn();
        let start_batch_id = self.active.next_batch_id();
        let number = self.manifest.next_segment;
        let mut next_manifest = self.manifest.clone();
        next_manifest.generation = next_manifest
            .generation
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL manifest generation exhausted"))?;
        let last_segment = next_manifest
            .segments
            .last_mut()
            .ok_or_else(|| Error::invariant("WAL manifest has no active segment"))?;
        last_segment.end_lsn = Some(end_lsn);
        last_segment.end_batch_id = Some(end_batch_id);
        last_segment.closed = true;
        let new_path = segment_path(&self.segment_directory, number);
        hit(&mut injector, "before_wal_segment_create")?;
        let start_after_lsn = Lsn::new(start_lsn.get().saturating_sub(1));
        let mut new_active = if new_path.exists() {
            let candidate = WalLog::open_with_segment_boundary(
                ProductionFile::open(&new_path)?,
                self.manifest.identity.clone(),
                WalPageImageFormat::ExperimentalBlink,
                start_after_lsn,
                Some(start_batch_id),
                false,
                false,
                None,
            )?;
            if candidate.history_start_lsn() != start_after_lsn
                || candidate.segment_start_batch_id() != Some(start_batch_id)
                || candidate.last_commit_lsn().is_some()
                || candidate.scan_report().committed_batches != 0
            {
                return Err(Error::corruption(
                    "unpublished WAL segment does not match the next segment boundary",
                ));
            }
            candidate
        } else {
            WalLog::open_with_segment_boundary(
                create_new_file(&new_path)?,
                self.manifest.identity.clone(),
                WalPageImageFormat::ExperimentalBlink,
                start_after_lsn,
                Some(start_batch_id),
                true,
                true,
                None,
            )?
        };
        hit(&mut injector, "before_wal_segment_file_sync")?;
        new_active.sync_all()?;
        hit(&mut injector, "after_wal_segment_file_sync")?;
        hit(&mut injector, "before_wal_segment_directory_sync")?;
        sync_directory(&self.segment_directory)?;
        hit(&mut injector, "after_wal_segment_directory_sync")?;
        next_manifest.segments.push(WalSegmentEntry {
            number,
            start_lsn,
            end_lsn: None,
            start_batch_id,
            end_batch_id: None,
            closed: false,
        });
        next_manifest.active_segment = number;
        next_manifest.next_segment = number
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL segment number exhausted"))?;
        hit(&mut injector, "before_wal_manifest_publish")?;
        next_manifest
            .write_atomic_with_fault_injector(&self.manifest_path, injector.as_deref_mut())?;
        hit(&mut injector, "after_wal_manifest_publish")?;
        let metrics = self.active.metrics()?;
        self.closed_wal_bytes = self
            .closed_wal_bytes
            .checked_add(metrics.wal_bytes)
            .ok_or_else(|| Error::invariant("retained WAL byte count overflow"))?;
        self.closed_metrics.accumulate_counters(&metrics);
        self.manifest = next_manifest;
        self.active = new_active;
        Ok(())
    }

    pub(crate) fn advance_checkpoint(
        &mut self,
        superblock: &super::BlinkSuperblock,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<u64> {
        let checkpoint_lsn = superblock.checkpoint_lsn;
        if checkpoint_lsn < self.manifest.checkpoint_lsn || checkpoint_lsn > self.last_commit_lsn()
        {
            return Err(Error::checkpoint("invalid segmented WAL checkpoint target"));
        }
        let mut next_manifest = self.manifest.clone();
        next_manifest.generation = next_manifest
            .generation
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL manifest generation exhausted"))?;
        next_manifest.checkpoint_lsn = checkpoint_lsn;
        next_manifest.root_page_id = superblock.root_page_id;
        next_manifest.free_list_head = superblock.free_list_head;
        next_manifest.high_water_page_id = superblock.high_water_page_id;
        let mut reclaimed_bytes = 0u64;
        let mut obsolete_numbers = Vec::new();
        for segment in &self.manifest.segments {
            if segment.closed && segment.end_lsn.is_some_and(|end| end <= checkpoint_lsn) {
                obsolete_numbers.push(segment.number);
                reclaimed_bytes = reclaimed_bytes
                    .checked_add(
                        fs::metadata(segment_path(&self.segment_directory, segment.number))?.len(),
                    )
                    .ok_or_else(|| Error::invariant("reclaimed WAL byte count overflow"))?;
            }
        }
        next_manifest.segments.retain(|segment| {
            !segment.closed || segment.end_lsn.is_some_and(|end| end > checkpoint_lsn)
        });
        next_manifest.validate()?;
        hit(&mut injector, "before_checkpoint_manifest_publish")?;
        next_manifest
            .write_atomic_with_fault_injector(&self.manifest_path, injector.as_deref_mut())?;
        self.manifest = next_manifest;
        self.closed_wal_bytes = self
            .closed_wal_bytes
            .checked_sub(reclaimed_bytes)
            .ok_or_else(|| Error::invariant("retained WAL byte count underflow"))?;
        hit(&mut injector, "after_checkpoint_manifest_publish")?;
        for number in obsolete_numbers {
            self.unlink_obsolete_segment(number, &mut injector)?;
        }
        self.sync_reclamation_directory(&mut injector)?;
        Ok(reclaimed_bytes)
    }

    pub(crate) fn cleanup_obsolete_segments(
        &self,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        let first_required = self.manifest.segments[0].number;
        if first_required == 1 {
            return Ok(());
        }
        for entry in fs::read_dir(&self.segment_directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(number) = name
                .strip_prefix("segment-")
                .and_then(|name| name.strip_suffix(".wal"))
                .and_then(|number| number.parse::<u64>().ok())
            else {
                continue;
            };
            if number == 0
                || number >= first_required
                || entry.path() != segment_path(&self.segment_directory, number)
            {
                continue;
            }
            if !entry.file_type()?.is_file() || entry.metadata()?.len() == 0 {
                continue;
            }
            let Ok(obsolete) = WalLog::open_with_segment_boundary(
                ProductionFile::open(entry.path())?,
                self.manifest.identity.clone(),
                WalPageImageFormat::ExperimentalBlink,
                Lsn::ZERO,
                None,
                false,
                false,
                None,
            ) else {
                continue;
            };
            if obsolete.history_start_lsn() >= self.manifest.checkpoint_lsn
                || obsolete
                    .last_commit_lsn()
                    .is_none_or(|end| end > self.manifest.checkpoint_lsn)
                || obsolete.next_lsn().get() > self.manifest.checkpoint_lsn.get().saturating_add(1)
            {
                continue;
            }
            self.unlink_obsolete_segment(number, &mut injector)?;
        }
        self.sync_reclamation_directory(&mut injector)
    }

    fn unlink_obsolete_segment(
        &self,
        number: u64,
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        hit(injector, "before_checkpoint_wal_unlink")?;
        fs::remove_file(segment_path(&self.segment_directory, number))?;
        hit(injector, "after_checkpoint_wal_unlink")
    }

    fn sync_reclamation_directory(
        &self,
        injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        hit(injector, "before_checkpoint_wal_directory_sync")?;
        sync_directory(&self.segment_directory)?;
        hit(injector, "after_checkpoint_wal_directory_sync")?;
        Ok(())
    }
}

fn sidecar_path(database_path: &Path, suffix: &str) -> PathBuf {
    let mut name = database_path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(suffix);
    database_path.with_file_name(name)
}

fn segment_path(directory: &Path, number: u64) -> PathBuf {
    directory.join(format!("segment-{number:020}.wal"))
}

fn parent_directory(path: &Path) -> Result<&Path> {
    path.parent()
        .ok_or_else(|| Error::invalid_input("WAL segment directory has no parent"))
}

fn create_new_file(path: &Path) -> Result<ProductionFile> {
    let file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(path)?;
    Ok(ProductionFile::from_file(file))
}

fn hit(injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>, point: &str) -> Result<()> {
    if let Some(injector) = injector.as_deref_mut() {
        injector.hit(point)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalSegmentEntry {
    pub number: u64,
    pub start_lsn: Lsn,
    pub end_lsn: Option<Lsn>,
    pub start_batch_id: u64,
    pub end_batch_id: Option<u64>,
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
            bytes.extend_from_slice(&segment.end_lsn.map_or(u64::MAX, Lsn::get).to_le_bytes());
            bytes.push(u8::from(segment.closed));
            bytes.extend_from_slice(&[0; 7]);
            bytes.extend_from_slice(&segment.start_batch_id.to_le_bytes());
            bytes.extend_from_slice(&segment.end_batch_id.unwrap_or(u64::MAX).to_le_bytes());
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
            if bytes[offset + 25..offset + 32]
                .iter()
                .any(|value| *value != 0)
            {
                return Err(Error::corruption("WAL segment reserved bytes are nonzero"));
            }
            let end_lsn = read_u64(bytes, offset + 16)?;
            let end_batch_id = read_u64(bytes, offset + 40)?;
            segments.push(WalSegmentEntry {
                number: read_u64(bytes, offset)?,
                start_lsn: Lsn::new(read_u64(bytes, offset + 8)?),
                end_lsn: (end_lsn != u64::MAX).then(|| Lsn::new(end_lsn)),
                start_batch_id: read_u64(bytes, offset + 32)?,
                end_batch_id: (end_batch_id != u64::MAX).then_some(end_batch_id),
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
        let mut previous_batch_id: Option<u64> = None;
        let mut active_count = 0usize;
        let first_segment = self.segments.first().unwrap();
        if first_segment.number == 0
            || self.checkpoint_lsn.get().checked_add(1) != Some(first_segment.start_lsn.get())
            || self.checkpoint_lsn == Lsn::ZERO
                && (first_segment.number != 1 || first_segment.start_batch_id != 1)
        {
            return Err(Error::corruption(
                "first WAL segment does not begin at the initial boundary",
            ));
        }
        for (segment_index, segment) in self.segments.iter().enumerate() {
            if segment.start_batch_id == 0
                || segment
                    .end_batch_id
                    .is_some_and(|end| end < segment.start_batch_id)
            {
                return Err(Error::corruption("WAL segment batch range is invalid"));
            }
            if segment.closed && segment.end_lsn.is_none() {
                return Err(Error::corruption("closed WAL segment has no end LSN"));
            }
            if segment.closed != segment.end_lsn.is_some() {
                return Err(Error::corruption("WAL segment LSN boundary is incomplete"));
            }
            if segment.closed != segment.end_batch_id.is_some() {
                return Err(Error::corruption(
                    "WAL segment batch boundary is incomplete",
                ));
            }
            if segment.end_lsn.is_some_and(|end| segment.start_lsn > end) {
                return Err(Error::corruption("WAL segment LSN range is reversed"));
            }
            if segment.start_lsn <= self.checkpoint_lsn
                && segment.end_lsn.is_some_and(|end| end > self.checkpoint_lsn)
            {
                return Err(Error::corruption(
                    "WAL segment range straddles the durable checkpoint boundary",
                ));
            }
            if let Some(number) = previous_number
                && segment.number != number.saturating_add(1)
            {
                return Err(Error::corruption("WAL segment numbers are not contiguous"));
            }
            if let Some(end) = previous_end
                && segment.start_lsn.get() != end.get().saturating_add(1)
            {
                return Err(Error::corruption(
                    "WAL segment LSN ranges are not contiguous",
                ));
            }
            if let Some(end_batch_id) = previous_batch_id
                && segment.start_batch_id != end_batch_id.saturating_add(1)
            {
                return Err(Error::corruption(
                    "WAL segment batch ranges are not contiguous",
                ));
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
            previous_end = segment.end_lsn;
            previous_batch_id = segment.end_batch_id;
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
        self.write_atomic_with_fault_injector(path, None)
    }

    fn write_atomic_with_fault_injector(
        &self,
        path: &Path,
        mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
    ) -> Result<()> {
        let bytes = self.encode()?;
        let parent = path
            .parent()
            .ok_or_else(|| Error::invalid_input("manifest path has no parent directory"))?;
        fs::create_dir_all(parent)?;
        let temporary_path = temporary_path(path, self.generation);
        let write_result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary_path)?;
            file.write_all(&bytes)?;
            hit(&mut injector, "before_wal_manifest_file_sync")?;
            file.sync_all()?;
            hit(&mut injector, "after_wal_manifest_file_sync")?;
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }
        fs::rename(&temporary_path, path)?;
        hit(&mut injector, "before_wal_manifest_directory_sync")?;
        sync_directory(parent)?;
        hit(&mut injector, "after_wal_manifest_directory_sync")?;
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
            checkpoint_lsn: Lsn::ZERO,
            root_page_id: PageId::new(2),
            free_list_head: Some(PageId::new(5)),
            high_water_page_id: PageId::new(9),
            format_version: 1,
            segments: vec![
                WalSegmentEntry {
                    number: 1,
                    start_lsn: Lsn::new(1),
                    end_lsn: Some(Lsn::new(10)),
                    start_batch_id: 1,
                    end_batch_id: Some(2),
                    closed: true,
                },
                WalSegmentEntry {
                    number: 2,
                    start_lsn: Lsn::new(11),
                    end_lsn: None,
                    start_batch_id: 3,
                    end_batch_id: None,
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

        let mut legacy_version = manifest().encode().unwrap();
        legacy_version[8..12].copy_from_slice(&1u32.to_le_bytes());
        let checksum_offset = legacy_version.len() - CHECKSUM_SIZE;
        let checksum = crc32c::crc32c(&legacy_version[..checksum_offset]);
        legacy_version[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
        assert!(SegmentedWalManifest::decode(&legacy_version).is_err());

        let mut invalid = manifest();
        invalid.segments[1].start_lsn = Lsn::new(10);
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

#[cfg(test)]
mod storage_tests {
    use super::*;
    use crate::blink::BlinkStore;
    use crate::btree::DatabaseConfig;
    use crate::durable_file::ProductionFile;
    use crate::fault::FaultInjector;
    use dodb_core::{
        DocumentKey, Error, Revision, RevisionState, TransactionCondition, TransactionMutation,
        TransactionRequest,
    };

    struct FailAt(&'static str);

    impl FaultInjector for FailAt {
        fn hit(&mut self, point: &str) -> Result<()> {
            if point == self.0 {
                return Err(Error::recovery(format!("injected failure at {point}")));
            }
            Ok(())
        }
    }

    fn database_path(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let directory = std::env::temp_dir().join(format!(
            "dodb-segmented-{test_name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        directory.join("tenant.db")
    }

    fn key(name: &[u8]) -> DocumentKey {
        DocumentKey::new(b"segmented".to_vec(), name.to_vec())
    }

    fn open(path: &Path, max_segment_bytes: u64) -> BlinkStore<ProductionFile, ProductionFile> {
        BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
            path,
            DatabaseConfig::default(),
            max_segment_bytes,
        )
        .unwrap()
    }

    fn assert_present(
        store: &mut BlinkStore<ProductionFile, ProductionFile>,
        document_key: &DocumentKey,
        value: &[u8],
        revision: Revision,
    ) {
        assert_eq!(
            store.get(document_key).unwrap(),
            RevisionState::Present {
                value: value.to_vec(),
                revision,
            }
        );
    }

    #[test]
    fn segmented_store_rotates_and_reopens_transaction_state() {
        let path = database_path("transactions");
        let mut store = open(&path, 1);
        assert!(matches!(
            store.get(&key(b"missing")).unwrap(),
            RevisionState::Missing { .. }
        ));

        let first = store.put(key(b"primary"), b"one".to_vec()).unwrap();
        let second = store
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: key(b"primary"),
                        value: b"two".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: key(b"secondary"),
                        value: b"atomic".to_vec(),
                    },
                ],
            ))
            .unwrap()
            .revision
            .unwrap();
        let conditional_revision = store
            .transact(TransactionRequest::new(
                vec![
                    TransactionCondition::RevisionEquals {
                        key: key(b"primary"),
                        expected_revision: second,
                    },
                    TransactionCondition::Exists {
                        key: key(b"secondary"),
                    },
                ],
                vec![TransactionMutation::Put {
                    key: key(b"secondary"),
                    value: b"conditional".to_vec(),
                }],
            ))
            .unwrap()
            .revision
            .unwrap();
        let failed_condition = store.transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: key(b"primary"),
                expected_revision: first,
            }],
            vec![TransactionMutation::Put {
                key: key(b"conditional"),
                value: b"must-not-appear".to_vec(),
            }],
        ));
        assert!(failed_condition.is_err());
        let deleted = store.delete(key(b"primary")).unwrap();
        let recreated = store.put(key(b"primary"), b"recreated".to_vec()).unwrap();
        assert!(first < second);
        assert!(second < conditional_revision);
        assert!(second < deleted);
        assert!(deleted < recreated);
        drop(store);

        let mut reopened = open(&path, 1);
        assert_present(&mut reopened, &key(b"primary"), b"recreated", recreated);
        assert_present(
            &mut reopened,
            &key(b"secondary"),
            b"conditional",
            conditional_revision,
        );
        assert!(matches!(
            reopened.get(&key(b"conditional")).unwrap(),
            RevisionState::Missing { .. }
        ));
        let manifest =
            SegmentedWalManifest::read(&sidecar_path(&path, SEGMENT_MANIFEST_SUFFIX)).unwrap();
        assert!(manifest.segments.len() >= 4);
        for pair in manifest.segments.windows(2) {
            assert_eq!(pair[1].start_lsn.get(), pair[0].end_lsn.unwrap().get() + 1);
            assert_eq!(pair[1].start_batch_id, pair[0].end_batch_id.unwrap() + 1);
        }
        let revision_after_reopen = reopened
            .put(key(b"after-reopen"), b"value".to_vec())
            .unwrap();
        assert!(revision_after_reopen > recreated);
        drop(reopened);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_wal_first_page_image_and_later_delta_recover_across_rotation() {
        let path = database_path("page-chain");
        let mut store = open(&path, 1);
        let first = store.put(key(b"chain"), vec![3; 32]).unwrap();
        let second = store.put(key(b"chain"), vec![4; 32]).unwrap();
        store.segmented_wal.as_mut().unwrap().max_segment_bytes = u64::MAX;
        let third = store.put(key(b"chain"), vec![5; 32]).unwrap();
        assert!(first < second && second < third);
        drop(store);

        let manifest =
            SegmentedWalManifest::read(&sidecar_path(&path, SEGMENT_MANIFEST_SUFFIX)).unwrap();
        assert_eq!(manifest.segments.len(), 2);
        let rotated_segment = &manifest.segments[1];
        let rotated_wal = WalLog::open_with_segment_boundary(
            ProductionFile::open(segment_path(
                &sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX),
                rotated_segment.number,
            ))
            .unwrap(),
            manifest.identity.clone(),
            WalPageImageFormat::ExperimentalBlink,
            Lsn::new(rotated_segment.start_lsn.get() - 1),
            Some(rotated_segment.start_batch_id),
            true,
            false,
            None,
        )
        .unwrap();
        let redo = rotated_wal.metrics().unwrap().redo;
        assert!(redo.page_image_records > 0);
        assert!(redo.page_delta_records > 0);

        let mut reopened = open(&path, u64::MAX);
        assert_present(&mut reopened, &key(b"chain"), &vec![5; 32], third);
        drop(reopened);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_active_torn_tail_is_discarded_without_losing_acknowledged_write() {
        let path = database_path("torn-tail");
        let mut store = open(&path, u64::MAX);
        let revision = store.put(key(b"durable"), b"value".to_vec()).unwrap();
        drop(store);
        let manifest =
            SegmentedWalManifest::read(&sidecar_path(&path, SEGMENT_MANIFEST_SUFFIX)).unwrap();
        let active_file = segment_path(
            &sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX),
            manifest.active_segment,
        );
        assert_eq!(manifest.generation, 1);
        assert_eq!(manifest.segments.last().unwrap().end_lsn, None);
        let active_length_before_tail = fs::metadata(&active_file).unwrap().len();
        OpenOptions::new()
            .append(true)
            .open(&active_file)
            .unwrap()
            .write_all(&[1, 2, 3, 4, 5])
            .unwrap();
        let mut reopened = open(&path, u64::MAX);
        assert_present(&mut reopened, &key(b"durable"), b"value", revision);
        assert_eq!(
            fs::metadata(&active_file).unwrap().len(),
            active_length_before_tail
        );
        drop(reopened);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_incomplete_transaction_recovery_accepts_writes_and_recovers_again() {
        for tail_case in 0..8 {
            let path = database_path(&format!("incomplete-transaction-{tail_case}"));
            let mut store = open(&path, u64::MAX);
            let acknowledged = store.put(key(b"acknowledged"), b"keep".to_vec()).unwrap();
            if tail_case >= 4 {
                store.checkpoint().unwrap();
            }
            let wal_path = segment_path(
                &sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX),
                store
                    .segmented_wal
                    .as_ref()
                    .unwrap()
                    .manifest
                    .active_segment,
            );
            let committed_length = fs::metadata(&wal_path).unwrap().len();
            store.put(key(b"incomplete"), b"discard".to_vec()).unwrap();
            drop(store);
            let bytes = fs::read(&wal_path).unwrap();
            let mut offset = committed_length as usize;
            let mut commit_offset = offset;
            while offset < bytes.len() {
                commit_offset = offset;
                offset += read_u32(&bytes, offset + 8).unwrap() as usize;
            }
            let cut = match tail_case % 4 {
                0 => committed_length as usize + 1,
                1 => commit_offset,
                2 => commit_offset + crate::wal::WAL_HEADER_SIZE,
                _ => bytes.len() - 1,
            };
            OpenOptions::new()
                .write(true)
                .open(&wal_path)
                .unwrap()
                .set_len(cut as u64)
                .unwrap();
            let mut recovered = open(&path, u64::MAX);
            assert_present(&mut recovered, &key(b"acknowledged"), b"keep", acknowledged);
            assert!(matches!(
                recovered.get(&key(b"incomplete")).unwrap(),
                RevisionState::Missing { .. }
            ));
            assert_eq!(fs::metadata(&wal_path).unwrap().len(), committed_length);
            let next_revision = recovered
                .put(key(b"after-recovery"), b"durable".to_vec())
                .unwrap();
            drop(recovered);
            let mut verified = open(&path, 1);
            assert_present(&mut verified, &key(b"acknowledged"), b"keep", acknowledged);
            assert_present(
                &mut verified,
                &key(b"after-recovery"),
                b"durable",
                next_revision,
            );
            let rotated_revision = verified
                .put(key(b"after-rotation"), b"rotated".to_vec())
                .unwrap();
            drop(verified);
            let mut rotated = open(&path, 1);
            assert_present(
                &mut rotated,
                &key(b"after-recovery"),
                b"durable",
                next_revision,
            );
            assert_present(
                &mut rotated,
                &key(b"after-rotation"),
                b"rotated",
                rotated_revision,
            );
            drop(rotated);
            fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }
    }

    #[test]
    fn segmented_metrics_include_closed_files_and_counters_across_rotations() {
        let path = database_path("metrics");
        let mut store = open(&path, 1);
        let mut previous = WalMetrics::default();
        for sequence in 1..=4 {
            store.put(key(&[sequence]), vec![sequence; 128]).unwrap();
            let metrics = store.wal_metrics().unwrap().unwrap();
            let actual_bytes: u64 = fs::read_dir(sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX))
                .unwrap()
                .map(|entry| entry.unwrap().metadata().unwrap().len())
                .sum();
            assert_eq!(metrics.wal_bytes, actual_bytes);
            assert_eq!(metrics.committed_batches, usize::from(sequence));
            assert_eq!(metrics.wal_syncs, u64::from(sequence) * 2);
            assert!(metrics.page_images > previous.page_images);
            assert!(metrics.physical_write_calls > previous.physical_write_calls);
            assert!(metrics.redo.page_image_records > previous.redo.page_image_records);
            previous = metrics;
        }
        drop(store);
        let reopened = open(&path, 1);
        let metrics = reopened.wal_metrics().unwrap().unwrap();
        assert_eq!(metrics.wal_bytes, previous.wal_bytes);
        assert_eq!(metrics.committed_batches, 4);
        assert_eq!(metrics.wal_syncs, 0);
        drop(reopened);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_checkpoint_reclaims_closed_files_and_repeats_after_reopen() {
        let path = database_path("checkpoint-reclamation");
        let mut store = open(&path, 1);
        let mut expected = Vec::new();
        for sequence in 0..192u16 {
            let document_key = key(&sequence.to_be_bytes());
            let value = vec![sequence as u8; 1024];
            let revision = store.put(document_key.clone(), value.clone()).unwrap();
            expected.push((document_key, value, revision));
        }
        assert!(store.split_metrics().leaf_splits > 0);
        let pinned = store.publisher.pin();
        let mut corrections = 0;
        let pinned_documents =
            super::super::scan_state(&pinned, None, usize::MAX, &mut corrections).unwrap();
        let metrics_before = store.wal_metrics().unwrap().unwrap();
        let next_lsn = store.next_lsn;
        let next_batch_id = store.next_batch_id;
        let report = store.checkpoint().unwrap();
        assert_eq!(report.checkpoint_lsn.get() + 1, next_lsn.get());
        assert!(report.pages_flushed > 1);
        assert!(report.bytes_written >= report.pages_flushed as u64 * PAGE_SIZE as u64);
        assert_eq!(report.wal_bytes_reclaimed, metrics_before.wal_bytes);
        let metrics_after = store.wal_metrics().unwrap().unwrap();
        assert_eq!(
            metrics_after.committed_batches,
            metrics_before.committed_batches
        );
        assert_eq!(metrics_after.wal_syncs, metrics_before.wal_syncs + 1);
        assert_eq!(store.next_lsn, next_lsn);
        assert_eq!(store.next_batch_id, next_batch_id);
        let manifest_path = sidecar_path(&path, SEGMENT_MANIFEST_SUFFIX);
        let manifest = SegmentedWalManifest::read(&manifest_path).unwrap();
        assert_eq!(manifest.checkpoint_lsn, report.checkpoint_lsn);
        assert_eq!(manifest.segments.len(), 1);
        assert!(!manifest.segments[0].closed);
        assert_eq!(manifest.segments[0].start_lsn, next_lsn);
        assert_eq!(manifest.segments[0].start_batch_id, next_batch_id);
        assert_eq!(
            fs::read_dir(sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX))
                .unwrap()
                .count(),
            1
        );
        let idle_report = store.checkpoint().unwrap();
        assert_eq!(idle_report.wal_bytes_reclaimed, 0);
        assert_eq!(idle_report.bytes_written, 0);
        assert_eq!(
            SegmentedWalManifest::read(&manifest_path).unwrap(),
            manifest
        );
        let stale_revision = expected[0].2;
        let deleted = store.delete(expected[0].0.clone()).unwrap();
        let recreated = store.put(expected[0].0.clone(), vec![99; 1024]).unwrap();
        assert!(deleted > stale_revision && recreated > deleted);
        assert!(
            store
                .transact(TransactionRequest::new(
                    vec![TransactionCondition::RevisionEquals {
                        key: expected[0].0.clone(),
                        expected_revision: stale_revision
                    }],
                    vec![TransactionMutation::Put {
                        key: expected[0].0.clone(),
                        value: b"rejected".to_vec()
                    }],
                ))
                .is_err()
        );
        expected[0] = (expected[0].0.clone(), vec![99; 1024], recreated);
        let mut corrections = 0;
        assert_eq!(
            super::super::scan_state(&pinned, None, usize::MAX, &mut corrections).unwrap(),
            pinned_documents
        );
        drop(pinned);
        drop(store);
        let mut reopened = open(&path, u64::MAX);
        for (document_key, value, revision) in &expected {
            assert_present(&mut reopened, document_key, value, *revision);
        }
        assert_eq!(
            reopened.current_superblock.checkpoint_lsn,
            manifest.checkpoint_lsn
        );
        let second_report = reopened.checkpoint().unwrap();
        assert!(second_report.checkpoint_lsn > manifest.checkpoint_lsn);
        assert!(second_report.wal_bytes_reclaimed > 0);
        let persisted_boundary = second_report.checkpoint_lsn;
        drop(reopened);
        let mut verified = open(&path, u64::MAX);
        assert_eq!(
            verified.current_superblock.checkpoint_lsn,
            persisted_boundary
        );
        for (document_key, value, revision) in &expected {
            assert_present(&mut verified, document_key, value, *revision);
        }
        verified.check_invariants().unwrap();
        drop(verified);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_obsolete_cleanup_leaves_unrecognized_files_unchanged() {
        let path = database_path("checkpoint-obsolete-unknown");
        let mut store = open(&path, 1);
        store.put(key(b"durable"), b"value".to_vec()).unwrap();
        let directory = sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX);
        let obsolete_path = segment_path(&directory, 1);
        let bytes = fs::read(&obsolete_path).unwrap();
        store.checkpoint().unwrap();
        drop(store);
        let incomplete_init = &bytes[..8];
        fs::write(&obsolete_path, incomplete_init).unwrap();
        let mut reopened = open(&path, 1);
        assert_eq!(fs::read(&obsolete_path).unwrap(), incomplete_init);
        assert_eq!(
            reopened.get(&key(b"durable")).unwrap().value(),
            Some(b"value".as_slice())
        );
        reopened.put(key(b"later"), b"next".to_vec()).unwrap();
        reopened.checkpoint().unwrap();
        assert_eq!(fs::read(&obsolete_path).unwrap(), incomplete_init);
        drop(reopened);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_checkpoint_recovery_rejects_missing_required_file_with_obsolete_files_present() {
        let path = database_path("checkpoint-required-file");
        let mut store = open(&path, 1);
        store.put(key(b"before"), b"checkpointed".to_vec()).unwrap();
        let directory = sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX);
        let obsolete_path = segment_path(&directory, 1);
        let obsolete_bytes = fs::read(&obsolete_path).unwrap();
        store.checkpoint().unwrap();
        store.put(key(b"after"), b"required".to_vec()).unwrap();
        drop(store);
        let manifest =
            SegmentedWalManifest::read(&sidecar_path(&path, SEGMENT_MANIFEST_SUFFIX)).unwrap();
        assert!(manifest.checkpoint_lsn > Lsn::ZERO);
        fs::write(&obsolete_path, obsolete_bytes).unwrap();
        let required_path = segment_path(&directory, manifest.active_segment);
        fs::remove_file(&required_path).unwrap();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &path,
                DatabaseConfig::default(),
                1
            )
            .is_err()
        );
        assert!(!required_path.exists());
        assert!(obsolete_path.exists());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_checkpoint_preserves_multikey_conditions_and_tombstone_revisions() {
        let path = database_path("checkpoint-transaction-contract");
        let mut store = open(&path, u64::MAX);
        let initial = store.put(key(b"primary"), b"initial".to_vec()).unwrap();
        store.checkpoint().unwrap();
        let committed = store
            .transact(TransactionRequest::new(
                vec![TransactionCondition::RevisionEquals {
                    key: key(b"primary"),
                    expected_revision: initial,
                }],
                vec![
                    TransactionMutation::Put {
                        key: key(b"primary"),
                        value: b"updated".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: key(b"secondary"),
                        value: b"atomic".to_vec(),
                    },
                ],
            ))
            .unwrap()
            .revision
            .unwrap();
        let deleted = store.delete(key(b"primary")).unwrap();
        store.checkpoint().unwrap();
        drop(store);
        let mut reopened = open(&path, u64::MAX);
        assert_eq!(
            reopened.get(&key(b"primary")).unwrap(),
            RevisionState::Missing { revision: deleted }
        );
        assert_present(&mut reopened, &key(b"secondary"), b"atomic", committed);
        assert!(
            reopened
                .transact(TransactionRequest::new(
                    vec![TransactionCondition::RevisionEquals {
                        key: key(b"primary"),
                        expected_revision: initial
                    }],
                    vec![TransactionMutation::Delete {
                        key: key(b"secondary")
                    }],
                ))
                .is_err()
        );
        let recreated = reopened
            .transact(TransactionRequest::new(
                vec![TransactionCondition::RevisionEquals {
                    key: key(b"primary"),
                    expected_revision: deleted,
                }],
                vec![
                    TransactionMutation::Put {
                        key: key(b"primary"),
                        value: b"recreated".to_vec(),
                    },
                    TransactionMutation::Delete {
                        key: key(b"secondary"),
                    },
                ],
            ))
            .unwrap()
            .revision
            .unwrap();
        assert!(recreated > deleted);
        reopened.checkpoint().unwrap();
        drop(reopened);
        let mut verified = open(&path, u64::MAX);
        assert_present(&mut verified, &key(b"primary"), b"recreated", recreated);
        assert_eq!(
            verified.get(&key(b"secondary")).unwrap(),
            RevisionState::Missing {
                revision: recreated
            }
        );
        drop(verified);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_recovery_uses_manifest_boundary_over_stale_wal_superblocks() {
        let path = database_path("checkpoint-authority");
        let mut store = open(&path, u64::MAX);
        let first = store.put(key(b"first"), b"before".to_vec()).unwrap();
        let checkpoint = store.checkpoint().unwrap();
        let mut stale = store.current_superblock.clone();
        stale.checkpoint_lsn = Lsn::ZERO;
        stale.checkpoint_sequence = 0;
        store.current_superblock = stale;
        let second = store.put(key(b"overflow"), vec![42; 16_384]).unwrap();
        drop(store);
        let mut reopened = open(&path, u64::MAX);
        assert_eq!(
            reopened.current_superblock.checkpoint_lsn,
            checkpoint.checkpoint_lsn
        );
        assert_present(&mut reopened, &key(b"first"), b"before", first);
        assert_present(&mut reopened, &key(b"overflow"), &vec![42; 16_384], second);
        let next = reopened.put(key(b"next"), b"after".to_vec()).unwrap();
        let next_checkpoint = reopened.checkpoint().unwrap();
        assert!(next_checkpoint.checkpoint_lsn > checkpoint.checkpoint_lsn);
        drop(reopened);
        let mut verified = open(&path, u64::MAX);
        assert_present(&mut verified, &key(b"next"), b"after", next);
        drop(verified);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    struct FailCheckpointAt {
        point: &'static str,
        metadata_ready: bool,
    }

    impl FaultInjector for FailCheckpointAt {
        fn hit(&mut self, point: &str) -> Result<()> {
            if point == "after_checkpoint_metadata_sync" {
                self.metadata_ready = true;
            }
            if point == self.point
                && (!point.starts_with("before_wal_manifest_")
                    && !point.starts_with("after_wal_manifest_")
                    || self.metadata_ready)
            {
                return Err(Error::durability(format!(
                    "injected checkpoint failure at {point}"
                )));
            }
            Ok(())
        }
    }

    #[test]
    fn segmented_checkpoint_faults_recover_and_allow_later_checkpoints() {
        let fault_points = [
            "before_checkpoint_data_flush",
            "during_checkpoint_page_write",
            "after_checkpoint_page_write",
            "before_checkpoint_data_sync",
            "after_checkpoint_data_sync",
            "before_checkpoint_superblock_write",
            "after_checkpoint_superblock_write",
            "before_checkpoint_metadata_sync",
            "after_checkpoint_metadata_sync",
            "before_checkpoint_manifest_publish",
            "before_wal_manifest_file_sync",
            "after_wal_manifest_file_sync",
            "before_wal_manifest_directory_sync",
            "after_wal_manifest_directory_sync",
            "after_checkpoint_manifest_publish",
            "before_checkpoint_wal_unlink",
            "after_checkpoint_wal_unlink",
            "before_checkpoint_wal_directory_sync",
            "after_checkpoint_wal_directory_sync",
        ];
        for (fault_index, fault_point) in fault_points.iter().enumerate() {
            let path = database_path(&format!("checkpoint-fault-{fault_index}"));
            let mut store = open(&path, 1);
            let first = store.put(key(b"first"), vec![1; 8192]).unwrap();
            let baseline = store.checkpoint().unwrap();
            let second = store.put(key(b"second"), vec![2; 8192]).unwrap();
            store.set_boxed_fault_injector(Box::new(FailCheckpointAt {
                point: fault_point,
                metadata_ready: false,
            }));
            assert!(store.checkpoint().is_err(), "{fault_point}");
            assert!(
                store.put(key(b"rejected"), b"no".to_vec()).is_err(),
                "{fault_point}"
            );
            drop(store);
            let mut reopened = open(&path, 1);
            assert!(
                reopened.current_superblock.checkpoint_lsn >= baseline.checkpoint_lsn,
                "{fault_point}"
            );
            assert_present(&mut reopened, &key(b"first"), &vec![1; 8192], first);
            assert_present(&mut reopened, &key(b"second"), &vec![2; 8192], second);
            let next = reopened.put(key(b"next"), b"recovered".to_vec()).unwrap();
            reopened.checkpoint().unwrap();
            drop(reopened);
            let mut verified = open(&path, 1);
            assert_present(&mut verified, &key(b"next"), b"recovered", next);
            assert_present(&mut verified, &key(b"second"), &vec![2; 8192], second);
            assert_eq!(
                fs::read_dir(sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX))
                    .unwrap()
                    .count(),
                1
            );
            drop(verified);
            fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }
    }

    #[test]
    fn segmented_rotation_faults_preserve_only_acknowledged_transactions() {
        let fault_points = [
            "before_wal_segment_close",
            "before_wal_segment_create",
            "before_wal_segment_file_sync",
            "after_wal_segment_file_sync",
            "before_wal_segment_directory_sync",
            "after_wal_segment_directory_sync",
            "before_wal_manifest_publish",
            "before_wal_manifest_file_sync",
            "after_wal_manifest_file_sync",
            "before_wal_manifest_directory_sync",
            "after_wal_manifest_directory_sync",
            "after_wal_manifest_publish",
        ];
        for (fault_index, fault_point) in fault_points.iter().enumerate() {
            let path = database_path(&format!("rotation-fault-{fault_index}"));
            let mut store = open(&path, 1);
            let committed_revision = store.put(key(b"acknowledged"), b"yes".to_vec()).unwrap();
            store.set_boxed_fault_injector(Box::new(FailAt(fault_point)));
            assert!(store.put(key(b"rejected"), b"no".to_vec()).is_err());
            drop(store);

            let mut reopened = open(&path, 1);
            assert_present(
                &mut reopened,
                &key(b"acknowledged"),
                b"yes",
                committed_revision,
            );
            assert!(matches!(
                reopened.get(&key(b"rejected")).unwrap(),
                RevisionState::Missing { .. }
            ));
            let retry_revision = reopened.put(key(b"retry"), b"recovered".to_vec()).unwrap();
            drop(reopened);
            let mut verified = open(&path, 1);
            assert_present(
                &mut verified,
                &key(b"acknowledged"),
                b"yes",
                committed_revision,
            );
            assert_present(&mut verified, &key(b"retry"), b"recovered", retry_revision);
            assert!(matches!(
                verified.get(&key(b"rejected")).unwrap(),
                RevisionState::Missing { .. }
            ));
            drop(verified);
            fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }
    }

    #[test]
    fn segmented_closed_damage_and_missing_files_fail_recovery() {
        let path = database_path("closed-damage");
        let mut store = open(&path, 1);
        store.put(key(b"first"), b"one".to_vec()).unwrap();
        store.put(key(b"second"), b"two".to_vec()).unwrap();
        drop(store);
        let manifest_path = sidecar_path(&path, SEGMENT_MANIFEST_SUFFIX);
        let manifest = SegmentedWalManifest::read(&manifest_path).unwrap();
        let closed_path = segment_path(
            &sidecar_path(&path, SEGMENT_DIRECTORY_SUFFIX),
            manifest.segments[0].number,
        );
        let mut bytes = fs::read(&closed_path).unwrap();
        bytes[70] ^= 0x80;
        fs::write(&closed_path, bytes).unwrap();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &path,
                DatabaseConfig::default(),
                1
            )
            .is_err()
        );
        fs::remove_file(closed_path).unwrap();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &path,
                DatabaseConfig::default(),
                1
            )
            .is_err()
        );
        let orphan_path = path.with_file_name("orphan.db");
        fs::create_dir(sidecar_path(&orphan_path, SEGMENT_DIRECTORY_SUFFIX)).unwrap();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_path(
                &orphan_path,
                DatabaseConfig::default()
            )
            .is_err()
        );
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_manifest_identity_and_active_transaction_boundary_are_verified() {
        let path = database_path("identity-boundary");
        let mut store = open(&path, 1);
        store.put(key(b"key"), b"value".to_vec()).unwrap();
        drop(store);
        let manifest_path = sidecar_path(&path, SEGMENT_MANIFEST_SUFFIX);
        let mut manifest = SegmentedWalManifest::read(&manifest_path).unwrap();
        manifest.identity.database_uuid[0] = 1;
        manifest.write_atomic(&manifest_path).unwrap();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &path,
                DatabaseConfig::default(),
                1
            )
            .is_err()
        );

        manifest.identity.database_uuid[0] = 0;
        let mut bytes = manifest.encode().unwrap();
        let start_batch_offset = FIXED_SIZE + 32;
        bytes[start_batch_offset..start_batch_offset + 8].copy_from_slice(&9u64.to_le_bytes());
        let checksum_offset = bytes.len() - CHECKSUM_SIZE;
        let checksum = crc32c::crc32c(&bytes[..checksum_offset]);
        bytes[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
        fs::write(&manifest_path, bytes).unwrap();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &path,
                DatabaseConfig::default(),
                1
            )
            .is_err()
        );
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn segmented_empty_active_reopens_and_rejects_legacy_conversion() {
        let path = database_path("empty-active");
        drop(open(&path, u64::MAX));
        let mut reopened = open(&path, u64::MAX);
        assert!(matches!(
            reopened.get(&key(b"empty")).unwrap(),
            RevisionState::Missing { .. }
        ));
        drop(reopened);
        let other_path = path.with_file_name("legacy.db");
        fs::write(&other_path, [1, 2, 3]).unwrap();
        fs::write(other_path.with_extension("wal"), [4, 5, 6]).unwrap();
        assert!(
            BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &other_path,
                DatabaseConfig::default(),
                u64::MAX
            )
            .is_err()
        );
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
