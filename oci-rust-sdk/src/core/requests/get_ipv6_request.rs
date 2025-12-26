use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIpv6Request {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPv6.
    pub ipv6_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for GetIpv6Request
pub struct GetIpv6RequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPv6.
    pub ipv6_id: String,
}

impl GetIpv6Request {
    /// Create a new GetIpv6Request with required fields
    pub fn new(required: GetIpv6RequestRequired) -> Self {
        Self {
            ipv6_id: required.ipv6_id,

            opc_request_id: None,
        }
    }

    /// Set ipv6_id
    pub fn set_ipv6_id(mut self, value: String) -> Self {
        self.ipv6_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}
