mod http_body_resource;
mod runtime_options;
mod source_map;

use bytes::Bytes;
use deno_core::anyhow::{Result, anyhow};
use deno_core::*;
use deno_error::JsErrorBox;
use http::*;
use http_body_resource::*;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Empty};
use runtime_options::*;
use serde::Serialize;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

struct SourceMapLoader {
    source_maps: RefCell<HashMap<String, Vec<u8>>>,
}

impl SourceMapLoader {
    fn new() -> Self {
        Self {
            source_maps: RefCell::new(HashMap::new()),
        }
    }

    fn register(&self, name: &str, source_map: Vec<u8>) {
        self.source_maps
            .borrow_mut()
            .insert(name.to_string(), source_map);
    }
}

impl ModuleLoader for SourceMapLoader {
    fn resolve(
        &self,
        _specifier: &str,
        _referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, error::ModuleLoaderError> {
        Err(error::ModuleLoaderError::generic(
            "Module loading not supported",
        ))
    }

    fn load(
        &self,
        _module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        ModuleLoadResponse::Sync(Err(error::ModuleLoaderError::generic(
            "Module loading not supported",
        )))
    }

    fn get_source_map(&self, file_name: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .borrow()
            .get(file_name)
            .map(|v| Cow::Owned(v.clone()))
    }

    fn source_map_source_exists(&self, _source_url: &str) -> Option<bool> {
        Some(true)
    }
}

pub type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;

pub type FetchHandlerFuture = Pin<Box<dyn Future<Output = Option<Response>> + Send>>;

pub trait FetchHandler: Send + Sync + 'static {
    fn handle(&self, request: Request) -> FetchHandlerFuture;
}

pub struct FetchHandlerHolder(pub Option<Arc<dyn FetchHandler>>);

#[derive(Serialize)]
pub struct FetchInterceptResult {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[op2(async(lazy))]
#[serde]
async fn op_fetch_intercept(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[string] method: String,
    #[serde] headers: Vec<(String, String)>,
    #[serde] body: Option<Vec<u8>>,
) -> Result<Option<FetchInterceptResult>, JsErrorBox> {
    let handler = {
        let state = state.borrow();
        state
            .try_borrow::<FetchHandlerHolder>()
            .and_then(|h| h.0.clone())
    };

    let Some(handler) = handler else {
        return Ok(None);
    };

    let mut request_builder = http::Request::builder().method(method.as_str()).uri(&url);

    for (key, value) in &headers {
        request_builder = request_builder.header(key.as_str(), value.as_str());
    }

    let body: UnsyncBoxBody<Bytes, anyhow::Error> = match body {
        Some(bytes) => UnsyncBoxBody::new(
            http_body_util::Full::new(Bytes::from(bytes)).map_err(|never| match never {}),
        ),
        None => UnsyncBoxBody::new(http_body_util::Empty::new().map_err(|never| match never {})),
    };

    let request = request_builder
        .body(body)
        .map_err(|e| JsErrorBox::generic(e.to_string()))?;

    let response = handler.handle(request).await;

    let Some(response) = response else {
        return Ok(None);
    };

    let status = response.status().as_u16();
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))?
        .to_bytes()
        .to_vec();

    Ok(Some(FetchInterceptResult {
        status,
        headers,
        body: body_bytes,
    }))
}

deno_core::extension!(fetch_intercept_extension, ops = [op_fetch_intercept],);

static RUNTIME_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/RUNJS_SNAPSHOT.bin"));

pub const JS_CALL_TIMEOUT_MS: u64 = 10;

type DeadlineMap = std::sync::Mutex<std::collections::HashMap<u32, std::time::Instant>>;

pub struct SkiInstance {
    runtime: Rc<RefCell<JsRuntime>>,
    run_handler: v8::Global<v8::Function>,
    next_id: Cell<u32>,
    driver_waker: Rc<Cell<Option<Waker>>>,
    deadlines: std::sync::Arc<DeadlineMap>,
}

struct WatchdogEntry {
    isolate: v8::IsolateHandle,
    deadlines: std::sync::Arc<DeadlineMap>,
}

