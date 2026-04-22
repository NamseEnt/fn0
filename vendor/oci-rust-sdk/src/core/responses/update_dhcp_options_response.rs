use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDhcpOptionsResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DhcpOptions instance.
    pub dhcp_options: DhcpOptions,
}

/// Required fields for UpdateDhcpOptionsResponse
pub struct UpdateDhcpOptionsResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DhcpOptions instance.
    pub dhcp_options: DhcpOptions,
}

impl UpdateDhcpOptionsResponse {
    /// Create a new UpdateDhcpOptionsResponse with required fields
    pub fn new(required: UpdateDhcpOptionsResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            dhcp_options: required.dhcp_options,
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

    /// Set dhcp_options
    pub fn set_dhcp_options(mut self, value: DhcpOptions) -> Self {
        self.dhcp_options = value;
        self
    }
}
