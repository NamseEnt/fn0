use chrono::{Duration, Utc};
use color_eyre::eyre::Result;
use std::collections::{BTreeSet, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::time::sleep;

use crate::watchdog::{
    self,
    host_infra::{HostInfo, HostInstanceState},
    task_orchestrator::{HealthCheckEntry, HostInfoEntry, SharedState},
    Context, HostId,
};

// 타임아웃 및 간격 상수
const HOST_INFRA_INTERVAL: StdDuration = StdDuration::from_secs(10);
const HEALTH_CHECK_INTERVAL: StdDuration = StdDuration::from_secs(5);
const REAPER_INTERVAL: StdDuration = StdDuration::from_secs(10);
const DNS_SYNCER_INTERVAL: StdDuration = StdDuration::from_secs(5);

const HEALTH_CHECK_ELAPSED_THRESHOLD: Duration = Duration::seconds(15);
const REGISTER_ELAPSED_THRESHOLD: Duration = Duration::seconds(60);
const HEALTHY_IP_THRESHOLD: Duration = Duration::milliseconds(7500);

pub async fn run() -> Result<()> {
    let host_infra = Arc::new(watchdog::host_infra::oci::OciHostInfra::new().await);
    let dns = Arc::new(watchdog::dns::cloudflare::CloudflareDns::new(None).await);
    let context = Arc::new(Context::new());
    let shared_state = Arc::new(SharedState::new());

    let task1 = host_infra_task(host_infra.clone(), shared_state.clone());
    let task2 = health_checker_task(shared_state.clone(), context.clone());
    let task3 = reaper_task(host_infra.clone(), shared_state.clone());
    let task4 = dns_syncer_task(dns.clone(), shared_state.clone());

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Shutdown signal received");
        }
        result = task1 => {
            eprintln!("host_infra_task exited: {:?}", result);
        }
        result = task2 => {
            eprintln!("health_checker_task exited: {:?}", result);
        }
        result = task3 => {
            eprintln!("reaper_task exited: {:?}", result);
        }
        result = task4 => {
            eprintln!("dns_syncer_task exited: {:?}", result);
        }
    }

    Ok(())
}

async fn host_infra_task(
    host_infra: Arc<dyn watchdog::host_infra::HostInfra>,
    shared_state: Arc<SharedState>,
) -> Result<()> {
    let mut interval = tokio::time::interval(HOST_INFRA_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let host_infos = match host_infra.get_host_infos().await {
            Ok(infos) => infos,
            Err(e) => {
                eprintln!("Failed to get host infos: {:?}", e);
                continue;
            }
        };

        let current_hosts: HashSet<HostId> = host_infos.iter().map(|h| h.id.clone()).collect();

        let removed_hosts: Vec<HostId> = shared_state
            .host_info_map
            .iter()
            .filter_map(|entry| {
                if !current_hosts.contains(entry.key()) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for host_id in &removed_hosts {
            shared_state.host_info_map.remove(host_id);
            shared_state.health_map.remove(host_id);
            let mut last_terminate_set = shared_state.last_terminate_set.lock().await;
            last_terminate_set.remove(host_id);
            println!("Host {:?} removed from infrastructure", host_id);
        }

        let now = Utc::now();
        for host_info in host_infos {
            shared_state
                .host_info_map
                .entry(host_info.id.clone())
                .and_modify(|entry| {
                    entry.info = host_info.clone();
                })
                .or_insert_with(|| {
                    println!("Host {:?} discovered", host_info.id);
                    HostInfoEntry {
                        info: host_info.clone(),
                        registered_at: now,
                    }
                });
        }

        println!(
            "Host info map updated: {} hosts, {} removed",
            shared_state.host_info_map.len(),
            removed_hosts.len()
        );
    }
}

async fn health_checker_task(
    shared_state: Arc<SharedState>,
    context: Arc<Context>,
) -> Result<()> {
    let mut interval = tokio::time::interval(HEALTH_CHECK_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let to_remove: Vec<HostId> = shared_state
            .health_map
            .iter()
            .filter_map(|entry| {
                let host_id = entry.key().clone();
                if let Some(host_entry) = shared_state.host_info_map.get(&host_id) {
                    if host_entry.info.instance_state == HostInstanceState::Terminating {
                        return Some(host_id);
                    }
                    None
                } else {
                    Some(host_id)
                }
            })
            .collect();

        for host_id in &to_remove {
            shared_state.health_map.remove(host_id);
            println!("Removed {:?} from health_map (not in host_info_map or terminating)", host_id);
        }

        let hosts: Vec<(HostId, HostInfo)> = shared_state
            .host_info_map
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().info.clone()))
            .collect();

        let mut tasks = Vec::new();
        for (host_id, host_info) in hosts {
            let shared_state = shared_state.clone();
            let domain = context.domain.clone();
            let task = tokio::spawn(async move {
                let sleep_ms = rand::random::<u64>() % 1000;
                sleep(StdDuration::from_millis(sleep_ms)).await;

                let health_response = watchdog::check_health_single(&host_info, &domain).await;

                if health_response.is_some() {
                    shared_state.health_map.insert(
                        host_id.clone(),
                        HealthCheckEntry {
                            last_check_time: Utc::now(),
                        },
                    );
                    println!("Health check passed for {:?}", host_id);
                } else {
                    println!("Health check failed/timeout for {:?}", host_id);
                }
            });
            tasks.push(task);
        }

        for task in tasks {
            let _ = task.await;
        }
    }
}

