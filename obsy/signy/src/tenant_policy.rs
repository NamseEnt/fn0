use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
#[cfg(test)]
use std::time::UNIX_EPOCH;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::object_storage::{ObjectStorage, TENANT_POLICY_PREFIX};
use crate::part::PartMeta;
use crate::tenant::TenantId;
use crate::trace_part::TracePartMeta;

const POLICY_FILE_SUFFIX: &str = ".json";
const POLICY_TEMP_SUFFIX: &str = ".json.tmp";

/// How long one tenant's data is kept.
///
/// A tenant the control plane has never pushed has no `TenantRetention` at
/// all, which is deliberately different from [`TenantRetention::Infinite`]:
/// both keep the data, but only the second one is an answer the control plane
/// actually gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TenantRetention {
    Finite(Duration),
    Infinite,
}

/// The kind of data a retention applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Logs,
    Traces,
    Metrics,
}

/// A retention the control plane set for one signal, kept with the string it
/// arrived as so a `GET` returns what was pushed.
#[derive(Clone)]
struct SignalRetention {
    retention: TenantRetention,
    raw: String,
}

/// Per-signal overrides of a tenant's `retention`. A signal with no override
/// uses the tenant's `retention`, which is why a signal cannot be kept longer
/// by saying nothing about it.
#[derive(Clone, Default)]
struct SignalRetentions {
    logs: Option<SignalRetention>,
    traces: Option<SignalRetention>,
    metrics: Option<SignalRetention>,
}

impl SignalRetentions {
    fn get(&self, signal: Signal) -> Option<&SignalRetention> {
        match signal {
            Signal::Logs => self.logs.as_ref(),
            Signal::Traces => self.traces.as_ref(),
            Signal::Metrics => self.metrics.as_ref(),
        }
    }

    fn raw(&self, signal: Signal) -> Option<String> {
        self.get(signal).map(|retention| retention.raw.clone())
    }
}

/// The per-signal retention strings of one push, as the control plane sent
/// them. `None` means the signal uses the tenant's `retention`.
#[derive(Clone, Copy, Default)]
pub struct SignalRetentionRequest<'a> {
    pub logs: Option<&'a str>,
    pub traces: Option<&'a str>,
    pub metrics: Option<&'a str>,
}

/// How much a tenant may keep stored on this instance.
///
/// A *stock*: nothing else bounds how much a tenant inside every other limit
/// can accumulate. Retention is the other half — it decides when bytes leave —
/// so a plan that sells a period and a size needs both, and this is the size.
///
/// Enforced by refusing writes rather than by deleting. A tenant over its limit
/// keeps every byte it already has and is told to stop sending; the bytes come
/// back on their own when retention retires the oldest parts. Deleting to make
/// room would be this engine choosing which of a customer's logs to destroy,
/// which is not a decision it has the standing to make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TenantStorageLimit {
    Unlimited,
    /// Zero means the tenant may not store anything, which is a suspended
    /// account that has not yet been deleted. It is spelled the same way
    /// `retention: "0"` is.
    Bytes(u64),
}

/// What the control plane pushed for one tenant.
///
/// The raw strings are kept as they arrived so a `GET` returns what was sent.
/// The control plane is the only authority here: nothing rewrites, caps or
/// otherwise reinterprets the values it pushed.
#[derive(Clone)]
struct PolicyEntry {
    revision: u64,
    retention: TenantRetention,
    raw: String,
    /// Bytes the tenant may keep stored, or `None` when the control plane has
    /// said nothing. Kept alongside the string it arrived as, so a `GET`
    /// returns what was pushed rather than a re-rendering of it.
    max_stored_bytes: Option<TenantStorageLimit>,
    raw_max_stored_bytes: Option<String>,
    signal_retentions: SignalRetentions,
    updated_at: SystemTime,
    project_managed: bool,
    revoked_fence: Option<u64>,
}

impl PolicyEntry {
    fn view(&self) -> PolicyView {
        PolicyView {
            revision: self.revision,
            retention: self.raw.clone(),
            max_stored_bytes: self.raw_max_stored_bytes.clone(),
            log_retention: self.signal_retentions.raw(Signal::Logs),
            trace_retention: self.signal_retentions.raw(Signal::Traces),
            metric_retention: self.signal_retentions.raw(Signal::Metrics),
            updated_at: self.updated_at,
        }
    }

    fn retention_for(&self, signal: Signal) -> TenantRetention {
        self.signal_retentions
            .get(signal)
            .map(|override_retention| override_retention.retention)
            .unwrap_or(self.retention)
    }
}

/// Whether a push at the same revision as the stored entry carries the exact
/// same policy body — the only case an equal revision is allowed to repeat.
fn same_policy_document(
    entry: &PolicyEntry,
    raw: &str,
    raw_max_stored_bytes: Option<&String>,
    signal_retentions: &SignalRetentions,
) -> bool {
    entry.raw == raw
        && entry.raw_max_stored_bytes.as_ref() == raw_max_stored_bytes
        && [Signal::Logs, Signal::Traces, Signal::Metrics]
            .into_iter()
            .all(|signal| entry.signal_retentions.raw(signal) == signal_retentions.raw(signal))
}

/// One tenant's policy as the admin endpoints report it.
#[derive(Debug)]
pub struct PolicyView {
    pub revision: u64,
    pub retention: String,
    pub max_stored_bytes: Option<String>,
    pub log_retention: Option<String>,
    pub trace_retention: Option<String>,
    pub metric_retention: Option<String>,
    pub updated_at: SystemTime,
}

/// Every policy this instance knows, read by queries, retention and merge.
pub struct PolicyMap {
    entries: BTreeMap<TenantId, PolicyEntry>,
}

impl PolicyMap {
    pub fn retention(&self, tenant: &TenantId) -> Option<TenantRetention> {
        self.entries.get(tenant).map(|entry| entry.retention)
    }

    pub fn signal_retention(&self, tenant: &TenantId, signal: Signal) -> Option<TenantRetention> {
        self.entries
            .get(tenant)
            .map(|entry| entry.retention_for(signal))
    }

    pub fn max_stored_bytes(&self, tenant: &TenantId) -> Option<TenantStorageLimit> {
        self.entries
            .get(tenant)
            .and_then(|entry| entry.max_stored_bytes)
    }

    pub fn contains(&self, tenant: &TenantId) -> bool {
        self.entries.contains_key(tenant)
    }

    /// Every tenant with a pushed policy, in name order.
    pub fn views(&self) -> impl Iterator<Item = (&TenantId, PolicyView)> {
        self.entries
            .iter()
            .map(|(tenant, entry)| (tenant, entry.view()))
    }

    pub fn view(&self, tenant: &TenantId) -> Option<PolicyView> {
        self.entries.get(tenant).map(PolicyEntry::view)
    }

    pub fn tenant_count(&self) -> usize {
        self.entries.len()
    }

    pub fn infinite_tenant_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| matches!(entry.retention, TenantRetention::Infinite))
            .count()
    }

    /// Age of the newest policy on this pod. It gates nothing; it tells an
    /// operator whether the control plane is still talking to this instance.
    /// Restored from the stored `updated_at`, so a restart does not reset it.
    ///
    /// Only meaningful once at least one policy exists: with an empty map there
    /// is no newest push and this reads zero, so an alert on it must be paired
    /// with `known_tenants > 0`. The "never pushed at all" state is the
    /// unknown-tenant gauge's to report, not this one's.
    pub fn newest_push_age(&self, now: SystemTime) -> Duration {
        self.entries
            .values()
            .map(|entry| entry.updated_at)
            .max()
            .map(|newest| now.duration_since(newest).unwrap_or_default())
            .unwrap_or_default()
    }
}

