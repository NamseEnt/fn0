use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to target specific resources for the rerun of a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RerunWorkRequestDetails {
    /// List of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to affected by the rerun of the work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instances: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

impl RerunWorkRequestDetails {
    /// Create a new RerunWorkRequestDetails
    pub fn new() -> Self {
        Self {
            managed_instances: None,

            work_request_details: None,
        }
    }

    /// Set managed_instances
    pub fn set_managed_instances(mut self, value: Option<Vec<String>>) -> Self {
        self.managed_instances = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set managed_instances (unwraps Option)
    pub fn with_managed_instances(mut self, value: Vec<String>) -> Self {
        self.managed_instances = Some(value);
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}

impl Default for RerunWorkRequestDetails {
    fn default() -> Self {
        Self::new()
    }
}
