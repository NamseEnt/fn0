use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details of the private IP that the public IP is assigned to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicIpByPrivateIpIdDetails {
    /// [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the private IP.
    pub private_ip_id: String,
}

/// Required fields for GetPublicIpByPrivateIpIdDetails
pub struct GetPublicIpByPrivateIpIdDetailsRequired {
    /// [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the private IP.
    pub private_ip_id: String,
}

impl GetPublicIpByPrivateIpIdDetails {
    /// Create a new GetPublicIpByPrivateIpIdDetails with required fields
    pub fn new(required: GetPublicIpByPrivateIpIdDetailsRequired) -> Self {
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
