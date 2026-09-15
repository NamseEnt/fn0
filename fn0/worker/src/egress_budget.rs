//! Worker-side egress credit backed by the control plane's monthly quota.
//!
//! Control charges a project's monthly usage when it grants credit, and the worker spends that
//! credit locally so most charges never leave the process. A grant request asks for at least the
//! bytes the current chunk needs; the requested size doubles while grants keep arriving in quick
//! succession, so a large stream does not pay one control round trip per megabyte, and falls back
//! to the minimum after a quiet period so an idle project does not strand a large grant.
//! Credit from an earlier UTC month is discarded. Any failure to reach control refuses the
//! charge: egress is fail-closed.

use crate::worker_pool::{self, RequestEnvelope};
use bytes::Bytes;
use dashmap::DashMap;
use fn0::{EgressBudget, EgressChargeFuture, EgressDenied};
use http_body_util::{BodyExt, Full};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc;

const MINIMUM_GRANT_BYTES: u64 = 256 * 1024;
const MAXIMUM_GRANT_BYTES: u64 = 64 * 1024 * 1024;
const GRANT_GROWTH_WINDOW: Duration = Duration::from_secs(10);
const EXHAUSTED_RECHECK_INTERVAL: Duration = Duration::from_secs(60);
const GRANT_DEADLINE: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub enum EgressGrantResponse {
    Granted { month: String, granted_bytes: u64 },
    QuotaExhausted { month: String },
    QuotaNotConfigured,
    Unauthorized,
    Error,
}

pub type EgressGrantFuture =
    Pin<Box<dyn Future<Output = anyhow::Result<EgressGrantResponse>> + Send + 'static>>;

pub trait EgressGrantSource: Send + Sync {
    fn request_grant(
        &self,
        project_id: &str,
        requested_bytes: u64,
        minimum_bytes: u64,
    ) -> EgressGrantFuture;
}

pub trait UtcMonthClock: Send + Sync {
    fn current_month(&self) -> String;
}

pub struct SystemUtcMonthClock;

impl UtcMonthClock for SystemUtcMonthClock {
    fn current_month(&self) -> String {
        chrono::Utc::now().format("%Y-%m").to_string()
    }
}

struct ProjectCredit {
    month: String,
    available_bytes: u64,
    next_grant_bytes: u64,
    last_grant_at: Option<tokio::time::Instant>,
}

struct ExhaustedMarker {
    month: String,
    recheck_at: tokio::time::Instant,
}

struct ProjectEgressState {
    credit: tokio::sync::Mutex<ProjectCredit>,
    exhausted: Mutex<Option<ExhaustedMarker>>,
}

pub struct ControlEgressBudget {
    control_project_id: String,
    grant_source: Arc<dyn EgressGrantSource>,
    month_clock: Arc<dyn UtcMonthClock>,
    projects: DashMap<String, Arc<ProjectEgressState>>,
}

impl ControlEgressBudget {
    pub fn new(
        control_project_id: String,
        grant_source: Arc<dyn EgressGrantSource>,
        month_clock: Arc<dyn UtcMonthClock>,
    ) -> Self {
        Self {
            control_project_id,
            grant_source,
            month_clock,
            projects: DashMap::new(),
        }
    }

    fn project_state(&self, project_id: &str) -> Arc<ProjectEgressState> {
        self.projects
            .entry(project_id.to_string())
            .or_insert_with(|| {
                Arc::new(ProjectEgressState {
                    credit: tokio::sync::Mutex::new(ProjectCredit {
                        month: String::new(),
                        available_bytes: 0,
                        next_grant_bytes: MINIMUM_GRANT_BYTES,
                        last_grant_at: None,
                    }),
                    exhausted: Mutex::new(None),
                })
            })
            .clone()
    }
}

