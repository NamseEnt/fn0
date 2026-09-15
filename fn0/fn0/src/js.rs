use crate::execute::WasmInjectEnvelope;
use crate::outbound_http::GuestOutboundHttp;
use crate::self_invoke;
use crate::{Body, Request, Response};
use anyhow::anyhow;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::http::header;
use ski::{FetchHandler, FetchHandlerFuture};
use std::sync::Arc;
use tokio::sync::mpsc;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

pub(crate) const SELF_INVOKE_PATH_PREFIX: &str = "/__self_invoke/";

pub(crate) struct WasmForwardingFetchHandler {
    sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
    project_id: String,
    guest_outbound_http: Option<Arc<GuestOutboundHttp>>,
}

impl WasmForwardingFetchHandler {
    pub(crate) fn new(
        sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
        project_id: String,
        guest_outbound_http: Option<Arc<GuestOutboundHttp>>,
    ) -> Self {
        Self {
            sender,
            project_id,
            guest_outbound_http,
        }
    }
}

impl FetchHandler for WasmForwardingFetchHandler {
    fn handle(&self, req: Request) -> FetchHandlerFuture {
        let path = req.uri().path().to_string();
        if path.starts_with(SELF_INVOKE_PATH_PREFIX) {
            let sender = self.sender.clone();
            return Box::pin(async move {
                match self_invoke::call_wasm_direct(sender, req).await {
                    Ok(resp) => Some(resp),
                    Err(e) => Some(error_response(500, &format!("self-invoke failed: {e}"))),
                }
            });
        }
        let network_scheme = matches!(req.uri().scheme_str(), Some("http" | "https"));
        let Some(guest_outbound_http) = self.guest_outbound_http.clone().filter(|_| network_scheme)
        else {
            return Box::pin(async { None });
        };
        let project_id = self.project_id.clone();
        Box::pin(
            async move { Some(send_external_fetch(guest_outbound_http, project_id, req).await) },
        )
    }

    fn allows_native_network_fetch(&self) -> bool {
        self.guest_outbound_http.is_none()
    }
}

async fn send_external_fetch(
    guest_outbound_http: Arc<GuestOutboundHttp>,
    project_id: String,
    request: Request,
) -> Response {
    let request = request.map(|body| {
        body.map_err(|error| ErrorCode::InternalError(Some(error.to_string())))
            .boxed_unsync()
    });
    match guest_outbound_http.send(&project_id, request, None).await {
        Ok((response, transmit)) => {
            tokio::spawn(async move {
                let _ = transmit.await;
            });
            response.map(|body| {
                body.map_err(|error_code| anyhow!("fetch response body: {error_code:?}"))
                    .boxed_unsync()
            })
        }
        Err(ErrorCode::DestinationIpProhibited) => {
            error_response(403, "fetch destination is not a public internet address")
        }
        Err(ErrorCode::HttpRequestDenied) => error_response(429, "monthly egress quota exhausted"),
        Err(error_code) => error_response(502, &format!("fetch failed: {error_code:?}")),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::egress::test_budget::CountingBudget;
    use crate::outbound_destination::{OutboundDialer, PrivateDestinationAccess};
    use http_body_util::Empty;

    fn fetch_request(uri: &str) -> Request {
        hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
            )
            .unwrap()
    }

    #[tokio::test]
    async fn guarded_handler_answers_private_fetch_with_forbidden_response() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let (sender, _receiver) = mpsc::unbounded_channel();
        let handler = WasmForwardingFetchHandler::new(
            sender,
            "project".to_string(),
            Some(Arc::new(GuestOutboundHttp::new(
                OutboundDialer::system(PrivateDestinationAccess::Blocked),
                CountingBudget::with_remaining(1_000),
                "fn0-control".to_string(),
            ))),
        );
        assert!(!handler.allows_native_network_fetch());
        let response = handler
            .handle(fetch_request(&format!("http://127.0.0.1:{listener_port}/")))
            .await
            .expect("guarded handler answers network fetches");
        assert_eq!(response.status(), 403);
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn unguarded_handler_leaves_network_fetch_to_the_runtime() {
        let (sender, _receiver) = mpsc::unbounded_channel();
        let handler = WasmForwardingFetchHandler::new(sender, "project".to_string(), None);
        assert!(handler.allows_native_network_fetch());
        assert!(
            handler
                .handle(fetch_request("https://example.com/"))
                .await
                .is_none()
        );
    }
}
