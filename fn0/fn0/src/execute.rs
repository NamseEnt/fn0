use crate::cache::Bundle;
use crate::doc_db_hijack::DocDbHijack;
use crate::measure_cpu_time::{Clock, SystemClock, TimeTracker};
use crate::object_storage_hijack::ObjectStorageHijack;
use crate::public_storage_hijack::PublicStorageHijack;
use crate::self_invoke::{
    self, INVOCATION_CANCELLATION, INVOCATION_DEADLINE, SELF_HOST, SelfInvokeHooks,
    SelfInvokeHooksOptions, call_service,
};
use crate::static_page_cache_hijack::StaticPageCacheHijack;
use crate::turso_hijack::TursoHijack;
use crate::websocket_hijack::WebSocketHijack;
use crate::{Request, Response, telemetry};
use anyhow::{Result, anyhow};
use futures::stream::{FuturesUnordered, StreamExt};
use http_body_util::BodyExt;
use opentelemetry::propagation::{Injector, TextMapPropagator};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::AsyncWrite;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use wasmtime::{Engine, Store, component::Linker};
use wasmtime_wasi::cli::AsyncStdoutStream;
use wasmtime_wasi::*;
use wasmtime_wasi_http::{
    WasiHttpCtx,
    p3::{Request as P3Request, WasiHttpCtxView, WasiHttpView, bindings::http::types::ErrorCode},
};

const TRACEPARENT_HEADER: &str = "traceparent";
const TRACESTATE_HEADER: &str = "tracestate";

struct TracingWriter {
    project_id: String,
    is_stderr: bool,
    buf: Vec<u8>,
}

impl TracingWriter {
    fn new(project_id: String, is_stderr: bool) -> Self {
        Self {
            project_id,
            is_stderr,
            buf: Vec::with_capacity(1024),
        }
    }

    /// Both streams are recorded at the same level, under the project's own
    /// tenant. A guest's logging library writes its whole output to stderr --
    /// the Forte SDK's does -- so raising stderr to `ERROR` marked every line
    /// a project ever printed as an error, and filed it under the platform's
    /// tenant besides. Which stream a line came from is an attribute; it is
    /// not a severity.
    fn emit_line(&self, line: &str) {
        let stream = if self.is_stderr { "stderr" } else { "stdout" };
        tracing::info!(
            fn0.project_tenant = %self.project_id,
            stream,
            "{}",
            line
        );
    }
}

impl AsyncWrite for TracingWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = Pin::get_mut(self);
        this.buf.extend_from_slice(buf);
        while let Some(pos) = this.buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = this.buf.drain(..=pos).collect();
            let line_str = String::from_utf8_lossy(&line[..line.len() - 1]);
            let trimmed = line_str.trim_end_matches('\r');
            if !trimmed.is_empty() {
                this.emit_line(trimmed);
            }
        }
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = Pin::get_mut(self);
        if !this.buf.is_empty() {
            let line_str = String::from_utf8_lossy(&this.buf);
            let trimmed = line_str.trim_end_matches(['\r', '\n']);
            if !trimmed.is_empty() {
                this.emit_line(trimmed);
            }
            this.buf.clear();
        }
        Poll::Ready(Ok(()))
    }
}

fn make_tracing_stream(project_id: String, is_stderr: bool) -> AsyncStdoutStream {
    AsyncStdoutStream::new(4096, TracingWriter::new(project_id, is_stderr))
}

pub use fn0_wasmtime::engine_config;

pub fn build_linker(engine: &Engine) -> Linker<ClientState<SystemClock>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker).unwrap();
    wasmtime_wasi::p3::add_to_linker(&mut linker).unwrap();
    wasmtime_wasi_http::p3::add_to_linker(&mut linker).unwrap();
    linker
}

pub fn spawn_epoch_ticker(engine: Engine) {
    std::thread::Builder::new()
        .name("fn0-epoch-ticker".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(3));
                engine.increment_epoch();
            }
        })
        .expect("failed to spawn epoch ticker thread");
}

