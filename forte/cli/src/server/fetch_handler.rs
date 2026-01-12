use bytes::Bytes;
use fn0::{FetchHandler, FetchHandlerFuture, Fn0};
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use std::sync::Arc;

use super::SimpleCache;

pub struct ForteFetchHandler {
    fn0: Arc<Fn0<SimpleCache>>,
}

impl ForteFetchHandler {
    pub fn new(fn0: Arc<Fn0<SimpleCache>>) -> Self {
        Self { fn0 }
    }
}

impl FetchHandler for ForteFetchHandler {
    fn handle(&self, req: fn0::Request) -> FetchHandlerFuture {
        let path = req.uri().path().to_string();

        if !path.starts_with("/__forte_hook/") {
            return Box::pin(async { None });
        }

        let fn0 = self.fn0.clone();

        Box::pin(async move {
            let response = fn0.run("backend", req, None).await;

            match response {
                Ok(resp) => Some(resp),
                Err(e) => {
                    eprintln!("[ForteFetchHandler] Error calling hook: {:?}", e);
                    let body: UnsyncBoxBody<Bytes, anyhow::Error> = Full::new(Bytes::from(format!("Hook error: {}", e)))
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .boxed_unsync();
                    Some(
                        hyper::Response::builder()
                            .status(500)
                            .header("content-type", "text/plain")
                            .body(body)
                            .unwrap(),
                    )
                }
            }
        })
    }
}
