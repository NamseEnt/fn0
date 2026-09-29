//! The bench collecty's memory experiments are run on.
//!
//! [`docs/MEMORY.md`](../docs/MEMORY.md) has nine experiments to run and each
//! one has to be comparable with the one before it, so the load and the thing
//! that takes the segments both have to be the same every time and both have
//! to be cheap enough to run over and over. That is what this is: a fixed-rate
//! OTLP generator over a fixed number of connections, and a sink that answers
//! collecty's collect route the way signy would.
//!
//! It is an example rather than a binary on purpose. An example builds against
//! the dev-dependencies, so the corpus can be real OTLP logs without the
//! shipped collecty linking the log message types to get them.
//!
//! ```
//! cargo run --release --example memrig -- --seconds 300 --eps 20000
//! ```
//!
//! The sink can be taken away for a window (`--outage-at`, `--outage-for`),
//! which is how the drain step of `MEMORY.md` §3 is reproduced without
//! stopping a process: collecty sees refusals, backs off, builds a backlog on
//! disk, and then drains it as fast as the sink will take it.
//!
//! Every export is timed from the client's side -- the request going out to
//! the response saying it was taken -- because that is the number an exporter
//! sitting beside collecty actually experiences, and a change that halves the
//! footprint while pushing the tail from 20 ms to 800 ms is not a change worth
//! having. `--latency-csv` writes every sample so a tail can be looked at
//! again later; the report keeps percentiles and a histogram beside them,
//! because a single p99 cannot say what shape the tail took.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1 as server_http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost::Message;
use tokio::net::{TcpListener, TcpStream};

struct Args {
    collecty: String,
    sink: SocketAddr,
    eps: u64,
    connections: usize,
    seconds: u64,
    records_per_export: usize,
    outage_at: Option<u64>,
    outage_for: u64,
    report: Option<String>,
    latency_csv: Option<String>,
    trace_eps: f64,
    metric_every: f64,
}

fn args() -> Args {
    let mut args = Args {
        collecty: "127.0.0.1:4318".to_string(),
        sink: "127.0.0.1:4319".parse().unwrap(),
        eps: 20_000,
        connections: 8,
        seconds: 300,
        records_per_export: 64,
        outage_at: None,
        outage_for: 180,
        report: None,
        latency_csv: None,
        // The soak's own shape: a low trace rate and a scrape every ten
        // seconds. Both matter to memory out of all proportion to their
        // bytes -- a signal that never fills a segment still rolls one every
        // `COLLECTY_SEGMENT_MAX_AGE`, and every roll builds a compressor.
        trace_eps: 5.0,
        metric_every: 10.0,
    };
    let mut raw = std::env::args().skip(1);
    while let Some(flag) = raw.next() {
        let mut value = || raw.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--collecty" => args.collecty = value(),
            "--sink" => args.sink = value().parse().expect("an address"),
            "--eps" => args.eps = value().parse().expect("a number"),
            "--connections" => {
                args.connections = value().parse::<usize>().expect("a number").max(1)
            }
            "--seconds" => args.seconds = value().parse().expect("a number"),
            "--records-per-export" => {
                args.records_per_export = value().parse::<usize>().expect("a number").max(1)
            }
            "--outage-at" => args.outage_at = Some(value().parse().expect("a number")),
            "--outage-for" => args.outage_for = value().parse().expect("a number"),
            "--report" => args.report = Some(value()),
            "--latency-csv" => args.latency_csv = Some(value()),
            "--trace-eps" => args.trace_eps = value().parse().expect("a number"),
            "--metric-every" => args.metric_every = value().parse().expect("a number"),
            other => panic!("unknown flag {other}"),
        }
    }
    args
}

fn attribute(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}

/// Deterministic, so two runs compare. xorshift64* is enough entropy for a
/// corpus and costs nothing.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn hex(&mut self, digits: usize) -> String {
        let mut out = String::with_capacity(digits);
        while out.len() < digits {
            out.push_str(&format!("{:016x}", self.next()));
        }
        out.truncate(digits);
        out
    }
}

