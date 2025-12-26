use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6VnicDetachResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.Ipv6 instance.
    pub ipv6: Ipv6,
}

/// Required fields for Ipv6VnicDetachResponse
pub struct Ipv6VnicDetachResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.Ipv6 instance.
    pub ipv6: Ipv6,
}

impl Ipv6VnicDetachResponse {
    /// Create a new Ipv6VnicDetachResponse with required fields
    pub fn new(required: Ipv6VnicDetachResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            ipv6: required.ipv6,
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

    /// Set ipv6
    pub fn set_ipv6(mut self, value: Ipv6) -> Self {
        self.ipv6 = value;
        self
    }
}
