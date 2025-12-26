use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeBootVolumeCompartmentRequest {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// Request to change the compartment of given boot volume.
    pub change_boot_volume_compartment_details: ChangeBootVolumeCompartmentDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for ChangeBootVolumeCompartmentRequest
pub struct ChangeBootVolumeCompartmentRequestRequired {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// Request to change the compartment of given boot volume.
    pub change_boot_volume_compartment_details: ChangeBootVolumeCompartmentDetails,
}

impl ChangeBootVolumeCompartmentRequest {
    /// Create a new ChangeBootVolumeCompartmentRequest with required fields
    pub fn new(required: ChangeBootVolumeCompartmentRequestRequired) -> Self {
        Self {
            boot_volume_id: required.boot_volume_id,

            change_boot_volume_compartment_details: required.change_boot_volume_compartment_details,

            opc_request_id: None,
        }
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: String) -> Self {
        self.boot_volume_id = value;
        self
    }

    /// Set change_boot_volume_compartment_details
    pub fn set_change_boot_volume_compartment_details(
        mut self,
        value: ChangeBootVolumeCompartmentDetails,
    ) -> Self {
        self.change_boot_volume_compartment_details = value;
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
