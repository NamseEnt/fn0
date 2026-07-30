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
//! exits. What fn0 receives is what this module mints at the end: an R2 token
//! scoped to three buckets that cannot delete a bucket or call the REST API,
//! and a token that can purge one zone's cache and nothing else.

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

pub struct Provisioner {
    client: reqwest::Client,
    api_token: String,
    account_id: String,
    zone_id: String,
}

pub struct Provisioned {
    pub zone_name: String,
    pub static_hostname: String,
    pub object_bucket: String,
    pub asset_bucket: String,
    pub page_bucket: String,
    pub dataplane_access_key_id: String,
    /// SHA-256 of the minted data-plane token. The token value never leaves
    /// this process: the hash is the only form fn0 needs, and unlike the token
    /// it cannot be replayed against the REST API.
    pub dataplane_secret: String,
    pub purge_token: String,
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
    pub fn new(api_token: String, account_id: String, zone_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_token,
            account_id,
            zone_id,
        }
    }

    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<(reqwest::StatusCode, Envelope<T>)> {
        let mut request = self
            .client
            .request(method, format!("{API_BASE}{path}"))
            .bearer_auth(&self.api_token);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        let envelope: Envelope<T> = serde_json::from_str(&text)
            .with_context(|| format!("{path} returned {status}: {text}"))?;
        Ok((status, envelope))
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
            if let Ok((_, envelope)) = self.call::<Token>(reqwest::Method::GET, &path, None).await
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

    pub async fn zone_name(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct Zone {
            name: String,
        }
        let (_, envelope) = self
            .call::<Zone>(
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

    async fn create_bucket(&self, name: &str) -> Result<()> {
        let (status, envelope) = self
            .call::<serde_json::Value>(
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

    async fn put_cors(&self, bucket: &str) -> Result<()> {
        let (status, envelope) = self
            .call::<serde_json::Value>(
                reqwest::Method::PUT,
                &format!("/accounts/{}/r2/buckets/{bucket}/cors", self.account_id),
                Some(serde_json::json!({
                    "rules": [{
                        "allowed": {
                            "methods": ["GET", "PUT", "HEAD"],
                            "origins": ["*"],
                            "headers": ["*"],
                        },
                        "exposeHeaders": ["ETag"],
                        "maxAgeSeconds": 86400,
                    }],
                })),
            )
            .await?;
        if envelope.success {
            return Ok(());
        }
        Err(anyhow!(
            "could not set CORS on {bucket} ({status}): {}",
            describe(&envelope.errors)
        ))
    }

    async fn attach_custom_domain(&self, bucket: &str, hostname: &str) -> Result<()> {
        let (status, envelope) = self
            .call::<serde_json::Value>(
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

    /// Adds the cache rule the assets hostname needs, keeping every other rule
    /// in the zone.
    ///
    /// A `PUT` to a phase entrypoint replaces the whole rule list, so the
    /// existing rules are read back and carried through. `PURGE` has to be in
    /// the method match or the purge API answers `success: true` while the edge
    /// keeps serving the old object. `browser_ttl: respect_origin` is what stops
    /// a fresh zone's four-hour default Browser Cache TTL from overriding the
    /// `max-age=0` fn0 stores on public objects — four hours of browser copies
    /// is the one staleness no purge can reach.
    async fn ensure_cache_rule(&self, hostname: &str) -> Result<()> {
        const RULE_DESCRIPTION: &str = "fn0 static assets, public objects and cached pages";

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
            .call::<Ruleset>(reqwest::Method::GET, &path, None)
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
                    r#"(http.host eq "{hostname}" and http.request.method in {{"GET" "HEAD" "PURGE"}})"#
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

    async fn mint_token(&self, name: &str, policy: serde_json::Value) -> Result<(String, String)> {
        #[derive(Deserialize)]
        struct Minted {
            id: String,
            value: String,
        }
        let (status, envelope) = self
            .call::<Minted>(
                reqwest::Method::POST,
                "/user/tokens",
                Some(serde_json::json!({ "name": name, "policies": [policy] })),
            )
            .await?;
        let minted = envelope.result.filter(|_| envelope.success).ok_or_else(|| {
            anyhow!(
                "could not mint the {name} token ({status}). The token needs User -> API Tokens -> Edit. {}",
                describe(&envelope.errors)
            )
        })?;
        Ok((minted.id, minted.value))
    }

    /// Runs the whole flow and returns only what fn0 is allowed to see.
    pub async fn run(&self, project_id: &str) -> Result<Provisioned> {
        self.verify().await?;
        let zone_name = self.zone_name().await?;
        let static_hostname = format!("static.{zone_name}");

        let object_bucket = format!("fn0-object-storage-{project_id}");
        let asset_bucket = "fn0-static-asset".to_string();
        let page_bucket = "fn0-static-page".to_string();

        for bucket in [&object_bucket, &asset_bucket, &page_bucket] {
            self.create_bucket(bucket).await?;
        }
        self.put_cors(&object_bucket).await?;
        self.put_cors(&asset_bucket).await?;
        self.attach_custom_domain(&asset_bucket, &static_hostname)
            .await?;
        self.ensure_cache_rule(&static_hostname).await?;

        let resources: serde_json::Map<String, serde_json::Value> =
            [&object_bucket, &asset_bucket, &page_bucket]
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

        let (dataplane_access_key_id, dataplane_token) = self
            .mint_token(
                &format!("fn0 data plane ({project_id})"),
                serde_json::json!({
                    "effect": "allow",
                    "resources": resources,
                    "permission_groups": [
                        { "id": R2_BUCKET_ITEM_READ },
                        { "id": R2_BUCKET_ITEM_WRITE },
                    ],
                }),
            )
            .await?;

        let (_, purge_token) = self
            .mint_token(
                &format!("fn0 cache purge ({project_id})"),
                serde_json::json!({
                    "effect": "allow",
                    "resources": {
                        format!("com.cloudflare.api.account.zone.{}", self.zone_id): "*",
                    },
                    "permission_groups": [{ "id": CACHE_PURGE }],
                }),
            )
            .await?;

        Ok(Provisioned {
            zone_name,
            static_hostname,
            object_bucket,
            asset_bucket,
            page_bucket,
            dataplane_access_key_id,
            dataplane_secret: hex_sha256(&dataplane_token),
            purge_token,
        })
    }

    /// Signs an origin certificate for `hostname` through the zone owner's own
    /// Origin CA.
    ///
    /// The key pair is generated here and the private key is sent to fn0
    /// alongside the certificate, because the worker has to present it during
    /// the TLS handshake. The signing token is not: fn0 can serve the
    /// certificate it was given and cannot mint another.
    pub async fn issue_origin_certificate(&self, hostname: &str) -> Result<IssuedCertificate> {
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
        let certificate = envelope.result.filter(|_| envelope.success).ok_or_else(|| {
            anyhow!(
                "Cloudflare would not sign the origin certificate ({status}). The token needs Zone -> SSL and Certificates -> Edit. {}",
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

fn already_exists(errors: &[ApiError]) -> bool {
    errors.iter().any(|error| {
        let message = error.message.to_lowercase();
        message.contains("already exists")
            || message.contains("already configured")
            || message.contains("already in use")
            || message.contains("duplicate")
    })
}
