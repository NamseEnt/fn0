//! Per-project presign mint ceiling, attached to the object-storage hijack by
//! the worker (absent in `forte dev`).
//!
//! This used to be the actuator for a cost-control loop: the control plane
//! metered R2 operations against the platform's bill, decided which projects
//! were over quota, and workers refused to mint for those. Objects now live in
//! each project's own Cloudflare account, so there is no platform bill to
//! ration and no reason to ask the control plane's permission. What is left is
//! a flat worker-local ceiling — policy against a runaway loop, not accounting.

use std::collections::HashMap;
use std::sync::Mutex;

pub const PRESIGN_MINT_PER_HOUR: u64 = 1_000;

#[derive(Default)]
pub struct PresignGate {
    minted: Mutex<HashMap<(String, i64), u64>>,
}

pub enum PresignDenied {
    RateLimited,
}

impl PresignGate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_mint(&self, project_id: &str, epoch_hour: i64) -> Result<(), PresignDenied> {
        let mut minted = self.minted.lock().unwrap();
        minted.retain(|(_, hour), _| *hour >= epoch_hour - 1);
        let count = minted
            .entry((project_id.to_string(), epoch_hour))
            .or_insert(0);
        if *count >= PRESIGN_MINT_PER_HOUR {
            return Err(PresignDenied::RateLimited);
        }
        *count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_hourly_ceiling_then_refuses() {
        let gate = PresignGate::new();
        for _ in 0..PRESIGN_MINT_PER_HOUR {
            assert!(gate.try_mint("proj1", 100).is_ok());
        }
        assert!(gate.try_mint("proj1", 100).is_err());
    }

    #[test]
    fn budgets_are_per_project_and_per_hour() {
        let gate = PresignGate::new();
        for _ in 0..PRESIGN_MINT_PER_HOUR {
            assert!(gate.try_mint("proj1", 100).is_ok());
        }
        assert!(gate.try_mint("proj2", 100).is_ok());
        assert!(gate.try_mint("proj1", 101).is_ok());
    }
}