/// One export, distinct from every other, and **as compressible as production
/// is**.
///
/// The corpus this replaced was four repeated line templates, which zstd took
/// at 22x. The collector the 24-hour soak ran achieved 4.87x on the load
/// harness's corpus, and the ratio is not cosmetic: it decides how large a
/// segment is, which decides the size of the buffer the send path allocates
/// per delivery and how often a compressor is built and dropped. A corpus that
/// compresses four times too well measures a collector shipping segments an
/// eighth of the real size.
fn export(records: usize, batch: usize) -> Vec<u8> {
    let lines = [
        "GET /v1/checkout 200 in {}ms",
        "connection reset by peer while reading upstream {}",
        "cache miss for key user:{}:profile, falling back to postgres",
        "retrying publish attempt {} of 5 after 400ms",
        "POST /v1/orders 201 in {}ms",
        "slow query took {}ms: select * from orders where tenant_id = $1",
        "rate limit bucket {} refilled",
        "worker {} picked up job from queue",
    ];
    let routes = [
        "/v1/checkout",
        "/v1/orders",
        "/v1/orders/{id}",
        "/v1/payments",
        "/healthz",
        "/v1/users/{id}/profile",
    ];
    let services = ["checkout", "orders", "payments", "gateway"];
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ batch as u64);
    let log_records = (0..records)
        .map(|index| {
            let seq = batch * records + index;
            let template = lines[rng.below(lines.len())];
            let body = template.replace("{}", &rng.below(9999).to_string());
            LogRecord {
                time_unix_nano: 1_700_000_000_000_000_000 + seq as u64 * 1_000_000,
                severity_number: 9,
                severity_text: "INFO".to_string(),
                body: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(format!(
                        "{body} request_id={} trace_id={}",
                        rng.hex(16),
                        rng.hex(32)
                    ))),
                }),
                attributes: vec![
                    attribute("http.route", routes[rng.below(routes.len())]),
                    attribute(
                        "net.peer.ip",
                        &format!(
                            "10.{}.{}.{}",
                            rng.below(256),
                            rng.below(256),
                            rng.below(256)
                        ),
                    ),
                    attribute("user.id", &format!("user-{}", rng.below(1_000_000))),
                    attribute("span.id", &rng.hex(16)),
                ],
                ..Default::default()
            }
        })
        .collect();

    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![
                    attribute("service.name", services[batch % services.len()]),
                    attribute("deployment.environment", "production"),
                    attribute("host.name", &format!("node-{:03}", batch % 64)),
                ],
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                log_records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

#[derive(Default)]
struct Counts {
    offered: AtomicU64,
    accepted: AtomicU64,
    failed: AtomicU64,
    sink_requests: AtomicU64,
    sink_bytes: AtomicU64,
    sink_refused: AtomicU64,
}

/// The sink stands where signy stands: it takes a segment, reads all of it,
/// and answers with the segment number it now holds. During an outage it
/// refuses with a `503`, which is a retry for collecty rather than a drop.
async fn sink(
    request: Request<Incoming>,
    counts: Arc<Counts>,
    down: Arc<AtomicBool>,
) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    let segment = request
        .headers()
        .get("x-collecty-segment")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if request.method() != Method::POST {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::new()))
            .unwrap());
    }
    let body = request.into_body().collect().await;
    let bytes = body
        .map(|collected| collected.to_bytes().len())
        .unwrap_or(0);

    if down.load(Ordering::Relaxed) {
        counts.sink_refused.fetch_add(1, Ordering::Relaxed);
        return Ok(Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Full::new(Bytes::from_static(b"the sink is away")))
            .unwrap());
    }

    counts.sink_requests.fetch_add(1, Ordering::Relaxed);
    counts.sink_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from(format!("{{\"stored\":{segment}}}"))))
        .unwrap())
}

