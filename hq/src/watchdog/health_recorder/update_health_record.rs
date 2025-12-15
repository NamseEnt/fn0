use super::*;
use crate::watchdog::{
    host_infra::{HostHealthKind, HostHealthResponse, HostInfo},
    Context,
};

pub fn update_single_health_record(
    context: &Context,
    record: &mut HealthRecord,
    _host_info: &HostInfo,
    health_response: Option<HostHealthResponse>,
) -> color_eyre::Result<HealthAction> {
    let &Context {
        start_time,
        max_graceful_shutdown_wait_time,
        max_healthy_check_retrials,
        max_start_timeout,
        ..
    } = context;

    match health_response {
        Some(host_health_response) => match host_health_response.kind {
            HostHealthKind::Good => match record.state {
                HealthState::GracefulShuttingDown => {}
                _ => {
                    record.state = HealthState::Healthy {
                        ip: host_health_response.ip,
                    };
                    record.state_transited_at = start_time;
                }
            },
            HostHealthKind::GracefulShuttingDown => match record.state {
                HealthState::Starting
                | HealthState::Healthy { .. }
                | HealthState::RetryingCheck { .. }
                | HealthState::MarkedForTermination
                | HealthState::InvisibleOnInfra => {
                    record.state = HealthState::GracefulShuttingDown;
                    record.state_transited_at = start_time;
                }
                HealthState::TerminatedConfirm => {
                    eprintln!("Unexpected state: Terminated confirmed but graceful shutting down");
                    record.state = HealthState::GracefulShuttingDown;
                    record.state_transited_at = start_time;
                }
                HealthState::GracefulShuttingDown => {}
            },
        },
        None => match &mut record.state {
            HealthState::Starting => {
                if record.state_transited_at + max_start_timeout < start_time {
                    record.state = HealthState::MarkedForTermination;
                    record.state_transited_at = start_time;
                }
            }
            HealthState::Healthy { .. } | HealthState::InvisibleOnInfra => {
                record.state = HealthState::RetryingCheck { retrials: 1 };
                record.state_transited_at = start_time;
            }
            HealthState::RetryingCheck { retrials } => {
                *retrials += 1;
                if *retrials > max_healthy_check_retrials {
                    record.state = HealthState::MarkedForTermination;
                    record.state_transited_at = start_time;
                }
            }
            HealthState::MarkedForTermination
            | HealthState::GracefulShuttingDown
            | HealthState::TerminatedConfirm => {}
        },
    }

    if let HealthState::GracefulShuttingDown = record.state {
        if record.state_transited_at + max_graceful_shutdown_wait_time < start_time {
            record.state = HealthState::MarkedForTermination;
            record.state_transited_at = start_time;
        }
    }

    if matches!(record.state, HealthState::MarkedForTermination) {
        Ok(HealthAction::Terminate)
    } else {
        Ok(HealthAction::None)
    }
}
