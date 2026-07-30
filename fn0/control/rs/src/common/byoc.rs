//! Resolving which Cloudflare account a project's storage lives in.
//!
//! Every control-plane operation that touches R2 or a cache used to read one
//! set of environment variables. With bring-your-own-Cloudflare the account is
//! a per-project fact, so those operations go through [`ProjectStorage`]
//! instead: it returns either the user's account or the platform's, unchanged,
//! for the projects the platform owns (`fn0-control`, the docs site, the apex
//! project).
//!
//! What the control plane can do in a user's account is deliberately small.
//! Provisioning — creating buckets, attaching the CDN hostname, writing the
//! cache rule, signing origin certificates — runs in the CLI on the user's own
//! machine with a token fn0 never receives. What is left here is object access
//! to three buckets and cache purge on one zone.

use crate::common::cloudflare::CloudflareClient;
use crate::common::vault;
use crate::docs::*;
use forte_sdk::*;

pub const OBJECT_STORAGE_BUCKET_PREFIX: &str =
    crate::common::r2_store::OBJECT_STORAGE_BUCKET_PREFIX;

/// The bucket names fn0 uses inside a user's own account. Unsuffixed because
/// the account is theirs and one set serves all of their projects.
pub const USER_ASSET_BUCKET: &str = "fn0-static-asset";
pub const USER_PAGE_BUCKET: &str = "fn0-static-page";

pub struct R2Keys {
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Everything an operation needs to reach one project's objects.
pub struct ProjectStorage {
    pub account_id: String,
    pub object_bucket: String,
    pub asset_bucket: String,
    pub page_bucket: String,
    pub public_base_url: String,
    pub object_keys: R2Keys,
    pub asset_keys: R2Keys,
    pub page_keys: R2Keys,
    /// `None` for platform-owned projects, which have no user zone to act on.
    pub connection: Option<Connection>,
}

pub struct Connection {
    pub zone_id: String,
    pub zone_name: String,
    pub static_hostname: String,
    /// Cache purge on `zone_id` only.
    pub purge_token: String,
    pub config_version: u64,
}

impl ProjectStorage {
    /// Where the project's objects are served from *right now*. A connection
    /// that is still migrating resolves to the platform account, because that
    /// is where the objects still are.
    pub async fn resolve(db: &doc_db::Database, project_id: &str) -> anyhow::Result<Self> {
        match (ProjectCloudflareConfigDocGet { project_id })
            .send_with(db)
            .await?
        {
            Some(config) if config.state != CloudflareConnectionState::Migrating => {
                Self::from_config(config).await
            }
            _ => Self::platform(project_id),
        }
    }

    /// The connected account regardless of migration state. Only the migration
    /// itself and the domain flow want this: everything else must see the
    /// account the project is actually being served from.
    pub async fn resolve_connected(
        db: &doc_db::Database,
        project_id: &str,
    ) -> anyhow::Result<Option<Self>> {
        match (ProjectCloudflareConfigDocGet { project_id })
            .send_with(db)
            .await?
        {
            Some(config) => Ok(Some(Self::from_config(config).await?)),
            None => Ok(None),
        }
    }

    async fn from_config(config: ProjectCloudflareConfigDoc) -> anyhow::Result<Self> {
        let secret_access_key =
            String::from_utf8(vault::decrypt(&config.dataplane_secret_ciphertext).await?)
                .map_err(|error| anyhow::anyhow!("data-plane secret is not utf8: {error}"))?;
        let purge_token = String::from_utf8(vault::decrypt(&config.purge_token_ciphertext).await?)
            .map_err(|error| anyhow::anyhow!("purge token is not utf8: {error}"))?;
        let keys = || R2Keys {
            access_key_id: config.dataplane_access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
        };
        Ok(Self {
            account_id: config.account_id.clone(),
            object_bucket: config.object_bucket.clone(),
            asset_bucket: config.asset_bucket.clone(),
            page_bucket: config.page_bucket.clone(),
            public_base_url: format!("https://{}", config.static_hostname),
            object_keys: keys(),
            asset_keys: keys(),
            page_keys: keys(),
            connection: Some(Connection {
                zone_id: config.zone_id,
                zone_name: config.zone_name,
                static_hostname: config.static_hostname,
                purge_token,
                config_version: config.config_version,
            }),
        })
    }

