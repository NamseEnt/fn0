use http_body_util::BodyExt;
use hyper::header::{AUTHORIZATION, HOST, HeaderValue};
use hyper::http::uri::Scheme;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use hyper_util::rt::TokioExecutor;
use std::sync::Arc;
use wasmtime_wasi_http::body::HyperOutgoingBody;
use wasmtime_wasi_http::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::types::{
    HostFutureIncomingResponse, IncomingResponse, OutgoingRequestConfig,
};

pub type Connector = hyper_rustls::HttpsConnector<HttpConnector>;
pub type HyperClient = Client<Connector, HyperOutgoingBody>;

#[derive(Clone)]
pub struct SharedHttpClient {
    inner: Arc<HyperClient>,
}

impl SharedHttpClient {
    pub fn new() -> Self {
        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        let client = Client::builder(TokioExecutor::new()).build(connector);
        Self {
            inner: Arc::new(client),
        }
    }

    pub fn from_client(client: Arc<HyperClient>) -> Self {
        Self { inner: client }
    }

    pub fn client(&self) -> Arc<HyperClient> {
        self.inner.clone()
    }
}

impl Default for SharedHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct TursoHijack {
    pub placeholder_host: String,
    pub target_host_suffix: String,
    pub group_token: String,
}

impl TursoHijack {
    pub fn target_host(&self, subdomain: &str) -> String {
        format!("{}{}", subdomain, self.target_host_suffix)
    }
}

#[derive(Clone)]
pub struct OutboundContext {
    pub shared_client: SharedHttpClient,
    pub turso_hijack: Option<Arc<TursoHijack>>,
    pub code_id: String,
}

pub fn send_request(
    ctx: OutboundContext,
    mut request: hyper::Request<HyperOutgoingBody>,
    config: OutgoingRequestConfig,
) -> HostFutureIncomingResponse {
    request.headers_mut().remove(HOST);

    let is_turso = ctx
        .turso_hijack
        .as_ref()
        .and_then(|h| request.uri().host().map(|host| host == h.placeholder_host))
        .unwrap_or(false);

    if is_turso {
        if let Err(err) = rewrite_turso_request(&mut request, &ctx) {
            let handle = wasmtime_wasi::runtime::spawn(async move { Ok(Err(err)) });
            return HostFutureIncomingResponse::pending(handle);
        }
    } else {
        ensure_scheme(&mut request, config.use_tls);
    }

    let client = ctx.shared_client.client();
    let between_bytes_timeout = config.between_bytes_timeout;
    let hijack = if is_turso {
        ctx.turso_hijack.clone()
    } else {
        None
    };
    let code_id = ctx.code_id.clone();

    let handle = wasmtime_wasi::runtime::spawn(async move {
        match client.request(request).await {
            Ok(resp) => {
                let resp = if hijack.is_some() {
                    crate::turso_tee::tee_response(resp, code_id)
                } else {
                    resp.map(|body| {
                        body.map_err(|err| ErrorCode::InternalError(Some(err.to_string())))
                            .boxed_unsync()
                    })
                };
                Ok(Ok(IncomingResponse {
                    resp,
                    worker: None,
                    between_bytes_timeout,
                }))
            }
            Err(err) => {
                tracing::warn!(%err, "shared http client request failed");
                Ok(Err(ErrorCode::InternalError(Some(err.to_string()))))
            }
        }
    });

    HostFutureIncomingResponse::pending(handle)
}

fn ensure_scheme(req: &mut hyper::Request<HyperOutgoingBody>, use_tls: bool) {
    let mut parts = req.uri().clone().into_parts();
    if parts.scheme.is_none() {
        parts.scheme = Some(if use_tls { Scheme::HTTPS } else { Scheme::HTTP });
    }
    if parts.path_and_query.is_none() {
        parts.path_and_query = Some("/".parse().unwrap());
    }
    if let Ok(uri) = hyper::Uri::from_parts(parts) {
        *req.uri_mut() = uri;
    }
}

fn rewrite_turso_request(
    req: &mut hyper::Request<HyperOutgoingBody>,
    ctx: &OutboundContext,
) -> Result<(), ErrorCode> {
    let hijack = ctx
        .turso_hijack
        .as_ref()
        .ok_or(ErrorCode::HttpRequestDenied)?;

    let subdomain = ctx
        .code_id
        .split("::")
        .next()
        .unwrap_or(&ctx.code_id)
        .to_string();

    let target_host = hijack.target_host(&subdomain);

    let path_and_query = req
        .uri()
        .path_and_query()
        .cloned()
        .unwrap_or_else(|| "/".parse().unwrap());

    let new_uri = hyper::Uri::builder()
        .scheme(Scheme::HTTPS)
        .authority(target_host.as_str())
        .path_and_query(path_and_query)
        .build()
        .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;

    *req.uri_mut() = new_uri;
    req.headers_mut().remove(HOST);
    req.headers_mut().insert(
        HOST,
        HeaderValue::from_str(&target_host).map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
    );

    let auth_value = format!("Bearer {}", hijack.group_token);
    req.headers_mut().insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value).map_err(|_| ErrorCode::HttpRequestDenied)?,
    );

    Ok(())
}
