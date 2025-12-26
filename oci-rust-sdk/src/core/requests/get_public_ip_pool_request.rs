use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicIpPoolRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP pool.
    pub public_ip_pool_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for GetPublicIpPoolRequest
pub struct GetPublicIpPoolRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP pool.
    pub public_ip_pool_id: String,
}

impl GetPublicIpPoolRequest {
    /// Create a new GetPublicIpPoolRequest with required fields
    pub fn new(required: GetPublicIpPoolRequestRequired) -> Self {
        Self {
            public_ip_pool_id: required.public_ip_pool_id,

            opc_request_id: None,
        }
    }

    /// Set public_ip_pool_id
    pub fn set_public_ip_pool_id(mut self, value: String) -> Self {
        self.public_ip_pool_id = value;
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
