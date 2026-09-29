use std::net::SocketAddr;

use bytes::Bytes;
use parking_lot::Mutex;

use super::*;
use crate::queue::Spool;
use crate::queue::{Queue, QueueLimits};
use crate::signal::Signal;
use crate::test_support::Scratch;

struct Scripted {
    respond: Box<dyn Fn(&Bytes) -> Outcome + Send + Sync>,
    segments: Mutex<Vec<(Signal, u64)>>,
    bytes: Mutex<Vec<usize>>,
}

impl Scripted {
    fn new(respond: impl Fn(&Bytes) -> Outcome + Send + Sync + 'static) -> Arc<Scripted> {
        Arc::new(Scripted {
            respond: Box::new(respond),
            segments: Mutex::new(Vec::new()),
            bytes: Mutex::new(Vec::new()),
        })
    }

    fn segments(&self) -> Vec<(Signal, u64)> {
        self.segments.lock().clone()
    }

    fn bytes(&self) -> Vec<usize> {
        self.bytes.lock().clone()
    }
}

impl Transport for Scripted {
    fn deliver<'a>(&'a self, shipment: Shipment) -> DeliverFuture<'a> {
        Box::pin(async move {
            self.segments
                .lock()
                .push((shipment.signal, shipment.segment));
            self.bytes.lock().push(shipment.body.len());
            (self.respond)(&shipment.body)
        })
    }
}

fn shipment(body: &'static [u8]) -> Shipment {
    Shipment {
        body: Bytes::from_static(body),
        sender: crate::queue::SenderId::generate().expect("an id"),
        signal: Signal::Logs,
        segment: 1,
    }
}

/// The sender closes the open segment itself, so a queue under test closes on
/// the first ask.
fn eager() -> QueueLimits {
    QueueLimits {
        max_segment_age: Duration::from_nanos(1),
        ..QueueLimits::default()
    }
}

/// One segment per record, so a test can count segments and records together.
fn queue_with(scratch: &Scratch, bodies: &[Vec<u8>]) -> Arc<Queue> {
    let queue =
        Arc::new(Queue::open(scratch.path(), eager(), crate::wire::ZSTD_LEVEL).expect("a queue"));
    for body in bodies {
        queue.append(Signal::Logs, body).expect("an append");
        queue.seal_if_due().expect("a seal");
    }
    queue
}

fn records(count: usize) -> Vec<Vec<u8>> {
    (0..count).map(|index| vec![index as u8; 32]).collect()
}

async fn deliver_all<T: Transport>(sender: &Sender<T>, queue: &Queue) {
    let (_tx, mut rx) = watch::channel(false);
    queue.seal_if_due().expect("a seal");
    while let Some((signal, seq)) = queue.oldest_sealed() {
        let segment = queue.read_segment(signal, seq).expect("a segment");
        sender.deliver(segment, &mut rx).await;
        queue.seal_if_due().expect("a seal");
    }
}

#[tokio::test]
async fn a_delivered_segment_advances_the_cursor_and_unlinks_the_file() {
    let scratch = Scratch::new("send-ok");
    let queue = queue_with(&scratch, &records(3));
    let transport = Scripted::new(|_| Outcome::Accepted(0));
    let sender = Sender::new(
        queue.clone(),
        Spool::new(queue.clone()),
        transport.clone(),
        SenderConfig::default(),
    );

    deliver_all(&sender, &queue).await;

    assert_eq!(
        transport.segments(),
        vec![(Signal::Logs, 1), (Signal::Logs, 2), (Signal::Logs, 3)]
    );
    assert_eq!(sender.stats().sent_segments.load(Ordering::Relaxed), 3);
    assert_eq!(sender.stats().sent_segments.load(Ordering::Relaxed), 3);
    assert!(!queue.has_sealed());
}

