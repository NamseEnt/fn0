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
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;


use std::borrow::Cow;

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

    let mut request_builder = http::Request::builder()
        .method(method.as_str())
        .uri(&url);

    for (key, value) in &headers {
        request_builder = request_builder.header(key.as_str(), value.as_str());
    }

    let body: UnsyncBoxBody<Bytes, anyhow::Error> = match body {
        Some(bytes) => UnsyncBoxBody::new(
            http_body_util::Full::new(Bytes::from(bytes))
                .map_err(|never| match never {}),
        ),
        None => UnsyncBoxBody::new(
            http_body_util::Empty::new().map_err(|never| match never {}),
        ),
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

deno_core::extension!(
    fetch_intercept_extension,
    ops = [op_fetch_intercept],
);

static RUNTIME_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/RUNJS_SNAPSHOT.bin"));

pub async fn run(
    code: &str,
    script_path: &str,
    request: Request,
    fetch_handler: Option<Arc<dyn FetchHandler>>,
) -> Result<Response> {
    let code = code.to_string();
    let script_url = format!("file://{}", script_path);

    static V8_INIT: std::sync::Once = std::sync::Once::new();
    V8_INIT.call_once(|| {
        let platform = v8::new_unprotected_default_platform(0, false).make_shared();
        JsRuntime::init_platform(Some(platform));
    });

    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let loader = Rc::new(SourceMapLoader::new());
            if let Some(sm) = source_map::extract_source_map(code.as_bytes()) {
                loader.register(&script_url, sm);
            }

            let mut runtime_options = runtime_options();
            runtime_options.startup_snapshot = Some(RUNTIME_SNAPSHOT);
            runtime_options.extensions.push(fetch_intercept_extension::init());
            runtime_options.module_loader = Some(loader);

            let mut runtime = JsRuntime::new(runtime_options);
            runtime.execute_script(script_url.clone(), code.to_string())?;

            register_hyper_request(&mut runtime, request, fetch_handler);

            let script_result =
                runtime.execute_script("[run]", ascii_str!("globalThis.__ski_runHandler();"))?;
            let run_future = runtime.resolve(script_result);
            runtime
                .with_event_loop_future(run_future, Default::default())
                .await?;

            let op_state = runtime.op_state();

            let response_parts = op_state
                .borrow_mut()
                .try_take::<ResponseParts>()
                .ok_or_else(|| anyhow!("Did not get a response from JavaScript"))?;

            let mut builder =
                hyper::Response::builder().status(StatusCode::from_u16(response_parts.status)?);

            for (key, value) in response_parts.headers {
                if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
                    builder = builder.header(name, value);
                }
            }

            let Some(rid) = response_parts.rid else {
                let body =
                    BodyExt::boxed_unsync(Empty::<Bytes>::new().map_err(|never| match never {}));
                return Ok(builder.body(body)?);
            };

            let resource = op_state
                .borrow_mut()
                .resource_table
                .get_any(rid)
                .map_err(|_| anyhow!("Resource not found"))?;

            let body_adapter = deno_fetch::ResourceToBodyAdapter::new(resource);
            let body = BodyExt::boxed_unsync(body_adapter.map_err(|e| anyhow::anyhow!(e)));

            Ok(builder.body(body)?)
        })
    })
    .await?
}

fn register_hyper_request(
    runtime: &mut JsRuntime,
    req: Request,
    fetch_handler: Option<Arc<dyn FetchHandler>>,
) {
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
    state.put(RequestParts {
        url,
        method,
        headers,
        rid,
    });
    state.put(FetchHandlerHolder(fetch_handler));
}

#[tokio::test]
async fn test() {
    run(
        "new MessageChannel();",
        Request::new(UnsyncBoxBody::new(
            http_body_util::Empty::new().map_err(|never| match never {}),
        )),
        None,
    )
    .await
    .unwrap();
}
