//! Per-project storage targets, fed from the worker manifest.
//!
//! Targets are built when a manifest version is applied, not when a request
//! arrives: resolving one costs an OCI KMS decrypt, and doing that in the
//! request path would put a network round trip in front of every guest write.
//! Plaintext secrets are cached by ciphertext, so a manifest that repeats the
//! same credential across a project's three stores costs one decrypt, and a
//! manifest version that changes nothing about a project costs none.
//!
//! A project with no `storage` entry resolves to the platform target built from
//! the process environment, which is what keeps `fn0-control`, the docs site
//! and the apex project working while user projects move to their own accounts.

use crate::vault_client::VaultClient;
use dashmap::DashMap;
use fn0::{ObjectStorageResolver, PublicStorageResolver, PublicStorageTarget, R2Credentials};
use fn0_shared_schema::{WorkerProjectStorage, WorkerR2Credential};
use opendal::Operator;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct PlatformTargets {
    pub object: Arc<R2Credentials>,
    pub public: Arc<PublicStorageTarget>,
    pub page: Operator,
}

impl PlatformTargets {
    pub fn from_env() -> anyhow::Result<Self> {
        let var =
            |name: &str| std::env::var(name).map_err(|_| anyhow::anyhow!("{name} is required"));

        let object = R2Credentials::for_account(
            &var("FN0_OBJECT_STORAGE_ACCOUNT_ID")?,
            std::env::var("FN0_OBJECT_STORAGE_REGION").unwrap_or_else(|_| "auto".to_string()),
            var("FN0_OBJECT_STORAGE_ACCESS_KEY_ID")?,
            var("FN0_OBJECT_STORAGE_SECRET_ACCESS_KEY")?,
        );

        let public = PublicStorageTarget {
            credentials: R2Credentials::for_account(
                &var("FN0_PUBLIC_STORAGE_ACCOUNT_ID")?,
                std::env::var("FN0_PUBLIC_STORAGE_REGION").unwrap_or_else(|_| "auto".to_string()),
                var("FN0_PUBLIC_STORAGE_ACCESS_KEY_ID")?,
                var("FN0_PUBLIC_STORAGE_SECRET_ACCESS_KEY")?,
            ),
            bucket: var("FN0_PUBLIC_STORAGE_BUCKET")?,
            base_url: var("FN0_PUBLIC_STORAGE_CDN_ORIGIN")?
                .trim_end_matches('/')
                .to_string(),
        };

        // Named FN0_STATIC_ASSET_STORAGE_* on the worker for historical
        // reasons: these credentials point at the private page-cache bucket,
        // not at the public asset bucket the name suggests.
        let page_account_id = var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")?;
        let page = page_operator(
            &std::env::var("FN0_STATIC_ASSET_STORAGE_ENDPOINT")
                .unwrap_or_else(|_| format!("https://{page_account_id}.r2.cloudflarestorage.com")),
            &var("FN0_STATIC_ASSET_STORAGE_BUCKET")?,
            "auto",
            &var("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID")?,
            &var("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY")?,
        )?;

        Ok(Self {
            object: Arc::new(object),
            public: Arc::new(public),
            page,
        })
    }
}

fn page_operator(
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
    object: Arc<R2Credentials>,
    public: Arc<PublicStorageTarget>,
    page: Operator,
}

pub struct ManifestStorageResolver {
    platform: PlatformTargets,
    vault: Arc<VaultClient>,
    projects: DashMap<String, ProjectTargets>,
    decrypted_secrets: DashMap<String, String>,
}

impl ManifestStorageResolver {
    pub fn new(platform: PlatformTargets, vault: Arc<VaultClient>) -> Self {
        Self {
            platform,
            vault,
            projects: DashMap::new(),
            decrypted_secrets: DashMap::new(),
        }
    }

    /// Rebuilds the per-project targets to match a freshly applied manifest.
    ///
    /// A project whose decrypt fails keeps the target it already had: the
    /// alternative is falling back to the platform account, which would write
    /// one tenant's objects into another's bucket.
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
            .flat_map(|storage| {
                [
                    storage.object.secret_ciphertext.as_str(),
                    storage.public.secret_ciphertext.as_str(),
                    storage.page.secret_ciphertext.as_str(),
                ]
            })
            .collect();
        self.decrypted_secrets
            .retain(|ciphertext, _| live_ciphertexts.contains(ciphertext.as_str()));
    }

    async fn build(&self, storage: &WorkerProjectStorage) -> anyhow::Result<ProjectTargets> {
        let object = self.credentials(storage, &storage.object).await?;
        let public = self.credentials(storage, &storage.public).await?;
        let page = self.credentials(storage, &storage.page).await?;
        let page_operator = page_operator(
            &format!("https://{}", page.endpoint_host),
            &storage.page_bucket,
            &page.region,
            &page.access_key_id,
            &page.secret_access_key,
        )?;
        Ok(ProjectTargets {
            config_version: storage.config_version,
            object: Arc::new(object),
            public: Arc::new(PublicStorageTarget {
                credentials: public,
                bucket: storage.public_bucket.clone(),
                base_url: storage.public_base_url.trim_end_matches('/').to_string(),
            }),
            page: page_operator,
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

    pub fn page_operator(&self, project_id: &str) -> Operator {
        self.projects
            .get(project_id)
            .map(|targets| targets.page.clone())
            .unwrap_or_else(|| self.platform.page.clone())
    }
}

impl ObjectStorageResolver for ManifestStorageResolver {
    fn resolve(&self, project_id: &str) -> Option<Arc<R2Credentials>> {
        Some(
            self.projects
                .get(project_id)
                .map(|targets| targets.object.clone())
                .unwrap_or_else(|| self.platform.object.clone()),
        )
    }
}

impl PublicStorageResolver for ManifestStorageResolver {
    fn resolve(&self, project_id: &str) -> Option<Arc<PublicStorageTarget>> {
        Some(
            self.projects
                .get(project_id)
                .map(|targets| targets.public.clone())
                .unwrap_or_else(|| self.platform.public.clone()),
        )
    }
}
