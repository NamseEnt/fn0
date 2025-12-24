use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update software packages on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePackagesOnManagedInstanceDetails {
    /// The list of package names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_names: Option<Vec<String>>,

    /// The types of updates to be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_types: Option<Vec<UpdatePackagesOnManagedInstanceDetailsUpdateTypes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

impl UpdatePackagesOnManagedInstanceDetails {
    /// Create a new UpdatePackagesOnManagedInstanceDetails
    pub fn new() -> Self {
        Self {
            package_names: None,

            update_types: None,

            work_request_details: None,
        }
    }

    /// Set package_names
    pub fn set_package_names(mut self, value: Option<Vec<String>>) -> Self {
        self.package_names = value;
        self
    }

    /// Set update_types
    pub fn set_update_types(
        mut self,
        value: Option<Vec<UpdatePackagesOnManagedInstanceDetailsUpdateTypes>>,
    ) -> Self {
        self.update_types = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set package_names (unwraps Option)
    pub fn with_package_names(mut self, value: Vec<String>) -> Self {
        self.package_names = Some(value);
        self
    }

    /// Set update_types (unwraps Option)
    pub fn with_update_types(
        mut self,
        value: Vec<UpdatePackagesOnManagedInstanceDetailsUpdateTypes>,
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

impl Default for UpdatePackagesOnManagedInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