static WATCHDOG: std::sync::OnceLock<std::sync::Mutex<Vec<std::sync::Arc<WatchdogEntry>>>> =
    std::sync::OnceLock::new();

fn watchdog_registry() -> &'static std::sync::Mutex<Vec<std::sync::Arc<WatchdogEntry>>> {
    WATCHDOG.get_or_init(|| {
        let mutex = std::sync::Mutex::new(Vec::new());
        std::thread::Builder::new()
            .name("ski-watchdog".into())
            .spawn(watchdog_loop)
            .expect("failed to spawn ski watchdog thread");
        mutex
    })
}

fn watchdog_loop() {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1));
        let registry = WATCHDOG.get().expect("registry initialized");
        let entries: Vec<_> = registry.lock().unwrap().clone();
        let now = std::time::Instant::now();
        for entry in entries {
            let expired = {
                let deadlines = entry.deadlines.lock().unwrap();
                deadlines.values().any(|d| now >= *d)
            };
            if expired {
                entry.isolate.terminate_execution();
            }
        }
    }
}

impl SkiInstance {
    pub fn load(
        code: &str,
        script_path: &str,
        fetch_handler: Option<Arc<dyn FetchHandler>>,
    ) -> Result<Self> {
        let script_url = format!("file://{}", script_path);
        let loader = Rc::new(SourceMapLoader::new());
        if let Some(sm) = source_map::extract_source_map(code.as_bytes()) {
            loader.register(&script_url, sm);
        }

        let mut options = runtime_options::runtime_options();
        options.startup_snapshot = Some(RUNTIME_SNAPSHOT);
        options.extensions.push(fetch_intercept_extension::init());
        options.module_loader = Some(loader);

        let mut runtime = JsRuntime::new(options);
        runtime.execute_script(script_url, code.to_string())?;

        runtime
            .op_state()
            .borrow_mut()
            .put(FetchHandlerHolder(fetch_handler));

        let isolate_handle = runtime.v8_isolate().thread_safe_handle();

        let run_handler: v8::Global<v8::Function> = {
            deno_core::scope!(scope, &mut runtime);
            let ctx = scope.get_current_context();
            let global_this = ctx.global(scope);
            let key = v8::String::new(scope, "__ski_runHandler")
                .ok_or_else(|| anyhow!("failed to create key"))?;
            let value = global_this
                .get(scope, key.into())
                .ok_or_else(|| anyhow!("__ski_runHandler not found"))?;
            let func = v8::Local::<v8::Function>::try_from(value)
                .map_err(|_| anyhow!("__ski_runHandler is not a function"))?;
            v8::Global::new(scope, func)
        };

        let deadlines: std::sync::Arc<DeadlineMap> =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

        let registry = watchdog_registry();
        registry
            .lock()
            .unwrap()
            .push(std::sync::Arc::new(WatchdogEntry {
                isolate: isolate_handle.clone(),
                deadlines: deadlines.clone(),
            }));

        Ok(Self {
            runtime: Rc::new(RefCell::new(runtime)),
            run_handler,
            next_id: Cell::new(0),
            driver_waker: Rc::new(Cell::new(None)),
            deadlines,
        })
    }

    pub fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Response>>>> {
        let id = self.next_id.get();
        self.next_id.set(id.wrapping_add(1));

        let runtime = self.runtime.clone();
        let run_handler = self.run_handler.clone();
        let deadlines = self.deadlines.clone();

        let call_fut = {
            let mut rt = runtime.borrow_mut();
            register_request(&mut rt, id, request);
            let id_global: v8::Global<v8::Value> = {
                deno_core::scope!(scope, &mut *rt);
                let v = v8::Integer::new_from_unsigned(scope, id);
                let v: v8::Local<v8::Value> = v.into();
                v8::Global::new(scope, v)
            };
            rt.call_with_args(&run_handler, &[id_global])
        };

        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(JS_CALL_TIMEOUT_MS);
        deadlines.lock().unwrap().insert(id, deadline);

        if let Some(waker) = self.driver_waker.replace(None) {
            waker.wake();
        }

        Box::pin(async move {
            let result = call_fut.await;
            deadlines.lock().unwrap().remove(&id);
            result.map_err(|e| anyhow!("js handler failed: {e:?}"))?;
            take_response(&runtime, id)
        })
    }

