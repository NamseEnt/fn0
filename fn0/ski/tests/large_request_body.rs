use bytes::Bytes;
use fn0_ski::{Request, SkiInstance};
use http_body_util::BodyExt;
use std::rc::Rc;
use std::time::Duration;
use tokio::task::LocalSet;

// Comfortably past deno_core's read buffer, which starts at 64 KiB.
const BODY_SIZE: usize = 300 * 1024;

fn user_js() -> &'static str {
    r#"
globalThis.handler = async (request) => {
  const text = await request.text();
  return new Response(String(text.length), { status: 200 });
};
"#
}

fn request_with_body(body_bytes: Bytes) -> Request {
    let body = http_body_util::combinators::UnsyncBoxBody::new(
        http_body_util::Full::new(body_bytes)
            .map_err(|never: std::convert::Infallible| match never {}),
    );
    hyper::Request::builder()
        .method("POST")
        .uri("http://localhost/")
        .body(body)
        .unwrap()
}

// Contract under test: a request body frame larger than the reader's buffer is
// handed over across several reads. deno_core's default `read_byob` copies a
// `read` result into a fixed buffer with no length check, from inside an op
// that cannot unwind, so a `read` that ignores its limit does not raise a JS
// error — it aborts the process. Forte posts a page's props as one frame, so
// any page whose props outgrow the buffer took the whole worker down with it.
#[test]
fn request_body_larger_than_the_read_buffer_is_delivered_whole() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local = LocalSet::new();

    local.block_on(&runtime, async {
        let instance = Rc::new(SkiInstance::load(user_js(), "/entry.js", None).unwrap());
        let driver_instance = instance.clone();
        let driver = tokio::task::spawn_local(async move {
            driver_instance.drive_forever().await;
        });

        let body = Bytes::from(vec![b'a'; BODY_SIZE]);
        let response = tokio::time::timeout(
            Duration::from_secs(30),
            instance.call(request_with_body(body)),
        )
        .await
        .expect("call hung")
        .expect("call failed");

        let read_back = response
            .into_body()
            .collect()
            .await
            .expect("body collect failed")
            .to_bytes();
        assert_eq!(
            std::str::from_utf8(&read_back).unwrap(),
            BODY_SIZE.to_string(),
            "the handler saw a truncated body"
        );

        driver.abort();
    });
}
