use crate::turso_queue::TursoQueue;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct QueuePoller {
    store: TursoQueue,
    poll_interval: Duration,
}

impl QueuePoller {
    pub fn new(turso_url: &str, auth_token: &str) -> Self {
        Self {
            store: TursoQueue::new(turso_url, auth_token),
            poll_interval: Duration::from_secs(1),
        }
    }

    pub fn store(&self) -> &TursoQueue {
        &self.store
    }

    pub async fn run<F>(&self, execute_fn: F, shutdown: CancellationToken)
    where
        F: Fn(String, String) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>,
    {
        if let Err(e) = self.store.create_tables().await {
            tracing::error!("Failed to create queue tables: {e}");
        }

        loop {
            if shutdown.is_cancelled() {
                break;
            }

            match self.store.claim_task().await {
                Ok(Some(task)) => {
                    let result =
                        (execute_fn)(task.task_name.clone(), task.payload.clone()).await;

                    match result {
                        Ok(()) => {
                            if let Err(e) = self.store.delete_task(&task.id).await {
                                tracing::error!("Failed to delete completed task {}: {e}", task.id);
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("{e:?}");
                            if task.retry_count + 1 >= task.max_retries {
                                if let Err(e) =
                                    self.store.move_to_dead_queue(&task, &error_msg).await
                                {
                                    tracing::error!(
                                        "Failed to move task {} to dead queue: {e}",
                                        task.id
                                    );
                                }
                            } else {
                                if let Err(e) = self.store.retry_task(&task.id).await {
                                    tracing::error!(
                                        "Failed to retry task {}: {e}",
                                        task.id
                                    );
                                }
                            }
                        }
                    }
                }
                Ok(None) => {
                    tokio::select! {
                        _ = tokio::time::sleep(self.poll_interval) => {}
                        _ = shutdown.cancelled() => { break; }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to claim task: {e}");
                    tokio::select! {
                        _ = tokio::time::sleep(self.poll_interval) => {}
                        _ = shutdown.cancelled() => { break; }
                    }
                }
            }
        }
    }
}
