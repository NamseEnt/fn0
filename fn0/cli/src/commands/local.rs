use bytes::Bytes;
use color_eyre::Result;
use fn0::cache::{Bundle, BundleCache, Error as CacheError};
use fn0::execute::ClientState;
use fn0::measure_cpu_time::SystemClock;
use fn0::wasmtime::Engine;
use fn0::wasmtime::component::Linker;
use fn0::{CodeExecutor, Deployment, ExecutionContext};
use http_body_util::{BodyExt, combinators::UnsyncBoxBody};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;

pub async fn execute(port: Option<u16>) -> Result<()> {
    println!("Starting local fn0 server...\n");

    crate::commands::build::execute().await?;

    let wasm_path = PathBuf::from("./dist/component.wasm");
    if !wasm_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "dist/component.wasm not found. Build first."
        ));
    }

    let engine = fn0::build_engine()?;
    fn0::spawn_epoch_ticker(engine.clone());
    let linker = fn0::build_linker(&engine);

    let cache = LocalCache::new(wasm_path, engine.clone(), linker.clone());
    let ctx = Arc::new(ExecutionContext::new(engine, linker, cache));
    let executor = Rc::new(CodeExecutor::new(ctx));

    let port = port.unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    println!("Server on http://localhost:{}", port);

    loop {
        let (socket, _) = listener.accept().await?;
        let executor = executor.clone();

        tokio::task::spawn_local(async move {
            let io = TokioIo::new(socket);
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let executor = executor.clone();
                        async move {
                            let resp = executor
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
                                            .map_err(
                                                |e: std::convert::Infallible| anyhow::anyhow!(e),
                                            ),
                                        );
                                    Ok(hyper::Response::builder().status(500).body(body).unwrap())
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

struct LocalCache {
    wasm_path: PathBuf,
    engine: Engine,
    linker: Linker<ClientState<SystemClock>>,
    cell: OnceCell<Arc<Bundle>>,
}

impl LocalCache {
    fn new(wasm_path: PathBuf, engine: Engine, linker: Linker<ClientState<SystemClock>>) -> Self {
        Self {
            wasm_path,
            engine,
            linker,
            cell: OnceCell::new(),
        }
    }

    async fn build_bundle(&self) -> Result<Arc<Bundle>, CacheError> {
        let data = tokio::fs::read(&self.wasm_path)
            .await
            .map_err(|e| CacheError::Storage(anyhow::anyhow!(e)))?;
        eprintln!("Compiling WASM ({} bytes)...", data.len());
        let cwasm = fn0::compile(&data).map_err(CacheError::Compile)?;
        eprintln!("Compilation done ({} bytes)", cwasm.len());
        let service_pre = fn0::build_service_pre(&self.engine, &self.linker, &cwasm)
            .map_err(CacheError::Compile)?;
        Ok(Arc::new(Bundle {
            service_pre,
            js: None,
            deployment: Deployment::Wasm,
            env_vars: Vec::new(),
        }))
    }
}

impl BundleCache for LocalCache {
    async fn get(&self, subdomain: &str) -> Result<Arc<Bundle>, CacheError> {
        if subdomain != "local" {
            return Err(CacheError::NotFound);
        }
        self.cell
            .get_or_try_init(|| self.build_bundle())
            .await
            .cloned()
    }

    async fn invalidate(&self, _subdomain: &str) {}
}
