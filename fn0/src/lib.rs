mod deployment;
pub mod execute;
pub mod measure_cpu_time;
pub mod telemetry;

use adapt_cache::AdaptCache;
use anyhow::*;
use bytes::Bytes;
use deployment::*;
pub use deployment::{CodeKind, DeploymentMap};
use execute::*;
pub use execute::EnvVars;
use http_body_util::combinators::UnsyncBoxBody;
use crate::measure_cpu_time::SystemClock;
use std::string::FromUtf8Error;
use std::sync::{Arc, RwLock};
use wasmtime::Engine;
use wasmtime_wasi_http::bindings::ProxyPre;

pub use ski::{FetchHandler, FetchHandlerFuture};
pub use wasmtime;

pub type WasmProxyPre = ProxyPre<ClientState<SystemClock>>;
pub type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;

pub struct Fn0<J>
where
    J: AdaptCache<String, FromUtf8Error>,
{
    js_cache: J,
    deployment_map: RwLock<DeploymentMap>,
    wasm_executor: WasmExecutor,
    env_vars: EnvVars,
}

impl<J> Fn0<J>
where
    J: AdaptCache<String, FromUtf8Error>,
{
    pub fn new<W>(wasm_proxy_cache: W, js_cache: J, deployment_map: DeploymentMap, env_vars: EnvVars) -> Self
    where
        W: AdaptCache<ProxyPre<ClientState<SystemClock>>, wasmtime::Error>,
    {
        Self {
            js_cache,
            deployment_map: RwLock::new(deployment_map),
            wasm_executor: WasmExecutor::new(wasm_proxy_cache, SystemClock, env_vars.clone()),
            env_vars,
        }
    }

    pub fn register_code(&self, code_id: &str, kind: CodeKind) {
        self.deployment_map.write().unwrap().register_code(code_id, kind);
    }

    pub fn update_env(&self, new_vars: Vec<(String, String)>) {
        self.wasm_executor.update_env(new_vars);
    }
    pub async fn run(
        &self,
        code_id: &str,
        script_path: &str,
        request: Request,
        fetch_handler: Option<Arc<dyn FetchHandler>>,
    ) -> Result<Response> {
        let code_kind = {
            let map = self.deployment_map.read().unwrap();
            map.code_kind(code_id)
        };
        let Some(code_kind) = code_kind else {
            return Err(anyhow!("code_id not found"));
        };
        match code_kind {
            CodeKind::Wasm => Ok(self.wasm_executor.run(code_id, request).await?),
            CodeKind::Js => {
                let js_code = self
                    .js_cache
                    .get(code_id, |bytes| {
                        String::from_utf8(bytes.to_vec()).map(|str| (str, bytes.len()))
                    })
                    .await
                    .map_err(|err| anyhow!("Failed to get JS code: {:?}", err))?;
                let response = ski::run(&js_code, script_path, request, fetch_handler).await?;
                Ok(response)
            }
        }
    }
}

pub fn compile(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let engine = Engine::new(&engine_config()).map_err(|e| anyhow!("{e}"))?;

    let result = if wasm_bytes.len() > 8 && wasm_bytes[4..8] == [0x0d, 0x00, 0x01, 0x00] {
        engine.precompile_component(wasm_bytes)
    } else {
        engine.precompile_module(wasm_bytes)
    };
    result.map_err(|e| anyhow!("{e}"))
}
