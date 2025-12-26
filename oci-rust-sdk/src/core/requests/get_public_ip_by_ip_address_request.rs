use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicIpByIpAddressRequest {
    /// IP address details for fetching the public IP.
    pub get_public_ip_by_ip_address_details: GetPublicIpByIpAddressDetails,
}


/// Required fields for GetPublicIpByIpAddressRequest
pub struct GetPublicIpByIpAddressRequestRequired {
    /// IP address details for fetching the public IP.
    pub get_public_ip_by_ip_address_details: GetPublicIpByIpAddressDetails,
}

impl GetPublicIpByIpAddressRequest {
    /// Create a new GetPublicIpByIpAddressRequest with required fields
    pub fn new(required: GetPublicIpByIpAddressRequestRequired) -> Self {
        Self {
            get_public_ip_by_ip_address_details: required.get_public_ip_by_ip_address_details,
}
    }

    /// Set get_public_ip_by_ip_address_details
    pub fn set_get_public_ip_by_ip_address_details(mut self, value: GetPublicIpByIpAddressDetails) -> Self {
        self.get_public_ip_by_ip_address_details = value;
        self
    }
}