pub(crate) struct BuildStoreOptions<'a, C: Clock> {
    pub(crate) engine: &'a Engine,
    pub(crate) project_id: &'a str,
    pub(crate) env_vars: &'a [(String, String)],
    pub(crate) time_tracker: TimeTracker<C>,
    pub(crate) is_timeout: Arc<AtomicBool>,
    pub(crate) hooks: SelfInvokeHooks,
    pub(crate) doc_db_hijack: Option<&'a DocDbHijack>,
    pub(crate) turso_hijack: Option<&'a TursoHijack>,
    pub(crate) queue_hijack: Option<&'a crate::QueueHijack>,
    pub(crate) cross_project_enqueue_hijack: Option<&'a crate::CrossProjectEnqueueHijack>,
    pub(crate) cross_project_invoke_hijack: Option<&'a crate::CrossProjectInvokeHijack>,
    pub(crate) vault_hijack: Option<&'a crate::VaultHijack>,
    pub(crate) object_storage_hijack: Option<&'a ObjectStorageHijack>,
    pub(crate) public_storage_hijack: Option<&'a PublicStorageHijack>,
    pub(crate) static_page_cache_hijack: Option<&'a StaticPageCacheHijack>,
    pub(crate) websocket_hijack: Option<&'a WebSocketHijack>,
}

