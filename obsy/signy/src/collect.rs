//! The one way telemetry enters this engine: a collecty's batch.
//!
//! There used to be two more — OTLP over HTTP on `POST /v1/logs`,
//! `/v1/traces`, `/v1/metrics`, and the same three services over gRPC on
//! their own listener. Both are gone. An engine that accepts a push straight
//! from an application has no disk queue in front of it, so a refusal it gives
//! is telemetry lost unless the application happens to hold it; collecty's
//! queue is what makes a refusal survivable, and it only helps when nothing
//! can go around it. So applications export to collecty, collecty ships here,
//! and this route is the whole ingest surface.
//!
//! What arrived over those routes still arrives: the payload of every record
//! below is an OTLP export request, protobuf-encoded, and the admission and
//! normalization it goes through is the code the push handlers used to call.
//! What went with them is the OTLP JSON encoding, which collecty does not take
//! either — see `docs/ARCHITECTURE.md`, "Ingest is OTLP".

use std::collections::VecDeque;
use std::io::Write;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost014::Message;

use futures_util::StreamExt;

use crate::AppState;
use crate::backpressure::{InflightBody, IngestError};
use crate::journal::{CollectSignal, PendingAppend};
use crate::log_ingest::OtlpLogIngest;
use crate::series_ingest::OtlpMetricIngest;
use crate::trace_ingest::{MAX_OTLP_REQUEST_BYTES, OtlpTraceIngest};

/// The header collecty writes in front of each record's payload: the payload's
/// length, little-endian. The signal is not repeated here — the request names
/// it once for the whole segment.
const COLLECT_RECORD_HEADER_BYTES: usize = 4;

/// The one ceiling this route has: a single record's payload, which is one
/// OTLP export and is bounded as `MAX_OTLP_REQUEST_BYTES` bounds one.
///
/// The batch around it has no ceiling any more. The body is decompressed and
/// ingested as it arrives, so what this server holds at any moment is one
/// record and the handful behind it still waiting on an fsync — not the batch.
/// How much a collecty ships in one request is now its own decision, made on
/// how long it is willing to wait for an answer, rather than on what this
/// route would agree to hold in memory.
pub const MAX_COLLECT_RECORD_BYTES: usize = MAX_OTLP_REQUEST_BYTES;

/// Records handed to the journal writer and not yet fsynced.
///
/// Awaiting each record before reading the next would put a whole fsync round
/// trip between one record and the next, and a batch of a thousand would pay a
/// thousand of them. The writer already groups whatever is in its channel into
/// one write and one `sync_all`, so handing records over back to back is what
/// lets a batch share fsyncs the way the merged path used to. The bound is
/// what stops a long stream from holding an unbounded number of decoded
/// exports.
const MAX_INFLIGHT_RECORDS: usize = 32;

/// Which collecty sent the segment, which of its three streams it belongs to,
/// and which segment of that stream it is.
///
/// The body carries no numbers of its own and does not need to: a segment is
/// sent from its first record every time, so counting while reading places
/// every record without a per-record cost on the wire. The signal is here for
/// the same reason — a segment holds one signal's exports and no others, so
/// the answer is the same for the whole body.
pub const COLLECT_SENDER_HEADER: &str = "x-collecty-sender";
pub const COLLECT_SIGNAL_HEADER: &str = "x-collecty-signal";
pub const COLLECT_SEGMENT_HEADER: &str = "x-collecty-segment";

/// Compressed bytes fed to the decoder before its output is drained.
///
/// Small on purpose: a decoder handed a whole chunk at once can produce every
/// record in it before any is taken away, and the point of reading this way is
/// that the server never holds the batch.
const DECODER_FEED_BYTES: usize = 16 * 1024;

