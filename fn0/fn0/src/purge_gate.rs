//! Per-project rate limit on app-requested edge invalidations.
//!
//! A purge is cheaper to trigger than the write that used to be the only way to
//! cause one, so the direct API needs its own ceiling: purges land on a
//! Cloudflare token shared by every project, and one app must not be able to
//! spend the whole budget.

use std::collections::HashMap;
use std::sync::Mutex;

pub const PURGE_PER_HOUR: u64 = 1_000;

#[derive(Default)]
pub struct PurgeGate {
    requested: Mutex<HashMap<(String, i64), u64>>,
}

impl PurgeGate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_purge(&self, project_id: &str, epoch_hour: i64) -> bool {
        let mut requested = self.requested.lock().unwrap();
        requested.retain(|(_, hour), _| *hour >= epoch_hour - 1);
        let count = requested
            .entry((project_id.to_string(), epoch_hour))
            .or_insert(0);
        if *count >= PURGE_PER_HOUR {
            return false;
        }
        *count += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_hourly_ceiling_then_refuses() {
        let gate = PurgeGate::new();
        for _ in 0..PURGE_PER_HOUR {
            assert!(gate.try_purge("proj1", 100));
        }
        assert!(!gate.try_purge("proj1", 100));
    }

    #[test]
    fn budgets_are_per_project_and_per_hour() {
        let gate = PurgeGate::new();
        for _ in 0..PURGE_PER_HOUR {
            assert!(gate.try_purge("proj1", 100));
        }
        assert!(gate.try_purge("proj2", 100));
        assert!(gate.try_purge("proj1", 101));
    }
}
