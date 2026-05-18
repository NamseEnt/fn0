//! Sync cross-project invocation hijack.
//!
//! When the allowed caller (e.g. fn0-control) makes an HTTP request to
//! `placeholder_host`, the hijack catches it and dispatches into the
//! worker pool via an injected `CrossProjectInvokeDispatcher`. The Host
//! header's first subdomain selects the target project_id, reusing the
//! worker pool's existing routing. The response is returned to the
//! caller wasm unmodified.
//!
//! In contrast to [`crate::CrossProjectEnqueueHijack`] (fire-and-forget,
//! queue), this hijack is sync: the caller awaits the upstream response.

use crate::{Request, Response};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use http_body_util::combinators::UnsyncBoxBody;
use std::sync::{Arc, OnceLock};
use tokio::sync::oneshot;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

/// Bridge from the hijack (defined in the runtime crate) to the worker
/// pool (defined in the binary crate). Worker provides a concrete impl
/// after `spawn_workers`, then installs it via `set_dispatcher`.
pub trait CrossProjectInvokeDispatcher: Send + Sync {
    fn dispatch(
        &self,
        target_project_id: String,
        req: Request,
    ) -> Result<oneshot::Receiver<Result<Response>>>;
}

#[derive(Clone)]
pub struct CrossProjectInvokeHijack {
    pub placeholder_host: String,
    allowed_caller_project_id: String,
    dispatcher: Arc<OnceLock<Arc<dyn CrossProjectInvokeDispatcher>>>,
}

impl CrossProjectInvokeHijack {
    pub fn new(placeholder_host: String, allowed_caller_project_id: String) -> Self {
        Self {
            placeholder_host,
            allowed_caller_project_id,
            dispatcher: Arc::new(OnceLock::new()),
        }
    }

    pub fn from_env() -> Result<Self> {
        let placeholder_host = std::env::var("FN0_CROSS_PROJECT_INVOKE_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-cross-project-invoke.fn0.dev".to_string());
        let allowed_caller_project_id =
            std::env::var("FN0_CROSS_PROJECT_INVOKE_ALLOWED_CALLER_PROJECT_ID").map_err(
                |_| {
                    anyhow::anyhow!(
                        "FN0_CROSS_PROJECT_INVOKE_ALLOWED_CALLER_PROJECT_ID is required"
                    )
                },
            )?;
        Ok(Self::new(placeholder_host, allowed_caller_project_id))
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub fn allowed_caller_project_id(&self) -> &str {
        &self.allowed_caller_project_id
    }

    /// Install the dispatcher used to route caught requests into the worker
    /// pool. Must be called exactly once after the worker pool is built.
    pub fn set_dispatcher(&self, dispatcher: Arc<dyn CrossProjectInvokeDispatcher>) {
        if self.dispatcher.set(dispatcher).is_err() {
            panic!("CrossProjectInvokeHijack dispatcher already set");
        }
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        let Some(host) = uri.host() else { return false };
        let suffix = format!(".{}", self.placeholder_host);
        host.ends_with(&suffix) && host.len() > suffix.len()
    }

    pub(crate) async fn handle_invoke(
        &self,
        caller_project_id: &str,
        request: hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    ) -> Result<hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
        if caller_project_id != self.allowed_caller_project_id {
            return Ok(synth_response(
                403,
                Bytes::from_static(b"cross project invoke forbidden"),
            ));
        }

        let Some(dispatcher) = self.dispatcher.get() else {
            return Ok(synth_response(
                503,
                Bytes::from_static(b"cross project invoke dispatcher not installed"),
            ));
        };

        let target_project_id = request
            .uri()
            .host()
            .and_then(|h| h.split('.').next())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ErrorCode::InternalError(Some("missing target subdomain".into())))?
            .to_string();

        let req: Request = request.map(|body| {
            body.map_err(|ec: ErrorCode| anyhow::anyhow!("error_code: {ec:?}"))
                .boxed_unsync()
        });

        let resp_rx = dispatcher
            .dispatch(target_project_id, req)
            .map_err(|e| ErrorCode::InternalError(Some(format!("dispatch: {e:#}"))))?;

        let resp = match resp_rx.await {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                return Err(ErrorCode::InternalError(Some(format!("upstream: {e:#}"))));
            }
            Err(_) => {
                return Err(ErrorCode::InternalError(Some(
                    "upstream dropped response".into(),
                )));
            }
        };

        Ok(resp.map(|body| {
            body.map_err(|err: anyhow::Error| ErrorCode::InternalError(Some(err.to_string())))
                .boxed_unsync()
        }))
    }
}

fn synth_response(status: u16, body: Bytes) -> hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>> {
    let body: UnsyncBoxBody<Bytes, ErrorCode> = Full::new(body)
        .map_err(|never: std::convert::Infallible| match never {})
        .boxed_unsync();
    hyper::Response::builder()
        .status(status)
        .body(body)
        .expect("static synth response builds")
}