/// One collected batch, one signal, read as it arrives.
///
/// collecty keeps a queue per signal, so a batch is one signal's exports back
/// to back inside one zstd stream, and the request says which. Every record is
/// a complete export on its own, which is what makes reading the body a record
/// at a time possible at all — nothing has to be held back waiting for a later
/// part of the batch to make sense of it.
pub async fn collect(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Body,
) -> Result<Response, IngestError> {
    // Ahead of the request counter, and once for the whole batch: the checks
    // behind it are the instance's, not a signal's, so a fenced or overloaded
    // server refuses the batch rather than dropping part of it.
    crate::backpressure::admit_batch(&state.shutdown, &state.ingest_gate)?;
    state
        .metrics
        .ingest_requests
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let result = collect_inner(&state, &headers, body).await;
    if result.is_err() {
        state
            .metrics
            .ingest_errors
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    result
}

async fn collect_inner(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    body: axum::body::Body,
) -> Result<Response, IngestError> {
    let encoding = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !encoding.eq_ignore_ascii_case("zstd") {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            format!("the collect route takes Content-Encoding: zstd, not {encoding:?}"),
        )
            .into());
    }

    let signal = collect_signal(headers)?;
    let sender = collect_sender(headers)?;
    let marks = state.journal.collect_marks();
    let at = sender
        .map(|(id, _)| marks.position(&id, signal))
        .unwrap_or(crate::journal::Position::START);

    // Already stored whole, on an attempt whose answer this collecty never
    // heard. Answered from the headers alone — the body is a copy of what is
    // already on disk and there is nothing to learn by decompressing it.
    if let Some((id, segment)) = sender
        && segment < at.segment
    {
        tracing::debug!(
            sender = %id,
            signal = signal.as_str(),
            segment,
            "a segment this instance already has, answered unread"
        );
        return Ok(answer(at, 0));
    }
    if let Some((id, segment)) = sender
        && segment > at.segment
    {
        tracing::warn!(
            sender = %id,
            signal = signal.as_str(),
            segment,
            expected = at.segment,
            "a collecty starts past what this instance has; the segments between are gone"
        );
    }

    // How many of this segment's records were stored by an earlier attempt.
    // Zero unless this is a resend of the one that was interrupted.
    let already = match sender {
        Some((_, segment)) if segment == at.segment => at.records,
        _ => 0,
    };

    let mut records = CollectedRecords::new(body)?;
    // One record can be several appends now, because the tenant is read off
    // each resource and one export may name more than one. They stay grouped
    // so that `MAX_INFLIGHT_RECORDS` keeps bounding records, which is what the
    // memory it defends is proportional to.
    let mut inflight: VecDeque<(InflightBody, Vec<PendingAppend>)> = VecDeque::new();
    let mut index = 0u64;
    let mut rejected = 0u64;

    while let Some(payload) = records.next().await? {
        let seen = index;
        index += 1;
        if seen < already {
            state
                .metrics
                .collect_skipped_records
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            continue;
        }
        let mark = sender.map(|(id, segment)| crate::journal::CollectMark {
            sender: id,
            signal,
            at: crate::journal::Position {
                segment,
                records: index,
            },
        });
        while inflight.len() >= MAX_INFLIGHT_RECORDS {
            settle_oldest(&mut inflight).await?;
        }
        let bytes = payload.len();
        let permit = admit_record(state, &mut inflight, bytes as u64).await?;
        match enqueue_record(state, signal, payload, mark).await {
            Ok((pending, refused)) => {
                rejected += refused;
                inflight.push_back((permit, pending));
            }
            // Permanent: a decode failure, a body past a limit, a tenant over
            // what it stores. Sending these back would only have collecty
            // halve the batch to find them and drop them anyway, one wasted
            // round trip at a time, so they are dropped here and counted.
            Err(error) if never_acceptable(error.status) => {
                tracing::warn!(
                    signal = signal.as_str(),
                    status = error.status.as_u16(),
                    bytes,
                    reason = error.message,
                    "dropping a collected record signy will not take"
                );
                state
                    .metrics
                    .collect_dropped_records
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                state
                    .metrics
                    .collect_dropped_bytes
                    .fetch_add(bytes as u64, std::sync::atomic::Ordering::Relaxed);
            }
            // Everything else is this instance's problem, not the record's —
            // it is behind, draining, or configured against the tenant that
            // sent this. Answering with it stops the batch here, and whatever
            // is already durable stays durable.
            Err(error) => return Err(error),
        }
    }

    while !inflight.is_empty() {
        settle_oldest(&mut inflight).await?;
    }

    // The body ended where it said it would, so the segment is done. This is
    // the record that says so, and it is what a collecty reads as permission
    // to unlink the file. It also covers records this instance will never
    // accept: those leave nothing in the WAL, and without it the collecty
    // would offer them forever.
    if let Some((id, segment)) = sender {
        state
            .journal
            .enqueue_mark(crate::journal::CollectMark {
                sender: id,
                signal,
                at: crate::journal::Position {
                    segment: segment + 1,
                    records: 0,
                },
            })
            .await
            .map_err(crate::log_ingest::journal_write_failed)?
            .settle()
            .await
            .map_err(crate::log_ingest::journal_write_failed)?;
    }

    Ok(answer(
        sender
            .map(|(id, _)| marks.position(&id, signal))
            .unwrap_or(crate::journal::Position::START),
        rejected,
    ))
}

