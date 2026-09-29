use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use axum::http::StatusCode;

use crate::backpressure::IngestError;
use crate::config::Config;
use crate::metrics::RuntimeMetrics;
use crate::tenant::TenantId;
use crate::tenant_policy::{TenantPolicy, TenantStorageLimit};

/// Enforces the per-tenant limits the control plane pushed.
///
/// The limits are *stocks* — stored bytes — plus one bound on concurrent
/// queries. There are no per-tenant rates: how fast the whole instance
/// accepts work is the global backpressure gate's question, answered from the
/// server's own state rather than from a number sold per tenant.
pub struct TenantQuota {
    config: Arc<Config>,
    metrics: Arc<RuntimeMetrics>,
    policy: Arc<TenantPolicy>,
    /// Held rather than passed in per call, so a storage limit cannot be
    /// enforced on one ingest path and forgotten on another — which is the
    /// failure the two transports were split apart to prevent.
    parts: Arc<crate::part_registry::PartRegistry>,
    trace_parts: Arc<crate::trace_registry::TraceRegistry>,
    series_parts: Arc<crate::series_registry::SeriesRegistry>,
    /// Queries a tenant currently has in flight.
    ///
    /// Without it one tenant issuing many concurrent scans takes every permit
    /// of the shared query semaphore, and the other tenants queue behind it
    /// however small their queries are.
    in_flight: Mutex<HashMap<TenantId, u32>>,
}

/// Releases a tenant's in-flight slot however the query ends, including a
/// cancelled future. A leaked slot is permanent: the tenant loses that much
/// concurrency for the life of the process.
pub struct QuerySlot {
    quota: Arc<TenantQuota>,
    tenant: TenantId,
}

impl std::fmt::Debug for QuerySlot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("QuerySlot")
            .field("tenant", &self.tenant)
            .finish()
    }
}

impl Drop for QuerySlot {
    fn drop(&mut self) {
        let mut in_flight = self.quota.in_flight.lock();
        if let Some(count) = in_flight.get_mut(&self.tenant) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                in_flight.remove(&self.tenant);
            }
        }
    }
}

/// What a client is told to wait after being refused for storage.
///
/// Not a computed time-to-clear: what frees the space is retention retiring
/// parts, whose timing depends on the tenant's own retention period and on when
/// a merge tick reaches those parts. Nothing here can turn that into a number.
/// It is a floor that keeps a refused client from spinning at its own retry
/// rate for something that will not change within a second.
const STORAGE_LIMIT_RETRY_AFTER: Duration = Duration::from_secs(60);

impl TenantQuota {
    pub fn new(
        config: Arc<Config>,
        metrics: Arc<RuntimeMetrics>,
        policy: Arc<TenantPolicy>,
        parts: Arc<crate::part_registry::PartRegistry>,
        trace_parts: Arc<crate::trace_registry::TraceRegistry>,
        series_parts: Arc<crate::series_registry::SeriesRegistry>,
    ) -> Self {
        Self {
            config,
            metrics,
            policy,
            parts,
            trace_parts,
            series_parts,
            in_flight: Mutex::new(HashMap::new()),
        }
    }

    /// A quota with no policy behind it, for tests exercising something else.
    /// Unlimited unless the configuration sets a default limit, which is the
    /// same answer production gives when the control plane is silent.
    #[cfg(test)]
    pub fn for_test(config: &Config) -> Arc<Self> {
        Arc::new(Self::new(
            Arc::new(config.clone()),
            Arc::new(RuntimeMetrics::new()),
            Arc::new(TenantPolicy::disabled()),
            Arc::new(crate::part_registry::PartRegistry::new()),
            Arc::new(crate::trace_registry::TraceRegistry::standalone()),
            Arc::new(crate::series_registry::SeriesRegistry::standalone()),
        ))
    }

    /// Whether the tenant may start another query, and the slot it holds while
    /// it runs.
    pub fn begin_query(self: &Arc<Self>, tenant: &TenantId) -> Result<QuerySlot, IngestError> {
        let limit = self.config.max_concurrent_queries_per_tenant as u32;
        let mut in_flight = self.in_flight.lock();
        let count = in_flight.entry(tenant.clone()).or_insert(0);
        if *count >= limit {
            let running = *count;
            drop(in_flight);
            return Err(self.query_refused(format!(
                "tenant {tenant} already has {running} queries in flight, at its limit of {limit}"
            )));
        }
        *count += 1;
        drop(in_flight);
        Ok(QuerySlot {
            quota: self.clone(),
            tenant: tenant.clone(),
        })
    }

    /// The tenant's storage limit in bytes, or `None` when it has none.
    pub fn max_stored_bytes_for(&self, tenant: &TenantId) -> Option<u64> {
        match self.resolve_storage_limit(tenant) {
            TenantStorageLimit::Unlimited => None,
            TenantStorageLimit::Bytes(bytes) => Some(bytes),
        }
    }

    /// The bytes a tenant has stored, every signal included.
    ///
    /// One function because two callers read it: `admit_storage` refuses on
    /// this number and the admin usage endpoint reports it. They were separate
    /// sums until 2026-08-27, and the reporting one was never taught about
    /// metric parts when M14 added them -- so a tenant was refused on a total
    /// larger than the one its control plane was showing it.
    pub fn tenant_stored_bytes(&self, tenant: &TenantId) -> u64 {
        self.parts
            .tenant_stored_bytes(tenant)
            .saturating_add(self.trace_parts.tenant_stored_bytes(tenant))
            .saturating_add(self.series_parts.tenant_stored_bytes(tenant))
    }

    fn resolve_storage_limit(&self, tenant: &TenantId) -> TenantStorageLimit {
        self.policy
            .max_stored_bytes(tenant)
            .unwrap_or(TenantStorageLimit::Unlimited)
    }

    /// Refuse the write when the tenant is already holding everything its plan
    /// sells it.
    ///
    /// Read from the registries' running census, so this costs the same on a
    /// tenant with one part as on one with ten thousand.
    ///
    /// The comparison is against what is *stored*, not against what this
    /// request would make stored. A request is at most the OTLP body limit and
    /// the storage limit is measured in gigabytes, so the overrun a "check
    /// before, not after" rule allows is one request deep — and checking the
    /// other way would mean estimating a body's compressed size before
    /// compressing it.
    pub fn admit_storage(&self, tenant: &TenantId) -> Result<(), IngestError> {
        let TenantStorageLimit::Bytes(limit) = self.resolve_storage_limit(tenant) else {
            return Ok(());
        };
        let stored = self.tenant_stored_bytes(tenant);
        if stored < limit {
            return Ok(());
        }
        self.metrics
            .storage_limit_rejected
            .fetch_add(1, Ordering::Relaxed);
        Err(IngestError {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: format!(
                "tenant {tenant} stores {stored} bytes, at its limit of {limit}; \
writes resume when retention retires enough of it"
            ),
            retry_after: Some(STORAGE_LIMIT_RETRY_AFTER),
        })
    }

    /// A read-side refusal, counted apart from the write-side one so an
    /// operator can tell which half of a tenant's quota is binding.
    fn query_refused(&self, message: String) -> IngestError {
        self.metrics
            .query_quota_rejected
            .fetch_add(1, Ordering::Relaxed);
        IngestError {
            status: StatusCode::TOO_MANY_REQUESTS,
            message,
            retry_after: Some(self.config.backpressure_retry_after),
        }
    }
}

#[cfg(test)]
mod tests {
    include!("tests/tenant_quota.rs");
}
