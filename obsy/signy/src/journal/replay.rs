#[allow(dead_code)]
pub fn replay(
    wal_path: &Path,
    ckpt_path: &Path,
    memtable: &MemTable,
) -> Result<(u64, u64), String> {
    let traces = TraceMemTable::new();
    replay_with_traces(wal_path, ckpt_path, memtable, &traces)
}

pub fn replay_with_signals(
    wal_path: &Path,
    ckpt_path: &Path,
    memtable: &MemTable,
    trace_memtable: &TraceMemTable,
    series_memtable: &SeriesMemTable,
) -> Result<(u64, u64), String> {
    replay_reporting(
        wal_path,
        ckpt_path,
        memtable,
        trace_memtable,
        series_memtable,
        &CollectMarks::default(),
    )
    .map(|report| (report.checkpoint, report.end_offset))
}

/// What a replay put back, so a restart can be told from a clean start.
///
/// Delivery is at-least-once by design: the checkpoint advances after a flush,
/// so a crash in between leaves records in the WAL that are already durable in
/// parts, and replay writes them a second time. The copies are collapsed the
/// first time the two parts are merged, but until then they are two log lines,
/// and this is the only thing that says how many there could be — an operator
/// could not otherwise tell a restart that duplicated nothing from one that
/// duplicated a minute of logs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayReport {
    pub checkpoint: u64,
    pub end_offset: u64,
    pub records: u64,
    pub entries: u64,
}

pub fn replay_with_traces(
    wal_path: &Path,
    ckpt_path: &Path,
    memtable: &MemTable,
    trace_memtable: &TraceMemTable,
) -> Result<(u64, u64), String> {
    let series = SeriesMemTable::new();
    replay_with_signals(wal_path, ckpt_path, memtable, trace_memtable, &series)
}

pub fn replay_reporting(
    wal_path: &Path,
    ckpt_path: &Path,
    memtable: &MemTable,
    trace_memtable: &TraceMemTable,
    series_memtable: &SeriesMemTable,
    collect_marks: &CollectMarks,
) -> Result<ReplayReport, String> {
    recover_unfinished_compaction(wal_path, ckpt_path).map_err(|e| e.to_string())?;
    let checkpoint = read_checkpoint(ckpt_path).map_err(|e| e.to_string())?;
    let mut report = ReplayReport {
        checkpoint,
        ..ReplayReport::default()
    };
    let end = replay_from(
        wal_path,
        checkpoint,
        memtable,
        trace_memtable,
        series_memtable,
        collect_marks,
        &mut report,
    )?;
    report.end_offset = end;
    Ok(report)
}

fn recover_unfinished_compaction(wal_path: &Path, ckpt_path: &Path) -> Result<(), IoError> {
    let state_path = wal_path.with_file_name(COMPACTION_STATE_FILE);
    let Some(state) = read_compaction_state(&state_path)? else {
        return Ok(());
    };
    let tmp_path = wal_path.with_extension("wal.compact.tmp");
    if !tmp_path.exists() {
        // The replacement WAL is already in place; replay its suffix from
        // checkpoint zero. The intent record still has to go, for the same
        // reason as above.
        remove_compaction_state(&state_path, wal_path)?;
        return Ok(());
    }

    // Rename never committed. Restore the old checkpoint before replay so a
    // crash between checkpoint=0 and rename cannot replay flushed records.
    write_checkpoint(ckpt_path, state.offset)?;
    std::fs::remove_file(&tmp_path)?;
    std::fs::remove_file(&state_path)?;
    if let Some(parent) = wal_path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn replay_from(
    wal_path: &Path,
    checkpoint: u64,
    memtable: &MemTable,
    trace_memtable: &TraceMemTable,
    series_memtable: &SeriesMemTable,
    collect_marks: &CollectMarks,
    report: &mut ReplayReport,
) -> Result<u64, String> {
    if !wal_path.exists() {
        if checkpoint == 0 {
            return Ok(0);
        }
        return Err(format!(
            "journal checkpoint {checkpoint} exists but WAL {} is missing",
            wal_path.display()
        ));
    }
    let mut file = std::fs::File::open(wal_path).map_err(|e| e.to_string())?;
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    if checkpoint > file_len {
        return Err(format!(
            "journal checkpoint {checkpoint} is beyond WAL length {file_len}"
        ));
    }
    if checkpoint == file_len {
        return Ok(checkpoint);
    }
    file.seek(SeekFrom::Start(checkpoint))
        .map_err(|e| e.to_string())?;
    let mut reader = std::io::BufReader::new(file);
    let mut offset = checkpoint;
    let mut replayed = 0u64;
    loop {
        let mut header = [0u8; RECORD_HEADER_SIZE];
        match reader.read_exact(&mut header) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.to_string()),
        }
        let len = u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let expected_crc = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let record_end = offset
            .checked_add(RECORD_HEADER_SIZE as u64)
            .and_then(|end| end.checked_add(len as u64))
            .ok_or_else(|| format!("journal record length overflows at offset {offset}"))?;
        if record_end > file_len {
            tracing::warn!(
                offset,
                len,
                "journal partial record at tail, stopping replay"
            );
            break;
        }
        if len > MAX_RECORD_BYTES {
            return Err(format!(
                "journal record at offset {offset} is too large: {len} bytes (maximum {MAX_RECORD_BYTES})"
            ));
        }
        let mut data = vec![0u8; len];
        match reader.read_exact(&mut data) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                tracing::warn!(offset, "journal partial record at tail, stopping replay");
                break;
            }
            Err(e) => return Err(e.to_string()),
        }
        let actual_crc = crc32fast::hash(&data);
        if actual_crc != expected_crc {
            if record_end == file_len {
                tracing::warn!(offset, "journal crc mismatch at tail, stopping replay");
                break;
            }
            return Err(format!("journal record crc mismatch at offset {offset}"));
        }
        // Ahead of the tenant frame: a mark belongs to no tenant, and the
        // records it covers are the ones being replayed around it. Recovering
        // it here is what stops a restart from taking a collecty's whole
        // resend as new.
        if let Some(mark) = decode_mark(&data) {
            collect_marks.advance(mark);
        } else if let Some((tenant, kind, payload)) = decode_tenant_record(&data)
            .map_err(|error| format!("journal record invalid at offset {offset}: {error}"))?
        {
            match kind {
                // The Loki push kind. Nothing writes it since ingest became
                // OTLP only, and nothing on disk or on the wire is versioned
                // here by decision — a data directory from before the change
                // is deleted, not migrated.
                TENANT_RECORD_KIND_LOGS => {
                    return Err(format!(
                        "journal record at offset {offset} is a Loki push record from before \
ingest became OTLP only; this engine versions nothing, so delete the data directory \
and re-ingest"
                    ));
                }
                TENANT_RECORD_KIND_TRACES => {
                    // Stored zstd-compressed; a payload that fails the zstd
                    // header is a WAL from before compression, and this
                    // engine versions nothing.
                    let payload = decompress_payload(payload).map_err(|e| {
                        format!(
                            "{e} at offset {offset}; if this WAL predates journal \
compression, delete the data directory and re-ingest"
                        )
                    })?;
                    replay_trace_record(&tenant, &payload, offset, trace_memtable)?;
                }
                TENANT_RECORD_KIND_OTLP_LOGS => {
                    let payload = decompress_payload(payload).map_err(|e| {
                        format!(
                            "{e} at offset {offset}; if this WAL predates journal \
compression, delete the data directory and re-ingest"
                        )
                    })?;
                    report.entries +=
                        replay_otlp_log_record(&tenant, &payload, offset, memtable)?;
                }
                TENANT_RECORD_KIND_OTLP_METRICS => {
                    let payload = decompress_payload(payload).map_err(|e| {
                        format!(
                            "{e} at offset {offset}; if this WAL predates journal \
compression, delete the data directory and re-ingest"
                        )
                    })?;
                    replay_otlp_metric_record(&tenant, &payload, offset, series_memtable)?;
                }
                other => {
                    return Err(format!(
                        "unsupported tenant journal record kind {other} at offset {offset}"
                    ));
                }
            }
        } else {
            return Err(format!(
                "journal record at offset {offset} has no tenant frame; that is a WAL from \
before tenant framing, and this engine versions nothing — delete the data directory \
and re-ingest"
            ));
        }
        offset += (RECORD_HEADER_SIZE + len) as u64;
        replayed += 1;
        report.records += 1;
    }
    if replayed > 0 {
        tracing::info!(replayed, offset, "journal replay complete");
    }
    Ok(offset)
}

