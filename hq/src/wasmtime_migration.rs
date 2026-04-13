use aws_sdk_s3::Client as S3Client;
use color_eyre::eyre::{Result, eyre};
use doc_db::{Deployment, DocDb, RebuildEntry, RebuildStatus};
use std::collections::HashMap;
use std::io::{Read, Write};
use tracing::*;

const REBUILD_TASK: &str = "wasmtime_rebuild";
const FINALIZE_TASK: &str = "wasmtime_finalize";
const CLEANUP_TASK: &str = "wasmtime_cleanup";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RebuildPayload {
    pub target_version: String,
    pub subdomain: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FinalizePayload {
    pub target_version: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CleanupPayload {
    pub bucket: String,
    pub key: String,
    pub forget_version: Option<String>,
}

/// Compute the most-recent active deployment per subdomain from the append-only
/// deployment log. Subdomains whose latest record is `Undeploy` are excluded.
pub fn latest_active_deployments(deployments: &[Deployment]) -> Vec<RebuildEntry> {
    let mut latest: HashMap<String, RebuildEntry> = HashMap::new();
    for d in deployments {
        match d {
            Deployment::Deploy {
                subdomain,
                code_id,
                code_version,
            } => {
                latest.insert(
                    subdomain.clone(),
                    RebuildEntry {
                        subdomain: subdomain.clone(),
                        code_id: *code_id,
                        code_version: *code_version,
                        status: RebuildStatus::Pending,
                    },
                );
            }
            Deployment::Undeploy { subdomain } => {
                latest.remove(subdomain);
            }
        }
    }
    let mut entries: Vec<_> = latest.into_values().collect();
    entries.sort_by(|a, b| a.subdomain.cmp(&b.subdomain));
    entries
}

/// Run on HQ startup. Initializes the active wasmtime version on first boot,
/// or starts/resumes a migration when HQ's compiled-in version differs from
/// what workers are currently running.
pub async fn ensure_migration(
    doc_db: &DocDb,
    deployments: &[Deployment],
) -> Result<()> {
    let target = fn0_wasmtime::WASMTIME_VERSION;
    let state = doc_db.get_wasmtime_state().await?;

    if state.active_version.is_none() {
        info!(version = target, "First boot: setting active wasmtime version");
        doc_db.set_active_wasmtime_version(target).await?;
        return Ok(());
    }

    let active = state.active_version.unwrap();
    doc_db.record_known_wasmtime_version(target).await?;

    if active == target {
        debug!(version = target, "wasmtime version already active");
        return Ok(());
    }

    info!(
        from = %active,
        to = target,
        "wasmtime version mismatch; ensuring migration"
    );

    let entries = latest_active_deployments(deployments);
    info!(
        target_version = target,
        deployment_count = entries.len(),
        "Recording rebuild entries"
    );
    doc_db.upsert_rebuild_entries(target, &entries).await?;

    for entry in &entries {
        let payload = serde_json::to_string(&RebuildPayload {
            target_version: target.to_string(),
            subdomain: entry.subdomain.clone(),
        })?;
        let job_id = format!("wasmtime_rebuild:{}:{}", target, entry.subdomain);
        doc_db.insert_job(&job_id, REBUILD_TASK, &payload).await?;
    }

    let pending = doc_db.pending_rebuild_count(target).await?;
    if pending == 0 && !entries.is_empty() {
        let payload = serde_json::to_string(&FinalizePayload {
            target_version: target.to_string(),
        })?;
        let job_id = format!("wasmtime_finalize:{}", target);
        doc_db.insert_job(&job_id, FINALIZE_TASK, &payload).await?;
    }

    if entries.is_empty() {
        info!("No live deployments; finalizing immediately");
        let payload = serde_json::to_string(&FinalizePayload {
            target_version: target.to_string(),
        })?;
        let job_id = format!("wasmtime_finalize:{}", target);
        doc_db.insert_job(&job_id, FINALIZE_TASK, &payload).await?;
    }

    Ok(())
}

pub async fn process_rebuild(
    s3_client: &S3Client,
    doc_db: &DocDb,
    wasm_bucket: &str,
    cwasm_bucket: &str,
    payload: &str,
) -> Result<(), String> {
    let payload: RebuildPayload =
        serde_json::from_str(payload).map_err(|e| format!("Invalid rebuild payload: {e}"))?;

    let target_version = payload.target_version;
    let subdomain = payload.subdomain;

    let cwasm_key = format!("bundles/wasmtime-{target_version}/{subdomain}.tar.zst");

    let already_built = head_object_exists(s3_client, cwasm_bucket, &cwasm_key)
        .await
        .map_err(|e| format!("HEAD {cwasm_key}: {e}"))?;

    if already_built {
        info!(%subdomain, %target_version, "cwasm bundle already exists; skipping recompile");
    } else {
        let raw_key = format!("raw/{subdomain}.raw.tar");
        let raw = match get_object(s3_client, wasm_bucket, &raw_key).await {
            Ok(b) => b,
            Err(err) => {
                let fallback = format!("bundles/{subdomain}.raw.tar");
                warn!(
                    %subdomain,
                    %raw_key,
                    %fallback,
                    %err,
                    "raw archive missing; falling back to upload bucket"
                );
                get_object(s3_client, wasm_bucket, &fallback)
                    .await
                    .map_err(|e| format!("Fetching raw bundle for {subdomain}: {e}"))?
            }
        };

        let cwasm_bundle = tokio::task::spawn_blocking(move || rebuild_bundle(raw))
            .await
            .map_err(|e| format!("rebuild join error: {e}"))?
            .map_err(|e| format!("rebuild for {subdomain}: {e}"))?;

        put_object(s3_client, cwasm_bucket, &cwasm_key, cwasm_bundle)
            .await
            .map_err(|e| format!("PUT {cwasm_key}: {e}"))?;
        info!(%subdomain, %target_version, "rebuilt cwasm bundle");
    }

    doc_db
        .mark_rebuild_done(&target_version, &subdomain)
        .await
        .map_err(|e| format!("mark_rebuild_done: {e}"))?;

    let pending = doc_db
        .pending_rebuild_count(&target_version)
        .await
        .map_err(|e| format!("pending_rebuild_count: {e}"))?;

    if pending == 0 {
        info!(%target_version, "all rebuilds done; enqueuing finalize");
        let finalize_payload = serde_json::to_string(&FinalizePayload {
            target_version: target_version.clone(),
        })
        .map_err(|e| format!("serialize finalize payload: {e}"))?;
        let job_id = format!("wasmtime_finalize:{}", target_version);
        doc_db
            .insert_job(&job_id, FINALIZE_TASK, &finalize_payload)
            .await
            .map_err(|e| format!("insert finalize: {e}"))?;
    }

    Ok(())
}

pub async fn process_finalize(
    doc_db: &DocDb,
    cwasm_bucket: &str,
    payload: &str,
) -> Result<(), String> {
    let payload: FinalizePayload =
        serde_json::from_str(payload).map_err(|e| format!("Invalid finalize payload: {e}"))?;

    let target_version = payload.target_version;

    let state = doc_db
        .get_wasmtime_state()
        .await
        .map_err(|e| format!("get_wasmtime_state: {e}"))?;

    let previous_active = state.active_version.clone();

    if state.active_version.as_deref() != Some(target_version.as_str()) {
        info!(%target_version, "promoting active wasmtime version");
        doc_db
            .set_active_wasmtime_version(&target_version)
            .await
            .map_err(|e| format!("set_active: {e}"))?;
    }

    let entries = doc_db
        .list_rebuild_entries(&target_version)
        .await
        .map_err(|e| format!("list_rebuild_entries: {e}"))?;

    if let Some(prev) = previous_active {
        if prev != target_version {
            info!(version = %prev, "enqueuing cleanup for previous wasmtime version");
            for entry in &entries {
                let key = format!("bundles/wasmtime-{prev}/{}.tar.zst", entry.subdomain);
                let cleanup = serde_json::to_string(&CleanupPayload {
                    bucket: cwasm_bucket.to_string(),
                    key: key.clone(),
                    forget_version: None,
                })
                .map_err(|e| format!("serialize cleanup: {e}"))?;
                let job_id = format!("wasmtime_cleanup:{prev}:{}", entry.subdomain);
                doc_db
                    .insert_job(&job_id, CLEANUP_TASK, &cleanup)
                    .await
                    .map_err(|e| format!("insert cleanup: {e}"))?;
            }

            let sentinel_key = format!("bundles/wasmtime-{prev}/.deprecated");
            let sentinel_payload = serde_json::to_string(&CleanupPayload {
                bucket: cwasm_bucket.to_string(),
                key: sentinel_key,
                forget_version: Some(prev.clone()),
            })
            .map_err(|e| format!("serialize cleanup sentinel: {e}"))?;
            let job_id = format!("wasmtime_cleanup_finalize:{prev}");
            doc_db
                .insert_job(&job_id, CLEANUP_TASK, &sentinel_payload)
                .await
                .map_err(|e| format!("insert cleanup sentinel: {e}"))?;
        }
    }

    if let Err(err) = doc_db.delete_rebuild_table(&target_version).await {
        warn!(%err, "Failed to drop rebuild progress table");
    }

    Ok(())
}

pub async fn process_cleanup(
    s3_client: &S3Client,
    doc_db: &DocDb,
    payload: &str,
) -> Result<(), String> {
    let payload: CleanupPayload =
        serde_json::from_str(payload).map_err(|e| format!("Invalid cleanup payload: {e}"))?;

    if let Err(err) = s3_client
        .delete_object()
        .bucket(&payload.bucket)
        .key(&payload.key)
        .send()
        .await
    {
        return Err(format!("S3 delete {}: {err}", payload.key));
    }

    if let Some(version) = payload.forget_version {
        doc_db
            .forget_wasmtime_version(&version)
            .await
            .map_err(|e| format!("forget_wasmtime_version: {e}"))?;
        info!(%version, "forgot wasmtime version after cleanup");
    }

    Ok(())
}

async fn get_object(
    s3_client: &S3Client,
    bucket: &str,
    key: &str,
) -> color_eyre::Result<Vec<u8>> {
    let output = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| eyre!("get_object {bucket}/{key}: {e}"))?;
    let body = output
        .body
        .collect()
        .await
        .map_err(|e| eyre!("body collect: {e}"))?;
    Ok(body.into_bytes().to_vec())
}

async fn put_object(
    s3_client: &S3Client,
    bucket: &str,
    key: &str,
    body: Vec<u8>,
) -> color_eyre::Result<()> {
    s3_client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(aws_sdk_s3::primitives::ByteStream::from(body))
        .send()
        .await
        .map_err(|e| eyre!("put_object {bucket}/{key}: {e}"))?;
    Ok(())
}

async fn head_object_exists(
    s3_client: &S3Client,
    bucket: &str,
    key: &str,
) -> color_eyre::Result<bool> {
    match s3_client
        .head_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
    {
        Ok(_) => Ok(true),
        Err(err) => {
            let svc_err = err.into_service_error();
            if svc_err.is_not_found() {
                Ok(false)
            } else {
                Err(eyre!("head_object {bucket}/{key}: {svc_err}"))
            }
        }
    }
}

/// Recompile a `.raw.tar` (containing manifest.json + backend.wasm + optional
/// frontend.js + public/*) into a `.tar.zst` cwasm bundle for the current
/// wasmtime version.
fn rebuild_bundle(raw_bytes: Vec<u8>) -> color_eyre::Result<Vec<u8>> {
    let cursor = std::io::Cursor::new(&raw_bytes);
    let mut archive = tar::Archive::new(cursor);

    let mut manifest: Option<Vec<u8>> = None;
    let mut backend_wasm: Option<Vec<u8>> = None;
    let mut frontend_js: Option<Vec<u8>> = None;
    let mut public_files: Vec<(String, Vec<u8>)> = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let path_str = path.to_string_lossy().to_string();

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;

        match path_str.as_str() {
            "manifest.json" => manifest = Some(buf),
            "backend.wasm" => backend_wasm = Some(buf),
            "frontend.js" => frontend_js = Some(buf),
            other if other.starts_with("public/") => {
                public_files.push((other.to_string(), buf));
            }
            _ => {}
        }
    }

    let manifest = manifest.ok_or_else(|| eyre!("raw bundle missing manifest.json"))?;
    let backend_wasm =
        backend_wasm.ok_or_else(|| eyre!("raw bundle missing backend.wasm"))?;

    let cwasm =
        fn0_wasmtime::compile(&backend_wasm).map_err(|e| eyre!("wasmtime compile: {e}"))?;
    let cwasm_zst = zstd::encode_all(cwasm.as_slice(), 22)?;

    let mut tar_buf: Vec<u8> = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_buf);
        append_to_tar(&mut builder, "manifest.json", &manifest)?;
        append_to_tar(&mut builder, "backend.cwasm.zst", &cwasm_zst)?;
        if let Some(js) = frontend_js {
            append_to_tar(&mut builder, "frontend.js", &js)?;
        }
        for (path, bytes) in &public_files {
            append_to_tar(&mut builder, path, bytes)?;
        }
        builder.finish()?;
    }

    let zst = zstd::encode_all(tar_buf.as_slice(), 22)?;
    Ok(zst)
}

