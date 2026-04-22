use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines an operation that is performed by a scheduled job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledJobOperation {
    /// The type of operation this scheduled job performs.
    pub operation_type: OperationTypes,

    /// The names of the target packages. This parameter only applies when the scheduled job is for installing, updating, or removing packages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_names: Option<Vec<String>>,

    /// Unique identifier for the Windows update. This parameter only applies if the scheduled job is for installing Windows updates. Note that this is not an OCID, but is a unique identifier assigned by Microsoft. For example: '6981d463-cd91-4a26-b7c4-ea4ded9183ed'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_update_names: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub manage_module_streams_details: Option<ManageModuleStreamsInScheduledJobDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_module_streams_details: Option<ModuleStreamDetails>,

    /// The software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm). This parameter only applies when the scheduled job is for attaching or detaching software sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_ids: Option<Vec<String>>,

    /// The number of minutes the service waits for the reboot to complete. If the instance doesn't reboot within the timeout, the service marks the reboot job as failed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reboot_timeout_in_mins: Option<i64>,
}

/// Required fields for ScheduledJobOperation
pub struct ScheduledJobOperationRequired {
    /// The type of operation this scheduled job performs.
    pub operation_type: OperationTypes,
}

impl ScheduledJobOperation {
    /// Create a new ScheduledJobOperation with required fields
    pub fn new(required: ScheduledJobOperationRequired) -> Self {
        Self {
            operation_type: required.operation_type,

            package_names: None,

            windows_update_names: None,

            manage_module_streams_details: None,

            switch_module_streams_details: None,

            software_source_ids: None,

            reboot_timeout_in_mins: None,
        }
    }

    /// Set operation_type
    pub fn set_operation_type(mut self, value: OperationTypes) -> Self {
        self.operation_type = value;
        self
    }

    /// Set package_names
    pub fn set_package_names(mut self, value: Option<Vec<String>>) -> Self {
        self.package_names = value;
        self
    }

    /// Set windows_update_names
    pub fn set_windows_update_names(mut self, value: Option<Vec<String>>) -> Self {
        self.windows_update_names = value;
        self
    }

    /// Set manage_module_streams_details
    pub fn set_manage_module_streams_details(
        mut self,
        value: Option<ManageModuleStreamsInScheduledJobDetails>,
    ) -> Self {
        self.manage_module_streams_details = value;
        self
    }

    /// Set switch_module_streams_details
    pub fn set_switch_module_streams_details(mut self, value: Option<ModuleStreamDetails>) -> Self {
        self.switch_module_streams_details = value;
        self
    }

    /// Set software_source_ids
    pub fn set_software_source_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.software_source_ids = value;
        self
    }

    /// Set reboot_timeout_in_mins
    pub fn set_reboot_timeout_in_mins(mut self, value: Option<i64>) -> Self {
        self.reboot_timeout_in_mins = value;
        self
    }

    /// Set package_names (unwraps Option)
    pub fn with_package_names(mut self, value: Vec<String>) -> Self {
        self.package_names = Some(value);
        self
    }

    /// Set windows_update_names (unwraps Option)
    pub fn with_windows_update_names(mut self, value: Vec<String>) -> Self {
        self.windows_update_names = Some(value);
        self
    }

    /// Set manage_module_streams_details (unwraps Option)
    pub fn with_manage_module_streams_details(
        mut self,
        value: ManageModuleStreamsInScheduledJobDetails,
    ) -> Self {
        self.manage_module_streams_details = Some(value);
        self
    }

    /// Set switch_module_streams_details (unwraps Option)
    pub fn with_switch_module_streams_details(mut self, value: ModuleStreamDetails) -> Self {
        self.switch_module_streams_details = Some(value);
        self
    }

    /// Set software_source_ids (unwraps Option)
    pub fn with_software_source_ids(mut self, value: Vec<String>) -> Self {
        self.software_source_ids = Some(value);
        self
    }

    /// Set reboot_timeout_in_mins (unwraps Option)
    pub fn with_reboot_timeout_in_mins(mut self, value: i64) -> Self {
        self.reboot_timeout_in_mins = Some(value);
        self
    }
}
