//! Public-object hijack: turns the placeholder endpoint that `object_storage::public`
//! talks to into a signed request against the project's public-object bucket.
//!
//! The bucket is the project's own and carries its own CDN hostname, so a
//! guest's key is the whole key — the object it writes at `clips/intro.mp4` is
//! served from `https://{bucket}.{zone}/clips/intro.mp4`.
//!
//! The cache header is stamped here rather than accepted from the guest. An app
//! that could set its own `max-age` would put copies in browsers that no purge
//! can reach, which is exactly what makes a stable public URL unsafe to embed in
//! a cached page.

use crate::purge_gate::PurgeGate;
use std::sync::Arc;

use crate::object_storage_hijack::{
    ObjectStorageHijack, PRESIGN_MAX_EXPIRES_SECS, canonical_query_string, hex_encode, hmac_sha256,
    sha256_hex, signing_key, uri_encode_query,
};
use crate::storage_target::{
    PublicStorageResolver, PublicStorageTarget, R2Credentials, StaticResolver,
};
use bytes::Bytes;
use chrono::Utc;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::header::{AUTHORIZATION, CACHE_CONTROL, HOST, HeaderName, HeaderValue};
use hyper::http::uri::Scheme;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

type HijackRequest = hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>;

/// Browsers must revalidate on every request while the edge holds the object,
/// so an overwrite plus a purge is visible to returning visitors immediately.
/// `s-maxage` is long because purge, not expiry, is the correctness mechanism.
const PUBLIC_CACHE_CONTROL: &str = "public, max-age=0, s-maxage=31536000";

#[derive(Clone)]
pub struct PublicStorageHijack {
    pub placeholder_host: String,
    backend: Backend,
    control_project_id: String,
    purge_gate: Option<Arc<PurgeGate>>,
}

#[derive(Clone)]
enum Backend {
    R2 {
        resolver: Arc<dyn PublicStorageResolver>,
    },
    /// `forte dev`. Delegates to the object-storage hijack's filesystem store so
    /// the two local backends cannot drift apart.
    LocalFs {
        delegate: Box<ObjectStorageHijack>,
        base_url: String,
    },
}

pub struct PublicStorageConfig {
    pub placeholder_host: String,
    pub account_id: String,
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub public_base_url: String,
    pub control_project_id: String,
}

impl PublicStorageHijack {
    pub fn new(config: PublicStorageConfig) -> Self {
        let target = PublicStorageTarget {
            credentials: R2Credentials::for_account(
                &config.account_id,
                config.region,
                config.access_key_id,
                config.secret_access_key,
            ),
            bucket: config.bucket,
            base_url: config.public_base_url.trim_end_matches('/').to_string(),
        };
        Self::new_resolved(
            config.placeholder_host,
            Arc::new(StaticResolver::new(target)),
            config.control_project_id,
        )
    }

    /// Signs each project's requests against whatever bucket the resolver
    /// names, so one worker can serve projects whose public objects live in
    /// different R2 accounts and behind different CDN origins.
    pub fn new_resolved(
        placeholder_host: String,
        resolver: Arc<dyn PublicStorageResolver>,
        control_project_id: String,
    ) -> Self {
        Self {
            placeholder_host,
            backend: Backend::R2 { resolver },
            control_project_id,
            purge_gate: None,
        }
    }

    pub fn with_purge_gate(mut self, gate: Arc<PurgeGate>) -> Self {
        self.purge_gate = Some(gate);
        self
    }

    /// `true` when the project may spend one more invalidation this hour.
    pub(crate) fn allow_purge(&self, project_id: &str) -> bool {
        match &self.purge_gate {
            Some(gate) => gate.try_purge(project_id, chrono::Utc::now().timestamp() / 3600),
            None => true,
        }
    }

    /// Local store for `forte dev`. `dev_base_url` is where the dev server
    /// serves these objects, so `url()` keeps working without a CDN.
    pub fn new_local(
        placeholder_host: String,
        root: std::path::PathBuf,
        dev_base_url: String,
    ) -> Self {
        Self {
            placeholder_host,
            backend: Backend::LocalFs {
                delegate: Box::new(ObjectStorageHijack::new_local(
                    placeholder_host_for_local(),
                    root,
                    dev_base_url.clone(),
                )),
                base_url: format!(
                    "{}/__fn0_public_storage",
                    dev_base_url.trim_end_matches('/')
                ),
            },
            control_project_id: String::new(),
            purge_gate: None,
        }
    }

