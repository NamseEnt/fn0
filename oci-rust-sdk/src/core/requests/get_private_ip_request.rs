use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPrivateIpRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the private IP or IPv6.
    pub private_ip_id: String,
}


/// Required fields for GetPrivateIpRequest
pub struct GetPrivateIpRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the private IP or IPv6.
    pub private_ip_id: String,
}

impl GetPrivateIpRequest {
    /// Create a new GetPrivateIpRequest with required fields
    pub fn new(required: GetPrivateIpRequestRequired) -> Self {
        Self {
            private_ip_id: required.private_ip_id,
}
    }

    /// Set private_ip_id
    pub fn set_private_ip_id(mut self, value: String) -> Self {
        self.private_ip_id = value;
        self
    }
}


