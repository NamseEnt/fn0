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
//! to three buckets, split across two credentials, and cache purge on one zone.

use crate::common::cloudflare::{CachePurgeFile, CloudflareClient};
use crate::common::r2_runtime::{RuntimeGrant, SessionCache};
use crate::common::vault;
use crate::docs::*;
use forte_sdk::*;

#[derive(Clone)]
pub struct R2Keys {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

#[derive(Clone)]
pub enum R2KeySource {
    Fixed(R2Keys),
    Runtime(SessionCache),
}

impl R2KeySource {
    pub fn new(
        broker_url: Option<&str>,
        project_id: &str,
        access_key_id: String,
        secret: String,
    ) -> anyhow::Result<Self> {
        Ok(match broker_url {
            Some(url) => Self::Runtime(RuntimeGrant::new(url, project_id, secret)?.storage()),
            None => Self::Fixed(R2Keys {
                access_key_id,
                secret_access_key: secret,
                session_token: None,
            }),
        })
    }

    pub async fn for_bucket(&self, bucket: &str, validity: i64) -> anyhow::Result<R2Keys> {
        match self {
            Self::Fixed(keys) => Ok(keys.clone()),
            Self::Runtime(provider) => {
                let session = provider.lease(bucket, validity).await?;
                Ok(R2Keys {
                    access_key_id: session.access_key_id,
                    secret_access_key: session.secret_access_key,
                    session_token: Some(session.session_token),
                })
            }
        }
    }
}

#[derive(Clone)]
pub enum ProjectPurge {
    Fixed(CloudflareClient),
    Runtime(RuntimeGrant),
}

impl ProjectPurge {
    pub fn new(
        broker_url: Option<&str>,
        project_id: &str,
        token: String,
        account_id: String,
        zone_id: String,
    ) -> anyhow::Result<Self> {
        Ok(match broker_url {
            Some(url) => Self::Runtime(RuntimeGrant::new(url, project_id, token)?),
            None => Self::Fixed(CloudflareClient::for_zone(token, account_id, zone_id)),
        })
    }

    pub async fn verify_token(&self) -> anyhow::Result<()> {
        match self {
            Self::Fixed(client) => client.verify_token().await.map(|_| ()),
            Self::Runtime(grant) => grant.verify_purge().await,
        }
    }

    pub async fn purge_cache_urls(&self, files: &[CachePurgeFile]) -> anyhow::Result<()> {
        match self {
            Self::Fixed(client) => client.purge_cache_urls(files).await,
            Self::Runtime(grant) => grant.purge_files(files).await,
        }
    }

    pub async fn purge_cache_tags(&self, tags: &[&str]) -> anyhow::Result<()> {
        match self {
            Self::Fixed(client) => client.purge_cache_tags(tags).await,
            Self::Runtime(grant) => grant.purge_tags(tags).await,
        }
    }
}

/// Everything an operation needs to reach one project's objects.
///
/// The two key sets are not interchangeable. `worker_keys` opens the buckets
/// holding user data; `frontend_asset_keys` opens
/// the frontend-asset bucket and nothing else. An operation takes the one that
/// matches what it is allowed to touch, so a wrong prefix in a delete loop
/// fails on a 403 rather than on user data.
pub struct ProjectStorage {
    pub account_id: String,
    pub private_object_storage_bucket: String,
    pub public_object_storage_bucket: String,
    pub frontend_asset_bucket: String,
    pub worker_keys: R2KeySource,
    pub frontend_asset_keys: R2KeySource,
    purge: ProjectPurge,
    pub connection: Connection,
}

pub struct Connection {
    pub zone_id: String,
    pub zone_name: String,
    pub frontend_asset_hostname: String,
    pub public_object_storage_hostname: String,
    /// Cache purge on `zone_id` only.
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
                 run `forte cloud init`"
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

        let broker_url = config.runtime_broker_url.as_deref();
        let purge = ProjectPurge::new(
            broker_url,
            &config.project_id,
            purge_token,
            config.account_id.clone(),
            config.zone_id.clone(),
        )?;
        Ok(Self {
            account_id: config.account_id,
            private_object_storage_bucket: config.private_object_storage_bucket,
            public_object_storage_bucket: config.public_object_storage_bucket,
            frontend_asset_bucket: config.frontend_asset_bucket,
            worker_keys: R2KeySource::new(
                broker_url,
                &config.project_id,
                config.worker_access_key_id,
                worker_secret,
            )?,
            frontend_asset_keys: R2KeySource::new(
                broker_url,
                &config.project_id,
                config.frontend_asset_access_key_id,
                frontend_asset_secret,
            )?,
            purge,
            connection: Connection {
                zone_id: config.zone_id,
                zone_name: config.zone_name,
                frontend_asset_hostname: config.frontend_asset_hostname,
                public_object_storage_hostname: config.public_object_storage_hostname,
                config_version: config.config_version,
            },
        })
    }

    pub fn purge_client(&self) -> ProjectPurge {
        self.purge.clone()
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
            broker_url: config.runtime_broker_url.clone(),
        },
        private_object_storage_bucket: config.private_object_storage_bucket.clone(),
        public_object_storage_bucket: config.public_object_storage_bucket.clone(),
        public_object_storage_base_url: format!(
            "https://{}",
            config.public_object_storage_hostname
        ),
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

/// Stores the configuration in one transaction.
///
/// The worker manifest is deliberately untouched: a manifest entry is only
/// written once the project has a registered domain, because that is the only
/// thing a worker routes on. `domain_set` refuses unconnected projects, so a
/// project that connects here and never registers a domain simply never
/// reaches the manifest. One write also settles two concurrent connects, since
/// the existence check happens inside the same transaction.
pub async fn connect_project(
    db: &doc_db::Database,
    config: ProjectCloudflareConfigDoc,
) -> anyhow::Result<ConnectOutcome> {
    let result = db
        .trx(|trx| {
            let config = config.clone();
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

                trx.create(config)?;
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
