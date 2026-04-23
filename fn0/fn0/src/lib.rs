pub mod cache;
pub mod execute;
mod js;
pub mod measure_cpu_time;
pub mod queue_poller;
mod self_invoke;
pub mod telemetry;
pub mod turso_queue;

use crate::measure_cpu_time::SystemClock;
use anyhow::{Result, anyhow};
use bytes::Bytes;
pub use cache::{Bundle, BundleCache, build_service_pre};
use execute::ClientState;
pub use execute::{build_linker, spawn_epoch_ticker};
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use wasmtime::Engine;
use wasmtime::component::Linker;
use wasmtime_wasi_http::p3::bindings::ServicePre;

pub use ski::{FetchHandler, FetchHandlerFuture};
pub use wasmtime;

pub type WasmProxyPre = ServicePre<ClientState<SystemClock>>;
pub type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;

const NEXT_HEADER: &str = "x-fn0-next";
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
}

impl<C: BundleCache> ExecutionContext<C> {
    pub fn new(engine: Engine, linker: Linker<ClientState<SystemClock>>, bundle_cache: C) -> Self {
        Self {
            engine,
            linker,
            bundle_cache,
        }
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
}

struct JsSlot {
    instance: std::rc::Rc<ski::SkiInstance>,
}

pub struct CodeExecutor<C: BundleCache> {
    ctx: Arc<ExecutionContext<C>>,
    instances: RefCell<HashMap<String, mpsc::UnboundedSender<execute::WasmInjectEnvelope>>>,
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

    pub async fn run(
        &self,
        subdomain: &str,
        _script_path: &str,
        request: Request,
        _fetch_handler: Option<Arc<dyn FetchHandler>>,
    ) -> Result<Response> {
        let bundle = self
            .ctx
            .bundle_cache
            .get(subdomain)
            .await
            .map_err(|e| anyhow!("bundle get failed for {subdomain}: {e}"))?;

        telemetry::function_invocation(subdomain);
        let start = std::time::Instant::now();

        let result = self.run_with_next(subdomain, bundle, request).await;

        telemetry::execution_time(subdomain, start.elapsed());
        result.map(strip_fn0_headers)
    }

    pub async fn run_backend_only(&self, subdomain: &str, request: Request) -> Result<Response> {
        let bundle = self
            .ctx
            .bundle_cache
            .get(subdomain)
            .await
            .map_err(|e| anyhow!("bundle get failed for {subdomain}: {e}"))?;
        self.run_wasm(subdomain, &bundle, request).await
    }

    async fn run_with_next(
        &self,
        subdomain: &str,
        bundle: Arc<Bundle>,
        request: Request,
    ) -> Result<Response> {
        let external_headers = request.headers().clone();
        let uri = request.uri().clone();

        let wasm_resp = self.run_wasm(subdomain, &bundle, request).await?;

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
                self.delegate_to_js(subdomain, &bundle, &external_headers, uri, wasm_resp)
                    .await
            }
            Some(other) => {
                tracing::error!(runtime = other, subdomain, "unknown x-fn0-next runtime");
                Ok(internal_error())
            }
        }
    }

    async fn run_wasm(
        &self,
        subdomain: &str,
        bundle: &Arc<Bundle>,
        request: Request,
    ) -> Result<Response> {
        let tx = self.wasm_instance_sender(subdomain, bundle);
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
        subdomain: &str,
        bundle: &Arc<Bundle>,
        external_headers: &hyper::HeaderMap,
        uri: hyper::Uri,
        wasm_resp: Response,
    ) -> Result<Response> {
        if bundle.js.is_none() {
            tracing::error!(subdomain, "x-fn0-next=js requested but bundle has no js");
            return Ok(internal_error());
        }
        let ski = self.get_or_spawn_js_instance(subdomain, bundle).await?;

        let js_entry_req = hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .header(hyper::header::CONTENT_TYPE, "application/json");
        let js_entry_req = external_headers.iter().fold(js_entry_req, |b, (k, v)| {
            if k == hyper::header::CONTENT_TYPE {
                b
            } else {
                b.header(k, v)
            }
        });
        let js_entry_req = js_entry_req.body(wasm_resp.into_body())?;

        ski.call(js_entry_req).await
    }

    async fn get_or_spawn_js_instance(
        &self,
        subdomain: &str,
        bundle: &Arc<Bundle>,
    ) -> Result<std::rc::Rc<ski::SkiInstance>> {
        if let Some(slot) = self.js_instances.borrow().get(subdomain) {
            return Ok(slot.instance.clone());
        }

        let js_code = bundle
            .js
            .as_ref()
            .ok_or_else(|| anyhow!("bundle has no js code for {subdomain}"))?;

        let fetch_handler: std::sync::Arc<dyn ski::FetchHandler> =
            std::sync::Arc::new(js::WasmForwardingFetchHandler);

        let instance = std::rc::Rc::new(ski::SkiInstance::load(
            js_code,
            "/entry.js",
            Some(fetch_handler),
        )?);
        self.js_instances.borrow_mut().insert(
            subdomain.to_string(),
            JsSlot {
                instance: instance.clone(),
            },
        );

        let driver_instance = instance.clone();
        tokio::task::spawn_local(async move {
            driver_instance.drive_forever().await;
        });

        Ok(instance)
    }

    fn wasm_instance_sender(
        &self,
        subdomain: &str,
        bundle: &Arc<Bundle>,
    ) -> mpsc::UnboundedSender<execute::WasmInjectEnvelope> {
        if let Some(tx) = self.instances.borrow().get(subdomain) {
            return tx.clone();
        }
        let (tx, rx) = mpsc::unbounded_channel();
        self.instances
            .borrow_mut()
            .insert(subdomain.to_string(), tx.clone());

        let ctx = self.ctx.clone();
        let bundle = bundle.clone();
        let subdomain_owned = subdomain.to_string();
        tokio::task::spawn_local(async move {
            let result =
                execute::run_wasm_instance_loop(&ctx.engine, bundle, subdomain_owned, rx).await;
            if let Err(e) = result {
                tracing::error!(?e, "wasm instance loop failed");
            }
        });

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
