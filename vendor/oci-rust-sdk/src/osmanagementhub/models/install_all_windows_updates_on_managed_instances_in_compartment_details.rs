use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to install all Windows updates of a specified type on managed instances within the specified compartment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// The types of Windows updates to be installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_update_types: Option<
        Vec<InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetailsWindowsUpdateTypes>,
    >,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetails
pub struct InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetails {
    /// Create a new InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetails with required fields
    pub fn new(
        required: InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetailsRequired,
    ) -> Self {
        Self {
            compartment_id: required.compartment_id,

            windows_update_types: None,

            work_request_details: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set windows_update_types
    pub fn set_windows_update_types(
        mut self,
        value: Option<
            Vec<InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetailsWindowsUpdateTypes>,
        >,
    ) -> Self {
        self.windows_update_types = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set windows_update_types (unwraps Option)
    pub fn with_windows_update_types(
        mut self,
        value: Vec<
            InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetailsWindowsUpdateTypes,
        >,
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
