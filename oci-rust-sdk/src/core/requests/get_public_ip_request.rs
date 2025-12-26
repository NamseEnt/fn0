use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicIpRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP.
    pub public_ip_id: String,
}


/// Required fields for GetPublicIpRequest
pub struct GetPublicIpRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP.
    pub public_ip_id: String,
}

impl GetPublicIpRequest {
    /// Create a new GetPublicIpRequest with required fields
    pub fn new(required: GetPublicIpRequestRequired) -> Self {
        Self {
            public_ip_id: required.public_ip_id,
}
    }

    /// Set public_ip_id
    pub fn set_public_ip_id(mut self, value: String) -> Self {
        self.public_ip_id = value;
        self
    }
}


