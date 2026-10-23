use super::*;

pub(super) struct PendingCheckpoint {
    token: Arc<()>,
    retained_bytes: u64,
    disk_bytes: u64,
}

pub struct BlinkCheckpointSnapshot {
    token: Arc<()>,
    file: Box<dyn DurableFile + Send>,
    pages: BTreeMap<PageId, Arc<[u8; PAGE_SIZE]>>,
    decoded_pages: BTreeMap<PageId, Arc<BlinkPage>>,
    dirty_superblock: Option<[u8; PAGE_SIZE]>,
    source_slot: SuperblockSlot,
    superblock: BlinkSuperblock,
    started: Instant,
    pub(super) injector: Option<Box<dyn FaultInjector + Send>>,
    base_job: Option<durable_bases::BasePackJob>,
}

pub struct BlinkCheckpointOutput {
    token: Arc<()>,
    superblock: BlinkSuperblock,
    report: BlinkCheckpointReport,
    phase_timings: [u64; 3],
    started: Instant,
    base: Option<durable_bases::BasePackOutput>,
    injector: Option<Box<dyn FaultInjector + Send>>,
}

impl BlinkCheckpointOutput {
    pub fn phase_timings(&self) -> [u64; 3] {
        self.phase_timings
    }
}

impl BlinkCheckpointSnapshot {
    pub fn retained_bytes(&self) -> u64 {
        (self.pages.len() as u64
            + self.decoded_pages.len() as u64
            + u64::from(self.dirty_superblock.is_some())
            + 1)
        .saturating_mul(PAGE_SIZE as u64)
    }

    pub fn checkpoint_lsn(&self) -> Lsn {
        self.superblock.checkpoint_lsn
    }

    pub fn set_boxed_fault_injector(&mut self, injector: Box<dyn FaultInjector + Send>) {
        self.injector = Some(injector);
    }

    pub fn run(mut self) -> Result<BlinkCheckpointOutput> {
        let mut injector = self.injector.take();
        if let Some(job) = self.base_job.take() {
            let encoding_started = Instant::now();
            for (page_id, page) in std::mem::take(&mut self.decoded_pages) {
                self.pages
                    .insert(page_id, Arc::new(encode_blink_page(page_id, &page)?));
            }
            let encoding_nanos = elapsed_nanos(encoding_started);
            let mut base = job.run(&self.pages, &mut injector)?;
            base.phase_timings[0] = base.phase_timings[0].saturating_add(encoding_nanos);
            return Ok(BlinkCheckpointOutput {
                token: self.token,
                superblock: self.superblock.clone(),
                report: BlinkCheckpointReport {
                    checkpoint_lsn: self.superblock.checkpoint_lsn,
                    checkpoint_sequence: self.superblock.checkpoint_sequence,
                    pages_flushed: self.pages.len(),
                    bytes_written: base.entry.bytes,
                    base_bytes_written: base.entry.bytes,
                    full_base_snapshots: u64::from(base.complete),
                    checkpoint_write_requests: base.write_calls,
                    ..BlinkCheckpointReport::default()
                },
                phase_timings: base.phase_timings,
                started: self.started,
                base: Some(base),
                injector,
            });
        }
        hit(&mut injector, "before_checkpoint_data_flush")?;
        let page_started = Instant::now();
        let mut bytes_written = 0u64;
        for (page_id, image) in &self.pages {
            hit(&mut injector, "during_checkpoint_page_write")?;
            write_all_at(
                &mut *self.file,
                page_id.get() * PAGE_SIZE as u64,
                &image[..],
            )?;
            bytes_written = bytes_written.saturating_add(PAGE_SIZE as u64);
            hit(&mut injector, "after_checkpoint_page_write")?;
        }
        if let Some(image) = &self.dirty_superblock {
            let offset = if self.source_slot == SuperblockSlot::A {
                0
            } else {
                PAGE_SIZE as u64
            };
            write_all_at(&mut *self.file, offset, image)?;
            bytes_written = bytes_written.saturating_add(PAGE_SIZE as u64);
        }
        let page_nanos = elapsed_nanos(page_started);
        hit(&mut injector, "before_checkpoint_data_sync")?;
        let data_sync_started = Instant::now();
        self.file.sync_data()?;
        let data_sync_nanos = elapsed_nanos(data_sync_started);
        hit(&mut injector, "after_checkpoint_data_sync")?;
        let offset = if self.source_slot == SuperblockSlot::A {
            PAGE_SIZE as u64
        } else {
            0
        };
        let image = encode_blink_superblock(&self.superblock)?;
        hit(&mut injector, "before_checkpoint_superblock_write")?;
        write_all_at(&mut *self.file, offset, &image)?;
        bytes_written = bytes_written.saturating_add(PAGE_SIZE as u64);
        hit(&mut injector, "after_checkpoint_superblock_write")?;
        hit(&mut injector, "before_checkpoint_metadata_sync")?;
        let metadata_sync_started = Instant::now();
        self.file.sync_data()?;
        let metadata_sync_nanos = elapsed_nanos(metadata_sync_started);
        hit(&mut injector, "after_checkpoint_metadata_sync")?;
        Ok(BlinkCheckpointOutput {
            token: self.token,
            report: BlinkCheckpointReport {
                checkpoint_lsn: self.superblock.checkpoint_lsn,
                checkpoint_sequence: self.superblock.checkpoint_sequence,
                pages_flushed: self.pages.len(),
                bytes_written,
                checkpoint_write_requests: self.pages.len() as u64
                    + u64::from(self.dirty_superblock.is_some())
                    + 1,
                ..BlinkCheckpointReport::default()
            },
            superblock: self.superblock,
            phase_timings: [page_nanos, data_sync_nanos, metadata_sync_nanos],
            started: self.started,
            base: None,
            injector,
        })
    }
}

