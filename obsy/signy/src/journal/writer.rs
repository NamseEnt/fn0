impl Journal {
    pub fn spawn(config: &Config, memtable: Arc<MemTable>) -> Result<Self, IoError> {
        Self::spawn_with_traces(config, memtable, Arc::new(TraceMemTable::new()))
    }

    pub fn spawn_with_traces(
        config: &Config,
        memtable: Arc<MemTable>,
        trace_memtable: Arc<TraceMemTable>,
    ) -> Result<Self, IoError> {
        Self::spawn_with_signals(
            config,
            memtable,
            trace_memtable,
            Arc::new(SeriesMemTable::new()),
        )
    }

    pub fn spawn_with_signals(
        config: &Config,
        memtable: Arc<MemTable>,
        trace_memtable: Arc<TraceMemTable>,
        series_memtable: Arc<SeriesMemTable>,
    ) -> Result<Self, IoError> {
        let marks = Arc::new(CollectMarks::load(&config.data_dir));
        Self::spawn_with_marks(config, memtable, trace_memtable, series_memtable, marks)
    }

    /// The same, for a startup that has already replayed the WAL into the
    /// marks it is handing over. Replay runs before the writer exists, so the
    /// marks it recovered cannot be read out of a journal that is not there
    /// yet.
    pub fn spawn_with_marks(
        config: &Config,
        memtable: Arc<MemTable>,
        trace_memtable: Arc<TraceMemTable>,
        series_memtable: Arc<SeriesMemTable>,
        collect_marks: Arc<CollectMarks>,
    ) -> Result<Self, IoError> {
        let dir = &config.data_dir;
        std::fs::create_dir_all(dir)?;
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);

        // Initialize the WAL synchronously so startup/readiness cannot race a
        // failed open in the background writer. Sync both the empty file and
        // its parent directory: fsyncing a newly-created file alone does not
        // make its directory entry crash-durable on POSIX filesystems.
        let wal_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)?;
        wal_file.sync_all()?;
        std::fs::File::open(dir)?.sync_all()?;
        drop(wal_file);

        let (tx, rx) = mpsc::channel::<JournalCmd>(4096);

        let max_batch_bytes = config.max_batch_bytes;
        let max_batch_ms = config.max_batch_ms;
        let healthy = Arc::new(AtomicBool::new(true));
        let metrics = Arc::new(JournalMetrics::default());
        let backlog = Arc::new(WalBacklog::default());
        backlog.set_wal_bytes(std::fs::metadata(&wal_path)?.len());
        backlog.set_checkpoint_bytes(read_checkpoint(&ckpt_path)?);

        let wal_path_clone = wal_path.clone();
        let ckpt_path_clone = ckpt_path.clone();
        let writer_health = healthy.clone();
        let trace_memtable_clone = trace_memtable.clone();
        let series_memtable_clone = series_memtable.clone();
        let writer_backlog = backlog.clone();
        let writer_memtable = memtable.clone();
        let writer_metrics = metrics.clone();
        let writer_marks = collect_marks.clone();
        let writer_dir = dir.clone();
        let metric_reserved_bytes = Arc::new(AtomicU64::new(0));
        tokio::spawn(async move {
            let result = writer_loop(
                rx,
                &wal_path_clone,
                &ckpt_path_clone,
                &writer_dir,
                writer_memtable,
                trace_memtable_clone,
                series_memtable_clone,
                max_batch_bytes,
                max_batch_ms,
                &writer_backlog,
                &writer_metrics,
                &writer_marks,
            )
            .await;
            writer_health.store(false, Ordering::Release);
            if let Err(e) = result {
                tracing::error!(error = %e, "journal writer terminated");
            }
        });

        Ok(Self {
            tx,
            collect_marks,
            wal_path,
            ckpt_path,
            healthy,
            metrics,
            memtable,
            trace_memtable,
            series_memtable,
            backlog,
            metric_reserved_bytes,
        })
    }

    /// Where the writer task's time went, for `/metrics` and for the soak.
    pub fn metrics(&self) -> &Arc<JournalMetrics> {
        &self.metrics
    }

    pub fn wal_backlog_bytes(&self) -> u64 {
        self.backlog.bytes()
    }

    pub fn wal_bytes(&self) -> u64 {
        self.backlog.wal_bytes()
    }

    /// The memtable this journal feeds. Held so the ingest gate can size both
    /// buffers from the journal alone, rather than every caller having to
    /// thread the log memtable in beside it.
    pub fn log_memtable(&self) -> &Arc<MemTable> {
        &self.memtable
    }

    /// An OTLP log export: `data` is the encoded `ExportLogsServiceRequest`
    /// itself — for the HTTP protobuf transport the received body verbatim —
    /// and `entries` is what `otlp_log::normalize_request` already produced
    /// for the memtable, so nothing is normalized twice and nothing is
    /// re-encoded into a second message for the WAL's sake.
    pub async fn append_otlp_logs(
        &self,
        tenant: TenantId,
        data: Vec<u8>,
        entries: Vec<LogEntry>,
    ) -> Result<(), IoError> {
        self.enqueue_otlp_logs(tenant, data, entries, None)
            .await?
            .settle()
            .await
    }

    pub async fn enqueue_otlp_logs(
        &self,
        tenant: TenantId,
        data: Vec<u8>,
        entries: Vec<LogEntry>,
        mark: Option<CollectMark>,
    ) -> Result<PendingAppend, IoError> {
        let payload = compress_payload(&data)?;
        self.enqueue_append(
            TENANT_RECORD_KIND_OTLP_LOGS,
            payload,
            Some(tenant),
            entries,
            Vec::new(),
            Vec::new(),
            mark,
            None,
            None,
        )
        .await
    }

    pub async fn append_trace(
        &self,
        tenant: TenantId,
        data: Vec<u8>,
        spans: Vec<TraceSpan>,
    ) -> Result<(), IoError> {
        self.enqueue_trace(tenant, data, spans, None)
            .await?
            .settle()
            .await
    }

    pub async fn enqueue_trace(
        &self,
        tenant: TenantId,
        data: Vec<u8>,
        spans: Vec<TraceSpan>,
        mark: Option<CollectMark>,
    ) -> Result<PendingAppend, IoError> {
        let payload = compress_payload(&data)?;
        self.enqueue_append(
            TENANT_RECORD_KIND_TRACES,
            payload,
            Some(tenant),
            Vec::new(),
            spans,
            Vec::new(),
            mark,
            None,
            None,
        )
        .await
    }

    /// An OTLP metrics export: `data` is the encoded
    /// `ExportMetricsServiceRequest`, `samples` what `series_ingest` already
    /// decomposed for the memtable. The checkpoint snapshots the series
    /// memtable with the other two, so a WAL prefix is only retired once the
    /// samples in it are durable in metric parts.
    pub async fn append_metrics(
        &self,
        tenant: TenantId,
        data: Vec<u8>,
        samples: Vec<MetricSample>,
    ) -> Result<(), IoError> {
        self.enqueue_metrics(tenant, data, samples, None)
            .await?
            .settle()
            .await
    }

    pub async fn enqueue_metrics(
        &self,
        tenant: TenantId,
        data: Vec<u8>,
        samples: Vec<MetricSample>,
        mark: Option<CollectMark>,
    ) -> Result<PendingAppend, IoError> {
        let payload = compress_payload(&data)?;
        self.enqueue_append(
            TENANT_RECORD_KIND_OTLP_METRICS,
            payload,
            Some(tenant),
            Vec::new(),
            Vec::new(),
            samples,
            mark,
            None,
            None,
        )
        .await
    }

    /// Enqueue every tenant portion of one metric export as one channel
    /// command. Sending the portions individually lets a closed/full journal
    /// accept and possibly fsync an earlier tenant before a later send fails.
    /// A single command makes channel admission all-or-nothing; the writer
    /// also writes and inserts all of its items in one fsync batch.
    pub(crate) async fn enqueue_metrics_reserved_batch(
        &self,
        records: Vec<ReservedMetricAppend>,
    ) -> Result<Vec<PendingAppend>, IoError> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let mut items = Vec::with_capacity(records.len());
        let mut pending = Vec::with_capacity(records.len());
        let queued_at = Instant::now();
        for record in records {
            let payload = compress_payload(&record.data)?;
            let framed_len = TENANT_RECORD_PREFIX_SIZE
                + record.tenant.as_str().len()
                + payload.len();
            if framed_len > MAX_RECORD_BYTES {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "journal record is too large: {framed_len} bytes (maximum {MAX_RECORD_BYTES})"
                    ),
                ));
            }
            let (done_tx, done_rx) = oneshot::channel();
            pending.push(PendingAppend(done_rx));
            items.push(AppendItem {
                kind: TENANT_RECORD_KIND_OTLP_METRICS,
                payload,
                tenant: Some(record.tenant),
                entries: Vec::new(),
                traces: Vec::new(),
                metric_samples: record.samples,
                metric_memory_permit: Some(record.metric_memory_permit),
                metric_series_admission: Some(record.metric_series_admission),
                mark: record.mark,
                queued_at,
                done: done_tx,
            });
        }

        self.tx
            .send(JournalCmd::AppendBatch(items))
            .await
            .map_err(|_| IoError::new(std::io::ErrorKind::BrokenPipe, "journal writer closed"))?;
        Ok(pending)
    }

    /// A number to remember with nothing to store under it.
    ///
    /// A record signy will never accept still has to move the mark past it,
    /// or the collecty that sent it resends it forever. Nothing else in a
    /// batch may need writing, so this is an append whose payload is empty.
    pub async fn enqueue_mark(&self, mark: CollectMark) -> Result<PendingAppend, IoError> {
        self.enqueue_append(
            TENANT_RECORD_KIND_OTLP_LOGS,
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Some(mark),
            None,
            None,
        )
        .await
    }

    pub fn collect_marks(&self) -> &Arc<CollectMarks> {
        &self.collect_marks
    }

    #[allow(clippy::too_many_arguments)]
    async fn enqueue_append(
        &self,
        kind: u8,
        payload: Vec<u8>,
        tenant: Option<TenantId>,
        entries: Vec<LogEntry>,
        traces: Vec<TraceSpan>,
        metric_samples: Vec<MetricSample>,
        mark: Option<CollectMark>,
        metric_memory_permit: Option<MetricMemoryPermit>,
        metric_series_admission: Option<crate::series::SeriesAdmission>,
    ) -> Result<PendingAppend, IoError> {
        let framed_len = tenant
            .as_ref()
            .map(|tenant| TENANT_RECORD_PREFIX_SIZE + tenant.as_str().len() + payload.len())
            .unwrap_or(0);
        if framed_len > MAX_RECORD_BYTES {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "journal record is too large: {framed_len} bytes (maximum {MAX_RECORD_BYTES})"
                ),
            ));
        }
        let (done_tx, done_rx) = oneshot::channel();
        // Stamped before the send rather than inside the writer: a full channel
        // makes `send` itself the wait, and a queue term that started after it
        // would report zero for exactly the case it exists to catch.
        let queued_at = Instant::now();
        self.tx
            .send(JournalCmd::Append(AppendItem {
                kind,
                payload,
                tenant,
                entries,
                traces,
                metric_samples,
                metric_memory_permit,
                metric_series_admission,
                mark,
                queued_at,
                done: done_tx,
            }))
            .await
            .map_err(|_| IoError::new(std::io::ErrorKind::BrokenPipe, "journal writer closed"))?;
        Ok(PendingAppend(done_rx))
    }

    pub fn trace_memtable(&self) -> Arc<TraceMemTable> {
        self.trace_memtable.clone()
    }

    pub fn series_memtable(&self) -> Arc<SeriesMemTable> {
        self.series_memtable.clone()
    }

    /// Reserve the projected growth of one or more metric exports against the
    /// same process-wide memtable ceiling used by logs and traces.  The
    /// caller has already inserted the reserved series entries; this permit
    /// covers the sample buffers that the journal writer will grow before it
    /// releases the charge after insertion.
    pub(crate) fn try_reserve_metric_bytes(
        &self,
        bytes: u64,
        limit: Option<u64>,
    ) -> Option<MetricMemoryPermit> {
        let Some(limit) = limit else {
            return Some(MetricMemoryPermit {
                counter: self.metric_reserved_bytes.clone(),
                bytes: 0,
            });
        };
        let observed = self
            .log_memtable()
            .approximate_size()
            .saturating_add(self.trace_memtable().approximate_size())
            .saturating_add(self.series_memtable().approximate_size()) as u64;
        let result = self.metric_reserved_bytes.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |reserved| {
                observed
                    .saturating_add(reserved)
                    .checked_add(bytes)
                    .filter(|total| *total <= limit)
                    .map(|_| reserved.saturating_add(bytes))
            },
        );
        result.ok().map(|_| MetricMemoryPermit {
            counter: self.metric_reserved_bytes.clone(),
            bytes,
        })
    }

    pub fn metric_reserved_bytes(&self) -> u64 {
        self.metric_reserved_bytes.load(Ordering::Acquire)
    }

    pub async fn checkpoint(&self) -> Result<CheckpointSnapshot, IoError> {
        let (done_tx, done_rx) = oneshot::channel();
        self.tx
            .send(JournalCmd::Checkpoint { done: done_tx })
            .await
            .map_err(|_| IoError::new(std::io::ErrorKind::BrokenPipe, "journal writer closed"))?;
        match done_rx.await {
            Ok(result) => result,
            Err(_) => Err(IoError::new(
                std::io::ErrorKind::BrokenPipe,
                "journal writer dropped",
            )),
        }
    }

    pub fn set_checkpoint(&self, offset: u64) -> Result<(), IoError> {
        write_checkpoint(&self.ckpt_path, offset)?;
        self.backlog.set_checkpoint_bytes(offset);
        Ok(())
    }

    /// Drops the durable WAL prefix through `offset`. This command runs in
    /// the writer task, so appends that arrived after the flush snapshot are
    /// copied into the replacement WAL before new appends can proceed.
    pub async fn compact_checkpoint(&self, offset: u64) -> Result<(), IoError> {
        let (done_tx, done_rx) = oneshot::channel();
        self.tx
            .send(JournalCmd::Compact {
                offset,
                done: done_tx,
            })
            .await
            .map_err(|_| IoError::new(std::io::ErrorKind::BrokenPipe, "journal writer closed"))?;
        done_rx.await.map_err(|_| {
            IoError::new(
                std::io::ErrorKind::BrokenPipe,
                "journal writer dropped during compaction",
            )
        })?
    }

    pub fn wal_path(&self) -> &Path {
        &self.wal_path
    }

    pub fn ckpt_path(&self) -> &Path {
        &self.ckpt_path
    }

    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Acquire)
    }
}


