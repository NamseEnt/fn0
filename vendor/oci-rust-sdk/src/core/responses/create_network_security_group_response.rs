use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNetworkSecurityGroupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.NetworkSecurityGroup instance.
    pub network_security_group: NetworkSecurityGroup,
}

/// Required fields for CreateNetworkSecurityGroupResponse
pub struct CreateNetworkSecurityGroupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.NetworkSecurityGroup instance.
    pub network_security_group: NetworkSecurityGroup,
}

impl CreateNetworkSecurityGroupResponse {
    /// Create a new CreateNetworkSecurityGroupResponse with required fields
    pub fn new(required: CreateNetworkSecurityGroupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            network_security_group: required.network_security_group,
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

    /// Set network_security_group
    pub fn set_network_security_group(mut self, value: NetworkSecurityGroup) -> Self {
        self.network_security_group = value;
        self
    }
}
