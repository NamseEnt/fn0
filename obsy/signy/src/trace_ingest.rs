use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost014::Message;

use crate::backpressure::IngestError;
use crate::journal::Journal;
use crate::trace::normalize_request;
use axum::http::StatusCode;

pub const MAX_OTLP_REQUEST_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_OTLP_SPANS: usize = 100_000;

/// Accepting one OTLP trace export, independent of how it arrived. The trace
/// counterpart to [`crate::log_ingest::OtlpLogIngest`], and split for the same
/// reason: a limit enforced on gRPC and forgotten on HTTP is not a limit.
pub struct OtlpTraceIngest<'a> {
    pub journal: &'a Journal,
    pub tenant_quota: &'a crate::tenant_quota::TenantQuota,
    pub tenant_policy: &'a crate::tenant_policy::TenantPolicy,
    pub metrics: &'a crate::metrics::RuntimeMetrics,
}

impl OtlpTraceIngest<'_> {
    /// See [`crate::log_ingest::OtlpLogIngest::admit_size`]: the tenant is no
    /// longer knowable this early, and the size still is.
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

    /// The trace counterpart to
    /// [`crate::log_ingest::OtlpLogIngest::enqueue_request`], with the same
    /// ordering: the span cap over the whole request and before the split, the
    /// drop tally counted rather than answered, one journal record per tenant.
    pub async fn enqueue_request(
        &self,
        request: ExportTraceServiceRequest,
        mark: Option<crate::journal::CollectMark>,
    ) -> Result<Vec<crate::journal::PendingAppend>, IngestError> {
        let span_count = count_spans(&request)?;
        if span_count > MAX_OTLP_SPANS {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("OTLP request contains more than {MAX_OTLP_SPANS} spans"),
            )
                .into());
        }

        let split = crate::otlp_tenant::split_traces(request, self.tenant_policy);
        split.dropped.record(self.metrics, "traces");

        let last = split.groups.len().saturating_sub(1);
        let mut pending = Vec::with_capacity(split.groups.len());
        for (index, (tenant, group)) in split.groups.into_iter().enumerate() {
            if let Err(error) = self.tenant_quota.admit_storage(&tenant) {
                tracing::warn!(%tenant, reason = error.message, "dropping spans for a tenant at its storage limit");
                continue;
            }
            let mark = if index == last { mark } else { None };
            pending.push(self.enqueue(tenant, group, mark).await?);
        }
        Ok(pending)
    }

    pub async fn enqueue(
        &self,
        tenant: crate::tenant::TenantId,
        request: ExportTraceServiceRequest,
        mark: Option<crate::journal::CollectMark>,
    ) -> Result<crate::journal::PendingAppend, IngestError> {
        let spans = normalize_request(&tenant, request.clone())
            .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
        let mut encoded = Vec::new();
        request.encode(&mut encoded).map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("failed to encode request: {error}"),
            )
        })?;
        self.journal
            .enqueue_trace(tenant, encoded, spans, mark)
            .await
            .map_err(crate::log_ingest::journal_write_failed)
    }
}

