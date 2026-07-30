//! Platform quota defaults.
//!
//! Per-build limits gate a single deploy. The per-project object-storage
//! quotas that used to live here were cost control over the platform's own R2
//! bill; a project's objects now sit in its owner's Cloudflare account, so
//! those numbers bounded nothing we pay for. What remains of abuse control is
//! the worker-local presign mint ceiling in `fn0::presign_gate`, which is
//! policy rather than accounting.

pub const MAX_FILES_PER_BUILD: usize = 10_000;
pub const MAX_TOTAL_SIZE_PER_BUILD: u64 = 1_000_000_000;
