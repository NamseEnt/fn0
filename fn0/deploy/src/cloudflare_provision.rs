use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.cloudflare.com/client/v4";
const PROVISIONING_TOKEN_MINUTES: i64 = 10;
const ZONE_READ: &str = "c8fed203ed3043cba015a93ad1616f1f";

#[derive(Debug, Deserialize)]
pub struct ReachableZone {
    pub zone_id: String,
    pub zone_name: String,
    pub account_id: String,
    pub account_name: String,
}

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

    pub async fn list(&self) -> Result<Vec<ReachableZone>> {
        let expires_on = (chrono::Utc::now()
            + chrono::Duration::minutes(PROVISIONING_TOKEN_MINUTES))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
        let reader = mint_token(
            &self.client,
            &self.setup_token,
            "fn0 setup (zone discovery)",
            serde_json::json!({
                "effect": "allow",
                "resources": { "com.cloudflare.api.account.*": "*" },
                "permission_groups": [{ "id": ZONE_READ }],
            }),
            Some(expires_on),
        )
        .await?;
        let result = self.zones(&reader.value).await;
        if let Err(error) = revoke_token(&self.client, &self.setup_token, &reader.id).await {
            eprintln!(
                "warning: could not revoke the zone discovery token: {error}. It expires by itself within {PROVISIONING_TOKEN_MINUTES} minutes."
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

#[derive(Serialize)]
pub struct ProvisionedResources {
    pub zone_name: String,
    pub frontend_asset_hostname: String,
    pub public_object_storage_hostname: String,
    pub private_object_storage_bucket: String,
    pub public_object_storage_bucket: String,
    pub frontend_asset_bucket: String,
}

pub struct ConnectCredentials {
    pub worker_access_key_id: String,
    pub worker_secret: String,
    pub frontend_asset_access_key_id: String,
    pub frontend_asset_secret: String,
    pub purge_token: String,
}

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

struct TemporaryToken {
    id: String,
    value: String,
}

async fn mint_token(
    client: &reqwest::Client,
    setup_token: &str,
    name: &str,
    policy: serde_json::Value,
    expires_on: Option<String>,
) -> Result<TemporaryToken> {
    #[derive(Deserialize)]
    struct Minted {
        id: String,
        value: String,
    }

    let body = match expires_on {
        Some(expires_on) => serde_json::json!({
            "name": name,
            "policies": [policy],
            "expires_on": expires_on,
        }),
        None => serde_json::json!({ "name": name, "policies": [policy] }),
    };
    let (status, envelope) = call::<Minted>(
        client,
        setup_token,
        reqwest::Method::POST,
        "/user/tokens",
        Some(body),
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
