pub mod cache;
pub mod cross_project_enqueue_hijack;
pub mod cross_project_invoke_hijack;
pub mod execute;
mod js;
pub mod measure_cpu_time;
pub mod metric_gate;
pub mod object_storage_hijack;
pub mod otlp_hijack;
mod panic_util;
pub mod presign_gate;
pub mod queue_hijack;
mod self_invoke;
pub mod telemetry;
pub mod turso_hijack;
pub mod vault_hijack;

pub use panic_util::panic_payload_string;

use crate::measure_cpu_time::SystemClock;
use anyhow::{Result, anyhow};
use bytes::Bytes;
pub use cache::{Bundle, BundleCache, build_service_pre};
use execute::ClientState;
pub use execute::{build_linker, spawn_epoch_ticker};
use futures::FutureExt;
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use wasmtime::Engine;
use wasmtime::component::Linker;
use wasmtime_wasi_http::p3::bindings::ServicePre;

pub use cross_project_enqueue_hijack::CrossProjectEnqueueHijack;
pub use cross_project_invoke_hijack::{CrossProjectInvokeDispatcher, CrossProjectInvokeHijack};
pub use metric_gate::MetricCardinalityGate;
pub use object_storage_hijack::{DevReadResult, ObjectStorageHijack};
pub use otlp_hijack::OtlpHijack;
pub use presign_gate::PresignGate;
pub use queue_hijack::QueueHijack;
pub use ski::{FetchHandler, FetchHandlerFuture};
pub use turso_hijack::TursoHijack;
pub use vault_hijack::VaultHijack;
pub use wasmtime;

pub type WasmProxyPre = ServicePre<ClientState<SystemClock>>;
pub type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;

const NEXT_HEADER: &str = "x-fn0-next";
const EXECUTION_TIME_METRIC_KEY_HEADER: &str = "x-fn0-execution-time-metric-key";
const FN0_HEADER_PREFIX: &str = "x-fn0-";

#[derive(Debug)]
pub struct BuildEngineError(wasmtime::Error);

impl std::fmt::Display for BuildEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "engine build failed: {:#}", self.0)
    }
}

impl std::error::Error for BuildEngineError {}

impl BuildEngineError {
    pub fn into_inner(self) -> wasmtime::Error {
        self.0
    }
}

pub fn build_engine() -> Result<Engine, BuildEngineError> {
    Engine::new(&execute::engine_config()).map_err(BuildEngineError)
}

pub struct ExecutionContext<C: BundleCache> {
    pub(crate) engine: Engine,
    pub(crate) linker: Linker<ClientState<SystemClock>>,
    pub(crate) bundle_cache: C,
    pub(crate) turso_hijack: Option<Arc<TursoHijack>>,
    pub(crate) otlp_hijack: Option<Arc<OtlpHijack>>,
    pub(crate) queue_hijack: Option<Arc<QueueHijack>>,
    pub(crate) cross_project_enqueue_hijack: Option<Arc<CrossProjectEnqueueHijack>>,
    pub(crate) cross_project_invoke_hijack: Option<Arc<CrossProjectInvokeHijack>>,
    pub(crate) vault_hijack: Option<Arc<VaultHijack>>,
    pub(crate) object_storage_hijack: Option<Arc<ObjectStorageHijack>>,
}

impl<C: BundleCache> ExecutionContext<C> {
    pub fn new(engine: Engine, linker: Linker<ClientState<SystemClock>>, bundle_cache: C) -> Self {
        Self {
            engine,
            linker,
            bundle_cache,
            turso_hijack: None,
            otlp_hijack: None,
            queue_hijack: None,
            cross_project_enqueue_hijack: None,
            cross_project_invoke_hijack: None,
            vault_hijack: None,
            object_storage_hijack: None,
        }
    }

    pub fn with_turso_hijack(mut self, turso_hijack: Arc<TursoHijack>) -> Self {
        self.turso_hijack = Some(turso_hijack);
        self
    }

    pub fn with_otlp_hijack(mut self, otlp_hijack: Arc<OtlpHijack>) -> Self {
        self.otlp_hijack = Some(otlp_hijack);
        self
    }

    pub fn with_queue_hijack(mut self, queue_hijack: Arc<QueueHijack>) -> Self {
        self.queue_hijack = Some(queue_hijack);
        self
    }

    pub fn with_cross_project_enqueue_hijack(
        mut self,
        hijack: Arc<CrossProjectEnqueueHijack>,
    ) -> Self {
        self.cross_project_enqueue_hijack = Some(hijack);
        self
    }

