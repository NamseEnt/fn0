use crate::cross_project_invoke_hijack::CrossProjectInvokeHijack;
use crate::cross_project_enqueue_hijack::CrossProjectEnqueueHijack;
use crate::execute::{ClientState, WasmInjectEnvelope};
use crate::measure_cpu_time::{Clock, TimeTracker, measure_cpu_time};
use crate::otlp_hijack::OtlpHijack;
use crate::queue_hijack::QueueHijack;
use crate::turso_hijack::TursoHijack;
use crate::vault_hijack::VaultHijack;
use crate::{Request, Response, telemetry};
use anyhow::{Result, anyhow};
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::http;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, oneshot};
use wasmtime::AsContextMut;
use wasmtime::component::Accessor;
use wasmtime_wasi::TrappableError;
use wasmtime_wasi_http::p3::Request as P3Request;
use wasmtime_wasi_http::p3::bindings::Service;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::p3::{RequestOptions, WasiHttpHooks, default_send_request};

tokio::task_local! {
    pub(crate) static SELF_HOST: String;
}

// Inject a request into the wasm instance loop (same project) and await its
// response via oneshot. Used by both call_wasm_direct (JS fetch -> wasm) and
// self_invoke_send (wasm wasi-http hook -> wasm). Replaces the previous raw
// pointer / WASM_RAW task_local plumbing — no unsafe, no
// `accessor.lifetime`-dependent state across tasks.
async fn inject_and_await(
    sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
    req: Request,
) -> Result<Response> {
    let (resp_tx, resp_rx) = oneshot::channel();
    if sender.send((req, resp_tx)).is_err() {
        return Err(anyhow!("self-invoke target wasm instance channel closed"));
    }
    resp_rx
        .await
        .unwrap_or_else(|_| Err(anyhow!("self-invoke target dropped response")))
}

pub async fn call_wasm_direct(
    sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
    req: Request,
) -> Result<Response> {
    inject_and_await(sender, req).await
}

pub(crate) fn extract_host(headers: &hyper::HeaderMap) -> Option<String> {
    let value = headers.get(hyper::header::HOST)?;
    let s = value.to_str().ok()?;
    Some(normalize_host(s))
}

fn normalize_host(host: &str) -> String {
    host.split(':').next().unwrap_or(host).to_ascii_lowercase()
}

fn matches_self(uri: &http::Uri, self_host: &str) -> bool {
    let Some(host) = uri.host() else { return false };
    host.eq_ignore_ascii_case(self_host)
}

type HookResponse = (
    http::Response<UnsyncBoxBody<Bytes, ErrorCode>>,
    Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send>,
);
type HookResult = std::result::Result<HookResponse, TrappableError<ErrorCode>>;

pub(crate) struct SelfInvokeHooks {
    project_id: String,
    self_invoke_sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
    turso_hijack: Option<Arc<TursoHijack>>,
    otlp_hijack: Option<Arc<OtlpHijack>>,
    queue_hijack: Option<Arc<QueueHijack>>,
    cross_project_enqueue_hijack: Option<Arc<CrossProjectEnqueueHijack>>,
    cross_project_invoke_hijack: Option<Arc<CrossProjectInvokeHijack>>,
    vault_hijack: Option<Arc<VaultHijack>>,
}

impl SelfInvokeHooks {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        project_id: String,
        self_invoke_sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
        turso_hijack: Option<Arc<TursoHijack>>,
        otlp_hijack: Option<Arc<OtlpHijack>>,
        queue_hijack: Option<Arc<QueueHijack>>,
        cross_project_enqueue_hijack: Option<Arc<CrossProjectEnqueueHijack>>,
        cross_project_invoke_hijack: Option<Arc<CrossProjectInvokeHijack>>,
        vault_hijack: Option<Arc<VaultHijack>>,
    ) -> Self {
        Self {
            project_id,
            self_invoke_sender,
            turso_hijack,
            otlp_hijack,
            queue_hijack,
            cross_project_enqueue_hijack,
            cross_project_invoke_hijack,
            vault_hijack,
        }
    }
}

