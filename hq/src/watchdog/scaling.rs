use crate::watchdog::{
    health_recorder::{HealthState, InMemoryHealthRecorder},
    host_infra::HostInfra,
    Context,
};

pub async fn try_scale_out(
    context: &Context,
    health_recorder: &InMemoryHealthRecorder,
    host_infra: &dyn HostInfra,
) -> color_eyre::Result<()> {
    let starting_count = health_recorder.get_starting_count().await;

    let Some(left_starting_count) = context.max_starting_count.checked_sub(starting_count) else {
        return Ok(());
    };

    println!("left_starting_count: {left_starting_count}");

    if left_starting_count == 0 {
        println!("left_starting_count == 0. No more starting hosts allowed");
        return Ok(());
    }

    let all_records = health_recorder.get_all_records().await;
    let alive_host_len = all_records
        .iter()
        .filter(|(_, record)| match record.state {
            HealthState::Starting
            | HealthState::Healthy { .. }
            | HealthState::RetryingCheck { .. }
            | HealthState::MarkedForTermination
            | HealthState::GracefulShuttingDown => true,
            HealthState::TerminatedConfirm | HealthState::InvisibleOnInfra => false,
        })
        .count();

    println!("alive_host_len: {alive_host_len}");

    if alive_host_len >= 1 {
        println!("alive_host_len >= 1. No space to launch new host instance");
        return Ok(());
    }

    println!("Launching new host instance");
    host_infra.launch_instances(1).await?;
    Ok(())
}
