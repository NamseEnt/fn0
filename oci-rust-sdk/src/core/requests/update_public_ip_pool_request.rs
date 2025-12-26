use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePublicIpPoolRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP pool.
    pub public_ip_pool_id: String,

    /// Public IP pool details.
    pub update_public_ip_pool_details: UpdatePublicIpPoolDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdatePublicIpPoolRequest
pub struct UpdatePublicIpPoolRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP pool.
    pub public_ip_pool_id: String,

    /// Public IP pool details.
    pub update_public_ip_pool_details: UpdatePublicIpPoolDetails,
}

impl UpdatePublicIpPoolRequest {
    /// Create a new UpdatePublicIpPoolRequest with required fields
    pub fn new(required: UpdatePublicIpPoolRequestRequired) -> Self {
        Self {
            public_ip_pool_id: required.public_ip_pool_id,

            update_public_ip_pool_details: required.update_public_ip_pool_details,

            opc_request_id: None,

            if_match: None,
}
    }

    /// Set public_ip_pool_id
    pub fn set_public_ip_pool_id(mut self, value: String) -> Self {
        self.public_ip_pool_id = value;
        self
    }

    /// Set update_public_ip_pool_details
    pub fn set_update_public_ip_pool_details(mut self, value: UpdatePublicIpPoolDetails) -> Self {
        self.update_public_ip_pool_details = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


