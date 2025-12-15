use crate::watchdog::host_id::HostId;
use crate::watchdog::host_infra::HostInfo;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::{BTreeSet, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct HostInfoEntry {
    pub info: HostInfo,
    pub registered_at: DateTime<Utc>,
}

pub struct HealthCheckEntry {
    pub last_check_time: DateTime<Utc>,
}

pub struct SharedState {
    pub host_info_map: Arc<DashMap<HostId, HostInfoEntry>>,
    pub health_map: Arc<DashMap<HostId, HealthCheckEntry>>,
    pub last_terminate_set: Arc<Mutex<HashSet<HostId>>>,
    pub dns_cache: Arc<Mutex<BTreeSet<IpAddr>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            host_info_map: Arc::new(DashMap::new()),
            health_map: Arc::new(DashMap::new()),
            last_terminate_set: Arc::new(Mutex::new(HashSet::new())),
            dns_cache: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }
}
