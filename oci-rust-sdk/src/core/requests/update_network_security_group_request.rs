use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNetworkSecurityGroupRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Details object for updating a network security group.
    pub update_network_security_group_details: UpdateNetworkSecurityGroupDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateNetworkSecurityGroupRequest
pub struct UpdateNetworkSecurityGroupRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Details object for updating a network security group.
    pub update_network_security_group_details: UpdateNetworkSecurityGroupDetails,
}

impl UpdateNetworkSecurityGroupRequest {
    /// Create a new UpdateNetworkSecurityGroupRequest with required fields
    pub fn new(required: UpdateNetworkSecurityGroupRequestRequired) -> Self {
        Self {
            network_security_group_id: required.network_security_group_id,

            update_network_security_group_details: required.update_network_security_group_details,

            if_match: None,
        }
    }

    /// Set network_security_group_id
    pub fn set_network_security_group_id(mut self, value: String) -> Self {
        self.network_security_group_id = value;
        self
    }

    /// Set update_network_security_group_details
    pub fn set_update_network_security_group_details(
        mut self,
        value: UpdateNetworkSecurityGroupDetails,
    ) -> Self {
        self.update_network_security_group_details = value;
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
