use crate::env_yaml;
use crate::vault_client::VaultClient;
use async_singleflight::Group;
use fn0::cache::{Bundle, BundleCache, Error};
use fn0::execute::ClientState;
use fn0::measure_cpu_time::SystemClock;
use fn0::wasmtime::Engine;
use fn0::wasmtime::component::Linker;
use opendal::{ErrorKind as OpendalErrorKind, Operator};
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Manifest {
    Wasm,
    WasmJs,
}

struct LruEntry {
    project_id: String,
    code_version: u64,
    bundle: Arc<Bundle>,
    size_bytes: usize,
}

struct Inner {
    registry: HashMap<String, u64>,
    domain_to_project_id: HashMap<String, String>,
    lru: VecDeque<LruEntry>,
    current_bytes: usize,
}

#[derive(Clone)]
pub struct S3BundleCache {
    engine: Engine,
    linker: Linker<ClientState<SystemClock>>,
    operator: Operator,
    vault: Arc<VaultClient>,
    inner: Arc<Mutex<Inner>>,
    cache_size_bytes: usize,
    singleflight: Arc<Group<String, Arc<Bundle>, Error>>,
}

impl S3BundleCache {
    pub fn new(
        engine: Engine,
        linker: Linker<ClientState<SystemClock>>,
        operator: Operator,
        vault: Arc<VaultClient>,
        cache_size_bytes: usize,
    ) -> Self {
        Self {
            engine,
            linker,
            operator,
            vault,
            inner: Arc::new(Mutex::new(Inner {
                registry: HashMap::new(),
                domain_to_project_id: HashMap::new(),
                lru: VecDeque::new(),
                current_bytes: 0,
            })),
            cache_size_bytes,
            singleflight: Arc::new(Group::new()),
        }
    }

    #[tracing::instrument(skip_all, fields(project_id = %project_id, code_version))]
    pub async fn register(&self, project_id: &str, code_version: u64) {
        let mut inner = self.inner.lock().await;
        let prev = inner.registry.insert(project_id.to_string(), code_version);
        if prev != Some(code_version)
            && let Some(pos) = inner.lru.iter().position(|e| e.project_id == project_id)
        {
            let removed = inner.lru.remove(pos).unwrap();
            inner.current_bytes = inner.current_bytes.saturating_sub(removed.size_bytes);
        }
    }

    pub async fn unregister(&self, project_id: &str) {
        let mut inner = self.inner.lock().await;
        inner.registry.remove(project_id);
        if let Some(pos) = inner.lru.iter().position(|e| e.project_id == project_id) {
            let removed = inner.lru.remove(pos).unwrap();
            inner.current_bytes = inner.current_bytes.saturating_sub(removed.size_bytes);
        }
    }

    pub async fn register_domain(&self, domain: &str, project_id: &str) {
        let mut inner = self.inner.lock().await;
        inner
            .domain_to_project_id
            .insert(domain.to_string(), project_id.to_string());
    }

    pub async fn unregister_domain(&self, domain: &str) {
        let mut inner = self.inner.lock().await;
        inner.domain_to_project_id.remove(domain);
    }

    pub async fn resolve_domain(&self, domain: &str) -> Option<String> {
        let inner = self.inner.lock().await;
        inner.domain_to_project_id.get(domain).cloned()
    }

    #[tracing::instrument(skip_all, fields(project_id = %project_id))]
    async fn get_impl(&self, project_id: &str) -> Result<Arc<Bundle>, Error> {
        let code_version = {
            let mut inner = self.inner.lock().await;
            let Some(&code_version) = inner.registry.get(project_id) else {
                return Err(Error::NotFound);
            };
            if let Some(pos) = inner
                .lru
                .iter()
                .position(|e| e.project_id == project_id && e.code_version == code_version)
            {
                let entry = inner.lru.remove(pos).unwrap();
                let bundle = entry.bundle.clone();
                inner.lru.push_front(entry);
                return Ok(bundle);
            }
            code_version
        };

        let (bundle, size) = self.fetch_and_build(project_id, code_version).await?;

        let mut inner = self.inner.lock().await;
        if let Some(pos) = inner.lru.iter().position(|e| e.project_id == project_id) {
            let removed = inner.lru.remove(pos).unwrap();
            inner.current_bytes = inner.current_bytes.saturating_sub(removed.size_bytes);
        }
        inner.lru.push_front(LruEntry {
            project_id: project_id.to_string(),
            code_version,
            bundle: bundle.clone(),
            size_bytes: size,
        });
        inner.current_bytes += size;

        while inner.current_bytes > self.cache_size_bytes && inner.lru.len() > 1 {
            if let Some(evicted) = inner.lru.pop_back() {
                inner.current_bytes = inner.current_bytes.saturating_sub(evicted.size_bytes);
            } else {
                break;
            }
        }

        Ok(bundle)
    }

