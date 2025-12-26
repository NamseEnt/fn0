use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIPSecConnectionRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,

    /// Details object for updating an IPSec connection.
    pub update_ipsec_connection_details: UpdateIPSecConnectionDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateIPSecConnectionRequest
pub struct UpdateIPSecConnectionRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,

    /// Details object for updating an IPSec connection.
    pub update_ipsec_connection_details: UpdateIPSecConnectionDetails,
}

impl UpdateIPSecConnectionRequest {
    /// Create a new UpdateIPSecConnectionRequest with required fields
    pub fn new(required: UpdateIPSecConnectionRequestRequired) -> Self {
        Self {
            ipsc_id: required.ipsc_id,

            update_ipsec_connection_details: required.update_ipsec_connection_details,

            if_match: None,
}
    }

    /// Set ipsc_id
    pub fn set_ipsc_id(mut self, value: String) -> Self {
        self.ipsc_id = value;
        self
    }

    /// Set update_ipsec_connection_details
    pub fn set_update_ipsec_connection_details(mut self, value: UpdateIPSecConnectionDetails) -> Self {
        self.update_ipsec_connection_details = value;
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