    pub fn drive(&self, cx: &mut Context<'_>) -> Poll<Result<(), deno_core::error::CoreError>> {
        let mut rt = self.runtime.borrow_mut();
        rt.poll_event_loop(cx, Default::default())
    }

    pub fn drive_forever(self: Rc<Self>) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(std::future::poll_fn(move |cx| {
            self.driver_waker.set(Some(cx.waker().clone()));
            match self.drive(cx) {
                Poll::Ready(Ok(())) => Poll::Pending,
                Poll::Ready(Err(e)) => {
                    eprintln!("[ski] event loop error: {e:?}");
                    Poll::Ready(())
                }
                Poll::Pending => Poll::Pending,
            }
        }))
    }
}

fn register_request(runtime: &mut JsRuntime, id: u32, req: Request) {
    let op_state = runtime.op_state();
    let mut state = op_state.borrow_mut();

    let (parts, body) = req.into_parts();
    let url = if parts.uri.scheme().is_some() {
        parts.uri.to_string()
    } else {
        format!("http://localhost{}", parts.uri)
    };
    let method = parts.method.to_string();
    let headers: Vec<(String, String)> = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let rid = if method == "GET" || method == "HEAD" {
        None
    } else {
        let resource = HttpBodyResource::new(body);
        Some(state.resource_table.add(resource))
    };

    let slot_map = state
        .try_borrow_mut::<SlotMap>()
        .expect("SlotMap not installed");
    slot_map.requests.insert(
        id,
        RequestParts {
            url,
            method,
            headers,
            rid,
        },
    );
}

fn take_response(runtime: &Rc<RefCell<JsRuntime>>, id: u32) -> Result<Response> {
    let rt = runtime.borrow();
    let op_state = rt.op_state();
    let mut state = op_state.borrow_mut();

    let response_parts = {
        let slot_map = state
            .try_borrow_mut::<SlotMap>()
            .ok_or_else(|| anyhow!("SlotMap missing"))?;
        slot_map
            .responses
            .remove(&id)
            .ok_or_else(|| anyhow!("No response produced for request {id}"))?
    };

    let mut builder =
        hyper::Response::builder().status(StatusCode::from_u16(response_parts.status)?);
    for (key, value) in response_parts.headers {
        if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
            builder = builder.header(name, value);
        }
    }

    let Some(rid) = response_parts.rid else {
        let body = BodyExt::boxed_unsync(Empty::<Bytes>::new().map_err(|never| match never {}));
        return Ok(builder.body(body)?);
    };

    let resource = state
        .resource_table
        .get_any(rid)
        .map_err(|_| anyhow!("Response body resource not found"))?;
    let body_adapter = deno_fetch::ResourceToBodyAdapter::new(resource);
    let body = BodyExt::boxed_unsync(body_adapter.map_err(|e| anyhow::anyhow!(e)));
    Ok(builder.body(body)?)
}

pub async fn run(
    code: &str,
    script_path: &str,
    request: Request,
    fetch_handler: Option<Arc<dyn FetchHandler>>,
) -> Result<Response> {
    let code = code.to_string();
    let script_path = script_path.to_string();

    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let instance = SkiInstance::load(&code, &script_path, fetch_handler)?;
            let call_fut = instance.call(request);
            drive_until(&instance, call_fut).await
        })
    })
    .await?
}

async fn drive_until<T>(
    instance: &SkiInstance,
    fut: Pin<Box<dyn Future<Output = Result<T>>>>,
) -> Result<T> {
    let mut fut = fut;
    std::future::poll_fn(move |cx| {
        if let Poll::Ready(result) = fut.as_mut().poll(cx) {
            return Poll::Ready(result);
        }
        if let Poll::Ready(Err(e)) = instance.drive(cx) {
            return Poll::Ready(Err(anyhow!("event loop error: {e:?}")));
        }
        Poll::Pending
    })
    .await
}

#[tokio::test]
async fn test() {
    run(
        "new MessageChannel();",
        "/test.js",
        Request::new(UnsyncBoxBody::new(
            http_body_util::Empty::new().map_err(|never| match never {}),
        )),
        None,
    )
    .await
    .unwrap();
}
