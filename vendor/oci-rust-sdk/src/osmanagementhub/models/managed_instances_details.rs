use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details about the managed instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstancesDetails {
    /// The list of managed instance OCIDs to be attached/detached.
    pub managed_instances: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for ManagedInstancesDetails
pub struct ManagedInstancesDetailsRequired {
    /// The list of managed instance OCIDs to be attached/detached.
    pub managed_instances: Vec<String>,
}

impl ManagedInstancesDetails {
    /// Create a new ManagedInstancesDetails with required fields
    pub fn new(required: ManagedInstancesDetailsRequired) -> Self {
        Self {
            managed_instances: required.managed_instances,

            work_request_details: None,
        }
    }

    /// Set managed_instances
    pub fn set_managed_instances(mut self, value: Vec<String>) -> Self {
        self.managed_instances = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}