/// A segment is sent from its first record every time. Nothing is remembered
/// about how far into one an earlier attempt reached — signy remembers that.
#[tokio::test]
async fn a_retried_segment_is_sent_whole_again() {
    let scratch = Scratch::new("send-retry");
    let queue = queue_with(&scratch, &records(1));
    let attempts = Arc::new(Mutex::new(0usize));
    let counter = attempts.clone();
    let transport = Scripted::new(move |_| {
        let mut seen = counter.lock();
        *seen += 1;
        if *seen < 3 {
            Outcome::Retry("signy is not up".to_string())
        } else {
            Outcome::Accepted(0)
        }
    });
    let sender = Sender::new(
        queue.clone(),
        Spool::new(queue.clone()),
        transport.clone(),
        SenderConfig::default(),
    );

    tokio::time::pause();
    deliver_all(&sender, &queue).await;

    assert_eq!(
        transport.segments(),
        vec![(Signal::Logs, 1), (Signal::Logs, 1), (Signal::Logs, 1)]
    );
    let sizes = transport.bytes();
    assert!(sizes[0] > 0);
    assert!(
        sizes.iter().all(|size| *size == sizes[0]),
        "the same bytes each time, not {sizes:?}"
    );
    assert_eq!(sender.stats().retries.load(Ordering::Relaxed), 2);
    assert!(!queue.has_sealed());
}

/// Halving is gone with the batch. signy drops a record it cannot decode on
/// its own side, so a refusal that reaches here is the segment's shape and no
/// amount of splitting fixes it.
#[tokio::test]
async fn a_refused_segment_is_dropped_whole_and_counted() {
    let scratch = Scratch::new("send-refused");
    let queue = queue_with(&scratch, &records(2));
    // On what the segment holds rather than on its first byte: the body is a
    // compressed stream now and every one of them starts the same way.
    let transport = Scripted::new(|body: &Bytes| {
        let plain = crate::wire::decompress(body, body.len() * 8).expect("a stream");
        let records = crate::wire::split_records(&plain).expect("framed records");
        if records[0][0] == 0 {
            Outcome::Refused("signy cannot read this".to_string())
        } else {
            Outcome::Accepted(0)
        }
    });
    let sender = Sender::new(
        queue.clone(),
        Spool::new(queue.clone()),
        transport.clone(),
        SenderConfig::default(),
    );

    deliver_all(&sender, &queue).await;

    assert_eq!(
        transport.segments(),
        vec![(Signal::Logs, 1), (Signal::Logs, 2)]
    );
    assert_eq!(sender.stats().refused_segments.load(Ordering::Relaxed), 1);
    assert_eq!(sender.stats().sent_segments.load(Ordering::Relaxed), 1);
    assert!(!queue.has_sealed());
}

#[tokio::test(start_paused = true)]
async fn shutdown_stops_a_retry_loop_without_advancing_the_cursor() {
    let scratch = Scratch::new("send-stop");
    let queue = queue_with(&scratch, &records(1));
    let transport = Scripted::new(|_| Outcome::Retry("signy is down".to_string()));
    let sender = Sender::new(
        queue.clone(),
        Spool::new(queue.clone()),
        transport.clone(),
        SenderConfig::default(),
    );

    let (tx, mut rx) = watch::channel(false);
    let (signal, seq) = queue.oldest_sealed().expect("a closed segment");
    let segment = queue.read_segment(signal, seq).expect("a segment");
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = tx.send(true);
    });
    sender.deliver(segment, &mut rx).await;

    assert_eq!(
        queue.oldest_sealed(),
        Some((Signal::Logs, 1)),
        "the segment is still owed"
    );
    assert_eq!(sender.stats().sent_segments.load(Ordering::Relaxed), 0);
}

/// Real time, not a paused clock. The run loop now waits on the spool thread,
/// and a paused runtime that sees every task blocked on another OS thread
/// decides it is idle and jumps the clock to the next timer -- which here is
/// the one that stops the loop, so it would stop before it had shipped
/// anything. The stop is waited for instead of timed.
#[tokio::test]
async fn the_run_loop_closes_the_open_segment_and_ships_it() {
    let scratch = Scratch::new("send-run");
    let queue =
        Arc::new(Queue::open(scratch.path(), eager(), crate::wire::ZSTD_LEVEL).expect("a queue"));
    for body in records(3) {
        queue.append(Signal::Logs, &body).expect("an append");
    }
    let transport = Scripted::new(|_| Outcome::Accepted(0));
    let sender = Sender::new(
        queue.clone(),
        Spool::new(queue.clone()),
        transport.clone(),
        SenderConfig::default(),
    );

    let (tx, rx) = watch::channel(false);
    let stop = async {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while transport.segments().is_empty() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        let _ = tx.send(true);
    };
    tokio::join!(sender.run(rx), stop);

    // One segment, because nothing closed it between the appends, and it went
    // whole.
    assert_eq!(transport.segments(), vec![(Signal::Logs, 1)]);
    assert_eq!(sender.stats().sent_segments.load(Ordering::Relaxed), 1);
    assert_eq!(queue.stats().appended_records, 3);
    assert!(!queue.has_sealed());
}

