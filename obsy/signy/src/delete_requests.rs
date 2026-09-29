//! The Loki delete API: a tenant asks for a selector and a time range to be
//! removed, and this decides what that means here.
//!
//! Two things have to be true and they have different costs. The data must stop
//! being readable, which has to happen now — a deletion request is usually
//! somebody exercising a right, and "it will disappear within the hour" is not
//! an answer. And the bytes must actually go, which cannot happen now, because
//! parts are immutable and rewriting one to drop a few rows is the most
//! expensive operation this engine has.
//!
//! So a request does both, at the speed each can go. Every read path funnels
//! through one scan, and that scan drops the matching rows from the moment the
//! request is accepted. The bytes leave when the parts holding them are next
//! rewritten, which merge and retention already do, using the same per-row
//! predicate. `status` reports which of the two has happened.
//!
//! Not offered: Loki's cancellation grace period, where a request sits
//! unapplied for 24 hours so it can be withdrawn. Hiding immediately and
//! allowing the request to be withdrawn until the rewrite lands is the same
//! affordance without a window in which the data is still being served.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::logql;
use crate::memtable::{Labels, LogEntry};
use crate::tenant::TenantId;

/// Requests one tenant may hold. Each one is a predicate every scan pays for,
/// so this is a bound on that cost as much as it is on storage.
pub const MAX_DELETE_REQUESTS_PER_TENANT: usize = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeleteStatus {
    /// Accepted and hidden from every query. The bytes are still in parts.
    Received,
    /// Every part that held matching rows has been rewritten without them.
    Processed,
}

impl DeleteStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Processed => "processed",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub request_id: String,
    pub tenant: String,
    pub query: String,
    pub start_ns: i64,
    pub end_ns: i64,
    pub created_at_ns: i64,
    pub status: DeleteStatus,
}

impl DeleteRequest {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "request_id": self.request_id,
            "start_time": self.start_ns / 1_000_000_000,
            "end_time": self.end_ns / 1_000_000_000,
            "query": self.query,
            "status": self.status.as_str(),
            "created_at": self.created_at_ns / 1_000_000_000,
        })
    }
}

/// A request with its selector already parsed. Parsing on every row would put
/// a LogQL parse inside the scan loop.
#[derive(Clone)]
struct CompiledRequest {
    request: DeleteRequest,
    selector: logql::LogQuery,
}

impl CompiledRequest {
    fn covers(&self, metadata: &[(String, String)], timestamp_ns: i64, line: &str) -> bool {
        if timestamp_ns < self.request.start_ns || timestamp_ns >= self.request.end_ns {
            return false;
        }
        // Matchers select on the row's own attributes now that no stream
        // exists. The pairs arrive canonical (sorted, unique), so the map is a
        // straight collect; built only while delete requests are outstanding.
        let attributes: Labels = metadata.iter().cloned().collect();
        if !self
            .selector
            .matchers
            .iter()
            .all(|matcher| matcher.matches(&attributes))
        {
            return false;
        }
        self.selector
            .line_filters
            .iter()
            .all(|filter| filter.matches(line))
    }
}

impl CompiledRequest {
    /// Whether this part could still contain a row the request covers,
    /// decided from the tenant segment's time span alone — with no stream
    /// table there is nothing else `meta.json` can rule out, so the status
    /// lags conservatively rather than ever claiming `processed` early.
    fn may_be_held_by(&self, tenant: &TenantId, meta: &crate::part::PartMeta) -> bool {
        meta.tenants.iter().any(|segment| {
            &segment.tenant == tenant
                && segment.min_ts_ns < self.request.end_ns
                && segment.max_ts_ns >= self.request.start_ns
        })
    }
}

#[derive(Default)]
pub struct DeleteRequestMetrics {
    pub accepted: AtomicU64,
    pub rejected: AtomicU64,
    pub cancelled: AtomicU64,
    pub hidden_rows: AtomicU64,
}

/// The registry every scan consults and the admin endpoints mutate.
pub struct DeleteRequests {
    by_tenant: RwLock<HashMap<TenantId, Vec<CompiledRequest>>>,
    /// Checked before the lock, so a scan pays nothing at all when no tenant
    /// has ever asked for a deletion — which is the normal case.
    outstanding: AtomicU64,
    storage: Option<Arc<crate::object_storage::ObjectStorage>>,
    pub metrics: DeleteRequestMetrics,
}

/// Why a submission was refused. Separated from a message so the handler can
/// choose a status code without matching on prose.
#[derive(Debug)]
pub enum DeleteRequestError {
    Invalid(String),
    TooMany,
    NotFound,
    Storage(String),
}

