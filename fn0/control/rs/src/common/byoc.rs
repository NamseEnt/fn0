//! Resolving which Cloudflare account a project's storage lives in.
//!
//! Every project's storage is its owner's, so this is a per-project lookup with
//! no fallback: a project that has not connected an account has nowhere for its
//! objects to go, and [`ProjectStorage::resolve`] says so rather than borrowing
//! somebody else's account.
//!
//! What the control plane can do in a user's account is deliberately small.
//! Provisioning — creating buckets, attaching the CDN hostnames, writing the
//! cache rule, signing origin certificates — runs in the CLI on the user's own
//! machine with a token fn0 never receives. What is left here is object access
//! to four buckets, split across two credentials, and cache purge on one zone.

use crate::common::cloudflare::CloudflareClient;
use crate::common::vault;
use crate::docs::*;
use forte_sdk::*;

pub struct R2Keys {
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Everything an operation needs to reach one project's objects.
///
/// The two key sets are not interchangeable. `worker_keys` opens the buckets
/// holding user data and the rendered-HTML cache; `frontend_asset_keys` opens
/// the frontend-asset bucket and nothing else. An operation takes the one that
/// matches what it is allowed to touch, so a wrong prefix in a delete loop
/// fails on a 403 rather than on user data.
pub struct ProjectStorage {
    pub account_id: String,
    pub private_object_storage_bucket: String,
    pub public_object_storage_bucket: String,
    pub frontend_asset_bucket: String,
    pub rendered_html_cache_bucket: String,
    pub worker_keys: R2Keys,
    pub frontend_asset_keys: R2Keys,
    pub connection: Connection,
}

pub struct Connection {
    pub zone_id: String,
    pub zone_name: String,
    pub frontend_asset_hostname: String,
    pub public_object_storage_hostname: String,
    /// Cache purge on `zone_id` only.
    pub purge_token: String,
    pub config_version: u64,
}

impl ProjectStorage {
    /// The account this project's objects live in: the owner's if they have
    /// connected one, the platform's otherwise.
    pub async fn resolve(db: &doc_db::Database, project_id: &str) -> anyhow::Result<Self> {
        match Self::resolve_if_connected(db, project_id).await? {
            Some(storage) => Ok(storage),
            None => anyhow::bail!(
                "project {project_id} has no Cloudflare account connected; \
                 run `forte cloudflare connect`"
            ),
        }
    }

    /// For the callers that have something to do either way. Tearing a project
    /// down is the case that matters: a project that was created and never
    /// connected owns no buckets, but it still owns a bundle, a database and
    /// its identity documents, and [`resolve`](Self::resolve) failing would
    /// leave those behind forever on a queue that redelivers.
    pub async fn resolve_if_connected(
        db: &doc_db::Database,
        project_id: &str,
    ) -> anyhow::Result<Option<Self>> {
        match (ProjectCloudflareConfigDocGet { project_id })
            .send_with(db)
            .await?
        {
            Some(config) => Self::from_config(config).await.map(Some),
            None => Ok(None),
        }
    }

    async fn from_config(config: ProjectCloudflareConfigDoc) -> anyhow::Result<Self> {
        let decrypt = |ciphertext: String, what: &'static str| async move {
            String::from_utf8(vault::decrypt(&ciphertext).await?)
                .map_err(|error| anyhow::anyhow!("{what} is not utf8: {error}"))
        };
        let worker_secret = decrypt(config.worker_secret_ciphertext, "worker secret").await?;
        let frontend_asset_secret = decrypt(
            config.frontend_asset_secret_ciphertext,
            "frontend asset secret",
        )
        .await?;
        let purge_token = decrypt(config.purge_token_ciphertext, "purge token").await?;

        Ok(Self {
            account_id: config.account_id,
            private_object_storage_bucket: config.private_object_storage_bucket,
            public_object_storage_bucket: config.public_object_storage_bucket,
            frontend_asset_bucket: config.frontend_asset_bucket,
            rendered_html_cache_bucket: config.rendered_html_cache_bucket,
            worker_keys: R2Keys {
                access_key_id: config.worker_access_key_id,
                secret_access_key: worker_secret,
            },
            frontend_asset_keys: R2Keys {
                access_key_id: config.frontend_asset_access_key_id,
                secret_access_key: frontend_asset_secret,
            },
            connection: Connection {
                zone_id: config.zone_id,
                zone_name: config.zone_name,
                frontend_asset_hostname: config.frontend_asset_hostname,
                public_object_storage_hostname: config.public_object_storage_hostname,
                purge_token,
                config_version: config.config_version,
            },
        })
    }