/// Per-tenant cutoffs resolved against one instant.
///
/// Every decision below reads `meta.json` only: the tenant index already
/// carries each tenant's row-group range, row count and timestamp bounds, so
/// no object body is downloaded to decide what has expired.
#[derive(Clone)]
pub struct Cutoffs {
    policies: Arc<PolicyMap>,
    now_ns: i64,
}

impl Cutoffs {
    /// Oldest timestamp still retained for `tenant`, or `None` when nothing
    /// expires — an unknown tenant or an explicitly infinite one.
    pub fn cutoff_ns(&self, tenant: &TenantId, signal: Signal) -> Option<i64> {
        match self.policies.signal_retention(tenant, signal)? {
            TenantRetention::Infinite => None,
            TenantRetention::Finite(period) => Some(
                self.now_ns
                    .saturating_sub(period.as_nanos().min(i64::MAX as u128) as i64),
            ),
        }
    }

    pub fn is_expired(&self, tenant: &TenantId, signal: Signal, timestamp_ns: i64) -> bool {
        self.cutoff_ns(tenant, signal)
            .is_some_and(|cutoff_ns| timestamp_ns < cutoff_ns)
    }

    /// Whether every tenant in the part has expired, which is the free
    /// whole-part deletion case.
    pub fn log_part_fully_expired(&self, meta: &PartMeta) -> bool {
        !meta.tenants.is_empty()
            && meta
                .tenants
                .iter()
                .all(|segment| self.is_expired(&segment.tenant, Signal::Logs, segment.max_ts_ns))
    }

    /// Rows the part holds for tenants whose whole segment has expired.
    pub fn expired_log_rows(&self, meta: &PartMeta) -> u64 {
        meta.tenants
            .iter()
            .filter(|segment| self.is_expired(&segment.tenant, Signal::Logs, segment.max_ts_ns))
            .map(|segment| segment.row_count)
            .sum()
    }

    /// Expired share of a part's rows, used to decide whether reclaiming them
    /// is worth one rewrite.
    pub fn expired_log_row_fraction(&self, meta: &PartMeta) -> f64 {
        if meta.row_count == 0 {
            return 0.0;
        }
        self.expired_log_rows(meta) as f64 / meta.row_count as f64
    }

    /// Whether the part still holds rows for a tenant at zero retention.
    ///
    /// Zero retention is how a tenant is deleted, so that path ignores
    /// `retention_rewrite_threshold`: any part still holding the tenant's rows
    /// is rewritten, which turns "the rows may survive in a large part
    /// indefinitely" into "the next few merge ticks" without job tracking.
    pub fn holds_zero_retention_rows(&self, meta: &PartMeta) -> bool {
        meta.tenants
            .iter()
            .any(|segment| segment.row_count > 0 && self.is_zero_retention(&segment.tenant))
    }

    fn is_zero_retention(&self, tenant: &TenantId) -> bool {
        matches!(
            self.policies.signal_retention(tenant, Signal::Logs),
            Some(TenantRetention::Finite(period)) if period.is_zero()
        )
    }

    /// Trace tenant segments carry no timestamps of their own, so a segment's
    /// bound comes from the row groups it owns.
    pub fn trace_part_fully_expired(&self, meta: &TracePartMeta) -> bool {
        if meta.tenants.is_empty() {
            return false;
        }
        meta.tenants.iter().all(|segment| {
            let groups = segment.row_group_start as usize..segment.row_group_end as usize;
            let segment_max = meta
                .row_group_max_ts
                .get(groups)
                .and_then(|bounds| bounds.iter().max().copied())
                .unwrap_or(meta.max_ts_ns);
            self.is_expired(&segment.tenant, Signal::Traces, segment_max)
        })
    }

    /// The metric part's whole-delete predicate. Metric tenant segments carry
    /// no per-segment time bounds, so every tenant is judged against the
    /// part's own `max_ts_ns` — conservative in exactly the trace predicate's
    /// direction: a part lives until its newest sample has expired for every
    /// tenant holding series in it.
    pub fn metric_part_fully_expired(&self, meta: &crate::series_part::SeriesPartMeta) -> bool {
        !meta.tenants.is_empty()
            && meta
                .tenants
                .iter()
                .all(|segment| self.is_expired(&segment.tenant, Signal::Metrics, meta.max_ts_ns))
    }
}

#[derive(Default)]
pub struct TenantPolicyMetrics {
    pub push_accepted: AtomicU64,
    /// A policy change the instance refused to make: a malformed body, an
    /// unparseable retention value, or a tenant id that is not one. Counts
    /// only requests that meant to change something, so a bad `GET` does not
    /// look like a control plane pushing garbage.
    pub push_rejected: AtomicU64,
    pub push_persist_errors: AtomicU64,
}

/// Why a policy change did not take effect.
#[derive(Debug)]
pub enum PolicyError {
    /// The request is malformed. Nothing was stored, so the control plane must
    /// fix the request rather than retry it.
    Invalid(String),
    /// The policy could not be made durable. Nothing was applied, and the
    /// control plane owns the retry.
    Persist(String),
    RevisionConflict {
        revision: u64,
    },
    ProjectManaged,
    AccessRevoked,
    FenceConflict {
        fence: u64,
    },
}

#[derive(Debug)]
pub enum PushResult {
    Applied(PolicyView),
    Duplicate(PolicyView),
    Stale(PolicyView),
}

/// Where the policy objects live.
///
/// One object per tenant, so a push is a single blind write: no
/// read-modify-write, no CAS, and no contention between two tenants pushed at
/// the same time.
enum PolicyStore {
    Remote(Arc<ObjectStorage>),
    /// With no object store configured the same files live under the data
    /// directory, written temp-file-then-rename like the rest of local state.
    Local(PathBuf),
}

#[derive(Serialize, Deserialize)]
struct PolicyDocument {
    #[serde(default)]
    revision: u64,
    retention: String,
    /// Absent in every policy stored before limits existed. `default` rather
    /// than required, because a stored policy that suddenly fails to
    /// deserialize is a fatal boot — the same trap `meta.json` fell into
    /// before it had a version field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_stored_bytes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    log_retention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trace_retention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metric_retention: Option<String>,
    #[serde(default)]
    project_managed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    revoked_fence: Option<u64>,
    updated_at: String,
}

impl PolicyStore {
    async fn put(
        &self,
        tenant: &TenantId,
        document: &PolicyDocument,
        revision: u64,
    ) -> Result<(), String> {
        let body = serde_json::to_vec(document)
            .map_err(|error| format!("failed to encode the tenant policy: {error}"))?;
        match self {
            Self::Remote(storage) => {
                storage
                    .put_tenant_policy_revision(tenant.as_str(), body, revision)
                    .await
            }
            Self::Local(root) => {
                let root = root.clone();
                let file_name = format!("{tenant}{POLICY_FILE_SUFFIX}");
                let temp_name = format!("{tenant}{POLICY_TEMP_SUFFIX}");
                tokio::task::spawn_blocking(move || {
                    write_local_policy(&root, &file_name, &temp_name, &body)
                })
                .await
                .map_err(|error| format!("tenant policy write task failed: {error}"))?
            }
        }
    }

