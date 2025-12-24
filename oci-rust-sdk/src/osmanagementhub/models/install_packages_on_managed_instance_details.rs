use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to install software packages on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPackagesOnManagedInstanceDetails {
    /// The list of package names.
    pub package_names: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for InstallPackagesOnManagedInstanceDetails
pub struct InstallPackagesOnManagedInstanceDetailsRequired {
    /// The list of package names.
    pub package_names: Vec<String>,
}

impl InstallPackagesOnManagedInstanceDetails {
    /// Create a new InstallPackagesOnManagedInstanceDetails with required fields
    pub fn new(required: InstallPackagesOnManagedInstanceDetailsRequired) -> Self {
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
