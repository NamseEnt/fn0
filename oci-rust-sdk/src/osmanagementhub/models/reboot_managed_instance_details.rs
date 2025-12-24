use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used for the reboot job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebootManagedInstanceDetails {
    /// The number of minutes the service waits for the reboot to complete. If the instance doesn't reboot within this time, the reboot job status is set to failed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reboot_timeout_in_mins: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

impl RebootManagedInstanceDetails {
    /// Create a new RebootManagedInstanceDetails
    pub fn new() -> Self {
        Self {
            reboot_timeout_in_mins: None,

            work_request_details: None,
        }
    }

    /// Set reboot_timeout_in_mins
    pub fn set_reboot_timeout_in_mins(mut self, value: Option<i64>) -> Self {
        self.reboot_timeout_in_mins = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set reboot_timeout_in_mins (unwraps Option)
    pub fn with_reboot_timeout_in_mins(mut self, value: i64) -> Self {
        self.reboot_timeout_in_mins = Some(value);
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}

impl Default for RebootManagedInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