impl EgressBudget for ControlEgressBudget {
    fn charge(&self, project_id: &str, byte_count: u64) -> EgressChargeFuture {
        if byte_count == 0 || project_id == self.control_project_id {
            return Box::pin(async { Ok(()) });
        }
        let state = self.project_state(project_id);
        let grant_source = self.grant_source.clone();
        let month_clock = self.month_clock.clone();
        let project_id = project_id.to_string();
        Box::pin(async move {
            let mut credit = state.credit.lock().await;
            let current_month = month_clock.current_month();
            if current_month > credit.month {
                credit.month = current_month;
                credit.available_bytes = 0;
            }
            if credit.available_bytes >= byte_count {
                credit.available_bytes -= byte_count;
                return Ok(());
            }
            let minimum_bytes = byte_count - credit.available_bytes;
            let now = tokio::time::Instant::now();
            credit.next_grant_bytes = match credit.last_grant_at {
                Some(last_grant_at) if now.duration_since(last_grant_at) < GRANT_GROWTH_WINDOW => {
                    credit
                        .next_grant_bytes
                        .saturating_mul(2)
                        .min(MAXIMUM_GRANT_BYTES)
                }
                _ => MINIMUM_GRANT_BYTES,
            };
            let requested_bytes = credit.next_grant_bytes.max(minimum_bytes);
            let response = tokio::time::timeout(
                GRANT_DEADLINE,
                grant_source.request_grant(&project_id, requested_bytes, minimum_bytes),
            )
            .await;
            let denied = match response {
                Ok(Ok(EgressGrantResponse::Granted {
                    month,
                    granted_bytes,
                })) if granted_bytes >= minimum_bytes => {
                    if month > credit.month {
                        credit.month = month;
                        credit.available_bytes = 0;
                    }
                    credit.available_bytes += granted_bytes - byte_count;
                    credit.last_grant_at = Some(tokio::time::Instant::now());
                    return Ok(());
                }
                Ok(Ok(EgressGrantResponse::Granted { .. })) => EgressDenied::BudgetUnavailable,
                Ok(Ok(EgressGrantResponse::QuotaExhausted { month })) => {
                    *state
                        .exhausted
                        .lock()
                        .expect("egress exhausted marker lock") = Some(ExhaustedMarker {
                        month,
                        recheck_at: tokio::time::Instant::now() + EXHAUSTED_RECHECK_INTERVAL,
                    });
                    EgressDenied::QuotaExhausted
                }
                Ok(Ok(EgressGrantResponse::QuotaNotConfigured)) => {
                    *state
                        .exhausted
                        .lock()
                        .expect("egress exhausted marker lock") = Some(ExhaustedMarker {
                        month: credit.month.clone(),
                        recheck_at: tokio::time::Instant::now() + EXHAUSTED_RECHECK_INTERVAL,
                    });
                    EgressDenied::QuotaNotConfigured
                }
                Ok(Ok(EgressGrantResponse::Unauthorized | EgressGrantResponse::Error))
                | Ok(Err(_))
                | Err(_) => EgressDenied::BudgetUnavailable,
            };
            tracing::warn!(%project_id, byte_count, ?denied, "egress charge refused");
            Err(denied)
        })
    }

    fn known_exhausted(&self, project_id: &str) -> bool {
        if project_id == self.control_project_id {
            return false;
        }
        let Some(state) = self.projects.get(project_id).map(|state| state.clone()) else {
            return false;
        };
        let exhausted = state
            .exhausted
            .lock()
            .expect("egress exhausted marker lock");
        exhausted.as_ref().is_some_and(|marker| {
            marker.month == self.month_clock.current_month()
                && tokio::time::Instant::now() < marker.recheck_at
        })
    }
}

pub struct ControlEgressGrantSource {
    control_project_id: String,
    worker_senders: OnceLock<Arc<Vec<mpsc::Sender<RequestEnvelope>>>>,
}

impl ControlEgressGrantSource {
    pub fn new(control_project_id: String) -> Self {
        Self {
            control_project_id,
            worker_senders: OnceLock::new(),
        }
    }

    pub fn set_worker_senders(&self, worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>) {
        if self.worker_senders.set(worker_senders).is_err() {
            panic!("egress grant source worker senders already set");
        }
    }
}

