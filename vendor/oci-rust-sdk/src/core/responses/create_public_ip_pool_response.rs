use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePublicIpPoolResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PublicIpPool instance.
    pub public_ip_pool: PublicIpPool,
}

/// Required fields for CreatePublicIpPoolResponse
pub struct CreatePublicIpPoolResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PublicIpPool instance.
    pub public_ip_pool: PublicIpPool,
}

impl CreatePublicIpPoolResponse {
    /// Create a new CreatePublicIpPoolResponse with required fields
    pub fn new(required: CreatePublicIpPoolResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            public_ip_pool: required.public_ip_pool,
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

    /// Set public_ip_pool
    pub fn set_public_ip_pool(mut self, value: PublicIpPool) -> Self {
        self.public_ip_pool = value;
        self
    }
}
