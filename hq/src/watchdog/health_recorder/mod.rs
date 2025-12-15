mod update_health_record;

use crate::watchdog::{host_infra::{HostHealthResponse, HostInfo, HostInstanceState}, Context, HostId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::IpAddr, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecord {
    pub state: HealthState,
    pub state_transited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthState {
    Starting,
    Healthy { ip: IpAddr },
    RetryingCheck { retrials: usize },
    MarkedForTermination,
    GracefulShuttingDown,
    TerminatedConfirm,
    InvisibleOnInfra,
}

pub type HealthRecords = BTreeMap<HostId, HealthRecord>;

pub enum HealthAction {
    None,
    Terminate,
}

pub struct InMemoryHealthRecorder {
    records: Arc<RwLock<BTreeMap<HostId, HealthRecord>>>,
}

impl InMemoryHealthRecorder {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub async fn update_single(
        &self,
        context: &Context,
        host_id: &HostId,
        host_info: &HostInfo,
        health_response: Option<HostHealthResponse>,
    ) -> color_eyre::Result<HealthAction> {
        let mut records = self.records.write().await;

        let record = records.entry(host_id.clone()).or_insert_with(|| {
            let state = match host_info.instance_state {
                HostInstanceState::Starting => HealthState::Starting,
                HostInstanceState::Running => match &health_response {
                    Some(hr) => match hr.kind {
                        crate::watchdog::host_infra::HostHealthKind::Good => {
                            HealthState::Healthy { ip: hr.ip }
                        }
                        crate::watchdog::host_infra::HostHealthKind::GracefulShuttingDown => {
                            HealthState::GracefulShuttingDown
                        }
                    },
                    None => HealthState::RetryingCheck { retrials: 1 },
                },
                HostInstanceState::Terminating => HealthState::TerminatedConfirm,
            };

            HealthRecord {
                state,
                state_transited_at: context.start_time,
            }
        });

        if matches!(host_info.instance_state, HostInstanceState::Terminating)
            && !matches!(record.state, HealthState::TerminatedConfirm)
        {
            record.state = HealthState::TerminatedConfirm;
            record.state_transited_at = context.start_time;
            return Ok(HealthAction::None);
        }

        update_health_record::update_single_health_record(context, record, host_info, health_response)
    }

    pub async fn mark_as_invisible(&self, context: &Context, host_id: &HostId) {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(host_id) {
            match record.state {
                HealthState::Starting
                | HealthState::Healthy { .. }
                | HealthState::RetryingCheck { .. }
                | HealthState::GracefulShuttingDown => {
                    record.state = HealthState::InvisibleOnInfra;
                    record.state_transited_at = context.start_time;
                }
                _ => {}
            }
        }
    }

    pub async fn get_healthy_ips(&self) -> Vec<IpAddr> {
        let records = self.records.read().await;
        records
            .values()
            .filter_map(|r| match r.state {
                HealthState::Healthy { ip } => Some(ip),
                _ => None,
            })
            .collect()
    }

    pub async fn get_all_records(&self) -> BTreeMap<HostId, HealthRecord> {
        self.records.read().await.clone()
    }

    pub async fn get_starting_count(&self) -> usize {
        let records = self.records.read().await;
        records
            .values()
            .filter(|r| matches!(r.state, HealthState::Starting))
            .count()
    }

    pub async fn cleanup_old_records(&self, current_time: DateTime<Utc>) {
        let mut records = self.records.write().await;
        records.retain(|_, record| {
            !matches!(
                record.state,
                HealthState::MarkedForTermination
                    | HealthState::TerminatedConfirm
                    | HealthState::InvisibleOnInfra
            ) || current_time - record.state_transited_at < Duration::minutes(5)
        });
    }
}
