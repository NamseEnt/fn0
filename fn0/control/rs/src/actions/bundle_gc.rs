//! Periodic GC for deploy artifacts.
//!
//! Driven from `cron_on_tick`:
//! - superseded versions: for each project, delete `CompiledBundleDoc` rows and
//!   their compiled/original R2 objects whose `code_version` is below the one
//!   the worker fleet currently serves, and delete the matching
//!   `{project_id}/{code_version}/` prefixes in the shared static asset bucket.
//! - abandoned uploads: delete `original/*` objects that never produced a
//!   `CompiledBundleDoc` once they are old enough, along with their static
//!   asset prefixes.

use crate::common::admin;
use crate::common::byoc::ProjectStorage;
use crate::common::r2_store::{BundleStore, ProjectR2Store, parse_compiled_key};
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

// Worker `manifest_poller` polls `WorkerManifestDoc` every 1s. A superseded
// `code_version` is only safe to delete once every worker has moved off the
// stale registry; gating on the *current latest* row's age by 60x the poll
// interval makes that window unreachable.
pub const STALE_REGISTRY_GRACE_SECS: i64 = 60;

// A superseded `{code_version}/` static prefix can still be fetched by a page
// that was served before the newer version went live and baked that prefix
// into its asset URLs (an open browser tab, cached HTML). Retaining the prefix
// for 24h after the newer version became latest outlasts any such page.
pub const STATIC_ASSET_GRACE_SECS: i64 = 24 * 60 * 60;

// An `original/*` upload with no `CompiledBundleDoc` is only abandoned if the
// compile is not merely slow. The cwasm-compiler Lambda is bounded far below
// this; 24h is a conservative floor that no in-flight compile reaches.
pub const ABANDONED_UPLOAD_TTL_SECS: i64 = 24 * 60 * 60;

const QUERY_PAGE_LIMIT: usize = 256;

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok {
        deleted_versions_count: u64,
        deleted_orphans_count: u64,
        deleted_static_prefixes_count: u64,
    },
    Unauthorized,
    Error {
        message: String,
    },
}

#[derive(Default)]
pub struct GcStats {
    pub deleted_versions: u64,
    pub deleted_orphans: u64,
    pub deleted_static_prefixes: u64,
}

pub async fn run_gc() -> anyhow::Result<GcStats> {
    let db = doc_db::turso();
    let bundle_store = BundleStore::from_env()?;
    let now = forte_sdk::now();

    let pending_fn0_wasmtime_version = (Fn0WasmtimeVersionDocGet {})
        .send_with(&db)
        .await?
        .and_then(|doc| doc.pending);

    let mut stats = GcStats::default();
    gc_superseded_versions(
        &db,
        &bundle_store,
        now,
        pending_fn0_wasmtime_version.as_deref(),
        &mut stats,
    )
    .await?;
    gc_abandoned_uploads(&db, &bundle_store, now, &mut stats).await?;

    let deleted = metrics::meter().u64_counter("fn0.gc.deleted").build();
    for (kind, count) in [
        ("compiled_versions", stats.deleted_versions),
        ("abandoned_uploads", stats.deleted_orphans),
        ("static_prefixes", stats.deleted_static_prefixes),
    ] {
        deleted.add(
            count,
            &[
                metrics::KeyValue::new("gc", "bundle_gc"),
                metrics::KeyValue::new("kind", kind),
            ],
        );
    }

    Ok(stats)
}

async fn gc_superseded_versions(
    db: &doc_db::Database,
    bundle_store: &BundleStore,
    now: DateTime,
    pending_fn0_wasmtime_version: Option<&str>,
    stats: &mut GcStats,
) -> anyhow::Result<()> {
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(db).await? else {
        return Ok(());
    };

    for (project_id, project_manifest) in &manifest.project_manifests {
        let latest = project_manifest.code_version;
        let docs = query_compiled_bundles(db, project_id).await?;

        let Some(latest_doc) = docs.iter().find(|doc| doc.code_version == latest) else {
            // The fleet serves a code_version with no CompiledBundleDoc. Without
            // the latest row's created_at there is no grace anchor, so skip the
            // whole project rather than risk deleting inside the stale window.
            continue;
        };
        let latest_age_secs = (now - latest_doc.created_at).num_seconds();

        if latest_age_secs >= STALE_REGISTRY_GRACE_SECS {
            for doc in &docs {
                if doc.code_version >= latest {
                    continue;
                }
                if let Some(pending) = pending_fn0_wasmtime_version
                    && doc.fn0_wasmtime_versions.iter().any(|v| v == pending)
                {
                    continue;
                }
                for fn0_wasmtime_version in &doc.fn0_wasmtime_versions {
                    let key = format!(
                        "compiled/{fn0_wasmtime_version}/{project_id}/{}.tar.zst",
                        doc.code_version
                    );
                    bundle_store.delete(&key, now).await?;
                }
                bundle_store
                    .delete(
                        &format!("original/{project_id}/{}.tar", doc.code_version),
                        now,
                    )
                    .await?;
                (CompiledBundleDocDelete {
                    project_id: project_id.as_str(),
                    code_version: doc.code_version,
                })
                .send_with(db)
                .await?;
                stats.deleted_versions += 1;
            }
        }

        if latest_age_secs >= STATIC_ASSET_GRACE_SECS {
            gc_superseded_frontend_assets(db, project_id, latest, now, stats).await?;
        }
    }
    Ok(())
}

