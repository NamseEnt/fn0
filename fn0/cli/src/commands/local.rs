use bytes::Bytes;
use color_eyre::Result;
use doc_db_protocol::DocDbRequest;
use fn0::cache::{Bundle, BundleCache, Error as CacheError};
use fn0::execute::ClientState;
use fn0::measure_cpu_time::SystemClock;
use fn0::wasmtime::Engine;
use fn0::wasmtime::component::Linker;
use fn0::{CodeExecutor, DocDbHijack, DocDbService, DocDbServiceFuture, ExecutionContext};
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

const LOCAL_CODE_VERSION: u64 = 1;
const DOC_DB_PLACEHOLDER_HOST: &str = "fn0-doc-db.fn0.dev";

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
    let doc_db_service = Arc::new(LocalDocDbService::new());
    let doc_db_hijack = Arc::new(DocDbHijack::new(
        DOC_DB_PLACEHOLDER_HOST.to_string(),
        doc_db_service,
    ));
    let ctx =
        Arc::new(ExecutionContext::new(engine, linker, cache).with_doc_db_hijack(doc_db_hijack));
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

struct LocalDocDbService {
    db: doc_db::Database,
}

impl LocalDocDbService {
    fn new() -> Self {
        Self {
            db: doc_db::memory(),
        }
    }
}

impl DocDbService for LocalDocDbService {
    fn execute<'a>(
        &'a self,
        _project_id: &'a str,
        request: DocDbRequest,
    ) -> DocDbServiceFuture<'a> {
        Box::pin(async move {
            self.db
                .execute_semantic(request)
                .await
                .map_err(|error| error.to_string())
        })
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
            env_vars: Vec::new(),
            code_version: Some(LOCAL_CODE_VERSION),
            static_cache_enabled: true,
        }))
    }
}

impl BundleCache for LocalCache {
    async fn get(&self, project_id: &str) -> Result<Arc<Bundle>, CacheError> {
        if project_id != "local" {
            return Err(CacheError::NotFound);
        }
        self.cell
            .get_or_try_init(|| self.build_bundle())
            .await
            .cloned()
    }

    async fn invalidate(&self, _project_id: &str) {}

    async fn registered_project_ids(&self) -> std::collections::HashSet<String> {
        std::collections::HashSet::from(["local".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_db_protocol::{BinaryDocument, DocDbKey, DocDbOperation, DocDbResult, encode_request};

    fn put_request(data: &[u8]) -> DocDbRequest {
        DocDbRequest::new(DocDbOperation::Put {
            key: DocDbKey::new("pk", "sk"),
            data: data.to_vec(),
        })
    }

    fn get_request() -> DocDbRequest {
        DocDbRequest::new(DocDbOperation::Get {
            key: DocDbKey::new("pk", "sk"),
        })
    }

    #[tokio::test]
    async fn keeps_one_memory_database_for_the_service_lifetime() {
        let service = LocalDocDbService::new();

        assert_eq!(
            service
                .execute("local", put_request(b"value"))
                .await
                .unwrap()
                .result,
            DocDbResult::Put
        );
        assert_eq!(
            service
                .execute("local", get_request())
                .await
                .unwrap()
                .result,
            DocDbResult::Get {
                data: Some(BinaryDocument {
                    data: b"value".to_vec(),
                })
            }
        );
    }

    #[tokio::test]
    async fn routes_semantic_requests_through_the_hijack_to_memory() {
        let hijack = DocDbHijack::new(
            DOC_DB_PLACEHOLDER_HOST.to_string(),
            Arc::new(LocalDocDbService::new()),
        );

        let put_body = encode_request(&put_request(b"value")).unwrap();
        assert_eq!(
            hijack.handle("local", &put_body).await.result,
            DocDbResult::Put
        );

        let get_body = encode_request(&get_request()).unwrap();
        assert_eq!(
            hijack.handle("local", &get_body).await.result,
            DocDbResult::Get {
                data: Some(BinaryDocument {
                    data: b"value".to_vec(),
                })
            }
        );
    }
}