    /// Builds the production hijack from worker environment variables.
    pub fn from_env() -> Result<Self, String> {
        let var = |name: &str| std::env::var(name).map_err(|_| format!("{name} must be set"));
        let placeholder_host = std::env::var("FN0_PUBLIC_STORAGE_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-public-storage.fn0.dev".to_string());
        let region =
            std::env::var("FN0_PUBLIC_STORAGE_REGION").unwrap_or_else(|_| "auto".to_string());
        Ok(Self::new(PublicStorageConfig {
            placeholder_host,
            account_id: var("FN0_PUBLIC_STORAGE_ACCOUNT_ID")?,
            bucket: var("FN0_PUBLIC_STORAGE_BUCKET")?,
            region,
            access_key_id: var("FN0_PUBLIC_STORAGE_ACCESS_KEY_ID")?,
            secret_access_key: var("FN0_PUBLIC_STORAGE_SECRET_ACCESS_KEY")?,
            public_base_url: var("FN0_PUBLIC_STORAGE_CDN_ORIGIN")?,
            control_project_id: var("FN0_CONTROL_PROJECT_ID")?,
        }))
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    /// The base a guest builds public URLs from.
    ///
    /// `None` when the project has no public storage target, which is what a
    /// project that has not connected a Cloudflare account looks like.
    pub fn public_base_url_for(&self, project_id: &str) -> Option<String> {
        match &self.backend {
            Backend::R2 { resolver } => Some(resolver.resolve(project_id)?.base_url.clone()),
            Backend::LocalFs { base_url, .. } => Some(base_url.clone()),
        }
    }

    /// Reads from the local store for the `forte dev` public route.
    pub fn dev_read(&self, key: &str) -> crate::DevReadResult {
        match &self.backend {
            Backend::LocalFs { delegate, .. } => delegate.dev_read(key),
            Backend::R2 { .. } => crate::DevReadResult::NotLocal,
        }
    }

    /// Where a platform queue task for this write is addressed.
    pub(crate) fn control_project_id(&self) -> &str {
        &self.control_project_id
    }

    /// The public URL a guest request path resolves to, used to invalidate the
    /// edge copy after a write.
    pub(crate) fn public_url_for(&self, project_id: &str, raw_path: &str) -> Option<String> {
        let key = raw_path.trim_start_matches('/');
        Some(format!("{}/{key}", self.public_base_url_for(project_id)?))
    }

    /// Where the dev server publishes the local store, used to build `url()`.
    pub fn dev_base_url(&self) -> &str {
        match &self.backend {
            Backend::LocalFs { base_url, .. } => base_url,
            Backend::R2 { .. } => "",
        }
    }

    pub(crate) fn is_local(&self) -> bool {
        matches!(self.backend, Backend::LocalFs { .. })
    }

    /// Serves a request against the local filesystem store (`forte dev`).
    pub(crate) async fn serve_local(
        &self,
        req: HijackRequest,
    ) -> Result<hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
        let Backend::LocalFs { delegate, .. } = &self.backend else {
            return Err(ErrorCode::InternalError(Some(
                "serve_local called on R2 backend".to_string(),
            )));
        };
        delegate.serve_local(req).await
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    /// Mints a query-signed upload URL for the object at the request path.
    ///
    /// `cache-control` and `content-type` go into `SignedHeaders`, so R2 rejects
    /// an upload that does not send exactly these values. Letting the uploader
    /// pick its own `max-age` would seed browser copies that no purge can reach,
    /// which is the one failure this whole design refuses to allow.
    pub(crate) fn presign_put(
        &self,
        req: &HijackRequest,
        project_id: &str,
        content_type: &str,
        expires_secs: u64,
        content_length: Option<u64>,
    ) -> Result<String, ErrorCode> {
        let Backend::R2 { resolver } = &self.backend else {
            return Err(ErrorCode::InternalError(Some(
                "presign_put called on local backend".to_string(),
            )));
        };
        let target = resolve(resolver.as_ref(), project_id)?;
        let R2Credentials {
            endpoint_host,
            region,
            access_key_id,
            secret_access_key,
        } = &target.credentials;
        let canonical_uri = object_path(&target.bucket, req.uri().path());
        let expires = expires_secs.clamp(1, PRESIGN_MAX_EXPIRES_SECS);
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date = now.format("%Y%m%d").to_string();
        let credential_scope = format!("{date}/{region}/s3/aws4_request");
        let credential = format!("{access_key_id}/{credential_scope}");

        let mut signed_header_names = vec!["cache-control", "content-type", "host"];
        if content_length.is_some() {
            signed_header_names.push("content-length");
        }
        signed_header_names.sort_unstable();
        let signed_headers = signed_header_names.join(";");

        let mut canonical_header_lines = vec![
            format!("cache-control:{PUBLIC_CACHE_CONTROL}"),
            format!("content-type:{content_type}"),
            format!("host:{endpoint_host}"),
        ];
        if let Some(content_length) = content_length {
            canonical_header_lines.push(format!("content-length:{content_length}"));
        }
        canonical_header_lines.sort();
        let canonical_headers = format!("{}\n", canonical_header_lines.join("\n"));

        let mut params = [
            (
                "X-Amz-Algorithm".to_string(),
                "AWS4-HMAC-SHA256".to_string(),
            ),
            (
                "X-Amz-Credential".to_string(),
                uri_encode_query(&credential),
            ),
            ("X-Amz-Date".to_string(), amz_date.clone()),
            ("X-Amz-Expires".to_string(), expires.to_string()),
            (
                "X-Amz-SignedHeaders".to_string(),
                uri_encode_query(&signed_headers),
            ),
        ];
        params.sort();
        let canonical_query = params
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&");

        let payload_hash = "UNSIGNED-PAYLOAD";
        let canonical_request = format!(
            "PUT\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let key = signing_key(secret_access_key, &date, region, "s3");
        let signature = hex_encode(&hmac_sha256(&key, string_to_sign.as_bytes()));

        Ok(format!(
            "https://{endpoint_host}{canonical_uri}?{canonical_query}&X-Amz-Signature={signature}"
        ))
    }

    /// Rewrites an unsigned S3 request in place into a signed R2 request against
    /// the shared static bucket, stamping the platform's cache policy.
    pub(crate) fn sign(&self, req: &mut HijackRequest, project_id: &str) -> Result<(), ErrorCode> {
        let Backend::R2 { resolver } = &self.backend else {
            return Err(ErrorCode::InternalError(Some(
                "sign called on local backend".to_string(),
            )));
        };
        let target = resolve(resolver.as_ref(), project_id)?;
        let R2Credentials {
            endpoint_host,
            region,
            access_key_id,
            secret_access_key,
        } = &target.credentials;
        let method = req.method().to_string();
        let path_and_query = req
            .uri()
            .path_and_query()
            .cloned()
            .unwrap_or_else(|| "/".parse().unwrap());
        let query = path_and_query.query();
        let canonical_uri = object_path(&target.bucket, path_and_query.path());

        let is_write = matches!(req.method(), &hyper::Method::PUT | &hyper::Method::POST);
        if is_write {
            req.headers_mut().insert(
                CACHE_CONTROL,
                HeaderValue::from_static(PUBLIC_CACHE_CONTROL),
            );
        }

        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date = now.format("%Y%m%d").to_string();
        let payload_hash = "UNSIGNED-PAYLOAD";
        let canonical_query = canonical_query_string(query);

        let (signed_headers, canonical_headers) = if is_write {
            (
                "cache-control;host;x-amz-content-sha256;x-amz-date",
                format!(
                    "cache-control:{PUBLIC_CACHE_CONTROL}\nhost:{}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n",
                    endpoint_host
                ),
            )
        } else {
            (
                "host;x-amz-content-sha256;x-amz-date",
                format!(
                    "host:{}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n",
                    endpoint_host
                ),
            )
        };

        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let credential_scope = format!("{date}/{region}/s3/aws4_request");
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let key = signing_key(secret_access_key, &date, region, "s3");
        let signature = hex_encode(&hmac_sha256(&key, string_to_sign.as_bytes()));
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, \
             SignedHeaders={signed_headers}, Signature={signature}",
            access_key_id
        );

        let new_path_and_query = match query {
            Some(query) => format!("{canonical_uri}?{query}"),
            None => canonical_uri,
        };
        let new_uri = hyper::Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority(endpoint_host.as_str())
            .path_and_query(new_path_and_query.as_str())
            .build()
            .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;
        *req.uri_mut() = new_uri;

        let headers = req.headers_mut();
        headers.remove(HOST);
        headers.insert(
            HOST,
            HeaderValue::from_str(endpoint_host).map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
        );
        headers.insert(
            HeaderName::from_static("x-amz-date"),
            HeaderValue::from_str(&amz_date).map_err(|_| ErrorCode::HttpRequestDenied)?,
        );
        headers.insert(
            HeaderName::from_static("x-amz-content-sha256"),
            HeaderValue::from_static("UNSIGNED-PAYLOAD"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&authorization).map_err(|_| ErrorCode::HttpRequestDenied)?,
        );
        Ok(())
    }
}

fn resolve(
    resolver: &dyn PublicStorageResolver,
    project_id: &str,
) -> Result<Arc<PublicStorageTarget>, ErrorCode> {
    resolver.resolve(project_id).ok_or_else(|| {
        ErrorCode::InternalError(Some(format!(
            "no public storage target for project {project_id}"
        )))
    })
}

/// `forte dev` routes public objects through the same local store as private
/// objects, so the delegate needs a host it will never actually match on.
fn placeholder_host_for_local() -> String {
    "fn0-public-storage.local".to_string()
}

fn object_path(bucket: &str, raw_path: &str) -> String {
    let key = raw_path.trim_start_matches('/');
    if key.is_empty() {
        format!("/{bucket}")
    } else {
        format!("/{bucket}/{key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::{BodyExt, Full};

    #[test]
    fn presigned_put_pins_the_cache_policy_into_the_signature() {
        let hijack = hijack();
        let req = request(hyper::Method::GET, "https://host/clips/intro.mp4");
        let url = hijack
            .presign_put(&req, "proj1", "video/mp4", 300, None)
            .unwrap();

        // Both headers must be signed, or an uploader could pick a browser-
        // cacheable max-age that no purge could ever reach.
        assert!(url.contains("X-Amz-SignedHeaders=cache-control%3Bcontent-type%3Bhost"));
        assert!(url.contains("X-Amz-Signature="));
        assert!(url.contains("/fn0-proj1-public-object-storage/clips/intro.mp4?"));
    }

    #[test]
    fn presigned_put_expiry_is_clamped() {
        let hijack = hijack();
        let req = request(hyper::Method::GET, "https://host/clips/intro.mp4");
        let url = hijack
            .presign_put(&req, "proj1", "video/mp4", 86_400, None)
            .unwrap();
        assert!(url.contains(&format!("X-Amz-Expires={PRESIGN_MAX_EXPIRES_SECS}")));
    }

    fn hijack() -> PublicStorageHijack {
        PublicStorageHijack::new(PublicStorageConfig {
            placeholder_host: "fn0-public-storage.fn0.dev".to_string(),
            account_id: "acct".to_string(),
            bucket: "fn0-proj1-public-object-storage".to_string(),
            region: "auto".to_string(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            public_base_url: "https://fn0-proj1-public-object-storage.example".to_string(),
            control_project_id: "fn0-control".to_string(),
        })
    }

    fn request(method: hyper::Method, uri: &str) -> HijackRequest {
        hyper::Request::builder()
            .method(method)
            .uri(uri)
            .body(
                Full::new(Bytes::new())
                    .map_err(|_: std::convert::Infallible| unreachable!())
                    .boxed_unsync(),
            )
            .unwrap()
    }

    #[test]
    fn a_guests_key_is_the_whole_key() {
        let mut req = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/clips/a.mp4",
        );
        hijack().sign(&mut req, "proj1").unwrap();
        assert_eq!(
            req.uri().path(),
            "/fn0-proj1-public-object-storage/clips/a.mp4"
        );
        assert_eq!(req.uri().host(), Some("acct.r2.cloudflarestorage.com"));
    }

    #[test]
    fn stamps_cache_control_on_writes() {
        let mut req = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/a.txt",
        );
        hijack().sign(&mut req, "proj1").unwrap();
        assert_eq!(
            req.headers().get(CACHE_CONTROL).unwrap(),
            "public, max-age=0, s-maxage=31536000"
        );
    }

    #[test]
    fn guest_cannot_choose_its_own_cache_policy() {
        let mut req = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/a.txt",
        );
        req.headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static("max-age=31536000"));
        hijack().sign(&mut req, "proj1").unwrap();
        assert_eq!(
            req.headers().get(CACHE_CONTROL).unwrap(),
            "public, max-age=0, s-maxage=31536000"
        );
    }

