//! Object-storage hijack: turns the placeholder S3 endpoint that user code
//! talks to into either a signed request against Cloudflare R2 (`new_r2`,
//! production) or a local-filesystem store (`new_local`, `forte dev`).
//!
//! User code never sees the R2 host or credentials: it sends unsigned S3 REST
//! requests to the injected placeholder host, and `sign_r2` rewrites the host,
//! prepends the per-project bucket, and adds the SigV4 signature. The body is
//! never buffered — signing uses `UNSIGNED-PAYLOAD`, so large uploads stream
//! straight through.

use crate::presign_gate::PresignGate;
use bytes::Bytes;
use chrono::Utc;
use hmac::{Hmac, Mac};
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HOST, HeaderName, HeaderValue};
use hyper::http::uri::Scheme;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

type HmacSha256 = Hmac<Sha256>;
type HijackRequest = hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>;
type HijackResponse = hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>>;

const BUCKET_PREFIX: &str = "fn0-object-storage-";
const SIDECAR_SUFFIX: &str = ".__fn0meta";

// Expiry is the only revocation mechanism for a minted URL: once minting is
// blocked, exposure ends when outstanding URLs age out. Keep this in minutes.
pub const PRESIGN_MAX_EXPIRES_SECS: u64 = 300;

#[derive(Clone)]
pub struct ObjectStorageHijack {
    pub placeholder_host: String,
    backend: Backend,
    presign_gate: Option<Arc<PresignGate>>,
}

#[derive(Clone)]
enum Backend {
    R2 {
        endpoint_host: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
    },
    LocalFs {
        root: PathBuf,
        dev_base_url: String,
    },
}

/// Result of [`ObjectStorageHijack::dev_read`].
pub enum DevReadResult {
    Found {
        data: Vec<u8>,
        content_type: Option<String>,
    },
    NotFound,
    NotLocal,
}

impl ObjectStorageHijack {
    pub fn new_r2(
        placeholder_host: String,
        account_id: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Self {
        Self {
            placeholder_host,
            backend: Backend::R2 {
                endpoint_host: format!("{account_id}.r2.cloudflarestorage.com"),
                region,
                access_key_id,
                secret_access_key,
            },
            presign_gate: None,
        }
    }

    pub fn new_local(placeholder_host: String, root: PathBuf, dev_base_url: String) -> Self {
        Self {
            placeholder_host,
            backend: Backend::LocalFs { root, dev_base_url },
            presign_gate: None,
        }
    }

    pub fn with_presign_gate(mut self, gate: Arc<PresignGate>) -> Self {
        self.presign_gate = Some(gate);
        self
    }

    pub(crate) fn presign_gate(&self) -> Option<&Arc<PresignGate>> {
        self.presign_gate.as_ref()
    }

    /// Builds the production R2 hijack from worker environment variables.
    pub fn from_env() -> Result<Self, String> {
        let var = |name: &str| std::env::var(name).map_err(|_| format!("{name} must be set"));
        let placeholder_host = std::env::var("FN0_OBJECT_STORAGE_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-object-storage.fn0.dev".to_string());
        let region =
            std::env::var("FN0_OBJECT_STORAGE_REGION").unwrap_or_else(|_| "auto".to_string());
        Ok(Self::new_r2(
            placeholder_host,
            var("FN0_OBJECT_STORAGE_ACCOUNT_ID")?,
            region,
            var("FN0_OBJECT_STORAGE_ACCESS_KEY_ID")?,
            var("FN0_OBJECT_STORAGE_SECRET_ACCESS_KEY")?,
        ))
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    pub(crate) fn is_local(&self) -> bool {
        matches!(self.backend, Backend::LocalFs { .. })
    }

    /// Rewrites an unsigned S3 request in place into a signed R2 request.
    pub(crate) fn sign_r2(
        &self,
        req: &mut HijackRequest,
        project_id: &str,
    ) -> Result<(), ErrorCode> {
        let Backend::R2 {
            endpoint_host,
            region,
            access_key_id,
            secret_access_key,
        } = &self.backend
        else {
            return Err(ErrorCode::InternalError(Some(
                "sign_r2 called on local backend".to_string(),
            )));
        };

        let bucket = format!("{BUCKET_PREFIX}{project_id}");
        let method = req.method().to_string();
        let path_and_query = req
            .uri()
            .path_and_query()
            .cloned()
            .unwrap_or_else(|| "/".parse().unwrap());
        let raw_path = path_and_query.path();
        let query = path_and_query.query();

        let canonical_uri = if raw_path == "/" {
            format!("/{bucket}")
        } else {
            format!("/{bucket}{raw_path}")
        };

        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date = now.format("%Y%m%d").to_string();
        let payload_hash = "UNSIGNED-PAYLOAD";
        let canonical_query = canonical_query_string(query);
        let canonical_headers = format!(
            "host:{endpoint_host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
        );
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
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
            "AWS4-HMAC-SHA256 Credential={access_key_id}/{credential_scope}, \
             SignedHeaders={signed_headers}, Signature={signature}"
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

    /// Serves an S3 request against the local filesystem (`forte dev`).
    pub(crate) async fn serve_local(
        &self,
        req: HijackRequest,
    ) -> Result<HijackResponse, ErrorCode> {
        let Backend::LocalFs { root, .. } = &self.backend else {
            return Err(ErrorCode::InternalError(Some(
                "serve_local called on R2 backend".to_string(),
            )));
        };

        let (parts, body) = req.into_parts();
        let method = parts.method;
        let raw_path = parts.uri.path().to_string();
        let query = parts.uri.query().map(str::to_string);

        if method == hyper::Method::GET && raw_path == "/" {
            return local_list(root, query.as_deref());
        }

        let key = percent_decode(raw_path.trim_start_matches('/'));
        let Some(blob_path) = safe_path(root, &key) else {
            return synth(400, None, Bytes::from_static(b"invalid key"));
        };
        let meta_path = sidecar_path(&blob_path);

        match method {
            hyper::Method::GET => match std::fs::read(&blob_path) {
                Ok(data) => {
                    let content_type = std::fs::read_to_string(&meta_path).ok();
                    synth(200, content_type.as_deref(), Bytes::from(data))
                }
                Err(_) => synth(404, None, Bytes::new()),
            },
            hyper::Method::HEAD => match std::fs::metadata(&blob_path) {
                Ok(metadata) => {
                    let content_type = std::fs::read_to_string(&meta_path).ok();
                    let mut builder = hyper::Response::builder()
                        .status(200)
                        .header(CONTENT_LENGTH, metadata.len());
                    if let Some(content_type) = content_type {
                        builder = builder.header(CONTENT_TYPE, content_type);
                    }
                    builder
                        .body(empty_body())
                        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))
                }
                Err(_) => synth(404, None, Bytes::new()),
            },
            hyper::Method::PUT => {
                let data = body
                    .collect()
                    .await
                    .map_err(|e| ErrorCode::InternalError(Some(format!("{e:?}"))))?
                    .to_bytes();
                if let Some(parent) = blob_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
                }
                std::fs::write(&blob_path, &data)
                    .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
                match parts
                    .headers
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                {
                    Some(content_type) => {
                        std::fs::write(&meta_path, content_type)
                            .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
                    }
                    None => {
                        let _ = std::fs::remove_file(&meta_path);
                    }
                }
                synth(200, None, Bytes::new())
            }
            hyper::Method::DELETE => {
                let _ = std::fs::remove_file(&blob_path);
                let _ = std::fs::remove_file(&meta_path);
                synth(204, None, Bytes::new())
            }
            _ => synth(405, None, Bytes::new()),
        }
    }

