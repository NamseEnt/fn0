use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRemotePeeringConnectionRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the remote peering connection (RPC).
    pub remote_peering_connection_id: String,

    /// Request to the update the peering connection to remote region
    pub update_remote_peering_connection_details: UpdateRemotePeeringConnectionDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateRemotePeeringConnectionRequest
pub struct UpdateRemotePeeringConnectionRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the remote peering connection (RPC).
    pub remote_peering_connection_id: String,

    /// Request to the update the peering connection to remote region
    pub update_remote_peering_connection_details: UpdateRemotePeeringConnectionDetails,
}

impl UpdateRemotePeeringConnectionRequest {
    /// Create a new UpdateRemotePeeringConnectionRequest with required fields
    pub fn new(required: UpdateRemotePeeringConnectionRequestRequired) -> Self {
        Self {
            remote_peering_connection_id: required.remote_peering_connection_id,

            update_remote_peering_connection_details: required
                .update_remote_peering_connection_details,

            if_match: None,
        }
    }

    /// Set remote_peering_connection_id
    pub fn set_remote_peering_connection_id(mut self, value: String) -> Self {
        self.remote_peering_connection_id = value;
        self
    }

    /// Set update_remote_peering_connection_details
    pub fn set_update_remote_peering_connection_details(
        mut self,
        value: UpdateRemotePeeringConnectionDetails,
    ) -> Self {
        self.update_remote_peering_connection_details = value;
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
