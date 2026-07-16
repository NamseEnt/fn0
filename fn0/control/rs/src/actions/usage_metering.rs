//! Hourly per-project storage usage metering.
//!
//! Driven from `cron_on_tick`. Persists two kinds of records per project:
//! - `ProjectOperationsUsageDoc`: read/write operation counts per hourly
//!   window, from the Cloudflare GraphQL Analytics API (sampled estimates).
//!   The last two completed windows are re-polled every run because analytics
//!   ingestion lags; the upsert is idempotent on (project, window).
//! - `ProjectStorageSnapshotDoc`: exact stored bytes / object counts, from
//!   ListObjectsV2 over the shared static bucket and each per-project
//!   object-storage bucket.
//!
//! Cloudflare retains analytics for 31 days; these docs are the durable
//! record that outlives it. Reads have no user-facing quota (downloads are
//! documented as unlimited) — read counts are recorded for platform cost
//! monitoring and abuse detection, which is the only sensor available for
//! presigned-URL traffic that never touches fn0 infrastructure.

use crate::common::admin;
use crate::common::cloudflare::CloudflareClient;
use crate::common::cloudflare_analytics::CloudflareAnalyticsClient;
use crate::common::r2_store::{OBJECT_STORAGE_BUCKET_PREFIX, ObjectStorageStore, StaticAssetStore};
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

// Analytics ingestion lag straddles window boundaries; re-polling one extra
// window self-heals late-arriving samples.
const REPOLLED_WINDOWS: i64 = 2;

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok {
        projects_count: u64,
        operations_docs_count: u64,
        snapshot_docs_count: u64,
    },
    Unauthorized,
    Error {
        message: String,
    },
}

#[derive(Default)]
pub struct MeteringStats {
    pub projects: u64,
    pub operations_docs: u64,
    pub snapshot_docs: u64,
}

pub async fn run_metering() -> anyhow::Result<MeteringStats> {
    let db = doc_db::turso();
    let now = forte_sdk::now();
    let current_epoch_hour = now.timestamp() / 3600;

    let cloudflare = CloudflareClient::from_env()?;
    let analytics = CloudflareAnalyticsClient::from_env()?;
    let static_store = StaticAssetStore::from_env()?;

    let bucket_project_ids: BTreeSet<String> = cloudflare
        .list_r2_bucket_names(OBJECT_STORAGE_BUCKET_PREFIX)
        .await?
        .into_iter()
        .filter_map(|name| {
            name.strip_prefix(OBJECT_STORAGE_BUCKET_PREFIX)
                .map(str::to_string)
        })
        .collect();

    let static_usage_by_project = snapshot_static_bucket(&static_store, now).await?;

    let mut project_ids = bucket_project_ids.clone();
    project_ids.extend(static_usage_by_project.keys().cloned());

    let mut stats = MeteringStats {
        projects: project_ids.len() as u64,
        ..Default::default()
    };

    for project_id in &project_ids {
        let (static_asset_bytes, static_asset_object_count) = static_usage_by_project
            .get(project_id)
            .copied()
            .unwrap_or((0, 0));
        let (object_storage_bytes, object_storage_object_count) =
            if bucket_project_ids.contains(project_id) {
                snapshot_object_storage_bucket(project_id, now).await?
            } else {
                (0, 0)
            };
        ProjectStorageSnapshotDocPut(ProjectStorageSnapshotDoc {
            project_id: project_id.clone(),
            snapshot_epoch_hour: current_epoch_hour,
            static_asset_bytes,
            static_asset_object_count,
            object_storage_bytes,
            object_storage_object_count,
            taken_at: now,
        })
        .send_with(&db)
        .await?;
        stats.snapshot_docs += 1;
    }

    for window_epoch_hour in (current_epoch_hour - REPOLLED_WINDOWS)..current_epoch_hour {
        stats.operations_docs += poll_operations_window(
            &db,
            &analytics,
            &static_store,
            &project_ids,
            window_epoch_hour,
            now,
        )
        .await?;
    }

    Ok(stats)
}

async fn snapshot_static_bucket(
    static_store: &StaticAssetStore,
    now: DateTime,
) -> anyhow::Result<HashMap<String, (u64, u64)>> {
    let objects = static_store.list_all("", now).await?;
    let mut by_project: HashMap<String, (u64, u64)> = HashMap::new();
    for object in objects {
        let Some((project_id, _)) = object.key.split_once('/') else {
            continue;
        };
        let entry = by_project.entry(project_id.to_string()).or_default();
        entry.0 += object.size;
        entry.1 += 1;
    }
    Ok(by_project)
}

async fn snapshot_object_storage_bucket(
    project_id: &str,
    now: DateTime,
) -> anyhow::Result<(u64, u64)> {
    let store = ObjectStorageStore::from_env(project_id)?;
    let objects = store.list_all("", now).await?;
    let bytes = objects.iter().map(|o| o.size).sum();
    Ok((bytes, objects.len() as u64))
}

