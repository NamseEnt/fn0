//! Per-project presign mint gate, attached to the object-storage hijack by
//! the worker (absent in `forte dev`). Minting is refused for projects the
//! control plane flagged over quota (`PresignBlockedDoc`) and rate-limited
//! per project per hour worker-locally. Mint counts accumulate per
//! (project, hour) for the worker to report to the control DB, which
//! evaluates the monthly mint quota against them.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub const PRESIGN_MINT_PER_HOUR: u64 = 1_000;

#[derive(Default)]
pub struct PresignGate {
    state: Mutex<GateState>,
}

#[derive(Default)]
struct GateState {
    blocked: HashSet<String>,
    minted: HashMap<(String, i64), u64>,
}

pub enum PresignDenied {
    ProjectBlocked,
    RateLimited,
}

impl PresignGate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_blocked(&self, blocked: HashSet<String>) {
        self.state.lock().unwrap().blocked = blocked;
    }

    pub fn try_mint(&self, project_id: &str, epoch_hour: i64) -> Result<(), PresignDenied> {
        let mut state = self.state.lock().unwrap();
        if state.blocked.contains(project_id) {
            return Err(PresignDenied::ProjectBlocked);
        }
        let minted = state
            .minted
            .entry((project_id.to_string(), epoch_hour))
            .or_insert(0);
        if *minted >= PRESIGN_MINT_PER_HOUR {
            return Err(PresignDenied::RateLimited);
        }
        *minted += 1;
        Ok(())
    }

    /// Returns all per-(project, hour) mint totals, then prunes hours before
    /// the previous one. The pruned hours are still part of the returned
    /// snapshot, so the caller reports every hour at least once after it
    /// closed.
    pub fn snapshot_minted(&self, current_epoch_hour: i64) -> Vec<(String, i64, u64)> {
        let mut state = self.state.lock().unwrap();
        let snapshot = state
            .minted
            .iter()
            .map(|((project_id, hour), count)| (project_id.clone(), *hour, *count))
            .collect();
        state
            .minted
            .retain(|(_, hour), _| *hour >= current_epoch_hour - 1);
        snapshot
    }
}
