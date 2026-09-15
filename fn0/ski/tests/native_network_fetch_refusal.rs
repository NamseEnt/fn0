use bytes::Bytes;
use fn0_ski::{FetchHandler, FetchHandlerFuture, Request};
use http_body_util::{BodyExt, Empty};
use std::sync::Arc;
use std::time::Duration;

struct PassThroughHandler {
    allows_native_network_fetch: bool,
}

impl FetchHandler for PassThroughHandler {
    fn handle(&self, _request: Request) -> FetchHandlerFuture {
        Box::pin(async { None })
    }

    fn allows_native_network_fetch(&self) -> bool {
        self.allows_native_network_fetch
    }
}

fn user_js(port: u16) -> String {
    format!(
        r#"
globalThis.handler = async () => {{
  const outcomes = [];
  try {{
    await fetch("http://127.0.0.1:{port}/through-fetch");
    outcomes.push("fetch-succeeded");
  }} catch (error) {{
    outcomes.push("fetch-refused");
  }}
  try {{
    const request = Deno.core.ops.op_fetch("GET", "http://127.0.0.1:{port}/through-op", [], null, false, null, null);
    await Deno.core.ops.op_fetch_send(request.requestRid);
    outcomes.push("op-succeeded");
  }} catch (error) {{
    outcomes.push("op-refused");
  }}
  return new Response(outcomes.join(","));
}};
"#
    )
}

fn empty_request() -> Request {
    let body = http_body_util::combinators::UnsyncBoxBody::new(
        Empty::<Bytes>::new().map_err(|never| match never {}),
    );
    hyper::Request::builder()
        .method("GET")
        .uri("http://localhost/")
        .body(body)
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn refused_native_fetch_never_reaches_the_network() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let response = fn0_ski::run(
        &user_js(port),
        "/native_network_fetch_refusal.js",
        empty_request(),
        Some(Arc::new(PassThroughHandler {
            allows_native_network_fetch: false,
        })),
    )
    .await
    .unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"fetch-refused,op-refused");
    assert!(
        tokio::time::timeout(Duration::from_millis(100), listener.accept())
            .await
            .is_err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn allowed_native_fetch_reaches_the_network() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_task = tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut request_paths = Vec::new();
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0_u8; 4096];
            let read_count = stream.read(&mut buffer).await.unwrap();
            let request_text = String::from_utf8_lossy(&buffer[..read_count]).to_string();
            request_paths.push(request_text.split(' ').nth(1).unwrap_or("").to_string());
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
                .await
                .unwrap();
        }
        request_paths
    });
    let response = fn0_ski::run(
        &user_js(port),
        "/native_network_fetch_allowed.js",
        empty_request(),
        Some(Arc::new(PassThroughHandler {
            allows_native_network_fetch: true,
        })),
    )
    .await
    .unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"fetch-succeeded,op-succeeded");
    assert_eq!(
        server_task.await.unwrap(),
        ["/through-fetch".to_string(), "/through-op".to_string()]
    );
}
