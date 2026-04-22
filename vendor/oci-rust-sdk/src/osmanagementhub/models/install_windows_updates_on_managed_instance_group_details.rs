use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the details for the Windows update installed on the managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallWindowsUpdatesOnManagedInstanceGroupDetails {
    /// The type of Windows updates to be applied.
    pub windows_update_types:
        Vec<InstallWindowsUpdatesOnManagedInstanceGroupDetailsWindowsUpdateTypes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for InstallWindowsUpdatesOnManagedInstanceGroupDetails
pub struct InstallWindowsUpdatesOnManagedInstanceGroupDetailsRequired {
    /// The type of Windows updates to be applied.
    pub windows_update_types:
        Vec<InstallWindowsUpdatesOnManagedInstanceGroupDetailsWindowsUpdateTypes>,
}

impl InstallWindowsUpdatesOnManagedInstanceGroupDetails {
    /// Create a new InstallWindowsUpdatesOnManagedInstanceGroupDetails with required fields
    pub fn new(required: InstallWindowsUpdatesOnManagedInstanceGroupDetailsRequired) -> Self {
        Self {
            windows_update_types: required.windows_update_types,

            work_request_details: None,
        }
    }

    /// Set windows_update_types
    pub fn set_windows_update_types(
        mut self,
        value: Vec<InstallWindowsUpdatesOnManagedInstanceGroupDetailsWindowsUpdateTypes>,
    ) -> Self {
        self.windows_update_types = value;
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
