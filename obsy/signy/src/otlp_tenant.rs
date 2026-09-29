use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::resource::v1::Resource;

use crate::tenant::TenantId;
use crate::tenant_policy::TenantPolicy;

/// The resource attribute that names the tenant.
///
/// One key for all three signals, because `Resource` is the one field
/// `ResourceLogs`, `ResourceSpans` and `ResourceMetrics` share, and it is
/// already a batch boundary. Nothing finer would do: an attribute on a record
/// or a span would split a request per row, and the scope is instrumentation
/// identity rather than ownership.
///
/// Not configurable. A key both ends have to agree on is a key that can be
/// disagreed about, and the failure is silent — every export from the side
/// that guessed wrong is dropped. Semantic conventions have no tenant key to
/// reuse, and reusing one that means something else (`service.namespace`,
/// `deployment.environment.name`) puts two meanings on one attribute.
pub const TENANT_ATTRIBUTE: &str = "tenant.id";

/// Why a resource could not be filed under a tenant.
///
/// Each one is a silent drop rather than a refusal, so an operator reading
/// `/metrics` is the only one who finds out — which is why they are counted
/// apart. They send you to three different places: an SDK that was never
/// configured, an SDK configured with a value this engine cannot store, and a
/// tenant the control plane has not onboarded.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DropReason {
    NoTenant,
    InvalidTenant,
    TenantNotServed,
}

/// What a split threw away, tallied for `/metrics`.
#[derive(Default, Clone, Copy, Debug)]
pub struct Dropped {
    pub no_tenant: u64,
    pub invalid_tenant: u64,
    pub tenant_not_served: u64,
}

impl Dropped {
    pub fn is_empty(&self) -> bool {
        self.no_tenant == 0 && self.invalid_tenant == 0 && self.tenant_not_served == 0
    }

    fn count(&mut self, reason: DropReason) {
        match reason {
            DropReason::NoTenant => self.no_tenant += 1,
            DropReason::InvalidTenant => self.invalid_tenant += 1,
            DropReason::TenantNotServed => self.tenant_not_served += 1,
        }
    }

    /// Publish the tally and say what it was, once per split rather than once
    /// per resource: a misconfigured exporter repeats the same mistake on
    /// every export it makes, and a log line each would bury everything else.
    pub fn record(&self, metrics: &crate::metrics::RuntimeMetrics, signal: &str) {
        use std::sync::atomic::Ordering::Relaxed;
        if self.is_empty() {
            return;
        }
        metrics
            .ingest_dropped_no_tenant
            .fetch_add(self.no_tenant, Relaxed);
        metrics
            .ingest_dropped_invalid_tenant
            .fetch_add(self.invalid_tenant, Relaxed);
        metrics
            .ingest_dropped_tenant_not_served
            .fetch_add(self.tenant_not_served, Relaxed);
        let tenant_not_served_by_signal = match signal {
            "logs" => &metrics.ingest_dropped_tenant_not_served_logs,
            "traces" => &metrics.ingest_dropped_tenant_not_served_traces,
            "metrics" => &metrics.ingest_dropped_tenant_not_served_metrics,
            _ => return,
        };
        tenant_not_served_by_signal.fetch_add(self.tenant_not_served, Relaxed);
        tracing::warn!(
            signal,
            attribute = TENANT_ATTRIBUTE,
            no_tenant = self.no_tenant,
            invalid_tenant = self.invalid_tenant,
            tenant_not_served = self.tenant_not_served,
            "dropped resources that name no tenant this instance serves"
        );
    }
}

/// A request's resources, grouped by the tenant they name.
pub struct Split<T> {
    /// One request per tenant, in the order the tenants were first seen.
    pub groups: Vec<(TenantId, T)>,
    pub dropped: Dropped,
}

impl<T> Split<T> {
    /// Whether the bytes that arrived are still an exact encoding of what will
    /// be stored, so a transport holding them can pass them through instead of
    /// re-encoding. One group and nothing thrown away is the ordinary case:
    /// an export comes from one process, and one process is one tenant.
    pub fn is_intact(&self) -> bool {
        self.groups.len() == 1 && self.dropped.is_empty()
    }
}