    async fn delete(&self, tenant: &TenantId) -> Result<(), String> {
        match self {
            Self::Remote(storage) => storage.delete_tenant_policy(tenant.as_str()).await,
            Self::Local(root) => {
                let path = root.join(format!("{tenant}{POLICY_FILE_SUFFIX}"));
                let root = root.clone();
                tokio::task::spawn_blocking(move || match std::fs::remove_file(&path) {
                    Ok(()) => crate::part::fsync_dir(&root).map_err(|error| error.to_string()),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(error) => Err(format!(
                        "failed to delete the tenant policy {}: {error}",
                        path.display()
                    )),
                })
                .await
                .map_err(|error| format!("tenant policy delete task failed: {error}"))?
            }
        }
    }

    /// Every stored policy. A failure is fatal at boot, so this never returns a
    /// partial map: booting with a silently empty one would unclamp every query
    /// and hand back data a downgrade had already hidden.
    async fn load_all(&self) -> Result<BTreeMap<TenantId, PolicyEntry>, String> {
        let documents = match self {
            Self::Remote(storage) => storage.load_tenant_policies().await?,
            Self::Local(root) => {
                let root = root.clone();
                tokio::task::spawn_blocking(move || read_local_policies(&root))
                    .await
                    .map_err(|error| format!("tenant policy load task failed: {error}"))??
            }
        };
        let mut entries = BTreeMap::new();
        for (file_name, body) in documents {
            // A temp file is an expected crash leftover; anything else under
            // this prefix is unexplained, and guessing is exactly what the
            // fatal load exists to prevent.
            if file_name.ends_with(POLICY_TEMP_SUFFIX) {
                continue;
            }
            let raw_tenant = file_name.strip_suffix(POLICY_FILE_SUFFIX).ok_or_else(|| {
                format!("unexpected object {file_name:?} under {TENANT_POLICY_PREFIX}")
            })?;
            let tenant = TenantId::parse(raw_tenant)
                .map_err(|error| format!("invalid tenant policy file {file_name:?}: {error}"))?;
            let document: PolicyDocument = serde_json::from_slice(&body)
                .map_err(|error| format!("invalid tenant policy {file_name:?}: {error}"))?;
            let retention = parse_retention(&document.retention).map_err(|error| {
                format!(
                    "invalid retention {:?} in {file_name:?}: {error}",
                    document.retention
                )
            })?;
            let max_stored_bytes = document
                .max_stored_bytes
                .as_deref()
                .map(|raw| {
                    parse_storage_limit(raw).map_err(|error| {
                        format!("invalid max_stored_bytes {raw:?} in {file_name:?}: {error}")
                    })
                })
                .transpose()?;
            let signal_retentions = parse_signal_retentions(
                &retention,
                SignalRetentionRequest {
                    logs: document.log_retention.as_deref(),
                    traces: document.trace_retention.as_deref(),
                    metrics: document.metric_retention.as_deref(),
                },
            )
            .map_err(|error| format!("invalid signal retention in {file_name:?}: {error}"))?;
            let updated_at = parse_timestamp(&document.updated_at).map_err(|error| {
                format!(
                    "invalid updated_at {:?} in {file_name:?}: {error}",
                    document.updated_at
                )
            })?;
            entries.insert(
                tenant,
                PolicyEntry {
                    revision: document.revision,
                    retention,
                    raw: document.retention,
                    max_stored_bytes,
                    raw_max_stored_bytes: document.max_stored_bytes,
                    signal_retentions,
                    updated_at,
                    project_managed: document.project_managed,
                    revoked_fence: document.revoked_fence,
                },
            );
        }
        Ok(entries)
    }
}

fn write_local_policy(
    root: &Path,
    file_name: &str,
    temp_name: &str,
    body: &[u8],
) -> Result<(), String> {
    use std::io::Write;

    std::fs::create_dir_all(root).map_err(|error| {
        format!(
            "failed to create the tenant policy directory {}: {error}",
            root.display()
        )
    })?;
    let temporary = root.join(temp_name);
    let path = root.join(file_name);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(body).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    std::fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    crate::part::fsync_dir(root).map_err(|error| error.to_string())
}

fn read_local_policies(root: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        // Nothing has ever been pushed on this machine. Distinct from a read
        // failure: an empty policy set is a valid state, an unreadable one is
        // not.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "failed to read the tenant policy directory {}: {error}",
                root.display()
            ));
        }
    };
    let mut documents = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|name| format!("non-UTF-8 tenant policy file {name:?}"))?;
        let body = std::fs::read(entry.path())
            .map_err(|error| format!("failed to read tenant policy {file_name:?}: {error}"))?;
        documents.push((file_name, body));
    }
    Ok(documents)
}

/// The tenant→retention map, pushed one tenant at a time by the control plane.
///
/// signy never calls out: the control plane learns immediately whether its
/// change took effect, and owns the retry when it did not. Nothing here is on
/// the ingest hot path, and a tenant that was never pushed keeps its data
/// forever — signy never invents a deletion.
pub struct TenantPolicy {
    store: Option<PolicyStore>,
    policies: RwLock<Arc<PolicyMap>>,
    /// Serializes a write with the map swap that follows it. Two pushes for the
    /// same tenant are blind writes, so without this the store and the in-memory
    /// map could end up disagreeing about which one landed last.
    write_lock: tokio::sync::Mutex<()>,
    /// Stamps `updated_at` and resolves cutoffs. Injected because those two
    /// decide what a tenant may read: `query_floor_ns` clamps every read path
    /// to this clock, so a test that cannot move it cannot exercise a retention
    /// boundary except by writing data in the past and hoping.
    clock: Arc<crate::clock::Clock>,
    pub metrics: TenantPolicyMetrics,
}

impl TenantPolicy {
    /// A storeless fixture that serves every tenant and never expires
    /// anything. Production always loads a store ([`Self::load`]); this exists
    /// for tests exercising paths that are not about the registry.
    pub fn disabled() -> Self {
        Self::with_entries(None, BTreeMap::new())
    }

    fn with_entries(store: Option<PolicyStore>, entries: BTreeMap<TenantId, PolicyEntry>) -> Self {
        Self::with_entries_and_clock(store, entries, crate::clock::Clock::system())
    }

    fn with_entries_and_clock(
        store: Option<PolicyStore>,
        entries: BTreeMap<TenantId, PolicyEntry>,
        clock: Arc<crate::clock::Clock>,
    ) -> Self {
        Self {
            store,
            policies: RwLock::new(Arc::new(PolicyMap { entries })),
            write_lock: tokio::sync::Mutex::new(()),
            clock,
            metrics: TenantPolicyMetrics::default(),
        }
    }

    /// A policy whose clock the test controls, so retention boundaries can be
    /// crossed by moving time instead of by backdating data.
    #[cfg(test)]
    pub fn enabled_with_clock(clock: Arc<crate::clock::Clock>) -> Self {
        Self::with_entries_and_clock(
            Some(PolicyStore::Remote(Arc::new(ObjectStorage::in_memory()))),
            BTreeMap::new(),
            clock,
        )
    }

    #[cfg(test)]
    pub fn clock(&self) -> &Arc<crate::clock::Clock> {
        &self.clock
    }

    /// Load every stored policy before the workers start. A failure is fatal,
    /// the same class as a manifest that cannot be read.
    pub async fn load(
        config: &Config,
        object_storage: Option<Arc<ObjectStorage>>,
    ) -> Result<Self, String> {
        let store = match object_storage {
            Some(storage) => PolicyStore::Remote(storage),
            None => PolicyStore::Local(config.data_dir.join(TENANT_POLICY_PREFIX)),
        };
        let entries = store.load_all().await?;
        tracing::info!(
            tenants = entries.len(),
            "loaded per-tenant retention policies"
        );
        Ok(Self::with_entries(Some(store), entries))
    }

