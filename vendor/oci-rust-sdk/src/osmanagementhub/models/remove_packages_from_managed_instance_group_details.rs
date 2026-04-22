use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The names of the packages to be removed from the managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovePackagesFromManagedInstanceGroupDetails {
    /// The list of package names.
    pub package_names: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for RemovePackagesFromManagedInstanceGroupDetails
pub struct RemovePackagesFromManagedInstanceGroupDetailsRequired {
    /// The list of package names.
    pub package_names: Vec<String>,
}

impl RemovePackagesFromManagedInstanceGroupDetails {
    /// Create a new RemovePackagesFromManagedInstanceGroupDetails with required fields
    pub fn new(required: RemovePackagesFromManagedInstanceGroupDetailsRequired) -> Self {
        Self {
            package_names: required.package_names,

            work_request_details: None,
        }
    }

    /// Set package_names
    pub fn set_package_names(mut self, value: Vec<String>) -> Self {
        self.package_names = value;
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