    fn platform(project_id: &str) -> anyhow::Result<Self> {
        let var = |name: &str| std::env::var(name).map_err(|_| anyhow::anyhow!("{name} not set"));
        Ok(Self {
            account_id: var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")?,
            object_bucket: format!("{OBJECT_STORAGE_BUCKET_PREFIX}{project_id}"),
            asset_bucket: var("FN0_STATIC_ASSET_STORAGE_BUCKET")?,
            page_bucket: var("FN0_STATIC_PAGE_STORAGE_BUCKET")?,
            public_base_url: var("FN0_PUBLIC_STORAGE_CDN_ORIGIN")?
                .trim_end_matches('/')
                .to_string(),
            object_keys: R2Keys {
                access_key_id: var("FN0_OBJECT_STORAGE_ACCESS_KEY_ID")?,
                secret_access_key: var("FN0_OBJECT_STORAGE_SECRET_ACCESS_KEY")?,
            },
            asset_keys: R2Keys {
                access_key_id: var("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID")?,
                secret_access_key: var("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY")?,
            },
            page_keys: R2Keys {
                access_key_id: var("FN0_STATIC_PAGE_STORAGE_ACCESS_KEY_ID")?,
                secret_access_key: var("FN0_STATIC_PAGE_STORAGE_SECRET_ACCESS_KEY")?,
            },
            connection: None,
        })
    }

    pub fn is_byoc(&self) -> bool {
        self.connection.is_some()
    }

    /// A client that can purge this project's zone and read its metadata, and
    /// nothing else. Named for what it can do rather than for whose account it
    /// belongs to, because that difference is the point.
    pub fn purge_client(&self) -> anyhow::Result<CloudflareClient> {
        match &self.connection {
            Some(connection) => Ok(CloudflareClient::for_zone(
                connection.purge_token.clone(),
                self.account_id.clone(),
                connection.zone_id.clone(),
            )),
            None => CloudflareClient::from_env(),
        }
    }

    /// Where a project's deployed frontend assets are fetched from, which the
    /// CLI has to know before it builds because it is baked into the bundle.
    pub fn asset_base_url(&self, project_id: &str, code_version: u64) -> String {
        format!("{}/{project_id}/{code_version}/", self.public_base_url)
    }
}

/// Mirrors a project's storage configuration into the worker manifest, which is
/// how workers learn to sign that project's traffic against the user's account.
///
/// Passing `None` puts the project back on the platform's account, which is
/// what disconnecting and tearing down both want.
pub async fn publish_storage_to_manifest(
    db: &doc_db::Database,
    project_id: &str,
    config: Option<&ProjectCloudflareConfigDoc>,
) -> anyhow::Result<()> {
    let storage = config.map(|config| {
        let credential = WorkerR2Credential {
            access_key_id: config.dataplane_access_key_id.clone(),
            secret_ciphertext: config.dataplane_secret_ciphertext.clone(),
        };
        WorkerProjectStorage {
            account_id: config.account_id.clone(),
            region: "auto".to_string(),
            object: credential.clone(),
            public: credential.clone(),
            public_bucket: config.asset_bucket.clone(),
            public_base_url: format!("https://{}", config.static_hostname),
            page: credential,
            page_bucket: config.page_bucket.clone(),
            config_version: config.config_version,
        }
    });

    let result = db
        .trx(|trx| {
            let project_id = project_id.to_string();
            let storage = storage.clone();
            async move {
                let mut manifest = match trx.get(WorkerManifestDocGet {}).await? {
                    Some(manifest) => manifest,
                    None => trx.create(WorkerManifestDoc {
                        manifest_version: 0,
                        project_manifests: std::collections::HashMap::new(),
                    })?,
                };
                let entry =
                    manifest
                        .project_manifests
                        .entry(project_id)
                        .or_insert(WorkerProjectManifest {
                            code_version: 0,
                            custom_domain: None,
                            static_cache_state: fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
                                .to_string(),
                            pending_code_version: None,
                            storage: None,
                        });
                if entry.storage != storage {
                    entry.storage = storage;
                    manifest.manifest_version += 1;
                }
                trx.commit::<_, std::convert::Infallible>(())
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(cancel) => match cancel {},
        doc_db::TrxResult::Conflict(detail) => {
            anyhow::bail!("publish_storage_to_manifest conflict: {detail:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}
