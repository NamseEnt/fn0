use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicIpByPrivateIpIdRequest {
    /// Private IP details for fetching the public IP.
    pub get_public_ip_by_private_ip_id_details: GetPublicIpByPrivateIpIdDetails,
}

/// Required fields for GetPublicIpByPrivateIpIdRequest
pub struct GetPublicIpByPrivateIpIdRequestRequired {
    /// Private IP details for fetching the public IP.
    pub get_public_ip_by_private_ip_id_details: GetPublicIpByPrivateIpIdDetails,
}

impl GetPublicIpByPrivateIpIdRequest {
    /// Create a new GetPublicIpByPrivateIpIdRequest with required fields
    pub fn new(required: GetPublicIpByPrivateIpIdRequestRequired) -> Self {
        Self {
            get_public_ip_by_private_ip_id_details: required.get_public_ip_by_private_ip_id_details,
        }
    }

    /// Set get_public_ip_by_private_ip_id_details
    pub fn set_get_public_ip_by_private_ip_id_details(
        mut self,
        value: GetPublicIpByPrivateIpIdDetails,
    ) -> Self {
        self.get_public_ip_by_private_ip_id_details = value;
        self
    }
}