/// The last segment this instance holds whole, and what it would not keep.
///
/// `stored` is the one a collecty reads: everything at or below it can be
/// unlinked. `rejected` is datapoints a metrics record lost to the active-series
/// cap — the number the OTLP `partial_success` carried while the push routes
/// existed, which has nowhere else to go now that they do not. A collecty
/// ignores it; it exists so that a partial acceptance is visible to whoever is
/// looking, rather than only in this process's log.
fn answer(at: crate::journal::Position, rejected: u64) -> Response {
    (
        [(header::CONTENT_TYPE, "application/json")],
        format!(
            "{{\"stored\":{},\"rejected\":{rejected}}}",
            at.whole_segments()
        ),
    )
        .into_response()
}

/// Which of the three signals the batch carries.
///
/// Required of every caller, collecty or not: a record no longer names its own
/// signal, so without this there is nothing to say what the body holds.
fn collect_signal(headers: &HeaderMap) -> Result<CollectSignal, IngestError> {
    headers
        .get(COLLECT_SIGNAL_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(CollectSignal::parse)
        .ok_or_else(|| {
            IngestError::from((
                StatusCode::BAD_REQUEST,
                format!("{COLLECT_SIGNAL_HEADER} must name a signal: logs, traces or metrics"),
            ))
        })
}

/// Who sent the segment and which segment it is, when the request says.
///
/// Absent means a caller that is not a collecty — a test, or a hand-made
/// request. Nothing is skipped and no mark is written for one: the numbering
/// belongs to a queue, and a caller without a queue has none.
fn collect_sender(
    headers: &HeaderMap,
) -> Result<Option<(crate::journal::SenderId, u64)>, IngestError> {
    let Some(raw) = headers.get(COLLECT_SENDER_HEADER) else {
        return Ok(None);
    };
    let sender = raw
        .to_str()
        .ok()
        .and_then(crate::journal::SenderId::parse)
        .ok_or_else(|| {
            IngestError::from((
                StatusCode::BAD_REQUEST,
                format!("{COLLECT_SENDER_HEADER} is not a sender id"),
            ))
        })?;
    let segment = headers
        .get(COLLECT_SEGMENT_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|segment| *segment > 0)
        .ok_or_else(|| {
            IngestError::from((
                StatusCode::BAD_REQUEST,
                format!("{COLLECT_SEGMENT_HEADER} must be a segment number above zero"),
            ))
        })?;
    Ok(Some((sender, segment)))
}

/// Charge one record against the in-flight ceiling, waiting on this batch's
/// own records before deciding the server is full.
///
/// The gate refuses rather than queues, and a long stream is its own biggest
/// contributor: without this it would eventually refuse itself while holding
/// everything it had admitted. Draining one record frees its charge, so the
/// batch throttles itself down to whatever the ceiling allows instead of
/// answering an overload it caused.
async fn admit_record(
    state: &Arc<AppState>,
    inflight: &mut VecDeque<(InflightBody, Vec<PendingAppend>)>,
    bytes: u64,
) -> Result<InflightBody, IngestError> {
    loop {
        match state.ingest_gate.admit_body(bytes) {
            Ok(permit) => return Ok(permit),
            Err(error) => {
                if inflight.is_empty() {
                    return Err(error);
                }
                settle_oldest(inflight).await?;
            }
        }
    }
}

async fn settle_oldest(
    inflight: &mut VecDeque<(InflightBody, Vec<PendingAppend>)>,
) -> Result<(), IngestError> {
    let Some((permit, pending)) = inflight.pop_front() else {
        return Ok(());
    };
    let mut settled = Ok(());
    // Every append of the record is awaited even after one has failed: they
    // were all handed to the writer, and leaving a receiver undropped behind a
    // failure would have the next settle read this record's answer.
    for pending in pending {
        let one = pending
            .settle()
            .await
            .map_err(crate::log_ingest::journal_write_failed);
        settled = settled.and(one);
    }
    drop(permit);
    settled
}

/// The record's appends and the legacy metric refusal count.
///
/// Metric memory/cardinality pressure is export-wide and is returned as 429;
/// it never becomes a partial-success count.  The count remains in the wire
/// response for compatibility with old callers and stays zero on this path.
async fn enqueue_record(
    state: &Arc<AppState>,
    signal: CollectSignal,
    payload: Vec<u8>,
    mark: Option<crate::journal::CollectMark>,
) -> Result<(Vec<PendingAppend>, u64), IngestError> {
    match signal {
        CollectSignal::Logs => collect_logs(state, payload, mark).await.map(|p| (p, 0)),
        CollectSignal::Traces => collect_traces(state, payload, mark).await.map(|p| (p, 0)),
        CollectSignal::Metrics => collect_metrics(state, payload, mark).await,
    }
}

/// Whether an answer means "not this body, however long it waits", so that
/// dropping it loses nothing a retry would have saved.
///
/// Every client error is here, and the one that is a judgement call is `429`.
///
/// A tenant this instance does not serve was the other one, answered `403`. It
/// no longer reaches a status at all: the tenant is a resource attribute, so
/// the resource is dropped inside a record that is still accepted, and
/// `signy_ingest_dropped_resources_total` is where that loss shows instead of
/// here.
///
/// Every `429` is retryable.  In particular, metric memory/cardinality
/// admission now reaches this point after the request was decoded; dropping
/// it here would defeat the collector's durable queue and turn backpressure
/// into silent loss.
fn never_acceptable(status: StatusCode) -> bool {
    status.is_client_error() && status != StatusCode::TOO_MANY_REQUESTS
}

/// The batch's records, handed out as the socket produces them.
///
/// Two layers unwrap here, neither of which needs the whole body. The frames
/// are decompressed into whatever the decoder has been able to produce, and
/// the plain bytes behind them are `length | payload` repeated, so a record can
/// be taken as soon as its last byte has arrived. What is buffered is one
/// record at most.
struct CollectedRecords {
    body: axum::body::BodyDataStream,
    decoder: zstd::stream::write::Decoder<'static, Vec<u8>>,
    pending: Bytes,
    ended: bool,
}

impl CollectedRecords {
    fn new(body: axum::body::Body) -> Result<CollectedRecords, IngestError> {
        let decoder = zstd::stream::write::Decoder::new(Vec::new()).map_err(|error| {
            IngestError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("a zstd decoder could not be made: {error}"),
            ))
        })?;
        Ok(CollectedRecords {
            body: body.into_data_stream(),
            decoder,
            pending: Bytes::new(),
            ended: false,
        })
    }

    async fn next(&mut self) -> Result<Option<Vec<u8>>, IngestError> {
        loop {
            if let Some(record) = self.take()? {
                return Ok(Some(record));
            }
            if !self.pending.is_empty() {
                let take = self.pending.len().min(DECODER_FEED_BYTES);
                let chunk = self.pending.split_to(take);
                self.decoder.write_all(&chunk).map_err(|error| {
                    IngestError::from((
                        StatusCode::BAD_REQUEST,
                        format!("the batch could not be decompressed: {error}"),
                    ))
                })?;
                continue;
            }
            if self.ended {
                let left = self.decoder.get_ref().len();
                if left == 0 {
                    return Ok(None);
                }
                return Err(self.truncated(left));
            }
            match self.body.next().await {
                Some(Ok(chunk)) => self.pending = chunk,
                Some(Err(error)) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        format!("the batch stopped arriving: {error}"),
                    )
                        .into());
                }
                None => {
                    self.ended = true;
                    self.decoder.flush().map_err(|error| {
                        IngestError::from((
                            StatusCode::BAD_REQUEST,
                            format!("the batch could not be decompressed: {error}"),
                        ))
                    })?;
                }
            }
        }
    }

    /// The record at the front of the decoded bytes, if all of it has arrived.
    fn take(&mut self) -> Result<Option<Vec<u8>>, IngestError> {
        let plain = self.decoder.get_ref();
        if plain.len() < COLLECT_RECORD_HEADER_BYTES {
            return Ok(None);
        }
        let len = u32::from_le_bytes(
            plain[..COLLECT_RECORD_HEADER_BYTES]
                .try_into()
                .expect("four bytes"),
        ) as usize;
        // Checked before the bytes are waited for, not after they arrive: the
        // length is what says how much to buffer, so trusting it first is what
        // an unbounded buffer would look like.
        if len > MAX_COLLECT_RECORD_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "a record claims {len} bytes, over the maximum of {MAX_COLLECT_RECORD_BYTES}"
                ),
            )
                .into());
        }
        let record_end = COLLECT_RECORD_HEADER_BYTES + len;
        if plain.len() < record_end {
            return Ok(None);
        }
        let plain = self.decoder.get_mut();
        let payload = plain[COLLECT_RECORD_HEADER_BYTES..record_end].to_vec();
        plain.drain(..record_end);
        Ok(Some(payload))
    }

    fn truncated(&self, left: usize) -> IngestError {
        let plain = self.decoder.get_ref();
        if left < COLLECT_RECORD_HEADER_BYTES {
            return (
                StatusCode::BAD_REQUEST,
                format!(
                    "a record header needs {COLLECT_RECORD_HEADER_BYTES} bytes and {left} are left"
                ),
            )
                .into();
        }
        let len = u32::from_le_bytes(
            plain[..COLLECT_RECORD_HEADER_BYTES]
                .try_into()
                .expect("four bytes"),
        );
        (
            StatusCode::BAD_REQUEST,
            format!(
                "a record claims {len} bytes and {} are left",
                left - COLLECT_RECORD_HEADER_BYTES
            ),
        )
            .into()
    }
}

