//! Provisioning a user's Cloudflare account, on the user's own machine.
//!
//! This is the half of bring-your-own-Cloudflare that deliberately does not run
//! on fn0's servers. Creating buckets, attaching a CDN hostname, writing a cache
//! rule and signing origin certificates all need an account-wide token, and a
//! token that can do those things can also delete every bucket in the account.
//! Measured, not assumed: an account-scoped `Workers R2 Storage Edit` token
//! reaches every bucket in the account over the S3 API, `DeleteBucket`
//! included.
//!
//! So the account-wide token stays here and is discarded when the command
//! exits. What fn0 receives is what this module mints at the end: two R2 tokens
//! that cannot delete a bucket or call the REST API, and a token that can purge
//! one zone's cache and nothing else.
//!
//! The two R2 tokens are split by who holds them. The worker token reaches the
//! project's objects, public objects and page cache, and is the only one
//! published to the fleet; the asset token reaches the deployed frontend and
//! stays in control. So a compromised worker cannot rewrite a deployed
//! frontend, and asset GC — which is the only thing that deletes on a schedule
//! — holds a credential that cannot open a bucket holding user data.
//!
//! There are two ways in, because the convenient one asks for a permission not
//! everyone should grant.
//!
//! [`Provisioner::run_managed`] takes a setup token carrying only
//! `User -> API Tokens -> Edit` and mints everything else from it, including
//! the short-lived token that does the provisioning. Cloudflare lets a token
//! grant permissions it does not hold, so one checkbox is enough; it also
//! refuses to let a minted token mint further tokens, so the provisioning token
//! cannot widen itself. Both measured against the live API. The catch is that a
//! token which can create tokens can create *any* token, so it is account-wide
//! however short its list looks.
//!
//! [`Provisioner::run_manual`] takes a token that can provision but cannot
//! create tokens, and mints nothing. The user makes the two long-lived
//! credentials themselves. It is more work and it never puts a token capable of
//! escalation on disk.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const API_BASE: &str = "https://api.cloudflare.com/client/v4";

// Permission group ids, read from `/user/tokens/permission_groups`. Cloudflare
// identifies these by uuid rather than by name in the tokens API.
const R2_BUCKET_ITEM_READ: &str = "6a018a9f2fc74eb6b293b0c548f38b39";
const R2_BUCKET_ITEM_WRITE: &str = "2efd5506f9c8494dacb1fa10a3e7d5b6";
const CACHE_PURGE: &str = "e17beae8b8cb423a99b1730f21238bed";

/// Origin CA's longest offered validity. Renewal would be a live-traffic event
/// on a hostname fn0 does not control the DNS for, so the fewer the better.
const CERTIFICATE_VALIDITY_DAYS: u32 = 5475;
const SECONDS_PER_DAY: i64 = 86_400;

/// rcgen generates an ECDSA P-256 key by default, which Origin CA signs under
/// `origin-ecc`. Asking for `origin-rsa` with an ECDSA CSR is rejected.
const CERTIFICATE_REQUEST_TYPE: &str = "origin-ecc";

// Permission groups the provisioning token is minted with. Setup is seconds of
// API calls; the expiry is generous by comparison and exists so that a crash
// between minting and revoking leaves nothing usable for long.
const PROVISIONING_TOKEN_MINUTES: i64 = 10;
const R2_STORAGE_WRITE: &str = "bf7481a1826f439697cb59a20b22293e";
const ZONE_READ: &str = "c8fed203ed3043cba015a93ad1616f1f";
const CACHE_SETTINGS_WRITE: &str = "9ff81cbbe65c400b97d92c3c1033cab6";
const ZONE_TRANSFORM_RULES_WRITE: &str = "0ac90a90249747bca6b047d97f0803e9";
const SSL_AND_CERTIFICATES_WRITE: &str = "c03055bc037c4ea9afb9a9f104b7b721";

/// One set of buckets per project. The two that are served publicly take their
/// hostname from their own name, so a bucket and the address it answers on are
/// the same string and cannot drift apart.
pub fn private_object_storage_bucket_name(project_id: &str) -> String {
    format!("fn0-{project_id}-private-object-storage")
}

pub fn public_object_storage_bucket_name(project_id: &str) -> String {
    format!("fn0-{project_id}-public-object-storage")
}

pub fn frontend_asset_bucket_name(project_id: &str) -> String {
    format!("fn0-{project_id}-frontend-asset")
}

pub fn rendered_html_cache_bucket_name(project_id: &str) -> String {
    format!("fn0-{project_id}-rendered-html-cache")
}

