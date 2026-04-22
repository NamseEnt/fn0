use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::{CONTENT_TYPE, HOST, HeaderValue};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

async fn run_request(with_host: bool) -> (String, Option<hyper::StatusCode>, Option<String>) {
    let connector = HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(connector);

    let body_bytes = Bytes::from_static(b"grant_type=authorization_code&code=FAKE_CODE_FOR_TEST");
    let mut builder = hyper::Request::builder()
        .method("POST")
        .uri("https://oauth2.googleapis.com/token")
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
    if with_host {
        builder = builder.header(HOST, HeaderValue::from_static("oauth2.googleapis.com"));
    }
    let req = builder.body(Full::new(body_bytes)).unwrap();

    let label = if with_host { "with-host" } else { "no-host" };
    match client.request(req).await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.into_body().collect().await.ok().map(|c| {
                String::from_utf8_lossy(&c.to_bytes())
                    .chars()
                    .take(400)
                    .collect()
            });
            (label.to_string(), Some(status), body)
        }
        Err(err) => {
            let txt = format!("{err:?}");
            (label.to_string(), None, Some(txt))
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "hits real oauth2.googleapis.com; run manually with --ignored"]
async fn host_header_with_h2_google() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    println!("=== Test A: POST to oauth2.googleapis.com/token WITH host header ===");
    let (label_a, status_a, body_a) = run_request(true).await;
    println!("[{label_a}] status={status_a:?}");
    println!("[{label_a}] body/err={body_a:?}");

    println!();
    println!("=== Test B: POST to oauth2.googleapis.com/token WITHOUT host header ===");
    let (label_b, status_b, body_b) = run_request(false).await;
    println!("[{label_b}] status={status_b:?}");
    println!("[{label_b}] body/err={body_b:?}");

    println!();
    println!("== SUMMARY ==");
    println!("  with-host: status={status_a:?}");
    println!("  no-host  : status={status_b:?}");
}
