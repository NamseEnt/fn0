use bytes::Bytes;
use fn0_ski::{Request, SkiInstance};
use http_body_util::{BodyExt, Empty};
use std::rc::Rc;
use std::time::Duration;
use tokio::task::LocalSet;

const CHUNK_COUNT: usize = 100;
const CHUNK_SIZE: usize = 1024;

fn user_js() -> String {
    format!(
        r#"
globalThis.handler = () => {{
  let sent = 0;
  const stream = new ReadableStream({{
    pull(controller) {{
      if (sent === {CHUNK_COUNT}) {{
        controller.close();
        return;
      }}
      controller.enqueue(new Uint8Array({CHUNK_SIZE}).fill(65));
      sent += 1;
    }},
  }});
  return new Response(stream, {{ status: 200 }});
}};
"#
    )
}

fn empty_request(url: &str) -> Request {
    let body = http_body_util::combinators::UnsyncBoxBody::new(
        Empty::<Bytes>::new().map_err(|never| match never {}),
    );
    hyper::Request::builder()
        .method("GET")
        .uri(url)
        .body(body)
        .unwrap()
}

// Contract under test: the Response returned by SkiInstance::call must be
// safe to collect from a thread other than the isolate's (this mirrors the
// production handoff: worker_pool isolate thread -> hyper connection task).
#[test]
fn response_body_collects_on_foreign_thread() {
    let (resp_tx, resp_rx) = std::sync::mpsc::channel();
    let (done_tx, done_rx) = tokio::sync::oneshot::channel::<()>();

    let isolate_thread = std::thread::Builder::new()
        .name("ski-isolate".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let local = LocalSet::new();
            local.block_on(&rt, async move {
                let instance = Rc::new(SkiInstance::load(&user_js(), "/entry.js", None).unwrap());
                let driver_inst = instance.clone();
                let driver = tokio::task::spawn_local(async move {
                    driver_inst.drive_forever().await;
                });

                let resp = instance
                    .call(empty_request("http://localhost/"))
                    .await
                    .unwrap();
                resp_tx.send(resp).unwrap();

                let _ = done_rx.await;
                driver.abort();
            });
        })
        .unwrap();

    let resp = resp_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("isolate thread produced no response");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let collected = rt.block_on(async {
        tokio::time::timeout(Duration::from_secs(30), resp.into_body().collect()).await
    });

    let _ = done_tx.send(());
    isolate_thread.join().unwrap();

    let bytes = collected
        .expect("body collect timed out")
        .expect("body collect failed")
        .to_bytes();
    assert_eq!(bytes.len(), CHUNK_COUNT * CHUNK_SIZE);
}
