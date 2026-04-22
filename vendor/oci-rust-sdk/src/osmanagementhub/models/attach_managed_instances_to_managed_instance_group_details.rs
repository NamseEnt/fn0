use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to attach managed instances to a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachManagedInstancesToManagedInstanceGroupDetails {
    /// List of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to attach to the group.
    pub managed_instances: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for AttachManagedInstancesToManagedInstanceGroupDetails
pub struct AttachManagedInstancesToManagedInstanceGroupDetailsRequired {
    /// List of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to attach to the group.
    pub managed_instances: Vec<String>,
}

impl AttachManagedInstancesToManagedInstanceGroupDetails {
    /// Create a new AttachManagedInstancesToManagedInstanceGroupDetails with required fields
    pub fn new(required: AttachManagedInstancesToManagedInstanceGroupDetailsRequired) -> Self {
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
