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
    pub code_id: String,
    pub req: Request,
    pub resp_tx: oneshot::Sender<Result<Response>>,
}

pub enum DispatchError {
    Full,
    Closed,
}

const QUEUE_CAPACITY: usize = 256;

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
    let idx = pick_worker(&env.code_id, senders.len());
    match senders[idx].try_send(env) {
        Ok(()) => Ok(()),
        Err(mpsc::error::TrySendError::Full(_)) => Err(DispatchError::Full),
        Err(mpsc::error::TrySendError::Closed(_)) => Err(DispatchError::Closed),
    }
}

fn pick_worker(code_id: &str, n: usize) -> usize {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(code_id.as_bytes());
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
        while let Some(env) = rx.recv().await {
            let executor = executor.clone();
            tokio::task::spawn_local(async move {
                let RequestEnvelope {
                    code_id,
                    req,
                    resp_tx,
                } = env;
                let outcome =
                    AssertUnwindSafe(executor.run(&code_id, "/", req, None))
                        .catch_unwind()
                        .await;
                match outcome {
                    Ok(result) => {
                        let _ = resp_tx.send(result);
                    }
                    Err(panic) => {
                        let panic_msg = panic_payload_string(&panic);
                        tracing::error!(
                            %code_id,
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