    /// A client that can purge this project's zone and nothing else. The
    /// Cloudflare API refuses this token for every other call, R2 included.
    pub fn purge_client(&self) -> CloudflareClient {
        CloudflareClient::for_zone(
            self.connection.purge_token.clone(),
            self.account_id.clone(),
            self.connection.zone_id.clone(),
        )
    }

    /// Where a project's deployed frontend assets are fetched from, which the
    /// CLI has to know before it builds because it is baked into the bundle.
    pub fn asset_base_url(&self, code_version: u64) -> String {
        format!(
            "https://{}/{code_version}/",
            self.connection.frontend_asset_hostname
        )
    }
}

/// How a stored configuration looks to a worker.
pub fn worker_storage_for(config: &ProjectCloudflareConfigDoc) -> WorkerProjectStorage {
    WorkerProjectStorage {
        account_id: config.account_id.clone(),
        region: "auto".to_string(),
        credential: WorkerR2Credential {
            access_key_id: config.worker_access_key_id.clone(),
            secret_ciphertext: config.worker_secret_ciphertext.clone(),
        },
        private_object_storage_bucket: config.private_object_storage_bucket.clone(),
        public_object_storage_bucket: config.public_object_storage_bucket.clone(),
        public_object_storage_base_url: format!(
            "https://{}",
            config.public_object_storage_hostname
        ),
        rendered_html_cache_bucket: config.rendered_html_cache_bucket.clone(),
        config_version: config.config_version,
    }
}

pub enum ConnectOutcome {
    Connected,
    AlreadyConnected {
        account_id: String,
        zone_name: String,
    },
}

/// Stores the configuration and points workers at it in one transaction.
///
/// Two writes that can fail independently would let control believe a project
/// is connected while the fleet still had no target for it, and the project
/// could not be reconnected to repair it because a configuration would already
/// exist. One transaction removes the state. It also settles two concurrent
/// connects, since the existence check happens inside it.
pub async fn connect_project(
    db: &doc_db::Database,
    config: ProjectCloudflareConfigDoc,
) -> anyhow::Result<ConnectOutcome> {
    let storage = worker_storage_for(&config);
    let result = db
        .trx(|trx| {
            let config = config.clone();
            let storage = storage.clone();
            async move {
                if let Some(existing) = trx
                    .get(ProjectCloudflareConfigDocGet {
                        project_id: config.project_id.as_str(),
                    })
                    .await?
                {
                    return trx.commit::<_, std::convert::Infallible>(
                        ConnectOutcome::AlreadyConnected {
                            account_id: existing.account_id.clone(),
                            zone_name: existing.zone_name.clone(),
                        },
                    );
                }

                let project_id = config.project_id.clone();
                trx.create(config)?;

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
                entry.storage = Some(storage);
                manifest.manifest_version += 1;
                trx.commit::<_, std::convert::Infallible>(ConnectOutcome::Connected)
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(outcome) => Ok(outcome),
        doc_db::TrxResult::Cancelled(cancel) => match cancel {},
        doc_db::TrxResult::Conflict(detail) => {
            anyhow::bail!("connect_project conflict: {detail:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

/// Mirrors a project's storage configuration into the worker manifest entry it
/// already has.
///
/// Passing `None` leaves the project with nowhere to store, which is what
/// tearing down wants. A project with no entry is left without one: teardown
/// removes the entry before it gets here, and creating one to blank its storage
/// resurrected the project as a `code_version: 0` route the worker registered
/// and could never serve.
pub async fn publish_storage_to_manifest(
    db: &doc_db::Database,
    project_id: &str,
    config: Option<&ProjectCloudflareConfigDoc>,
) -> anyhow::Result<()> {
    let storage = config.map(worker_storage_for);

    let result = db
        .trx(|trx| {
            let project_id = project_id.to_string();
            let storage = storage.clone();
            async move {
                let Some(mut manifest) = trx.get(WorkerManifestDocGet {}).await? else {
                    return trx.commit::<_, std::convert::Infallible>(());
                };
                if let Some(entry) = manifest.project_manifests.get_mut(&project_id)
                    && entry.storage != storage
                {
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
