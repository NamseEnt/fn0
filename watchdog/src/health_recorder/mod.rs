pub mod s3;

use crate::{
    Context, WorkerId,
    worker_infra::{WorkerHealthResponse, WorkerHealthResponseMap},
};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, future::Future, pin::Pin};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecord {
    pub state: HealthState,
    pub state_transited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthState {
    Healthy,
    RetryingCheck { retrials: usize },
    MarkedForTermination,
    GracefulShuttingDown,
    TerminatedConfirm,
    InvisibleOnInfra,
}

pub type HealthRecords = BTreeMap<WorkerId, HealthRecord>;

pub trait HealthRecorder: Send + Sync {
    fn read_all<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<HealthRecords>> + 'a + Send>>;
    fn write_all<'a>(
        &'a self,
        records: HealthRecords,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a + Send>>;
}

pub fn update_health_records(
    context: &Context,
    health_records: &mut HealthRecords,
    worker_health_response_map: WorkerHealthResponseMap,
) -> anyhow::Result<()> {
    let &Context {
        start_time,
        max_graceful_shutdown_wait_time,
        max_healthy_check_retrials,
        ..
    } = context;

    let worker_health_response_not_in_records = worker_health_response_map
        .iter()
        .filter(|(id, _)| !health_records.contains_key(id));

    let mut new_worker_records = vec![];

    for (worker_id, (_worker_info, worker_status)) in worker_health_response_not_in_records {
        new_worker_records.push((
            worker_id.clone(),
            HealthRecord {
                state: match worker_status {
                    Some(worker_status) => match worker_status {
                        WorkerHealthResponse::Good => HealthState::Healthy,
                        WorkerHealthResponse::GracefulShuttingDown => {
                            HealthState::GracefulShuttingDown
                        }
                    },
                    None => HealthState::RetryingCheck { retrials: 1 },
                },
                state_transited_at: start_time,
            },
        ));
    }

    health_records.retain(|worker_id, record| {
        let Some((_worker_info, health_response)) = worker_health_response_map.get(worker_id)
        else {
            match record.state {
                HealthState::Healthy
                | HealthState::RetryingCheck { .. }
                | HealthState::GracefulShuttingDown => {
                    record.state = HealthState::InvisibleOnInfra;
                    record.state_transited_at = start_time;
                    return true;
                }
                HealthState::MarkedForTermination
                | HealthState::TerminatedConfirm
                | HealthState::InvisibleOnInfra => {
                    return start_time - record.state_transited_at < TimeDelta::minutes(5);
                }
            }
        };

        match health_response {
            Some(worker_status) => match worker_status {
                WorkerHealthResponse::Good => {
                    record.state = HealthState::Healthy;
                    record.state_transited_at = start_time;
                }
                WorkerHealthResponse::GracefulShuttingDown => match record.state {
                    HealthState::Healthy
                    | HealthState::RetryingCheck { .. }
                    | HealthState::MarkedForTermination
                    | HealthState::InvisibleOnInfra => {
                        record.state = HealthState::GracefulShuttingDown;
                        record.state_transited_at = start_time;
                    }
                    HealthState::TerminatedConfirm => {
                        eprintln!(
                            "Unexpected state: Terminated confirmed but graceful shutting down"
                        );
                        record.state = HealthState::GracefulShuttingDown;
                        record.state_transited_at = start_time;
                    }
                    HealthState::GracefulShuttingDown => {}
                },
            },
            None => match &mut record.state {
                HealthState::Healthy | HealthState::InvisibleOnInfra => {
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

        if let HealthState::GracefulShuttingDown = record.state
            && record.state_transited_at + max_graceful_shutdown_wait_time < start_time
        {
            record.state = HealthState::MarkedForTermination;
            record.state_transited_at = start_time;
        }

        true
    });

    health_records.extend(new_worker_records);

    Ok(())
}

pub fn get_workers_to_terminate(health_records: &HealthRecords) -> Vec<WorkerId> {
    health_records
        .iter()
        .filter_map(|(id, record)| {
            if let HealthState::MarkedForTermination = record.state {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker_infra::{WorkerInfo, WorkerInstanceState};
    use chrono::{TimeDelta, Utc};

    // Helper function to create a test Context
    fn create_test_context(
        start_time: DateTime<Utc>,
        max_graceful_shutdown_wait_time: TimeDelta,
        max_healthy_check_retrials: usize,
    ) -> Context {
        Context {
            start_time,
            domain: "test.example.com".to_string(),
            max_graceful_shutdown_wait_time,
            max_healthy_check_retrials,
            max_start_timeout: TimeDelta::minutes(10),
            max_starting_count: 5,
        }
    }

    // Helper function to create a WorkerId
    fn worker_id(id: &str) -> WorkerId {
        WorkerId(id.to_string())
    }

    // Helper function to create a WorkerInfo
    fn create_worker_info(id: &str, state: WorkerInstanceState) -> WorkerInfo {
        WorkerInfo {
            id: worker_id(id),
            ip: None,
            instance_state: state,
            instance_created: Utc::now(),
        }
    }

    // Helper function to create a HealthRecord
    fn create_health_record(state: HealthState, transited_at: DateTime<Utc>) -> HealthRecord {
        HealthRecord {
            state,
            state_transited_at: transited_at,
        }
    }

    // ========================================
    // 1. 신규 워커 감지 (New Workers Discovery)
    // ========================================

    #[test]
    fn test_new_worker_with_good_response() {
        // Case 1.1: 정상 워커 신규 발견
        let start_time = Utc::now();
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);
        let mut health_records = HealthRecords::new();

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::Good)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        assert_eq!(health_records.len(), 1);
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_new_worker_with_graceful_shutdown() {
        // Case 1.2: 종료 중인 워커 신규 발견
        let start_time = Utc::now();
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);
        let mut health_records = HealthRecords::new();

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::GracefulShuttingDown)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        assert_eq!(health_records.len(), 1);
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_new_worker_with_no_response() {
        // Case 1.3: 응답 없는 워커 신규 발견
        let start_time = Utc::now();
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);
        let mut health_records = HealthRecords::new();

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        assert_eq!(health_records.len(), 1);
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::RetryingCheck { retrials: 1 }));
        assert_eq!(record.state_transited_at, start_time);
    }

    // ========================================
    // 2. 기존 워커의 상태 업데이트 (Happy Path)
    // ========================================

    #[test]
    fn test_healthy_worker_stays_healthy() {
        // Case 2.1: Healthy 유지
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Healthy, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::Good)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy));
        assert_eq!(record.state_transited_at, start_time); // 시간이 갱신됨
    }

    #[test]
    fn test_retrying_worker_recovers() {
        // Case 2.2: 복구 (Retrying -> Healthy)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::RetryingCheck { retrials: 2 }, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::Good)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_healthy_worker_receives_graceful_shutdown() {
        // Case 2.3: 종료 신호 수신 (Healthy -> Graceful)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Healthy, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::GracefulShuttingDown)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, start_time);
    }

    // ========================================
    // 3. 헬스 체크 실패 및 재시도 로직 (Retry Logic)
    // ========================================

    #[test]
    fn test_healthy_worker_first_failure() {
        // Case 3.1: 최초 실패
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Healthy, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::RetryingCheck { retrials: 1 }));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_retrying_worker_increases_retrials() {
        // Case 3.2: 재시도 횟수 증가
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::RetryingCheck { retrials: 2 }, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::RetryingCheck { retrials: 3 }));
        // state가 변경되지 않았으므로 transited_at은 이전 시간 유지
        assert_eq!(record.state_transited_at, previous_time);
    }

    #[test]
    fn test_max_retrials_exceeded() {
        // Case 3.3: 최대 재시도 횟수 초과
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::RetryingCheck { retrials: 3 }, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::MarkedForTermination));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_marked_for_termination_stays_unchanged_on_no_response() {
        // Case 3.4: 이미 종료 예정인 경우 (MarkedForTermination)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::MarkedForTermination, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::MarkedForTermination));
        assert_eq!(record.state_transited_at, previous_time); // 변경 없음
    }

    #[test]
    fn test_graceful_shutting_down_stays_unchanged_on_no_response() {
        // Case 3.4: 이미 종료 예정인 경우 (GracefulShuttingDown)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::GracefulShuttingDown, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, previous_time); // 변경 없음
    }

    #[test]
    fn test_terminated_confirm_stays_unchanged_on_no_response() {
        // Case 3.4: 이미 종료 예정인 경우 (TerminatedConfirm)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::TerminatedConfirm, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::TerminatedConfirm));
        assert_eq!(record.state_transited_at, previous_time); // 변경 없음
    }

    // ========================================
    // 4. 인프라에서 사라진 워커 처리 (Infrastructure Sync)
    // ========================================

    #[test]
    fn test_healthy_worker_disappears_from_infra() {
        // Case 4.1: 사라짐 감지 - Healthy 상태
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Healthy, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_retrying_worker_disappears_from_infra() {
        // Case 4.1: 사라짐 감지 - RetryingCheck 상태
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::RetryingCheck { retrials: 2 }, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_graceful_shutting_down_worker_disappears_from_infra() {
        // Case 4.1: 사라짐 감지 - GracefulShuttingDown 상태
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::GracefulShuttingDown, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_invisible_worker_returns() {
        // Case 4.2: 사라졌던 워커의 복귀
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::InvisibleOnInfra, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::Good)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_invisible_worker_stays_retained() {
        // Case 4.3: 사라진 상태 유지 (5분 미만 경과)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(2); // 2분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::InvisibleOnInfra, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 여전히 존재해야 함
        assert!(health_records.contains_key(&worker_id("worker1")));
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
    }

    // ========================================
    // 5. 오래된 레코드 정리 (Cleanup / Retention Policy)
    // ========================================

    #[test]
    fn test_old_marked_for_termination_is_deleted() {
        // Case 5.1: 정리 대상 삭제 - MarkedForTermination
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::MarkedForTermination, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 삭제되어야 함
        assert!(!health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_old_terminated_confirm_is_deleted() {
        // Case 5.1: 정리 대상 삭제 - TerminatedConfirm
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::TerminatedConfirm, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 삭제되어야 함
        assert!(!health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_old_invisible_on_infra_is_deleted() {
        // Case 5.1: 정리 대상 삭제 - InvisibleOnInfra
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::InvisibleOnInfra, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 삭제되어야 함
        assert!(!health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_recent_marked_for_termination_is_retained() {
        // Case 5.2: 정리 대상 아님 (시간 미달) - MarkedForTermination
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::MarkedForTermination, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 보존되어야 함
        assert!(health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_recent_terminated_confirm_is_retained() {
        // Case 5.2: 정리 대상 아님 (시간 미달) - TerminatedConfirm
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::TerminatedConfirm, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 보존되어야 함
        assert!(health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_recent_invisible_on_infra_is_retained() {
        // Case 5.2: 정리 대상 아님 (시간 미달) - InvisibleOnInfra
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::InvisibleOnInfra, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 보존되어야 함
        assert!(health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_healthy_worker_not_deleted_when_missing_from_infra() {
        // Case 5.3: 정상 상태 워커는 InvisibleOnInfra로 전환됨 (삭제 안 됨)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Healthy, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // 빈 맵

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // 레코드가 보존되고 InvisibleOnInfra로 변경됨
        assert!(health_records.contains_key(&worker_id("worker1")));
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
    }

    // ========================================
    // 6. 타임아웃 기반 강제 종료 (Graceful Shutdown Timeout)
    // ========================================

    #[test]
    fn test_graceful_shutdown_timeout_exceeded() {
        // Case 6.1: 제한 시간 초과
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::GracefulShuttingDown, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::GracefulShuttingDown)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::MarkedForTermination));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_graceful_shutdown_within_timeout() {
        // Case 6.2: 제한 시간 이내
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3분 전
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::GracefulShuttingDown, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::GracefulShuttingDown)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, previous_time); // 변경 없음
    }

    // ========================================
    // 7. 엣지 케이스 (Edge Cases)
    // ========================================

    #[test]
    fn test_terminated_confirm_receives_graceful_shutdown_signal() {
        // Case 7.1: 종료 확정 후 다시 Graceful 신호 수신
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::TerminatedConfirm, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (worker_info, Some(WorkerHealthResponse::GracefulShuttingDown)),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, start_time);
    }
}
