use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the details for updating the packages on the managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAllPackagesOnManagedInstanceGroupDetails {
    /// The type of updates to be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_types: Option<Vec<UpdateAllPackagesOnManagedInstanceGroupDetailsUpdateTypes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

impl UpdateAllPackagesOnManagedInstanceGroupDetails {
    /// Create a new UpdateAllPackagesOnManagedInstanceGroupDetails
    pub fn new() -> Self {
        Self {
            update_types: None,

            work_request_details: None,
        }
    }

    /// Set update_types
    pub fn set_update_types(
        mut self,
        value: Option<Vec<UpdateAllPackagesOnManagedInstanceGroupDetailsUpdateTypes>>,
    ) -> Self {
        self.update_types = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set update_types (unwraps Option)
    pub fn with_update_types(
        mut self,
        value: Vec<UpdateAllPackagesOnManagedInstanceGroupDetailsUpdateTypes>,
    ) -> Self {
        self.update_types = Some(value);
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}

impl Default for UpdateAllPackagesOnManagedInstanceGroupDetails {
    fn default() -> Self {
        Self::new()
    }
}
