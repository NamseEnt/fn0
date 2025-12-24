use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// IP address of the public IP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicIpByIpAddressDetails {
    /// The public IP address. Example: 203.0.113.2
    pub ip_address: String,
}

/// Required fields for GetPublicIpByIpAddressDetails
pub struct GetPublicIpByIpAddressDetailsRequired {
    /// The public IP address. Example: 203.0.113.2
    pub ip_address: String,
}

impl GetPublicIpByIpAddressDetails {
    /// Create a new GetPublicIpByIpAddressDetails with required fields
    pub fn new(required: GetPublicIpByIpAddressDetailsRequired) -> Self {
        Self {
            ip_address: required.ip_address,
        }
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: String) -> Self {
        self.ip_address = value;
        self
    }
}
