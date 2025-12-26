use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details for providing placement constraints using the compute bare metal host OCID. This placement constraint is only applicable during the launch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeBareMetalHostPlacementConstraintDetails {
    /// The OCID of the compute bare metal host. This is only available for dedicated capacity customers.
    pub compute_bare_metal_host_id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for ComputeBareMetalHostPlacementConstraintDetails
pub struct ComputeBareMetalHostPlacementConstraintDetailsRequired {
    /// The OCID of the compute bare metal host. This is only available for dedicated capacity customers.
    pub compute_bare_metal_host_id: String,

    pub r#type: String,
}

impl ComputeBareMetalHostPlacementConstraintDetails {
    /// Create a new ComputeBareMetalHostPlacementConstraintDetails with required fields
    pub fn new(required: ComputeBareMetalHostPlacementConstraintDetailsRequired) -> Self {
        Self {
            compute_bare_metal_host_id: required.compute_bare_metal_host_id,

            r#type: required.r#type,
        }
    }

    /// Set compute_bare_metal_host_id
    pub fn set_compute_bare_metal_host_id(mut self, value: String) -> Self {
        self.compute_bare_metal_host_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