/// signy can be ahead: it answered an attempt collecty never heard. Everything
/// up to what it names goes at once.
#[tokio::test]
async fn an_answer_beyond_the_segment_clears_everything_under_it() {
    let scratch = Scratch::new("send-ahead");
    let queue = queue_with(&scratch, &records(3));
    let transport = Scripted::new(|_| Outcome::Accepted(3));
    let sender = Sender::new(
        queue.clone(),
        Spool::new(queue.clone()),
        transport.clone(),
        SenderConfig::default(),
    );

    deliver_all(&sender, &queue).await;

    assert_eq!(
        transport.segments(),
        vec![(Signal::Logs, 1)],
        "the rest were already stored"
    );
    assert!(
        !queue.has_sealed(),
        "segments two and three went with the answer"
    );
}

type SeenRequest = (String, String, String, String, String, String, String);

async fn fake_signy(status: http::StatusCode, seen: Arc<Mutex<Vec<SeenRequest>>>) -> SocketAddr {
    fake_signy_with_body(status, Bytes::from_static(br#"{"stored":42}"#), seen).await
}

async fn fake_signy_with_body(
    status: http::StatusCode,
    response_body: Bytes,
    seen: Arc<Mutex<Vec<SeenRequest>>>,
) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a bound port");
    let address = listener.local_addr().expect("an address");

    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let seen = seen.clone();
            let response_body = response_body.clone();
            tokio::spawn(async move {
                let service = hyper::service::service_fn(
                    move |request: http::Request<hyper::body::Incoming>| {
                        let seen = seen.clone();
                        let response_body = response_body.clone();
                        async move {
                            let header = |name: &str| {
                                request
                                    .headers()
                                    .get(name)
                                    .and_then(|value| value.to_str().ok())
                                    .unwrap_or("")
                                    .to_string()
                            };
                            seen.lock().push((
                                request.uri().path().to_string(),
                                header("content-encoding"),
                                header(super::transport::SENDER_HEADER),
                                header(super::transport::SIGNAL_HEADER),
                                header(super::transport::SEGMENT_HEADER),
                                header(super::transport::ACCESS_CLIENT_ID_HEADER),
                                header(super::transport::ACCESS_CLIENT_SECRET_HEADER),
                            ));
                            http::Response::builder()
                                .status(status)
                                .body(http_body_util::Full::new(response_body))
                        }
                    },
                );
                let _ = hyper::server::conn::http1::Builder::new()
                    .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                    .await;
            });
        }
    });

    address
}

async fn fake_https_signy(
    status: http::StatusCode,
    seen: Arc<Mutex<Vec<SeenRequest>>>,
) -> (SocketAddr, rustls::pki_types::CertificateDer<'static>) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).expect("a certificate");
    let certificate = cert.der().clone();
    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certificate.clone()],
            rustls::pki_types::PrivateKeyDer::Pkcs8(signing_key.serialize_der().into()),
        )
        .expect("a server config");
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a bound port");
    let address = listener.local_addr().expect("an address");

    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let acceptor = acceptor.clone();
            let seen = seen.clone();
            tokio::spawn(async move {
                let Ok(stream) = acceptor.accept(stream).await else {
                    return;
                };
                let service = hyper::service::service_fn(
                    move |request: http::Request<hyper::body::Incoming>| {
                        let seen = seen.clone();
                        async move {
                            let header = |name: &str| {
                                request
                                    .headers()
                                    .get(name)
                                    .and_then(|value| value.to_str().ok())
                                    .unwrap_or("")
                                    .to_string()
                            };
                            seen.lock().push((
                                request.uri().path().to_string(),
                                header("content-encoding"),
                                header(super::transport::SENDER_HEADER),
                                header(super::transport::SIGNAL_HEADER),
                                header(super::transport::SEGMENT_HEADER),
                                header(super::transport::ACCESS_CLIENT_ID_HEADER),
                                header(super::transport::ACCESS_CLIENT_SECRET_HEADER),
                            ));
                            http::Response::builder().status(status).body(
                                http_body_util::Full::new(Bytes::from_static(br#"{"stored":42}"#)),
                            )
                        }
                    },
                );
                let _ = hyper::server::conn::http1::Builder::new()
                    .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                    .await;
            });
        }
    });

    (address, certificate)
}

