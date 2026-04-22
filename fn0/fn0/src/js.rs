use crate::self_invoke;
use crate::{Body, Request, Response};
use anyhow::anyhow;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::http::header;
use ski::{FetchHandler, FetchHandlerFuture};

pub(crate) const SELF_INVOKE_PATH_PREFIX: &str = "/__self_invoke/";

pub(crate) struct WasmForwardingFetchHandler;

impl FetchHandler for WasmForwardingFetchHandler {
    fn handle(&self, req: Request) -> FetchHandlerFuture {
        let path = req.uri().path().to_string();
        if !path.starts_with(SELF_INVOKE_PATH_PREFIX) {
            return Box::pin(async { None });
        }
        Box::pin(async move {
            match self_invoke::call_wasm_direct(req).await {
                Ok(resp) => Some(resp),
                Err(e) => Some(error_response(500, &format!("self-invoke failed: {e}"))),
            }
        })
    }
}

fn error_response(status: u16, message: &str) -> Response {
    let body: Body = Full::new(Bytes::copy_from_slice(message.as_bytes()))
        .map_err(|e| anyhow!("{e}"))
        .boxed_unsync();
    hyper::Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(body)
        .expect("failed to build error response")
}