/// The payload is the export as
/// it arrived, normalized here exactly as ingest normalized it before the
/// crash. An `EmptyRequest` is skipped rather than fatal — a record was only
/// appended after normalizing non-empty, so hitting one here is a bug to log,
/// not a reason to refuse startup — while any other normalization failure is
/// the same hard error a corrupt kind-0 record is.
fn replay_otlp_log_record(
    tenant: &TenantId,
    payload: &[u8],
    offset: u64,
    memtable: &MemTable,
) -> Result<u64, String> {
    let request = ExportLogsServiceRequest::decode(payload)
        .map_err(|e| format!("OTLP log protobuf decode failed at offset {offset}: {e}"))?;
    let entries = match crate::otlp_log::normalize_request(request) {
        Ok(entries) => entries,
        Err(crate::otlp_log::OtlpLogError::EmptyRequest) => {
            tracing::warn!(offset, "empty OTLP log record in journal, skipping");
            return Ok(0);
        }
        Err(e) => return Err(format!("OTLP log record invalid at offset {offset}: {e}")),
    };
    let replayed_entries = entries.len() as u64;
    memtable.insert(tenant.clone(), entries);
    Ok(replayed_entries)
}

fn replay_trace_record(
    tenant: &TenantId,
    payload: &[u8],
    offset: u64,
    trace_memtable: &TraceMemTable,
) -> Result<(), String> {
    let request = ExportTraceServiceRequest::decode(payload)
        .map_err(|e| format!("trace protobuf decode failed at offset {offset}: {e}"))?;
    let spans = normalize_request(tenant, request)
        .map_err(|e| format!("trace record invalid at offset {offset}: {e}"))?;
    trace_memtable.insert(spans);
    Ok(())
}

/// The payload is the export as it arrived, re-run through the same pure
/// decomposition live ingest used, then inserted in WAL order — which is what
/// makes the delta-to-cumulative fold reproduce the totals it produced the
/// first time. An `EmptyRequest` is a logged bug, not a refused startup, on
/// the same reasoning as the log record's.
fn replay_otlp_metric_record(
    tenant: &TenantId,
    payload: &[u8],
    offset: u64,
    series_memtable: &SeriesMemTable,
) -> Result<(), String> {
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    let request = ExportMetricsServiceRequest::decode(payload)
        .map_err(|e| format!("OTLP metrics protobuf decode failed at offset {offset}: {e}"))?;
    let samples = match crate::series_ingest::normalize_request(tenant, &request) {
        Ok(samples) => samples,
        Err(crate::series_ingest::MetricIngestError::EmptyRequest) => {
            tracing::warn!(offset, "empty OTLP metrics record in journal, skipping");
            return Ok(());
        }
        Err(e) => return Err(format!("OTLP metrics record invalid at offset {offset}: {e}")),
    };
    series_memtable.insert(samples);
    Ok(())
}

