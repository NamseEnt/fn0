use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use prost014::Message;

use crate::backpressure::IngestError;
use crate::config::Config;
use crate::journal::Journal;
use crate::otlp_log::normalize_request;
use crate::trace_ingest::MAX_OTLP_REQUEST_BYTES;
use axum::http::StatusCode;

/// Records one export may carry. Sized like `MAX_OTLP_SPANS` and for the same
/// reason: the request is normalized in memory before anything is appended, so
/// the count has to be bounded before that work starts.
pub const MAX_OTLP_LOG_RECORDS: usize = 100_000;

/// Accepting one OTLP log export, independent of how it arrived.
///
/// gRPC and HTTP differ only in how the request is framed and how a refusal is
/// spelled, so both go through this. Splitting the decision here rather than
/// duplicating it is what keeps a limit from being enforced on one transport
/// and forgotten on the other.
pub struct OtlpLogIngest<'a> {
    pub journal: &'a Journal,
    pub config: &'a Config,
    pub tenant_quota: &'a crate::tenant_quota::TenantQuota,
    pub tenant_policy: &'a crate::tenant_policy::TenantPolicy,
    pub metrics: &'a crate::metrics::RuntimeMetrics,
    pub clock: &'a crate::clock::Clock,
}

impl OtlpLogIngest<'_> {
    /// What the size alone decides, before the body is decoded and before
    /// anyone knows whose it is.
    ///
    /// The tenant used to be half of this check, read off a header the
    /// transport had in hand. It now lives inside the payload, so the only
    /// thing knowable this early is how big the payload is — and that is the
    /// half worth keeping early, since it is what bounds the decode.
    pub fn admit_size(&self, encoded_len: usize) -> Result<(), IngestError> {
        if encoded_len > MAX_OTLP_REQUEST_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("OTLP request exceeds the maximum of {MAX_OTLP_REQUEST_BYTES} bytes"),
            )
                .into());
        }
        Ok(())
    }

    /// File an export under the tenants its resources name, one journal record
    /// per tenant.
    ///
    /// The record cap is counted over the whole request and before the split,
    /// so N groups cannot multiply what one export may carry. What the split
    /// throws away — a resource naming no tenant, an unparseable one, one this
    /// instance does not serve — is counted and dropped here rather than
    /// refused: the answer an ingest gives says whether the body arrived, and
    /// nothing about whose it was.
    ///
    /// Groups are appended in order and a failure part way through leaves the
    /// earlier ones durable. A retry then re-sends them, which is the same
    /// at-least-once the WAL replay already has, and it cannot arise at all
    /// while an export names one tenant — which is every export a collecty
    /// forwards, since it keeps one record per export.
    pub async fn enqueue_request(
        &self,
        request: ExportLogsServiceRequest,
        wire: Option<Vec<u8>>,
        mark: Option<crate::journal::CollectMark>,
    ) -> Result<Vec<crate::journal::PendingAppend>, IngestError> {
        let record_count = count_records(&request)?;
        if record_count > MAX_OTLP_LOG_RECORDS {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("OTLP request contains more than {MAX_OTLP_LOG_RECORDS} log records"),
            )
                .into());
        }

        let split = crate::otlp_tenant::split_logs(request, self.tenant_policy);
        split.dropped.record(self.metrics, "logs");
        // Only an untouched single-tenant request is still described by the
        // bytes that arrived. A split one is not, and neither is one a drop
        // took a resource out of.
        let mut wire = split.is_intact().then_some(wire).flatten();

        let last = split.groups.len().saturating_sub(1);
        let mut pending = Vec::with_capacity(split.groups.len());
        for (index, (tenant, group)) in split.groups.into_iter().enumerate() {
            // A tenant at its storage limit is dropped like an unserved one.
            // A request may now carry several tenants, and one tenant's full
            // plan must not refuse another's data; `signy_storage_limit_
            // rejected_total`, which this bumps, is where it shows.
            if let Err(error) = self.tenant_quota.admit_storage(&tenant) {
                tracing::warn!(%tenant, reason = error.message, "dropping logs for a tenant at its storage limit");
                continue;
            }
            // The mark accounts for the whole record, so it rides the last
            // append: a crash before that one leaves the record unaccounted
            // and the collecty offers it again.
            let mark = if index == last { mark } else { None };
            pending.push(self.enqueue(tenant, group, wire.take(), mark).await?);
        }
        Ok(pending)
    }

    /// The same admission and normalization, stopping at the point the writer
    /// has the record and has not yet fsynced it. A caller with a run of
    /// records to write hands them over back to back and awaits them
    /// afterwards, so one fsync covers the run instead of one each.
    pub async fn enqueue(
        &self,
        tenant: crate::tenant::TenantId,
        request: ExportLogsServiceRequest,
        wire: Option<Vec<u8>>,
        mark: Option<crate::journal::CollectMark>,
    ) -> Result<crate::journal::PendingAppend, IngestError> {
        // The WAL bytes are settled before normalization consumes the message:
        // the HTTP protobuf body arrives as exactly the bytes the WAL wants,
        // and the other transports re-encode the message they decoded.
        let encoded = match wire {
            Some(bytes) => bytes,
            None => request.encode_to_vec(),
        };
        let entries = normalize_request(request)
            .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
        // A record that arrives over OTLP is still bounded: a line has a size
        // limit, and a timestamp outside the window would land in a partition
        // retention has already swept.
        let window = crate::ingest::TimestampWindow::from_config(self.config, self.clock);
        for entry in &entries {
            if entry.line.len() > self.config.max_line_bytes {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "log line is {} bytes, exceeding the maximum of {}",
                        entry.line.len(),
                        self.config.max_line_bytes
                    ),
                )
                    .into());
            }
            window
                .validate(entry.timestamp_ns)
                .map_err(|error| (StatusCode::BAD_REQUEST, error))?;
        }

        // The WAL stores the export itself (`encoded` above). This used to
        // materialize a Loki `PushRequest` — a second message with a clone
        // per line and per label, serialized just so replay had one decoder —
        // measured as the largest remaining term of `docs/VISION.md`
        // invariant II's copy count. Replay decodes by the record's kind
        // instead, the way traces always have.
        self.journal
            .enqueue_otlp_logs(tenant, encoded, entries, mark)
            .await
            .map_err(journal_write_failed)
    }
}

/// Records one export carries, counted before anything is split or decoded
/// further, because the normalization this bounds happens per group and the
/// cap is a whole-request one.
fn count_records(request: &ExportLogsServiceRequest) -> Result<usize, IngestError> {
    request
        .resource_logs
        .iter()
        .flat_map(|resource| resource.scope_logs.iter())
        .map(|scope| scope.log_records.len())
        .try_fold(0usize, |count, records| count.checked_add(records))
        .ok_or_else(|| {
            IngestError::from((
                StatusCode::PAYLOAD_TOO_LARGE,
                "OTLP log record count overflow".to_string(),
            ))
        })
}

pub fn journal_write_failed(error: std::io::Error) -> IngestError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("journal write failed: {error}"),
    )
        .into()
}

#[cfg(test)]
mod tests {
    include!("tests/log_ingest.rs");
}