fn log_ingest(state: &Arc<AppState>) -> OtlpLogIngest<'_> {
    OtlpLogIngest {
        journal: &state.journal,
        config: &state.config,
        tenant_quota: &state.tenant_quota,
        tenant_policy: &state.tenant_policy,
        metrics: &state.metrics,
        clock: &state.clock,
    }
}

async fn collect_logs(
    state: &Arc<AppState>,
    payload: Vec<u8>,
    mark: Option<crate::journal::CollectMark>,
) -> Result<Vec<PendingAppend>, IngestError> {
    let ingest = log_ingest(state);
    ingest.admit_size(payload.len())?;
    let request = ExportLogsServiceRequest::decode(payload.as_slice()).map_err(|error| {
        IngestError::from((
            StatusCode::BAD_REQUEST,
            format!("OTLP protobuf decode failed: {error}"),
        ))
    })?;
    ingest.enqueue_request(request, Some(payload), mark).await
}

async fn collect_traces(
    state: &Arc<AppState>,
    payload: Vec<u8>,
    mark: Option<crate::journal::CollectMark>,
) -> Result<Vec<PendingAppend>, IngestError> {
    let ingest = OtlpTraceIngest {
        journal: &state.journal,
        tenant_quota: &state.tenant_quota,
        tenant_policy: &state.tenant_policy,
        metrics: &state.metrics,
    };
    ingest.admit_size(payload.len())?;
    let request = ExportTraceServiceRequest::decode(payload.as_slice()).map_err(|error| {
        IngestError::from((
            StatusCode::BAD_REQUEST,
            format!("OTLP protobuf decode failed: {error}"),
        ))
    })?;
    ingest.enqueue_request(request, mark).await
}