    /// Mints a presigned URL for the object at the request path, without
    /// sending anything upstream. R2 returns a SigV4 query-signed URL; the
    /// local backend returns a `forte dev` server URL.
    pub(crate) fn presign(
        &self,
        req: &HijackRequest,
        project_id: &str,
        method: &str,
        expires_secs: u64,
        content_length: Option<u64>,
    ) -> Result<String, ErrorCode> {
        let raw_path = req.uri().path();
        match &self.backend {
            Backend::LocalFs { dev_base_url, .. } => Ok(format!(
                "{}/__fn0_object_storage{raw_path}",
                dev_base_url.trim_end_matches('/')
            )),
            Backend::R2 {
                endpoint_host,
                region,
                access_key_id,
                secret_access_key,
            } => {
                let bucket = format!("{BUCKET_PREFIX}{project_id}");
                let canonical_uri = if raw_path == "/" {
                    format!("/{bucket}")
                } else {
                    format!("/{bucket}{raw_path}")
                };
                let expires = expires_secs.clamp(1, PRESIGN_MAX_EXPIRES_SECS);
                let now = Utc::now();
                let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
                let date = now.format("%Y%m%d").to_string();
                let credential_scope = format!("{date}/{region}/s3/aws4_request");
                let credential = format!("{access_key_id}/{credential_scope}");

                let (signed_headers, canonical_headers) = match content_length {
                    Some(content_length) => (
                        "content-length;host".to_string(),
                        format!("content-length:{content_length}\nhost:{endpoint_host}\n"),
                    ),
                    None => ("host".to_string(), format!("host:{endpoint_host}\n")),
                };

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

                let canonical_request = format!(
                    "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\nUNSIGNED-PAYLOAD"
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
        }
    }

    /// dev-only: reads an object from the local store. The `forte dev` server
    /// calls this to serve presigned-URL downloads.
    pub fn dev_read(&self, key: &str) -> DevReadResult {
        let Backend::LocalFs { root, .. } = &self.backend else {
            return DevReadResult::NotLocal;
        };
        let Some(blob_path) = safe_path(root, key) else {
            return DevReadResult::NotFound;
        };
        match std::fs::read(&blob_path) {
            Ok(data) => DevReadResult::Found {
                content_type: std::fs::read_to_string(sidecar_path(&blob_path)).ok(),
                data,
            },
            Err(_) => DevReadResult::NotFound,
        }
    }

    /// dev-only: writes an object to the local store. The `forte dev` server
    /// calls this to accept presigned-URL uploads.
    pub fn dev_write(
        &self,
        key: &str,
        data: &[u8],
        content_type: Option<&str>,
    ) -> Result<(), String> {
        let Backend::LocalFs { root, .. } = &self.backend else {
            return Err("dev_write called on non-local backend".to_string());
        };
        let blob_path = safe_path(root, key).ok_or_else(|| "invalid key".to_string())?;
        if let Some(parent) = blob_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&blob_path, data).map_err(|e| e.to_string())?;
        let meta_path = sidecar_path(&blob_path);
        match content_type {
            Some(content_type) => {
                std::fs::write(&meta_path, content_type).map_err(|e| e.to_string())?;
            }
            None => {
                let _ = std::fs::remove_file(&meta_path);
            }
        }
        Ok(())
    }
}

fn local_list(root: &Path, query: Option<&str>) -> Result<HijackResponse, ErrorCode> {
    let params = parse_query(query);
    let prefix = params
        .get("prefix")
        .map(|p| percent_decode(p))
        .unwrap_or_default();
    let start_after = params.get("start-after").map(|s| percent_decode(s));
    let max_keys: usize = params
        .get("max-keys")
        .and_then(|m| m.parse().ok())
        .unwrap_or(1000);

    let mut all = Vec::new();
    collect_files(root, root, &mut all);
    all.sort_by(|a, b| a.0.cmp(&b.0));

    let mut selected = Vec::new();
    let mut truncated = false;
    for (key, size) in all {
        if !key.starts_with(&prefix) {
            continue;
        }
        if let Some(start_after) = &start_after
            && &key <= start_after
        {
            continue;
        }
        if selected.len() == max_keys {
            truncated = true;
            break;
        }
        selected.push((key, size));
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult>");
    xml.push_str(&format!("<IsTruncated>{truncated}</IsTruncated>"));
    for (key, size) in &selected {
        xml.push_str(&format!(
            "<Contents><Key>{}</Key><Size>{size}</Size></Contents>",
            xml_escape(key)
        ));
    }
    xml.push_str("</ListBucketResult>");
    synth(200, Some("application/xml"), Bytes::from(xml))
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, u64)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else if let Ok(relative) = path.strip_prefix(root) {
            let key = relative.to_string_lossy().replace('\\', "/");
            if key.ends_with(SIDECAR_SUFFIX) {
                continue;
            }
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            out.push((key, size));
        }
    }
}