pub(crate) fn build_store<C>(options: BuildStoreOptions<'_, C>) -> Store<ClientState<C>>
where
    C: Clock,
{
    let BuildStoreOptions {
        engine,
        project_id,
        env_vars,
        time_tracker,
        is_timeout,
        hooks,
        doc_db_hijack,
        turso_hijack,
        queue_hijack,
        cross_project_enqueue_hijack,
        cross_project_invoke_hijack,
        vault_hijack,
        object_storage_hijack,
        public_storage_hijack,
        static_page_cache_hijack,
        websocket_hijack,
    } = options;
    let wasi = {
        let mut builder = WasiCtx::builder();
        builder.stdout(make_tracing_stream(project_id.to_string(), false));
        builder.stderr(make_tracing_stream(project_id.to_string(), true));
        for (key, value) in env_vars {
            if doc_db_hijack.is_some() && key == "FN0_DOC_DB_URL" {
                continue;
            }
            if turso_hijack.is_some() && (key == "TURSO_URL" || key == "TURSO_AUTH_TOKEN") {
                continue;
            }
            if queue_hijack.is_some() && key == "FN0_QUEUE_URL" {
                continue;
            }
            if cross_project_enqueue_hijack.is_some() && key == "FN0_CROSS_PROJECT_ENQUEUE_URL" {
                continue;
            }
            if cross_project_invoke_hijack.is_some() && key == "FN0_CROSS_PROJECT_INVOKE_URL" {
                continue;
            }
            if vault_hijack.is_some() && key == "FN0_VAULT_URL" {
                continue;
            }
            if object_storage_hijack.is_some() && key == "FN0_OBJECT_STORAGE_URL" {
                continue;
            }
            if public_storage_hijack.is_some()
                && (key == "FN0_PUBLIC_STORAGE_URL" || key == "FN0_PUBLIC_STORAGE_BASE_URL")
            {
                continue;
            }
            if static_page_cache_hijack.is_some() && key == "FN0_STATIC_PAGE_CACHE_URL" {
                continue;
            }
            if websocket_hijack.is_some() && key == "FN0_WEBSOCKET_URL" {
                continue;
            }
            builder.env(key, value);
        }
        if let Some(hijack) = doc_db_hijack {
            builder.env("FN0_DOC_DB_URL", hijack.placeholder_url());
        }
        if let Some(hijack) = turso_hijack {
            builder.env("TURSO_URL", format!("http://{}", hijack.placeholder_host));
            builder.env("TURSO_AUTH_TOKEN", "");
        }
        if let Some(hijack) = queue_hijack {
            builder.env("FN0_QUEUE_URL", hijack.placeholder_url());
        }
        if let Some(hijack) = cross_project_enqueue_hijack
            && project_id == hijack.allowed_caller_project_id()
        {
            builder.env("FN0_CROSS_PROJECT_ENQUEUE_URL", hijack.placeholder_url());
        }
        if let Some(hijack) = cross_project_invoke_hijack
            && project_id == hijack.allowed_caller_project_id()
        {
            builder.env("FN0_CROSS_PROJECT_INVOKE_URL", hijack.placeholder_url());
        }
        if let Some(hijack) = vault_hijack {
            builder.env("FN0_VAULT_URL", hijack.placeholder_url());
        }
        if let Some(hijack) = object_storage_hijack {
            builder.env("FN0_OBJECT_STORAGE_URL", hijack.placeholder_url());
        }
        // Both or neither: `object_storage::public::bucket()` needs the base URL
        // to build the URLs it hands back, so injecting only the endpoint would
        // hand the guest a bucket that panics on first use.
        if let Some(hijack) = public_storage_hijack
            && let Some(base_url) = hijack.public_base_url_for(project_id)
        {
            builder.env("FN0_PUBLIC_STORAGE_URL", hijack.placeholder_url());
            builder.env("FN0_PUBLIC_STORAGE_BASE_URL", base_url);
        }
        if let Some(hijack) = static_page_cache_hijack {
            builder.env("FN0_STATIC_PAGE_CACHE_URL", hijack.placeholder_url());
        }
        if let Some(hijack) = websocket_hijack {
            builder.env("FN0_WEBSOCKET_URL", hijack.placeholder_url());
        }
        builder.build()
    };

    let mut store = Store::new(
        engine,
        ClientState {
            table: ResourceTable::new(),
            wasi,
            http: WasiHttpCtx::new(),
            time_tracker,
            is_timeout,
            hooks,
        },
    );
    store.epoch_deadline_trap();
    store.set_epoch_deadline(1);
    store.epoch_deadline_async_yield_and_update(1);
    let project_id_for_timeout = project_id.to_string();
    store.epoch_deadline_callback(move |context| {
        let state = context.data();
        let cpu_time = state.time_tracker.duration();
        if cpu_time > Duration::from_millis(1000) {
            telemetry::cpu_timeout(&project_id_for_timeout);
            state.is_timeout.store(true, Ordering::Relaxed);
            return Ok(wasmtime::UpdateDeadline::Interrupt);
        }
        Ok(wasmtime::UpdateDeadline::Continue(1))
    });

    store
}

pub struct WasmInjectEnvelope {
    pub request: Request,
    pub response_sender: oneshot::Sender<Result<Response>>,
    pub cancellation: CancellationToken,
}

impl WasmInjectEnvelope {
    pub fn new(
        mut request: Request,
        response_sender: oneshot::Sender<Result<Response>>,
        cancellation: CancellationToken,
    ) -> Self {
        propagate_trace_context(request.headers_mut());
        Self {
            request,
            response_sender,
            cancellation,
        }
    }
}

/// The guest continues the worker's trace and inherits its sampling decision.
/// A `traceparent` that arrived from outside is removed first: honouring it
/// would let any visitor mark their own requests as sampled.
fn propagate_trace_context(headers: &mut hyper::HeaderMap) {
    headers.remove(TRACEPARENT_HEADER);
    headers.remove(TRACESTATE_HEADER);
    let context = tracing::Span::current().context();
    TraceContextPropagator::new().inject_context(&context, &mut HeaderInjector(headers));
}

struct HeaderInjector<'a>(&'a mut hyper::HeaderMap);

impl Injector for HeaderInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(name), Ok(value)) = (
            hyper::header::HeaderName::from_bytes(key.as_bytes()),
            hyper::header::HeaderValue::from_str(&value),
        ) {
            self.0.insert(name, value);
        }
    }
}