/// Read the tenant off one resource.
///
/// A missing `Resource` and a `Resource` without the attribute are the same
/// thing — nobody said who this belongs to. A value that is not a string is
/// invalid rather than absent: something set the key, and reading an integer
/// as "unset" would hide that.
fn tenant_of(resource: Option<&Resource>) -> Result<TenantId, DropReason> {
    let attributes = match resource {
        Some(resource) => &resource.attributes,
        None => return Err(DropReason::NoTenant),
    };
    let value = attributes
        .iter()
        .find(|attribute| attribute.key == TENANT_ATTRIBUTE)
        .and_then(|attribute| attribute.value.as_ref())
        .ok_or(DropReason::NoTenant)?;
    match &value.value {
        Some(opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(raw)) => {
            TenantId::parse(raw).map_err(|_| DropReason::InvalidTenant)
        }
        _ => Err(DropReason::InvalidTenant),
    }
}

/// Group resources by tenant, dropping the ones that name none this instance
/// serves.
///
/// Order is first-seen rather than sorted, so the request a group is rebuilt
/// into holds its resources in the order they arrived and two runs over the
/// same body produce the same grouping. The search is linear because the
/// tenant count in one export is one in every deployment that exists today,
/// and a handful in the one that does not.
fn group_by_tenant<R>(
    resources: Vec<R>,
    resource_of: impl Fn(&R) -> Option<&Resource>,
    policy: &TenantPolicy,
) -> (Vec<(TenantId, Vec<R>)>, Dropped) {
    let mut groups: Vec<(TenantId, Vec<R>)> = Vec::new();
    let mut dropped = Dropped::default();
    for entry in resources {
        let tenant = match tenant_of(resource_of(&entry)) {
            Ok(tenant) if policy.is_tenant_allowed(&tenant) => tenant,
            Ok(_) => {
                dropped.count(DropReason::TenantNotServed);
                continue;
            }
            Err(reason) => {
                dropped.count(reason);
                continue;
            }
        };
        match groups.iter_mut().find(|(seen, _)| *seen == tenant) {
            Some((_, entries)) => entries.push(entry),
            None => groups.push((tenant, vec![entry])),
        }
    }
    (groups, dropped)
}

pub fn split_logs(
    request: ExportLogsServiceRequest,
    policy: &TenantPolicy,
) -> Split<ExportLogsServiceRequest> {
    let (groups, dropped) = group_by_tenant(
        request.resource_logs,
        |entry| entry.resource.as_ref(),
        policy,
    );
    Split {
        groups: groups
            .into_iter()
            .map(|(tenant, resource_logs)| (tenant, ExportLogsServiceRequest { resource_logs }))
            .collect(),
        dropped,
    }
}

pub fn split_traces(
    request: ExportTraceServiceRequest,
    policy: &TenantPolicy,
) -> Split<ExportTraceServiceRequest> {
    let (groups, dropped) = group_by_tenant(
        request.resource_spans,
        |entry| entry.resource.as_ref(),
        policy,
    );
    Split {
        groups: groups
            .into_iter()
            .map(|(tenant, resource_spans)| (tenant, ExportTraceServiceRequest { resource_spans }))
            .collect(),
        dropped,
    }
}

pub fn split_metrics(
    request: ExportMetricsServiceRequest,
    policy: &TenantPolicy,
) -> Split<ExportMetricsServiceRequest> {
    let (groups, dropped) = group_by_tenant(
        request.resource_metrics,
        |entry| entry.resource.as_ref(),
        policy,
    );
    Split {
        groups: groups
            .into_iter()
            .map(|(tenant, resource_metrics)| {
                (tenant, ExportMetricsServiceRequest { resource_metrics })
            })
            .collect(),
        dropped,
    }
}

/// Whether an attribute key is the routing key, so storage can leave it out.
///
/// The tenant is how a row was filed, not something the row is about. Left in,
/// it would land in every log entry's structured metadata and on every span's
/// resource, where a query could select on it — a second, unenforced copy of
/// the isolation the `_tenant` column already provides.
pub fn is_tenant_attribute(key: &str) -> bool {
    key == TENANT_ATTRIBUTE
}

/// A `Resource` naming `tenant`, the way an exporter configured for it would
/// send one.
#[cfg(test)]
pub fn tenant_resource(tenant: &TenantId) -> Resource {
    Resource {
        attributes: vec![opentelemetry_proto::tonic::common::v1::KeyValue {
            key: TENANT_ATTRIBUTE.to_string(),
            value: Some(opentelemetry_proto::tonic::common::v1::AnyValue {
                value: Some(
                    opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                        tenant.as_str().to_string(),
                    ),
                ),
            }),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// A `Resource` naming [`crate::tenant::test_tenant`].
#[cfg(test)]
pub fn test_tenant_resource() -> Resource {
    tenant_resource(&crate::tenant::test_tenant())
}

#[cfg(test)]
mod tests {
    include!("tests/otlp_tenant.rs");
}