    pub fn with_cross_project_invoke_hijack(
        mut self,
        hijack: Arc<CrossProjectInvokeHijack>,
    ) -> Self {
        self.cross_project_invoke_hijack = Some(hijack);
        self
    }

    pub fn with_vault_hijack(mut self, vault_hijack: Arc<VaultHijack>) -> Self {
        self.vault_hijack = Some(vault_hijack);
        self
    }

    pub fn with_object_storage_hijack(
        mut self,
        object_storage_hijack: Arc<ObjectStorageHijack>,
    ) -> Self {
        self.object_storage_hijack = Some(object_storage_hijack);
        self
    }

    pub fn bundle_cache(&self) -> &C {
        &self.bundle_cache
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn linker(&self) -> &Linker<ClientState<SystemClock>> {
        &self.linker
    }

    pub fn turso_hijack(&self) -> Option<&Arc<TursoHijack>> {
        self.turso_hijack.as_ref()
    }

    pub fn otlp_hijack(&self) -> Option<&Arc<OtlpHijack>> {
        self.otlp_hijack.as_ref()
    }

    pub fn queue_hijack(&self) -> Option<&Arc<QueueHijack>> {
        self.queue_hijack.as_ref()
    }

    pub fn cross_project_enqueue_hijack(&self) -> Option<&Arc<CrossProjectEnqueueHijack>> {
        self.cross_project_enqueue_hijack.as_ref()
    }

    pub fn cross_project_invoke_hijack(&self) -> Option<&Arc<CrossProjectInvokeHijack>> {
        self.cross_project_invoke_hijack.as_ref()
    }

    pub fn vault_hijack(&self) -> Option<&Arc<VaultHijack>> {
        self.vault_hijack.as_ref()
    }

    pub fn object_storage_hijack(&self) -> Option<&Arc<ObjectStorageHijack>> {
        self.object_storage_hijack.as_ref()
    }
}

struct JsSlot {
    instance: std::rc::Rc<ski::SkiInstance>,
    bundle: Arc<Bundle>,
    poisoned: std::rc::Rc<Cell<bool>>,
    driver: tokio::task::JoinHandle<()>,
}

struct WasmSlot {
    sender: mpsc::UnboundedSender<execute::WasmInjectEnvelope>,
    bundle: Arc<Bundle>,
    driver: tokio::task::JoinHandle<()>,
}

pub struct CodeExecutor<C: BundleCache> {
    ctx: Arc<ExecutionContext<C>>,
    instances: RefCell<HashMap<String, WasmSlot>>,
    js_instances: RefCell<HashMap<String, JsSlot>>,
}

impl<C: BundleCache> CodeExecutor<C> {
    pub fn new(ctx: Arc<ExecutionContext<C>>) -> Self {
        Self {
            ctx,
            instances: RefCell::new(HashMap::new()),
            js_instances: RefCell::new(HashMap::new()),
        }
    }

    pub fn context(&self) -> &Arc<ExecutionContext<C>> {
        &self.ctx
    }

    #[tracing::instrument(skip_all, fields(project_id = %project_id))]
    pub async fn run(
        &self,
        project_id: &str,
        _script_path: &str,
        request: Request,
        _fetch_handler: Option<Arc<dyn FetchHandler>>,
    ) -> Result<Response> {
        let bundle_start = std::time::Instant::now();
        let bundle = self.ctx.bundle_cache.get(project_id).await?;
        telemetry::stage_duration("bundle_get", bundle_start.elapsed());

        telemetry::function_invocation();
        let start = std::time::Instant::now();

        let result = self.run_with_next(project_id, bundle, request).await;

        let key = result
            .as_ref()
            .ok()
            .and_then(|r| r.headers().get(EXECUTION_TIME_METRIC_KEY_HEADER))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        telemetry::execution_time(&key, start.elapsed());
        result.map(strip_fn0_headers)
    }

    pub async fn sweep_unregistered(&self) {
        let registered = self.ctx.bundle_cache.registered_project_ids().await;

        let dead: Vec<String> = self
            .instances
            .borrow()
            .keys()
            .filter(|project_id| !registered.contains(*project_id))
            .cloned()
            .collect();
        for project_id in &dead {
            if let Some(slot) = self.instances.borrow_mut().remove(project_id) {
                slot.driver.abort();
            }
        }

        let dead_js: Vec<String> = self
            .js_instances
            .borrow()
            .keys()
            .filter(|project_id| !registered.contains(*project_id))
            .cloned()
            .collect();
        for project_id in &dead_js {
            if let Some(slot) = self.js_instances.borrow_mut().remove(project_id) {
                slot.driver.abort();
            }
        }
    }

