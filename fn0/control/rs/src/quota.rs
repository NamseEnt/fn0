//! Platform quota defaults.
//!
//! Per-build limits gate a single deploy; the per-project object-storage
//! quotas feed the hourly presign enforcement pass (`actions/presign_quota`).
//! Hourly caps exist for early abuse cut-off; only the monthly (rolling
//! 30-day) caps bound platform cost. Per-project overrides live in
//! `ProjectQuotaOverridesDoc` (the paid "contact us" path).

pub const MAX_FILES_PER_BUILD: usize = 10_000;
pub const MAX_TOTAL_SIZE_PER_BUILD: u64 = 1_000_000_000;

pub const QUOTA_ROLLING_WINDOW_HOURS: i64 = 720;

pub const PRESIGN_MINT_PER_MONTH: u64 = 100_000;
pub const OBJECT_STORAGE_CLASS_A_PER_HOUR: u64 = 2_000;
pub const OBJECT_STORAGE_CLASS_A_PER_MONTH: u64 = 100_000;
pub const OBJECT_STORAGE_CLASS_B_PER_HOUR: u64 = 5_000;
pub const OBJECT_STORAGE_CLASS_B_PER_MONTH: u64 = 100_000;
