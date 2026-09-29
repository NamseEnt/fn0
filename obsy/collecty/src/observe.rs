use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::metrics::v1::{
    AggregationTemporality, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    metric, number_data_point,
};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost::Message;

use crate::queue::{Queue, Spool};
use crate::send::SenderStats;

#[derive(Clone, Copy, Debug, Default)]
pub struct Observation {
    pub queued_bytes: u64,
    pub segments: u64,
    pub appended_records: u64,
    pub appended_bytes: u64,
    pub dropped_bytes: u64,
    pub dropped_segments: u64,
    pub sent_segments: u64,
    pub sent_bytes: u64,
    pub refused_segments: u64,
    pub refused_bytes: u64,
    pub retries: u64,
    pub host_metrics_exports: u64,
    pub host_metrics_errors: u64,
    pub journal_exports: u64,
    pub journal_errors: u64,
}

#[derive(Default)]
pub struct SourceStats {
    pub host_metrics_exports: AtomicU64,
    pub host_metrics_errors: AtomicU64,
    pub journal_exports: AtomicU64,
    pub journal_errors: AtomicU64,
}

enum Kind {
    Gauge,
    Counter,
}

struct Family {
    name: &'static str,
    unit: &'static str,
    kind: Kind,
    read: fn(&Observation) -> u64,
}

const FAMILIES: &[Family] = &[
    // There is no separate backlog gauge any more. A segment signy has
    // answered for is unlinked on the spot, so what the queue occupies and
    // what it still owes are the same number.
    Family {
        name: "collecty_queue_bytes",
        unit: "By",
        kind: Kind::Gauge,
        read: |observation| observation.queued_bytes,
    },
    Family {
        name: "collecty_queue_segments",
        unit: "{segment}",
        kind: Kind::Gauge,
        read: |observation| observation.segments,
    },
    Family {
        name: "collecty_records_appended_total",
        unit: "{record}",
        kind: Kind::Counter,
        read: |observation| observation.appended_records,
    },
    // Plain bytes, before the segment compresses them. Against
    // `collecty_bytes_sent_total` this is the ratio a host is achieving.
    Family {
        name: "collecty_bytes_appended_total",
        unit: "By",
        kind: Kind::Counter,
        read: |observation| observation.appended_bytes,
    },
    Family {
        name: "collecty_queue_dropped_bytes_total",
        unit: "By",
        kind: Kind::Counter,
        read: |observation| observation.dropped_bytes,
    },
    Family {
        name: "collecty_queue_dropped_segments_total",
        unit: "{segment}",
        kind: Kind::Counter,
        read: |observation| observation.dropped_segments,
    },
    Family {
        name: "collecty_segments_sent_total",
        unit: "{segment}",
        kind: Kind::Counter,
        read: |observation| observation.sent_segments,
    },
    Family {
        name: "collecty_bytes_sent_total",
        unit: "By",
        kind: Kind::Counter,
        read: |observation| observation.sent_bytes,
    },
    Family {
        name: "collecty_segments_refused_total",
        unit: "{segment}",
        kind: Kind::Counter,
        read: |observation| observation.refused_segments,
    },
    Family {
        name: "collecty_bytes_refused_total",
        unit: "By",
        kind: Kind::Counter,
        read: |observation| observation.refused_bytes,
    },
    Family {
        name: "collecty_send_retries_total",
        unit: "{retry}",
        kind: Kind::Counter,
        read: |observation| observation.retries,
    },
    Family {
        name: "collecty_host_metrics_exports_total",
        unit: "{export}",
        kind: Kind::Counter,
        read: |observation| observation.host_metrics_exports,
    },
    Family {
        name: "collecty_host_metrics_errors_total",
        unit: "{error}",
        kind: Kind::Counter,
        read: |observation| observation.host_metrics_errors,
    },
    Family {
        name: "collecty_journal_exports_total",
        unit: "{export}",
        kind: Kind::Counter,
        read: |observation| observation.journal_exports,
    },
    Family {
        name: "collecty_journal_errors_total",
        unit: "{error}",
        kind: Kind::Counter,
        read: |observation| observation.journal_errors,
    },
];

pub struct Reporter {
    queue: Arc<Queue>,
    stats: Arc<SenderStats>,
    spool: Spool,
    started_unix_nanos: u64,
    generated_telemetry_tenant: Option<String>,
    source_stats: Arc<SourceStats>,
}