/// A zone the setup token can reach, and the account that owns it.
pub struct ReachableZone {
    pub zone_id: String,
    pub zone_name: String,
    pub account_id: String,
    pub account_name: String,
}

/// Reads which zones a token can reach, so nobody has to copy a hex id out of
/// the dashboard. Separate from [`Provisioner`], which cannot be built until an
/// account and a zone have been picked — which is what this is for.
pub struct ZoneDiscovery {
    client: reqwest::Client,
    setup_token: String,
}

impl ZoneDiscovery {
    pub fn new(setup_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            setup_token,
        }
    }

    /// `GET /zones` carries the owning account on every zone, so one call
    /// settles both ids.
    ///
    /// A setup token carries only `API Tokens -> Edit` and cannot read zones,
    /// so `mint_reader` has it mint one that can and revokes it afterwards. The
    /// minted policy scopes `com.cloudflare.api.account.*` — a wildcard the
    /// tokens API accepts, and the only form available here, since the account
    /// id is the thing being discovered. Measured against the live API.
    pub async fn list(&self, mint_reader: bool) -> Result<Vec<ReachableZone>> {
        if !mint_reader {
            return self.zones(&self.setup_token).await;
        }
        let expires_on = (chrono::Utc::now()
            + chrono::Duration::minutes(PROVISIONING_TOKEN_MINUTES))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
        let reader = mint_token(
            &self.client,
            &self.setup_token,
            "fn0 setup (zone discovery)",
            vec![serde_json::json!({
                "effect": "allow",
                "resources": { "com.cloudflare.api.account.*": "*" },
                "permission_groups": [{ "id": ZONE_READ }],
            })],
            Some(expires_on),
        )
        .await?;
        let result = self.zones(&reader.value).await;
        if let Err(error) = revoke_token(&self.client, &self.setup_token, &reader.id).await {
            eprintln!(
                "warning: could not revoke the zone discovery token: {error}. It expires by \
                 itself within {PROVISIONING_TOKEN_MINUTES} minutes."
            );
        }
        result
    }

    async fn zones(&self, token: &str) -> Result<Vec<ReachableZone>> {
        #[derive(Deserialize)]
        struct Zone {
            id: String,
            name: String,
            account: Account,
        }
        #[derive(Deserialize)]
        struct Account {
            id: String,
            #[serde(default)]
            name: String,
        }
        let (status, envelope) = call::<Vec<Zone>>(
            &self.client,
            token,
            reqwest::Method::GET,
            "/zones?per_page=200",
            None,
        )
        .await?;
        let zones = envelope.result.filter(|_| envelope.success).ok_or_else(|| {
            anyhow!(
                "could not list your zones ({status}). The token needs Zone -> Zone -> Read. {}",
                describe(&envelope.errors)
            )
        })?;
        Ok(zones
            .into_iter()
            .map(|zone| ReachableZone {
                zone_id: zone.id,
                zone_name: zone.name,
                account_id: zone.account.id,
                account_name: zone.account.name,
            })
            .collect())
    }
}

pub struct Provisioner {
    client: reqwest::Client,
    /// The user's setup token. Used only to mint and revoke; never to
    /// provision, because it is not granted the permissions to.
    setup_token: String,
    account_id: String,
    zone_id: String,
}

/// A token minted for the length of one command, and revoked at the end of it.
struct TemporaryToken {
    id: String,
    value: String,
}

/// What provisioning creates. Names only — no credentials.
pub struct ProvisionedResources {
    pub zone_name: String,
    pub frontend_asset_hostname: String,
    pub public_object_storage_hostname: String,
    pub private_object_storage_bucket: String,
    pub public_object_storage_bucket: String,
    pub frontend_asset_bucket: String,
    pub rendered_html_cache_bucket: String,
}

/// The three long-lived credentials fn0 is given.
pub struct ConnectCredentials {
    /// Reaches the two object-storage buckets and the rendered-HTML cache. The
    /// only R2 credential published to the worker fleet.
    pub worker_access_key_id: String,
    /// SHA-256 of the token, which is what R2 takes as an S3 secret access key.
    /// The token value never leaves the user's machine: the hash is the only
    /// form fn0 needs, and unlike the token it cannot be replayed against the
    /// REST API.
    pub worker_secret: String,
    /// Reaches the frontend-asset bucket only, and stays in control. Asset GC
    /// runs on a schedule and deletes; holding a credential that cannot open a
    /// bucket of user data is what keeps a bug in it from being unbounded.
    pub frontend_asset_access_key_id: String,
    pub frontend_asset_secret: String,
    pub purge_token: String,
}

