use crate::metrics::{Metrics, MetricsTx};
use adapt_cache::AdaptCache;
use bytes::Bytes;
use http_body_util::BodyExt;
use measure_cpu_time::measure_cpu_time;
use tokio::sync::{
    mpsc::{Receiver, UnboundedReceiver, UnboundedSender, unbounded_channel},
    oneshot,
};
use wasmtime::{
    Engine, Store,
    component::{Component, Linker},
};
use wasmtime_wasi::*;
use wasmtime_wasi_http::{
    WasiHttpCtx, WasiHttpView,
    bindings::{
        ProxyPre,
        http::types::{ErrorCode, Scheme},
    },
    body::HyperOutgoingBody,
};

pub type Response = hyper::Response<HyperOutgoingBody>;

pub struct Job {
    pub req: hyper::Request<hyper::body::Incoming>,
    pub res_tx: oneshot::Sender<Response>,
    pub code_id: String,
}

pub struct Executor<A: AdaptCache<MyInstance, wasmtime::Error>> {
    engine: Engine,
    proxy_cache: A,
    job_rx: Receiver<Job>,
    linker: Linker<ClientState>,
    free_instances: Vec<MyInstance>,
    free_instance_tx: UnboundedSender<MyInstance>,
    free_instance_rx: UnboundedReceiver<MyInstance>,
    metrics_tx: MetricsTx,
}

impl<A> Executor<A>
where
    A: AdaptCache<MyInstance, wasmtime::Error>,
{
    pub fn new(
        engine: Engine,
        proxy_cache: A,
        job_rx: Receiver<Job>,
        metrics_tx: MetricsTx,
    ) -> Self {
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker).unwrap();
        wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker).unwrap();
        let (free_instance_tx, free_instance_rx) = unbounded_channel::<MyInstance>();
        Self {
            engine,
            proxy_cache,
            job_rx,
            linker,
            free_instances: Vec::new(),
            free_instance_tx,
            free_instance_rx,
            metrics_tx,
        }
    }
    pub async fn run(&mut self) {
        tokio::select! {
            biased;
            Some(instance) = self.free_instance_rx.recv() => {
                self.free_instances.push(instance)
            }
            Some(job) = self.job_rx.recv() => {
                self.spawn_job_runner(job);
            }
        }
    }
    fn spawn_job_runner(&mut self, job: Job) {
        let free_instance = self.try_pop_free_instance(&job.code_id);
        let proxy_cache = self.proxy_cache.clone();
        let engine = self.engine.clone();
        let linker = self.linker.clone();
        let free_instance_tx = self.free_instance_tx.clone();
        let metrics_tx = self.metrics_tx.clone();

        // TODO: Throttle and hard limit for same code_id

        tokio::spawn(async move {
            run_job(
                job,
                free_instance,
                proxy_cache,
                engine,
                linker,
                free_instance_tx,
                metrics_tx,
            )
            .await;
        });
    }
    fn try_pop_free_instance(&mut self, code_id: &str) -> Option<MyInstance> {
        let index = self
            .free_instances
            .iter()
            .enumerate()
            .find_map(|(index, instance)| {
                if instance.code_id == code_id {
                    Some(index)
                } else {
                    None
                }
            })?;
        Some(self.free_instances.remove(index))
    }
}

async fn run_job<A>(
    job: Job,
    free_instance: Option<MyInstance>,
    proxy_cache: A,
    engine: Engine,
    linker: Linker<ClientState>,
    free_instance_tx: UnboundedSender<MyInstance>,
    metrics_tx: MetricsTx,
) where
    A: AdaptCache<MyInstance, wasmtime::Error>,
{
    let Ok(instance) = pop_or_create_instance(
        free_instance,
        job.code_id.clone(),
        proxy_cache,
        engine,
        linker,
        metrics_tx.clone(),
    )
    .await
    else {
        let _ = job.res_tx.send(internal_error_response());
        return;
    };

    let response = handle_request(instance.pre.clone(), job.req, job.code_id, metrics_tx).await;

    let _ = job.res_tx.send(response);
    let _ = free_instance_tx.send(instance);
}