    #[test]
    fn reads_carry_no_cache_header() {
        let mut req = request(
            hyper::Method::GET,
            "http://fn0-public-storage.fn0.dev/a.txt",
        );
        hijack().sign(&mut req, "proj1").unwrap();
        assert!(req.headers().get(CACHE_CONTROL).is_none());
    }

    #[test]
    fn dev_urls_carry_no_project_segment() {
        let hijack = PublicStorageHijack::new_local(
            "fn0-public-storage.fn0.dev".to_string(),
            std::path::PathBuf::from("/tmp/forte-public"),
            "http://localhost:3000".to_string(),
        );
        assert_eq!(
            hijack.public_url_for("app", "/clips/intro.mp4").unwrap(),
            "http://localhost:3000/__fn0_public_storage/clips/intro.mp4"
        );
    }

    #[test]
    fn purged_url_matches_the_url_handed_to_the_app() {
        let hijack = hijack();
        assert_eq!(
            hijack.public_url_for("proj1", "/clips/intro.mp4").unwrap(),
            "https://fn0-proj1-public-object-storage.example/clips/intro.mp4"
        );
    }

    #[test]
    fn public_base_url_is_the_projects_own_cdn_origin() {
        assert_eq!(
            hijack().public_base_url_for("proj1").unwrap(),
            "https://fn0-proj1-public-object-storage.example"
        );
    }

