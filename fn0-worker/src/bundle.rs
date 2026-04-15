use crate::bundle_cache::BundleCache;
use crate::bundle_store::BundleStore;
use aws_sdk_s3::Client;
use bytes::Bytes;
use color_eyre::eyre::{Result, eyre};
use fn0::{Deployment, Fn0, WasmProxyPre};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;
use std::string::FromUtf8Error;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Manifest {
    Wasm,
    Forte { frontend_script_path: String },
}

pub struct BundleFetcher {
    s3_client: Client,
    bucket: String,
    store: Arc<BundleStore>,
    wasm_cache: BundleCache<WasmProxyPre, fn0::wasmtime::Error>,
    js_cache: BundleCache<String, FromUtf8Error>,
    fn0: Arc<Fn0<BundleCache<String, FromUtf8Error>>>,
    loaded: Mutex<HashMap<String, (u64, u64)>>,
}

impl BundleFetcher {
    pub fn new(
        s3_client: Client,
        bucket: String,
        store: Arc<BundleStore>,
        wasm_cache: BundleCache<WasmProxyPre, fn0::wasmtime::Error>,
        js_cache: BundleCache<String, FromUtf8Error>,
        fn0: Arc<Fn0<BundleCache<String, FromUtf8Error>>>,
    ) -> Self {
        Self {
            s3_client,
            bucket,
            store,
            wasm_cache,
            js_cache,
            fn0,
            loaded: Mutex::new(HashMap::new()),
        }
    }

    pub async fn fetch_and_register(
        &self,
        subdomain: &str,
        code_id: u64,
        code_version: u64,
    ) -> Result<()> {
        if let Some(&existing) = self.loaded.lock().unwrap().get(subdomain) {
            if existing == (code_id, code_version) {
                return Ok(());
            }
        }

        let key = format!(
            "bundles/{version}/{subdomain}.tar.zst",
            version = fn0::FN0_WASMTIME_VERSION,
        );
        let output = self
            .s3_client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;

        let compressed = output.body.collect().await?.into_bytes();
        let tar_bytes = zstd::decode_all(compressed.as_ref())?;

        let mut manifest: Option<Manifest> = None;
        let mut backend_bytes: Option<Bytes> = None;
        let mut frontend_bytes: Option<Bytes> = None;
        let mut assets: HashMap<String, Vec<u8>> = HashMap::new();

        let mut archive = tar::Archive::new(tar_bytes.as_slice());
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            let path_str = path.to_string_lossy().to_string();

            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;

            match path_str.as_str() {
                "manifest.json" => {
                    manifest = Some(serde_json::from_slice(&buf)?);
                }
                "backend.cwasm.zst" => {
                    let decompressed = zstd::decode_all(buf.as_slice())?;
                    backend_bytes = Some(Bytes::from(decompressed));
                }
                "frontend.js" => {
                    frontend_bytes = Some(Bytes::from(buf));
                }
                other if other.starts_with("public/") => {
                    let rel = &other["public/".len()..];
                    assets.insert(format!("/{rel}"), buf);
                }
                _ => {}
            }
        }

        let manifest = manifest.ok_or_else(|| eyre!("bundle missing manifest.json"))?;
        let backend = backend_bytes.ok_or_else(|| eyre!("bundle missing backend.cwasm.zst"))?;

        let backend_key = format!("{subdomain}::backend");
        let frontend_key = format!("{subdomain}::frontend");

        self.wasm_cache.invalidate(&backend_key).await;
        self.js_cache.invalidate(&frontend_key).await;

        self.store.insert(&backend_key, backend);

        let deployment = match manifest {
            Manifest::Wasm => Deployment::Wasm,
            Manifest::Forte {
                frontend_script_path,
            } => {
                let frontend =
                    frontend_bytes.ok_or_else(|| eyre!("forte bundle missing frontend.js"))?;
                self.store.insert(&frontend_key, frontend);
                Deployment::Forte {
                    frontend_script_path,
                }
            }
        };

        let is_forte = matches!(deployment, Deployment::Forte { .. });
        self.fn0.register_deployment(subdomain, deployment);

        if is_forte {
            self.fn0.set_public_assets(subdomain, assets);
        }

        self.loaded
            .lock()
            .unwrap()
            .insert(subdomain.to_string(), (code_id, code_version));

        Ok(())
    }

    pub async fn unregister(&self, subdomain: &str) {
        self.loaded.lock().unwrap().remove(subdomain);
        let backend_key = format!("{subdomain}::backend");
        let frontend_key = format!("{subdomain}::frontend");
        self.store.remove(&backend_key);
        self.store.remove(&frontend_key);
        self.wasm_cache.invalidate(&backend_key).await;
        self.js_cache.invalidate(&frontend_key).await;
        self.fn0.unregister_deployment(subdomain);
    }
}
