use fn0::cache::{Bundle, BundleCache, Error as CacheError};
use fn0::execute::ClientState;
use fn0::measure_cpu_time::SystemClock;
use fn0::wasmtime::Engine;
use fn0::wasmtime::component::Linker;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SimpleCache {
    wasm_path: String,
    js_path: String,
    engine: Engine,
    linker: Linker<ClientState<SystemClock>>,
    state: Mutex<State>,
}

struct State {
    bundle: Option<Arc<Bundle>>,
    env_vars: Vec<(String, String)>,
}

impl SimpleCache {
    pub fn new(
        wasm_path: String,
        js_path: String,
        engine: Engine,
        linker: Linker<ClientState<SystemClock>>,
        env_vars: Vec<(String, String)>,
    ) -> Self {
        Self {
            wasm_path,
            js_path,
            engine,
            linker,
            state: Mutex::new(State {
                bundle: None,
                env_vars,
            }),
        }
    }

    pub async fn set_env(&self, new_vars: Vec<(String, String)>) {
        let mut state = self.state.lock().await;
        state.env_vars = new_vars;
        state.bundle = None;
    }

    async fn build_bundle(
        &self,
        env_vars: Vec<(String, String)>,
    ) -> Result<Arc<Bundle>, CacheError> {
        let wasm_data = tokio::fs::read(&self.wasm_path)
            .await
            .map_err(|e| CacheError::Storage(anyhow::anyhow!("read {}: {e}", self.wasm_path)))?;
        let cwasm = fn0::compile(&wasm_data).map_err(CacheError::Compile)?;
        let service_pre = fn0::build_service_pre(&self.engine, &self.linker, &cwasm)
            .map_err(CacheError::Compile)?;

        let js = if self.js_path.is_empty() {
            None
        } else {
            let js_bytes = tokio::fs::read(&self.js_path)
                .await
                .map_err(|e| CacheError::Storage(anyhow::anyhow!("read {}: {e}", self.js_path)))?;
            let js_str =
                String::from_utf8(js_bytes).map_err(|e| CacheError::Decode(anyhow::anyhow!(e)))?;
            Some(js_str)
        };

        Ok(Arc::new(Bundle {
            service_pre,
            js,
            env_vars,
            code_version: None,
            static_cache_enabled: false,
        }))
    }
}

impl BundleCache for SimpleCache {
    async fn get(&self, subdomain: &str) -> Result<Arc<Bundle>, CacheError> {
        if subdomain != super::DEV_CODE_ID {
            return Err(CacheError::NotFound);
        }
        let env_vars = {
            let state = self.state.lock().await;
            if let Some(b) = &state.bundle {
                return Ok(b.clone());
            }
            state.env_vars.clone()
        };
        let bundle = self.build_bundle(env_vars).await?;
        let mut state = self.state.lock().await;
        state.bundle = Some(bundle.clone());
        Ok(bundle)
    }

    async fn invalidate(&self, _subdomain: &str) {
        let mut state = self.state.lock().await;
        state.bundle = None;
    }

    async fn registered_project_ids(&self) -> std::collections::HashSet<String> {
        std::collections::HashSet::from([super::DEV_CODE_ID.to_string()])
    }
}