fn client_config_trusting(
    certificate: rustls::pki_types::CertificateDer<'static>,
) -> rustls::ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).expect("a trusted root");
    rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth()
}

#[tokio::test]
async fn a_success_over_http_carries_the_encoding_and_who_sent_which_segment() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let address = fake_signy(http::StatusCode::OK, seen.clone()).await;
    let transport = HttpTransport::new(format!("http://{address}"), Duration::from_secs(5));
    let mut shipment = shipment(b"frames");
    shipment.signal = Signal::Traces;
    shipment.segment = 7;
    let sender = shipment.sender.to_string();

    let outcome = transport.deliver(shipment).await;

    assert_eq!(
        outcome,
        Outcome::Accepted(42),
        "the answer says how far this sender got"
    );
    assert_eq!(
        seen.lock().as_slice(),
        [(
            "/signy/api/v1/collect".to_string(),
            "zstd".to_string(),
            sender,
            "traces".to_string(),
            "7".to_string(),
            "".to_string(),
            "".to_string()
        )]
    );
}

#[tokio::test]
async fn access_credentials_are_sent_as_the_two_cloudflare_headers() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let address = fake_signy(http::StatusCode::OK, seen.clone()).await;
    let transport = HttpTransport::with_access(
        format!("http://{address}"),
        Duration::from_secs(5),
        Some("client-id".to_string()),
        Some("client-secret".to_string()),
    );

    let outcome = transport.deliver(shipment(b"frames")).await;

    assert_eq!(outcome, Outcome::Accepted(42));
    assert_eq!(seen.lock()[0].5, "client-id");
    assert_eq!(seen.lock()[0].6, "client-secret");
}

#[tokio::test]
async fn access_credentials_are_redacted_from_error_reasons() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let address = fake_signy_with_body(
        http::StatusCode::SERVICE_UNAVAILABLE,
        Bytes::from_static(b"client client-secret"),
        seen,
    )
    .await;
    let transport = HttpTransport::with_access(
        format!("http://{address}"),
        Duration::from_secs(5),
        Some("client".to_string()),
        Some("client-secret".to_string()),
    );

    let outcome = transport.deliver(shipment(b"frames")).await;
    let reason = match outcome {
        Outcome::Retry(reason) => reason,
        other => panic!("expected retry, got {other:?}"),
    };

    assert!(!reason.contains("client"));
    assert!(!reason.contains("client-secret"));
    assert_eq!(reason.matches("[REDACTED]").count(), 2);
}

#[tokio::test]
async fn a_credential_cannot_change_the_stored_cursor_parsing() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let address = fake_signy(http::StatusCode::OK, seen).await;
    let transport = HttpTransport::with_access(
        format!("http://{address}"),
        Duration::from_secs(5),
        Some("client-id".to_string()),
        Some("stored".to_string()),
    );

    assert_eq!(
        transport.deliver(shipment(b"frames")).await,
        Outcome::Accepted(42)
    );
}

#[tokio::test]
async fn a_verified_https_certificate_and_hostname_are_required() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let (address, certificate) = fake_https_signy(http::StatusCode::OK, seen.clone()).await;
    let transport = HttpTransport::with_tls_config(
        format!("https://localhost:{}", address.port()),
        Duration::from_secs(5),
        Some("client-id".to_string()),
        Some("client-secret".to_string()),
        client_config_trusting(certificate),
    );

    let outcome = transport.deliver(shipment(b"frames")).await;

    assert_eq!(outcome, Outcome::Accepted(42));
    assert_eq!(seen.lock()[0].5, "client-id");
    assert_eq!(seen.lock()[0].6, "client-secret");
}