async fn gc_superseded_frontend_assets(
    db: &doc_db::Database,
    project_id: &str,
    latest: u64,
    now: DateTime,
    stats: &mut GcStats,
) -> anyhow::Result<()> {
    let store = ProjectR2Store::frontend_assets(&ProjectStorage::resolve(db, project_id).await?);
    let objects = store.list_all("", now).await?;
    let mut keys_by_code_version: HashMap<u64, Vec<String>> = HashMap::new();
    for object in objects {
        let Some(code_version) = parse_frontend_asset_key(&object.key) else {
            continue;
        };
        keys_by_code_version
            .entry(code_version)
            .or_default()
            .push(object.key);
    }
    for (code_version, keys) in keys_by_code_version {
        if code_version >= latest {
            continue;
        }
        for key in keys {
            store.delete(&key, now).await?;
        }
        stats.deleted_static_prefixes += 1;
    }
    Ok(())
}

async fn gc_abandoned_uploads(
    db: &doc_db::Database,
    bundle_store: &BundleStore,
    now: DateTime,
    stats: &mut GcStats,
) -> anyhow::Result<()> {
    let originals = bundle_store.list_all("original/", now).await?;
    let compiled_pairs: HashSet<(String, u64)> = bundle_store
        .list_all("compiled/", now)
        .await?
        .iter()
        .filter_map(|object| parse_compiled_key(&object.key))
        .collect();

    for object in &originals {
        let Some((project_id, code_version)) = parse_original_key(&object.key) else {
            continue;
        };
        if (now - object.last_modified).num_seconds() < ABANDONED_UPLOAD_TTL_SECS {
            continue;
        }
        let doc = (CompiledBundleDocGet {
            project_id: project_id.as_str(),
            code_version,
        })
        .send_with(db)
        .await?;
        if doc.is_some() {
            continue;
        }
        if compiled_pairs.contains(&(project_id.clone(), code_version)) {
            // A compiled object exists but no CompiledBundleDoc: the
            // bundle_compiled callback was lost. The upload is recoverable by
            // re-running the callback, so it is not abandoned — keep it.
            tracing::warn!(
                %project_id,
                code_version,
                "bundle_gc: compiled object without CompiledBundleDoc, skipping",
            );
            continue;
        }
        bundle_store.delete(&object.key, now).await?;
        stats.deleted_orphans += 1;
        gc_abandoned_frontend_assets(db, &project_id, code_version, now, stats).await?;
    }
    Ok(())
}

async fn gc_abandoned_frontend_assets(
    db: &doc_db::Database,
    project_id: &str,
    code_version: u64,
    now: DateTime,
    stats: &mut GcStats,
) -> anyhow::Result<()> {
    let store = ProjectR2Store::frontend_assets(&ProjectStorage::resolve(db, project_id).await?);
    let objects = store.list_all(&format!("{code_version}/"), now).await?;
    if objects.is_empty() {
        return Ok(());
    }
    for object in objects {
        store.delete(&object.key, now).await?;
    }
    stats.deleted_static_prefixes += 1;
    Ok(())
}

async fn query_compiled_bundles(
    db: &doc_db::Database,
    project_id: &str,
) -> anyhow::Result<Vec<CompiledBundleDoc>> {
    let mut all = Vec::new();
    let mut after: Option<u64> = None;
    loop {
        let page: Vec<CompiledBundleDoc> = (CompiledBundleDocQuery {
            project_id,
            code_version: after,
            limit: Some(QUERY_PAGE_LIMIT),
        })
        .send_with(db)
        .await?;
        let page_len = page.len();
        if let Some(last) = page.last() {
            after = Some(last.code_version);
        }
        all.extend(page);
        if page_len < QUERY_PAGE_LIMIT {
            break;
        }
    }
    Ok(all)
}

fn parse_original_key(key: &str) -> Option<(String, u64)> {
    let rest = key.strip_prefix("original/")?.strip_suffix(".tar")?;
    let (project_id, code_version) = rest.split_once('/')?;
    Some((project_id.to_string(), code_version.parse().ok()?))
}

fn parse_frontend_asset_key(key: &str) -> Option<u64> {
    key.split_once('/')?.0.parse().ok()
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }
    match run_gc().await {
        Ok(stats) => Output::Ok {
            deleted_versions_count: stats.deleted_versions,
            deleted_orphans_count: stats.deleted_orphans,
            deleted_static_prefixes_count: stats.deleted_static_prefixes,
        },
        Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
