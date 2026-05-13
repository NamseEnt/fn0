use crate::control_invoke_queue_hijack::ControlInvokeQueueHijack;
use crate::execute::ClientState;
use crate::measure_cpu_time::{Clock, SystemClock, TimeTracker, measure_cpu_time};
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
use wasmtime::AsContextMut;
use wasmtime::component::Accessor;
use wasmtime_wasi::TrappableError;
use wasmtime_wasi_http::p3::Request as P3Request;
use wasmtime_wasi_http::p3::bindings::Service;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::p3::{RequestOptions, WasiHttpHooks, default_send_request};

tokio::task_local! {
    pub(crate) static WASM_RAW: WasmRaw;
    pub(crate) static SELF_HOST: String;
}

#[derive(Copy, Clone)]
pub(crate) struct WasmRaw {
    accessor: usize,
    service: usize,
}

impl WasmRaw {
    pub(crate) fn new(
        accessor: &Accessor<ClientState<SystemClock>>,
        service: &Service,
    ) -> Self {
        Self {
            accessor: accessor as *const _ as usize,
            service: service as *const _ as usize,
        }
    }

    unsafe fn accessor(self) -> &'static Accessor<ClientState<SystemClock>> {
        unsafe { &*(self.accessor as *const Accessor<ClientState<SystemClock>>) }
    }

    unsafe fn service(self) -> &'static Service {
        unsafe { &*(self.service as *const Service) }
    }
}

pub(crate) async fn scope_wasm_execution<F, T>(
    accessor: &Accessor<ClientState<SystemClock>>,
    service: &Service,
    future: F,
) -> T
where
    F: Future<Output = T>,
{
    let raw = WasmRaw::new(accessor, service);
    WASM_RAW.scope(raw, future).await
}

pub async fn call_wasm_direct(req: Request) -> Result<Response> {
    let raw = WASM_RAW
        .try_with(|r| *r)
        .map_err(|_| anyhow!("call_wasm_direct invoked outside wasm scope"))?;
    let accessor = unsafe { raw.accessor() };
    let service = unsafe { raw.service() };

    let (time_tracker, is_timeout, project_id) = accessor.with(|mut access| {
        let state = access.data_mut();
        (
            state.time_tracker.clone(),
            state.is_timeout.clone(),
            state.project_id.clone(),
        )
    });

    let req_http = req.map(|body| {
        body.map_err(|err| ErrorCode::InternalError(Some(err.to_string())))
            .boxed_unsync()
    });
    let (p3_req, req_io) = P3Request::from_http(req_http);
    call_service(
        accessor,
        service,
        p3_req,
        req_io,
        &project_id,
        time_tracker,
        &is_timeout,
    )
    .await
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
    turso_hijack: Option<Arc<TursoHijack>>,
    otlp_hijack: Option<Arc<OtlpHijack>>,
    queue_hijack: Option<Arc<QueueHijack>>,
    control_invoke_queue_hijack: Option<Arc<ControlInvokeQueueHijack>>,
    vault_hijack: Option<Arc<VaultHijack>>,
}

impl SelfInvokeHooks {
    pub(crate) fn new(
        project_id: String,
        turso_hijack: Option<Arc<TursoHijack>>,
        otlp_hijack: Option<Arc<OtlpHijack>>,
        queue_hijack: Option<Arc<QueueHijack>>,
        control_invoke_queue_hijack: Option<Arc<ControlInvokeQueueHijack>>,
        vault_hijack: Option<Arc<VaultHijack>>,
    ) -> Self {
        Self {
            project_id,
            turso_hijack,
            otlp_hijack,
            queue_hijack,
            control_invoke_queue_hijack,
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
            return self_invoke_send(request);
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

        if let Some(hijack) = self.control_invoke_queue_hijack.clone()
            && hijack.matches(request.uri())
        {
            return control_invoke_queue_send(hijack, self.project_id.clone(), request, options);
        }

        if let Some(hijack) = self.vault_hijack.clone()
            && hijack.matches(request.uri())
        {
            return vault_send(hijack, self.project_id.clone(), request, options);
        }

        if let Some(hijack) = self.otlp_hijack.clone()
            && hijack.matches(request.uri())
        {
            return otlp_send(hijack, request, options);
        }

        default_send(request, options)
    }
}

fn self_invoke_send(
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let raw = WASM_RAW.try_with(|r| *r).map_err(|_| {
            TrappableError::from(ErrorCode::InternalError(Some(
                "self-invoke outside wasm scope".into(),
            )))
        })?;
        let accessor = unsafe { raw.accessor() };
        let service = unsafe { raw.service() };

        let (p3_req, req_io) = P3Request::from_http(request);
        let handle_result = service.handle(accessor, p3_req).await;

        match handle_result {
            Ok(Ok(resp)) => {
                let http_resp = accessor
                    .with(|mut access| resp.into_http(access.as_context_mut(), req_io))
                    .map_err(|e| {
                        TrappableError::from(ErrorCode::InternalError(Some(format!("{e:?}"))))
                    })?;
                let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
                    Box::new(async { Ok(()) });
                Ok((http_resp, io))
            }
            Ok(Err(ec)) => Err(ec.into()),
            Err(e) => Err(ErrorCode::InternalError(Some(format!("{e:?}"))).into()),
        }
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

        let (res, io) = default_send_request(request, options).await?;
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
                let (res, io) = default_send_request(signed, options).await?;
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

fn control_invoke_queue_send(
    hijack: Arc<ControlInvokeQueueHijack>,
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

        let action = match hijack.handle_invoke(&project_id, &body_bytes) {
            Ok(a) => a,
            Err(ec) => return Err(ec.into()),
        };

        match action {
            crate::control_invoke_queue_hijack::HijackAction::Forward(signed) => {
                let (res, io) = default_send_request(signed, options).await?;
                let res = res.map(BodyExt::boxed_unsync);
                let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
                    Box::new(io);
                Ok((res, io))
            }
            crate::control_invoke_queue_hijack::HijackAction::Synthesized(resp) => {
                let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> =
                    Box::new(async { Ok(()) });
                Ok((resp, io))
            }
        }
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

        let (res, io) = default_send_request(signed, options).await?;
        let res = res.map(BodyExt::boxed_unsync);
        let io: Box<dyn Future<Output = std::result::Result<(), ErrorCode>> + Send> = Box::new(io);
        Ok((res, io))
    })
}

fn otlp_send(
    hijack: Arc<OtlpHijack>,
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
            match default_send_request(forward_request, options).await {
                Ok((_resp, io)) => {
                    let _ = io.await;
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
    request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let (res, io) = default_send_request(request, options).await?;
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