async fn pop_or_create_instance<A>(
    free_instance: Option<MyInstance>,
    code_id: String,
    proxy_cache: A,
    engine: Engine,
    linker: Linker<ClientState>,
    metrics_tx: MetricsTx,
) -> Result<MyInstance, ()>
where
    A: AdaptCache<MyInstance, wasmtime::Error>,
{
    if let Some(instance) = free_instance {
        metrics_tx.send(Metrics::ReuseInstance { code_id });
        return Ok(instance);
    }
    match proxy_cache
        .get(&code_id.clone(), |bytes| {
            let component = unsafe { Component::deserialize(&engine, &bytes)? };
            let instance_pre = linker.instantiate_pre(&component)?;
            let proxy_pre = ProxyPre::new(instance_pre)?;
            metrics_tx.send(Metrics::CreateInstance {
                code_id: code_id.clone(),
            });
            Ok((
                MyInstance {
                    code_id: code_id.clone(),
                    pre: proxy_pre,
                },
                bytes.len(),
            ))
        })
        .await
    {
        Ok(instance) => Ok(instance),
        Err(error) => {
            metrics_tx.send(Metrics::ProxyCacheError { code_id, error });
            Err(())
        }
    }
}

#[derive(Clone)]
pub struct MyInstance {
    code_id: String,
    pre: ProxyPre<ClientState>,
}

async fn handle_request(
    pre: ProxyPre<ClientState>,
    req: hyper::Request<hyper::body::Incoming>,
    code_id: String,
    metrics_tx: MetricsTx,
) -> Response {
    /*
    Rules or Features
    - 최대 시간 15분
    - task future를 감싸서 poll한 시간만 재서 cpu time을 측정
    - 1 req 1 instance, drop after each job
    - 모든 내부 에러는 500 에러로 사용자에게 전달
    - 모든 내부 에러는 로그로 남김
    - instance 재사용하지 않음.
    */
    let mut store = Store::new(
        pre.engine(),
        ClientState {
            table: ResourceTable::new(),
            wasi: WasiCtx::builder().inherit_stdio().build(),
            http: WasiHttpCtx::new(),
        },
    );
    store.epoch_deadline_trap();
    store.set_epoch_deadline(15 * 60);
    store.epoch_deadline_async_yield_and_update(1);

    let (tx, rx) = tokio::sync::oneshot::channel();
    let req = match store.data_mut().new_incoming_request(Scheme::Http, req) {
        Ok(x) => x,
        Err(error) => {
            metrics_tx.send(Metrics::Wasmtime {
                func: "new_incoming_request",
                code_id,
                error,
            });
            return internal_error_response();
        }
    };
    let out = match store.data_mut().new_response_outparam(tx) {
        Ok(x) => x,
        Err(error) => {
            metrics_tx.send(Metrics::Wasmtime {
                func: "new_response_outparam",
                code_id,
                error,
            });
            return internal_error_response();
        }
    };

    // NOTE: Bad guys can put infinite loop in initialization code so it would be needed to measure cpu time from here.
    // But also it includes wasmtime's instantiation.
    // I guess we can put limit with fuel for initialization.
    let proxy = match pre.instantiate_async(&mut store).await {
        Ok(x) => x,
        Err(error) => {
            metrics_tx.send(Metrics::Wasmtime {
                func: "instantiate_async",
                code_id,
                error,
            });
            return internal_error_response();
        }
    };

    let task = tokio::task::spawn({
        let code_id = code_id.clone();
        let metrics_tx = metrics_tx.clone();
        async move {
            let (result, cpu_time) = measure_cpu_time(
                proxy
                    .wasi_http_incoming_handler()
                    .call_handle(store, req, out),
            )
            .await;

            metrics_tx.send(Metrics::CpuTime { code_id, cpu_time });

            result
        }
    });

    let result = rx.await;

    if let Err(_oneshot_recv_err) = result {
        let result = task.await;
        if let Err(error) = result {
            metrics_tx.send(Metrics::RequestTaskJoinError { code_id, error });
            return internal_error_response();
        }
        let result = result.unwrap();

        if let Err(error) = result {
            match error.downcast::<wasmtime::Trap>() {
                Ok(trap) => {
                    metrics_tx.send(Metrics::Trapped { code_id, trap });
                }
                Err(error) => {
                    metrics_tx.send(Metrics::CanceledUnexpectedly { code_id, error });
                }
            }
        }

        return internal_error_response();
    }

    let result = result.unwrap();

    if let Ok(response) = result {
        return response;
    }

    let error_code: ErrorCode = result.unwrap_err();

    metrics_tx.send(Metrics::ProxyReturnsErrorCode {
        code_id,
        error_code,
    });
    internal_error_response()
}

fn internal_error_response() -> Response {
    let body = http_body_util::Full::new(Bytes::from("Internal Server Error"))
        .map_err(|_| ErrorCode::InternalError(None));
    let mut res = hyper::Response::new(HyperOutgoingBody::new(body));
    *res.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
    res
}