fn safe_path(root: &Path, key: &str) -> Option<PathBuf> {
    if key.is_empty()
        || key
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == "..")
    {
        return None;
    }
    Some(root.join(key))
}

fn synth(
    status: u16,
    content_type: Option<&str>,
    body: Bytes,
) -> Result<HijackResponse, ErrorCode> {
    let mut builder = hyper::Response::builder()
        .status(status)
        .header(CONTENT_LENGTH, body.len());
    if let Some(content_type) = content_type {
        builder = builder.header(CONTENT_TYPE, content_type);
    }
    builder
        .body(
            Full::new(body)
                .map_err(|never: std::convert::Infallible| match never {})
                .boxed_unsync(),
        )
        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))
}

fn empty_body() -> UnsyncBoxBody<Bytes, ErrorCode> {
    Full::new(Bytes::new())
        .map_err(|never: std::convert::Infallible| match never {})
        .boxed_unsync()
}

fn parse_query(query: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(query) = query else {
        return map;
    };
    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        map.insert(key.to_string(), value.to_string());
    }
    map
}

fn canonical_query_string(query: Option<&str>) -> String {
    let Some(query) = query else {
        return String::new();
    };
    let mut params: Vec<(&str, &str)> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| pair.split_once('=').unwrap_or((pair, "")))
        .collect();
    params.sort_unstable();
    params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 3 <= bytes.len()
            && let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn sha256_hex(data: &[u8]) -> String {
    hex_encode(&Sha256::digest(data))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn sidecar_path(blob_path: &Path) -> PathBuf {
    blob_path.with_file_name(format!(
        "{}{SIDECAR_SUFFIX}",
        blob_path.file_name().and_then(|n| n.to_str()).unwrap_or("")
    ))
}

/// Percent-encodes a value for an S3 canonical query string (RFC 3986
/// unreserved set; `/` is encoded).
fn uri_encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}


