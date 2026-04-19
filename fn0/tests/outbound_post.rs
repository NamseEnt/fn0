//! Integration test: wasm → WasiHttpView::send_request override → SharedHttpClient
//! → local HTTPS (h2) server → response back to wasm.
//!
//! Goal: prove that a POST with a non-trivial body round-trips end-to-end through
//! the exact production code path, against a locally-controlled TLS server, so the
//! test is fully deterministic and does not depend on external infrastructure.

use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::{Empty, Full};
use hyper::server::conn::http2 as server_http2;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

const TEST_WASM_BYTES: &[u8] = include_bytes!(
    "../test-wasm/outbound-post/target/wasm32-wasip2/release/outbound_post.wasm"
);

#[derive(Clone)]
struct InMemCache {
    bytes: Bytes,
}

impl InMemCache {
    fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }
}

impl<T, E> adapt_cache::AdaptCache<T, E> for InMemCache
where
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    async fn get(
        &self,
        _id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> Result<T, adapt_cache::Error<E>> {
        match convert(self.bytes.clone()) {
            Ok((value, _)) => Ok(value),
            Err(e) => Err(adapt_cache::Error::ConvertError(e)),
        }
    }
}

async fn handle_token(
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, std::convert::Infallible> {
    let method = req.method().clone();
    let ct = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body_bytes = match req.into_body().collect().await {
        Ok(c) => c.to_bytes(),
        Err(_) => Bytes::new(),
    };
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    eprintln!(
        "[test-server] method={method} content-type={ct} body={body_str}",
    );
    if method != hyper::Method::POST
        || !body_str.contains("grant_type=")
        || !ct.contains("application/x-www-form-urlencoded")
    {
        return Ok(hyper::Response::builder()
            .status(400)
            .body(Full::new(Bytes::from_static(b"bad request\n")))
            .unwrap());
    }
    Ok(hyper::Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from_static(b"{\"ok\":true}\n")))
        .unwrap())
}

async fn start_https_server() -> (SocketAddr, rustls::pki_types::CertificateDer<'static>) {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let issued = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .expect("generate self-signed cert");
    let cert_der = issued.cert.der().clone();
    let key_der = rustls::pki_types::PrivatePkcs8KeyDer::from(
        issued.signing_key.serialize_der(),
    );

    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der.clone()], key_der.into())
        .expect("server config");
    server_config.alpn_protocols = vec![b"h2".to_vec()];

    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(p) => p,
                Err(_) => continue,
            };
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(err) => {
                        eprintln!("[test-server] tls accept error: {err}");
                        return;
                    }
                };
                let result = server_http2::Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(tls_stream), service_fn(handle_token))
                    .await;
                if let Err(err) = result {
                    eprintln!("[test-server] serve error: {err}");
                }
            });
        }
    });

    (addr, cert_der)
}

fn build_shared_client(
    trust_cert: rustls::pki_types::CertificateDer<'static>,
) -> fn0::SharedHttpClient {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(trust_cert).expect("add root");

    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls_config)
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    let hyper_client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
        .build::<_, wasmtime_wasi_http::body::HyperOutgoingBody>(connector);

    fn0::SharedHttpClient::from_client(Arc::new(hyper_client))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wasm_outbound_post_round_trips_through_shared_client() {
    let (server_addr, cert_der) = start_https_server().await;
    let target_url = format!("https://localhost:{}/token", server_addr.port());
    eprintln!("[test] target_url={target_url}");

    let shared = build_shared_client(cert_der);

    // Pre-serialize the test wasm into a cwasm that the executor's cache
    // closure (which uses Component::deserialize) can load. Must use the same
    // engine_config as the executor so CPU feature negotiation matches.
    let serialize_engine =
        wasmtime::Engine::new(&fn0::execute::engine_config()).expect("engine");
    let component = wasmtime::component::Component::new(&serialize_engine, TEST_WASM_BYTES)
        .expect("component from wasm");
    let cwasm_bytes = Bytes::from(component.serialize().expect("serialize component"));

    let env_vars: fn0::execute::EnvVars =
        Arc::new(RwLock::new(HashMap::new()));
    env_vars.write().unwrap().insert(
        "test-app".to_string(),
        vec![("TARGET_URL".to_string(), target_url)],
    );

    let executor = fn0::execute::WasmExecutor::new(
        InMemCache::new(cwasm_bytes),
        fn0::measure_cpu_time::SystemClock,
        env_vars,
        shared,
        None,
    );

    let req_body: fn0::Body = Empty::<Bytes>::new()
        .map_err(|e: std::convert::Infallible| match e {})
        .map_err(|_| anyhow::anyhow!("unreachable"))
        .boxed_unsync();
    let req = hyper::Request::builder()
        .method("GET")
        .uri("http://test-app.local/")
        .header("host", "test-app.local")
        .body(req_body)
        .unwrap();

    let resp = executor
        .run("test-app::backend", req)
        .await
        .expect("executor run");

    let status = resp.status();
    let body = resp
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    eprintln!("[test] wasm -> us: status={status} body={body_str}");

    assert_eq!(status, 200, "unexpected status; body: {body_str}");
    assert!(
        body_str.contains("\"ok\":true"),
        "unexpected body: {body_str}"
    );
}
