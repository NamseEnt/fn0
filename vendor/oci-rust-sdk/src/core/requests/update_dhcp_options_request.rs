use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDhcpOptionsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the set of DHCP options.
    pub dhcp_id: String,

    /// Request object for updating a set of DHCP options.
    pub update_dhcp_details: UpdateDhcpDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateDhcpOptionsRequest
pub struct UpdateDhcpOptionsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the set of DHCP options.
    pub dhcp_id: String,

    /// Request object for updating a set of DHCP options.
    pub update_dhcp_details: UpdateDhcpDetails,
}

impl UpdateDhcpOptionsRequest {
    /// Create a new UpdateDhcpOptionsRequest with required fields
    pub fn new(required: UpdateDhcpOptionsRequestRequired) -> Self {
        Self {
            dhcp_id: required.dhcp_id,

            update_dhcp_details: required.update_dhcp_details,

            if_match: None,
        }
    }

    /// Set dhcp_id
    pub fn set_dhcp_id(mut self, value: String) -> Self {
        self.dhcp_id = value;
        self
    }

    /// Set update_dhcp_details
    pub fn set_update_dhcp_details(mut self, value: UpdateDhcpDetails) -> Self {
        self.update_dhcp_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}
