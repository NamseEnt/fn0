use fn0_adapt_cache::AdaptCache;
use bytes::Bytes;
use color_eyre::Result;
use fn0::{CodeKind, DeploymentMap, Fn0};
use http_body_util::{BodyExt, combinators::UnsyncBoxBody};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub async fn execute(port: Option<u16>) -> Result<()> {
    println!("Starting local fn0 server...\n");

    crate::commands::build::execute().await?;

    let wasm_path = PathBuf::from("./dist/component.wasm");
    if !wasm_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "dist/component.wasm not found. Build first."
        ));
    }

    let mut deployment_map = DeploymentMap::new();
    deployment_map.register_code("local", CodeKind::Wasm);

    let cache = LocalCache::new(wasm_path);
    let env_vars = std::sync::Arc::new(std::sync::RwLock::new(Vec::new()));
    let fn0 = Arc::new(Fn0::new(cache.clone(), cache, deployment_map, env_vars));

    let port = port.unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    println!("Server on http://localhost:{}", port);

    loop {
        let (socket, _) = listener.accept().await?;
        let fn0 = fn0.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(socket);
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let fn0 = fn0.clone();
                        async move {
                            let resp = fn0
                                .run(
                                    "local",
                                    "/",
                                    req.map(|body| {
                                        UnsyncBoxBody::new(body)
                                            .map_err(|e: hyper::Error| anyhow::anyhow!(e))
                                            .boxed_unsync()
                                    }),
                                    None,
                                )
                                .await;

                            match resp {
                                Ok(r) => Ok::<_, anyhow::Error>(r),
                                Err(e) => {
                                    eprintln!("Error: {:?}", e);
                                    let body: UnsyncBoxBody<Bytes, anyhow::Error> =
                                        UnsyncBoxBody::new(
                                            http_body_util::Full::new(Bytes::from(
                                                "Internal Server Error",
                                            ))
                                            .map_err(|e: std::convert::Infallible| {
                                                anyhow::anyhow!(e)
                                            }),
                                        );
                                    Ok(hyper::Response::builder()
                                        .status(500)
                                        .body(body)
                                        .unwrap())
                                }
                            }
                        }
                    }),
                )
                .await
            {
                eprintln!("Failed to serve connection: {}", err);
            }
        });
    }
}

#[derive(Clone)]
struct LocalCache {
    wasm_path: PathBuf,
    memory: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl LocalCache {
    fn new(wasm_path: PathBuf) -> Self {
        Self {
            wasm_path,
            memory: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<T: Clone + Send + Sync + 'static, E: Send + 'static> AdaptCache<T, E> for LocalCache {
    async fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> std::result::Result<T, fn0_adapt_cache::Error<E>> {
        let mut cache = self.memory.lock().await;

        let bytes = if let Some(data) = cache.get(id) {
            Bytes::copy_from_slice(data)
        } else {
            if id != "local" {
                return Err(fn0_adapt_cache::Error::NotFound);
            }

            let data = tokio::fs::read(&self.wasm_path)
                .await
                .map_err(|e| fn0_adapt_cache::Error::StorageError(anyhow::anyhow!(e)))?;

            eprintln!("Compiling WASM ({} bytes)...", data.len());
            let cwasm = fn0::compile(&data)
                .map_err(|e| fn0_adapt_cache::Error::StorageError(e))?;
            eprintln!("Compilation done ({} bytes)", cwasm.len());

            cache.insert(id.to_string(), cwasm.clone());
            Bytes::from(cwasm)
        };

        let (converted, _) = convert(bytes).map_err(fn0_adapt_cache::Error::ConvertError)?;
        Ok(converted)
    }
}
