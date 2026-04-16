use crate::{Request, Response, telemetry};
use adapt_cache::AdaptCache;
use anyhow::{Result, anyhow};
use http_body_util::BodyExt;
use crate::measure_cpu_time::{Clock, TimeTracker, measure_cpu_time};
use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::{mpsc::Sender, oneshot};
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
};

pub type EnvVars = Arc<RwLock<std::collections::HashMap<String, Vec<(String, String)>>>>;

pub struct Job {
    pub req: Request,
    pub res_tx: oneshot::Sender<Result<Response>>,
    pub code_id: String,
}

pub struct WasmExecutor {
    job_tx: Sender<Job>,
    env_vars: EnvVars,
}

impl WasmExecutor {
    pub fn new<A, C>(proxy_cache: A, clock: C, env_vars: EnvVars) -> Self
    where
        A: AdaptCache<ProxyPre<ClientState<C>>, wasmtime::Error>,
        C: Clock,
    {
        let (job_tx, mut job_rx) = tokio::sync::mpsc::channel(10 * 1024);
        let engine = Engine::new(&engine_config()).unwrap();

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker).unwrap();
        wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker).unwrap();

        tokio::spawn({
            let proxy_cache = proxy_cache.clone();
            let engine = engine.clone();
            let linker = linker.clone();
            let clock = clock.clone();
            let env_vars = env_vars.clone();

            async move {
                let mut interval = tokio::time::interval(Duration::from_millis(3));
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            engine.increment_epoch();
                        }

                        res = job_rx.recv() => {
                            match res {
                                Some(job) => {
                                    let proxy_cache = proxy_cache.clone();
                                    let engine = engine.clone();
                                    let linker = linker.clone();
                                    let clock = clock.clone();
                                    let env_vars = env_vars.clone();

                                    tokio::spawn(async move {
                                        run_job(job, proxy_cache, engine, linker, clock, env_vars).await;
                                    });
                                },
                                None => break,
                            }
                        }
                    }
                }
            }
        });

        Self { job_tx, env_vars }
    }

    pub fn set_env(&self, code_id: &str, new_vars: Vec<(String, String)>) {
        if let Ok(mut env) = self.env_vars.write() {
            env.insert(code_id.to_string(), new_vars);
        }
    }

    pub fn clear_env(&self, code_id: &str) {
        if let Ok(mut env) = self.env_vars.write() {
            env.remove(code_id);
        }
    }

    pub(crate) async fn run(&self, code_id: &str, request: Request) -> Result<Response> {
        let (res_tx, res_rx) = oneshot::channel();
        let job = Job {
            req: request,
            res_tx,
            code_id: code_id.to_string(),
        };

        self.job_tx
            .send(job)
            .await
            .map_err(|_| anyhow!("job_tx closed"))?;

        res_rx.await?
    }
}

pub use fn0_wasmtime::engine_config;

async fn run_job<A, C>(
    job: Job,
    proxy_cache: A,
    engine: Engine,
    linker: Linker<ClientState<C>>,
    clock: C,
    env_vars: EnvVars,
) where
    A: AdaptCache<ProxyPre<ClientState<C>>, wasmtime::Error>,
    C: Clock,
{
    let proxy_pre = match get_proxy_pre(job.code_id.clone(), proxy_cache, engine, linker).await {
        Ok(x) => x,
        Err(error) => {
            let _ = job.res_tx.send(Err(anyhow!("Failed to get proxy pre: {error:?}")));
            return;
        }
    };

    let result = handle_request(proxy_pre, job.req, job.code_id, clock, env_vars).await;

    let _ = job.res_tx.send(result);
}

async fn get_proxy_pre<A, C>(
    code_id: String,
    proxy_cache: A,
    engine: Engine,
    linker: Linker<ClientState<C>>,
) -> Result<ProxyPre<ClientState<C>>, adapt_cache::Error<wasmtime::Error>>
where
    A: AdaptCache<ProxyPre<ClientState<C>>, wasmtime::Error>,
    C: Clock,
{
    match proxy_cache
        .get(&code_id.clone(), |bytes| {
            let component = unsafe { Component::deserialize(&engine, &bytes)? };
            let instance_pre = linker.instantiate_pre(&component)?;
            let proxy_pre = ProxyPre::new(instance_pre)?;

            telemetry::create_instance(&code_id);
            Ok((proxy_pre, bytes.len()))
        })
        .await
    {
        Ok(proxy_pre) => Ok(proxy_pre),
        Err(error) => {
            telemetry::proxy_cache_error(&code_id, &format!("{error:?}"));
            Err(error)
        }
    }
}