impl<F: DurableFile, W: DurableFile> BlinkStore<F, W> {
    pub fn set_checkpoint_retention_limit(&mut self, bytes: u64) -> Result<()> {
        if self.segmented_wal.is_none() || bytes < 4 * PAGE_SIZE as u64 {
            return Err(Error::invalid_input(
                "invalid segmented checkpoint retention limit",
            ));
        }
        self.checkpoint_retention_limit = Some(bytes);
        Ok(())
    }

    pub fn checkpoint_retained_image_bytes(&self) -> u64 {
        self.physical_checkpoint
            .as_ref()
            .map_or(0, |pending| pending.retained_bytes)
    }

    pub fn prepare_segmented_checkpoint(&mut self) -> Result<Option<BlinkCheckpointSnapshot>> {
        if let Some(message) = &self.broken {
            return Err(Error::durability(message.clone()));
        }
        if self.physical_checkpoint.is_some() {
            return Err(Error::checkpoint("segmented checkpoint already running"));
        }
        let wal = self
            .segmented_wal
            .as_ref()
            .ok_or_else(|| Error::invalid_input("independent checkpoint requires segmented WAL"))?;
        let target = wal.last_commit_lsn();
        if target == wal.manifest().checkpoint_lsn {
            return Ok(None);
        }
        if let Err(error) = self.check_invariants() {
            self.broken = Some(error.to_string());
            return Err(error);
        }
        let file = self
            .file
            .try_clone_for_background()
            .ok_or_else(|| Error::checkpoint("database file cannot be cloned for checkpoint"))?;
        let superblock = BlinkSuperblock {
            generation: self
                .current_superblock
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::invariant("checkpoint generation exhausted"))?,
            checkpoint_lsn: target,
            checkpoint_sequence: target.get(),
            ..self.current_superblock.clone()
        };
        if let Err(error) = self
            .segmented_wal
            .as_mut()
            .unwrap()
            .rotate(self.fault_injector.as_deref_mut())
        {
            self.broken = Some(error.to_string());
            return Err(error);
        }
        let base_job = self.segmented_wal.as_ref().unwrap().base_job(
            self.dirty_pages.len(),
            self.state.pages.len(),
            self.checkpoint_retention_limit.unwrap_or(512 * 1024 * 1024),
        )?;
        let page_count = if base_job.as_ref().is_some_and(|job| job.complete) {
            self.state.pages.len()
        } else {
            self.dirty_pages.len()
        };
        if base_job.is_some() {
            let limit = self.checkpoint_retention_limit.unwrap_or(512 * 1024 * 1024);
            let wal = self.segmented_wal.as_ref().unwrap();
            if wal
                .base_bytes()
                .saturating_add(wal.metrics()?.wal_bytes)
                .saturating_add(durable_bases::encoded_size(page_count as u64))
                > limit.saturating_add(durable_bases::encoded_size(self.state.pages.len() as u64))
                || (page_count as u64 + 2).saturating_mul(PAGE_SIZE as u64) > limit
            {
                return Err(Error::overloaded(
                    "durable checkpoint base capacity exceeded",
                ));
            }
        }
        let mut decoded_pages = BTreeMap::new();
        if base_job.as_ref().is_some_and(|job| job.complete) {
            for (page_id, page) in &self.state.pages {
                if !self.dirty_pages.contains_key(page_id) {
                    decoded_pages.insert(*page_id, Arc::clone(page));
                }
            }
        }
        let snapshot = BlinkCheckpointSnapshot {
            token: Arc::new(()),
            file,
            pages: std::mem::take(&mut self.dirty_pages),
            decoded_pages,
            dirty_superblock: self.dirty_superblock.take(),
            source_slot: self.active_slot,
            superblock,
            started: Instant::now(),
            injector: None,
            base_job,
        };
        self.physical_checkpoint = Some(PendingCheckpoint {
            token: Arc::clone(&snapshot.token),
            retained_bytes: snapshot.retained_bytes(),
            disk_bytes: if snapshot.base_job.is_some() {
                durable_bases::encoded_size(page_count as u64)
            } else {
                0
            },
        });
        Ok(Some(snapshot))
    }

    pub fn complete_segmented_checkpoint(
        &mut self,
        output: Result<BlinkCheckpointOutput>,
    ) -> Result<BlinkCheckpointReport> {
        let result = self.complete_segmented_checkpoint_inner(output);
        self.physical_checkpoint = None;
        if let Err(error) = &result {
            self.broken = Some(error.to_string());
        }
        result
    }

    fn complete_segmented_checkpoint_inner(
        &mut self,
        output: Result<BlinkCheckpointOutput>,
    ) -> Result<BlinkCheckpointReport> {
        let mut output = output?;
        if self.fault_injector.is_none() {
            self.fault_injector = output.injector.take();
        }
        let pending = self
            .physical_checkpoint
            .as_ref()
            .ok_or_else(|| Error::checkpoint("no segmented checkpoint is running"))?;
        if !Arc::ptr_eq(&pending.token, &output.token) {
            return Err(Error::checkpoint(
                "checkpoint belongs to another task or store",
            ));
        }
        if let Some(message) = &self.broken {
            return Err(Error::durability(message.clone()));
        }
        if output.base.is_some() {
            let wal = self.segmented_wal.as_mut().unwrap();
            let (wal_bytes, base_bytes) = wal.advance_checkpoint_with_base(
                &output.superblock,
                output.base.as_ref(),
                self.fault_injector.as_deref_mut(),
            )?;
            output.report.wal_bytes_reclaimed = wal_bytes;
            output.report.base_bytes_reclaimed = base_bytes;
            output.report.base_bytes_retained = wal.base_bytes();
        } else {
            output.report.wal_bytes_reclaimed = self
                .segmented_wal
                .as_mut()
                .unwrap()
                .advance_checkpoint(&output.superblock, self.fault_injector.as_deref_mut())?;
        }
        self.current_superblock.checkpoint_lsn = output.superblock.checkpoint_lsn;
        self.current_superblock.checkpoint_sequence = output.superblock.checkpoint_sequence;
        self.current_superblock.generation = self
            .current_superblock
            .generation
            .max(output.superblock.generation);
        output.report.duration_nanos = elapsed_nanos(output.started);
        Ok(output.report)
    }

    pub(super) fn check_segmented_group_capacity(&self, commits: &[WalCommit]) -> Result<()> {
        let Some(wal) = &self.segmented_wal else {
            return Ok(());
        };
        let limit = match self.checkpoint_retention_limit {
            Some(limit) => limit,
            None if wal.has_durable_bases() => 512 * 1024 * 1024,
            None => return Ok(()),
        };
        let image_count = commits
            .iter()
            .map(|commit| commit.pages.len() as u64)
            .sum::<u64>();
        let maximum_append = image_count
            .saturating_mul(PAGE_SIZE as u64 + 256)
            .saturating_add(commits.len() as u64 * 256)
            .saturating_add(2 * PAGE_SIZE as u64);
        let maximum_images = (self.dirty_pages.len() as u64 + image_count + 2)
            .saturating_mul(PAGE_SIZE as u64)
            .saturating_add(self.checkpoint_retained_image_bytes());
        let mut recovery_limit = limit;
        let base_reserve = if wal.has_durable_bases() {
            let highest_page = commits
                .iter()
                .flat_map(|commit| &commit.pages)
                .map(|page| page.page_id.get())
                .max()
                .unwrap_or(FIRST_DATA_PAGE)
                .max(self.current_superblock.high_water_page_id.get());
            let live_bytes = durable_bases::encoded_size(highest_page.saturating_sub(1));
            recovery_limit = recovery_limit.saturating_add(live_bytes);
            let pending_bytes = self
                .physical_checkpoint
                .as_ref()
                .map_or(0, |pending| pending.disk_bytes);
            wal.base_bytes()
                .saturating_add(live_bytes.max(pending_bytes))
        } else {
            0
        };
        if wal
            .metrics()?
            .wal_bytes
            .saturating_add(maximum_append)
            .saturating_add(base_reserve)
            > recovery_limit
            || maximum_images > limit
        {
            return Err(Error::overloaded("segmented checkpoint capacity exceeded"));
        }
        Ok(())
    }
}

fn hit(injector: &mut Option<Box<dyn FaultInjector + Send>>, point: &str) -> Result<()> {
    if let Some(injector) = injector.as_deref_mut() {
        injector.hit(point)?;
    }
    Ok(())
}