async fn poll_operations_window(
    db: &doc_db::Database,
    analytics: &CloudflareAnalyticsClient,
    static_store: &StaticAssetStore,
    project_ids: &BTreeSet<String>,
    window_epoch_hour: i64,
    now: DateTime,
) -> anyhow::Result<u64> {
    let window_start = chrono::DateTime::from_timestamp(window_epoch_hour * 3600, 0)
        .ok_or_else(|| anyhow::anyhow!("window start out of range"))?;
    let window_end = chrono::DateTime::from_timestamp((window_epoch_hour + 1) * 3600, 0)
        .ok_or_else(|| anyhow::anyhow!("window end out of range"))?;

    struct ProjectOperations {
        static_asset_reads: SampledOperationCount,
        static_asset_writes: SampledOperationCount,
        object_storage_reads: SampledOperationCount,
        object_storage_writes: SampledOperationCount,
    }

    // A count no analytics row ever contributed to is an observed zero, which
    // is exact — hence `is_valid: true` on the fresh state, ANDed down by
    // whichever rows accumulate into it.
    impl Default for ProjectOperations {
        fn default() -> Self {
            let fresh = SampledOperationCount {
                is_valid: true,
                ..SampledOperationCount::default()
            };
            Self {
                static_asset_reads: fresh,
                static_asset_writes: fresh,
                object_storage_reads: fresh,
                object_storage_writes: fresh,
            }
        }
    }

    let mut by_project: HashMap<String, ProjectOperations> = HashMap::new();

    for row in analytics
        .r2_operations_by_bucket(window_start, window_end)
        .await?
    {
        let Some(project_id) = row.bucket_name.strip_prefix(OBJECT_STORAGE_BUCKET_PREFIX) else {
            continue;
        };
        let operations = by_project.entry(project_id.to_string()).or_default();
        match operation_class(&row.action_type) {
            Some(OperationClass::A) => accumulate(&mut operations.object_storage_writes, row.count),
            Some(OperationClass::B) => accumulate(&mut operations.object_storage_reads, row.count),
            None => {}
        }
    }

    for project_id in project_ids {
        let rows = analytics
            .r2_operations_for_key_prefix(
                static_store.bucket(),
                &format!("{project_id}/"),
                window_start,
                window_end,
            )
            .await?;
        if rows.is_empty() {
            continue;
        }
        let operations = by_project.entry(project_id.clone()).or_default();
        for row in rows {
            match operation_class(&row.action_type) {
                Some(OperationClass::A) => {
                    accumulate(&mut operations.static_asset_writes, row.count)
                }
                Some(OperationClass::B) => {
                    accumulate(&mut operations.static_asset_reads, row.count)
                }
                None => {}
            }
        }
    }

    let mut written = 0;
    for (project_id, operations) in by_project {
        let has_activity = [
            &operations.static_asset_reads,
            &operations.static_asset_writes,
            &operations.object_storage_reads,
            &operations.object_storage_writes,
        ]
        .iter()
        .any(|count| count.estimate > 0);
        if !has_activity {
            continue;
        }
        ProjectOperationsUsageDocPut(ProjectOperationsUsageDoc {
            project_id,
            window_start_epoch_hour: window_epoch_hour,
            static_asset_reads: operations.static_asset_reads,
            static_asset_writes: operations.static_asset_writes,
            object_storage_reads: operations.object_storage_reads,
            object_storage_writes: operations.object_storage_writes,
            polled_at: now,
        })
        .send_with(db)
        .await?;
        written += 1;
    }
    Ok(written)
}

enum OperationClass {
    A,
    B,
}

// R2 billing classes per https://developers.cloudflare.com/r2/pricing/.
// Class A = mutations/lists (charged higher), Class B = reads. Deletes and
// aborts are free and excluded. Unknown action types are skipped with a
// warning rather than guessed into a billable class.
fn operation_class(action_type: &str) -> Option<OperationClass> {
    match action_type {
        "ListBuckets"
        | "PutBucket"
        | "ListObjects"
        | "PutObject"
        | "CopyObject"
        | "CompleteMultipartUpload"
        | "CreateMultipartUpload"
        | "LifecycleStorageTierTransition"
        | "ListMultipartUploads"
        | "UploadPart"
        | "UploadPartCopy"
        | "ListParts"
        | "PutBucketEncryption"
        | "PutBucketCors"
        | "PutBucketLifecycleConfiguration" => Some(OperationClass::A),
        "HeadBucket"
        | "HeadObject"
        | "GetObject"
        | "UsageSummary"
        | "GetBucketEncryption"
        | "GetBucketLocation"
        | "GetBucketCors"
        | "GetBucketLifecycleConfiguration" => Some(OperationClass::B),
        "DeleteObject" | "DeleteBucket" | "AbortMultipartUpload" => None,
        other => {
            tracing::warn!(action_type = %other, "usage_metering: unclassified R2 action type");
            None
        }
    }
}

// Summing per-action-type bounds overstates the combined interval width
// (errors across groups partially cancel), so the stored bounds are
// conservative. `is_valid` is the AND across contributing rows.
fn accumulate(total: &mut SampledOperationCount, row: SampledOperationCount) {
    total.estimate += row.estimate;
    total.lower_bound += row.lower_bound;
    total.upper_bound += row.upper_bound;
    total.is_valid = total.is_valid && row.is_valid;
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }
    match run_metering().await {
        Ok(stats) => Output::Ok {
            projects_count: stats.projects,
            operations_docs_count: stats.operations_docs,
            snapshot_docs_count: stats.snapshot_docs,
        },
        Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
