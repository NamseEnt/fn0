use std::net::SocketAddr;

use http::header::{CONTENT_ENCODING, CONTENT_TYPE};
use http::{Method, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use opentelemetry_proto::tonic::metrics::v1::ResourceMetrics;
use opentelemetry_proto::tonic::trace::v1::ResourceSpans;
use prost::Message;
use tokio::sync::oneshot;

use super::*;
use crate::queue::QueueLimits;
use crate::queue::{Queue, Spool};
use crate::test_support::Scratch;

/// The tests take what the sender would take, so the open segment has to close
/// the moment they ask.
fn eager_limits() -> QueueLimits {
    QueueLimits {
        max_segment_age: std::time::Duration::from_nanos(1),
        ..QueueLimits::default()
    }
}

struct Harness {
    _scratch: Scratch,
    queue: Arc<Queue>,
    addr: SocketAddr,
    client: Client<HttpConnector, Full<Bytes>>,
    shutdown: Option<oneshot::Sender<()>>,
    server: tokio::task::JoinHandle<io::Result<()>>,
}

/// One export, spelled the way an OTLP/HTTP client spells it.
struct Export {
    method: Method,
    path: &'static str,
    content_type: Option<&'static str>,
    content_encoding: Option<&'static str>,
    body: Vec<u8>,
}

impl Export {
    fn to(path: &'static str, body: Vec<u8>) -> Export {
        Export {
            method: Method::POST,
            path,
            content_type: Some(PROTOBUF),
            content_encoding: None,
            body,
        }
    }
}

impl Harness {
    async fn start(label: &str, max_request_bytes: usize) -> Harness {
        let scratch = Scratch::new(label);
        let queue = Arc::new(
            Queue::open(
                &scratch.path().join("queue"),
                eager_limits(),
                crate::wire::ZSTD_LEVEL,
            )
            .expect("a queue"),
        );
        // Port zero, so a test never collides with another test or with
        // whatever else is listening on this machine.
        let listener = bind("127.0.0.1:0".parse().expect("a literal address")).expect("a listener");
        let addr = listener.local_addr().expect("a bound address");
        let intake = Intake::new(
            Spool::new(queue.clone()),
            max_request_bytes,
            DEFAULT_MAX_INFLIGHT_BYTES,
        );
        let (tx, rx) = oneshot::channel();
        let server = tokio::spawn(serve(intake, listener, async move {
            let _ = rx.await;
        }));

        Harness {
            _scratch: scratch,
            queue,
            addr,
            client: Client::builder(TokioExecutor::new()).build(HttpConnector::new()),
            shutdown: Some(tx),
            server,
        }
    }

    async fn send(&self, export: Export) -> (StatusCode, String) {
        let mut request = http::Request::builder()
            .method(export.method)
            .uri(format!("http://{}{}", self.addr, export.path));
        if let Some(content_type) = export.content_type {
            request = request.header(CONTENT_TYPE, content_type);
        }
        if let Some(encoding) = export.content_encoding {
            request = request.header(CONTENT_ENCODING, encoding);
        }
        let response = self
            .client
            .request(
                request
                    .body(Full::new(Bytes::from(export.body)))
                    .expect("a well-formed request"),
            )
            .await
            .expect("an answer");
        let status = response.status();
        let body = response.into_body().collect().await.expect("a body");
        (
            status,
            String::from_utf8_lossy(&body.to_bytes()).into_owned(),
        )
    }

    async fn export(&self, path: &'static str, body: Vec<u8>) -> StatusCode {
        self.send(Export::to(path, body)).await.0
    }

    /// Every record the queue is holding, under the signal of the segment it
    /// is in — which is the only place the signal is written down now.
    fn records(&self) -> Vec<(Signal, Vec<u8>)> {
        self.sealed()
            .into_iter()
            .flat_map(|(signal, body)| {
                let plain = crate::wire::decompress(&body, body.len() * 8).expect("a stream");
                crate::wire::split_records(&plain)
                    .expect("framed records")
                    .into_iter()
                    .map(|payload| (signal, payload.to_vec()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// What the sender would ship: every open segment closed, then read whole,
    /// oldest first.
    fn sealed(&self) -> Vec<(Signal, Bytes)> {
        self.queue.seal_if_due().expect("a seal");
        let mut bodies = Vec::new();
        while let Some((signal, seq)) = self.queue.oldest_sealed() {
            let sealed = self.queue.read_segment(signal, seq).expect("a segment");
            bodies.push((signal, sealed.body));
            self.queue.commit(signal, seq).expect("a commit");
            self.queue.seal_if_due().expect("a seal");
        }
        bodies
    }

    async fn stop(mut self) {
        let _ = self.shutdown.take().expect("one shutdown").send(());
        let _ = self.server.await;
    }
}

fn logs(body: &str) -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![LogRecord {
                    time_unix_nano: 7,
                    body: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(body.to_string())),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[tokio::test]
async fn an_export_is_stored_as_a_frame_that_decompresses_to_the_request() {
    let harness = Harness::start("receive-store", DEFAULT_MAX_REQUEST_BYTES).await;
    let request = logs("a line worth compressing, repeated repeated repeated");
    let expected = request.encode_to_vec();

    let status = harness.export("/v1/logs", expected.clone()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(harness.records(), vec![(Signal::Logs, expected)]);
    harness.stop().await;
}

/// Three exports, three segments: nothing shares a segment with another
/// signal, and the order they arrived in is the order they are sent in.
/// A body arriving as several frames, which is what a body of any size
/// actually does. `route` no longer joins them into one buffer, so the thing
/// that could break is the order or a dropped piece, and the record on disk
/// says whether either happened.
struct InPieces(std::vec::IntoIter<Bytes>);

impl hyper::body::Body for InPieces {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Bytes>, Self::Error>>> {
        std::task::Poll::Ready(
            self.0
                .next()
                .map(|piece| Ok(hyper::body::Frame::data(piece))),
        )
    }
}

#[tokio::test]
async fn a_body_that_arrives_in_pieces_is_one_record_again() {
    let harness = Harness::start("receive-pieces", DEFAULT_MAX_REQUEST_BYTES).await;
    // Long enough to be worth cutting up, and cut unevenly so that no piece
    // boundary lines up with anything the record format cares about.
    let export =
        logs(&"a log line that is long enough to be worth cutting up. ".repeat(64)).encode_to_vec();
    let pieces: Vec<Bytes> = export.chunks(777).map(Bytes::copy_from_slice).collect();
    assert!(pieces.len() > 3, "the body has to arrive in several pieces");

    let client = Client::builder(TokioExecutor::new()).build_http::<InPieces>();
    let response = client
        .request(
            http::Request::builder()
                .method(Method::POST)
                .uri(format!("http://{}/v1/logs", harness.addr))
                .header(CONTENT_TYPE, "application/x-protobuf")
                .body(InPieces(pieces.into_iter()))
                .expect("a well-formed request"),
        )
        .await
        .expect("an answer");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        harness.records(),
        vec![(Signal::Logs, export)],
        "the record is the export, byte for byte and in order"
    );
    harness.stop().await;
}

/// The same, but the client stops halfway through.
struct FailsPartway(std::vec::IntoIter<Bytes>);

impl hyper::body::Body for FailsPartway {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Bytes>, Self::Error>>> {
        std::task::Poll::Ready(Some(match self.0.next() {
            Some(piece) => Ok(hyper::body::Frame::data(piece)),
            None => Err(std::io::Error::other("the client went away mid-body")),
        }))
    }
}

/// An export that never finished arriving is not half an export.
///
/// Reading the body frame by frame could have meant queueing what turned up
/// and then discovering the rest was never coming, which would put a truncated
/// protobuf into a segment for signy to refuse -- and it refuses whole
/// segments. The frames are held until the body ends, so a body that ends
/// badly is a request that queued nothing.
#[tokio::test]
async fn a_body_that_stops_halfway_queues_nothing() {
    let harness = Harness::start("receive-truncated", DEFAULT_MAX_REQUEST_BYTES).await;
    let lost = logs("an export that never arrived in full").encode_to_vec();
    let pieces: Vec<Bytes> = lost.chunks(16).map(Bytes::copy_from_slice).collect();
    assert!(
        pieces.len() > 1,
        "some of it has to arrive before the rest does not"
    );

    let client = Client::builder(TokioExecutor::new()).build_http::<FailsPartway>();
    let outcome = client
        .request(
            http::Request::builder()
                .method(Method::POST)
                .uri(format!("http://{}/v1/logs", harness.addr))
                .header(CONTENT_TYPE, "application/x-protobuf")
                .body(FailsPartway(pieces.into_iter()))
                .expect("a well-formed request"),
        )
        .await;
    // Not a vacuous test: the request has to have failed rather than never
    // been made, or the body was never read and there was nothing to be
    // partial about.
    assert!(
        !matches!(&outcome, Ok(response) if response.status().is_success()),
        "a truncated export was answered as a success"
    );

    // An export that does arrive, afterwards. It is here so that the check
    // below is not just a check made too early: whatever the truncated one
    // left behind would be in front of this one.
    let kept = logs("an export that did arrive").encode_to_vec();
    assert_eq!(
        harness.export("/v1/logs", kept.clone()).await,
        StatusCode::OK
    );

    assert_eq!(harness.records(), vec![(Signal::Logs, kept)]);
    harness.stop().await;
}

#[tokio::test]
async fn every_signal_lands_in_a_segment_of_its_own() {
    let harness = Harness::start("receive-signals", DEFAULT_MAX_REQUEST_BYTES).await;

    assert_eq!(
        harness
            .export("/v1/logs", logs("one log").encode_to_vec())
            .await,
        StatusCode::OK
    );
    assert_eq!(
        harness
            .export(
                "/v1/traces",
                ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans::default()],
                }
                .encode_to_vec(),
            )
            .await,
        StatusCode::OK
    );
    assert_eq!(
        harness
            .export(
                "/v1/metrics",
                ExportMetricsServiceRequest {
                    resource_metrics: vec![ResourceMetrics::default()],
                }
                .encode_to_vec(),
            )
            .await,
        StatusCode::OK
    );

    let sealed = harness.sealed();
    let signals: Vec<Signal> = sealed.iter().map(|(signal, _)| *signal).collect();
    assert_eq!(signals, vec![Signal::Logs, Signal::Traces, Signal::Metrics]);
    harness.stop().await;
}

/// A successful export answers with an empty body, which is a valid
/// `ExportLogsServiceResponse` carrying no `partial_success`.
#[tokio::test]
async fn a_success_answers_with_an_empty_protobuf_body() {
    let harness = Harness::start("receive-ack", DEFAULT_MAX_REQUEST_BYTES).await;

    let (status, body) = harness
        .send(Export::to("/v1/logs", logs("one line").encode_to_vec()))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.is_empty(),
        "a success should answer no bytes: {body:?}"
    );
    assert_eq!(
        ExportLogsServiceRequest::decode(body.as_bytes()).expect("an empty response decodes"),
        ExportLogsServiceRequest::default()
    );
    harness.stop().await;
}

#[tokio::test]
async fn an_export_over_the_ceiling_is_refused_and_stores_nothing() {
    let harness = Harness::start("receive-ceiling", 64).await;

    let (status, _) = harness
        .send(Export::to(
            "/v1/logs",
            logs(&"x".repeat(4096)).encode_to_vec(),
        ))
        .await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(harness.records().is_empty());
    assert_eq!(harness.queue.stats().appended_records, 0);
    harness.stop().await;
}

#[tokio::test]
async fn a_payload_over_the_ceiling_is_refused_off_the_http_path_too() {
    let scratch = Scratch::new("intake-ceiling");
    let queue = Arc::new(
        Queue::open(
            &scratch.path().join("queue"),
            eager_limits(),
            crate::wire::ZSTD_LEVEL,
        )
        .expect("a queue"),
    );
    let intake = Intake::new(Spool::new(queue.clone()), 64, DEFAULT_MAX_INFLIGHT_BYTES);

    let refusal = intake
        .accept(Signal::Logs, vec![bytes::Bytes::from(vec![0u8; 128])])
        .await
        .expect_err("a refusal");

    assert_eq!(refusal.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(queue.stats().appended_records, 0);
}

#[tokio::test]
async fn an_empty_export_succeeds_without_storing_a_record() {
    let harness = Harness::start("receive-empty", DEFAULT_MAX_REQUEST_BYTES).await;

    let status = harness
        .export(
            "/v1/logs",
            ExportLogsServiceRequest {
                resource_logs: Vec::new(),
            }
            .encode_to_vec(),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(harness.queue.stats().appended_records, 0);
    harness.stop().await;
}

/// The JSON encoding and a compressed body are both refused rather than
/// stored: neither survives the concatenation the queue is built on.
#[tokio::test]
async fn only_uncompressed_protobuf_is_taken() {
    let harness = Harness::start("receive-encoding", DEFAULT_MAX_REQUEST_BYTES).await;
    let body = logs("one line").encode_to_vec();

    let (json, _) = harness
        .send(Export {
            content_type: Some("application/json"),
            ..Export::to("/v1/logs", body.clone())
        })
        .await;
    let (gzip, reason) = harness
        .send(Export {
            content_encoding: Some("gzip"),
            ..Export::to("/v1/logs", body.clone())
        })
        .await;
    let (missing, _) = harness
        .send(Export {
            content_type: None,
            ..Export::to("/v1/logs", body)
        })
        .await;

    assert_eq!(json, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(gzip, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(missing, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(
        reason.contains("gzip"),
        "the refusal should name it: {reason}"
    );
    assert_eq!(harness.queue.stats().appended_records, 0);
    harness.stop().await;
}

/// A content type with parameters is the same content type.
#[tokio::test]
async fn a_charset_parameter_does_not_change_the_media_type() {
    let harness = Harness::start("receive-charset", DEFAULT_MAX_REQUEST_BYTES).await;

    let (status, _) = harness
        .send(Export {
            content_type: Some("application/x-protobuf; charset=utf-8"),
            ..Export::to("/v1/logs", logs("one line").encode_to_vec())
        })
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(harness.queue.stats().appended_records, 1);
    harness.stop().await;
}

#[tokio::test]
async fn a_path_that_is_not_a_signal_is_not_answered_for() {
    let harness = Harness::start("receive-path", DEFAULT_MAX_REQUEST_BYTES).await;
    let body = logs("one line").encode_to_vec();

    let (unknown, _) = harness.send(Export::to("/v1/profiles", body.clone())).await;
    let (root, _) = harness.send(Export::to("/", body.clone())).await;
    let (wrong_method, _) = harness
        .send(Export {
            method: Method::GET,
            ..Export::to("/v1/logs", Vec::new())
        })
        .await;

    assert_eq!(unknown, StatusCode::NOT_FOUND);
    assert_eq!(root, StatusCode::NOT_FOUND);
    assert_eq!(wrong_method, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(harness.queue.stats().appended_records, 0);
    harness.stop().await;
}

#[tokio::test]
async fn separate_exports_reassemble_into_one_merged_request() {
    let harness = Harness::start("receive-merge", DEFAULT_MAX_REQUEST_BYTES).await;
    let requests: Vec<_> = (0..6).map(|index| logs(&format!("line {index}"))).collect();

    for request in &requests {
        assert_eq!(
            harness.export("/v1/logs", request.encode_to_vec()).await,
            StatusCode::OK
        );
    }

    let sealed = harness.sealed();
    assert_eq!(sealed.len(), 1, "one signal, one segment");
    let (signal, body) = &sealed[0];
    assert_eq!(*signal, Signal::Logs);
    let plain = crate::wire::decompress(body, body.len() * 8).expect("plain");
    let mut payloads = Vec::new();
    for payload in crate::wire::split_records(&plain).expect("framed records") {
        payloads.extend_from_slice(payload);
    }
    let merged = ExportLogsServiceRequest::decode(payloads.as_slice()).expect("a merged request");

    assert_eq!(merged.resource_logs.len(), requests.len());
    for (index, request) in requests.iter().enumerate() {
        assert_eq!(merged.resource_logs[index], request.resource_logs[0]);
    }
    harness.stop().await;
}

/// The client here keeps its connection pooled and open. Shutdown has to
/// close it rather than wait on it: the caller seals the open segment once
/// this returns, and a process that never returns never seals.
#[tokio::test]
async fn an_idle_keep_alive_connection_does_not_hold_shutdown_open() {
    let harness = Harness::start("receive-drain", DEFAULT_MAX_REQUEST_BYTES).await;
    assert_eq!(
        harness
            .export("/v1/logs", logs("one line").encode_to_vec())
            .await,
        StatusCode::OK
    );

    tokio::time::timeout(std::time::Duration::from_secs(5), harness.stop())
        .await
        .expect("shutdown should not wait on an idle connection");
}

#[tokio::test]
async fn binding_over_a_live_address_is_refused() {
    let harness = Harness::start("receive-bind", DEFAULT_MAX_REQUEST_BYTES).await;

    let error = bind(harness.addr).expect_err("a refusal");

    assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse);
    assert!(
        error.to_string().contains(&harness.addr.to_string()),
        "the error does not name the address: {error}"
    );
    harness.stop().await;
}