async fn collect_metrics(
    state: &Arc<AppState>,
    payload: Vec<u8>,
    mark: Option<crate::journal::CollectMark>,
) -> Result<(Vec<PendingAppend>, u64), IngestError> {
    let ingest = OtlpMetricIngest {
        journal: &state.journal,
        config: &state.config,
        tenant_quota: &state.tenant_quota,
        tenant_policy: &state.tenant_policy,
        metrics: &state.metrics,
        clock: &state.clock,
    };
    ingest.admit_size(payload.len())?;
    let request = {
        // Scoped so the guard ends before the await below: a suspended task
        // would otherwise leave its label on a worker thread.
        let _arena = crate::memprof::enter(crate::memprof::Arena::Ingest);
        ExportMetricsServiceRequest::decode(payload.as_slice()).map_err(|error| {
            IngestError::from((
                StatusCode::BAD_REQUEST,
                format!("OTLP protobuf decode failed: {error}"),
            ))
        })?
    };
    let (pending, outcome) = ingest.enqueue_request(request, mark).await?;
    // Logged as well as answered: a collecty reads the status and nothing
    // else, so the log is where an operator finds the reason.
    if let Some(partial) = outcome.partial_success() {
        tracing::warn!(
            rejected_points = partial.rejected_data_points,
            reason = partial.error_message,
            "collected metrics were partially refused"
        );
    }
    Ok((pending, outcome.rejected_data_points))
}

#[cfg(test)]
mod tests {
    include!("tests/collect.rs");
}
