use forte_sdk::*;
use serde::{Deserialize, Serialize};

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

pub struct CloudflareClient {
    api_token: String,
    account_id: String,
    zone_id: Option<String>,
}

impl CloudflareClient {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_token: std::env::var("FN0_CLOUDFLARE_API_TOKEN")
                .map_err(|_| anyhow::anyhow!("FN0_CLOUDFLARE_API_TOKEN not set"))?,
            account_id: std::env::var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID not set"))?,
            zone_id: std::env::var("FN0_CLOUDFLARE_ZONE_ID").ok(),
        })
    }

    /// A client bound to a user's own account and zone, driven by their token.
    pub fn for_account(api_token: String, account_id: String, zone_id: String) -> Self {
        Self {
            api_token,
            account_id,
            zone_id: Some(zone_id),
        }
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    async fn call(
        &self,
        method: &str,
        path: &str,
        body: Vec<u8>,
    ) -> anyhow::Result<(u16, Vec<u8>)> {
        let url = format!("{CLOUDFLARE_API_BASE}{path}");
        let req = http::Request::builder()
            .uri(url)
            .method(method)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .body(body)?;
        let resp = http::Client::new().send(req).await?;
        let status = resp.status().as_u16();
        let body = resp.into_body().bytes().await.to_vec();
        Ok((status, body))
    }

    /// `location_hint` is `None` for a user's own account: their objects should
    /// land wherever Cloudflare places them for that account, not where the
    /// platform's own fleet happens to sit.
    pub async fn create_r2_bucket(
        &self,
        bucket_name: &str,
        location_hint: Option<&str>,
    ) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            name: &'a str,
            #[serde(rename = "locationHint", skip_serializing_if = "Option::is_none")]
            location_hint: Option<&'a str>,
        }
        let payload = serde_json::to_vec(&Body {
            name: bucket_name,
            location_hint,
        })?;
        let path = format!("/accounts/{}/r2/buckets", self.account_id);
        let (status, body) = self.call("POST", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        if response_indicates_already_exists(&body) {
            return Ok(());
        }
        anyhow::bail!(
            "create_r2_bucket {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn r2_bucket_exists(&self, bucket_name: &str) -> anyhow::Result<bool> {
        let path = format!("/accounts/{}/r2/buckets/{}", self.account_id, bucket_name);
        let (status, body) = self.call("GET", &path, Vec::new()).await?;
        if (200..300).contains(&status) {
            return Ok(true);
        }
        if status == 404 {
            return Ok(false);
        }
        anyhow::bail!(
            "r2_bucket_exists {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn delete_r2_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
        let path = format!("/accounts/{}/r2/buckets/{}", self.account_id, bucket_name);
        let (status, body) = self.call("DELETE", &path, Vec::new()).await?;
        if (200..300).contains(&status) || status == 404 {
            return Ok(());
        }
        anyhow::bail!(
            "delete_r2_bucket {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn put_r2_bucket_cors(
        &self,
        bucket_name: &str,
        methods: &[&str],
        allow_origin: &str,
        expose_headers: &[&str],
    ) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            rules: Vec<Rule<'a>>,
        }
        #[derive(Serialize)]
        struct Rule<'a> {
            allowed: Allowed<'a>,
            #[serde(rename = "exposeHeaders")]
            expose_headers: Vec<&'a str>,
            #[serde(rename = "maxAgeSeconds")]
            max_age_seconds: u32,
        }
        #[derive(Serialize)]
        struct Allowed<'a> {
            methods: Vec<&'a str>,
            origins: Vec<&'a str>,
            headers: Vec<&'a str>,
        }
        let payload = serde_json::to_vec(&Body {
            rules: vec![Rule {
                allowed: Allowed {
                    methods: methods.to_vec(),
                    origins: vec![allow_origin],
                    headers: vec!["*"],
                },
                expose_headers: expose_headers.to_vec(),
                max_age_seconds: 86400,
            }],
        })?;
        let path = format!(
            "/accounts/{}/r2/buckets/{}/cors",
            self.account_id, bucket_name
        );
        let (status, body) = self.call("PUT", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        anyhow::bail!(
            "put_r2_bucket_cors {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn list_r2_bucket_names(&self, name_contains: &str) -> anyhow::Result<Vec<String>> {
        #[derive(Deserialize)]
        struct Envelope {
            result: Option<ListResult>,
            #[serde(default)]
            result_info: Option<ListResultInfo>,
        }
        #[derive(Deserialize)]
        struct ListResult {
            buckets: Vec<Bucket>,
        }
        #[derive(Deserialize)]
        struct Bucket {
            name: String,
        }
        #[derive(Deserialize)]
        struct ListResultInfo {
            cursor: Option<String>,
        }

        const PER_PAGE: usize = 1000;
        let mut names = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut path = format!(
                "/accounts/{}/r2/buckets?name_contains={name_contains}&per_page={PER_PAGE}",
                self.account_id
            );
            if let Some(cursor) = &cursor {
                path.push_str(&format!("&cursor={cursor}"));
            }
            let (status, body) = self.call("GET", &path, Vec::new()).await?;
            if !(200..300).contains(&status) {
                anyhow::bail!(
                    "list_r2_bucket_names failed (status={status}): {}",
                    String::from_utf8_lossy(&body)
                );
            }
            let envelope: Envelope = serde_json::from_slice(&body)?;
            let buckets = envelope
                .result
                .ok_or_else(|| anyhow::anyhow!("list_r2_bucket_names: missing result"))?
                .buckets;
            let page_len = buckets.len();
            names.extend(buckets.into_iter().map(|b| b.name));
            if page_len < PER_PAGE {
                break;
            }
            match envelope.result_info.and_then(|i| i.cursor) {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        Ok(names)
    }

    pub async fn purge_cache_tags(&self, tags: &[&str]) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            tags: &'a [&'a str],
        }
        let payload = serde_json::to_vec(&Body { tags })?;
        let path = format!("/zones/{}/purge_cache", self.zone_id()?);
        let (status, body) = self.call("POST", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        anyhow::bail!(
            "purge_cache_tags failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    /// Purges by exact URL. Cloudflare accepts 100 per request against a far
    /// larger budget than the tag path (800/s vs 5/min account-wide), so this is
    /// the route for per-object invalidation.
    ///
    /// Requires the zone's Cache Rule to match `PURGE`; without it the API still
    /// answers `success: true` while the edge keeps serving the old object.
    pub async fn purge_cache_urls(&self, urls: &[String]) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            files: &'a [String],
        }
        let payload = serde_json::to_vec(&Body { files: urls })?;
        let path = format!("/zones/{}/purge_cache", self.zone_id()?);
        let (status, body) = self.call("POST", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        anyhow::bail!(
            "purge_cache_urls failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }
}

/// The Cloudflare API token id, which doubles as the R2 S3 access key id. The
/// matching secret is the SHA-256 of the token value, derived by the caller.
pub struct VerifiedToken {
    pub id: String,
}

impl CloudflareClient {
    pub async fn verify_token(&self) -> anyhow::Result<VerifiedToken> {
        #[derive(Deserialize)]
        struct Envelope {
            result: Option<TokenResult>,
        }
        #[derive(Deserialize)]
        struct TokenResult {
            id: String,
            status: String,
        }
        let path = format!("/accounts/{}/tokens/verify", self.account_id);
        let (status, body) = self.call("GET", &path, Vec::new()).await?;
        if !(200..300).contains(&status) {
            anyhow::bail!(
                "verify_token failed (status={status}): {}",
                String::from_utf8_lossy(&body)
            );
        }
        let result = serde_json::from_slice::<Envelope>(&body)?
            .result
            .ok_or_else(|| anyhow::anyhow!("verify_token: missing result"))?;
        if result.status != "active" {
            anyhow::bail!("token status is {}, expected active", result.status);
        }
        Ok(VerifiedToken { id: result.id })
    }

    pub async fn zone_name(&self) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Envelope {
            result: Option<ZoneResult>,
        }
        #[derive(Deserialize)]
        struct ZoneResult {
            name: String,
        }
        let path = format!("/zones/{}", self.zone_id()?);
        let (status, body) = self.call("GET", &path, Vec::new()).await?;
        if !(200..300).contains(&status) {
            anyhow::bail!(
                "zone_name failed (status={status}): {}",
                String::from_utf8_lossy(&body)
            );
        }
        Ok(serde_json::from_slice::<Envelope>(&body)?
            .result
            .ok_or_else(|| anyhow::anyhow!("zone_name: missing result"))?
            .name)
    }

    /// Attaches `hostname` to `bucket_name` so the bucket is publicly readable
    /// through the zone's CDN. Idempotent: an already-attached domain answers
    /// with an "already exists" error that is not a failure here.
    pub async fn create_r2_custom_domain(
        &self,
        bucket_name: &str,
        hostname: &str,
    ) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            domain: &'a str,
            #[serde(rename = "zoneId")]
            zone_id: &'a str,
            enabled: bool,
        }
        let zone_id = self.zone_id()?;
        let payload = serde_json::to_vec(&Body {
            domain: hostname,
            zone_id,
            enabled: true,
        })?;
        let path = format!(
            "/accounts/{}/r2/buckets/{bucket_name}/domains/custom",
            self.account_id
        );
        let (status, body) = self.call("POST", &path, payload).await?;
        if (200..300).contains(&status) || response_indicates_already_exists(&body) {
            return Ok(());
        }
        anyhow::bail!(
            "create_r2_custom_domain {hostname} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn delete_r2_custom_domain(
        &self,
        bucket_name: &str,
        hostname: &str,
    ) -> anyhow::Result<()> {
        let path = format!(
            "/accounts/{}/r2/buckets/{bucket_name}/domains/custom/{hostname}",
            self.account_id
        );
        let (status, body) = self.call("DELETE", &path, Vec::new()).await?;
        if (200..300).contains(&status) || status == 404 {
            return Ok(());
        }
        anyhow::bail!(
            "delete_r2_custom_domain {hostname} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    /// Signs `csr` through the zone owner's Cloudflare Origin CA.
    ///
    /// The certificate is trusted only by the Cloudflare edge, for the edge to
    /// origin leg, which is exactly the leg the worker terminates. We hold the
    /// private key and it never leaves the platform, so the user grants
    /// certificate issuance without handing over a key.
    pub async fn create_origin_ca_certificate(
        &self,
        csr: &str,
        hostnames: &[String],
        request_type: &str,
        requested_validity_days: u32,
    ) -> anyhow::Result<OriginCertificate> {
        #[derive(Serialize)]
        struct Body<'a> {
            csr: &'a str,
            hostnames: &'a [String],
            request_type: &'a str,
            requested_validity: u32,
        }
        #[derive(Deserialize)]
        struct Envelope {
            result: Option<CertificateResult>,
        }
        #[derive(Deserialize)]
        struct CertificateResult {
            id: String,
            certificate: String,
            expires_on: String,
        }
        let payload = serde_json::to_vec(&Body {
            csr,
            hostnames,
            request_type,
            requested_validity: requested_validity_days,
        })?;
        let (status, body) = self.call("POST", "/certificates", payload).await?;
        if !(200..300).contains(&status) {
            anyhow::bail!(
                "create_origin_ca_certificate failed (status={status}): {}",
                String::from_utf8_lossy(&body)
            );
        }
        let result = serde_json::from_slice::<Envelope>(&body)?
            .result
            .ok_or_else(|| anyhow::anyhow!("create_origin_ca_certificate: missing result"))?;
        Ok(OriginCertificate {
            id: result.id,
            certificate_pem: result.certificate,
            expires_on: result.expires_on,
        })
    }

    pub async fn revoke_origin_ca_certificate(&self, certificate_id: &str) -> anyhow::Result<()> {
        let path = format!("/certificates/{certificate_id}");
        let (status, body) = self.call("DELETE", &path, Vec::new()).await?;
        if (200..300).contains(&status) || status == 404 {
            return Ok(());
        }
        anyhow::bail!(
            "revoke_origin_ca_certificate failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    /// Creates the cache rule the static page and public object caches depend
    /// on, leaving the zone's other cache rules alone.
    ///
    /// `PURGE` has to be in the method match: without it the purge API still
    /// answers `success: true` while the edge keeps serving the old object,
    /// which is a silent year of stale bytes under `s-maxage=31536000` (#68).
    ///
    /// The phase entrypoint only accepts a whole rule list, and a `PUT` of one
    /// rule deletes every other rule in the zone. This is a user's own zone and
    /// this runs on every deploy, so the existing rules are read back and
    /// carried through untouched; ours is matched by description so a re-run
    /// replaces it instead of stacking duplicates.
    /// The zone's current cache rules, as opaque values so the ones fn0 does
    /// not own survive a round trip untouched.
    ///
    /// Doubles as the connect-time probe for `Zone -> Cache Rules`: a token
    /// without it answers 403 here rather than failing later inside a deploy.
    pub async fn read_cache_rules(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        #[derive(Deserialize)]
        struct Envelope {
            result: Option<RulesetResult>,
        }
        #[derive(Deserialize)]
        struct RulesetResult {
            #[serde(default)]
            rules: Vec<serde_json::Value>,
        }

        let path = format!(
            "/zones/{}/rulesets/phases/http_request_cache_settings/entrypoint",
            self.zone_id()?
        );
        let (status, body) = self.call("GET", &path, Vec::new()).await?;
        if (200..300).contains(&status) {
            return Ok(serde_json::from_slice::<Envelope>(&body)?
                .result
                .map(|ruleset| ruleset.rules)
                .unwrap_or_default());
        }
        // A zone with no cache rules yet has no entrypoint ruleset at all,
        // which reads as 404 rather than an empty list.
        if status == 404 {
            return Ok(Vec::new());
        }
        anyhow::bail!(
            "read_cache_rules failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn ensure_cache_rule(&self, hostname: &str) -> anyhow::Result<()> {
        const RULE_DESCRIPTION: &str = "fn0 static assets, public objects and cached pages";

        #[derive(Serialize)]
        struct Body {
            rules: Vec<serde_json::Value>,
        }

        let zone_id = self.zone_id()?;
        let path =
            format!("/zones/{zone_id}/rulesets/phases/http_request_cache_settings/entrypoint");

        let mut rules = self.read_cache_rules().await?;

        rules.retain(|rule| {
            rule.get("description").and_then(|value| value.as_str()) != Some(RULE_DESCRIPTION)
        });
        // Ahead of the zone's own rules: the first match wins in a ruleset, and
        // a broad user rule that disabled caching would otherwise swallow the
        // asset hostname.
        rules.insert(
            0,
            serde_json::json!({
                "action": "set_cache_settings",
                "expression": format!(
                    r#"(http.host eq "{hostname}" and http.request.method in {{"GET" "HEAD" "PURGE"}})"#
                ),
                "description": RULE_DESCRIPTION,
                "action_parameters": { "cache": true },
            }),
        );

        let payload = serde_json::to_vec(&Body { rules })?;
        let (status, body) = self.call("PUT", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        anyhow::bail!(
            "ensure_cache_rule failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    fn zone_id(&self) -> anyhow::Result<&str> {
        self.zone_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("no Cloudflare zone id for this client"))
    }
}

pub struct OriginCertificate {
    pub id: String,
    pub certificate_pem: String,
    pub expires_on: String,
}

#[derive(Deserialize)]
struct CloudflareEnvelope {
    #[serde(default)]
    errors: Vec<CloudflareError>,
}

#[derive(Deserialize)]
struct CloudflareError {
    #[serde(default)]
    message: String,
}

fn response_indicates_already_exists(body: &[u8]) -> bool {
    let Ok(env) = serde_json::from_slice::<CloudflareEnvelope>(body) else {
        return false;
    };
    env.errors.iter().any(|e| {
        let m = e.message.to_lowercase();
        m.contains("already exists")
            || m.contains("already configured")
            || m.contains("already in use")
            || m.contains("duplicate")
    })
}
