//! Core Services Module
//!
//! Contains models for Oracle Cloud Infrastructure Core Services:
//! - Compute (instances, images, shapes)
//! - VirtualNetwork (VCNs, subnets, security lists)
//! - Blockstorage (volumes, backups)

pub mod client;
pub mod error;
pub mod models;
pub mod region;
pub mod retry;

// Re-exports for convenience
pub use error::{OciError, Result};
pub use region::Region;
pub use retry::{Retrier, RetryConfiguration};