async fn reaper_task(
    host_infra: Arc<dyn watchdog::host_infra::HostInfra>,
    shared_state: Arc<SharedState>,
) -> Result<()> {
    let mut interval = tokio::time::interval(REAPER_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let now = Utc::now();
        let last_terminate_set = shared_state.last_terminate_set.lock().await.clone();

        let terminate_targets: Vec<(HostId, HostInfo)> = shared_state
            .host_info_map
            .iter()
            .filter_map(|entry| {
                let host_id = entry.key();
                let host_entry = entry.value();

                if last_terminate_set.contains(host_id) {
                    return None;
                }

                if host_entry.info.instance_state == HostInstanceState::Terminating {
                    return None;
                }

                let register_elapsed = now - host_entry.registered_at;
                if register_elapsed < REGISTER_ELAPSED_THRESHOLD {
                    return None;
                }

                let health_check_elapsed = if let Some(health_entry) = shared_state.health_map.get(host_id) {
                    now - health_entry.last_check_time
                } else {
                    Duration::days(10000)
                };

                if health_check_elapsed > HEALTH_CHECK_ELAPSED_THRESHOLD {
                    Some((host_id.clone(), host_entry.info.clone()))
                } else {
                    None
                }
            })
            .collect();

        if !terminate_targets.is_empty() {
            println!("Reaper found {} hosts to terminate", terminate_targets.len());
        }

        let mut tasks = Vec::new();
        for (host_id, _host_info) in &terminate_targets {
            let host_infra = host_infra.clone();
            let host_id = host_id.clone();
            let task = tokio::spawn(async move {
                let sleep_ms = rand::random::<u64>() % 1000;
                sleep(StdDuration::from_millis(sleep_ms)).await;

                if let Err(e) = host_infra.terminate(&host_id).await {
                    eprintln!("Failed to terminate host {:?}: {:?}", host_id, e);
                } else {
                    println!("Terminated host {:?}", host_id);
                }
            });
            tasks.push(task);
        }

        for task in tasks {
            let _ = task.await;
        }

        let mut new_terminate_set = HashSet::new();
        for (host_id, _) in terminate_targets {
            new_terminate_set.insert(host_id);
        }
        *shared_state.last_terminate_set.lock().await = new_terminate_set;
    }
}

async fn dns_syncer_task(
    dns: Arc<dyn watchdog::dns::Dns>,
    shared_state: Arc<SharedState>,
) -> Result<()> {
    let mut interval = tokio::time::interval(DNS_SYNCER_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let now = Utc::now();

        let healthy_ips: BTreeSet<IpAddr> = shared_state
            .health_map
            .iter()
            .filter_map(|entry| {
                let host_id = entry.key();
                let health_entry = entry.value();

                let elapsed = now - health_entry.last_check_time;
                if elapsed > HEALTHY_IP_THRESHOLD {
                    return None;
                }

                shared_state
                    .host_info_map
                    .get(host_id)
                    .and_then(|host_entry| host_entry.info.ip)
            })
            .collect();

        let cached_ips = shared_state.dns_cache.lock().await.clone();

        if healthy_ips == cached_ips {
            continue;
        }

        let healthy_ips_vec: Vec<IpAddr> = healthy_ips.iter().copied().collect();
        match dns.sync_ips(healthy_ips_vec).await {
            Ok(_) => {
                println!(
                    "DNS synced. Previous: {}, Current: {}",
                    cached_ips.len(),
                    healthy_ips.len()
                );
                *shared_state.dns_cache.lock().await = healthy_ips;
            }
            Err(e) => {
                eprintln!("Failed to sync DNS: {:?}", e);
            }
        }
    }
}
