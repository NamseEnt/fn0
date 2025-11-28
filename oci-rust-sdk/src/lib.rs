pub mod core;
#[cfg(feature = "os_management_hub")]
pub mod os_management_hub;

pub use core::{
    client::OciClient,
    error::{OciError, Result},
    retry::{Retrier, RetryConfiguration},
};
// Re-export os_management_hub types for convenience
#[cfg(feature = "os_management_hub")]
pub use os_management_hub::{
    models::*,
    requests::*,
    OsManagementHub,
};