pub(crate) struct WasmInstanceLoopOptions<'a> {
    pub(crate) engine: &'a Engine,
    pub(crate) bundle: Arc<Bundle>,
    pub(crate) project_id: String,
    pub(crate) self_invoke_sender: mpsc::UnboundedSender<WasmInjectEnvelope>,
    pub(crate) rx: mpsc::UnboundedReceiver<WasmInjectEnvelope>,
    pub(crate) doc_db_hijack: Option<Arc<DocDbHijack>>,
    pub(crate) turso_hijack: Option<Arc<TursoHijack>>,
    pub(crate) otlp_hijack: Option<Arc<crate::OtlpHijack>>,
    pub(crate) queue_hijack: Option<Arc<crate::QueueHijack>>,
    pub(crate) cross_project_enqueue_hijack: Option<Arc<crate::CrossProjectEnqueueHijack>>,
    pub(crate) cross_project_invoke_hijack: Option<Arc<crate::CrossProjectInvokeHijack>>,
    pub(crate) vault_hijack: Option<Arc<crate::VaultHijack>>,
    pub(crate) object_storage_hijack: Option<Arc<ObjectStorageHijack>>,
    pub(crate) public_storage_hijack: Option<Arc<crate::PublicStorageHijack>>,
    pub(crate) static_page_cache_hijack: Option<Arc<crate::StaticPageCacheHijack>>,
    pub(crate) websocket_hijack: Option<Arc<WebSocketHijack>>,
    pub(crate) guest_outbound_http: Option<Arc<crate::GuestOutboundHttp>>,
}

