use bytes::Bytes;
use fn0_ski::{FetchHandler, FetchHandlerFuture, Request, SkiInstance};
use http_body_util::{BodyExt, Empty};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::LocalSet;

// A cold instance carries the bundle's deferred module instantiation, so it
// runs on a far larger budget than a warm one. Spending that here would make
// the test wait out the cold budget instead of the per-call one, so the first
// call returns immediately and the spin happens on the second.
//
// The spin has to sit behind an await: `SkiInstance::call` registers the
// deadline only once the handler has suspended, so a handler that loops before
// its first await is never watched at all.
fn user_js() -> &'static str {
    r#"
let calls = 0;
globalThis.handler = async () => {
  calls += 1;
  if (calls === 1) {
    return new Response("warm");
  }
  await fetch("http://spin.invalid/");
  while (true) {}
};
"#
}

struct StubFetch;

impl FetchHandler for StubFetch {
    fn handle(&self, _request: Request) -> FetchHandlerFuture {
        Box::pin(async move {
            let body = http_body_util::combinators::UnsyncBoxBody::new(
                http_body_util::Full::new(Bytes::from_static(b"ok"))
                    .map_err(|never: std::convert::Infallible| match never {}),
            );
            Some(hyper::Response::builder().status(200).body(body).unwrap())
        })
    }
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

// Contract under test: a call that outruns its CPU budget leaves the isolate
// dead, and the instance says so. Callers cache one instance per project and
// reuse it across requests, so without a readable termination flag a single
// over-budget render takes every later request down with it.
#[test]
fn over_budget_call_marks_the_instance_terminated() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local = LocalSet::new();

    local.block_on(&runtime, async {
        let instance =
            Rc::new(SkiInstance::load(user_js(), "/entry.js", Some(Arc::new(StubFetch))).unwrap());
        let driver_instance = instance.clone();
        let driver = tokio::task::spawn_local(async move {
            driver_instance.drive_forever().await;
        });

        let warm = instance.call(empty_request()).await;
        assert!(warm.is_ok(), "first call failed: {:?}", warm.err());
        assert!(!instance.is_terminated());

        let spun = tokio::time::timeout(Duration::from_secs(30), instance.call(empty_request()))
            .await
            .expect("watchdog never cut the over-budget call");
        assert!(spun.is_err(), "over-budget call was expected to fail");
        assert!(
            instance.is_terminated(),
            "instance must report termination so the caller respawns it"
        );

        let after = tokio::time::timeout(Duration::from_secs(30), instance.call(empty_request()))
            .await
            .expect("call on a terminated instance hung");
        assert!(
            after.is_err(),
            "a terminated isolate cannot serve later calls"
        );

        driver.abort();
    });
}