impl Reporter {
    pub fn new(
        queue: Arc<Queue>,
        stats: Arc<SenderStats>,
        spool: Spool,
        generated_telemetry_tenant: Option<String>,
        source_stats: Arc<SourceStats>,
    ) -> Reporter {
        Reporter {
            queue,
            stats,
            spool,
            started_unix_nanos: unix_nanos(),
            generated_telemetry_tenant,
            source_stats,
        }
    }

    pub fn observe(&self) -> Observation {
        let queued = self.queue.stats();
        Observation {
            queued_bytes: queued.queued_bytes,
            segments: queued.segments as u64,
            appended_records: queued.appended_records,
            appended_bytes: queued.appended_bytes,
            dropped_bytes: queued.dropped_bytes,
            dropped_segments: queued.dropped_segments,
            sent_segments: self.stats.sent_segments.load(Ordering::Relaxed),
            sent_bytes: self.stats.sent_bytes.load(Ordering::Relaxed),
            refused_segments: self.stats.refused_segments.load(Ordering::Relaxed),
            refused_bytes: self.stats.refused_bytes.load(Ordering::Relaxed),
            retries: self.stats.retries.load(Ordering::Relaxed),
            host_metrics_exports: self
                .source_stats
                .host_metrics_exports
                .load(Ordering::Relaxed),
            host_metrics_errors: self
                .source_stats
                .host_metrics_errors
                .load(Ordering::Relaxed),
            journal_exports: self.source_stats.journal_exports.load(Ordering::Relaxed),
            journal_errors: self.source_stats.journal_errors.load(Ordering::Relaxed),
        }
    }

    /// The export to push, or `None` when no tenant was configured.
    ///
    /// signy drops an export naming no tenant and says nothing about it, so a
    /// collecty with none produces bytes that are queued, shipped and thrown
    /// away. The stderr summary below runs either way, so the numbers are
    /// still available — only the push of them is not.
    pub fn export(&self, observed: &Observation) -> Option<Vec<u8>> {
        let tenant_id = self.generated_telemetry_tenant.as_deref()?;
        Some(encode(
            observed,
            self.started_unix_nanos,
            unix_nanos(),
            tenant_id,
        ))
    }

    /// The queue's own numbers, and how the one thread behind them is coping.
    ///
    /// The queue's files are handled by two threads: one for everything that
    /// takes the queue's lock, one for reading segments back out. Both are
    /// reported, because what went wrong when they were one thread was not the
    /// channel filling — it never got near — but short work waiting behind
    /// long work in the same queue. The wait figures are cumulative, so a
    /// run's last line covers the run.
    pub fn log(&self, observed: &Observation) {
        let spool = self.spool.report();
        tracing::info!(
            spool_requests = spool.writes.requests,
            spool_max_depth = spool.writes.max_depth,
            spool_wait_p99_us = spool.writes.wait_p99_us,
            spool_wait_max_us = spool.writes.wait_max_us,
            reader_requests = spool.reads.requests,
            reader_max_depth = spool.reads.max_depth,
            reader_wait_p99_us = spool.reads.wait_p99_us,
            reader_wait_max_us = spool.reads.wait_max_us,
            queued_bytes = observed.queued_bytes,
            segments = observed.segments,
            appended_records = observed.appended_records,
            appended_bytes = observed.appended_bytes,
            dropped_bytes = observed.dropped_bytes,
            dropped_segments = observed.dropped_segments,
            sent_segments = observed.sent_segments,
            sent_bytes = observed.sent_bytes,
            refused_segments = observed.refused_segments,
            retries = observed.retries,
            "queue report"
        );
    }
}

