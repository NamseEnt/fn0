use anyhow::Result;
use bytes::Bytes;
use fn0::cache::BundleCache;
use fn0::{CodeExecutor, ExecutionContext, panic_payload_string};
use futures::FutureExt;
use http_body_util::combinators::UnsyncBoxBody;
use std::hash::Hasher;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tokio::sync::{mpsc, oneshot};

pub type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;

pub struct RequestEnvelope {
    pub project_id: String,
    pub req: Request,
    pub resp_tx: oneshot::Sender<Result<Response>>,
    pub enqueued_at: std::time::Instant,
}

pub enum DispatchError {
    Full,
    Closed,
}

const QUEUE_CAPACITY: usize = 256;
const SWEEP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

pub fn spawn_workers<C>(
    ctx: Arc<ExecutionContext<C>>,
    num_threads: usize,
) -> Vec<mpsc::Sender<RequestEnvelope>>
where
    C: BundleCache,
{
    assert!(num_threads > 0, "worker pool must have at least one thread");
    let mut senders = Vec::with_capacity(num_threads);

    for idx in 0..num_threads {
        let (tx, rx) = mpsc::channel::<RequestEnvelope>(QUEUE_CAPACITY);
        senders.push(tx);
        let ctx = ctx.clone();
        thread::Builder::new()
            .name(format!("fn0-worker-{idx}"))
            .spawn(move || run_worker(idx, ctx, rx))
            .expect("failed to spawn worker thread");
    }

    senders
}

pub fn dispatch(
    senders: &[mpsc::Sender<RequestEnvelope>],
    env: RequestEnvelope,
) -> Result<(), DispatchError> {
    let idx = pick_worker(&env.project_id, senders.len());
    match senders[idx].try_send(env) {
        Ok(()) => Ok(()),
        Err(mpsc::error::TrySendError::Full(_)) => Err(DispatchError::Full),
        Err(mpsc::error::TrySendError::Closed(_)) => Err(DispatchError::Closed),
    }
}

fn pick_worker(project_id: &str, n: usize) -> usize {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(project_id.as_bytes());
    (hasher.finish() as usize) % n
}

fn run_worker<C>(idx: usize, ctx: Arc<ExecutionContext<C>>, mut rx: mpsc::Receiver<RequestEnvelope>)
where
    C: BundleCache,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(format!("fn0-worker-{idx}"))
        .build()
        .expect("failed to build current_thread runtime");
    let local = tokio::task::LocalSet::new();

    rt.block_on(local.run_until(async move {
        let executor = Rc::new(CodeExecutor::new(ctx));

        let sweep_executor = executor.clone();
        tokio::task::spawn_local(async move {
            let mut interval = tokio::time::interval(SWEEP_INTERVAL);
            interval.tick().await;
            loop {
                interval.tick().await;
                sweep_executor.sweep_unregistered().await;
            }
        });

        while let Some(env) = rx.recv().await {
            fn0::telemetry::stage_duration("queue_wait", env.enqueued_at.elapsed());
            let executor = executor.clone();
            tokio::task::spawn_local(async move {
                let RequestEnvelope {
                    project_id,
                    req,
                    resp_tx,
                    enqueued_at: _,
                } = env;
                let outcome =
                    AssertUnwindSafe(executor.run(&project_id, "/", req, None))
                        .catch_unwind()
                        .await;
                match outcome {
                    Ok(result) => {
                        if resp_tx.send(result).is_err() {
                            fn0::telemetry::oneshot_drop_before_response();
                        }
                    }
                    Err(panic) => {
                        let panic_msg = panic_payload_string(&panic);
                        fn0::telemetry::panicked();
                        tracing::error!(
                            %project_id,
                            panic = %panic_msg,
                            "executor panicked; response channel dropped"
                        );
                    }
                }
            });
        }
    }));

    tracing::info!(worker = idx, "worker thread exiting");
}

pub fn default_num_threads() -> usize {
    if let Ok(s) = std::env::var("FN0_WORKER_THREADS")
        && let Ok(n) = s.parse::<usize>()
        && n > 0
    {
        return n;
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