    pub fn is_enabled(&self) -> bool {
        self.store.is_some()
    }

    /// The current map, or `None` for the storeless test fixture
    /// ([`Self::disabled`]). `None` must always mean "delete nothing".
    pub fn snapshot(&self) -> Option<Arc<PolicyMap>> {
        self.is_enabled().then(|| self.policies.read().clone())
    }

    pub fn cutoffs_at(&self, now_ns: i64) -> Option<Cutoffs> {
        Some(Cutoffs {
            policies: self.snapshot()?,
            now_ns,
        })
    }

    pub fn cutoffs_now(&self) -> Option<Cutoffs> {
        self.cutoffs_at(self.clock.now_ns())
    }

    /// Oldest timestamp `tenant` may still read. `None` leaves the requested
    /// range untouched, which is the fail-open behaviour every read path wants
    /// for a tenant the control plane has said nothing about.
    pub fn query_floor_ns(&self, tenant: &TenantId, signal: Signal) -> Option<i64> {
        self.cutoffs_now()?.cutoff_ns(tenant, signal)
    }

    pub fn view(&self, tenant: &TenantId) -> Option<PolicyView> {
        self.snapshot()?.view(tenant)
    }

    /// Whether this instance serves `tenant`.
    ///
    /// The pushed policies *are* the tenant registry: a push onboards a tenant
    /// the moment it is durable, and a delete returns it to unknown, which
    /// every request path refuses.
    pub fn is_tenant_allowed(&self, tenant: &TenantId) -> bool {
        match self.snapshot() {
            Some(policies) => policies
                .entries
                .get(tenant)
                .is_some_and(|entry| entry.revoked_fence.is_none()),
            None => true,
        }
    }

    /// Store one tenant's retention, then apply it.
    ///
    /// The order is the whole point of the push shape: a success is a promise
    /// that the policy survives a restart, so the control plane's retry loop
    /// terminates on a real guarantee.
    /// A push carries the whole policy, so omitting `max_stored_bytes` clears
    /// it rather than leaving the previous value in place. Merging instead
    /// would make the stored state depend on the order of pushes, and a
    /// control plane retrying a push it believes to be complete would silently
    /// keep a limit it thinks it removed.
    pub async fn push(
        &self,
        tenant: &TenantId,
        revision: u64,
        raw: &str,
        raw_max_stored_bytes: Option<&str>,
    ) -> Result<PushResult, PolicyError> {
        self.push_with_signal_retentions(
            tenant,
            revision,
            raw,
            raw_max_stored_bytes,
            SignalRetentionRequest::default(),
        )
        .await
    }

    /// [`Self::push`] with per-signal overrides of `raw`. The body is still the
    /// whole policy: an override left out is cleared.
    ///
    /// fn0 is the source of truth for both the policy and its revision
    /// number; this only fences stale or conflicting writes against the
    /// revision it was given.
    pub async fn push_with_signal_retentions(
        &self,
        tenant: &TenantId,
        revision: u64,
        raw: &str,
        raw_max_stored_bytes: Option<&str>,
        raw_signal_retentions: SignalRetentionRequest<'_>,
    ) -> Result<PushResult, PolicyError> {
        self.push_with_signal_retentions_managed(
            tenant,
            revision,
            raw,
            raw_max_stored_bytes,
            raw_signal_retentions,
            false,
        )
        .await
    }

    pub async fn push_project_with_signal_retentions(
        &self,
        tenant: &TenantId,
        revision: u64,
        raw: &str,
        raw_max_stored_bytes: Option<&str>,
        raw_signal_retentions: SignalRetentionRequest<'_>,
    ) -> Result<PushResult, PolicyError> {
        self.push_with_signal_retentions_managed(
            tenant,
            revision,
            raw,
            raw_max_stored_bytes,
            raw_signal_retentions,
            true,
        )
        .await
    }

    async fn push_with_signal_retentions_managed(
        &self,
        tenant: &TenantId,
        revision: u64,
        raw: &str,
        raw_max_stored_bytes: Option<&str>,
        raw_signal_retentions: SignalRetentionRequest<'_>,
        project_managed: bool,
    ) -> Result<PushResult, PolicyError> {
        let Some(store) = &self.store else {
            return Err(PolicyError::Invalid(
                "per-tenant retention is not enabled".to_string(),
            ));
        };
        let retention = match parse_retention(raw) {
            Ok(retention) => retention,
            Err(error) => {
                self.metrics.push_rejected.fetch_add(1, Ordering::Relaxed);
                return Err(PolicyError::Invalid(error));
            }
        };
        let max_stored_bytes = match raw_max_stored_bytes.map(parse_storage_limit).transpose() {
            Ok(limit) => limit,
            Err(error) => {
                self.metrics.push_rejected.fetch_add(1, Ordering::Relaxed);
                return Err(PolicyError::Invalid(error));
            }
        };
        let signal_retentions = match parse_signal_retentions(&retention, raw_signal_retentions) {
            Ok(signal_retentions) => signal_retentions,
            Err(error) => {
                self.metrics.push_rejected.fetch_add(1, Ordering::Relaxed);
                return Err(PolicyError::Invalid(error));
            }
        };
        let raw = raw.trim().to_string();
        let raw_max_stored_bytes = raw_max_stored_bytes.map(|limit| limit.trim().to_string());

        // Stamped under the lock, not on arrival: two concurrent pushes for the
        // same tenant commit in lock order, and a timestamp taken before the
        // wait could let the one that commits last store the older `updated_at`
        // and walk the push age backwards.
        let _guard = self.write_lock.lock().await;
        if let Some(current) = self.policies.read().entries.get(tenant).cloned() {
            if current.revoked_fence.is_some() {
                return Err(PolicyError::AccessRevoked);
            }
            if !project_managed && current.project_managed {
                return Err(PolicyError::ProjectManaged);
            }
            match revision.cmp(&current.revision) {
                std::cmp::Ordering::Less => {
                    return Ok(PushResult::Stale(current.view()));
                }
                std::cmp::Ordering::Equal => {
                    if same_policy_document(
                        &current,
                        &raw,
                        raw_max_stored_bytes.as_ref(),
                        &signal_retentions,
                    ) && (!project_managed || current.project_managed)
                    {
                        return Ok(PushResult::Duplicate(current.view()));
                    }
                    if revision == current.revision
                        && (!same_policy_document(
                            &current,
                            &raw,
                            raw_max_stored_bytes.as_ref(),
                            &signal_retentions,
                        ) || !project_managed)
                    {
                        return Err(PolicyError::RevisionConflict { revision });
                    }
                }
                std::cmp::Ordering::Greater => {}
            }
        }
        let updated_at = self.clock.now();
        let document = PolicyDocument {
            revision,
            retention: raw.clone(),
            max_stored_bytes: raw_max_stored_bytes.clone(),
            log_retention: signal_retentions.raw(Signal::Logs),
            trace_retention: signal_retentions.raw(Signal::Traces),
            metric_retention: signal_retentions.raw(Signal::Metrics),
            project_managed,
            revoked_fence: None,
            updated_at: format_timestamp(updated_at),
        };
        if let Err(error) = store.put(tenant, &document, revision).await {
            self.metrics
                .push_persist_errors
                .fetch_add(1, Ordering::Relaxed);
            return Err(PolicyError::Persist(error));
        }
        let entry = PolicyEntry {
            revision,
            retention,
            raw,
            max_stored_bytes,
            raw_max_stored_bytes,
            signal_retentions,
            updated_at,
            project_managed,
            revoked_fence: None,
        };
        let view = entry.view();
        self.mutate(|entries| {
            entries.insert(tenant.clone(), entry);
        });
        self.metrics.push_accepted.fetch_add(1, Ordering::Relaxed);
        Ok(PushResult::Applied(view))
    }

