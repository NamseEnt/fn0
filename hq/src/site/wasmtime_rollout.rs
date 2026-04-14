use super::*;
use crate::deployment_cache::ActiveDeployment;
use aws_sdk_s3::{
    Client as S3Client,
    primitives::ByteStream,
    types::{Delete, ObjectIdentifier},
};
use bytes::Bytes;
use color_eyre::eyre::{Result, eyre};
use doc_db::{WasmtimeRolloutPhase, WasmtimeRolloutState};
use std::collections::HashMap;
use std::io::Read;

const WASMTIME_VERSION_ENV: &str = "FN0_WASMTIME_VERSION";

impl Site {
    pub(super) fn desired_wasmtime_version(&self) -> Result<String> {
        self.host_provider
            .envs()
            .get(WASMTIME_VERSION_ENV)
            .cloned()
            .ok_or_else(|| eyre!("{WASMTIME_VERSION_ENV} is missing from worker envs"))
    }

    pub(super) async fn reconcile_wasmtime_rollout(&self) -> Result<WasmtimeRolloutState> {
        let desired_version = self.desired_wasmtime_version()?;
        let mut state = self
            .doc_db
            .init_wasmtime_rollout_state(&self.name, &desired_version)
            .await?;

        if state.phase == WasmtimeRolloutPhase::Idle && state.current_version != desired_version {
            state.previous_version = Some(state.current_version.clone());
            state.target_version = desired_version;
            state.phase = WasmtimeRolloutPhase::Rebundling;
            self.doc_db.upsert_wasmtime_rollout_state(&state).await?;
        }

        if state.phase != WasmtimeRolloutPhase::Idle {
            let deployments = self.deployment_cache.current_deployments();
            let required_versions = required_versions_for_state(&state);
            let all_ready = self
                .ensure_versioned_bundles_ready(&required_versions, &deployments)
                .await?;

            if state.phase == WasmtimeRolloutPhase::Rebundling && all_ready {
                state.phase = WasmtimeRolloutPhase::ReadyToRoll;
                self.doc_db.upsert_wasmtime_rollout_state(&state).await?;
            }
        }

        Ok(state)
    }

    pub(super) async fn set_wasmtime_rollout_phase(
        &self,
        state: &WasmtimeRolloutState,
        phase: WasmtimeRolloutPhase,
    ) -> Result<WasmtimeRolloutState> {
        let mut updated = state.clone();
        updated.phase = phase;
        self.doc_db.upsert_wasmtime_rollout_state(&updated).await?;
        Ok(updated)
    }

    pub(super) async fn finish_wasmtime_rollout_cleanup(
        &self,
        state: &WasmtimeRolloutState,
    ) -> Result<WasmtimeRolloutState> {
        let mut updated = state.clone();
        if let Some(previous_version) = &state.previous_version {
            self.delete_target_prefix(&format!("bundles/{previous_version}/"))
                .await?;
        }
        updated.current_version = updated.target_version.clone();
        updated.previous_version = None;
        updated.phase = WasmtimeRolloutPhase::Idle;
        self.doc_db.upsert_wasmtime_rollout_state(&updated).await?;
        Ok(updated)
    }

    async fn ensure_versioned_bundles_ready(
        &self,
        bundle_versions: &[String],
        deployments: &[ActiveDeployment],
    ) -> Result<bool> {
        let (target_client, target_bucket) = self.target_bundle_store_client().await?;
        let mut all_ready = true;

        for deployment in deployments {
            for bundle_version in bundle_versions {
                let target_key = versioned_bundle_key(
                    bundle_version,
                    &deployment.subdomain,
                    deployment.code_version,
                );

                if self
                    .target_object_exists(&target_client, &target_bucket, &target_key)
                    .await?
                {
                    continue;
                }

                all_ready = false;
                self.compile_and_upload_versioned_bundle(
                    deployment,
                    bundle_version,
                    &target_client,
                    &target_bucket,
                )
                .await?;
            }
        }

        Ok(all_ready)
    }

