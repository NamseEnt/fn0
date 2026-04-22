use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVolumeGroupCompartmentRequest {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,

    /// Request to change the compartment of given volume group.
    pub change_volume_group_compartment_details: ChangeVolumeGroupCompartmentDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for ChangeVolumeGroupCompartmentRequest
pub struct ChangeVolumeGroupCompartmentRequestRequired {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,

    /// Request to change the compartment of given volume group.
    pub change_volume_group_compartment_details: ChangeVolumeGroupCompartmentDetails,
}

impl ChangeVolumeGroupCompartmentRequest {
    /// Create a new ChangeVolumeGroupCompartmentRequest with required fields
    pub fn new(required: ChangeVolumeGroupCompartmentRequestRequired) -> Self {
        Self {
            volume_group_id: required.volume_group_id,

            change_volume_group_compartment_details: required
                .change_volume_group_compartment_details,

            opc_request_id: None,
        }
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: String) -> Self {
        self.volume_group_id = value;
        self
    }

    /// Set change_volume_group_compartment_details
    pub fn set_change_volume_group_compartment_details(
        mut self,
        value: ChangeVolumeGroupCompartmentDetails,
    ) -> Self {
        self.change_volume_group_compartment_details = value;
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
