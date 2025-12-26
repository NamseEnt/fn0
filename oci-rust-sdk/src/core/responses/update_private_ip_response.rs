use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePrivateIpResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PrivateIp instance.
    pub private_ip: PrivateIp,
}


/// Required fields for UpdatePrivateIpResponse
pub struct UpdatePrivateIpResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PrivateIp instance.
    pub private_ip: PrivateIp,
}

impl UpdatePrivateIpResponse {
    /// Create a new UpdatePrivateIpResponse with required fields
    pub fn new(required: UpdatePrivateIpResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            private_ip: required.private_ip,
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

    /// Set private_ip
    pub fn set_private_ip(mut self, value: PrivateIp) -> Self {
        self.private_ip = value;
        self
    }
}