    async fn compile_and_upload_versioned_bundle(
        &self,
        deployment: &ActiveDeployment,
        bundle_version: &str,
        target_client: &S3Client,
        target_bucket: &str,
    ) -> Result<()> {
        let source_key = source_bundle_key(&deployment.subdomain, deployment.code_version);
        let output = self
            .deploy_context
            .s3_client
            .get_object()
            .bucket(&self.deploy_context.wasm_bucket)
            .key(&source_key)
            .send()
            .await
            .map_err(|err| eyre!("failed to load {source_key}: {err}"))?;

        let tar_bytes = output
            .body
            .collect()
            .await
            .map_err(|err| eyre!("failed to read {source_key}: {err}"))?
            .into_bytes();

        let archive = RawBundleArchive::from_tar_bytes(tar_bytes)?;
        let compiled = archive.compile()?;
        let encoded = zstd::encode_all(compiled.as_slice(), 0)?;
        let target_key = versioned_bundle_key(
            bundle_version,
            &deployment.subdomain,
            deployment.code_version,
        );

        target_client
            .put_object()
            .bucket(target_bucket)
            .key(&target_key)
            .body(ByteStream::from(encoded))
            .send()
            .await
            .map_err(|err| eyre!("failed to upload {target_key}: {err}"))?;

        info!(
            subdomain = %deployment.subdomain,
            code_version = deployment.code_version,
            bundle_version,
            "Uploaded versioned wasmtime bundle"
        );
        Ok(())
    }

    async fn target_bundle_store_client(&self) -> Result<(S3Client, String)> {
        let envs = self.host_provider.envs();
        let bucket = envs
            .get("CWASM_BUCKET")
            .cloned()
            .ok_or_else(|| eyre!("CWASM_BUCKET is missing from worker envs"))?;
        let endpoint = envs
            .get("S3_ENDPOINT")
            .cloned()
            .ok_or_else(|| eyre!("S3_ENDPOINT is missing from worker envs"))?;
        let region = envs
            .get("S3_REGION")
            .cloned()
            .unwrap_or_else(|| "us-east-1".to_string());
        let access_key_id = envs
            .get("AWS_ACCESS_KEY_ID")
            .cloned()
            .ok_or_else(|| eyre!("AWS_ACCESS_KEY_ID is missing from worker envs"))?;
        let secret_access_key = envs
            .get("AWS_SECRET_ACCESS_KEY")
            .cloned()
            .ok_or_else(|| eyre!("AWS_SECRET_ACCESS_KEY is missing from worker envs"))?;

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .endpoint_url(endpoint)
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "hq-worker-bundle-store",
            ))
            .load()
            .await;

        let client = S3Client::from_conf(
            aws_sdk_s3::config::Builder::from(&config)
                .force_path_style(true)
                .build(),
        );

        Ok((client, bucket))
    }

    async fn target_object_exists(
        &self,
        client: &S3Client,
        bucket: &str,
        key: &str,
    ) -> Result<bool> {
        let resp = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(key)
            .max_keys(1)
            .send()
            .await
            .map_err(|err| eyre!("failed to list {key}: {err}"))?;

        Ok(resp
            .contents()
            .iter()
            .any(|object| object.key().is_some_and(|object_key| object_key == key)))
    }

    async fn delete_target_prefix(&self, prefix: &str) -> Result<()> {
        let (client, bucket) = self.target_bundle_store_client().await?;
        let mut continuation_token = None;

        loop {
            let mut request = client.list_objects_v2().bucket(&bucket).prefix(prefix);
            if let Some(token) = continuation_token.as_deref() {
                request = request.continuation_token(token);
            }

            let resp = request
                .send()
                .await
                .map_err(|err| eyre!("failed to list cleanup prefix {prefix}: {err}"))?;

            let keys: Vec<_> = resp
                .contents()
                .iter()
                .filter_map(|object| object.key().map(str::to_string))
                .collect();

            if !keys.is_empty() {
                let identifiers: Vec<_> = keys
                    .iter()
                    .map(|key| ObjectIdentifier::builder().key(key).build())
                    .collect::<std::result::Result<_, _>>()
                    .map_err(|err| eyre!("failed to build delete identifiers: {err}"))?;

                client
                    .delete_objects()
                    .bucket(&bucket)
                    .delete(
                        Delete::builder()
                            .set_objects(Some(identifiers))
                            .build()
                            .map_err(|err| eyre!("failed to build delete payload: {err}"))?,
                    )
                    .send()
                    .await
                    .map_err(|err| eyre!("failed to delete cleanup prefix {prefix}: {err}"))?;
            }

            if !resp.is_truncated().unwrap_or(false) {
                break;
            }
            continuation_token = resp.next_continuation_token().map(str::to_string);
        }

        Ok(())
    }
}

