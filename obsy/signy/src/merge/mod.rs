use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::watch;
use tokio::time::interval;

use crate::compaction_tier::IDLE_PARTITION_AGE;
use crate::config::Config;
use crate::metrics::RuntimeMetrics;
use crate::object_storage::RemoteCache;
use crate::part::{self, PartReader};
use crate::part_registry::PartRegistry;
use crate::shutdown::wait_for_drain;
use crate::tenant_policy::{Cutoffs, TenantPolicy};

include!("scheduler.rs");
include!("transaction.rs");
include!("selection.rs");

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