impl WasiHttpHooks for SelfInvokeHooks {
    fn send_request(
        &mut self,
        request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
        options: Option<RequestOptions>,
        _fut: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send>,
    ) -> Box<dyn Future<Output = HookResult> + Send> {
        let self_host = SELF_HOST.try_with(|h| h.clone()).ok();
        let is_self = self_host
            .as_deref()
            .map(|h| matches_self(request.uri(), h))
            .unwrap_or(false);

        if is_self {
            return self_invoke_send(self.self_invoke_sender.clone(), request);
        }

        if let Some(hijack) = self.turso_hijack.clone()
            && hijack.matches(request.uri())
        {
            return turso_send(hijack, self.project_id.clone(), request, options);
        }

        if let Some(hijack) = self.queue_hijack.clone()
            && hijack.matches(request.uri())
        {
            return queue_send(hijack, self.project_id.clone(), request, options);
        }

        if let Some(hijack) = self.cross_project_enqueue_hijack.clone()
            && hijack.matches(request.uri())
        {
            return cross_project_enqueue_send(hijack, self.project_id.clone(), request, options);
        }

        if let Some(hijack) = self.cross_project_invoke_hijack.clone()
            && hijack.matches(request.uri())
        {
            return cross_project_invoke_send(hijack, self.project_id.clone(), request);
        }

        if let Some(hijack) = self.vault_hijack.clone()
            && hijack.matches(request.uri())
        {
            return vault_send(hijack, self.project_id.clone(), request, options);
        }

        if let Some(hijack) = self.otlp_hijack.clone()
            && hijack.matches(request.uri())
        {
            return otlp_send(hijack, self.project_id.clone(), request, options);
        }

        default_send(self.project_id.clone(), request, options)
    }
}

fn self_invoke_send(
    sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let req: Request = request.map(|body| {
            body.map_err(|ec: ErrorCode| anyhow!("error_code: {ec:?}"))
                .boxed_unsync()
        });
        let resp = match inject_and_await(sender, req).await {
            Ok(r) => r,
            Err(e) => return Err(ErrorCode::InternalError(Some(format!("{e:?}"))).into()),
        };
        let http_resp = resp.map(|body| {
            body.map_err(|err: anyhow::Error| {
                ErrorCode::InternalError(Some(err.to_string()))
            })
            .boxed_unsync()
        });
        let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
            Box::new(async { Ok(()) });
        Ok((http_resp, io))
    })
}

fn turso_send(
    hijack: Arc<TursoHijack>,
    project_id: String,
    mut request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        if let Err(e) = hijack.rewrite(&mut request, &project_id) {
            return Err(e.into());
        }

        let send_start = std::time::Instant::now();
        let (res, io) = default_send_request(request, options).await?;
        telemetry::stage_duration("hijack_turso", &project_id, send_start.elapsed());
        let res = res.map(BodyExt::boxed_unsync);
        let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> = Box::new(io);
        Ok((res, io))
    })
}

fn queue_send(
    hijack: Arc<QueueHijack>,
    project_id: String,
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let (_parts, body) = request.into_parts();
        let body_bytes = match body.collect().await {
            Ok(c) => c.to_bytes(),
            Err(e) => return Err(ErrorCode::InternalError(Some(format!("{e:?}"))).into()),
        };

        let action = match hijack.handle_enqueue(&project_id, &body_bytes) {
            Ok(a) => a,
            Err(ec) => return Err(ec.into()),
        };

        hijack.record_usage(&project_id);

        match action {
            crate::queue_hijack::HijackAction::Forward(signed) => {
                let send_start = std::time::Instant::now();
                let (res, io) = default_send_request(signed, options).await?;
                telemetry::stage_duration("hijack_queue", &project_id, send_start.elapsed());
                let res = res.map(BodyExt::boxed_unsync);
                let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
                    Box::new(io);
                Ok((res, io))
            }
            crate::queue_hijack::HijackAction::Synthesized(resp) => {
                let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
                    Box::new(async { Ok(()) });
                Ok((resp, io))
            }
        }
    })
}

fn cross_project_enqueue_send(
    hijack: Arc<CrossProjectEnqueueHijack>,
    project_id: String,
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let (_parts, body) = request.into_parts();
        let body_bytes = match body.collect().await {
            Ok(c) => c.to_bytes(),
            Err(e) => return Err(ErrorCode::InternalError(Some(format!("{e:?}"))).into()),
        };

        let action = match hijack.handle_enqueue(&project_id, &body_bytes) {
            Ok(a) => a,
            Err(ec) => return Err(ec.into()),
        };

        match action {
            crate::cross_project_enqueue_hijack::HijackAction::Forward(signed) => {
                let send_start = std::time::Instant::now();
                let (res, io) = default_send_request(signed, options).await?;
                telemetry::stage_duration(
                    "hijack_cross_project_enqueue",
                    &project_id,
                    send_start.elapsed(),
                );
                let res = res.map(BodyExt::boxed_unsync);
                let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
                    Box::new(io);
                Ok((res, io))
            }
            crate::cross_project_enqueue_hijack::HijackAction::Synthesized(resp) => {
                let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
                    Box::new(async { Ok(()) });
                Ok((resp, io))
            }
        }
    })
}

fn cross_project_invoke_send(
    hijack: Arc<CrossProjectInvokeHijack>,
    caller_project_id: String,
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let send_start = std::time::Instant::now();
        let resp = match hijack.handle_invoke(&caller_project_id, request).await {
            Ok(r) => r,
            Err(ec) => return Err(ec.into()),
        };
        telemetry::stage_duration(
            "hijack_cross_project_invoke",
            &caller_project_id,
            send_start.elapsed(),
        );
        let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
            Box::new(async { Ok(()) });
        Ok((resp, io))
    })
}

