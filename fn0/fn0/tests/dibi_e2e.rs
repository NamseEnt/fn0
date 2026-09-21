use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use anyhow::{Result, anyhow, ensure};
use bytes::Bytes;
use dibi::{
    ConflictInjection, DibiEngine, DibiServer, DibiServerConfig, DibiServerTestSupport,
};
use dibi::dibi_protocol::{RequestOperation, encode_request_frame};
use fn0::{
    Bundle, BundleCache, CodeExecutor, DibiBridge, DibiBridgeConfig, DibiHijack,
    ExecutionContext,
};
use http_body_util::{BodyExt, Full};
use hyper::{Method, StatusCode, Uri, header};
use rcgen::generate_simple_self_signed;
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};

const WORKER_TOKEN: &[u8] = b"e2e-worker-token";
const PLACEHOLDER_HOST: &str = "fn0-dibi.fn0.dev";

struct StaticBundleCache {
    bundle: Arc<Bundle>,
}

impl BundleCache for StaticBundleCache {
    async fn get(&self, _project_id: &str) -> Result<Arc<Bundle>, fn0::cache::Error> {
        Ok(self.bundle.clone())
    }

    async fn invalidate(&self, _project_id: &str) {}

    async fn registered_project_ids(&self) -> std::collections::HashSet<String> {
        std::collections::HashSet::new()
    }
}

struct RunningServer {
    support: Arc<DibiServerTestSupport>,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<std::result::Result<(), dibi::ServerError>>>,
}

impl RunningServer {
    async fn stop(&mut self) -> Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown.send(()).map_err(|_| anyhow!("server shutdown failed"))?;
        }
        self.task
            .take()
            .ok_or_else(|| anyhow!("server task already stopped"))?
            .await
            .map_err(|error| anyhow!("server task failed: {error}"))??;
        Ok(())
    }
}

struct DibiFixture {
    _directory: TempDir,
    data_dir: PathBuf,
    cert_path: PathBuf,
    key_path: PathBuf,
    certificate_pem: String,
    server: RunningServer,
    address: std::net::SocketAddr,
}

impl DibiFixture {
    async fn new() -> Result<Self> {
        let directory = tempfile::tempdir()?;
        let data_dir = directory.path().join("rocksdb");
        let cert_path = directory.path().join("server-cert.pem");
        let key_path = directory.path().join("server-key.pem");
        let certificate = generate_simple_self_signed(vec!["localhost".to_owned()])?;
        let certificate_pem = certificate.cert.pem();
        fs::write(&cert_path, &certificate_pem)?;
        fs::write(&key_path, certificate.signing_key.serialize_pem())?;
        let (server, address) = start_server(&data_dir, &cert_path, &key_path).await?;
        Ok(Self {
            _directory: directory,
            data_dir,
            cert_path,
            key_path,
            certificate_pem,
            server,
            address,
        })
    }

    async fn restart(&mut self) -> Result<()> {
        self.server.stop().await?;
        let (server, address) = start_server(&self.data_dir, &self.cert_path, &self.key_path).await?;
        self.server = server;
        self.address = address;
        Ok(())
    }
}

async fn start_server(
    data_dir: &Path,
    cert_path: &Path,
    key_path: &Path,
) -> Result<(RunningServer, std::net::SocketAddr)> {
    let support = Arc::new(DibiServerTestSupport::default());
    let config = DibiServerConfig::new(
        data_dir,
        "127.0.0.1:0".parse()?,
        cert_path,
        key_path,
        WORKER_TOKEN.to_vec(),
    )
    .with_test_support(support.clone());
    let server = DibiServer::bind(config)?;
    let address = server.local_addr()?;
    let (shutdown, receiver) = oneshot::channel();
    let task = tokio::spawn(server.run(async move {
        let _ = receiver.await;
    }));
    Ok((
        RunningServer {
            support,
            shutdown: Some(shutdown),
        task: Some(task),
        },
        address,
    ))
}

fn build_hijack(
    address: std::net::SocketAddr,
    certificate_pem: Option<&str>,
    worker_token: &[u8],
) -> Result<Arc<DibiHijack>> {
    let bridge = Arc::new(DibiBridge::new(DibiBridgeConfig {
        placeholder_host: PLACEHOLDER_HOST.to_owned(),
        target_host: address.ip().to_string(),
        target_port: address.port(),
        server_name: "localhost".to_owned(),
        worker_token: worker_token.to_vec(),
        ca_certificate: certificate_pem.map(str::as_bytes).map(ToOwned::to_owned),
        connect_timeout: Duration::from_secs(2),
        request_timeout: Duration::from_secs(2),
    })?);
    Ok(Arc::new(DibiHijack::new(
        PLACEHOLDER_HOST.to_owned(),
        bridge,
    )))
}