#[derive(Default)]
struct RawBundleArchive {
    manifest: Option<Vec<u8>>,
    backend_wasm: Option<Vec<u8>>,
    frontend_js: Option<Vec<u8>>,
    public_assets: HashMap<String, Vec<u8>>,
}

impl RawBundleArchive {
    fn from_tar_bytes(bytes: Bytes) -> Result<Self> {
        let mut archive = tar::Archive::new(bytes.as_ref());
        let mut bundle = Self::default();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_string_lossy().to_string();
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents)?;

            match path.as_str() {
                "manifest.json" => bundle.manifest = Some(contents),
                "backend.wasm" => bundle.backend_wasm = Some(contents),
                "frontend.js" => bundle.frontend_js = Some(contents),
                other if other.starts_with("public/") => {
                    bundle.public_assets.insert(other.to_string(), contents);
                }
                _ => {}
            }
        }

        Ok(bundle)
    }

    fn compile(self) -> Result<Vec<u8>> {
        let manifest = self
            .manifest
            .ok_or_else(|| eyre!("raw bundle is missing manifest.json"))?;
        let backend_wasm = self
            .backend_wasm
            .ok_or_else(|| eyre!("raw bundle is missing backend.wasm"))?;
        let backend_cwasm = fn0_wasmtime::compile(&backend_wasm).map_err(|err| eyre!("{err}"))?;
        let backend_cwasm_zst = zstd::encode_all(backend_cwasm.as_slice(), 22)?;

        let mut output = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut output);
            append_bytes(&mut builder, "manifest.json", &manifest)?;
            append_bytes(&mut builder, "backend.cwasm.zst", &backend_cwasm_zst)?;
            if let Some(frontend_js) = self.frontend_js {
                append_bytes(&mut builder, "frontend.js", &frontend_js)?;
            }
            for (path, bytes) in self.public_assets {
                append_bytes(&mut builder, &path, &bytes)?;
            }
            builder.finish()?;
        }
        Ok(output)
    }
}

fn append_bytes<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    path: &str,
    data: &[u8],
) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append_data(&mut header, path, data)?;
    Ok(())
}

pub(super) fn source_bundle_key(subdomain: &str, code_version: u64) -> String {
    format!("sources/{subdomain}/{code_version}.raw.tar")
}

pub(super) fn versioned_bundle_key(
    wasmtime_version: &str,
    subdomain: &str,
    code_version: u64,
) -> String {
    format!("bundles/{wasmtime_version}/{subdomain}/{code_version}.tar.zst")
}

fn required_versions_for_state(state: &WasmtimeRolloutState) -> Vec<String> {
    let mut versions = vec![state.current_version.clone()];
    if state.phase != WasmtimeRolloutPhase::Idle && state.target_version != state.current_version {
        versions.push(state.target_version.clone());
    }
    versions.sort();
    versions.dedup();
    versions
}