fn vault_send(
    hijack: Arc<VaultHijack>,
    project_id: String,
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let (parts, body) = request.into_parts();
        let body_bytes = match body.collect().await {
            Ok(c) => c.to_bytes(),
            Err(e) => return Err(ErrorCode::InternalError(Some(format!("{e:?}"))).into()),
        };

        let method = parts.method.as_str();
        let path = parts
            .uri
            .path_and_query()
            .map(|pq| pq.path())
            .unwrap_or("/");

        let signed = match hijack.build_signed_request(&project_id, method, path, &body_bytes) {
            Ok(req) => req,
            Err(ec) => return Err(ec.into()),
        };

        let send_start = std::time::Instant::now();
        let (res, io) = default_send_request(signed, options).await?;
        telemetry::stage_duration("hijack_vault", &project_id, send_start.elapsed());
        let res = res.map(BodyExt::boxed_unsync);
        let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> = Box::new(io);
        Ok((res, io))
    })
}

fn otlp_send(
    hijack: Arc<OtlpHijack>,
    project_id: String,
    mut request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        if let Err(e) = hijack.rewrite(&mut request) {
            return Err(e.into());
        }

        let (parts, body) = request.into_parts();
        let body_bytes = match body.collect().await {
            Ok(c) => c.to_bytes(),
            Err(e) => return Err(ErrorCode::InternalError(Some(format!("{e:?}"))).into()),
        };
        let forward_body = http_body_util::Full::new(body_bytes)
            .map_err(|never: std::convert::Infallible| match never {})
            .boxed_unsync();
        let forward_request = http::Request::from_parts(parts, forward_body);

        tokio::task::spawn_local(async move {
            let send_start = std::time::Instant::now();
            match default_send_request(forward_request, options).await {
                Ok((_resp, io)) => {
                    let _ = io.await;
                    telemetry::stage_duration("hijack_otlp", &project_id, send_start.elapsed());
                }
                Err(err) => {
                    tracing::warn!(?err, "otlp forward failed");
                }
            }
        });

        let response = http::Response::builder()
            .status(202)
            .body(
                http_body_util::Empty::<Bytes>::new()
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
            )
            .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
        let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
            Box::new(async { Ok(()) });
        Ok((response, io))
    })
}

fn default_send(
    project_id: String,
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let send_start = std::time::Instant::now();
        let (res, io) = default_send_request(request, options).await?;
        telemetry::stage_duration("outbound_fetch", &project_id, send_start.elapsed());
        let res = res.map(BodyExt::boxed_unsync);
        let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> = Box::new(io);
        Ok((res, io))
    })
}

pub(crate) async fn call_service<C: Clock>(
    accessor: &Accessor<ClientState<C>>,
    service: &Service,
    p3_req: P3Request,
    req_io: impl Future<Output = std::result::Result<(), ErrorCode>> + Send + 'static,
    project_id: &str,
    time_tracker: TimeTracker<C>,
    is_timeout: &Arc<AtomicBool>,
) -> Result<Response> {
    let handle_fut = service.handle(accessor, p3_req);
    let handle_result = measure_cpu_time(time_tracker, handle_fut).await;

    match handle_result {
        Ok(Ok(resp)) => {
            let http_resp = accessor
                .with(|mut access| resp.into_http(access.as_context_mut(), req_io))
                .map_err(|error| {
                    telemetry::wasmtime_error("response_into_http", project_id, &format!("{error:?}"));
                    anyhow!("response into_http failed: {error:?}")
                })?;
            Ok(http_resp.map(|body| {
                body.map_err(|ec| anyhow!("error_code: {ec:?}"))
                    .boxed_unsync()
            }))
        }
        Ok(Err(ec)) => {
            telemetry::proxy_returns_error_code(project_id, &format!("{ec:?}"));
            Err(anyhow!("proxy returned error code: {ec:?}"))
        }
        Err(error) => Err(classify_wasm_error(error, project_id, is_timeout)),
    }
}

pub(crate) fn classify_wasm_error(
    error: wasmtime::Error,
    project_id: &str,
    is_timeout: &Arc<AtomicBool>,
) -> anyhow::Error {
    match error.downcast::<wasmtime::Trap>() {
        Ok(trap) => {
            telemetry::trapped(project_id, &format!("{trap:?}"));
            if is_timeout.load(Ordering::Relaxed) {
                anyhow!("CPU time limit exceeded (trapped: {trap:?})")
            } else {
                anyhow!("wasm trapped: {trap:?}")
            }
        }
        Err(error) => {
            telemetry::canceled_unexpectedly(project_id, &format!("{error:?}"));
            anyhow!("wasm error: {error:?}")
        }
    }
}