    #[tracing::instrument(skip_all, fields(project_id = %project_id))]
    pub async fn run_backend_only(&self, project_id: &str, request: Request) -> Result<Response> {
        let bundle = self.ctx.bundle_cache.get(project_id).await?;
        self.run_wasm(project_id, &bundle, request).await
    }

    #[tracing::instrument(skip_all, fields(project_id = %project_id))]
    async fn run_with_next(
        &self,
        project_id: &str,
        bundle: Arc<Bundle>,
        request: Request,
    ) -> Result<Response> {
        let external_headers = request.headers().clone();
        let uri = request.uri().clone();

        let wasm_resp = self.run_wasm(project_id, &bundle, request).await?;

        if wasm_resp.status() != hyper::StatusCode::OK {
            return Ok(wasm_resp);
        }

        let next = wasm_resp
            .headers()
            .get(NEXT_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        match next.as_deref() {
            None | Some("") => Ok(wasm_resp),
            Some("js") => {
                self.delegate_to_js(project_id, &bundle, &external_headers, uri, wasm_resp)
                    .await
            }
            Some(other) => {
                tracing::error!(runtime = other, project_id, "unknown x-fn0-next runtime");
                Ok(internal_error())
            }
        }
    }

    #[tracing::instrument(skip_all, fields(project_id = %project_id))]
    async fn run_wasm(
        &self,
        project_id: &str,
        bundle: &Arc<Bundle>,
        request: Request,
    ) -> Result<Response> {
        let tx = self.wasm_instance_sender(project_id, bundle);
        let (resp_tx, resp_rx) = oneshot::channel();
        if tx.send((request, resp_tx)).is_err() {
            return Err(anyhow!("wasm instance channel closed"));
        }
        resp_rx
            .await
            .unwrap_or_else(|_| Err(anyhow!("wasm instance dropped response")))
    }

    async fn delegate_to_js(
        &self,
        project_id: &str,
        bundle: &Arc<Bundle>,
        external_headers: &hyper::HeaderMap,
        uri: hyper::Uri,
        wasm_resp: Response,
    ) -> Result<Response> {
        if bundle.js.is_none() {
            tracing::error!(project_id, "x-fn0-next=js requested but bundle has no js");
            return Ok(internal_error());
        }
        // body is buffered up-front: a retry rebuilds the js request from it.
        let wasm_body_bytes = wasm_resp.into_body().collect().await?.to_bytes();

        let mut attempt = 0;
        loop {
            attempt += 1;
            let (ski, poisoned) = self.get_or_spawn_js_instance(project_id, bundle).await?;

            let js_entry_req = hyper::Request::builder()
                .method("POST")
                .uri(uri.clone())
                .header(hyper::header::CONTENT_TYPE, "application/json");
            let js_entry_req = external_headers.iter().fold(js_entry_req, |b, (k, v)| {
                if k == hyper::header::CONTENT_TYPE {
                    b
                } else {
                    b.header(k, v)
                }
            });
            let body: Body = UnsyncBoxBody::new(
                http_body_util::Full::new(wasm_body_bytes.clone())
                    .map_err(|e: std::convert::Infallible| anyhow!(e)),
            );
            let js_entry_req = js_entry_req.body(body)?;

            // ski.call() enters v8 synchronously before returning its future, so
            // the call itself goes inside the catch_unwind block — not
            // AssertUnwindSafe(ski.call(..)).
            let outcome = AssertUnwindSafe(async { ski.call(js_entry_req).await })
                .catch_unwind()
                .await;
            match outcome {
                Ok(result) => return result,
                Err(panic) => {
                    poisoned.set(true);
                    telemetry::panicked();
                    let msg = panic_util::panic_payload_string(&panic);
                    tracing::error!(project_id, attempt, "js instance panicked: {msg}");
                    if attempt >= 2 {
                        return Ok(internal_error());
                    }
                }
            }
        }
    }

    // Returns the cached js instance and its poison flag, evicting+respawning a
    // poisoned one. This body has no `.await`, so on the single-threaded worker
    // runtime it runs atomically — concurrent requests after a panic respawn
    // exactly once. Do not add an `.await` here.
    async fn get_or_spawn_js_instance(
        &self,
        project_id: &str,
        bundle: &Arc<Bundle>,
    ) -> Result<(std::rc::Rc<ski::SkiInstance>, std::rc::Rc<Cell<bool>>)> {
        {
            let mut js_instances = self.js_instances.borrow_mut();
            if let Some(slot) = js_instances.get(project_id) {
                let same_bundle = Arc::ptr_eq(&slot.bundle, bundle);
                let poisoned = slot.poisoned.get();
                if same_bundle && !poisoned {
                    return Ok((slot.instance.clone(), slot.poisoned.clone()));
                }
                if same_bundle && poisoned {
                    tracing::warn!(project_id, "js instance poisoned; respawning");
                }
                if let Some(removed) = js_instances.remove(project_id) {
                    removed.driver.abort();
                }
            }
        }

        let js_code = bundle
            .js
            .as_ref()
            .ok_or_else(|| anyhow!("bundle has no js code for {project_id}"))?;

        let wasm_sender = self.wasm_instance_sender(project_id, bundle);
        let fetch_handler: std::sync::Arc<dyn ski::FetchHandler> =
            std::sync::Arc::new(js::WasmForwardingFetchHandler::new(wasm_sender));

        let instance = std::rc::Rc::new(ski::SkiInstance::load(
            js_code,
            "/entry.js",
            Some(fetch_handler),
        )?);
        let poisoned = std::rc::Rc::new(Cell::new(false));

        let driver_instance = instance.clone();
        let driver = tokio::task::spawn_local(async move {
            driver_instance.drive_forever().await;
        });

        self.js_instances.borrow_mut().insert(
            project_id.to_string(),
            JsSlot {
                instance: instance.clone(),
                bundle: bundle.clone(),
                poisoned: poisoned.clone(),
                driver,
            },
        );

        Ok((instance, poisoned))
    }

    fn wasm_instance_sender(
        &self,
        project_id: &str,
        bundle: &Arc<Bundle>,
    ) -> mpsc::UnboundedSender<execute::WasmInjectEnvelope> {
        {
            let mut instances = self.instances.borrow_mut();
            if let Some(slot) = instances.get(project_id) {
                let same_bundle = Arc::ptr_eq(&slot.bundle, bundle);
                let alive = !slot.sender.is_closed();
                if same_bundle && alive {
                    return slot.sender.clone();
                }
                if same_bundle && !alive {
                    tracing::warn!(project_id, "wasm instance dead; respawning");
                }
                if let Some(removed) = instances.remove(project_id) {
                    removed.driver.abort();
                }
            }
        }

        let (tx, rx) = mpsc::unbounded_channel();
        telemetry::create_instance();

        let ctx = self.ctx.clone();
        let slot_bundle = bundle.clone();
        let bundle = bundle.clone();
        let project_id_owned = project_id.to_string();
        let turso_hijack = ctx.turso_hijack.clone();
        let otlp_hijack = ctx.otlp_hijack.clone();
        let queue_hijack = ctx.queue_hijack.clone();
        let cross_project_enqueue_hijack = ctx.cross_project_enqueue_hijack.clone();
        let cross_project_invoke_hijack = ctx.cross_project_invoke_hijack.clone();
        let vault_hijack = ctx.vault_hijack.clone();
        let object_storage_hijack = ctx.object_storage_hijack.clone();
        let project_id_for_log = project_id_owned.clone();
        let self_invoke_sender = tx.clone();
        let driver = tokio::task::spawn_local(async move {
            let outcome = AssertUnwindSafe(execute::run_wasm_instance_loop(
                &ctx.engine,
                bundle,
                project_id_owned,
                self_invoke_sender,
                rx,
                turso_hijack,
                otlp_hijack,
                queue_hijack,
                cross_project_enqueue_hijack,
                cross_project_invoke_hijack,
                vault_hijack,
                object_storage_hijack,
            ))
            .catch_unwind()
            .await;
            match outcome {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!(project_id = %project_id_for_log, ?e, "wasm instance loop failed");
                }
                Err(panic) => {
                    let panic_msg = panic_util::panic_payload_string(&panic);
                    telemetry::panicked();
                    tracing::error!(
                        project_id = %project_id_for_log,
                        panic = %panic_msg,
                        "wasm instance loop panicked; will respawn on next request"
                    );
                }
            }
        });

        self.instances.borrow_mut().insert(
            project_id.to_string(),
            WasmSlot {
                sender: tx.clone(),
                bundle: slot_bundle,
                driver,
            },
        );

        tx
    }
}

fn strip_fn0_headers(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    let to_remove: Vec<_> = headers
        .keys()
        .filter(|k| k.as_str().starts_with(FN0_HEADER_PREFIX))
        .cloned()
        .collect();
    for name in to_remove {
        headers.remove(&name);
    }
    resp
}

fn internal_error() -> Response {
    let body: Body = UnsyncBoxBody::new(
        http_body_util::Empty::<Bytes>::new().map_err(|e: std::convert::Infallible| anyhow!(e)),
    );
    hyper::Response::builder().status(500).body(body).unwrap()
}

pub use fn0_wasmtime::{VERSION as FN0_WASMTIME_VERSION, compile};
