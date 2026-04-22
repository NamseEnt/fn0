use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeBootVolumeBackupCompartmentRequest {
    /// The OCID of the boot volume backup.
    pub boot_volume_backup_id: String,

    /// Request to change the compartment of given boot volume backup.
    pub change_boot_volume_backup_compartment_details: ChangeBootVolumeBackupCompartmentDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for ChangeBootVolumeBackupCompartmentRequest
pub struct ChangeBootVolumeBackupCompartmentRequestRequired {
    /// The OCID of the boot volume backup.
    pub boot_volume_backup_id: String,

    /// Request to change the compartment of given boot volume backup.
    pub change_boot_volume_backup_compartment_details: ChangeBootVolumeBackupCompartmentDetails,
}

impl ChangeBootVolumeBackupCompartmentRequest {
    /// Create a new ChangeBootVolumeBackupCompartmentRequest with required fields
    pub fn new(required: ChangeBootVolumeBackupCompartmentRequestRequired) -> Self {
        Self {
            boot_volume_backup_id: required.boot_volume_backup_id,

            change_boot_volume_backup_compartment_details: required
                .change_boot_volume_backup_compartment_details,

            opc_request_id: None,
        }
    }

    /// Set boot_volume_backup_id
    pub fn set_boot_volume_backup_id(mut self, value: String) -> Self {
        self.boot_volume_backup_id = value;
        self
    }

    /// Set change_boot_volume_backup_compartment_details
    pub fn set_change_boot_volume_backup_compartment_details(
        mut self,
        value: ChangeBootVolumeBackupCompartmentDetails,
    ) -> Self {
        self.change_boot_volume_backup_compartment_details = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}
