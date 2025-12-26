use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertiseByoipRangeRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource containing the BYOIP CIDR block.
    pub byoip_range_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for AdvertiseByoipRangeRequest
pub struct AdvertiseByoipRangeRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource containing the BYOIP CIDR block.
    pub byoip_range_id: String,
}

impl AdvertiseByoipRangeRequest {
    /// Create a new AdvertiseByoipRangeRequest with required fields
    pub fn new(required: AdvertiseByoipRangeRequestRequired) -> Self {
        Self {
            byoip_range_id: required.byoip_range_id,

            opc_request_id: None,
}
    }

    /// Set byoip_range_id
    pub fn set_byoip_range_id(mut self, value: String) -> Self {
        self.byoip_range_id = value;
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