/// One export, timed from the client's side.
///
/// `micros` is the request going out to the response coming back, which is
/// what the exporter beside collecty waits for. `t` is when it went out, so
/// the outage window can be cut out at report time rather than guessed at:
/// while the sink is away the far end is refusing and the backlog policy
/// decides what ingest feels, which is a different experiment from how many
/// threads the runtime has.
struct Sample {
    t: f32,
    micros: u32,
    path: &'static str,
    ok: bool,
}

/// Percentiles and the histogram they came from.
///
/// The histogram is kept because a p99 on its own cannot say what the tail
/// did: doubling and a long thin tail read the same in one number and are not
/// the same regression. Buckets are powers of two in microseconds, which is
/// one `leading_zeros` per sample and fine enough to see a shape.
fn latency_json(samples: &[&Sample]) -> String {
    if samples.is_empty() {
        return "null".to_string();
    }
    let mut micros: Vec<u32> = samples.iter().map(|s| s.micros).collect();
    micros.sort_unstable();
    let at = |q: f64| -> u32 {
        let index = ((micros.len() as f64 - 1.0) * q).round() as usize;
        micros[index]
    };
    // 33 and not 32: a sample at `u32::MAX` has no leading zeros and lands in
    // the last bucket, which would be one past the end of a 32-wide array.
    let mut buckets = [0u64; 33];
    for value in &micros {
        buckets[(32 - value.max(&1).leading_zeros()) as usize] += 1;
    }
    let hist: Vec<String> = buckets
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
        .map(|(index, count)| format!("[{}, {}]", 1u64 << index, count))
        .collect();
    format!(
        "{{\"count\": {}, \"p50_us\": {}, \"p95_us\": {}, \"p99_us\": {}, \
\"max_us\": {}, \"hist_le_us\": [{}]}}",
        micros.len(),
        at(0.50),
        at(0.95),
        at(0.99),
        micros[micros.len() - 1],
        hist.join(", ")
    )
}

/// When the run started and when it stops. Together because a sample is timed
/// against the first and the loop ends on the second.
#[derive(Clone, Copy)]
struct Window {
    started: Instant,
    until: Instant,
}