    pub async fn revoke_project_access(
        &self,
        tenant: &TenantId,
        fence: u64,
    ) -> Result<(), PolicyError> {
        if fence == 0 {
            return Err(PolicyError::Invalid(
                "fence must be greater than zero".to_string(),
            ));
        }
        let Some(store) = &self.store else {
            return Err(PolicyError::Invalid(
                "per-tenant retention is not enabled".to_string(),
            ));
        };
        let _guard = self.write_lock.lock().await;
        let current = self.policies.read().entries.get(tenant).cloned();
        if let Some(current_fence) = current.as_ref().and_then(|entry| entry.revoked_fence) {
            if fence < current_fence {
                return Err(PolicyError::FenceConflict {
                    fence: current_fence,
                });
            }
            if fence == current_fence {
                return Ok(());
            }
        }
        let updated_at = self.clock.now();
        let (revision, retention, raw, max_stored_bytes, raw_max_stored_bytes, signal_retentions) =
            match current {
                Some(current) => (
                    current.revision,
                    current.retention,
                    current.raw,
                    current.max_stored_bytes,
                    current.raw_max_stored_bytes,
                    current.signal_retentions,
                ),
                None => (
                    0,
                    TenantRetention::Infinite,
                    "infinite".to_string(),
                    None,
                    None,
                    SignalRetentions::default(),
                ),
            };
        let document = PolicyDocument {
            revision,
            retention: raw.clone(),
            max_stored_bytes: raw_max_stored_bytes.clone(),
            log_retention: signal_retentions.raw(Signal::Logs),
            trace_retention: signal_retentions.raw(Signal::Traces),
            metric_retention: signal_retentions.raw(Signal::Metrics),
            project_managed: true,
            revoked_fence: Some(fence),
            updated_at: format_timestamp(updated_at),
        };
        if let Err(error) = store.put(tenant, &document, revision).await {
            self.metrics
                .push_persist_errors
                .fetch_add(1, Ordering::Relaxed);
            return Err(PolicyError::Persist(error));
        }
        let entry = PolicyEntry {
            revision,
            retention,
            raw,
            max_stored_bytes,
            raw_max_stored_bytes,
            signal_retentions,
            updated_at,
            project_managed: true,
            revoked_fence: Some(fence),
        };
        self.mutate(|entries| {
            entries.insert(tenant.clone(), entry);
        });
        self.metrics.push_accepted.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn max_stored_bytes(&self, tenant: &TenantId) -> Option<TenantStorageLimit> {
        self.snapshot()?.max_stored_bytes(tenant)
    }

    /// Return the tenant to *unknown*, which keeps its data forever. This is
    /// not tenant deletion; that is `retention: "0"`.
    pub async fn remove(&self, tenant: &TenantId) -> Result<(), PolicyError> {
        let Some(store) = &self.store else {
            return Err(PolicyError::Invalid(
                "per-tenant retention is not enabled".to_string(),
            ));
        };
        let _guard = self.write_lock.lock().await;
        if self
            .policies
            .read()
            .entries
            .get(tenant)
            .is_some_and(|entry| entry.project_managed)
        {
            return Err(PolicyError::ProjectManaged);
        }
        if let Err(error) = store.delete(tenant).await {
            self.metrics
                .push_persist_errors
                .fetch_add(1, Ordering::Relaxed);
            return Err(PolicyError::Persist(error));
        }
        self.mutate(|entries| {
            entries.remove(tenant);
        });
        self.metrics.push_accepted.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn record_rejected_push(&self) {
        self.metrics.push_rejected.fetch_add(1, Ordering::Relaxed);
    }

    /// Copy-insert-swap. Pushes are rare, so paying a map copy per push keeps
    /// every query read to one `Arc` clone.
    fn mutate(&self, change: impl FnOnce(&mut BTreeMap<TenantId, PolicyEntry>)) {
        let mut policies = self.policies.write();
        let mut entries = policies.entries.clone();
        change(&mut entries);
        *policies = Arc::new(PolicyMap { entries });
    }

    #[cfg(test)]
    pub fn install_for_test(&self, retentions: BTreeMap<TenantId, TenantRetention>) {
        self.mutate(|entries| {
            entries.clear();
            for (tenant, retention) in retentions {
                let raw = match retention {
                    TenantRetention::Infinite => "infinite".to_string(),
                    TenantRetention::Finite(period) => format!("{}ms", period.as_millis()),
                };
                entries.insert(
                    tenant,
                    PolicyEntry {
                        revision: 0,
                        retention,
                        raw,
                        max_stored_bytes: None,
                        raw_max_stored_bytes: None,
                        signal_retentions: SignalRetentions::default(),
                        updated_at: SystemTime::now(),
                        project_managed: false,
                        revoked_fence: None,
                    },
                );
            }
        });
    }

    #[cfg(test)]
    pub fn enabled_for_test() -> Self {
        Self::with_entries(
            Some(PolicyStore::Remote(Arc::new(ObjectStorage::in_memory()))),
            BTreeMap::new(),
        )
    }

    #[cfg(test)]
    pub fn for_test_with_store(store: Arc<ObjectStorage>) -> Self {
        Self::with_entries(Some(PolicyStore::Remote(store)), BTreeMap::new())
    }

    #[cfg(test)]
    pub fn for_test_with_local_store(root: PathBuf) -> Self {
        Self::with_entries(Some(PolicyStore::Local(root)), BTreeMap::new())
    }
}

/// Only tests reach for this now; production code reads the clock its
/// `TenantPolicy` or `AppState` was built with.
#[cfg(test)]
pub fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

fn format_timestamp(at: SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(at).to_rfc3339()
}

fn parse_timestamp(raw: &str) -> Result<SystemTime, String> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .map(|parsed| SystemTime::from(parsed.with_timezone(&chrono::Utc)))
        .map_err(|error| format!("{error}"))
}

/// Prometheus-style duration, or the literal `infinite`. Untrusted input.
pub fn parse_retention(raw: &str) -> Result<TenantRetention, String> {
    let value = raw.trim();
    if value.eq_ignore_ascii_case("infinite") {
        return Ok(TenantRetention::Infinite);
    }
    if value == "0" {
        // Deleting a tenant is `retention: "0"`: the cutoff lands at now, so
        // queries empty immediately and every part holding the tenant becomes
        // eligible for rewrite regardless of the rewrite threshold.
        return Ok(TenantRetention::Finite(Duration::ZERO));
    }
    let (number, unit_nanos) = if let Some(number) = value.strip_suffix("ms") {
        (number, 1_000_000u64)
    } else if let Some(number) = value.strip_suffix('s') {
        (number, 1_000_000_000u64)
    } else if let Some(number) = value.strip_suffix('m') {
        (number, 60 * 1_000_000_000u64)
    } else if let Some(number) = value.strip_suffix('h') {
        (number, 60 * 60 * 1_000_000_000u64)
    } else if let Some(number) = value.strip_suffix('d') {
        (number, 24 * 60 * 60 * 1_000_000_000u64)
    } else if let Some(number) = value.strip_suffix('w') {
        (number, 7 * 24 * 60 * 60 * 1_000_000_000u64)
    } else if let Some(number) = value.strip_suffix('y') {
        (number, 365 * 24 * 60 * 60 * 1_000_000_000u64)
    } else {
        return Err(
            "expected a duration such as 90m, 24h, 7d, or the literal \"infinite\"".to_string(),
        );
    };
    let number: u64 = number.trim().parse().map_err(|error| format!("{error}"))?;
    let nanos = number
        .checked_mul(unit_nanos)
        .ok_or_else(|| "duration overflow".to_string())?;
    Ok(TenantRetention::Finite(Duration::from_nanos(nanos)))
}

/// Per-signal overrides of `retention`.
///
/// `retention: "0"` is how a tenant is deleted, so it refuses an override that
/// would keep one of the tenant's signals alive.
fn parse_signal_retentions(
    retention: &TenantRetention,
    request: SignalRetentionRequest<'_>,
) -> Result<SignalRetentions, String> {
    let tenant_is_deleted =
        matches!(retention, TenantRetention::Finite(period) if period.is_zero());
    let parse = |name: &str, raw: Option<&str>| -> Result<Option<SignalRetention>, String> {
        let Some(raw) = raw else {
            return Ok(None);
        };
        let signal_retention = parse_retention(raw).map_err(|error| format!("{name}: {error}"))?;
        let keeps_data =
            !matches!(signal_retention, TenantRetention::Finite(period) if period.is_zero());
        if tenant_is_deleted && keeps_data {
            return Err(format!(
                "{name}: a tenant at retention \"0\" is being deleted, so no signal may keep data"
            ));
        }
        Ok(Some(SignalRetention {
            retention: signal_retention,
            raw: raw.trim().to_string(),
        }))
    };
    Ok(SignalRetentions {
        logs: parse("log_retention", request.logs)?,
        traces: parse("trace_retention", request.traces)?,
        metrics: parse("metric_retention", request.metrics)?,
    })
}

/// Parses a storage limit: a byte size such as `10GiB`, `0`, or the literal
/// `unlimited`. `"0"` means the tenant may not store anything, mirroring
/// `retention: "0"`.
pub fn parse_storage_limit(raw: &str) -> Result<TenantStorageLimit, String> {
    let value = raw.trim();
    if value.eq_ignore_ascii_case("unlimited") {
        return Ok(TenantStorageLimit::Unlimited);
    }
    if value.ends_with("/s") {
        return Err("a storage limit is a size, not a rate; drop the \"/s\"".to_string());
    }
    let (number, unit) = if let Some(number) = value.strip_suffix("KiB") {
        (number, 1024u64)
    } else if let Some(number) = value.strip_suffix("MiB") {
        (number, 1024 * 1024u64)
    } else if let Some(number) = value.strip_suffix("GiB") {
        (number, 1024 * 1024 * 1024u64)
    } else if let Some(number) = value.strip_suffix('B') {
        (number, 1u64)
    } else {
        (value, 1u64)
    };
    let number: u64 = number.trim().parse().map_err(|_| {
        "expected a byte size such as 512MiB, 10GiB, 0, or the literal \"unlimited\"".to_string()
    })?;
    let bytes = number
        .checked_mul(unit)
        .ok_or_else(|| "storage limit overflow".to_string())?;
    Ok(TenantStorageLimit::Bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant(raw: &str) -> TenantId {
        TenantId::parse(raw).expect("valid tenant id")
    }

    #[tokio::test]
    async fn a_signal_override_replaces_the_tenant_retention_for_that_signal_only() {
        let policy = TenantPolicy::enabled_for_test();
        policy
            .push_with_signal_retentions(
                &tenant("acme"),
                1,
                "30d",
                None,
                SignalRetentionRequest {
                    traces: Some("3d"),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let now_ns = 100 * 24 * 60 * 60 * 1_000_000_000i64;
        let cutoffs = policy.cutoffs_at(now_ns).unwrap();
        let four_days_ago_ns = now_ns - 4 * 24 * 60 * 60 * 1_000_000_000;

        assert!(cutoffs.is_expired(&tenant("acme"), Signal::Traces, four_days_ago_ns));
        assert!(!cutoffs.is_expired(&tenant("acme"), Signal::Logs, four_days_ago_ns));
        assert!(!cutoffs.is_expired(&tenant("acme"), Signal::Metrics, four_days_ago_ns));
        assert_eq!(
            policy
                .view(&tenant("acme"))
                .unwrap()
                .trace_retention
                .as_deref(),
            Some("3d")
        );
    }

    #[tokio::test]
    async fn signal_overrides_survive_a_restart_and_clear_when_omitted() {
        let store = Arc::new(ObjectStorage::in_memory());
        let policy = TenantPolicy::for_test_with_store(store.clone());
        policy
            .push_with_signal_retentions(
                &tenant("acme"),
                1,
                "30d",
                None,
                SignalRetentionRequest {
                    logs: Some("14d"),
                    traces: Some("3d"),
                    metrics: None,
                },
            )
            .await
            .unwrap();
        let entries = PolicyStore::Remote(store.clone()).load_all().await.unwrap();
        let restored = entries.get(&tenant("acme")).unwrap().view();
        assert_eq!(restored.log_retention.as_deref(), Some("14d"));
        assert_eq!(restored.trace_retention.as_deref(), Some("3d"));
        assert_eq!(restored.metric_retention, None);

        policy.push(&tenant("acme"), 2, "30d", None).await.unwrap();
        let entries = PolicyStore::Remote(store).load_all().await.unwrap();
        let cleared = entries.get(&tenant("acme")).unwrap().view();
        assert_eq!(cleared.log_retention, None);
        assert_eq!(cleared.trace_retention, None);
    }

    #[tokio::test]
    async fn a_deleted_tenant_cannot_keep_a_signal() {
        let policy = TenantPolicy::enabled_for_test();
        let refused = policy
            .push_with_signal_retentions(
                &tenant("acme"),
                1,
                "0",
                None,
                SignalRetentionRequest {
                    metrics: Some("30d"),
                    ..Default::default()
                },
            )
            .await;
        assert!(matches!(refused, Err(PolicyError::Invalid(_))));
        assert!(policy.view(&tenant("acme")).is_none());
    }

    fn days(count: u64) -> Duration {
        Duration::from_secs(count * 24 * 60 * 60)
    }

    #[test]
    fn parses_storage_limits_and_refuses_a_rate_spelling() {
        assert_eq!(
            parse_storage_limit("10GiB").unwrap(),
            TenantStorageLimit::Bytes(10 * 1024 * 1024 * 1024)
        );
        assert_eq!(
            parse_storage_limit(" 512MiB ").unwrap(),
            TenantStorageLimit::Bytes(512 * 1024 * 1024)
        );
        assert_eq!(
            parse_storage_limit("0").unwrap(),
            TenantStorageLimit::Bytes(0)
        );
        assert_eq!(
            parse_storage_limit("unlimited").unwrap(),
            TenantStorageLimit::Unlimited
        );
        // A size and a rate are different things, and "4MiB/s" of storage is a
        // control plane that filled in the wrong field. Refusing it is how the
        // mistake surfaces at the push rather than as a limit nobody set.
        assert!(parse_storage_limit("4MiB/s").is_err());
        assert!(parse_storage_limit("forever").is_err());
    }

    #[test]
    fn parses_prometheus_durations_and_the_infinite_literal() {
        assert_eq!(
            parse_retention("30d").unwrap(),
            TenantRetention::Finite(days(30))
        );
        assert_eq!(
            parse_retention("90m").unwrap(),
            TenantRetention::Finite(Duration::from_secs(90 * 60))
        );
        assert_eq!(
            parse_retention("infinite").unwrap(),
            TenantRetention::Infinite
        );
        assert_eq!(
            parse_retention("0").unwrap(),
            TenantRetention::Finite(Duration::ZERO)
        );
        assert!(parse_retention("7").is_err());
        assert!(parse_retention("forever").is_err());
        assert!(parse_retention("-1d").is_err());
    }

    #[test]
    fn an_unknown_tenant_never_expires() {
        let policy = TenantPolicy::enabled_for_test();
        assert_eq!(policy.query_floor_ns(&tenant("acme"), Signal::Logs), None);

        policy.install_for_test(
            [(
                tenant("acme"),
                TenantRetention::Finite(Duration::from_nanos(100)),
            )]
            .into_iter()
            .collect(),
        );
        let cutoffs = policy.cutoffs_at(1_000).unwrap();
        assert_eq!(cutoffs.cutoff_ns(&tenant("acme"), Signal::Logs), Some(900));
        assert!(cutoffs.is_expired(&tenant("acme"), Signal::Logs, 899));
        assert!(!cutoffs.is_expired(&tenant("acme"), Signal::Logs, 900));
        assert_eq!(cutoffs.cutoff_ns(&tenant("hobby"), Signal::Logs), None);
        assert!(!cutoffs.is_expired(&tenant("hobby"), Signal::Logs, i64::MIN));
    }

    #[test]
    fn a_disabled_policy_never_expires_anything() {
        let policy = TenantPolicy::disabled();
        assert!(!policy.is_enabled());
        assert!(policy.cutoffs_now().is_none());
        assert!(policy.snapshot().is_none());
        assert_eq!(policy.query_floor_ns(&tenant("acme"), Signal::Logs), None);
    }

    /// The pushed policies are the tenant registry: enabled and empty serves
    /// nobody, a push onboards, a remove offboards, and the switched-off
    /// policy has no registry at all.
    #[tokio::test]
    async fn the_registry_follows_pushes_and_removes() {
        let disabled = TenantPolicy::disabled();
        assert!(disabled.is_tenant_allowed(&tenant("anyone")));

        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = TenantPolicy::for_test_with_store(storage.clone());
        assert!(!policy.is_tenant_allowed(&tenant("acme")));

        policy.push(&tenant("acme"), 1, "30d", None).await.unwrap();
        assert!(policy.is_tenant_allowed(&tenant("acme")));
        assert!(!policy.is_tenant_allowed(&tenant("stranger")));

        // Onboarding survives a restart with the policy it rode in on.
        let config = Config::default();
        let restarted = TenantPolicy::load(&config, Some(storage)).await.unwrap();
        assert!(restarted.is_tenant_allowed(&tenant("acme")));

        policy.remove(&tenant("acme")).await.unwrap();
        assert!(!policy.is_tenant_allowed(&tenant("acme")));
    }

    #[tokio::test]
    async fn a_push_is_durable_and_survives_a_restart() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = TenantPolicy::for_test_with_store(storage.clone());
        policy.push(&tenant("acme"), 1, "30d", None).await.unwrap();
        policy
            .push(&tenant("intern"), 1, "infinite", None)
            .await
            .unwrap();
        // A downgrade taken before the restart must still be in force after it.
        policy.push(&tenant("acme"), 2, "7d", None).await.unwrap();
        assert_eq!(
            policy.metrics.push_accepted.load(Ordering::Relaxed),
            3,
            "every push is accepted"
        );

        let config = Config::default();
        let restarted = TenantPolicy::load(&config, Some(storage)).await.unwrap();
        let map = restarted.snapshot().unwrap();
        assert_eq!(map.tenant_count(), 2);
        assert_eq!(
            map.retention(&tenant("acme")),
            Some(TenantRetention::Finite(days(7))),
            "the newest policy wins, not the first one stored"
        );
        assert_eq!(
            map.retention(&tenant("intern")),
            Some(TenantRetention::Infinite)
        );
        assert_eq!(map.view(&tenant("acme")).unwrap().retention, "7d");
    }

    #[tokio::test]
    async fn revision_fence_accepts_newer_retries_and_rejects_older_payloads() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = TenantPolicy::for_test_with_store(storage);

        assert!(matches!(
            policy.push(&tenant("acme"), 1, "30d", None).await,
            Ok(PushResult::Applied(_))
        ));
        assert!(matches!(
            policy.push(&tenant("acme"), 2, "7d", None).await,
            Ok(PushResult::Applied(_))
        ));
        assert!(matches!(
            policy.push(&tenant("acme"), 1, "30d", None).await,
            Ok(PushResult::Stale(view)) if view.revision == 2
        ));
        assert!(matches!(
            policy.push(&tenant("acme"), 2, "7d", None).await,
            Ok(PushResult::Duplicate(view)) if view.revision == 2
        ));
        assert!(matches!(
            policy.push(&tenant("acme"), 2, "14d", None).await,
            Err(PolicyError::RevisionConflict { revision: 2 })
        ));
        assert_eq!(policy.view(&tenant("acme")).unwrap().revision, 2);
    }

    #[tokio::test]
    async fn concurrent_revisions_converge_on_the_newest_policy() {
        let policy = Arc::new(TenantPolicy::for_test_with_store(Arc::new(
            ObjectStorage::in_memory(),
        )));
        let older_policy = policy.clone();
        let newer_policy = policy.clone();
        let tenant_id = tenant("acme");
        let (older_result, newer_result) = tokio::join!(
            older_policy.push(&tenant_id, 10, "10d", None),
            newer_policy.push(&tenant_id, 11, "11d", None),
        );

        assert!(older_result.is_ok());
        assert!(newer_result.is_ok());
        let view = policy.view(&tenant("acme")).unwrap();
        assert_eq!(view.revision, 11);
        assert_eq!(view.retention, "11d");
    }

    #[tokio::test]
    async fn a_legacy_policy_loads_with_revision_zero() {
        let storage = Arc::new(ObjectStorage::in_memory());
        storage
            .put_tenant_policy_revision(
                "acme",
                br#"{"retention":"30d","updated_at":"2026-09-18T00:00:00Z"}"#.to_vec(),
                1,
            )
            .await
            .unwrap();
        let policy = TenantPolicy::load(&Config::default(), Some(storage))
            .await
            .unwrap();
        assert_eq!(policy.view(&tenant("acme")).unwrap().revision, 0);
    }

    #[test]
    fn zero_retention_expires_everything_and_forces_a_rewrite() {
        let policy = TenantPolicy::enabled_for_test();
        policy.install_for_test(
            [
                (tenant("acme"), TenantRetention::Finite(Duration::ZERO)),
                (tenant("beta"), TenantRetention::Infinite),
            ]
            .into_iter()
            .collect(),
        );
        let cutoffs = policy.cutoffs_at(1_000).unwrap();
        // The cutoff sits at now, so every query for the tenant empties from
        // the next request onward.
        assert_eq!(
            cutoffs.cutoff_ns(&tenant("acme"), Signal::Logs),
            Some(1_000)
        );
        assert!(cutoffs.is_expired(&tenant("acme"), Signal::Logs, 999));
        assert!(!cutoffs.is_expired(&tenant("beta"), Signal::Logs, i64::MIN));
        assert!(cutoffs.is_zero_retention(&tenant("acme")));
        assert!(!cutoffs.is_zero_retention(&tenant("beta")));
        assert!(!cutoffs.is_zero_retention(&tenant("hobby")));
    }

    #[tokio::test]
    async fn a_local_store_round_trips_without_an_object_store() {
        let data_dir = tempdir("tenant-policy-local");
        let dir = data_dir.join(TENANT_POLICY_PREFIX);
        let policy = TenantPolicy::for_test_with_local_store(dir.clone());
        policy.push(&tenant("acme"), 1, "7d", None).await.unwrap();
        assert_eq!(
            policy.snapshot().unwrap().retention(&tenant("acme")),
            Some(TenantRetention::Finite(days(7)))
        );

        let config = Config {
            data_dir: data_dir.clone(),
            ..Config::default()
        };
        let restarted = TenantPolicy::load(&config, None).await.unwrap();
        assert_eq!(
            restarted.snapshot().unwrap().retention(&tenant("acme")),
            Some(TenantRetention::Finite(days(7)))
        );

        // A crash leftover is expected and ignored; anything else is not.
        std::fs::write(dir.join("acme.json.tmp"), b"garbage").unwrap();
        assert!(TenantPolicy::load(&config, None).await.is_ok());
        std::fs::write(dir.join("acme.txt"), b"garbage").unwrap();
        assert!(TenantPolicy::load(&config, None).await.is_err());
        std::fs::remove_dir_all(&data_dir).unwrap();
    }

    #[tokio::test]
    async fn an_unreadable_policy_object_fails_the_load() {
        let storage = Arc::new(ObjectStorage::in_memory());
        storage
            .put_tenant_policy_revision("acme", b"{\"retention\":\"soon\"".to_vec(), 1)
            .await
            .unwrap();
        let config = Config::default();
        assert!(TenantPolicy::load(&config, Some(storage)).await.is_err());
    }

    /// What retention actually promises is a boundary in time, and the previous
    /// tests could only approach it by writing data with backdated timestamps
    /// and asserting the far side. That leaves the edge itself — the case a
    /// user notices — unexercised.
    ///
    /// With the clock injected the data stays put and time moves, which is the
    /// real situation, and the assertion can sit exactly on the boundary.
    #[tokio::test]
    async fn a_retention_boundary_is_crossed_by_time_passing() {
        let start_ns = 1_800_000_000_000_000_000i64;
        let clock = crate::clock::Clock::fixed(start_ns);
        let policy = TenantPolicy::enabled_with_clock(clock.clone());
        policy.push(&tenant("acme"), 1, "7d", None).await.unwrap();

        let seven_days = Duration::from_secs(7 * 24 * 60 * 60);
        let floor = || {
            policy
                .query_floor_ns(&tenant("acme"), Signal::Logs)
                .expect("finite")
        };

        // A row written now sits exactly at the floor's far side.
        assert_eq!(floor(), start_ns - seven_days.as_nanos() as i64);

        // One nanosecond before the boundary: still readable.
        clock.advance(seven_days);
        assert_eq!(floor(), start_ns);
        assert!(
            !policy
                .cutoffs_now()
                .unwrap()
                .is_expired(&tenant("acme"), Signal::Logs, start_ns),
            "a row exactly at the cutoff is retained"
        );

        // One nanosecond past it: gone.
        clock.advance(Duration::from_nanos(1));
        assert!(
            policy
                .cutoffs_now()
                .unwrap()
                .is_expired(&tenant("acme"), Signal::Logs, start_ns),
            "one nanosecond past the cutoff must expire"
        );
    }

    /// An upgrade applies at deletion time, not write time. Stated that way it
    /// is a claim about the clock, so moving the clock is how to check it.
    #[tokio::test]
    async fn an_upgrade_rescues_data_the_old_plan_had_already_passed() {
        let start_ns = 1_800_000_000_000_000_000i64;
        let clock = crate::clock::Clock::fixed(start_ns);
        let policy = TenantPolicy::enabled_with_clock(clock.clone());
        policy.push(&tenant("acme"), 1, "1d", None).await.unwrap();

        clock.advance(Duration::from_secs(2 * 24 * 60 * 60));
        assert!(
            policy
                .cutoffs_now()
                .unwrap()
                .is_expired(&tenant("acme"), Signal::Logs, start_ns),
            "two days in, a one-day plan has passed this row"
        );

        // The control plane upgrades the tenant. The row is still on disk, and
        // the new plan covers it again.
        policy.push(&tenant("acme"), 2, "30d", None).await.unwrap();
        assert!(
            !policy
                .cutoffs_now()
                .unwrap()
                .is_expired(&tenant("acme"), Signal::Logs, start_ns),
            "the upgrade must apply to data written under the old plan"
        );
    }

    #[tokio::test]
    async fn a_failing_store_changes_neither_the_map_nor_the_object() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = TenantPolicy::for_test_with_store(storage.clone());
        policy.push(&tenant("acme"), 1, "30d", None).await.unwrap();

        // A malformed value is rejected before anything is written.
        assert!(matches!(
            policy.push(&tenant("acme"), 2, "soon", None).await,
            Err(PolicyError::Invalid(_))
        ));
        assert_eq!(policy.metrics.push_rejected.load(Ordering::Relaxed), 1);
        assert_eq!(
            policy.snapshot().unwrap().retention(&tenant("acme")),
            Some(TenantRetention::Finite(days(30))),
            "a rejected push leaves the previous policy in force"
        );
        assert_eq!(storage.load_tenant_policies().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn a_delete_returns_the_tenant_to_unknown() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = TenantPolicy::for_test_with_store(storage.clone());
        policy.push(&tenant("acme"), 1, "1ms", None).await.unwrap();
        assert!(
            policy
                .query_floor_ns(&tenant("acme"), Signal::Logs)
                .is_some()
        );

        policy.remove(&tenant("acme")).await.unwrap();
        assert_eq!(
            policy.query_floor_ns(&tenant("acme"), Signal::Logs),
            None,
            "an unknown tenant is never clamped"
        );
        assert!(storage.load_tenant_policies().await.unwrap().is_empty());
        // Deleting an unknown tenant is a no-op, not an error.
        policy.remove(&tenant("hobby")).await.unwrap();
    }

    /// The control plane is the only authority on retention. A pushed value is
    /// applied exactly as sent, and the two ways of saying "keep forever" —
    /// an explicit `infinite` and never having been pushed — must produce the
    /// same retention, or asking for preservation would cost data that staying
    /// silent would have kept.
    #[tokio::test]
    async fn nothing_reinterprets_a_pushed_value() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = TenantPolicy::for_test_with_store(storage);
        policy
            .push(&tenant("acme"), 1, "3650d", None)
            .await
            .unwrap();
        policy
            .push(&tenant("intern"), 1, "infinite", None)
            .await
            .unwrap();

        let map = policy.snapshot().unwrap();
        assert_eq!(
            map.retention(&tenant("acme")),
            Some(TenantRetention::Finite(days(3650))),
            "a long finite plan is honoured as sent"
        );
        assert_eq!(map.view(&tenant("acme")).unwrap().retention, "3650d");
        assert_eq!(
            map.retention(&tenant("intern")),
            Some(TenantRetention::Infinite)
        );
        assert_eq!(map.infinite_tenant_count(), 1);

        let cutoffs = policy.cutoffs_at(1_000).unwrap();
        assert_eq!(
            cutoffs.cutoff_ns(&tenant("intern"), Signal::Logs),
            cutoffs.cutoff_ns(&tenant("never_pushed"), Signal::Logs),
            "an explicit infinite keeps exactly as much as never being pushed"
        );
    }

    fn tempdir(label: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("signy-{label}-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }
}
