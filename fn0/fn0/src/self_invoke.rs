use crate::execute::ClientState;
use crate::measure_cpu_time::{Clock, SystemClock, TimeTracker, measure_cpu_time};
use crate::otlp_hijack::OtlpHijack;
use crate::turso_hijack::TursoHijack;
use crate::{Request, Response, telemetry};
use anyhow::{Result, anyhow};
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::http;
use std::cell::Cell;
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

pub async fn call_wasm_direct(req: Request) -> Result<Response> {
    let acc_ptr = ACCESSOR_PTR.with(|c| c.get());
    let svc_ptr = SERVICE_PTR.with(|c| c.get());
    let (Some(acc_ptr), Some(svc_ptr)) = (acc_ptr, svc_ptr) else {
        return Err(anyhow!("no accessor installed in current thread"));
    };
    let accessor: &Accessor<ClientState<SystemClock>> = unsafe { &*acc_ptr };
    let service: &Service = unsafe { &*svc_ptr };

    let (time_tracker, is_timeout, code_id) = accessor.with(|mut access| {
        let state = access.data_mut();
        (
            state.time_tracker.clone(),
            state.is_timeout.clone(),
            state.code_id.clone(),
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
        &code_id,
        time_tracker,
        &is_timeout,
    )
    .await
}

thread_local! {
    static ACCESSOR_PTR: Cell<Option<*const Accessor<ClientState<SystemClock>>>> = const { Cell::new(None) };
    static SERVICE_PTR: Cell<Option<*const Service>> = const { Cell::new(None) };
}

tokio::task_local! {
    pub(crate) static SELF_HOST: String;
}

pub(crate) struct AccessorGuard;

impl AccessorGuard {
    pub(crate) fn install(
        accessor: &Accessor<ClientState<SystemClock>>,
        service: &Service,
    ) -> Self {
        ACCESSOR_PTR.with(|c| c.set(Some(accessor as *const _)));
        SERVICE_PTR.with(|c| c.set(Some(service as *const _)));
        Self
    }
}

impl Drop for AccessorGuard {
    fn drop(&mut self) {
        ACCESSOR_PTR.with(|c| c.set(None));
        SERVICE_PTR.with(|c| c.set(None));
    }
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
    turso_hijack: Option<Arc<TursoHijack>>,
    otlp_hijack: Option<Arc<OtlpHijack>>,
}

impl SelfInvokeHooks {
    pub(crate) fn new(
        turso_hijack: Option<Arc<TursoHijack>>,
        otlp_hijack: Option<Arc<OtlpHijack>>,
    ) -> Self {
        Self {
            turso_hijack,
            otlp_hijack,
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
            return turso_send(hijack, request, options);
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
        let acc_ptr = ACCESSOR_PTR.with(|c| c.get());
        let svc_ptr = SERVICE_PTR.with(|c| c.get());
        let (Some(acc_ptr), Some(svc_ptr)) = (acc_ptr, svc_ptr) else {
            return Err(
                ErrorCode::InternalError(Some("self-invoke accessor slot empty".into())).into(),
            );
        };
        let accessor: &Accessor<ClientState<SystemClock>> = unsafe { &*acc_ptr };
        let service: &Service = unsafe { &*svc_ptr };

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
    mut request: http::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    options: Option<RequestOptions>,
) -> Box<dyn Future<Output = HookResult> + Send> {
    Box::new(async move {
        let acc_ptr = ACCESSOR_PTR.with(|c| c.get());
        let Some(acc_ptr) = acc_ptr else {
            return Err(
                ErrorCode::InternalError(Some("turso hijack accessor slot empty".into())).into(),
            );
        };
        let accessor: &Accessor<ClientState<SystemClock>> = unsafe { &*acc_ptr };
        let subdomain = accessor.with(|mut access| access.data_mut().code_id.clone());

        if let Err(e) = hijack.rewrite(&mut request, &subdomain) {
            return Err(e.into());
        }

        let (res, io) = default_send_request(request, options).await?;
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
    code_id: &str,
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
                    telemetry::wasmtime_error("response_into_http", code_id, &format!("{error:?}"));
                    anyhow!("response into_http failed: {error:?}")
                })?;
            Ok(http_resp.map(|body| {
                body.map_err(|ec| anyhow!("error_code: {ec:?}"))
                    .boxed_unsync()
            }))
        }
        Ok(Err(ec)) => {
            telemetry::proxy_returns_error_code(code_id, &format!("{ec:?}"));
            Err(anyhow!("proxy returned error code: {ec:?}"))
        }
        Err(error) => Err(classify_wasm_error(error, code_id, is_timeout)),
    }
}

pub(crate) fn classify_wasm_error(
    error: wasmtime::Error,
    code_id: &str,
    is_timeout: &Arc<AtomicBool>,
) -> anyhow::Error {
    match error.downcast::<wasmtime::Trap>() {
        Ok(trap) => {
            telemetry::trapped(code_id, &format!("{trap:?}"));
            if is_timeout.load(Ordering::Relaxed) {
                anyhow!("CPU time limit exceeded (trapped: {trap:?})")
            } else {
                anyhow!("trapped: {trap:?}")
            }
        }
        Err(error) => {
            telemetry::canceled_unexpectedly(code_id, &format!("{error:?}"));
            if is_timeout.load(Ordering::Relaxed) {
                anyhow!("CPU time limit exceeded: {error:?}")
            } else {
                anyhow!("canceled unexpectedly: {error:?}")
            }
        }
    }
}