pub struct ClientState {
    wasi: WasiCtx,
    http: WasiHttpCtx,
    table: ResourceTable,
}

impl WasiView for ClientState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl WasiHttpView for ClientState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    // ===== Test Infrastructure =====

    /// Helper to create an Engine with async support enabled (matching production)
    fn create_test_engine() -> Engine {
        let mut config = wasmtime::Config::new();
        config.async_support(true);
        Engine::new(&config).unwrap()
    }

    /// Helper to create a test linker with WASI support
    fn create_test_linker(engine: &Engine) -> Linker<ClientState> {
        let mut linker = Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker).unwrap();
        wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker).unwrap();
        linker
    }

    /// Mock AdaptCache for testing
    #[derive(Clone)]
    struct MockAdaptCache {
        components: Arc<Mutex<HashMap<String, Bytes>>>,
        should_fail: Arc<AtomicBool>,
    }

    impl MockAdaptCache {
        fn new() -> Self {
            Self {
                components: Arc::new(Mutex::new(HashMap::new())),
                should_fail: Arc::new(AtomicBool::new(false)),
            }
        }

        fn insert(&self, id: String, bytes: Bytes) {
            self.components.lock().unwrap().insert(id, bytes);
        }

        fn set_should_fail(&self, should_fail: bool) {
            self.should_fail.store(should_fail, Ordering::SeqCst);
        }
    }

    impl AdaptCache<MyInstance, wasmtime::Error> for MockAdaptCache {
        async fn get(
            &self,
            id: &str,
            convert: impl FnOnce(Bytes) -> std::result::Result<(MyInstance, usize), wasmtime::Error>
            + Send,
        ) -> Result<MyInstance, adapt_cache::Error<wasmtime::Error>> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(adapt_cache::Error::StorageError(anyhow::anyhow!(
                    "Mock storage error"
                )));
            }

            let components = self.components.lock().unwrap();
            let bytes = components
                .get(id)
                .ok_or(adapt_cache::Error::NotFound)?
                .clone();
            drop(components);

            let (instance, _size) = convert(bytes).map_err(adapt_cache::Error::ConvertError)?;
            Ok(instance)
        }
    }

    /// Test MetricsTx that collects metrics for verification
    #[derive(Clone)]
    struct TestMetricsTx {
        metrics: Arc<Mutex<Vec<Metrics>>>,
    }

    impl TestMetricsTx {
        fn new() -> Self {
            Self {
                metrics: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn into_metrics_tx(self) -> MetricsTx {
            let metrics = self.metrics.clone();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            // Spawn a task to collect metrics
            tokio::spawn(async move {
                while let Some(metric) = rx.recv().await {
                    metrics.lock().unwrap().push(metric);
                }
            });

            MetricsTx::from_sender(tx)
        }

        async fn assert_contains(&self, predicate: impl Fn(&Metrics) -> bool) {
            // Give the background task time to collect metrics
            for _ in 0..10 {
                tokio::task::yield_now().await;
                let metrics = self.metrics.lock().unwrap();
                if metrics.iter().any(&predicate) {
                    return;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            let metrics = self.metrics.lock().unwrap();
            assert!(
                metrics.iter().any(predicate),
                "Expected metric not found. Metrics: {:?}",
                metrics
            );
        }

        fn count(&self, predicate: impl Fn(&Metrics) -> bool) -> usize {
            self.metrics
                .lock()
                .unwrap()
                .iter()
                .filter(|m| predicate(m))
                .count()
        }
    }

    // ===== Unit Tests =====

    mod unit {
        use super::*;

        #[test]
        fn test_internal_error_response() {
            let response = internal_error_response();

            assert_eq!(response.status(), hyper::StatusCode::INTERNAL_SERVER_ERROR);

            // Verify body content
            let (parts, _body) = response.into_parts();
            assert_eq!(parts.status, hyper::StatusCode::INTERNAL_SERVER_ERROR);
        }

        #[tokio::test]
        async fn test_try_pop_free_instance_empty() {
            let engine = create_test_engine();
            let proxy_cache = MockAdaptCache::new();
            let (_job_tx, job_rx) = mpsc::channel(10);
            let test_metrics = TestMetricsTx::new();

            let mut executor =
                Executor::new(engine, proxy_cache, job_rx, test_metrics.into_metrics_tx());

            let result = executor.try_pop_free_instance("code-a");
            assert!(
                result.is_none(),
                "Should return None when free_instances is empty"
            );
        }

        #[tokio::test]
        async fn test_try_pop_free_instance_not_found() {
            let engine = create_test_engine();
            let proxy_cache = MockAdaptCache::new();
            let (_job_tx, job_rx) = mpsc::channel(10);
            let test_metrics = TestMetricsTx::new();

            let mut executor = Executor::new(
                engine.clone(),
                proxy_cache,
                job_rx,
                test_metrics.into_metrics_tx(),
            );

            // Create a fake instance for different code_id
            let serialized = load_precompiled_sample_component();
            let component = deserialize_component(&engine, &serialized);
            let linker = create_test_linker(&engine);
            let instance_pre = linker.instantiate_pre(&component).unwrap();
            let proxy_pre = ProxyPre::new(instance_pre).unwrap();

            executor.free_instances.push(MyInstance {
                code_id: "code-b".to_string(),
                pre: proxy_pre,
            });

            let result = executor.try_pop_free_instance("code-a");
            assert!(
                result.is_none(),
                "Should return None when code_id not found"
            );
            assert_eq!(
                executor.free_instances.len(),
                1,
                "Should not remove instance"
            );
        }

        #[tokio::test]

        async fn test_try_pop_free_instance_found() {
            let engine = create_test_engine();
            let proxy_cache = MockAdaptCache::new();
            let (_job_tx, job_rx) = mpsc::channel(10);
            let test_metrics = TestMetricsTx::new();

            let mut executor = Executor::new(
                engine.clone(),
                proxy_cache,
                job_rx,
                test_metrics.into_metrics_tx(),
            );

            // Add instance with matching code_id
            let serialized = load_precompiled_sample_component();
            let component = deserialize_component(&engine, &serialized);
            let linker = create_test_linker(&engine);
            let instance_pre = linker.instantiate_pre(&component).unwrap();
            let proxy_pre = ProxyPre::new(instance_pre).unwrap();

            executor.free_instances.push(MyInstance {
                code_id: "code-a".to_string(),
                pre: proxy_pre,
            });

            let result = executor.try_pop_free_instance("code-a");
            assert!(result.is_some(), "Should find and return instance");
            assert_eq!(result.unwrap().code_id, "code-a");
            assert_eq!(
                executor.free_instances.len(),
                0,
                "Should remove instance from pool"
            );
        }

        #[tokio::test]

        async fn test_try_pop_free_instance_multiple_same_code_id() {
            let engine = create_test_engine();
            let proxy_cache = MockAdaptCache::new();
            let (_job_tx, job_rx) = mpsc::channel(10);
            let test_metrics = TestMetricsTx::new();

            let mut executor = Executor::new(
                engine.clone(),
                proxy_cache,
                job_rx,
                test_metrics.into_metrics_tx(),
            );

            // Add multiple instances with same code_id
            let serialized = load_precompiled_sample_component();
            for _ in 0..3 {
                let component = deserialize_component(&engine, &serialized);
                let linker = create_test_linker(&engine);
                let instance_pre = linker.instantiate_pre(&component).unwrap();
                let proxy_pre = ProxyPre::new(instance_pre).unwrap();

                executor.free_instances.push(MyInstance {
                    code_id: "code-a".to_string(),
                    pre: proxy_pre,
                });
            }

            let result = executor.try_pop_free_instance("code-a");
            assert!(result.is_some(), "Should find instance");
            assert_eq!(executor.free_instances.len(), 2, "Should have 2 remaining");
        }

        #[tokio::test]

        async fn test_try_pop_free_instance_selects_correct_from_mixed() {
            let engine = create_test_engine();
            let proxy_cache = MockAdaptCache::new();
            let (_job_tx, job_rx) = mpsc::channel(10);
            let test_metrics = TestMetricsTx::new();

            let mut executor = Executor::new(
                engine.clone(),
                proxy_cache,
                job_rx,
                test_metrics.into_metrics_tx(),
            );

            // Add instances: code-a, code-b, code-c, code-b
            let serialized = load_precompiled_sample_component();
            for code_id in ["code-a", "code-b", "code-c", "code-b"] {
                let component = deserialize_component(&engine, &serialized);
                let linker = create_test_linker(&engine);
                let instance_pre = linker.instantiate_pre(&component).unwrap();
                let proxy_pre = ProxyPre::new(instance_pre).unwrap();

                executor.free_instances.push(MyInstance {
                    code_id: code_id.to_string(),
                    pre: proxy_pre,
                });
            }

            // Pop code-b (should get first code-b at index 1)
            let result = executor.try_pop_free_instance("code-b");
            assert!(result.is_some());
            assert_eq!(result.unwrap().code_id, "code-b");

            // Remaining should be: code-a, code-c, code-b
            assert_eq!(executor.free_instances.len(), 3);
            assert_eq!(executor.free_instances[0].code_id, "code-a");
            assert_eq!(executor.free_instances[1].code_id, "code-c");
            assert_eq!(executor.free_instances[2].code_id, "code-b");
        }
    }

    fn load_precompiled_sample_component() -> Vec<u8> {
        const WASM: &[u8] =
            include_bytes!("../../sample-wasi-http-rust/sample_wasi_http_rust.wasm");
        static CWASM: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        CWASM
            .get_or_init(|| Engine::default().precompile_component(WASM).unwrap())
            .to_vec()
    }

    // Helper function to deserialize a component from bytes
    fn deserialize_component(engine: &Engine, bytes: &[u8]) -> Component {
        unsafe { Component::deserialize(engine, bytes).expect("Failed to deserialize component") }
    }

    // ===== Integration Tests =====

    mod integration {
        use super::*;

        #[tokio::test]

        async fn test_pop_or_create_instance_creates_when_no_free() {
            let engine = create_test_engine();
            let cache = MockAdaptCache::new();
            let linker = create_test_linker(&engine);
            let test_metrics = TestMetricsTx::new();

            // Prepare cache with a component
            let serialized = load_precompiled_sample_component();
            cache.insert("code-a".to_string(), Bytes::from(serialized));

            let result = pop_or_create_instance(
                None,
                "code-a".to_string(),
                cache,
                engine,
                linker,
                test_metrics.clone().into_metrics_tx(),
            )
            .await;

            assert!(result.is_ok(), "Should successfully create instance");

            // Verify CreateInstance metric
            test_metrics.assert_contains(
                |m| matches!(m, Metrics::CreateInstance { code_id } if code_id == "code-a"),
            ).await;
        }

        #[tokio::test]

        async fn test_pop_or_create_instance_reuses_when_available() {
            let engine = create_test_engine();
            let cache = MockAdaptCache::new();
            let linker = create_test_linker(&engine);
            let test_metrics = TestMetricsTx::new();

            // Create an existing instance
            let serialized = load_precompiled_sample_component();
            let component = deserialize_component(&engine, &serialized);
            let instance_pre = linker.instantiate_pre(&component).unwrap();
            let proxy_pre = ProxyPre::new(instance_pre).unwrap();
            let existing = MyInstance {
                code_id: "code-a".to_string(),
                pre: proxy_pre,
            };

            let result = pop_or_create_instance(
                Some(existing),
                "code-a".to_string(),
                cache,
                engine,
                linker,
                test_metrics.clone().into_metrics_tx(),
            )
            .await;

            assert!(result.is_ok(), "Should successfully reuse instance");

            // Verify ReuseInstance metric
            test_metrics.assert_contains(
                |m| matches!(m, Metrics::ReuseInstance { code_id } if code_id == "code-a"),
            ).await;

            // Should NOT have CreateInstance metric
            assert_eq!(
                test_metrics.count(|m| matches!(m, Metrics::CreateInstance { .. })),
                0
            );
        }

        #[tokio::test]

        async fn test_pop_or_create_instance_fails_when_cache_fails() {
            let engine = create_test_engine();
            let cache = MockAdaptCache::new();
            cache.set_should_fail(true);
            let linker = create_test_linker(&engine);
            let test_metrics = TestMetricsTx::new();

            let result = pop_or_create_instance(
                None,
                "code-a".to_string(),
                cache,
                engine,
                linker,
                test_metrics.clone().into_metrics_tx(),
            )
            .await;

            assert!(result.is_err(), "Should fail when cache fails");

            // Verify ProxyCacheError metric
            test_metrics.assert_contains(
                |m| matches!(m, Metrics::ProxyCacheError { code_id, .. } if code_id == "code-a"),
            ).await;
        }

        #[tokio::test]

        async fn test_pop_or_create_instance_fails_when_not_found() {
            let engine = create_test_engine();
            let cache = MockAdaptCache::new();
            // Don't insert any component - should trigger NotFound
            let linker = create_test_linker(&engine);
            let test_metrics = TestMetricsTx::new();

            let result = pop_or_create_instance(
                None,
                "nonexistent".to_string(),
                cache,
                engine,
                linker,
                test_metrics.clone().into_metrics_tx(),
            )
            .await;

            assert!(result.is_err(), "Should fail when component not found");

            // Verify ProxyCacheError metric
            test_metrics.assert_contains(|m| matches!(m, Metrics::ProxyCacheError { code_id, .. } if code_id == "nonexistent")).await;
        }
    }
}