fn append_to_tar<W: Write>(
    builder: &mut tar::Builder<W>,
    path: &str,
    data: &[u8],
) -> color_eyre::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, data)
        .map_err(|e| eyre!("tar append {path}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_db::Deployment;

    #[test]
    fn latest_active_picks_last_deploy_per_subdomain() {
        let deployments = vec![
            Deployment::Deploy {
                subdomain: "alice".into(),
                code_id: 1,
                code_version: 1,
            },
            Deployment::Deploy {
                subdomain: "alice".into(),
                code_id: 1,
                code_version: 2,
            },
            Deployment::Deploy {
                subdomain: "bob".into(),
                code_id: 2,
                code_version: 7,
            },
        ];
        let entries = latest_active_deployments(&deployments);
        assert_eq!(entries.len(), 2);
        let alice = entries.iter().find(|e| e.subdomain == "alice").unwrap();
        assert_eq!(alice.code_version, 2);
        let bob = entries.iter().find(|e| e.subdomain == "bob").unwrap();
        assert_eq!(bob.code_version, 7);
    }

    #[test]
    fn latest_active_drops_undeploy() {
        let deployments = vec![
            Deployment::Deploy {
                subdomain: "alice".into(),
                code_id: 1,
                code_version: 1,
            },
            Deployment::Undeploy {
                subdomain: "alice".into(),
            },
            Deployment::Deploy {
                subdomain: "bob".into(),
                code_id: 2,
                code_version: 1,
            },
        ];
        let entries = latest_active_deployments(&deployments);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].subdomain, "bob");
    }

    #[test]
    fn latest_active_redeploy_after_undeploy() {
        let deployments = vec![
            Deployment::Deploy {
                subdomain: "alice".into(),
                code_id: 1,
                code_version: 1,
            },
            Deployment::Undeploy {
                subdomain: "alice".into(),
            },
            Deployment::Deploy {
                subdomain: "alice".into(),
                code_id: 1,
                code_version: 2,
            },
        ];
        let entries = latest_active_deployments(&deployments);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].code_version, 2);
    }

    #[tokio::test]
    async fn ensure_migration_first_boot_sets_active() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        ensure_migration(&db, &[]).await.unwrap();
        let s = db.get_wasmtime_state().await.unwrap();
        assert_eq!(s.active_version.as_deref(), Some(fn0_wasmtime::WASMTIME_VERSION));
    }

    #[tokio::test]
    async fn ensure_migration_same_version_noop() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.set_active_wasmtime_version(fn0_wasmtime::WASMTIME_VERSION)
            .await
            .unwrap();
        ensure_migration(&db, &[]).await.unwrap();
        assert!(!db.job_exists_for_task("wasmtime_rebuild").await.unwrap());
        assert!(!db.job_exists_for_task("wasmtime_finalize").await.unwrap());
    }

    #[tokio::test]
    async fn ensure_migration_enqueues_jobs_for_each_deployment() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.set_active_wasmtime_version("99-old").await.unwrap();
        let deployments = vec![
            Deployment::Deploy {
                subdomain: "alice".into(),
                code_id: 1,
                code_version: 1,
            },
            Deployment::Deploy {
                subdomain: "bob".into(),
                code_id: 2,
                code_version: 1,
            },
        ];
        ensure_migration(&db, &deployments).await.unwrap();
        let entries = db
            .list_rebuild_entries(fn0_wasmtime::WASMTIME_VERSION)
            .await
            .unwrap();
        assert_eq!(entries.len(), 2);
        assert!(db.job_exists_for_task("wasmtime_rebuild").await.unwrap());
    }

    #[tokio::test]
    async fn ensure_migration_is_idempotent_resume() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.set_active_wasmtime_version("99-old").await.unwrap();
        let deployments = vec![Deployment::Deploy {
            subdomain: "alice".into(),
            code_id: 1,
            code_version: 1,
        }];
        ensure_migration(&db, &deployments).await.unwrap();
        ensure_migration(&db, &deployments).await.unwrap();
        let entries = db
            .list_rebuild_entries(fn0_wasmtime::WASMTIME_VERSION)
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn ensure_migration_no_deployments_finalizes_immediately() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.set_active_wasmtime_version("99-old").await.unwrap();
        ensure_migration(&db, &[]).await.unwrap();
        assert!(db.job_exists_for_task("wasmtime_finalize").await.unwrap());
    }
}