/// The tenant ID is stamped onto the generated metrics resource because that
/// is where signy reads it from. collecty never writes it onto anything it
/// forwards — those carry their own, and collecty does not decode them.
pub fn encode(observed: &Observation, started: u64, now: u64, tenant_id: &str) -> Vec<u8> {
    let metrics = FAMILIES
        .iter()
        .map(|family| {
            let points = vec![NumberDataPoint {
                start_time_unix_nano: started,
                time_unix_nano: now,
                value: Some(number_data_point::Value::AsInt(
                    (family.read)(observed) as i64
                )),
                ..Default::default()
            }];
            Metric {
                name: family.name.to_string(),
                unit: family.unit.to_string(),
                data: Some(match family.kind {
                    Kind::Gauge => metric::Data::Gauge(Gauge {
                        data_points: points,
                    }),
                    Kind::Counter => metric::Data::Sum(Sum {
                        data_points: points,
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                        is_monotonic: true,
                    }),
                }),
                ..Default::default()
            }
        })
        .collect();

    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                attributes: vec![
                    attribute(crate::TENANT_ATTRIBUTE, tenant_id),
                    attribute("service.name", "collecty"),
                ],
                ..Default::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                metrics,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

fn attribute(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}

fn unix_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_is_exported_once_with_one_point() {
        let observed = Observation {
            queued_bytes: 11,
            sent_segments: 22,
            ..Observation::default()
        };

        let export = ExportMetricsServiceRequest::decode(
            encode(&observed, 5, 9, "collecty-tenant").as_slice(),
        )
        .expect("a decodable export");
        let scope = &export.resource_metrics[0].scope_metrics[0];

        assert_eq!(scope.metrics.len(), FAMILIES.len());
        for metric in &scope.metrics {
            let points = match metric.data.as_ref().expect("data") {
                metric::Data::Gauge(gauge) => &gauge.data_points,
                metric::Data::Sum(sum) => &sum.data_points,
                other => panic!("unexpected metric shape: {other:?}"),
            };
            assert_eq!(points.len(), 1, "{}", metric.name);
            assert!(points[0].attributes.is_empty(), "{}", metric.name);
        }
    }

    /// The one export collecty builds itself has to say whose it is, or signy
    /// drops it without telling anyone.
    #[test]
    fn the_self_export_names_the_configured_tenant() {
        let export = ExportMetricsServiceRequest::decode(
            encode(&Observation::default(), 5, 9, "collecty-tenant").as_slice(),
        )
        .expect("a decodable export");
        let attributes = &export.resource_metrics[0]
            .resource
            .as_ref()
            .expect("a resource")
            .attributes;
        let named = attributes
            .iter()
            .find(|attribute| attribute.key == crate::TENANT_ATTRIBUTE)
            .expect("the tenant attribute");
        assert_eq!(
            named.value.as_ref().and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::StringValue(
                "collecty-tenant".to_string()
            ))
        );
        assert!(
            attributes
                .iter()
                .any(|attribute| attribute.key == "service.name"),
            "the export still identifies itself as collecty"
        );
    }

    /// No tenant, no push. The summary on stderr still runs, so the numbers
    /// are not lost — only the export of them is.
    #[test]
    fn a_reporter_without_a_tenant_produces_no_export() {
        let scratch = crate::test_support::Scratch::new("observe-no-tenant");
        let queue = Arc::new(
            Queue::open(
                scratch.path(),
                crate::queue::QueueLimits::default(),
                crate::wire::ZSTD_LEVEL,
            )
            .expect("a queue"),
        );
        let spool = Spool::new(queue.clone());
        let reporter = Reporter::new(
            queue,
            Arc::new(SenderStats::default()),
            spool,
            None,
            Arc::new(SourceStats::default()),
        );
        assert!(reporter.export(&reporter.observe()).is_none());
    }

    #[test]
    fn a_gauge_carries_the_value_it_was_given_and_a_counter_is_monotonic() {
        let observed = Observation {
            queued_bytes: 4096,
            sent_segments: 7,
            ..Observation::default()
        };
        let export = ExportMetricsServiceRequest::decode(
            encode(&observed, 5, 9, "collecty-tenant").as_slice(),
        )
        .expect("a decodable export");
        let scope = &export.resource_metrics[0].scope_metrics[0];

        let queued = scope
            .metrics
            .iter()
            .find(|metric| metric.name == "collecty_queue_bytes")
            .expect("the queue gauge");
        let metric::Data::Gauge(gauge) = queued.data.as_ref().expect("data") else {
            panic!("collecty_queue_bytes must be a gauge");
        };
        assert_eq!(
            gauge.data_points[0].value,
            Some(number_data_point::Value::AsInt(4096))
        );

        let sent = scope
            .metrics
            .iter()
            .find(|metric| metric.name == "collecty_segments_sent_total")
            .expect("the sent counter");
        let metric::Data::Sum(sum) = sent.data.as_ref().expect("data") else {
            panic!("collecty_segments_sent_total must be a sum");
        };
        assert!(sum.is_monotonic);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Cumulative as i32
        );
        assert_eq!(
            sum.data_points[0].value,
            Some(number_data_point::Value::AsInt(7))
        );
    }

    #[test]
    fn the_export_names_collecty_as_the_service() {
        let export = ExportMetricsServiceRequest::decode(
            encode(&Observation::default(), 0, 0, "collecty-tenant").as_slice(),
        )
        .expect("an export");
        let resource = export.resource_metrics[0]
            .resource
            .as_ref()
            .expect("a resource");
        assert!(
            resource
                .attributes
                .iter()
                .any(|attribute| attribute.key == "service.name")
        );
    }
}