pub(crate) async fn run_wasm_instance_loop(options: WasmInstanceLoopOptions<'_>) -> Result<()> {
    let WasmInstanceLoopOptions {
        engine,
        bundle,
        project_id,
        self_invoke_sender,
        mut rx,
        doc_db_hijack,
        turso_hijack,
        otlp_hijack,
        queue_hijack,
        cross_project_enqueue_hijack,
        cross_project_invoke_hijack,
        vault_hijack,
        object_storage_hijack,
        public_storage_hijack,
        static_page_cache_hijack,
        websocket_hijack,
        guest_outbound_http,
    } = options;
    let time_tracker = TimeTracker::new(SystemClock);
    let is_timeout = Arc::new(AtomicBool::new(false));

    let mut store = build_store(BuildStoreOptions {
        engine,
        project_id: &project_id,
        env_vars: &bundle.env_vars,
        time_tracker: time_tracker.clone(),
        is_timeout: is_timeout.clone(),
        hooks: SelfInvokeHooks::new(SelfInvokeHooksOptions {
            project_id: project_id.clone(),
            self_invoke_sender,
            doc_db_hijack: doc_db_hijack.clone(),
            turso_hijack: turso_hijack.clone(),
            otlp_hijack: otlp_hijack.clone(),
            queue_hijack: queue_hijack.clone(),
            cross_project_enqueue_hijack: cross_project_enqueue_hijack.clone(),
            cross_project_invoke_hijack: cross_project_invoke_hijack.clone(),
            vault_hijack: vault_hijack.clone(),
            object_storage_hijack: object_storage_hijack.clone(),
            public_storage_hijack: public_storage_hijack.clone(),
            static_page_cache_hijack: static_page_cache_hijack.clone(),
            websocket_hijack: websocket_hijack.clone(),
            guest_outbound_http,
        }),
        doc_db_hijack: doc_db_hijack.as_deref(),
        turso_hijack: turso_hijack.as_deref(),
        queue_hijack: queue_hijack.as_deref(),
        cross_project_enqueue_hijack: cross_project_enqueue_hijack.as_deref(),
        cross_project_invoke_hijack: cross_project_invoke_hijack.as_deref(),
        vault_hijack: vault_hijack.as_deref(),
        object_storage_hijack: object_storage_hijack.as_deref(),
        public_storage_hijack: public_storage_hijack.as_deref(),
        static_page_cache_hijack: static_page_cache_hijack.as_deref(),
        websocket_hijack: websocket_hijack.as_deref(),
    });

    let instantiate_start = std::time::Instant::now();
    let service = bundle
        .service_pre
        .instantiate_async(&mut store)
        .await
        .map_err(|error| {
            telemetry::failure(telemetry::FailureComponent::Instantiate, "wasmtime_error");
            anyhow!("instantiate_async failed: {error:?}")
        })?;
    telemetry::stage_duration("instantiate", instantiate_start.elapsed());

    let project_id_for_cpu = project_id.clone();
    let run_result = store
        .run_concurrent(async move |accessor| -> Result<()> {
            let mut pending: FuturesUnordered<Pin<Box<dyn Future<Output = ()> + Send>>> =
                FuturesUnordered::new();

            loop {
                tokio::select! {
                    biased;
                    maybe = rx.recv() => {
                        match maybe {
                            Some(envelope) => {
                                let WasmInjectEnvelope {
                                    request,
                                    response_sender,
                                    cancellation,
                                } = envelope;
                                let self_host = self_invoke::extract_host(request.headers())
                                    .unwrap_or_default();
                                let service_ref = &service;
                                let time_tracker = time_tracker.clone();
                                let is_timeout = is_timeout.clone();
                                pending.push(Box::pin(async move {
                                    let call_start = std::time::Instant::now();
                                    let invocation_deadline =
                                        std::time::Instant::now() + Duration::from_secs(15);
                                    let invocation = INVOCATION_DEADLINE.scope(
                                        invocation_deadline,
                                        INVOCATION_CANCELLATION.scope(
                                            cancellation.clone(),
                                            SELF_HOST.scope(self_host, async move {
                                                let req_http = request.map(|body| {
                                                    body.map_err(|error| {
                                                        if let Some(limit_error) = error
                                                            .downcast_ref::<crate::RequestBodyTooLarge>()
                                                        {
                                                            ErrorCode::HttpRequestBodySize(Some(
                                                                limit_error.limit,
                                                            ))
                                                        } else {
                                                            ErrorCode::InternalError(Some(
                                                                error.to_string(),
                                                            ))
                                                        }
                                                    })
                                                    .boxed_unsync()
                                                });
                                                let (p3_req, req_io) =
                                                    P3Request::from_http(req_http);
                                                call_service(
                                                    accessor,
                                                    service_ref,
                                                    p3_req,
                                                    req_io,
                                                    time_tracker,
                                                    &is_timeout,
                                                )
                                                .await
                                            }),
                                        ),
                                    );
                                    let result = tokio::select! {
                                        _ = cancellation.cancelled() => {
                                            Err(anyhow!("request cancelled"))
                                        }
                                        result = invocation => result,
                                    };
                                    telemetry::stage_duration("wasm_call", call_start.elapsed());
                                    if response_sender.send(result).is_err() {
                                        telemetry::failure(
                                            telemetry::FailureComponent::ResponseChannel,
                                            "receiver_dropped",
                                        );
                                    }
                                }));
                            }
                            None => {
                                while pending.next().await.is_some() {}
                                break;
                            }
                        }
                    }
                    Some(()) = pending.next() => {}
                }
            }

            telemetry::cpu_time(&project_id_for_cpu, time_tracker.duration());
            Ok(())
        })
        .await;

    match run_result {
        Ok(inner) => inner,
        Err(error) => {
            telemetry::failure(telemetry::FailureComponent::RunConcurrent, "wasmtime_error");
            Err(anyhow!("run_concurrent failed: {error:?}"))
        }
    }
}

pub struct ClientState<C: Clock> {
    wasi: WasiCtx,
    http: WasiHttpCtx,
    table: ResourceTable,
    pub(crate) time_tracker: TimeTracker<C>,
    pub(crate) is_timeout: Arc<AtomicBool>,
    hooks: SelfInvokeHooks,
}

impl<C: Clock> WasiView for ClientState<C> {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl<C: Clock> WasiHttpView for ClientState<C> {
    fn http(&mut self) -> WasiHttpCtxView<'_> {
        WasiHttpCtxView {
            ctx: &mut self.http,
            table: &mut self.table,
            hooks: &mut self.hooks,
        }
    }
}