#[allow(clippy::too_many_arguments)]
async fn writer_loop(
    mut rx: mpsc::Receiver<JournalCmd>,
    path: &Path,
    ckpt_path: &Path,
    dir: &Path,
    memtable: Arc<MemTable>,
    trace_memtable: Arc<TraceMemTable>,
    series_memtable: Arc<SeriesMemTable>,
    max_batch_bytes: usize,
    max_batch_ms: u64,
    backlog: &WalBacklog,
    metrics: &JournalMetrics,
    collect_marks: &CollectMarks,
) -> Result<(), IoError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;

    let mut good_len = file.metadata().await?.len();
    backlog.set_wal_bytes(good_len);

    loop {
        let first = match rx.recv().await {
            Some(c) => c,
            None => break,
        };

        let mut pending_checkpoint: Option<oneshot::Sender<Result<CheckpointSnapshot, IoError>>> =
            None;
        let mut pending_compact: Option<(u64, oneshot::Sender<Result<(), IoError>>)> = None;

        let mut batch: Vec<AppendItem> = Vec::new();
        let mut batch_bytes = 0usize;
        let mut closed = false;

        match first {
            JournalCmd::Append(item) => {
                batch_bytes += item_len(&item);
                batch.push(item);
                // Take what has already arrived and write it. Waiting for more
                // charged every push the full linger even when the channel was
                // empty, which fixed single-connection throughput at
                // 1000/max_batch_ms pushes per second regardless of load.
                //
                // Batching still happens, and happens where it should: this
                // task is busy writing and fsyncing, so everything that arrives
                // during that window is already queued when the next iteration
                // looks. Group commit forms behind the write rather than in
                // front of it.
                let deadline = tokio::time::Instant::now() + Duration::from_millis(max_batch_ms);
                while batch_bytes < max_batch_bytes {
                    let next = if max_batch_ms == 0 {
                        match rx.try_recv() {
                            Ok(command) => Ok(Some(command)),
                            Err(mpsc::error::TryRecvError::Empty) => Err(()),
                            Err(mpsc::error::TryRecvError::Disconnected) => Ok(None),
                        }
                    } else {
                        // A deliberate linger: trades latency for fewer fsyncs
                        // on a disk where an fsync costs more than the wait.
                        tokio::time::timeout_at(deadline, rx.recv())
                            .await
                            .map_err(|_| ())
                    };
                    match next {
                        Ok(Some(JournalCmd::Append(item))) => {
                            batch_bytes += item_len(&item);
                            batch.push(item);
                        }
                        Ok(Some(JournalCmd::AppendBatch(items))) => {
                            batch_bytes = batch_bytes.saturating_add(
                                items.iter().map(item_len).fold(0usize, usize::saturating_add),
                            );
                            batch.extend(items);
                        }
                        Ok(Some(JournalCmd::Checkpoint { done })) => {
                            pending_checkpoint = Some(done);
                            break;
                        }
                        Ok(Some(JournalCmd::Compact { offset, done })) => {
                            pending_compact = Some((offset, done));
                            break;
                        }
                        Ok(None) => {
                            closed = true;
                            break;
                        }
                        Err(()) => break,
                    }
                }
            }
            JournalCmd::AppendBatch(items) => {
                batch_bytes = items
                    .iter()
                    .map(item_len)
                    .fold(0usize, usize::saturating_add);
                batch.extend(items);
                // A batch command is already an enqueue-atomic metric
                // export. It may share this fsync with neighboring appends,
                // but no part of the command can arrive independently.
                let deadline = tokio::time::Instant::now() + Duration::from_millis(max_batch_ms);
                while batch_bytes < max_batch_bytes {
                    let next = if max_batch_ms == 0 {
                        match rx.try_recv() {
                            Ok(command) => Ok(Some(command)),
                            Err(mpsc::error::TryRecvError::Empty) => Err(()),
                            Err(mpsc::error::TryRecvError::Disconnected) => Ok(None),
                        }
                    } else {
                        tokio::time::timeout_at(deadline, rx.recv())
                            .await
                            .map_err(|_| ())
                    };
                    match next {
                        Ok(Some(JournalCmd::Append(item))) => {
                            batch_bytes = batch_bytes.saturating_add(item_len(&item));
                            batch.push(item);
                        }
                        Ok(Some(JournalCmd::AppendBatch(items))) => {
                            batch_bytes = batch_bytes.saturating_add(
                                items.iter().map(item_len).fold(0usize, usize::saturating_add),
                            );
                            batch.extend(items);
                        }
                        Ok(Some(JournalCmd::Checkpoint { done })) => {
                            pending_checkpoint = Some(done);
                            break;
                        }
                        Ok(Some(JournalCmd::Compact { offset, done })) => {
                            pending_compact = Some((offset, done));
                            break;
                        }
                        Ok(None) => {
                            closed = true;
                            break;
                        }
                        Err(()) => break,
                    }
                }
            }
            JournalCmd::Checkpoint { done } => {
                pending_checkpoint = Some(done);
            }
            JournalCmd::Compact { offset, done } => {
                pending_compact = Some((offset, done));
            }
        }

        if !batch.is_empty() {
            // Taken before the framing so the queue term ends where the work on
            // this batch begins, and every push in the batch is measured
            // against the same instant.
            let batch_started = Instant::now();
            let mut oldest_queued = batch_started;
            for item in &batch {
                metrics
                    .append_queue_wait
                    .observe(batch_started.saturating_duration_since(item.queued_at));
                oldest_queued = oldest_queued.min(item.queued_at);
            }
            metrics.batches.fetch_add(1, Ordering::Relaxed);
            metrics
                .batched_records
                .fetch_add(batch.len() as u64, Ordering::Relaxed);
            let wal_arena = crate::memprof::enter(crate::memprof::Arena::Wal);
            let mut buf = Vec::with_capacity(batch_bytes + batch.len() * RECORD_HEADER_SIZE);
            // The highest number each collecty reached in this batch. Written
            // after the records, in the same buffer and so under the same
            // `sync_all`: a mark that survived a crash is a promise that
            // everything it covers survived with it.
            let mut batch_marks: Vec<CollectMark> = Vec::new();
            for item in &batch {
                if let Some(mark) = item.mark {
                    match batch_marks
                        .iter_mut()
                        .find(|seen| seen.sender == mark.sender)
                    {
                        Some(seen) => seen.at = seen.at.max(mark.at),
                        None => batch_marks.push(mark),
                    }
                }
            }
            for item in &batch {
                let (kind, payload) = (&item.kind, &item.payload);
                let Some(tenant) = item.tenant.as_ref() else {
                    continue;
                };
                // Framed here, straight into the batch buffer. The frame used
                // to be its own prefix+tenant+payload Vec built on the ingest
                // task, which copied every export once more than the write
                // needed; the CRC covers exactly the framed bytes, computed
                // over the pieces in the order they land.
                let tenant_bytes = tenant.as_str().as_bytes();
                let framed_len = framed_record_len(tenant, payload);
                let len = u32::try_from(framed_len).map_err(|_| {
                    IoError::new(
                        std::io::ErrorKind::InvalidInput,
                        "journal record exceeds u32",
                    )
                })?;
                let mut hasher = crc32fast::Hasher::new();
                hasher.update(TENANT_RECORD_MAGIC);
                hasher.update(&[*kind, tenant_bytes.len() as u8]);
                hasher.update(tenant_bytes);
                hasher.update(payload);
                let crc = hasher.finalize();
                buf.extend_from_slice(&len.to_le_bytes());
                buf.extend_from_slice(&crc.to_le_bytes());
                buf.extend_from_slice(TENANT_RECORD_MAGIC);
                buf.push(*kind);
                buf.push(tenant_bytes.len() as u8);
                buf.extend_from_slice(tenant_bytes);
                buf.extend_from_slice(payload);
            }

            for mark in &batch_marks {
                let mut framed = Vec::with_capacity(MARK_RECORD_BYTES);
                frame_mark(mark, &mut framed);
                buf.extend_from_slice(&(framed.len() as u32).to_le_bytes());
                buf.extend_from_slice(&crc32fast::hash(&framed).to_le_bytes());
                buf.extend_from_slice(&framed);
            }

            drop(wal_arena);
            let framed = Instant::now();
            let mut fsync_started = framed;
            let write_result = async {
                file.write_all(&buf).await?;
                file.flush().await?;
                fsync_started = Instant::now();
                file.sync_all().await
            }
            .await;
            let fsynced = Instant::now();
            metrics
                .batch_write
                .observe(fsync_started.saturating_duration_since(framed));
            metrics
                .batch_fsync
                .observe(fsynced.saturating_duration_since(fsync_started));

            match write_result {
                Ok(()) => {
                    good_len += buf.len() as u64;
                    // Published before the acks below: a client that sees its
                    // 204 must already be counted against the next request's
                    // backpressure gate.
                    backlog.set_wal_bytes(good_len);
                    // The entries were allocated under the ingest label and
                    // are moved, not copied, so the memtable's own nodes are
                    // charged to the same arena its contents already are.
                    let _arena = crate::memprof::enter(crate::memprof::Arena::Ingest);
                    // After the fsync, never before: the mark is what lets a
                    // resend be skipped, and skipping a record this instance
                    // has not actually stored is the one way to lose one.
                    for mark in &batch_marks {
                        collect_marks.advance(*mark);
                    }
                    let records = batch.len();
                    for item in batch.drain(..) {
                        if let Some(tenant) = item.tenant {
                            memtable.insert(tenant, item.entries);
                        }
                        trace_memtable.insert(item.traces);
                        series_memtable.insert(item.metric_samples);
                        if let Some(admission) = item.metric_series_admission {
                            admission.commit();
                        }
                        drop(item.metric_memory_permit);
                        let _ = item.done.send(Ok(()));
                    }
                    let inserted = Instant::now();
                    let insert = inserted.saturating_duration_since(fsynced);
                    metrics.batch_insert.observe(insert);
                    // Named per event, because a histogram cannot say which
                    // phase any one slow batch was in and the tail is what the
                    // push p95 argument is about.
                    let total = inserted.saturating_duration_since(batch_started);
                    if total >= SLOW_BATCH {
                        tracing::warn!(
                            records,
                            bytes = buf.len(),
                            total_ms = total.as_secs_f64() * 1e3,
                            queue_ms = batch_started
                                .saturating_duration_since(oldest_queued)
                                .as_secs_f64()
                                * 1e3,
                            write_ms = fsync_started.saturating_duration_since(framed).as_secs_f64()
                                * 1e3,
                            fsync_ms = fsynced.saturating_duration_since(fsync_started).as_secs_f64()
                                * 1e3,
                            insert_ms = insert.as_secs_f64() * 1e3,
                            "journal batch slow"
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "journal write failed, truncating partial record");
                    for item in batch.drain(..) {
                        let _ = item.done.send(Err(IoError::new(e.kind(), e.to_string())));
                    }
                    let recovered = async {
                        file.set_len(good_len).await?;
                        file.sync_all().await
                    }
                    .await;
                    if let Err(te) = recovered {
                        tracing::error!(error = %te, "journal truncate failed, fencing writer");
                        return Err(te);
                    }
                    backlog.set_wal_bytes(good_len);
                }
            }
        }

        if let Some(done) = pending_checkpoint {
            let checkpoint_started = Instant::now();
            if let Err(e) = file.sync_all().await {
                let _ = done.send(Err(IoError::new(e.kind(), e.to_string())));
                return Err(e);
            }
            let offset = good_len;
            // Ahead of the offset the flush loop will store when it is done,
            // which is the order these two files have to be written in: marks
            // ahead of the checkpoint name records the WAL still holds, and
            // marks behind it would have those records sent a second time.
            if let Err(error) = collect_marks.store(dir) {
                tracing::warn!(%error, "the collect marks could not be written");
            }
            let snapshot = memtable.begin_flush();
            let trace_snapshot = trace_memtable.begin_flush();
            let series_snapshot = series_memtable.begin_flush();
            metrics.checkpoint.observe(checkpoint_started.elapsed());
            let _ = done.send(Ok(CheckpointSnapshot {
                offset,
                snapshot,
                trace_snapshot,
                series_snapshot,
            }));
        }

        if let Some((offset, done)) = pending_compact {
            let result = compact_wal(&mut file, path, ckpt_path, offset, &mut good_len).await;
            match result {
                Ok(()) => {
                    // Compaction resets the checkpoint to zero and leaves only
                    // the retained suffix, so both halves of the backlog move.
                    backlog.set_checkpoint_bytes(0);
                    backlog.set_wal_bytes(good_len);
                    let _ = done.send(Ok(()));
                }
                Err(error) => {
                    let error_for_caller = IoError::new(error.kind(), error.to_string());
                    let _ = done.send(Err(error_for_caller));
                    // Compaction can fail before or after replacing the WAL
                    // (for example, a directory fsync can fail after rename).
                    // Reopen the path and continue serving appends so the
                    // caller can retry the same checkpoint instead of
                    // permanently fencing the journal writer.
                    let reopened = async {
                        let reopened = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(path)
                            .await?;
                        let length = reopened.metadata().await?.len();
                        Ok::<_, IoError>((reopened, length))
                    }
                    .await;
                    match reopened {
                        Ok((reopened, length)) => {
                            file = reopened;
                            good_len = length;
                            backlog.set_checkpoint_bytes(read_checkpoint(ckpt_path).unwrap_or(0));
                            backlog.set_wal_bytes(good_len);
                        }
                        Err(reopen_error) => {
                            tracing::error!(
                                error = %reopen_error,
                                "journal reopen after compaction failure failed; fencing writer"
                            );
                            return Err(reopen_error);
                        }
                    }
                }
            }
        }

        if closed {
            break;
        }
    }

    Ok(())
}



/// What one append costs the batch buffer. A mark with nothing under it costs
/// its own record and no tenant frame.
fn item_len(item: &AppendItem) -> usize {
    item.tenant
        .as_ref()
        .map(|tenant| framed_record_len(tenant, &item.payload))
        .unwrap_or(0)
        + item
            .mark
            .map(|_| RECORD_HEADER_SIZE + MARK_RECORD_BYTES)
            .unwrap_or(0)
}