fn build_executor(hijack: Arc<DibiHijack>) -> Result<Rc<CodeExecutor<StaticBundleCache>>> {
    let engine = fn0::build_engine()?;
    let linker = fn0::build_linker(&engine);
    let wasm = include_bytes!(env!("FN0_DIBI_E2E_WASM"));
    ensure!(
        !wasm
            .windows(b"fn0:dibi-transport".len())
            .any(|window| window == b"fn0:dibi-transport"),
        "fixture contains the removed custom Dibi WIT import"
    );
    let compiled = fn0::compile(wasm).map_err(|error| anyhow!("fixture compile failed: {error:?}"))?;
    let service_pre = fn0::build_service_pre(&engine, &linker, &compiled)
        .map_err(|error| anyhow!("fixture link failed: {error:?}"))?;
    let bundle = Arc::new(Bundle {
        service_pre,
        js: None,
        env_vars: vec![("DIBI_URL".to_owned(), "http://evil.example".to_owned())],
        code_version: Some(1),
        static_cache_enabled: false,
    });
    let context = ExecutionContext::new(engine, linker, StaticBundleCache { bundle })
        .with_dibi_hijack(hijack);
    Ok(Rc::new(CodeExecutor::new(Arc::new(context))))
}

async fn invoke(
    executor: &CodeExecutor<StaticBundleCache>,
    project_id: &str,
    path: &str,
    body: Option<&[u8]>,
) -> Result<(StatusCode, serde_json::Value)> {
    let request_body = body.unwrap_or_default().to_vec();
    let request = hyper::Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            Full::new(Bytes::from(request_body))
                .map_err(|never: std::convert::Infallible| anyhow!(never))
                .boxed_unsync(),
        )?;
    let response = executor.run_backend_only(project_id, request).await?;
    let status = response.status();
    let body = response.into_body().collect().await?.to_bytes();
    let value = serde_json::from_slice(&body).unwrap_or_else(|_| {
        serde_json::json!({
            "body": String::from_utf8_lossy(&body).to_string(),
        })
    });
    Ok((status, value))
}

async fn invoke_ok(
    executor: &CodeExecutor<StaticBundleCache>,
    project_id: &str,
    path: &str,
    body: Option<&[u8]>,
) -> Result<serde_json::Value> {
    let (status, value) = invoke(executor, project_id, path, body).await?;
    ensure!(status == StatusCode::OK, "{path} returned {status}: {value}");
    Ok(value)
}

fn dibi_request(
    method: Method,
    uri: Uri,
    body: Vec<u8>,
) -> Result<hyper::Request<http_body_util::combinators::UnsyncBoxBody<Bytes, wasmtime_wasi_http::p3::bindings::http::types::ErrorCode>>> {
    Ok(hyper::Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(
            Full::new(Bytes::from(body))
                .map_err(|never: std::convert::Infallible| match never {})
                .boxed_unsync(),
        )?)
}