async fn handle_request<C>(
    pre: ProxyPre<ClientState<C>>,
    req: Request,
    code_id: String,
    clock: C,
    env_vars: EnvVars,
) -> Result<Response>
where
    C: Clock + Send + 'static,
{
    let time_tracker = TimeTracker::new(clock);
    let is_timeout = Arc::new(AtomicBool::new(false));

    let wasi = {
        let mut builder = WasiCtx::builder();
        builder.inherit_stdio();
        let subdomain = code_id.split("::").next().unwrap_or(&code_id);
        if let Ok(map) = env_vars.read() {
            if let Some(vars) = map.get(subdomain) {
                for (key, value) in vars.iter() {
                    builder.env(key, value);
                }
            }
        }
        builder.build()
    };

    let mut store = Store::new(
        pre.engine(),
        ClientState {
            table: ResourceTable::new(),
            wasi,
            http: WasiHttpCtx::new(),
            time_tracker: time_tracker.clone(),
            code_id: code_id.clone(),
            is_timeout: is_timeout.clone(),
        },
    );
    store.epoch_deadline_trap();
    store.set_epoch_deadline(1);
    store.epoch_deadline_async_yield_and_update(1);
    store.epoch_deadline_callback({
        |context| {
            let state = context.data();
            let cpu_time = state.time_tracker.duration();
            if cpu_time > Duration::from_millis(1000) {
                telemetry::cpu_timeout(&state.code_id, cpu_time);
                state.is_timeout.store(true, Ordering::Relaxed);
                return Ok(wasmtime::UpdateDeadline::Interrupt);
            }
            Ok(wasmtime::UpdateDeadline::Continue(1))
        }
    });

    let (tx, rx) = tokio::sync::oneshot::channel();
    let req: wasmtime::component::Resource<wasmtime_wasi_http::types::HostIncomingRequest> =
        match store.data_mut().new_incoming_request(
            Scheme::Http,
            req.map(|body| {
                body.map_err(|err| ErrorCode::InternalError(Some(err.to_string())))
                    .boxed_unsync()
            }),
        ) {
            Ok(x) => x,
            Err(error) => {
                telemetry::wasmtime_error("new_incoming_request", &code_id, &format!("{error:?}"));
                return Err(anyhow!("new_incoming_request failed: {error:?}"));
            }
        };
    let out = match store.data_mut().new_response_outparam(tx) {
        Ok(x) => x,
        Err(error) => {
            telemetry::wasmtime_error("new_response_outparam", &code_id, &format!("{error:?}"));
            return Err(anyhow!("new_response_outparam failed: {error:?}"));
        }
    };

    let proxy = match pre.instantiate_async(&mut store).await {
        Ok(x) => x,
        Err(error) => {
            telemetry::wasmtime_error("instantiate_async", &code_id, &format!("{error:?}"));
            return Err(anyhow!("instantiate_async failed: {error:?}"));
        }
    };

    let task = tokio::task::spawn({
        let code_id = code_id.clone();
        async move {
            let result = measure_cpu_time(
                time_tracker.clone(),
                proxy
                    .wasi_http_incoming_handler()
                    .call_handle(store, req, out),
            )
            .await;

            telemetry::cpu_time(&code_id, time_tracker.duration());

            result
        }
    });

    let result = rx.await;

    if let Err(_oneshot_recv_err) = result {
        let result = task.await;
        if let Err(error) = result {
            telemetry::request_task_join_error(&code_id, &format!("{error:?}"));
            return Err(anyhow!("request task join error: {error:?}"));
        }
        let result = result.unwrap();

        if let Err(error) = result {
            match error.downcast::<wasmtime::Trap>() {
                Ok(trap) => {
                    telemetry::trapped(&code_id, &format!("{trap:?}"));
                    if is_timeout.load(Ordering::Relaxed) {
                        return Err(anyhow!("CPU time limit exceeded (trapped: {trap:?})"));
                    }
                    return Err(anyhow!("trapped: {trap:?}"));
                }
                Err(error) => {
                    telemetry::canceled_unexpectedly(&code_id, &format!("{error:?}"));
                    if is_timeout.load(Ordering::Relaxed) {
                        return Err(anyhow!("CPU time limit exceeded: {error:?}"));
                    }
                    return Err(anyhow!("canceled unexpectedly: {error:?}"));
                }
            }
        }

        if is_timeout.load(Ordering::Relaxed) {
            return Err(anyhow!("CPU time limit exceeded"));
        }

        return Err(anyhow!("no response received"));
    }

    let result = result.unwrap();

    if let Ok(response) = result {
        return Ok(response.map(|body| {
            body.map_err(|error_code| anyhow!("error_code: {error_code:?}"))
                .boxed_unsync()
        }));
    }

    let error_code: ErrorCode = result.unwrap_err();

    telemetry::proxy_returns_error_code(&code_id, &format!("{error_code:?}"));
    Err(anyhow!("proxy returned error code: {error_code:?}"))
}


pub struct ClientState<C: Clock> {
    wasi: WasiCtx,
    http: WasiHttpCtx,
    table: ResourceTable,
    time_tracker: TimeTracker<C>,
    code_id: String,
    is_timeout: Arc<AtomicBool>,
}

impl<C: Clock> WasiView for ClientState<C> {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl<C: Clock> WasiHttpView for ClientState<C> {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
