use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUpgradeStatusRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetUpgradeStatusRequest
pub struct GetUpgradeStatusRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,
}

impl GetUpgradeStatusRequest {
    /// Create a new GetUpgradeStatusRequest with required fields
    pub fn new(required: GetUpgradeStatusRequestRequired) -> Self {
        Self {
            drg_id: required.drg_id,

            opc_request_id: None,
}
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
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


