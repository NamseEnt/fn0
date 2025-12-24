use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to associate managed instances to a management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssociateManagedInstancesWithManagementStationDetails {
    /// List of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to associate to the management station.
    pub managed_instances: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for AssociateManagedInstancesWithManagementStationDetails
pub struct AssociateManagedInstancesWithManagementStationDetailsRequired {
    /// List of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to associate to the management station.
    pub managed_instances: Vec<String>,
}

impl AssociateManagedInstancesWithManagementStationDetails {
    /// Create a new AssociateManagedInstancesWithManagementStationDetails with required fields
    pub fn new(required: AssociateManagedInstancesWithManagementStationDetailsRequired) -> Self {
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
