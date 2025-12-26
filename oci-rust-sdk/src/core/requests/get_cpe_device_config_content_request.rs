use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCpeDeviceConfigContentRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE.
    pub cpe_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for GetCpeDeviceConfigContentRequest
pub struct GetCpeDeviceConfigContentRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE.
    pub cpe_id: String,
}

impl GetCpeDeviceConfigContentRequest {
    /// Create a new GetCpeDeviceConfigContentRequest with required fields
    pub fn new(required: GetCpeDeviceConfigContentRequestRequired) -> Self {
        Self {
            cpe_id: required.cpe_id,

            opc_request_id: None,
        }
    }

    /// Set cpe_id
    pub fn set_cpe_id(mut self, value: String) -> Self {
        self.cpe_id = value;
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