impl EgressGrantSource for ControlEgressGrantSource {
    fn request_grant(
        &self,
        project_id: &str,
        requested_bytes: u64,
        minimum_bytes: u64,
    ) -> EgressGrantFuture {
        let worker_senders = self.worker_senders.get().cloned();
        let control_project_id = self.control_project_id.clone();
        let project_id = project_id.to_string();
        Box::pin(async move {
            let Some(worker_senders) = worker_senders else {
                anyhow::bail!("egress grant source is not connected to worker threads");
            };
            let body = serde_json::to_vec(&serde_json::json!({
                "project_id": project_id,
                "requested_bytes": requested_bytes,
                "minimum_bytes": minimum_bytes,
            }))?;
            let request = hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri("https://fn0-control.internal/__forte_action/egress_grant")
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .header("x-fn0-internal-egress-grant", "true")
                .body(
                    Full::new(Bytes::from(body))
                        .map_err(|never: std::convert::Infallible| match never {})
                        .boxed_unsync(),
                )?;
            let response = worker_pool::invoke_and_wait(
                &worker_senders,
                |response_sender| {
                    RequestEnvelope::new(control_project_id, request, response_sender)
                },
                GRANT_DEADLINE,
                GRANT_DEADLINE,
            )
            .await?;
            if !response.status().is_success() {
                anyhow::bail!("egress grant returned status {}", response.status());
            }
            let body = response.into_body().collect().await?.to_bytes();
            Ok(serde_json::from_slice(&body)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct ScriptedGrantSource {
        responses: Mutex<VecDeque<anyhow::Result<EgressGrantResponse>>>,
        requests: Mutex<Vec<(String, u64, u64)>>,
    }

    impl ScriptedGrantSource {
        fn new(responses: Vec<anyhow::Result<EgressGrantResponse>>) -> Arc<Self> {
            Arc::new(Self {
                responses: Mutex::new(responses.into()),
                requests: Mutex::new(Vec::new()),
            })
        }
    }

    impl EgressGrantSource for ScriptedGrantSource {
        fn request_grant(
            &self,
            project_id: &str,
            requested_bytes: u64,
            minimum_bytes: u64,
        ) -> EgressGrantFuture {
            self.requests.lock().unwrap().push((
                project_id.to_string(),
                requested_bytes,
                minimum_bytes,
            ));
            let response = self
                .responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected grant request");
            Box::pin(async move { response })
        }
    }

    struct FixedMonthClock(Mutex<String>);

    impl UtcMonthClock for FixedMonthClock {
        fn current_month(&self) -> String {
            self.0.lock().unwrap().clone()
        }
    }

    fn september_clock() -> Arc<FixedMonthClock> {
        Arc::new(FixedMonthClock(Mutex::new("2026-09".to_string())))
    }

    fn granted(granted_bytes: u64) -> anyhow::Result<EgressGrantResponse> {
        Ok(EgressGrantResponse::Granted {
            month: "2026-09".to_string(),
            granted_bytes,
        })
    }

    #[tokio::test(start_paused = true)]
    async fn one_grant_covers_many_small_charges() {
        let grant_source = ScriptedGrantSource::new(vec![granted(MINIMUM_GRANT_BYTES)]);
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source.clone(),
            september_clock(),
        );
        for _ in 0..16 {
            budget.charge("project", 1024).await.unwrap();
        }
        assert_eq!(
            grant_source.requests.lock().unwrap().as_slice(),
            [("project".to_string(), MINIMUM_GRANT_BYTES, 1024)]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn large_chunk_asks_for_at_least_its_own_size() {
        let chunk_bytes = MINIMUM_GRANT_BYTES * 3;
        let grant_source = ScriptedGrantSource::new(vec![granted(chunk_bytes)]);
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source.clone(),
            september_clock(),
        );
        budget.charge("project", chunk_bytes).await.unwrap();
        assert_eq!(
            grant_source.requests.lock().unwrap()[0],
            ("project".to_string(), chunk_bytes, chunk_bytes)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn grant_size_grows_with_rapid_use_and_resets_after_quiet_period() {
        let grant_source = ScriptedGrantSource::new(vec![
            granted(MINIMUM_GRANT_BYTES),
            granted(MINIMUM_GRANT_BYTES * 2),
            granted(MINIMUM_GRANT_BYTES),
        ]);
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source.clone(),
            september_clock(),
        );
        budget.charge("project", MINIMUM_GRANT_BYTES).await.unwrap();
        budget.charge("project", 1).await.unwrap();
        budget
            .charge("project", MINIMUM_GRANT_BYTES * 2 - 1)
            .await
            .unwrap();
        tokio::time::advance(GRANT_GROWTH_WINDOW + Duration::from_secs(1)).await;
        budget.charge("project", 1).await.unwrap();
        let requested_sizes: Vec<u64> = grant_source
            .requests
            .lock()
            .unwrap()
            .iter()
            .map(|(_, requested_bytes, _)| *requested_bytes)
            .collect();
        assert_eq!(
            requested_sizes,
            [
                MINIMUM_GRANT_BYTES,
                MINIMUM_GRANT_BYTES * 2,
                MINIMUM_GRANT_BYTES
            ]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn exhausted_quota_refuses_and_is_remembered_until_recheck() {
        let grant_source = ScriptedGrantSource::new(vec![
            Ok(EgressGrantResponse::QuotaExhausted {
                month: "2026-09".to_string(),
            }),
            granted(MINIMUM_GRANT_BYTES),
        ]);
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source.clone(),
            september_clock(),
        );
        assert!(!budget.known_exhausted("project"));
        assert_eq!(
            budget.charge("project", 1).await,
            Err(EgressDenied::QuotaExhausted)
        );
        assert!(budget.known_exhausted("project"));
        assert!(!budget.known_exhausted("other-project"));
        tokio::time::advance(EXHAUSTED_RECHECK_INTERVAL).await;
        assert!(!budget.known_exhausted("project"));
        budget.charge("project", 1).await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn missing_quota_document_refuses() {
        let grant_source =
            ScriptedGrantSource::new(vec![Ok(EgressGrantResponse::QuotaNotConfigured)]);
        let budget =
            ControlEgressBudget::new("fn0-control".to_string(), grant_source, september_clock());
        assert_eq!(
            budget.charge("project", 1).await,
            Err(EgressDenied::QuotaNotConfigured)
        );
        assert!(budget.known_exhausted("project"));
    }

    #[tokio::test(start_paused = true)]
    async fn unreachable_control_fails_closed_without_marking_exhausted() {
        let grant_source = ScriptedGrantSource::new(vec![
            Err(anyhow::anyhow!("worker queue full")),
            Ok(EgressGrantResponse::Error),
        ]);
        let budget =
            ControlEgressBudget::new("fn0-control".to_string(), grant_source, september_clock());
        assert_eq!(
            budget.charge("project", 1).await,
            Err(EgressDenied::BudgetUnavailable)
        );
        assert_eq!(
            budget.charge("project", 1).await,
            Err(EgressDenied::BudgetUnavailable)
        );
        assert!(!budget.known_exhausted("project"));
    }

    struct HangingGrantSource;

    impl EgressGrantSource for HangingGrantSource {
        fn request_grant(&self, _: &str, _: u64, _: u64) -> EgressGrantFuture {
            Box::pin(std::future::pending())
        }
    }

    #[tokio::test(start_paused = true)]
    async fn lost_grant_response_fails_closed_at_deadline() {
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            Arc::new(HangingGrantSource),
            september_clock(),
        );
        let started_at = tokio::time::Instant::now();
        assert_eq!(
            budget.charge("project", 1).await,
            Err(EgressDenied::BudgetUnavailable)
        );
        assert_eq!(started_at.elapsed(), GRANT_DEADLINE);
    }

    #[tokio::test(start_paused = true)]
    async fn credit_from_previous_month_is_discarded() {
        let grant_source = ScriptedGrantSource::new(vec![
            granted(MINIMUM_GRANT_BYTES),
            Ok(EgressGrantResponse::Granted {
                month: "2026-10".to_string(),
                granted_bytes: MINIMUM_GRANT_BYTES,
            }),
        ]);
        let month_clock = september_clock();
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source.clone(),
            month_clock.clone(),
        );
        budget.charge("project", 1).await.unwrap();
        *month_clock.0.lock().unwrap() = "2026-10".to_string();
        budget.charge("project", 1).await.unwrap();
        assert_eq!(grant_source.requests.lock().unwrap().len(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn clock_skew_at_month_boundary_keeps_the_granted_credit() {
        let grant_source = ScriptedGrantSource::new(vec![Ok(EgressGrantResponse::Granted {
            month: "2026-10".to_string(),
            granted_bytes: MINIMUM_GRANT_BYTES,
        })]);
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source.clone(),
            september_clock(),
        );
        budget.charge("project", 1).await.unwrap();
        budget.charge("project", 1).await.unwrap();
        assert_eq!(grant_source.requests.lock().unwrap().len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn control_project_is_never_charged() {
        let grant_source = ScriptedGrantSource::new(Vec::new());
        let budget = ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source.clone(),
            september_clock(),
        );
        budget.charge("fn0-control", 1 << 30).await.unwrap();
        assert!(grant_source.requests.lock().unwrap().is_empty());
    }

    struct CountingGrantSource {
        request_count: AtomicUsize,
        remaining_bytes: Mutex<u64>,
    }

    impl EgressGrantSource for CountingGrantSource {
        fn request_grant(
            &self,
            _project_id: &str,
            requested_bytes: u64,
            minimum_bytes: u64,
        ) -> EgressGrantFuture {
            self.request_count.fetch_add(1, Ordering::AcqRel);
            let mut remaining_bytes = self.remaining_bytes.lock().unwrap();
            let response = if *remaining_bytes < minimum_bytes || *remaining_bytes == 0 {
                EgressGrantResponse::QuotaExhausted {
                    month: "2026-09".to_string(),
                }
            } else {
                let granted_bytes = requested_bytes.min(*remaining_bytes);
                *remaining_bytes -= granted_bytes;
                EgressGrantResponse::Granted {
                    month: "2026-09".to_string(),
                    granted_bytes,
                }
            };
            Box::pin(async move { Ok(response) })
        }
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_connections_cannot_exceed_the_project_budget() {
        let project_limit = MINIMUM_GRANT_BYTES * 4;
        let grant_source = Arc::new(CountingGrantSource {
            request_count: AtomicUsize::new(0),
            remaining_bytes: Mutex::new(project_limit),
        });
        let budget = Arc::new(ControlEgressBudget::new(
            "fn0-control".to_string(),
            grant_source,
            september_clock(),
        ));
        let charge_tasks: Vec<_> = (0..64)
            .map(|_| {
                let budget = budget.clone();
                tokio::spawn(async move { budget.charge("project", 64 * 1024).await })
            })
            .collect();
        let mut accepted_bytes = 0;
        for charge_task in charge_tasks {
            if charge_task.await.unwrap().is_ok() {
                accepted_bytes += 64 * 1024;
            }
        }
        assert_eq!(accepted_bytes, project_limit);
    }
}