    #[test]
    fn each_project_signs_against_its_own_account_and_bucket() {
        struct PerProject;
        impl PublicStorageResolver for PerProject {
            fn resolve(&self, project_id: &str) -> Option<Arc<PublicStorageTarget>> {
                Some(Arc::new(PublicStorageTarget {
                    credentials: R2Credentials::for_account(
                        &format!("acct-{project_id}"),
                        "auto".to_string(),
                        format!("key-{project_id}"),
                        "secret".to_string(),
                    ),
                    bucket: format!("assets-{project_id}"),
                    base_url: format!("https://static.{project_id}.example"),
                }))
            }
        }
        let hijack = PublicStorageHijack::new_resolved(
            "fn0-public-storage.fn0.dev".to_string(),
            Arc::new(PerProject),
            "fn0-control".to_string(),
        );

        let mut first = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/a.txt",
        );
        hijack.sign(&mut first, "one").unwrap();
        assert_eq!(
            first.uri().host(),
            Some("acct-one.r2.cloudflarestorage.com")
        );
        assert_eq!(first.uri().path(), "/assets-one/a.txt");

        let mut second = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/a.txt",
        );
        hijack.sign(&mut second, "two").unwrap();
        assert_eq!(
            second.uri().host(),
            Some("acct-two.r2.cloudflarestorage.com")
        );
        assert_eq!(second.uri().path(), "/assets-two/a.txt");

        assert_eq!(
            hijack.public_base_url_for("two").unwrap(),
            "https://static.two.example"
        );
    }

    #[test]
    fn an_unconfigured_project_cannot_write_public_objects() {
        struct NoTarget;
        impl PublicStorageResolver for NoTarget {
            fn resolve(&self, _project_id: &str) -> Option<Arc<PublicStorageTarget>> {
                None
            }
        }
        let hijack = PublicStorageHijack::new_resolved(
            "fn0-public-storage.fn0.dev".to_string(),
            Arc::new(NoTarget),
            "fn0-control".to_string(),
        );
        let mut req = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/a.txt",
        );
        assert!(hijack.sign(&mut req, "proj1").is_err());
        assert!(hijack.public_base_url_for("proj1").is_none());
    }

    #[test]
    fn a_traversing_key_cannot_leave_the_projects_bucket() {
        let mut req = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/../fn0-proj2-public-object-storage/x",
        );
        hijack().sign(&mut req, "proj1").unwrap();
        assert!(
            req.uri()
                .path()
                .starts_with("/fn0-proj1-public-object-storage/")
        );
    }
}