/// One worker, one TCP connection, its own share of the rate.
///
/// A connection of its own rather than a pooled client: the scaling test of
/// `MEMORY.md` §4 varies exactly this number, so it has to mean what it says.
async fn drive(
    address: String,
    path: &'static str,
    bodies: Arc<Vec<Vec<u8>>>,
    mut offset: usize,
    period: Duration,
    window: Window,
    counts: Arc<Counts>,
) -> Vec<Sample> {
    let mut samples = Vec::new();
    let mut sender: Option<hyper::client::conn::http1::SendRequest<Full<Bytes>>> = None;
    let mut ticker = tokio::time::interval(period);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
    while Instant::now() < window.until {
        ticker.tick().await;
        if sender.as_ref().is_none_or(|s| s.is_closed()) {
            let Ok(stream) = TcpStream::connect(&address).await else {
                counts.failed.fetch_add(1, Ordering::Relaxed);
                continue;
            };
            let _ = stream.set_nodelay(true);
            let Ok((new_sender, connection)) =
                hyper::client::conn::http1::handshake(TokioIo::new(stream)).await
            else {
                counts.failed.fetch_add(1, Ordering::Relaxed);
                continue;
            };
            tokio::spawn(async move {
                let _ = connection.await;
            });
            sender = Some(new_sender);
        }

        let body = &bodies[offset % bodies.len()];
        offset += 1;
        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("http://{address}{path}"))
            .header(http::header::HOST, address.as_str())
            .header(http::header::CONTENT_TYPE, "application/x-protobuf")
            .body(Full::new(Bytes::from(body.clone())))
            .expect("a well-formed export");

        counts.offered.fetch_add(1, Ordering::Relaxed);
        let sent = Instant::now();
        let ok = match sender
            .as_mut()
            .expect("connected")
            .send_request(request)
            .await
        {
            Ok(response) => {
                let status = response.status();
                // The answer is not the answer until its body is done with:
                // collecty's success is an empty body, but a refusal carries
                // text, and stopping at the status would time them differently.
                let _ = response.into_body().collect().await;
                if status.is_success() {
                    counts.accepted.fetch_add(1, Ordering::Relaxed);
                    true
                } else {
                    counts.failed.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
            Err(_) => {
                counts.failed.fetch_add(1, Ordering::Relaxed);
                sender = None;
                false
            }
        };
        samples.push(Sample {
            t: sent.duration_since(window.started).as_secs_f32(),
            micros: sent.elapsed().as_micros().min(u32::MAX as u128) as u32,
            path,
            ok,
        });
    }
    samples
}

#[tokio::main]
async fn main() {
    let args = args();
    let counts = Arc::new(Counts::default());
    let down = Arc::new(AtomicBool::new(false));

    let listener = TcpListener::bind(args.sink).await.expect("the sink binds");
    {
        let counts = counts.clone();
        let down = down.clone();
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    continue;
                };
                let counts = counts.clone();
                let down = down.clone();
                tokio::spawn(async move {
                    let service =
                        service_fn(move |request| sink(request, counts.clone(), down.clone()));
                    let _ = server_http1::Builder::new()
                        .serve_connection(TokioIo::new(stream), service)
                        .await;
                });
            }
        });
    }

    // A pool of distinct exports, cycled: a real collector never sends the
    // same batch twice, and a segment full of one repeated body would
    // compress like nothing a deployment ever produces.
    let exports_per_second = (args.eps as f64 / args.records_per_export as f64).max(1.0);
    let bodies: Arc<Vec<Vec<u8>>> = Arc::new(
        (0..512)
            .map(|batch| export(args.records_per_export, batch))
            .collect(),
    );
    let body_bytes: usize = bodies.iter().map(|body| body.len()).sum();
    // What a segment would achieve on this corpus. Printed because it decides
    // segment size, and segment size is what the send path allocates per
    // delivery: a corpus that compresses too well measures the wrong
    // collector. The 24-hour soak's collector achieved 4.87x.
    let sample: Vec<u8> = bodies
        .iter()
        .take(64)
        .flat_map(|body| {
            let mut framed = (body.len() as u32).to_le_bytes().to_vec();
            framed.extend_from_slice(body);
            framed
        })
        .collect();
    let ratio = sample.len() as f64
        / zstd::encode_all(sample.as_slice(), 3)
            .expect("the sample compresses")
            .len() as f64;
    eprintln!(
        "memrig: {} exports/s of {} records, mean body {} B, zstd(3) {:.2}x, {} connections, \
{} trace eps, a scrape every {}s, {} s",
        exports_per_second as u64,
        args.records_per_export,
        body_bytes / bodies.len(),
        ratio,
        args.connections,
        args.trace_eps,
        args.metric_every,
        args.seconds,
    );

    let started = Instant::now();
    let window = Window {
        started,
        until: started + Duration::from_secs(args.seconds),
    };
    let per_worker = exports_per_second / args.connections as f64;
    let period = Duration::from_secs_f64(1.0 / per_worker);

    if let Some(at) = args.outage_at {
        let down = down.clone();
        let outage_for = args.outage_for;
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(at)).await;
            eprintln!("memrig: the sink is away at t={at}s for {outage_for}s");
            down.store(true, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_secs(outage_for)).await;
            down.store(false, Ordering::Relaxed);
            eprintln!("memrig: the sink is back at t={}s", at + outage_for);
        });
    }

    let mut workers: Vec<_> = (0..args.connections)
        .map(|index| {
            tokio::spawn(drive(
                args.collecty.clone(),
                "/v1/logs",
                bodies.clone(),
                index * 97,
                period,
                window,
                counts.clone(),
            ))
        })
        .collect();

    // The other two signals, on their own connections and their own rates.
    // They carry almost no bytes and they are not optional: each keeps a
    // segment open that rolls on age rather than size, which is a compressor
    // built and dropped every second per signal.
    let small: Arc<Vec<Vec<u8>>> = Arc::new((0..64).map(|batch| export(2, batch + 9000)).collect());
    if args.trace_eps > 0.0 {
        workers.push(tokio::spawn(drive(
            args.collecty.clone(),
            "/v1/traces",
            small.clone(),
            3,
            Duration::from_secs_f64((2.0 / args.trace_eps).max(0.001)),
            window,
            counts.clone(),
        )));
    }
    if args.metric_every > 0.0 {
        workers.push(tokio::spawn(drive(
            args.collecty.clone(),
            "/v1/metrics",
            small.clone(),
            7,
            Duration::from_secs_f64(args.metric_every),
            window,
            counts.clone(),
        )));
    }

    let mut samples = Vec::new();
    for worker in workers {
        if let Ok(mut theirs) = worker.await {
            samples.append(&mut theirs);
        }
    }
    samples.sort_by(|a, b| a.t.total_cmp(&b.t));

    let elapsed = started.elapsed().as_secs_f64();
    let offered = counts.offered.load(Ordering::Relaxed);
    let accepted = counts.accepted.load(Ordering::Relaxed);

    if let Some(path) = &args.latency_csv {
        let mut csv = String::from("t,path,micros,ok\n");
        for sample in &samples {
            csv.push_str(&format!(
                "{:.3},{},{},{}\n",
                sample.t,
                sample.path,
                sample.micros,
                if sample.ok { 1 } else { 0 }
            ));
        }
        let _ = std::fs::write(path, csv);
    }

    // Only `/v1/logs`, which is the load. The other two signals are there for
    // what they do to segments and compressors, and folding a handful of them
    // per second into a percentile would say nothing about either.
    //
    // Split at the outage, because during it the sink is refusing, the queue
    // is filling and what ingest feels is the backlog policy rather than the
    // runtime. Mixing the windows would let a change to one hide in the other.
    let phase_of = |t: f32| -> &'static str {
        match args.outage_at {
            Some(at) if t < at as f32 => "steady",
            Some(at) if t < (at + args.outage_for) as f32 => "outage",
            Some(_) => "draining",
            None => "steady",
        }
    };
    let phases = ["steady", "outage", "draining"];
    let latency: Vec<String> = phases
        .iter()
        .map(|phase| {
            let of_phase: Vec<&Sample> = samples
                .iter()
                .filter(|s| s.path == "/v1/logs" && phase_of(s.t) == *phase)
                .collect();
            format!("    \"{phase}\": {}", latency_json(&of_phase))
        })
        .collect();

    let report = format!(
        "{{\n  \"seconds\": {elapsed:.2},\n  \"offered_exports\": {offered},\n  \
\"accepted_exports\": {accepted},\n  \"failed_exports\": {},\n  \
\"offered_eps\": {:.0},\n  \"accepted_eps\": {:.0},\n  \
\"sink_requests\": {},\n  \"sink_bytes\": {},\n  \"sink_refused\": {},\n  \
\"ingest_latency\": {{\n{}\n  }}\n}}\n",
        counts.failed.load(Ordering::Relaxed),
        offered as f64 * args.records_per_export as f64 / elapsed,
        accepted as f64 * args.records_per_export as f64 / elapsed,
        counts.sink_requests.load(Ordering::Relaxed),
        counts.sink_bytes.load(Ordering::Relaxed),
        counts.sink_refused.load(Ordering::Relaxed),
        latency.join(",\n"),
    );
    print!("{report}");
    if let Some(path) = args.report {
        let _ = std::fs::write(path, &report);
    }
}
