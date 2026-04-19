//! Regression: `fn0::outbound::send_request` must strip any `host` header
//! before handing the request to `hyper-util`, otherwise h2 servers that treat
//! `Host` + `:authority` coexistence as malformed (e.g. Google's GFE) reject
//! the stream with RST_STREAM PROTOCOL_ERROR and the client sees
//! `hyper::Error(Http2, Error { kind: Reset(_, PROTOCOL_ERROR, Remote) })`
//! surfaced as `client error (SendRequest)`.
//!
//! This test boots a local h2-only TLS server that mimics that strict policy:
//! if the incoming request carries a `host` header, the stream is reset with
//! PROTOCOL_ERROR; otherwise a 200 OK is returned. The client-side request is
//! built with a pre-attached `host` header — matching what
//! `wasmtime-wasi-http::http_impl::handle` synthesises at runtime — and is
//! dispatched through `fn0::outbound::send_request`.

use bytes::Bytes;
use fn0::outbound::{OutboundContext, SharedHttpClient, send_request};
use h2::Reason;
use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioExecutor;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use wasmtime_wasi_http::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::body::HyperOutgoingBody;
use wasmtime_wasi_http::types::OutgoingRequestConfig;

async fn start_host_rejecting_h2_server(
) -> (SocketAddr, rustls::pki_types::CertificateDer<'static>) {
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
                let mut h2_conn: h2::server::Connection<_, Bytes> =
                    match h2::server::handshake(tls_stream).await {
                        Ok(c) => c,
                        Err(err) => {
                            eprintln!("[test-server] h2 handshake error: {err}");
                            return;
                        }
                    };
                while let Some(result) = h2_conn.accept().await {
                    let (request, mut respond): (
                        hyper::Request<h2::RecvStream>,
                        h2::server::SendResponse<Bytes>,
                    ) = match result {
                        Ok(p) => p,
                        Err(err) => {
                            eprintln!("[test-server] accept stream error: {err}");
                            return;
                        }
                    };
                    let has_host = request.headers().contains_key(hyper::header::HOST);
                    eprintln!(
                        "[test-server] stream received; host-header-present={has_host}",
                    );
                    if has_host {
                        respond.send_reset(Reason::PROTOCOL_ERROR);
                        continue;
                    }
                    let response = hyper::Response::builder()
                        .status(200)
                        .header("content-type", "text/plain")
                        .body(())
                        .unwrap();
                    let mut send_stream = match respond.send_response(response, false) {
                        Ok(s) => s,
                        Err(err) => {
                            eprintln!("[test-server] send_response error: {err}");
                            continue;
                        }
                    };
                    let _ = send_stream.send_data(Bytes::from_static(b"accepted"), true);
                }
            });
        }
    });

    (addr, cert_der)
}

fn build_shared_client(
    trust_cert: rustls::pki_types::CertificateDer<'static>,
) -> SharedHttpClient {
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
        .build::<_, HyperOutgoingBody>(connector);

    SharedHttpClient::from_client(Arc::new(hyper_client))
}

fn make_outgoing_body(bytes: &'static [u8]) -> HyperOutgoingBody {
    Full::new(Bytes::from_static(bytes))
        .map_err(|_: std::convert::Infallible| {
            ErrorCode::InternalError(Some("unreachable".to_string()))
        })
        .boxed_unsync()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn send_request_strips_host_for_h2_strict_server() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .try_init();

    let (server_addr, cert_der) = start_host_rejecting_h2_server().await;
    let target_uri = format!("https://localhost:{}/token", server_addr.port())
        .parse::<hyper::Uri>()
        .unwrap();

    let shared = build_shared_client(cert_der);
    let ctx = OutboundContext {
        shared_client: shared,
        turso_hijack: None,
        code_id: "test::backend".to_string(),
    };

    let req = hyper::Request::builder()
        .method("POST")
        .uri(target_uri.clone())
        .header(
            hyper::header::HOST,
            format!("localhost:{}", server_addr.port()),
        )
        .header("content-type", "application/x-www-form-urlencoded")
        .body(make_outgoing_body(b"grant_type=authorization_code&code=FAKE"))
        .unwrap();

    let config = OutgoingRequestConfig {
        use_tls: true,
        connect_timeout: Duration::from_secs(10),
        first_byte_timeout: Duration::from_secs(10),
        between_bytes_timeout: Duration::from_secs(10),
    };

    let mut fut = send_request(ctx, req, config);
    use wasmtime_wasi::p2::Pollable;
    fut.ready().await;
    let result = fut.unwrap_ready().expect("trap-free");

    let resp = match result {
        Ok(r) => r,
        Err(err) => panic!(
            "expected 200 OK after Host-header strip, got wasi-http ErrorCode: {err:?}"
        ),
    };

    assert_eq!(resp.resp.status(), 200);
    let body = resp
        .resp
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    assert_eq!(&body[..], b"accepted", "unexpected body: {body:?}");
}
