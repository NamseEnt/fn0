use super::*;
use crate::{worker_infra::WorkerHealthKind, *};

pub fn update_health_records(
    context: &Context,
    health_records: &mut HealthRecords,
    worker_health_response_map: WorkerHealthResponseMap,
) -> color_eyre::Result<()> {
    let &Context {
        start_time,
        max_graceful_shutdown_wait_time,
        max_healthy_check_retrials,
        max_start_timeout,
        ..
    } = context;

    let worker_health_response_not_in_records = worker_health_response_map
        .iter()
        .filter(|(id, _)| !health_records.contains_key(id));

    let mut new_worker_records = vec![];

    for (worker_id, (worker_info, worker_status)) in worker_health_response_not_in_records {
        new_worker_records.push((
            worker_id.clone(),
            HealthRecord {
                state: {
                    match worker_info.instance_state {
                        WorkerInstanceState::Starting => HealthState::Starting,
                        WorkerInstanceState::Running => match worker_status {
                            Some(worker_health_response) => match worker_health_response.kind {
                                WorkerHealthKind::Good => HealthState::Healthy {
                                    ip: worker_health_response.ip,
                                },
                                WorkerHealthKind::GracefulShuttingDown => {
                                    HealthState::GracefulShuttingDown
                                }
                            },
                            None => HealthState::RetryingCheck { retrials: 1 },
                        },
                        WorkerInstanceState::Terminating => continue,
                    }
                },
                state_transited_at: start_time,
            },
        ));
    }

    health_records.retain(|worker_id, record| {
        let Some((_worker_info, health_response)) = worker_health_response_map.get(worker_id)
        else {
            match record.state {
                HealthState::Starting
                | HealthState::Healthy { .. }
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
            Some(worker_health_response) => match worker_health_response.kind {
                WorkerHealthKind::Good => {
                    record.state = HealthState::Healthy {
                        ip: worker_health_response.ip,
                    };
                    record.state_transited_at = start_time;
                }
                WorkerHealthKind::GracefulShuttingDown => match record.state {
                    HealthState::Starting
                    | HealthState::Healthy { .. }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker_infra::{WorkerHealthKind, WorkerHealthResponse, WorkerInfo, WorkerInstanceState};
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

    // Helper function to create a test IP address
    fn test_ip() -> std::net::IpAddr {
        "127.0.0.1".parse().unwrap()
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
    // 1. New Workers Discovery
    // ========================================

    #[test]
    fn test_new_worker_with_good_response() {
        // Case 1.1: New healthy worker discovered
        let start_time = Utc::now();
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);
        let mut health_records = HealthRecords::new();

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::Good,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        assert_eq!(health_records.len(), 1);
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy { .. }));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_new_worker_with_graceful_shutdown() {
        // Case 1.2: New worker in graceful shutdown discovered
        let start_time = Utc::now();
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);
        let mut health_records = HealthRecords::new();

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::GracefulShuttingDown,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        assert_eq!(health_records.len(), 1);
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_new_worker_with_no_response() {
        // Case 1.3: New worker with no response discovered
        let start_time = Utc::now();
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);
        let mut health_records = HealthRecords::new();

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        assert_eq!(health_records.len(), 1);
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(
            record.state,
            HealthState::RetryingCheck { retrials: 1 }
        ));
        assert_eq!(record.state_transited_at, start_time);
    }

    // ========================================
    // 2. Existing Worker State Updates (Happy Path)
    // ========================================

    #[test]
    fn test_healthy_worker_stays_healthy() {
        // Case 2.1: Healthy status maintained
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(
                HealthState::Healthy { ip: test_ip() },
                previous_time,
            ),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::Good,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy { .. }));
        assert_eq!(record.state_transited_at, start_time); // Time is updated
    }

    #[test]
    fn test_retrying_worker_recovers() {
        // Case 2.2: Recovery (Retrying -> Healthy)
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
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::Good,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy { .. }));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_healthy_worker_receives_graceful_shutdown() {
        // Case 2.3: Shutdown signal received (Healthy -> Graceful)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(
                HealthState::Healthy { ip: test_ip() },
                previous_time,
            ),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::GracefulShuttingDown,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, start_time);
    }

    // ========================================
    // 3. Health Check Failures and Retry Logic
    // ========================================

    #[test]
    fn test_healthy_worker_first_failure() {
        // Case 3.1: First failure
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(
                HealthState::Healthy { ip: test_ip() },
                previous_time,
            ),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(
            record.state,
            HealthState::RetryingCheck { retrials: 1 }
        ));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_retrying_worker_increases_retrials() {
        // Case 3.2: Retry count increases
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
        assert!(matches!(
            record.state,
            HealthState::RetryingCheck { retrials: 3 }
        ));
        // Since state hasn't changed, transited_at keeps the previous time
        assert_eq!(record.state_transited_at, previous_time);
    }

    #[test]
    fn test_max_retrials_exceeded() {
        // Case 3.3: Max retry count exceeded
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
        // Case 3.4: Already scheduled for termination (MarkedForTermination)
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
        assert_eq!(record.state_transited_at, previous_time); // No change
    }

    #[test]
    fn test_graceful_shutting_down_stays_unchanged_on_no_response() {
        // Case 3.4: Already scheduled for termination (GracefulShuttingDown)
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
        assert_eq!(record.state_transited_at, previous_time); // No change
    }

    #[test]
    fn test_terminated_confirm_stays_unchanged_on_no_response() {
        // Case 3.4: Already scheduled for termination (TerminatedConfirm)
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
        assert_eq!(record.state_transited_at, previous_time); // No change
    }

    // ========================================
    // 4. Infrastructure Sync (Disappeared Workers)
    // ========================================

    #[test]
    fn test_healthy_worker_disappears_from_infra() {
        // Case 4.1: Disappearance detected - Healthy state
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(
                HealthState::Healthy { ip: test_ip() },
                previous_time,
            ),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_retrying_worker_disappears_from_infra() {
        // Case 4.1: Disappearance detected - RetryingCheck state
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::RetryingCheck { retrials: 2 }, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_graceful_shutting_down_worker_disappears_from_infra() {
        // Case 4.1: Disappearance detected - GracefulShuttingDown state
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::GracefulShuttingDown, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_invisible_worker_returns() {
        // Case 4.2: Return of disappeared worker
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
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::Good,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy { .. }));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_invisible_worker_stays_retained() {
        // Case 4.3: Invisible state maintained (less than 5 minutes elapsed)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(2); // 2 minutes ago
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::InvisibleOnInfra, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record should still exist
        assert!(health_records.contains_key(&worker_id("worker1")));
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
    }

    // ========================================
    // 5. Cleanup / Retention Policy
    // ========================================

    #[test]
    fn test_old_marked_for_termination_is_deleted() {
        // Case 5.1: Delete cleanup target - MarkedForTermination
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6 minutes ago
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::MarkedForTermination, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record should be deleted
        assert!(!health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_old_terminated_confirm_is_deleted() {
        // Case 5.1: Delete cleanup target - TerminatedConfirm
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6 minutes ago
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::TerminatedConfirm, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record should be deleted
        assert!(!health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_old_invisible_on_infra_is_deleted() {
        // Case 5.1: Delete cleanup target - InvisibleOnInfra
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6 minutes ago
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::InvisibleOnInfra, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record should be deleted
        assert!(!health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_recent_marked_for_termination_is_retained() {
        // Case 5.2: Not a cleanup target (insufficient time) - MarkedForTermination
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3 minutes ago
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::MarkedForTermination, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record should be preserved
        assert!(health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_recent_terminated_confirm_is_retained() {
        // Case 5.2: Not a cleanup target (insufficient time) - TerminatedConfirm
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3 minutes ago
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::TerminatedConfirm, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record should be preserved
        assert!(health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_recent_invisible_on_infra_is_retained() {
        // Case 5.2: Not a cleanup target (insufficient time) - InvisibleOnInfra
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3 minutes ago
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::InvisibleOnInfra, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record should be preserved
        assert!(health_records.contains_key(&worker_id("worker1")));
    }

    #[test]
    fn test_healthy_worker_not_deleted_when_missing_from_infra() {
        // Case 5.3: Healthy worker transitions to InvisibleOnInfra (not deleted)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(
                HealthState::Healthy { ip: test_ip() },
                previous_time,
            ),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        // Record is preserved and changed to InvisibleOnInfra
        assert!(health_records.contains_key(&worker_id("worker1")));
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
    }

    // ========================================
    // 6. Graceful Shutdown Timeout
    // ========================================

    #[test]
    fn test_graceful_shutdown_timeout_exceeded() {
        // Case 6.1: Timeout exceeded
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(6); // 6 minutes ago
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
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::GracefulShuttingDown,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::MarkedForTermination));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_graceful_shutdown_within_timeout() {
        // Case 6.2: Within timeout
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(3); // 3 minutes ago
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
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::GracefulShuttingDown,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, previous_time); // No change
    }

    // ========================================
    // 7. Starting State Handling
    // ========================================

    #[test]
    fn test_new_worker_in_starting_state() {
        // Case 7.1: New worker detected - Starting state
        let start_time = Utc::now();
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);
        let mut health_records = HealthRecords::new();

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Starting);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        assert_eq!(health_records.len(), 1);
        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Starting));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_starting_to_healthy_transition() {
        // Case 7.2: Starting → Healthy transition
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Starting, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::Good,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Healthy { .. }));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_starting_to_graceful_shutdown_transition() {
        // Case 7.3: Starting → GracefulShuttingDown transition
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Starting, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Running);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(
            worker_id("worker1"),
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::GracefulShuttingDown,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_starting_state_maintained_within_timeout() {
        // Case 7.4: Starting state maintained (within timeout)
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(5); // 5 minutes ago (within 10 minute timeout)
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Starting, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Starting);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::Starting));
        assert_eq!(record.state_transited_at, previous_time); // No change
    }

    #[test]
    fn test_starting_timeout_exceeded() {
        // Case 7.5: Starting timeout exceeded → MarkedForTermination
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(11); // 11 minutes ago (exceeds 10 minute timeout)
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Starting, previous_time),
        );

        let worker_info = create_worker_info("worker1", WorkerInstanceState::Starting);
        let mut response_map = WorkerHealthResponseMap::new();
        response_map.insert(worker_id("worker1"), (worker_info, None));

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::MarkedForTermination));
        assert_eq!(record.state_transited_at, start_time);
    }

    #[test]
    fn test_starting_worker_disappears_from_infra() {
        // Case 7.6: Starting state worker disappears from infrastructure
        let start_time = Utc::now();
        let previous_time = start_time - TimeDelta::minutes(1);
        let context = create_test_context(start_time, TimeDelta::minutes(5), 3);

        let mut health_records = HealthRecords::new();
        health_records.insert(
            worker_id("worker1"),
            create_health_record(HealthState::Starting, previous_time),
        );

        let response_map = WorkerHealthResponseMap::new(); // Empty map

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::InvisibleOnInfra));
        assert_eq!(record.state_transited_at, start_time);
    }

    // ========================================
    // 8. Edge Cases
    // ========================================

    #[test]
    fn test_terminated_confirm_receives_graceful_shutdown_signal() {
        // Case 8.1: Receiving Graceful signal again after termination confirmed
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
            (
                worker_info,
                Some(WorkerHealthResponse {
                    kind: WorkerHealthKind::GracefulShuttingDown,
                    ip: test_ip(),
                }),
            ),
        );

        update_health_records(&context, &mut health_records, response_map).unwrap();

        let record = health_records.get(&worker_id("worker1")).unwrap();
        assert!(matches!(record.state, HealthState::GracefulShuttingDown));
        assert_eq!(record.state_transited_at, start_time);
    }
}