/// The ids of the credentials the managed path minted, so a connect that fn0
/// refused does not leave them live on the user's account.
pub struct MintedCredentialIds {
    pub worker: String,
    pub frontend_asset: String,
    pub purge: String,
}

pub struct IssuedCertificate {
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub not_after_epoch_seconds: i64,
}

#[derive(Deserialize)]
struct Envelope<T> {
    success: bool,
    #[serde(default)]
    errors: Vec<ApiError>,
    result: Option<T>,
}

#[derive(Deserialize)]
struct ApiError {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    message: String,
}

async fn mint_token(
    client: &reqwest::Client,
    setup_token: &str,
    name: &str,
    policies: Vec<serde_json::Value>,
    expires_on: Option<String>,
) -> Result<TemporaryToken> {
    #[derive(Deserialize)]
    struct Minted {
        id: String,
        value: String,
    }
    let (status, envelope) = call::<Minted>(
        client,
        setup_token,
        reqwest::Method::POST,
        "/user/tokens",
        Some(match expires_on {
            Some(expires_on) => serde_json::json!({
                "name": name, "policies": policies, "expires_on": expires_on,
            }),
            None => serde_json::json!({ "name": name, "policies": policies }),
        }),
    )
    .await?;
    let minted = envelope.result.filter(|_| envelope.success).ok_or_else(|| {
        anyhow!(
            "could not mint the {name} token ({status}). The token needs User -> API Tokens -> Edit. {}",
            describe(&envelope.errors)
        )
    })?;
    Ok(TemporaryToken {
        id: minted.id,
        value: minted.value,
    })
}

async fn revoke_token(client: &reqwest::Client, setup_token: &str, id: &str) -> Result<()> {
    let (status, envelope) = call::<serde_json::Value>(
        client,
        setup_token,
        reqwest::Method::DELETE,
        &format!("/user/tokens/{id}"),
        None,
    )
    .await?;
    if envelope.success {
        return Ok(());
    }
    Err(anyhow!(
        "could not revoke token {id} ({status}): {}",
        describe(&envelope.errors)
    ))
}

async fn call<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    token: &str,
    method: reqwest::Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<(reqwest::StatusCode, Envelope<T>)> {
    let mut request = client
        .request(method, format!("{API_BASE}{path}"))
        .bearer_auth(token);
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await?;
    let status = response.status();
    let text = response.text().await?;
    let envelope: Envelope<T> =
        serde_json::from_str(&text).with_context(|| format!("{path} returned {status}: {text}"))?;
    Ok((status, envelope))
}

fn describe(errors: &[ApiError]) -> String {
    if errors.is_empty() {
        return "no detail".to_string();
    }
    errors
        .iter()
        .map(|error| format!("{} ({})", error.message, error.code))
        .collect::<Vec<_>>()
        .join("; ")
}