#[tokio::test]
async fn https_keeps_refusal_and_retry_status_contracts() {
    let refusing_seen = Arc::new(Mutex::new(Vec::new()));
    let (refusing_address, refusing_certificate) =
        fake_https_signy(http::StatusCode::BAD_REQUEST, refusing_seen).await;
    let refusing = HttpTransport::with_tls_config(
        format!("https://localhost:{}", refusing_address.port()),
        Duration::from_secs(5),
        None,
        None,
        client_config_trusting(refusing_certificate),
    );
    assert!(matches!(
        refusing.deliver(shipment(b"frames")).await,
        Outcome::Refused(_)
    ));

    let retrying_seen = Arc::new(Mutex::new(Vec::new()));
    let (retrying_address, retrying_certificate) =
        fake_https_signy(http::StatusCode::SERVICE_UNAVAILABLE, retrying_seen).await;
    let retrying = HttpTransport::with_tls_config(
        format!("https://localhost:{}", retrying_address.port()),
        Duration::from_secs(5),
        None,
        None,
        client_config_trusting(retrying_certificate),
    );
    assert!(matches!(
        retrying.deliver(shipment(b"frames")).await,
        Outcome::Retry(_)
    ));
}

#[tokio::test]
async fn an_untrusted_or_mismatched_https_certificate_is_rejected() {
    let untrusted_seen = Arc::new(Mutex::new(Vec::new()));
    let (untrusted_address, _) =
        fake_https_signy(http::StatusCode::OK, untrusted_seen.clone()).await;
    let untrusted = HttpTransport::new(
        format!("https://localhost:{}", untrusted_address.port()),
        Duration::from_secs(5),
    );
    assert!(matches!(
        untrusted.deliver(shipment(b"frames")).await,
        Outcome::Retry(_)
    ));
    assert!(untrusted_seen.lock().is_empty());

    let mismatched_seen = Arc::new(Mutex::new(Vec::new()));
    let (mismatched_address, certificate) =
        fake_https_signy(http::StatusCode::OK, mismatched_seen.clone()).await;
    let mismatched = HttpTransport::with_tls_config(
        format!("https://127.0.0.1:{}", mismatched_address.port()),
        Duration::from_secs(5),
        None,
        None,
        client_config_trusting(certificate),
    );
    assert!(matches!(
        mismatched.deliver(shipment(b"frames")).await,
        Outcome::Retry(_)
    ));
    assert!(mismatched_seen.lock().is_empty());
}

#[tokio::test]
async fn a_bad_request_refuses_the_payload_and_a_server_error_asks_for_a_retry() {
    let refusing = fake_signy(
        http::StatusCode::BAD_REQUEST,
        Arc::new(Mutex::new(Vec::new())),
    )
    .await;
    let transport = HttpTransport::new(format!("http://{refusing}"), Duration::from_secs(5));
    let outcome = transport.deliver(shipment(b"frames")).await;
    assert!(matches!(outcome, Outcome::Refused(_)), "{outcome:?}");

    let unavailable = fake_signy(
        http::StatusCode::SERVICE_UNAVAILABLE,
        Arc::new(Mutex::new(Vec::new())),
    )
    .await;
    let transport = HttpTransport::new(format!("http://{unavailable}"), Duration::from_secs(5));
    let outcome = transport.deliver(shipment(b"frames")).await;
    assert!(matches!(outcome, Outcome::Retry(_)), "{outcome:?}");
}

#[tokio::test]
async fn a_refused_connection_asks_for_a_retry_rather_than_dropping() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a bound port");
    let address = listener.local_addr().expect("an address");
    drop(listener);

    let transport = HttpTransport::new(format!("http://{address}"), Duration::from_secs(5));
    let outcome = transport.deliver(shipment(b"frames")).await;

    assert!(matches!(outcome, Outcome::Retry(_)), "{outcome:?}");
}