impl DeleteRequests {
    pub fn new(storage: Option<Arc<crate::object_storage::ObjectStorage>>) -> Self {
        Self {
            by_tenant: RwLock::new(HashMap::new()),
            outstanding: AtomicU64::new(0),
            storage,
            metrics: DeleteRequestMetrics::default(),
        }
    }

    /// True when no request exists anywhere, which lets a scan skip the lock.
    pub fn is_empty(&self) -> bool {
        self.outstanding.load(Ordering::Acquire) == 0
    }

    /// Loads what previous runs accepted. A failure is fatal to startup: coming
    /// up with a subset would serve data a tenant asked to have deleted.
    pub async fn load(&self) -> Result<usize, String> {
        let Some(storage) = &self.storage else {
            return Ok(0);
        };
        let bodies = storage.load_delete_requests().await?;
        let mut by_tenant = self.by_tenant.write();
        let mut loaded = 0usize;
        for body in bodies {
            let request: DeleteRequest = serde_json::from_slice(&body)
                .map_err(|error| format!("invalid stored delete request: {error}"))?;
            let tenant = TenantId::parse(&request.tenant).map_err(|error| {
                format!(
                    "stored delete request {} names an invalid tenant: {error}",
                    request.request_id
                )
            })?;
            let selector = parse_selector(&request.query)
                .map_err(|error| format!("stored delete request is unusable: {error}"))?;
            by_tenant
                .entry(tenant)
                .or_default()
                .push(CompiledRequest { request, selector });
            loaded += 1;
        }
        self.outstanding.store(loaded as u64, Ordering::Release);
        Ok(loaded)
    }

    pub async fn submit(
        &self,
        tenant: &TenantId,
        query: &str,
        start_ns: i64,
        end_ns: i64,
        now_ns: i64,
    ) -> Result<DeleteRequest, DeleteRequestError> {
        if end_ns <= start_ns {
            return Err(DeleteRequestError::Invalid(
                "end must be after start".to_string(),
            ));
        }
        let selector = parse_selector(query).map_err(DeleteRequestError::Invalid)?;
        let request = DeleteRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            tenant: tenant.as_str().to_string(),
            query: query.to_string(),
            start_ns,
            end_ns,
            created_at_ns: now_ns,
            status: DeleteStatus::Received,
        };

        {
            let by_tenant = self.by_tenant.read();
            if by_tenant
                .get(tenant)
                .is_some_and(|requests| requests.len() >= MAX_DELETE_REQUESTS_PER_TENANT)
            {
                self.metrics.rejected.fetch_add(1, Ordering::Relaxed);
                return Err(DeleteRequestError::TooMany);
            }
        }

        // Durable before visible. A request that is hiding rows but would not
        // survive a restart is the one failure mode this must not have.
        if let Some(storage) = &self.storage {
            let body = serde_json::to_vec_pretty(&request)
                .map_err(|error| DeleteRequestError::Storage(error.to_string()))?;
            storage
                .put_delete_request(tenant.as_str(), &request.request_id, body)
                .await
                .map_err(DeleteRequestError::Storage)?;
        }

