use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVolumeBackupCompartmentRequest {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,

    /// Request to change the compartment of given volume backup.
    pub change_volume_backup_compartment_details: ChangeVolumeBackupCompartmentDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for ChangeVolumeBackupCompartmentRequest
pub struct ChangeVolumeBackupCompartmentRequestRequired {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,

    /// Request to change the compartment of given volume backup.
    pub change_volume_backup_compartment_details: ChangeVolumeBackupCompartmentDetails,
}

impl ChangeVolumeBackupCompartmentRequest {
    /// Create a new ChangeVolumeBackupCompartmentRequest with required fields
    pub fn new(required: ChangeVolumeBackupCompartmentRequestRequired) -> Self {
        Self {
            volume_backup_id: required.volume_backup_id,

            change_volume_backup_compartment_details: required
                .change_volume_backup_compartment_details,

            opc_request_id: None,
        }
    }

    /// Set volume_backup_id
    pub fn set_volume_backup_id(mut self, value: String) -> Self {
        self.volume_backup_id = value;
        self
    }

    /// Set change_volume_backup_compartment_details
    pub fn set_change_volume_backup_compartment_details(
        mut self,
        value: ChangeVolumeBackupCompartmentDetails,
    ) -> Self {
        self.change_volume_backup_compartment_details = value;
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
