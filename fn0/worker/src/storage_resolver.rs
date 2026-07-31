//! Per-project storage targets, fed from the worker manifest.
//!
//! Targets are built when a manifest version is applied, not when a request
//! arrives: resolving one costs an OCI KMS decrypt, and doing that in the
//! request path would put a network round trip in front of every guest write.
//! Plaintext secrets are cached by ciphertext, so a manifest version that
//! changes nothing about a project costs no decrypt at all.
//!
//! A project with no `storage` entry resolves to nothing rather than to a
//! fallback. Every project's objects live in its owner's own Cloudflare
//! account, so there is no account left to fall back to, and guessing one would
//! write a tenant's objects into somebody else's bucket.

use crate::vault_client::VaultClient;
use dashmap::DashMap;
use fn0::{
    ObjectStorageResolver, PrivateObjectStorageTarget, PublicStorageResolver, PublicStorageTarget,
    R2Credentials,
};
use fn0_shared_schema::{WorkerProjectStorage, WorkerR2Credential};
use opendal::Operator;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

fn rendered_html_cache_operator(
    endpoint: &str,
    bucket: &str,
    region: &str,
    access_key_id: &str,
    secret_access_key: &str,
) -> anyhow::Result<Operator> {
    Ok(Operator::new(
        opendal::services::S3::default()
            .bucket(bucket)
            .region(region)
            .endpoint(endpoint)
            .access_key_id(access_key_id)
            .secret_access_key(secret_access_key)
            .disable_config_load()
            .disable_ec2_metadata(),
    )?
    .finish())
}

struct ProjectTargets {
    config_version: u64,
    private_object_storage: Arc<PrivateObjectStorageTarget>,
    public_object_storage: Arc<PublicStorageTarget>,
    rendered_html_cache: Operator,
}

pub struct ManifestStorageResolver {
    vault: Arc<VaultClient>,
    projects: DashMap<String, ProjectTargets>,
    decrypted_secrets: DashMap<String, String>,
}

impl ManifestStorageResolver {
    pub fn new(vault: Arc<VaultClient>) -> Self {
        Self {
            vault,
            projects: DashMap::new(),
            decrypted_secrets: DashMap::new(),
        }
    }

    /// Rebuilds the per-project targets to match a freshly applied manifest.
    ///
    /// A project whose decrypt fails keeps the target it already had, so a
    /// transient KMS failure does not take a serving project offline.
    pub async fn apply(&self, storages: HashMap<&String, &WorkerProjectStorage>) {
        for (project_id, storage) in &storages {
            if self
                .projects
                .get(*project_id)
                .is_some_and(|current| current.config_version == storage.config_version)
            {
                continue;
            }
            match self.build(storage).await {
                Ok(targets) => {
                    self.projects.insert((*project_id).clone(), targets);
                }
                Err(error) => {
                    tracing::error!(
                        %error,
                        project_id = %project_id,
                        config_version = storage.config_version,
                        "project storage target could not be built"
                    );
                }
            }
        }

        self.projects
            .retain(|project_id, _| storages.contains_key(project_id));

        let live_ciphertexts: HashSet<&str> = storages
            .values()
            .map(|storage| storage.credential.secret_ciphertext.as_str())
            .collect();
        self.decrypted_secrets
            .retain(|ciphertext, _| live_ciphertexts.contains(ciphertext.as_str()));
    }

    async fn build(&self, storage: &WorkerProjectStorage) -> anyhow::Result<ProjectTargets> {
        let credentials = self.credentials(storage, &storage.credential).await?;
        let rendered_html_cache = rendered_html_cache_operator(
            &format!("https://{}", credentials.endpoint_host),
            &storage.rendered_html_cache_bucket,
            &credentials.region,
            &credentials.access_key_id,
            &credentials.secret_access_key,
        )?;
        Ok(ProjectTargets {
            config_version: storage.config_version,
            private_object_storage: Arc::new(PrivateObjectStorageTarget {
                credentials: credentials.clone(),
                bucket: storage.private_object_storage_bucket.clone(),
            }),
            public_object_storage: Arc::new(PublicStorageTarget {
                credentials,
                bucket: storage.public_object_storage_bucket.clone(),
                base_url: storage
                    .public_object_storage_base_url
                    .trim_end_matches('/')
                    .to_string(),
            }),
            rendered_html_cache,
        })
    }

    async fn credentials(
        &self,
        storage: &WorkerProjectStorage,
        credential: &WorkerR2Credential,
    ) -> anyhow::Result<R2Credentials> {
        Ok(R2Credentials::for_account(
            &storage.account_id,
            storage.region.clone(),
            credential.access_key_id.clone(),
            self.secret(&credential.secret_ciphertext).await?,
        ))
    }

    async fn secret(&self, ciphertext: &str) -> anyhow::Result<String> {
        if let Some(cached) = self.decrypted_secrets.get(ciphertext) {
            return Ok(cached.clone());
        }
        let plaintext = String::from_utf8(self.vault.decrypt(ciphertext).await?)
            .map_err(|error| anyhow::anyhow!("decrypted R2 secret is not utf8: {error}"))?;
        self.decrypted_secrets
            .insert(ciphertext.to_string(), plaintext.clone());
        Ok(plaintext)
    }

    pub fn rendered_html_cache_operator(&self, project_id: &str) -> Option<Operator> {
        self.projects
            .get(project_id)
            .map(|targets| targets.rendered_html_cache.clone())
    }
}

impl ObjectStorageResolver for ManifestStorageResolver {
    fn resolve(&self, project_id: &str) -> Option<Arc<PrivateObjectStorageTarget>> {
        self.projects
            .get(project_id)
            .map(|targets| targets.private_object_storage.clone())
    }
}

impl PublicStorageResolver for ManifestStorageResolver {
    fn resolve(&self, project_id: &str) -> Option<Arc<PublicStorageTarget>> {
        self.projects
            .get(project_id)
            .map(|targets| targets.public_object_storage.clone())
    }
}