    #[tracing::instrument(skip_all, fields(project_id = %project_id, code_version))]
    async fn fetch_and_build(
        &self,
        project_id: &str,
        code_version: u64,
    ) -> Result<(Arc<Bundle>, usize), Error> {
        let key = format!(
            "compiled/{fn0_wasmtime_version}/{project_id}/{code_version}.tar.zst",
            fn0_wasmtime_version = fn0::FN0_WASMTIME_VERSION,
        );
        let compressed = match self.operator.read(&key).await {
            Ok(buf) => buf.to_vec(),
            Err(e) if e.kind() == OpendalErrorKind::NotFound => return Err(Error::NotFound),
            Err(e) => return Err(Error::Storage(anyhow::anyhow!(e))),
        };

        let tar_bytes = zstd::decode_all(compressed.as_slice())
            .map_err(|e| Error::Decode(anyhow::anyhow!(e)))?;

        let mut manifest: Option<Manifest> = None;
        let mut wasm_bytes: Option<Vec<u8>> = None;
        let mut js_bytes: Option<Vec<u8>> = None;
        let mut env_yaml_bytes: Option<Vec<u8>> = None;

        let mut archive = tar::Archive::new(tar_bytes.as_slice());
        let entries = archive
            .entries()
            .map_err(|e| Error::Decode(anyhow::anyhow!(e)))?;
        for entry in entries {
            let mut entry = entry.map_err(|e| Error::Decode(anyhow::anyhow!(e)))?;
            let path = entry
                .path()
                .map_err(|e| Error::Decode(anyhow::anyhow!(e)))?
                .to_path_buf();
            let path_str = path.to_string_lossy().to_string();

            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| Error::Decode(anyhow::anyhow!(e)))?;

            match path_str.as_str() {
                "manifest.json" => {
                    manifest = Some(
                        serde_json::from_slice(&buf)
                            .map_err(|e| Error::Decode(anyhow::anyhow!(e)))?,
                    );
                }
                "wasm.cwasm.zst" => {
                    let decompressed = zstd::decode_all(buf.as_slice())
                        .map_err(|e| Error::Decode(anyhow::anyhow!(e)))?;
                    wasm_bytes = Some(decompressed);
                }
                "entry.js" => {
                    js_bytes = Some(buf);
                }
                "env.yaml" => {
                    env_yaml_bytes = Some(buf);
                }
                _ => {}
            }
        }

        let manifest =
            manifest.ok_or_else(|| Error::Decode(anyhow::anyhow!("missing manifest.json")))?;
        let cwasm =
            wasm_bytes.ok_or_else(|| Error::Decode(anyhow::anyhow!("missing wasm.cwasm.zst")))?;

        let service_pre =
            fn0::build_service_pre(&self.engine, &self.linker, &cwasm).map_err(Error::Compile)?;
        let cwasm_size = cwasm.len();

        let js = match manifest {
            Manifest::Wasm => None,
            Manifest::WasmJs => {
                let js = js_bytes.ok_or_else(|| {
                    Error::Decode(anyhow::anyhow!("wasmjs bundle missing entry.js"))
                })?;
                let js_str =
                    String::from_utf8(js).map_err(|e| Error::Decode(anyhow::anyhow!(e)))?;
                Some(js_str)
            }
        };
        let js_size = js.as_ref().map(|s| s.len()).unwrap_or(0);

        let env_vars = match env_yaml_bytes {
            Some(bytes) => env_yaml::load(&bytes, &self.vault)
                .await
                .map_err(|e| Error::Decode(anyhow::anyhow!("env.yaml load: {e}")))?,
            None => Vec::new(),
        };

        let size = cwasm_size + js_size;
        Ok((
            Arc::new(Bundle {
                service_pre,
                js,
                env_vars,
            }),
            size,
        ))
    }
}

impl BundleCache for S3BundleCache {
    async fn get(&self, project_id: &str) -> Result<Arc<Bundle>, Error> {
        let key = project_id.to_string();
        let provider = self.clone();
        let key_for_task = key.clone();
        let singleflight = provider.singleflight.clone();
        singleflight
            .work(&key, async move { provider.get_impl(&key_for_task).await })
            .await
            .map_err(|opt| opt.unwrap_or(Error::SingleflightLeaderFailed))
    }

    async fn invalidate(&self, project_id: &str) {
        let project_id = project_id.to_string();
        let mut inner = self.inner.lock().await;
        if let Some(pos) = inner.lru.iter().position(|e| e.project_id == project_id) {
            let removed = inner.lru.remove(pos).unwrap();
            inner.current_bytes = inner.current_bytes.saturating_sub(removed.size_bytes);
        }
    }

    async fn registered_project_ids(&self) -> std::collections::HashSet<String> {
        self.inner.lock().await.registry.keys().cloned().collect()
    }
}