impl Provisioner {
    pub fn new(setup_token: String, account_id: String, zone_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            setup_token,
            account_id,
            zone_id,
        }
    }

    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        token: &str,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<(reqwest::StatusCode, Envelope<T>)> {
        call(&self.client, token, method, path, body).await
    }

    /// Confirms the token works and returns its id, which is also the S3 access
    /// key id if this token is ever used against R2 directly.
    pub async fn verify(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct Token {
            id: String,
            status: String,
        }
        // A token made under My Profile is user-owned; one made on the
        // account's own API Tokens page is account-owned, and each is only
        // accepted by its own endpoint. Onboarding documents the former.
        for path in [
            "/user/tokens/verify".to_string(),
            format!("/accounts/{}/tokens/verify", self.account_id),
        ] {
            if let Ok((_, envelope)) = self
                .call::<Token>(&self.setup_token, reqwest::Method::GET, &path, None)
                .await
                && let Some(token) = envelope.result.filter(|_| envelope.success)
            {
                if token.status != "active" {
                    return Err(anyhow!("the API token is {}, not active", token.status));
                }
                return Ok(token.id);
            }
        }
        Err(anyhow!(
            "Cloudflare rejected the API token. Check that it was copied whole and has not expired."
        ))
    }

    async fn zone_name(&self, token: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct Zone {
            name: String,
        }
        let (_, envelope) = self
            .call::<Zone>(
                token,
                reqwest::Method::GET,
                &format!("/zones/{}", self.zone_id),
                None,
            )
            .await?;
        envelope
            .result
            .filter(|_| envelope.success)
            .map(|zone| zone.name)
            .ok_or_else(|| {
                anyhow!(
                    "could not read the zone. The token needs Zone -> Zone -> Read on it. {}",
                    describe(&envelope.errors)
                )
            })
    }

    async fn create_bucket(&self, token: &str, name: &str) -> Result<()> {
        let (status, envelope) = self
            .call::<serde_json::Value>(
                token,
                reqwest::Method::POST,
                &format!("/accounts/{}/r2/buckets", self.account_id),
                Some(serde_json::json!({ "name": name })),
            )
            .await?;
        if envelope.success || already_exists(&envelope.errors) {
            return Ok(());
        }
        Err(anyhow!(
            "could not create bucket {name} ({status}). The token needs Account -> Workers R2 Storage -> Edit. {}",
            describe(&envelope.errors)
        ))
    }

    /// Bounds who may read these objects from a browser to the one origin the
    /// project answers on. `None` removes the configuration entirely, which is
    /// how "no origin may read this" has to be spelled: R2 rejects both an
    /// empty `origins` and an empty `rules` with `10001`, measured against the
    /// live API.
    async fn put_cors(&self, token: &str, bucket: &str, app_origin: Option<&str>) -> Result<()> {
        let path = format!("/accounts/{}/r2/buckets/{bucket}/cors", self.account_id);
        let (method, body) = match app_origin {
            Some(origin) => (
                reqwest::Method::PUT,
                Some(serde_json::json!({
                    "rules": [{
                        "allowed": {
                            "methods": ["GET", "PUT", "HEAD"],
                            "origins": [origin],
                            "headers": ["*"],
                        },
                        "exposeHeaders": ["ETag"],
                        "maxAgeSeconds": 86400,
                    }],
                })),
            ),
            None => (reqwest::Method::DELETE, None),
        };
        let (status, envelope) = self
            .call::<serde_json::Value>(token, method, &path, body)
            .await?;
        // Deleting a configuration that was never written is the state asked
        // for, not a failure.
        if envelope.success || (app_origin.is_none() && cors_absent(&envelope.errors)) {
            return Ok(());
        }
        Err(anyhow!(
            "could not set CORS on {bucket} ({status}): {}",
            describe(&envelope.errors)
        ))
    }

    async fn attach_custom_domain(&self, token: &str, bucket: &str, hostname: &str) -> Result<()> {
        if self.custom_domain_present(token, bucket, hostname).await {
            return Ok(());
        }
        let (status, envelope) = self
            .call::<serde_json::Value>(
                token,
                reqwest::Method::POST,
                &format!(
                    "/accounts/{}/r2/buckets/{bucket}/domains/custom",
                    self.account_id
                ),
                Some(serde_json::json!({
                    "domain": hostname,
                    "zoneId": self.zone_id,
                    "enabled": true,
                })),
            )
            .await?;
        if envelope.success || already_exists(&envelope.errors) {
            return Ok(());
        }
        Err(anyhow!(
            "could not point {hostname} at {bucket} ({status}): {}",
            describe(&envelope.errors)
        ))
    }

    /// Whether `hostname` is already attached to `bucket` specifically.
    ///
    /// Cloudflare answers "already in use" for a hostname attached to any
    /// bucket, and [`already_exists`] deliberately refuses to read that as
    /// success — pointed at someone else's bucket it would serve 404s. Asking
    /// this bucket first is what makes a retry after a partial provision
    /// distinguishable from that.
    async fn custom_domain_present(&self, token: &str, bucket: &str, hostname: &str) -> bool {
        #[derive(Deserialize)]
        struct Domain {
            domain: String,
        }
        #[derive(Deserialize)]
        struct Domains {
            #[serde(default)]
            domains: Vec<Domain>,
        }

        let Ok((_, envelope)) = self
            .call::<Domains>(
                token,
                reqwest::Method::GET,
                &format!(
                    "/accounts/{}/r2/buckets/{bucket}/domains/custom",
                    self.account_id
                ),
                None,
            )
            .await
        else {
            return false;
        };
        envelope
            .result
            .filter(|_| envelope.success)
            .is_some_and(|listed| {
                listed
                    .domains
                    .iter()
                    .any(|attached| attached.domain == hostname)
            })
    }

    /// Adds the cache rule fn0's public hostnames need, keeping every other rule
    /// in the zone.
    ///
    /// The match is a wildcard over the whole zone rather than one hostname, so
    /// this rule is written once and never grows: a free zone allows ten cache
    /// rules, which a rule per project would exhaust at ten projects. Both
    /// halves of the pattern are required — a bare `*-frontend-asset` would also
    /// match a hostname of the user's own and quietly pull it into fn0's caching
    /// policy.
    ///
    /// A `PUT` to a phase entrypoint replaces the whole rule list, so the
    /// existing rules are read back and carried through. `PURGE` has to be in
    /// the method match or the purge API answers `success: true` while the edge
    /// keeps serving the old object. `browser_ttl: respect_origin` is what stops
    /// a fresh zone's four-hour default Browser Cache TTL from overriding the
    /// `max-age=0` fn0 stores on public objects — four hours of browser copies
    /// is the one staleness no purge can reach.
    async fn ensure_cache_rule(&self, token: &str, zone_name: &str) -> Result<()> {
        const RULE_DESCRIPTION: &str = "fn0 frontend assets and public objects";

        #[derive(Deserialize)]
        struct Ruleset {
            #[serde(default)]
            rules: Vec<serde_json::Value>,
        }

        let path = format!(
            "/zones/{}/rulesets/phases/http_request_cache_settings/entrypoint",
            self.zone_id
        );
        let (status, envelope) = self
            .call::<Ruleset>(token, reqwest::Method::GET, &path, None)
            .await?;
        // A zone with no cache rules has no entrypoint ruleset at all, which
        // reads as 404 rather than an empty list.
        let mut rules = if envelope.success {
            envelope.result.map(|set| set.rules).unwrap_or_default()
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Vec::new()
        } else {
            return Err(anyhow!(
                "could not read the zone's cache rules ({status}). The token needs Zone -> Cache Rules -> Edit. {}",
                describe(&envelope.errors)
            ));
        };

        rules.retain(|rule| {
            rule.get("description").and_then(|value| value.as_str()) != Some(RULE_DESCRIPTION)
        });
        // Ahead of the zone's own rules: first match wins, and a broad user rule
        // that disabled caching would otherwise swallow the assets hostname.
        rules.insert(
            0,
            serde_json::json!({
                "action": "set_cache_settings",
                "expression": format!(
                    r#"((http.host wildcard "fn0-*-frontend-asset.{zone_name}" or http.host wildcard "fn0-*-public-object-storage.{zone_name}") and http.request.method in {{"GET" "HEAD" "PURGE"}})"#
                ),
                "description": RULE_DESCRIPTION,
                "action_parameters": {
                    "cache": true,
                    "browser_ttl": { "mode": "respect_origin" },
                },
            }),
        );

        let (status, envelope) = self
            .call::<serde_json::Value>(
                token,
                reqwest::Method::PUT,
                &path,
                Some(serde_json::json!({ "rules": rules })),
            )
            .await?;
        if envelope.success {
            return Ok(());
        }
        Err(anyhow!(
            "could not write the zone's cache rules ({status}). The token needs Zone -> Cache Rules -> Edit. {}",
            describe(&envelope.errors)
        ))
    }

    /// Drops the `Vary: Origin` R2 attaches to every CORS response on these
    /// hostnames.
    ///
    /// The header gives Cloudflare a separate cache entry per requesting
    /// origin, and purge by URL clears only the entry for a request that sent
    /// no `Origin` at all. A browser sends one, so every purge left the copy
    /// browsers actually read untouched — for the `s-maxage` of a year that fn0
    /// stores on public objects.
    ///
    /// Response header rules all run, in order, so this one goes last: a zone
    /// rule that adds `Vary` back has to be overridden, not overridden by.
    async fn ensure_vary_rule(&self, token: &str, zone_name: &str) -> Result<()> {
        const RULE_DESCRIPTION: &str = "fn0 single cache entry per public object";

        #[derive(Deserialize)]
        struct Ruleset {
            #[serde(default)]
            rules: Vec<serde_json::Value>,
        }

        let path = format!(
            "/zones/{}/rulesets/phases/http_response_headers_transform/entrypoint",
            self.zone_id
        );
        let (status, envelope) = self
            .call::<Ruleset>(token, reqwest::Method::GET, &path, None)
            .await?;
        // A zone with no response header rules has no entrypoint ruleset at
        // all, which reads as 404 rather than an empty list.
        let mut rules = if envelope.success {
            envelope.result.map(|set| set.rules).unwrap_or_default()
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Vec::new()
        } else {
            return Err(anyhow!(
                "could not read the zone's response header rules ({status}). The token needs Zone -> Transform Rules -> Edit. {}",
                describe(&envelope.errors)
            ));
        };

        rules.retain(|rule| {
            rule.get("description").and_then(|value| value.as_str()) != Some(RULE_DESCRIPTION)
        });
        rules.push(serde_json::json!({
            "action": "rewrite",
            "expression": format!(
                r#"(http.host wildcard "fn0-*-frontend-asset.{zone_name}" or http.host wildcard "fn0-*-public-object-storage.{zone_name}")"#
            ),
            "description": RULE_DESCRIPTION,
            "action_parameters": {
                "headers": {
                    "Vary": { "operation": "remove" },
                },
            },
        }));

        let (status, envelope) = self
            .call::<serde_json::Value>(
                token,
                reqwest::Method::PUT,
                &path,
                Some(serde_json::json!({ "rules": rules })),
            )
            .await?;
        if envelope.success {
            return Ok(());
        }
        Err(anyhow!(
            "could not write the zone's response header rules ({status}). The token needs Zone -> Transform Rules -> Edit. {}",
            describe(&envelope.errors)
        ))
    }

    async fn mint_with_expiry(
        &self,
        name: &str,
        policies: Vec<serde_json::Value>,
        expires_on: Option<String>,
    ) -> Result<(String, String)> {
        let minted =
            mint_token(&self.client, &self.setup_token, name, policies, expires_on).await?;
        Ok((minted.id, minted.value))
    }

    /// Mints a token that can do the provisioning, since the setup token
    /// itself is only allowed to create tokens.
    async fn mint_provisioning_token(&self, purpose: &str) -> Result<TemporaryToken> {
        let expires_on = (chrono::Utc::now()
            + chrono::Duration::minutes(PROVISIONING_TOKEN_MINUTES))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
        let (id, value) = self
            .mint_with_expiry(
                &format!("fn0 setup ({purpose})"),
                vec![
                    serde_json::json!({
                        "effect": "allow",
                        "resources": { format!("com.cloudflare.api.account.{}", self.account_id): "*" },
                        "permission_groups": [{ "id": R2_STORAGE_WRITE }],
                    }),
                    serde_json::json!({
                        "effect": "allow",
                        "resources": { format!("com.cloudflare.api.account.zone.{}", self.zone_id): "*" },
                        "permission_groups": [
                            { "id": ZONE_READ },
                            { "id": CACHE_SETTINGS_WRITE },
                            { "id": ZONE_TRANSFORM_RULES_WRITE },
                            { "id": SSL_AND_CERTIFICATES_WRITE },
                        ],
                    }),
                ],
                Some(expires_on),
            )
            .await?;
        Ok(TemporaryToken { id, value })
    }

    async fn revoke_token(&self, purpose: &str, id: &str) -> Result<()> {
        revoke_token(&self.client, &self.setup_token, id)
            .await
            .with_context(|| format!("the {purpose} token"))
    }

    /// The convenient path: one `API Tokens -> Edit` token, everything else
    /// minted here and the provisioning token revoked on the way out.
    pub async fn run_managed(
        &self,
        project_id: &str,
        app_origin: &str,
    ) -> Result<(
        ProvisionedResources,
        ConnectCredentials,
        MintedCredentialIds,
    )> {
        self.verify().await?;
        let provisioning = self.mint_provisioning_token(project_id).await?;
        let result = async {
            let resources = self
                .provision(&provisioning.value, project_id, app_origin)
                .await?;
            let (credentials, minted) = self.mint_credentials(project_id, &resources).await?;
            Ok((resources, credentials, minted))
        }
        .await;
        // Best effort, and reported rather than swallowed: the token expires on
        // its own, but a user who has to wait for that should know why.
        if let Err(error) = self.revoke_token("provisioning", &provisioning.id).await {
            eprintln!(
                "warning: {error}. It expires by itself within \
                 {PROVISIONING_TOKEN_MINUTES} minutes."
            );
        }
        result
    }

    /// The careful path: provision with the token as given, mint nothing. The
    /// caller's token is expected to be unable to create tokens, which is the
    /// whole reason to choose this.
    pub async fn run_manual(
        &self,
        project_id: &str,
        app_origin: &str,
    ) -> Result<ProvisionedResources> {
        self.verify().await?;
        self.provision(&self.setup_token, project_id, app_origin)
            .await
    }

    async fn provision(
        &self,
        token: &str,
        project_id: &str,
        app_origin: &str,
    ) -> Result<ProvisionedResources> {
        let zone_name = self.zone_name(token).await?;

        let private_object_storage_bucket = private_object_storage_bucket_name(project_id);
        let public_object_storage_bucket = public_object_storage_bucket_name(project_id);
        let frontend_asset_bucket = frontend_asset_bucket_name(project_id);
        let rendered_html_cache_bucket = rendered_html_cache_bucket_name(project_id);
        let frontend_asset_hostname = format!("{frontend_asset_bucket}.{zone_name}");
        let public_object_storage_hostname = format!("{public_object_storage_bucket}.{zone_name}");

        for bucket in [
            &private_object_storage_bucket,
            &public_object_storage_bucket,
            &frontend_asset_bucket,
            &rendered_html_cache_bucket,
        ] {
            self.create_bucket(token, bucket).await?;
        }
        for bucket in [
            &private_object_storage_bucket,
            &public_object_storage_bucket,
            &frontend_asset_bucket,
        ] {
            self.put_cors(token, bucket, Some(app_origin)).await?;
        }
        self.attach_custom_domain(token, &frontend_asset_bucket, &frontend_asset_hostname)
            .await?;
        self.attach_custom_domain(
            token,
            &public_object_storage_bucket,
            &public_object_storage_hostname,
        )
        .await?;
        self.ensure_cache_rule(token, &zone_name).await?;
        self.ensure_vary_rule(token, &zone_name).await?;

        Ok(ProvisionedResources {
            zone_name,
            frontend_asset_hostname,
            public_object_storage_hostname,
            private_object_storage_bucket,
            public_object_storage_bucket,
            frontend_asset_bucket,
            rendered_html_cache_bucket,
        })
    }

    fn bucket_scope(&self, buckets: &[&String]) -> serde_json::Value {
        let resources: serde_json::Map<String, serde_json::Value> = buckets
            .iter()
            .map(|bucket| {
                (
                    format!(
                        "com.cloudflare.edge.r2.bucket.{}_default_{bucket}",
                        self.account_id
                    ),
                    serde_json::Value::String("*".to_string()),
                )
            })
            .collect();
        serde_json::json!({
            "effect": "allow",
            "resources": resources,
            "permission_groups": [
                { "id": R2_BUCKET_ITEM_READ },
                { "id": R2_BUCKET_ITEM_WRITE },
            ],
        })
    }

    async fn mint_credentials(
        &self,
        project_id: &str,
        resources: &ProvisionedResources,
    ) -> Result<(ConnectCredentials, MintedCredentialIds)> {
        let (worker_access_key_id, worker_token) = self
            .mint_with_expiry(
                &format!("fn0 worker ({project_id})"),
                vec![self.bucket_scope(&[
                    &resources.private_object_storage_bucket,
                    &resources.public_object_storage_bucket,
                    &resources.rendered_html_cache_bucket,
                ])],
                None,
            )
            .await?;

        let (frontend_asset_access_key_id, frontend_asset_token) = self
            .mint_with_expiry(
                &format!("fn0 frontend assets ({project_id})"),
                vec![self.bucket_scope(&[&resources.frontend_asset_bucket])],
                None,
            )
            .await?;

        let (purge_token_id, purge_token) = self
            .mint_with_expiry(
                &format!("fn0 cache purge ({project_id})"),
                vec![serde_json::json!({
                    "effect": "allow",
                    "resources": {
                        format!("com.cloudflare.api.account.zone.{}", self.zone_id): "*",
                    },
                    "permission_groups": [{ "id": CACHE_PURGE }],
                })],
                None,
            )
            .await?;

        let minted = MintedCredentialIds {
            worker: worker_access_key_id.clone(),
            frontend_asset: frontend_asset_access_key_id.clone(),
            purge: purge_token_id,
        };
        Ok((
            ConnectCredentials {
                worker_access_key_id,
                worker_secret: hex_sha256(&worker_token),
                frontend_asset_access_key_id,
                frontend_asset_secret: hex_sha256(&frontend_asset_token),
                purge_token,
            },
            minted,
        ))
    }

    /// Best effort, and reported rather than swallowed: unlike the provisioning
    /// token these carry no expiry, so one left behind stays until the user
    /// finds it.
    pub async fn revoke_minted_credentials(&self, ids: &MintedCredentialIds) {
        for (purpose, id) in [
            ("worker", &ids.worker),
            ("frontend assets", &ids.frontend_asset),
            ("cache purge", &ids.purge),
        ] {
            if let Err(error) = self.revoke_token(purpose, id).await {
                eprintln!("warning: {error}. Delete it in the Cloudflare dashboard.");
            }
        }
    }

    /// Signs an origin certificate for `hostname` through the zone owner's own
    /// Origin CA.
    ///
    /// The key pair is generated here and the private key is sent to fn0
    /// alongside the certificate, because the worker has to present it during
    /// the TLS handshake. Nothing that can sign another one is: the token that
    /// did the signing is revoked before this returns.
    pub async fn issue_origin_certificate(
        &self,
        hostname: &str,
        mint_signing_token: bool,
    ) -> Result<IssuedCertificate> {
        self.verify().await?;
        if !mint_signing_token {
            return self
                .sign_origin_certificate(&self.setup_token, hostname)
                .await;
        }
        let signing = self.mint_provisioning_token(hostname).await?;
        let result = self.sign_origin_certificate(&signing.value, hostname).await;
        if let Err(error) = self.revoke_token("signing", &signing.id).await {
            eprintln!(
                "warning: {error}. It expires by itself within \
                 {PROVISIONING_TOKEN_MINUTES} minutes."
            );
        }
        result
    }

    /// Repoints the buckets' CORS at a domain the project has moved to.
    /// Provisioning writes the same rules for the domain the project starts
    /// with; this is how they follow it afterwards.
    pub async fn put_app_cors(
        &self,
        project_id: &str,
        app_origin: &str,
        mint_writing_token: bool,
    ) -> Result<()> {
        let buckets = [
            private_object_storage_bucket_name(project_id),
            public_object_storage_bucket_name(project_id),
            frontend_asset_bucket_name(project_id),
        ];
        if !mint_writing_token {
            for bucket in &buckets {
                self.put_cors(&self.setup_token, bucket, Some(app_origin))
                    .await?;
            }
            return Ok(());
        }
        let writing = self.mint_provisioning_token(app_origin).await?;
        let result = async {
            for bucket in &buckets {
                self.put_cors(&writing.value, bucket, Some(app_origin))
                    .await?;
            }
            Ok(())
        }
        .await;
        if let Err(error) = self.revoke_token("CORS", &writing.id).await {
            eprintln!(
                "warning: {error}. It expires by itself within \
                 {PROVISIONING_TOKEN_MINUTES} minutes."
            );
        }
        result
    }

    async fn sign_origin_certificate(
        &self,
        token: &str,
        hostname: &str,
    ) -> Result<IssuedCertificate> {
        #[derive(Serialize)]
        struct Body<'a> {
            csr: &'a str,
            hostnames: [&'a str; 1],
            request_type: &'a str,
            requested_validity: u32,
        }
        #[derive(Deserialize)]
        struct Certificate {
            certificate: String,
        }

        let key_pair = rcgen::KeyPair::generate()
            .map_err(|error| anyhow!("could not generate a key pair: {error}"))?;
        let mut params = rcgen::CertificateParams::new(vec![hostname.to_string()])
            .map_err(|error| anyhow!("could not build the certificate request: {error}"))?;
        params.distinguished_name = rcgen::DistinguishedName::new();
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, hostname);
        let csr_pem = params
            .serialize_request(&key_pair)
            .map_err(|error| anyhow!("could not sign the certificate request: {error}"))?
            .pem()
            .map_err(|error| anyhow!("could not encode the certificate request: {error}"))?;

        let (status, envelope) = self
            .call::<Certificate>(
                token,
                reqwest::Method::POST,
                "/certificates",
                Some(serde_json::to_value(Body {
                    csr: &csr_pem,
                    hostnames: [hostname],
                    request_type: CERTIFICATE_REQUEST_TYPE,
                    requested_validity: CERTIFICATE_VALIDITY_DAYS,
                })?),
            )
            .await?;
        let certificate = envelope
            .result
            .filter(|_| envelope.success)
            .ok_or_else(|| {
                anyhow!(
                    "Cloudflare would not sign the origin certificate ({status}): {}",
                    describe(&envelope.errors)
                )
            })?;

        Ok(IssuedCertificate {
            certificate_pem: certificate.certificate,
            private_key_pem: key_pair.serialize_pem(),
            // Derived rather than parsed out of the response: Cloudflare answers
            // with a Go-formatted timestamp, and the validity we asked for is
            // the same fact without a format to get wrong.
            not_after_epoch_seconds: chrono::Utc::now().timestamp()
                + i64::from(CERTIFICATE_VALIDITY_DAYS) * SECONDS_PER_DAY,
        })
    }
}

/// R2's S3 API takes the SHA-256 of the token value as the secret access key,
/// lowercase hex — the same string `printf '%s' <token> | sha256sum` produces.
fn hex_sha256(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Deliberately does not accept "already in use": Cloudflare answers that when a
/// hostname is attached to a *different* bucket, which is a failure to report
/// rather than a step to skip. Treating it as success ends in a hostname that
/// resolves and serves 404 for every object.
/// `10059 The CORS configuration does not exist.`, measured.
fn cors_absent(errors: &[ApiError]) -> bool {
    errors.iter().any(|error| error.code == 10059)
}

fn already_exists(errors: &[ApiError]) -> bool {
    errors.iter().any(|error| {
        let message = error.message.to_lowercase();
        message.contains("already exists")
            || message.contains("already configured")
            || message.contains("duplicate")
    })
}
