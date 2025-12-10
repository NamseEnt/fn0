pub mod oci_lambda_proxy;

use crate::WorkerId;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::{
    collections::BTreeMap,
    future::Future,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    str::FromStr,
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub id: WorkerId,
    pub instance_created: DateTime<Utc>,
    pub ip: Option<IpAddr>,
    pub instance_state: WorkerInstanceState,
}

#[derive(Debug, Clone)]
pub enum WorkerInstanceState {
    Starting,
    Running,
    Terminating,
}

pub type WorkerInfos = Vec<WorkerInfo>;
pub type WorkerHealthResponseMap = BTreeMap<WorkerId, (WorkerInfo, Option<WorkerHealthResponse>)>;

pub trait WorkerInfra: Send + Sync {
    fn get_worker_infos<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<WorkerInfos>> + 'a + Send>>;

    fn terminate<'a>(
        &'a self,
        worker_id: &'a WorkerId,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>>;

    fn launch_instances<'a>(
        &'a self,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>>;
}

impl dyn WorkerInfra {
    pub async fn get_worker_health_responses(
        &self,
        domain: &str,
    ) -> color_eyre::Result<WorkerHealthResponseMap> {
        let workers_infos = self.get_worker_infos().await?;
        println!("workers_infos: {workers_infos:?}");
        Ok(futures::stream::iter(workers_infos)
            .map(|worker_info| async move {
                let Some(ip) = worker_info.ip else {
                    return (worker_info.id.clone(), (worker_info, None));
                };
                let addr = SocketAddr::new(ip, 443);
                let Ok(res) = reqwest::Client::builder()
                    .resolve(&format!("a.{domain}"), addr)
                    .timeout(Duration::from_secs(2))
                    .build()
                    .unwrap()
                    .get(format!("https://a.{domain}/health"))
                    .send()
                    .await
                else {
                    return (worker_info.id.clone(), (worker_info, None));
                };

                if !res.status().is_success() {
                    return (worker_info.id.clone(), (worker_info, None));
                }

                let Ok(body) = res.text().await else {
                    return (worker_info.id.clone(), (worker_info, None));
                };

                let Ok(kind) = body.parse::<WorkerHealthKind>() else {
                    panic!("Failed to parse health response: {body}");
                };

                (
                    worker_info.id.clone(),
                    (worker_info, Some(WorkerHealthResponse { kind, ip })),
                )
            })
            .buffer_unordered(32)
            .collect()
            .await)
    }

    pub async fn send_terminate_workers(&self, worker_ids: impl IntoIterator<Item = WorkerId>) {
        futures::stream::iter(worker_ids)
            .for_each_concurrent(16, |worker_id| async move {
                if let Err(e) = self.terminate(&worker_id).await {
                    println!("Failed to terminate worker {worker_id:?}: {e}");
                }
            })
            .await
    }
}

#[derive(Debug, Clone)]
pub struct WorkerHealthResponse {
    pub kind: WorkerHealthKind,
    pub ip: IpAddr,
}

#[derive(Debug, Clone)]
pub enum WorkerHealthKind {
    Good,
    GracefulShuttingDown,
}

impl FromStr for WorkerHealthKind {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "good" => Ok(WorkerHealthKind::Good),
            "graceful_shutting_down" => Ok(WorkerHealthKind::GracefulShuttingDown),
            _ => color_eyre::eyre::bail!("invalid health response: {}", s),
        }
    }
}
