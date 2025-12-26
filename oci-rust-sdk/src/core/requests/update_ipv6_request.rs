use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIpv6Request {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPv6.
    pub ipv6_id: String,

    /// IPv6 details to be updated.
    pub update_ipv6_details: UpdateIpv6Details,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for UpdateIpv6Request
pub struct UpdateIpv6RequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPv6.
    pub ipv6_id: String,

    /// IPv6 details to be updated.
    pub update_ipv6_details: UpdateIpv6Details,
}

impl UpdateIpv6Request {
    /// Create a new UpdateIpv6Request with required fields
    pub fn new(required: UpdateIpv6RequestRequired) -> Self {
        Self {
            ipv6_id: required.ipv6_id,

            update_ipv6_details: required.update_ipv6_details,

            if_match: None,

            opc_request_id: None,
        }
    }

    /// Set ipv6_id
    pub fn set_ipv6_id(mut self, value: String) -> Self {
        self.ipv6_id = value;
        self
    }

    /// Set update_ipv6_details
    pub fn set_update_ipv6_details(mut self, value: UpdateIpv6Details) -> Self {
        self.update_ipv6_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}
