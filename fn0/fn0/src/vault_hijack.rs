use anyhow::Result;
use base64::Engine;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::http::uri::Scheme;
use oci_rust_sdk::auth::{RequestSigner, SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use serde_json::Value;
use std::sync::Arc;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

#[derive(Clone, Copy)]
enum Op {
    Decrypt,
    Encrypt,
    GenerateDek,
}

impl Op {
    fn from_path(path: &str) -> Option<Self> {
        match path {
            "/decrypt" => Some(Op::Decrypt),
            "/encrypt" => Some(Op::Encrypt),
            "/generate-dek" => Some(Op::GenerateDek),
            _ => None,
        }
    }

    fn oci_path(&self) -> &'static str {
        match self {
            Op::Decrypt => "/20180608/decrypt",
            Op::Encrypt => "/20180608/encrypt",
            Op::GenerateDek => "/20180608/generateDataEncryptionKey",
        }
    }
}

#[derive(Clone)]
pub struct VaultHijack {
    pub placeholder_host: String,
    crypto_host: String,
    allowed_subdomain: String,
    key_ocid: String,
    signer: Arc<RequestSigner>,
}

impl VaultHijack {
    pub fn new(
        placeholder_host: String,
        crypto_endpoint: String,
        allowed_subdomain: String,
        key_ocid: String,
        tenancy: String,
        user: String,
        fingerprint: String,
        private_key_pem: String,
    ) -> Result<Self> {
        let crypto_host = host_from_endpoint(&crypto_endpoint)?;

        let provider: Arc<SimpleAuthProvider> = Arc::new(
            SimpleAuthProvider::builder(SimpleAuthProviderRequiredFields {
                tenancy,
                user,
                fingerprint,
                private_key: private_key_pem,
            })
            .build(),
        );
        let signer = Arc::new(
            RequestSigner::new(provider as Arc<dyn oci_rust_sdk::auth::AuthProvider>)
                .map_err(|e| anyhow::anyhow!("vault signer init failed: {e:?}"))?,
        );

        Ok(Self {
            placeholder_host,
            crypto_host,
            allowed_subdomain,
            key_ocid,
            signer,
        })
    }

    pub fn from_env() -> Result<Self> {
        let crypto_endpoint = std::env::var("FN0_VAULT_CRYPTO_ENDPOINT")
            .map_err(|_| anyhow::anyhow!("FN0_VAULT_CRYPTO_ENDPOINT is required"))?;
        let placeholder_host = std::env::var("FN0_VAULT_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-vault.fn0.dev".to_string());
        let allowed_subdomain = std::env::var("FN0_VAULT_ALLOWED_SUBDOMAIN")
            .map_err(|_| anyhow::anyhow!("FN0_VAULT_ALLOWED_SUBDOMAIN is required"))?;
        let key_ocid = std::env::var("FN0_VAULT_KEY_OCID")
            .map_err(|_| anyhow::anyhow!("FN0_VAULT_KEY_OCID is required"))?;
        let tenancy = std::env::var("FN0_VAULT_OCI_TENANCY_ID")
            .map_err(|_| anyhow::anyhow!("FN0_VAULT_OCI_TENANCY_ID is required"))?;
        let user = std::env::var("FN0_VAULT_OCI_USER_ID")
            .map_err(|_| anyhow::anyhow!("FN0_VAULT_OCI_USER_ID is required"))?;
        let fingerprint = std::env::var("FN0_VAULT_OCI_FINGERPRINT")
            .map_err(|_| anyhow::anyhow!("FN0_VAULT_OCI_FINGERPRINT is required"))?;
        let private_key_b64 = std::env::var("FN0_VAULT_OCI_PRIVATE_KEY_BASE64")
            .map_err(|_| anyhow::anyhow!("FN0_VAULT_OCI_PRIVATE_KEY_BASE64 is required"))?;
        let private_key_pem = base64::engine::general_purpose::STANDARD
            .decode(private_key_b64.as_bytes())
            .map_err(|e| anyhow::anyhow!("vault private key base64 decode: {e}"))
            .and_then(|b| {
                String::from_utf8(b).map_err(|e| anyhow::anyhow!("vault private key utf8: {e}"))
            })?;

        Self::new(
            placeholder_host,
            crypto_endpoint,
            allowed_subdomain,
            key_ocid,
            tenancy,
            user,
            fingerprint,
            private_key_pem,
        )
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    pub(crate) fn build_signed_request(
        &self,
        subdomain: &str,
        method: &str,
        path: &str,
        body_bytes: &[u8],
    ) -> Result<hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
        if subdomain != self.allowed_subdomain {
            return Err(ErrorCode::HttpRequestDenied);
        }
        if method != hyper::Method::POST.as_str() {
            return Err(ErrorCode::HttpRequestMethodInvalid);
        }
        let op = Op::from_path(path).ok_or(ErrorCode::HttpRequestDenied)?;

        let oci_body = build_oci_body(op, &self.key_ocid, body_bytes)
            .map_err(|e| ErrorCode::InternalError(Some(format!("vault body build: {e}"))))?;

        let oci_path = op.oci_path();
        let url_str = format!("https://{}{}", self.crypto_host, oci_path);
        let url = url::Url::parse(&url_str)
            .map_err(|e| ErrorCode::InternalError(Some(format!("vault url parse: {e}"))))?;

        let mut headers = reqwest::header::HeaderMap::new();
        self.signer
            .sign_request(method, &url, &mut headers, Some(&oci_body))
            .map_err(|e| ErrorCode::InternalError(Some(format!("vault sign: {e:?}"))))?;

        let uri = hyper::Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority(self.crypto_host.as_str())
            .path_and_query(oci_path)
            .build()
            .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;

        let mut builder = hyper::Request::builder().method(method).uri(uri);
        for (name, value) in headers.iter() {
            builder = builder.header(name.as_str(), value);
        }

        let body: UnsyncBoxBody<Bytes, ErrorCode> =
            http_body_util::Full::new(Bytes::from(oci_body))
                .map_err(|never: std::convert::Infallible| match never {})
                .boxed_unsync();

        builder
            .body(body)
            .map_err(|e| ErrorCode::InternalError(Some(format!("vault request build: {e}"))))
    }
}

fn build_oci_body(op: Op, key_ocid: &str, wasm_body: &[u8]) -> Result<Vec<u8>> {
    let mut value: Value = if wasm_body.is_empty() {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_slice(wasm_body).map_err(|e| anyhow::anyhow!("vault body parse: {e}"))?
    };

    let obj = value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("vault body must be a JSON object"))?;

    obj.insert("keyId".into(), Value::String(key_ocid.to_string()));

    if let Op::GenerateDek = op {
        obj.entry("keyShape")
            .or_insert_with(|| serde_json::json!({ "algorithm": "AES", "length": 32 }));
        obj.entry("includePlaintextKey")
            .or_insert(Value::Bool(false));
    }

    Ok(serde_json::to_vec(&value)?)
}

fn host_from_endpoint(endpoint: &str) -> Result<String> {
    let url =
        url::Url::parse(endpoint).map_err(|e| anyhow::anyhow!("vault endpoint parse: {e}"))?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("vault endpoint missing host: {endpoint}"))?;
    if let Some(port) = url.port() {
        Ok(format!("{host}:{port}"))
    } else {
        Ok(host.to_string())
    }
}
