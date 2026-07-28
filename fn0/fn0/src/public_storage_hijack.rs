//! Public-object hijack: turns the placeholder endpoint that `object_storage::public`
//! talks to into a signed request against the shared static-asset bucket.
//!
//! Unlike [`crate::object_storage_hijack`], every project shares one bucket here
//! and is separated by a `{project_id}/public/` key prefix, because the bucket's
//! custom domain is what serves the objects and a domain per project would
//! exhaust the zone's DNS records.
//!
//! The cache header is stamped here rather than accepted from the guest. An app
//! that could set its own `max-age` would put copies in browsers that no purge
//! can reach, which is exactly what makes a stable public URL unsafe to embed in
//! a cached page.

use crate::object_storage_hijack::{
    canonical_query_string, hex_encode, hmac_sha256, sha256_hex, signing_key,
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

const KEY_PREFIX: &str = "public";

#[derive(Clone)]
pub struct PublicStorageHijack {
    pub placeholder_host: String,
    endpoint_host: String,
    bucket: String,
    region: String,
    access_key_id: String,
    secret_access_key: String,
    public_base_url: String,
    control_project_id: String,
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
        Self {
            placeholder_host: config.placeholder_host,
            endpoint_host: format!("{}.r2.cloudflarestorage.com", config.account_id),
            bucket: config.bucket,
            region: config.region,
            access_key_id: config.access_key_id,
            secret_access_key: config.secret_access_key,
            public_base_url: config.public_base_url.trim_end_matches('/').to_string(),
            control_project_id: config.control_project_id,
        }
    }

    /// Builds the production hijack from worker environment variables.
    pub fn from_env() -> Result<Self, String> {
        let var = |name: &str| std::env::var(name).map_err(|_| format!("{name} must be set"));
        let placeholder_host = std::env::var("FN0_PUBLIC_STORAGE_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-public-storage.fn0.dev".to_string());
        let region =
            std::env::var("FN0_PUBLIC_STORAGE_REGION").unwrap_or_else(|_| "auto".to_string());
        // Deliberately not the worker's `FN0_STATIC_ASSET_STORAGE_*`: those name
        // the private `fn0-static-page-*` bucket that holds cached HTML. Public
        // objects belong in the bucket that carries the CDN custom domain.
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

    /// The base a guest builds public URLs from, already scoped to the project.
    pub fn public_base_url_for(&self, project_id: &str) -> String {
        format!("{}/{project_id}/{KEY_PREFIX}", self.public_base_url)
    }

    /// Where a platform queue task for this write is addressed.
    pub(crate) fn control_project_id(&self) -> &str {
        &self.control_project_id
    }

    /// The public URL a guest request path resolves to, used to invalidate the
    /// edge copy after a write.
    pub(crate) fn public_url_for(&self, project_id: &str, raw_path: &str) -> String {
        let key = raw_path.trim_start_matches('/');
        format!("{}/{key}", self.public_base_url_for(project_id))
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    /// The object key a guest request lands on, namespaced to the project.
    fn object_path(&self, project_id: &str, raw_path: &str) -> String {
        let key = raw_path.trim_start_matches('/');
        if key.is_empty() {
            format!("/{}/{project_id}/{KEY_PREFIX}", self.bucket)
        } else {
            format!("/{}/{project_id}/{KEY_PREFIX}/{key}", self.bucket)
        }
    }

    /// Rewrites an unsigned S3 request in place into a signed R2 request against
    /// the shared static bucket, stamping the platform's cache policy.
    pub(crate) fn sign(&self, req: &mut HijackRequest, project_id: &str) -> Result<(), ErrorCode> {
        let method = req.method().to_string();
        let path_and_query = req
            .uri()
            .path_and_query()
            .cloned()
            .unwrap_or_else(|| "/".parse().unwrap());
        let query = path_and_query.query();
        let canonical_uri = self.object_path(project_id, path_and_query.path());

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
                    self.endpoint_host
                ),
            )
        } else {
            (
                "host;x-amz-content-sha256;x-amz-date",
                format!(
                    "host:{}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n",
                    self.endpoint_host
                ),
            )
        };

        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let credential_scope = format!("{date}/{}/s3/aws4_request", self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let key = signing_key(&self.secret_access_key, &date, &self.region, "s3");
        let signature = hex_encode(&hmac_sha256(&key, string_to_sign.as_bytes()));
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, \
             SignedHeaders={signed_headers}, Signature={signature}",
            self.access_key_id
        );

        let new_path_and_query = match query {
            Some(query) => format!("{canonical_uri}?{query}"),
            None => canonical_uri,
        };
        let new_uri = hyper::Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority(self.endpoint_host.as_str())
            .path_and_query(new_path_and_query.as_str())
            .build()
            .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;
        *req.uri_mut() = new_uri;

        let headers = req.headers_mut();
        headers.remove(HOST);
        headers.insert(
            HOST,
            HeaderValue::from_str(&self.endpoint_host)
                .map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::{BodyExt, Full};

    fn hijack() -> PublicStorageHijack {
        PublicStorageHijack::new(PublicStorageConfig {
            placeholder_host: "fn0-public-storage.fn0.dev".to_string(),
            account_id: "acct".to_string(),
            bucket: "fn0-static-asset".to_string(),
            region: "auto".to_string(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            public_base_url: "https://static.fn0.dev".to_string(),
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
    fn namespaces_keys_under_the_project() {
        let mut req = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/clips/a.mp4",
        );
        hijack().sign(&mut req, "proj1").unwrap();
        assert_eq!(
            req.uri().path(),
            "/fn0-static-asset/proj1/public/clips/a.mp4"
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
    fn purged_url_matches_the_url_handed_to_the_app() {
        let hijack = hijack();
        assert_eq!(
            hijack.public_url_for("proj1", "/clips/intro.mp4"),
            "https://static.fn0.dev/proj1/public/clips/intro.mp4"
        );
    }

    #[test]
    fn public_base_url_is_scoped_to_the_project() {
        assert_eq!(
            hijack().public_base_url_for("proj1"),
            "https://static.fn0.dev/proj1/public"
        );
    }

    #[test]
    fn one_project_cannot_reach_another_via_traversal() {
        let mut req = request(
            hyper::Method::PUT,
            "http://fn0-public-storage.fn0.dev/../proj2/public/x",
        );
        hijack().sign(&mut req, "proj1").unwrap();
        assert!(
            req.uri()
                .path()
                .starts_with("/fn0-static-asset/proj1/public/")
        );
    }
}
