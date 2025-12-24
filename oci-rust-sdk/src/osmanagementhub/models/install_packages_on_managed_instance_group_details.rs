use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The names of the packages to be installed on the managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPackagesOnManagedInstanceGroupDetails {
    /// The list of package names.
    pub package_names: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,

    /// Indicates whether this is the latest package version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest: Option<bool>,
}

/// Required fields for InstallPackagesOnManagedInstanceGroupDetails
pub struct InstallPackagesOnManagedInstanceGroupDetailsRequired {
    /// The list of package names.
    pub package_names: Vec<String>,
}

impl InstallPackagesOnManagedInstanceGroupDetails {
    /// Create a new InstallPackagesOnManagedInstanceGroupDetails with required fields
    pub fn new(required: InstallPackagesOnManagedInstanceGroupDetailsRequired) -> Self {
        Self {
            package_names: required.package_names,

            work_request_details: None,

            is_latest: None,
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

    /// Set is_latest
    pub fn set_is_latest(mut self, value: Option<bool>) -> Self {
        self.is_latest = value;
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }

    /// Set is_latest (unwraps Option)
    pub fn with_is_latest(mut self, value: bool) -> Self {
        self.is_latest = Some(value);
        self
    }
}