async fn hijack_status(
    hijack: &DibiHijack,
    project_id: &str,
    request: hyper::Request<http_body_util::combinators::UnsyncBoxBody<Bytes, wasmtime_wasi_http::p3::bindings::http::types::ErrorCode>>,
) -> Result<StatusCode> {
    Ok(hijack.handle_http(project_id, request).await?.status())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dibi_production_path() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let mut fixture = DibiFixture::new().await?;
            let hijack = build_hijack(
                fixture.address,
                Some(&fixture.certificate_pem),
                WORKER_TOKEN,
            )?;
            let executor = build_executor(hijack.clone())?;

            let crud = invoke_ok(&executor, "project-a", "/crud", None).await?;
            ensure!(crud["missing_before"] == true);
            ensure!(crud["first"] == serde_json::json!([102, 105, 114, 115, 116]));
            ensure!(crud["second"] == serde_json::json!([115, 101, 99, 111, 110, 100]));
            ensure!(crud["binary"] == serde_json::json!([0, 1, 2, 255, 0]));
            ensure!(crud["missing_after"] == true);

            let keys = invoke_ok(&executor, "project-a", "/keys", None).await?;
            let expected_keys = serde_json::json!(["", "a", "a\u{0}b", "a/b", "a&b", "한글", "日本語", "😀"]);
            ensure!(keys["query"] == expected_keys);
            ensure!(keys["scan"] == expected_keys);

            let batch = invoke_ok(&executor, "project-a", "/batch", None).await?;
            ensure!(batch["A"] == serde_json::json!([97]));
            ensure!(batch["B"] == serde_json::json!([98]));
            ensure!(batch["C"].is_null());
            ensure!(batch["X"] == serde_json::json!([50]));

            let transaction = invoke_ok(&executor, "project-a", "/transaction", None).await?;
            ensure!(transaction["rollback_c_missing"] == true);
            ensure!(transaction["rollback_d"] == serde_json::json!([101, 120, 105, 115, 116, 105, 110, 103]));
            ensure!(transaction["pending_a"] == serde_json::json!([112, 101, 110, 100, 105, 110, 103]));
            ensure!(transaction["committed_a"] == serde_json::json!([112, 101, 110, 100, 105, 110, 103]));
            ensure!(transaction["committed_b"] == serde_json::json!([99, 111, 109, 109, 105, 116, 116, 101, 100]));

            let tenant_a = invoke_ok(
                &executor,
                "project-a",
                "/tenant/write",
                Some(br#"{"value":"A"}"#),
            )
            .await?;
            let tenant_b = invoke_ok(
                &executor,
                "project-b",
                "/tenant/write",
                Some(br#"{"value":"B"}"#),
            )
            .await?;
            ensure!(tenant_a == "A");
            ensure!(tenant_b == "B");
            let tenant_a_read = invoke_ok(&executor, "project-a", "/tenant/read", None).await?;
            let tenant_b_read = invoke_ok(&executor, "project-b", "/tenant/read", None).await?;
            ensure!(tenant_a_read["get"] == serde_json::json!([65]));
            ensure!(tenant_b_read["get"] == serde_json::json!([66]));
            ensure!(tenant_a_read["query"] == serde_json::json!([[65]]));
            ensure!(tenant_b_read["query"] == serde_json::json!([[66]]));
            ensure!(tenant_a_read["scan"] == serde_json::json!([[65]]));
            ensure!(tenant_b_read["scan"] == serde_json::json!([[66]]));
            ensure!(tenant_a_read["admin"] == serde_json::json!([[65]]));
            ensure!(tenant_b_read["admin"] == serde_json::json!([[66]]));

            let success_seed = invoke_ok(&executor, "project-a", "/seed/conflict", None).await?;
            ensure!(success_seed == true);
            let success = invoke_ok(&executor, "project-a", "/trx/success", None).await?;
            ensure!(success["attempts"] == 1);
            ensure!(success["value"] == 2);

            invoke_ok(&executor, "project-a", "/seed/conflict", None).await?;
            fixture.server.support.inject_conflict_once(ConflictInjection::Put {
                tenant: "project-a".to_owned(),
                pk: "e2e-doc".to_owned(),
                sk: "conflict".to_owned(),
                data: serde_json::to_vec(&serde_json::json!({"key":"conflict","value":99}))?,
            });
            let conflict = invoke_ok(&executor, "project-a", "/trx/conflict", None).await?;
            ensure!(conflict["attempts"] == 2);
            ensure!(conflict["value"] == 100);

            invoke_ok(&executor, "project-a", "/seed/dependency", None).await?;
            fixture.server.support.inject_conflict_once(ConflictInjection::Put {
                tenant: "project-a".to_owned(),
                pk: "e2e-doc".to_owned(),
                sk: "dependency-a".to_owned(),
                data: serde_json::to_vec(&serde_json::json!({"key":"dependency-a","value":5}))?,
            });
            let dependency = invoke_ok(&executor, "project-a", "/trx/dependency", None).await?;
            ensure!(dependency == 2);

            invoke_ok(&executor, "project-a", "/seed/missing", None).await?;
            fixture.server.support.inject_conflict_once(ConflictInjection::Put {
                tenant: "project-a".to_owned(),
                pk: "e2e-doc".to_owned(),
                sk: "missing-a".to_owned(),
                data: serde_json::to_vec(&serde_json::json!({"key":"missing-a","value":7}))?,
            });
            let missing = invoke_ok(&executor, "project-a", "/trx/missing", None).await?;
            ensure!(missing == 2);

            invoke_ok(&executor, "project-a", "/seed/readonly", None).await?;
            fixture.server.support.inject_conflict_once(ConflictInjection::Put {
                tenant: "project-a".to_owned(),
                pk: "e2e-doc".to_owned(),
                sk: "readonly".to_owned(),
                data: serde_json::to_vec(&serde_json::json!({"key":"readonly","value":2}))?,
            });
            let readonly = invoke_ok(&executor, "project-a", "/trx/readonly", None).await?;
            ensure!(readonly["attempts"] == 2);
            ensure!(readonly["value"] == 2);

            invoke_ok(&executor, "project-a", "/seed/conflict", None).await?;
            fixture.server.support.synchronize_batch_gets(2);
            let (concurrent_first, concurrent_second) = tokio::join!(
                invoke(&executor, "project-a", "/trx/conflict", None),
                invoke(&executor, "project-a", "/trx/conflict", None),
            );
            let concurrent_first = concurrent_first?;
            let concurrent_second = concurrent_second?;
            ensure!(concurrent_first.0 == StatusCode::OK);
            ensure!(concurrent_second.0 == StatusCode::OK);
            let concurrent_attempts = [
                concurrent_first.1["attempts"].as_i64().unwrap_or_default(),
                concurrent_second.1["attempts"].as_i64().unwrap_or_default(),
            ];
            ensure!(concurrent_attempts.contains(&2));

            let malformed_before = fixture.server.support.metrics().request_stream_count();
            let malformed = hijack_status(
                &hijack,
                "project-a",
                dibi_request(
                    Method::POST,
                    Uri::from_static("http://fn0-dibi.fn0.dev/"),
                    vec![0, 1, 2],
                )?,
            )
            .await?;
            ensure!(malformed == StatusCode::BAD_REQUEST);
            ensure!(fixture.server.support.metrics().request_stream_count() == malformed_before);

            let spoofed_frame = encode_request_frame(
                1,
                "project-b",
                &RequestOperation::Get {
                    pk: "shared".to_owned(),
                    sk: "key".to_owned(),
                },
            );
            let spoofed = hijack_status(
                &hijack,
                "project-a",
                dibi_request(
                    Method::POST,
                    Uri::from_static("http://fn0-dibi.fn0.dev/"),
                    spoofed_frame,
                )?,
            )
            .await?;
            ensure!(spoofed == StatusCode::BAD_REQUEST);
            ensure!(fixture.server.support.metrics().request_stream_count() == malformed_before);

            let wrong_method = hijack_status(
                &hijack,
                "project-a",
                dibi_request(
                    Method::GET,
                    Uri::from_static("http://fn0-dibi.fn0.dev/"),
                    Vec::new(),
                )?,
            )
            .await?;
            ensure!(wrong_method == StatusCode::METHOD_NOT_ALLOWED);
            let wrong_path = hijack_status(
                &hijack,
                "project-a",
                dibi_request(
                    Method::POST,
                    Uri::from_static("http://fn0-dibi.fn0.dev/other"),
                    Vec::new(),
                )?,
            )
            .await?;
            ensure!(wrong_path == StatusCode::NOT_FOUND);
            let oversized = hijack_status(
                &hijack,
                "project-a",
                dibi_request(
                    Method::POST,
                    Uri::from_static("http://fn0-dibi.fn0.dev/"),
                    vec![0; dibi::dibi_protocol::MAX_FRAME_SIZE + 1],
                )?,
            )
            .await?;
            ensure!(oversized == StatusCode::PAYLOAD_TOO_LARGE);
            ensure!(hijack.matches(&Uri::from_static("http://fn0-dibi.fn0.dev/")));
            ensure!(!hijack.matches(&Uri::from_static("http://evil.example/")));

            let persistence_data = invoke_ok(
                &executor,
                "project-a",
                "/tenant/write",
                Some(br#"{"value":"persisted"}"#),
            )
            .await?;
            ensure!(persistence_data == "persisted");

            let bad_token_hijack = build_hijack(
                fixture.address,
                Some(&fixture.certificate_pem),
                b"wrong-worker-token",
            )?;
            let bad_token_executor = build_executor(bad_token_hijack)?;
            let (bad_token_status, bad_token_body) = invoke(
                &bad_token_executor,
                "project-a",
                "/error",
                None,
            )
            .await?;
            ensure!(bad_token_status == StatusCode::INTERNAL_SERVER_ERROR);
            ensure!(!bad_token_body.to_string().contains("wrong-worker-token"));

            let bad_certificate_hijack = build_hijack(fixture.address, None, WORKER_TOKEN)?;
            let bad_certificate_executor = build_executor(bad_certificate_hijack)?;
            let (bad_certificate_status, _) = invoke(
                &bad_certificate_executor,
                "project-a",
                "/error",
                None,
            )
            .await?;
            ensure!(bad_certificate_status == StatusCode::INTERNAL_SERVER_ERROR);

            let first_metrics = fixture.server.support.metrics();
            ensure!(first_metrics.connection_count() == 1);
            ensure!(first_metrics.authentication_count() == 1);
            ensure!(first_metrics.request_stream_count() > 1);
            fixture.restart().await?;
            let persisted = invoke_ok(&executor, "project-a", "/tenant/read", None).await?;
            ensure!(persisted["get"] == serde_json::json!([112, 101, 114, 115, 105, 115, 116, 101, 100]));
            let reopened = DibiEngine::open(&fixture.data_dir)?;
            let stored = reopened
                .get_with_version("project-a", "shared", "key")?
                .ok_or_else(|| anyhow!("persisted document is missing"))?;
            ensure!(stored.version > 0);
            ensure!(fixture.server.support.metrics().connection_count() == 1);
            ensure!(fixture.server.support.metrics().authentication_count() == 1);

            fixture.server.stop().await?;
            Ok::<(), anyhow::Error>(())
        })
        .await
}
