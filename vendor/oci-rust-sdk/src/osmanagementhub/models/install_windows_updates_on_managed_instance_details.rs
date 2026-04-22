use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to install Windows updates on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallWindowsUpdatesOnManagedInstanceDetails {
    /// The list of Windows update unique identifiers. Note that this is not an OCID, but is a unique identifier assigned by Microsoft. Example: '6981d463-cd91-4a26-b7c4-ea4ded9183ed'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_update_name: Option<Vec<String>>,

    /// The types of Windows updates to be installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_update_types:
        Option<Vec<InstallWindowsUpdatesOnManagedInstanceDetailsWindowsUpdateTypes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

impl InstallWindowsUpdatesOnManagedInstanceDetails {
    /// Create a new InstallWindowsUpdatesOnManagedInstanceDetails
    pub fn new() -> Self {
        Self {
            windows_update_name: None,

            windows_update_types: None,

            work_request_details: None,
        }
    }

    /// Set windows_update_name
    pub fn set_windows_update_name(mut self, value: Option<Vec<String>>) -> Self {
        self.windows_update_name = value;
        self
    }

    /// Set windows_update_types
    pub fn set_windows_update_types(
        mut self,
        value: Option<Vec<InstallWindowsUpdatesOnManagedInstanceDetailsWindowsUpdateTypes>>,
    ) -> Self {
        self.windows_update_types = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set windows_update_name (unwraps Option)
    pub fn with_windows_update_name(mut self, value: Vec<String>) -> Self {
        self.windows_update_name = Some(value);
        self
    }

    /// Set windows_update_types (unwraps Option)
    pub fn with_windows_update_types(
        mut self,
        value: Vec<InstallWindowsUpdatesOnManagedInstanceDetailsWindowsUpdateTypes>,
    ) -> Self {
        self.windows_update_types = Some(value);
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}

impl Default for InstallWindowsUpdatesOnManagedInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