/// Spans one export carries, counted over the whole request before it is
/// split: the cap bounds the normalization each group pays for, so N groups
/// must not be able to multiply it.
fn count_spans(request: &ExportTraceServiceRequest) -> Result<usize, IngestError> {
    request
        .resource_spans
        .iter()
        .flat_map(|resource| resource.scope_spans.iter())
        .map(|scope| scope.spans.len())
        .try_fold(0usize, |count, spans| count.checked_add(spans))
        .ok_or_else(|| {
            IngestError::from((
                StatusCode::PAYLOAD_TOO_LARGE,
                "OTLP span count overflow".to_string(),
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backpressure::IngestGate;
    use crate::config::Config;
    use crate::journal;
    use crate::part_registry::PartRegistry;
    use crate::tenant::test_tenant;
    use crate::trace::TraceMemTable;
    use crate::trace_registry::TraceRegistry;
    use opentelemetry_proto::tonic::common::v1::AnyValue;
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use std::sync::Arc;
    use std::time::Duration;

    /// The collect route's own sequence over one record, so these tests refuse
    /// what the route refuses and store what it stores.
    struct Ingest {
        journal: Arc<Journal>,
        shutdown: Arc<crate::shutdown::ShutdownState>,
        ingest_gate: Arc<IngestGate>,
        tenant_quota: Arc<crate::tenant_quota::TenantQuota>,
        tenant_policy: Arc<crate::tenant_policy::TenantPolicy>,
        metrics: Arc<crate::metrics::RuntimeMetrics>,
    }

    impl Ingest {
        fn over(journal: Arc<Journal>, config: &Config) -> Self {
            let ingest_gate = IngestGate::for_test(&journal, config);
            Self {
                journal,
                shutdown: Arc::new(crate::shutdown::ShutdownState::new()),
                ingest_gate,
                tenant_quota: crate::tenant_quota::TenantQuota::for_test(config),
                tenant_policy: Arc::new(crate::tenant_policy::TenantPolicy::disabled()),
                metrics: Arc::new(crate::metrics::RuntimeMetrics::new()),
            }
        }

        async fn accept(&self, request: ExportTraceServiceRequest) -> Result<(), IngestError> {
            let ingest = OtlpTraceIngest {
                journal: &self.journal,
                tenant_quota: &self.tenant_quota,
                tenant_policy: &self.tenant_policy,
                metrics: &self.metrics,
            };
            crate::backpressure::admit_batch(&self.shutdown, &self.ingest_gate)?;
            ingest.admit_size(request.encoded_len())?;
            for pending in ingest.enqueue_request(request, None).await? {
                pending
                    .settle()
                    .await
                    .map_err(crate::log_ingest::journal_write_failed)?;
            }
            Ok(())
        }
    }

    fn journal_over(config: &Config, traces: Arc<TraceMemTable>) -> Arc<Journal> {
        std::fs::create_dir_all(&config.data_dir).unwrap();
        Arc::new(
            journal::Journal::spawn_with_traces(
                config,
                Arc::new(crate::memtable::MemTable::new()),
                traces,
            )
            .unwrap(),
        )
    }

    fn config_at(name: &str) -> Config {
        Config {
            data_dir: std::env::temp_dir().join(format!("signy-{name}-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        }
    }

    /// The tenant rides in the payload, so what makes a request the test
    /// tenant's is the resource `request` builds.
    fn request() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(crate::otlp_tenant::test_tenant_resource()),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![1; 16],
                        span_id: vec![2; 8],
                        start_time_unix_nano: 10,
                        end_time_unix_nano: 20,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    #[tokio::test]
    async fn an_export_is_rejected_while_draining_for_shutdown() {
        let config = config_at("trace-drain");
        let trace_memtable = Arc::new(TraceMemTable::new());
        let journal = journal_over(&config, trace_memtable.clone());
        let ingest = Ingest::over(journal, &config);
        ingest.shutdown.begin_drain();

        let error = ingest.accept(request()).await.unwrap_err();

        assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(
            trace_memtable.is_empty(),
            "a drained OTLP request must not be appended"
        );
    }

    /// One process, one memory budget: every signal answers to the same
    /// thresholds, or refusing one just moves the overrun into another.
    #[tokio::test]
    async fn an_export_is_refused_once_the_buffers_are_over_their_limit() {
        let config = Config {
            flush_max_bytes: 1,
            max_memtable_bytes: Some(1),
            ..config_at("trace-ingest")
        };
        let trace_memtable = Arc::new(TraceMemTable::new());
        let journal = journal_over(&config, trace_memtable.clone());
        let ingest = Ingest::over(journal, &config);

        ingest
            .accept(request())
            .await
            .expect("the first export is under the limit");
        let error = ingest
            .accept(request())
            .await
            .expect_err("a full buffer must be refused");

        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            trace_memtable
                .query_trace_id(&test_tenant(), &"01".repeat(16))
                .len(),
            1,
            "the refused export must not have been appended"
        );
    }

    #[tokio::test]
    async fn an_export_is_acknowledged_after_the_journal_append() {
        let config = config_at("trace-ingest");
        let trace_memtable = Arc::new(TraceMemTable::new());
        let journal = journal_over(&config, trace_memtable.clone());
        let ingest = Ingest::over(journal, &config);

        ingest.accept(request()).await.unwrap();

        assert_eq!(
            trace_memtable
                .query_trace_id(&test_tenant(), &"01".repeat(16))
                .len(),
            1
        );

        let replayed = TraceMemTable::new();
        journal::replay_with_traces(
            ingest.journal.wal_path(),
            ingest.journal.ckpt_path(),
            &crate::memtable::MemTable::new(),
            &replayed,
        )
        .unwrap();
        assert_eq!(
            replayed
                .query_trace_id(&test_tenant(), &"01".repeat(16))
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn an_invalid_export_is_rejected_without_inserting() {
        let config = config_at("trace-invalid");
        let trace_memtable = Arc::new(TraceMemTable::new());
        let journal = journal_over(&config, trace_memtable.clone());
        let ingest = Ingest::over(journal, &config);
        let mut invalid = request();
        invalid.resource_spans[0].scope_spans[0].spans[0].trace_id = vec![0; 16];

        let error = ingest.accept(invalid).await.unwrap_err();

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(trace_memtable.is_empty());
    }

    #[tokio::test]
    async fn an_oversized_export_is_rejected_without_inserting() {
        let config = config_at("trace-oversized");
        let trace_memtable = Arc::new(TraceMemTable::new());
        let journal = journal_over(&config, trace_memtable.clone());
        let ingest = Ingest::over(journal, &config);
        let mut oversized = request();
        oversized.resource_spans[0].scope_spans[0].spans[0].name =
            "x".repeat(MAX_OTLP_REQUEST_BYTES);

        let error = ingest.accept(oversized).await.unwrap_err();

        // Permanent for this record, and the collect route reads it that way:
        // a client error is dropped and counted rather than answered, because
        // resending the identical bytes produces the identical refusal.
        assert_eq!(error.status, StatusCode::PAYLOAD_TOO_LARGE);
        assert!(trace_memtable.is_empty());
    }

    #[tokio::test]
    async fn an_export_with_too_many_spans_is_rejected_before_normalization() {
        let config = config_at("trace-span-limit");
        let trace_memtable = Arc::new(TraceMemTable::new());
        let journal = journal_over(&config, trace_memtable.clone());
        let ingest = Ingest::over(journal, &config);
        let mut too_many = request();
        too_many.resource_spans[0].scope_spans[0].spans = vec![Span::default(); MAX_OTLP_SPANS + 1];

        let error = ingest.accept(too_many).await.unwrap_err();

        // Same class as the oversized body: a span count over the cap cannot
        // become acceptable by being retried.
        assert_eq!(error.status, StatusCode::PAYLOAD_TOO_LARGE);
        assert!(trace_memtable.is_empty());
    }

    #[tokio::test]
    async fn an_export_flushes_to_a_trace_part_and_reloads_after_restart() {
        let config = Config {
            flush_max_interval: Duration::from_millis(20),
            flush_check_interval: Duration::from_millis(10),
            ..config_at("trace-flush")
        };
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let log_memtable = Arc::new(crate::memtable::MemTable::new());
        let trace_memtable = Arc::new(TraceMemTable::new());
        let journal = Arc::new(
            journal::Journal::spawn_with_traces(
                &config,
                log_memtable.clone(),
                trace_memtable.clone(),
            )
            .unwrap(),
        );
        let parts = Arc::new(PartRegistry::new());
        let trace_parts = Arc::new(TraceRegistry::new(parts.operation_lock()));
        let healthy = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let flush_task = tokio::spawn(crate::flush::flush_loop(
            log_memtable,
            trace_memtable.clone(),
            journal.clone(),
            parts.clone(),
            trace_parts.clone(),
            Arc::new(crate::series_registry::SeriesRegistry::standalone()),
            None,
            Arc::new(config.clone()),
            healthy,
            Arc::new(crate::metrics::RuntimeMetrics::new()),
            tokio::sync::watch::channel(false).1,
        ));

        let ingest = Ingest::over(journal, &config);
        ingest.accept(request()).await.unwrap();
        for _ in 0..100 {
            if trace_parts.part_count() == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(trace_parts.part_count(), 1);
        assert_eq!(
            trace_parts
                .query_trace_id(&test_tenant(), &"01".repeat(16), None, None)
                .unwrap()
                .len(),
            1
        );
        flush_task.abort();

        let restored =
            TraceRegistry::load_from_disk(&config.data_dir.join("traces"), parts.operation_lock())
                .unwrap();
        assert_eq!(
            restored
                .query_trace_id(&test_tenant(), &"01".repeat(16), None, None)
                .unwrap()
                .len(),
            1
        );
    }

    // Keep the generated AnyValue import exercised when the dependency's
    // generated API changes; the test data intentionally uses the minimal
    // valid request above.
    #[allow(dead_code)]
    fn _any_value_type_is_available(_: AnyValue) {}
}