        let mut by_tenant = self.by_tenant.write();
        let requests = by_tenant.entry(tenant.clone()).or_default();
        if requests.len() >= MAX_DELETE_REQUESTS_PER_TENANT {
            self.metrics.rejected.fetch_add(1, Ordering::Relaxed);
            return Err(DeleteRequestError::TooMany);
        }
        requests.push(CompiledRequest {
            request: request.clone(),
            selector,
        });
        self.outstanding.fetch_add(1, Ordering::Release);
        self.metrics.accepted.fetch_add(1, Ordering::Relaxed);
        Ok(request)
    }

    pub fn list(&self, tenant: &TenantId) -> Vec<DeleteRequest> {
        self.by_tenant
            .read()
            .get(tenant)
            .map(|requests| {
                requests
                    .iter()
                    .map(|compiled| compiled.request.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Withdraws a request. The rows it was hiding become readable again, which
    /// is only true while its status is `received` — once the rewrite has run
    /// they are gone and there is nothing to withdraw.
    pub async fn cancel(
        &self,
        tenant: &TenantId,
        request_id: &str,
    ) -> Result<(), DeleteRequestError> {
        {
            let by_tenant = self.by_tenant.read();
            let Some(found) = by_tenant.get(tenant).and_then(|requests| {
                requests
                    .iter()
                    .find(|compiled| compiled.request.request_id == request_id)
            }) else {
                return Err(DeleteRequestError::NotFound);
            };
            if found.request.status == DeleteStatus::Processed {
                return Err(DeleteRequestError::Invalid(
                    "this request has already been applied and its data is gone".to_string(),
                ));
            }
        }
        if let Some(storage) = &self.storage {
            storage
                .remove_delete_request(tenant.as_str(), request_id)
                .await
                .map_err(DeleteRequestError::Storage)?;
        }
        let mut by_tenant = self.by_tenant.write();
        if let Some(requests) = by_tenant.get_mut(tenant) {
            let before = requests.len();
            requests.retain(|compiled| compiled.request.request_id != request_id);
            let removed = before - requests.len();
            self.outstanding
                .fetch_sub(removed as u64, Ordering::Release);
            self.metrics
                .cancelled
                .fetch_add(removed as u64, Ordering::Relaxed);
        }
        Ok(())
    }

    /// The predicate the scan applies, as an owned snapshot so a query holds no
    /// lock while it runs. Empty when nothing is outstanding, which is the
    /// normal case and costs one atomic load.
    pub fn mask_for(&self, tenant: &TenantId) -> DeleteMask {
        if self.is_empty() {
            return DeleteMask::default();
        }
        DeleteMask {
            requests: self
                .by_tenant
                .read()
                .get(tenant)
                .cloned()
                .unwrap_or_default(),
        }
    }

    /// Every tenant's requests at once, for the rewrite path.
    pub fn masks(&self) -> DeleteMasks {
        if self.is_empty() {
            return DeleteMasks::default();
        }
        DeleteMasks {
            by_tenant: self.by_tenant.read().clone(),
        }
    }

    /// Promotes a request to `processed` once no part could still hold a row it
    /// covers.
    ///
    /// Conservative in the safe direction: only the tenant segment's time
    /// span can rule a part out, so the status lags — a worse answer than it
    /// could be but never a wrong one. Claiming `processed` while bytes
    /// remain is the failure that matters.
    pub fn mark_processed(&self, metas: &[crate::part::PartMeta]) {
        if self.is_empty() {
            return;
        }
        let mut by_tenant = self.by_tenant.write();
        for (tenant, requests) in by_tenant.iter_mut() {
            for compiled in requests.iter_mut() {
                if compiled.request.status == DeleteStatus::Processed {
                    continue;
                }
                if !metas
                    .iter()
                    .any(|meta| compiled.may_be_held_by(tenant, meta))
                {
                    compiled.request.status = DeleteStatus::Processed;
                }
            }
        }
    }
}

/// The rows one tenant's outstanding requests remove, ready to apply per row.
#[derive(Clone, Default)]
pub struct DeleteMask {
    requests: Vec<CompiledRequest>,
}

impl DeleteMask {
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn hides(&self, entry: &LogEntry) -> bool {
        self.requests.iter().any(|compiled| {
            compiled.covers(&entry.structured_metadata, entry.timestamp_ns, &entry.line)
        })
    }
}

/// Every tenant's mask at once, for the rewrite that reads rows from more than
/// one of them. Taken as a snapshot per merge tick: a request accepted midway
/// through a rewrite is applied by the next one, and until then the scan is
/// already hiding its rows.
#[derive(Clone, Default)]
pub struct DeleteMasks {
    by_tenant: HashMap<TenantId, Vec<CompiledRequest>>,
}

impl DeleteMasks {
    pub fn is_empty(&self) -> bool {
        self.by_tenant.is_empty()
    }

    pub fn hides_row(
        &self,
        tenant: &TenantId,
        timestamp_ns: i64,
        line: &str,
        metadata: &[(String, String)],
    ) -> bool {
        self.by_tenant.get(tenant).is_some_and(|requests| {
            requests
                .iter()
                .any(|compiled| compiled.covers(metadata, timestamp_ns, line))
        })
    }

    /// Whether this part could still hold a row some request covers, decided
    /// from `meta.json` alone.
    ///
    /// Merge asks this to decide whether a part is worth rewriting. It is the
    /// same question `mark_processed` answers and it is conservative in the
    /// same direction: a false positive costs one rewrite that drops nothing,
    /// a false negative leaves deleted bytes on disk.
    pub fn may_cover_part(&self, meta: &crate::part::PartMeta) -> bool {
        self.by_tenant.iter().any(|(tenant, requests)| {
            requests
                .iter()
                .any(|compiled| compiled.may_be_held_by(tenant, meta))
        })
    }
}

/// A delete selector is the flat filter grammar's canonical form
/// (`crate::query::canonical_filter_query`): attr filters and line filters
/// name rows that exist. `parse=` is not in the grammar here — a value a
/// parser derives would make the deletion change meaning whenever the parser
/// does. A stored request from before the flat form fails this parse, and
/// that failure is fatal to startup by design: the fix is to delete the
/// stored request object and re-submit it in the flat form.
fn parse_selector(query: &str) -> Result<logql::LogQuery, String> {
    let params = crate::query::parse_filter_params(query, 0, crate::query::DELETE_FILTER_PARAMS)?;
    if params.query.matchers.is_empty() {
        return Err(
            "a delete query must name at least one attr filter, like attr=app=api".to_string(),
        );
    }
    Ok(params.query)
}
