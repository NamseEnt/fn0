use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVolumeCompartmentRequest {
    /// The OCID of the volume.
    pub volume_id: String,

    /// Request to change the compartment of given volume.
    pub change_volume_compartment_details: ChangeVolumeCompartmentDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for ChangeVolumeCompartmentRequest
pub struct ChangeVolumeCompartmentRequestRequired {
    /// The OCID of the volume.
    pub volume_id: String,

    /// Request to change the compartment of given volume.
    pub change_volume_compartment_details: ChangeVolumeCompartmentDetails,
}

impl ChangeVolumeCompartmentRequest {
    /// Create a new ChangeVolumeCompartmentRequest with required fields
    pub fn new(required: ChangeVolumeCompartmentRequestRequired) -> Self {
        Self {
            volume_id: required.volume_id,

            change_volume_compartment_details: required.change_volume_compartment_details,

            opc_request_id: None,
        }
    }

    /// Set volume_id
    pub fn set_volume_id(mut self, value: String) -> Self {
        self.volume_id = value;
        self
    }

    /// Set change_volume_compartment_details
    pub fn set_change_volume_compartment_details(
        mut self,
        value: ChangeVolumeCompartmentDetails,
    ) -> Self {
        self.change_volume_compartment_details = value;
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
