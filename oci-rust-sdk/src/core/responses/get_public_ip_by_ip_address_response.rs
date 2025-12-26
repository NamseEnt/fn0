use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicIpByIpAddressResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PublicIp instance.
    pub public_ip: PublicIp,
}

/// Required fields for GetPublicIpByIpAddressResponse
pub struct GetPublicIpByIpAddressResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PublicIp instance.
    pub public_ip: PublicIp,
}

impl GetPublicIpByIpAddressResponse {
    /// Create a new GetPublicIpByIpAddressResponse with required fields
    pub fn new(required: GetPublicIpByIpAddressResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            public_ip: required.public_ip,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set public_ip
    pub fn set_public_ip(mut self, value: PublicIp) -> Self {
        self.public_ip = value;
        self
    }
}
