mod deployment;
pub mod execute;
mod forte_fetch;
pub mod measure_cpu_time;
pub mod queue_poller;
pub mod telemetry;
pub mod turso_queue;

use adapt_cache::AdaptCache;
use anyhow::*;
use bytes::Bytes;
pub use deployment::{Deployment, DeploymentMap};
use execute::*;
pub use execute::EnvVars;
use forte_fetch::ForteFetchHandler;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use crate::measure_cpu_time::SystemClock;
use std::collections::HashMap;
use std::string::FromUtf8Error;
use std::sync::{Arc, RwLock};
use wasmtime_wasi_http::bindings::ProxyPre;

pub use ski::{FetchHandler, FetchHandlerFuture};
pub use wasmtime;

pub type WasmProxyPre = ProxyPre<ClientState<SystemClock>>;
pub type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;

#[derive(Clone)]
pub struct StaticAsset {
    pub bytes: Bytes,
    pub content_type: &'static str,
}

pub struct Fn0<J>
where
    J: AdaptCache<String, FromUtf8Error>,
{
    js_cache: J,
    deployment_map: RwLock<DeploymentMap>,
    wasm_executor: WasmExecutor,
    public_assets: RwLock<HashMap<String, Arc<HashMap<String, StaticAsset>>>>,
}

impl<J> Fn0<J>
where
    J: AdaptCache<String, FromUtf8Error> + Send + Sync + 'static,
{
    pub fn new<W>(wasm_proxy_cache: W, js_cache: J, deployment_map: DeploymentMap, env_vars: EnvVars) -> Self
    where
        W: AdaptCache<ProxyPre<ClientState<SystemClock>>, wasmtime::Error>,
    {
        Self {
            js_cache,
            deployment_map: RwLock::new(deployment_map),
            wasm_executor: WasmExecutor::new(wasm_proxy_cache, SystemClock, env_vars),
            public_assets: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_deployment(&self, code_id: &str, deployment: Deployment) {
        self.deployment_map
            .write()
            .unwrap()
            .register_deployment(code_id, deployment);
    }

    pub fn unregister_deployment(&self, code_id: &str) {
        self.deployment_map
            .write()
            .unwrap()
            .unregister_deployment(code_id);
        self.public_assets.write().unwrap().remove(code_id);
        self.wasm_executor.clear_env(code_id);
    }

    pub fn set_public_assets(&self, code_id: &str, assets: HashMap<String, Vec<u8>>) {
        let converted: HashMap<String, StaticAsset> = assets
            .into_iter()
            .map(|(path, bytes)| {
                let content_type = get_content_type(&path);
                (
                    path,
                    StaticAsset {
                        bytes: Bytes::from(bytes),
                        content_type,
                    },
                )
            })
            .collect();
        self.public_assets
            .write()
            .unwrap()
            .insert(code_id.to_string(), Arc::new(converted));
    }

    pub fn set_env(&self, code_id: &str, new_vars: Vec<(String, String)>) {
        self.wasm_executor.set_env(code_id, new_vars);
    }

    pub fn clear_env(&self, code_id: &str) {
        self.wasm_executor.clear_env(code_id);
    }

    pub async fn run(
        self: &Arc<Self>,
        code_id: &str,
        _script_path: &str,
        request: Request,
        fetch_handler: Option<Arc<dyn FetchHandler>>,
    ) -> Result<Response> {
        let deployment = {
            let map = self.deployment_map.read().unwrap();
            map.deployment(code_id)
        };
        let Some(deployment) = deployment else {
            return Err(anyhow!("code_id not found: {}", code_id));
        };

        telemetry::function_invocation(code_id);
        let start = std::time::Instant::now();

        let result = match deployment {
            Deployment::Wasm => self.wasm_executor.run(&format!("{code_id}::backend"), request).await,
            Deployment::Forte => {
                self.run_forte(code_id, request, fetch_handler)
                    .await
            }
        };

        telemetry::execution_time(code_id, start.elapsed());

        result
    }

    pub async fn run_backend_only(
        self: &Arc<Self>,
        code_id: &str,
        request: Request,
    ) -> Result<Response> {
        self.wasm_executor
            .run(&format!("{code_id}::backend"), request)
            .await
    }

    pub(crate) async fn run_forte_backend(
        self: &Arc<Self>,
        code_id: &str,
        _script_path: &str,
        request: Request,
        _fetch_handler: Option<Arc<dyn FetchHandler>>,
    ) -> anyhow::Result<Response> {
        self.wasm_executor
            .run(&format!("{code_id}::backend"), request)
            .await
    }

    async fn run_forte(
        self: &Arc<Self>,
        code_id: &str,
        request: Request,
        _fetch_handler: Option<Arc<dyn FetchHandler>>,
    ) -> Result<Response> {
        let uri = request.uri().clone();
        let path = uri.path().to_string();

        if let Some(asset_response) = self.try_serve_public_asset(code_id, &path) {
            return Ok(asset_response);
        }

        let original_headers = request.headers().clone();

        let backend_response = self
            .wasm_executor
            .run(&format!("{code_id}::backend"), request)
            .await?;

        let backend_status = backend_response.status();

        if backend_status.is_redirection()
            || backend_status.is_client_error()
            || backend_status.is_server_error()
        {
            return Ok(backend_response);
        }

        if path.starts_with("/__forte_hook/")
            || path.starts_with("/__forte_action/")
            || path.starts_with("/api/")
        {
            return Ok(backend_response);
        }

        let frontend_request = hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(backend_response.into_body())?;

        let js_code = self
            .js_cache
            .get(&format!("{code_id}::frontend"), |bytes| {
                String::from_utf8(bytes.to_vec()).map(|s| (s, bytes.len()))
            })
            .await
            .map_err(|err| anyhow!("Failed to get frontend JS code: {:?}", err))?;

        let handler = Arc::new(ForteFetchHandler::new(
            self.clone(),
            code_id.to_string(),
            original_headers,
        ));

        let mut ssr_response = ski::run(
            &js_code,
            "/frontend.js",
            frontend_request,
            Some(handler.clone()),
        )
        .await?;

        for cookie in handler.get_collected_cookies() {
            if let std::result::Result::Ok(value) = cookie.parse() {
                ssr_response
                    .headers_mut()
                    .append(hyper::header::SET_COOKIE, value);
            }
        }

        Ok(ssr_response)
    }

    fn try_serve_public_asset(&self, code_id: &str, path: &str) -> Option<Response> {
        let assets = {
            let guard = self.public_assets.read().unwrap();
            guard.get(code_id).cloned()
        }?;

        let stripped = path.strip_prefix("/public/").map(|s| format!("/{s}"));

        let asset = assets
            .get(path)
            .or_else(|| stripped.as_deref().and_then(|p| assets.get(p)))?;

        let body: Body = Full::new(asset.bytes.clone())
            .map_err(|e| anyhow!("{e}"))
            .boxed_unsync();

        Some(
            hyper::Response::builder()
                .status(200)
                .header("content-type", asset.content_type)
                .header("cache-control", "public, max-age=3600")
                .body(body)
                .unwrap(),
        )
    }
}

fn get_content_type(path: &str) -> &'static str {
    let ext = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("pdf") => "application/pdf",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        _ => "application/octet-stream",
    }
}

pub use fn0_wasmtime::{compile, VERSION as FN0_WASMTIME_VERSION};
