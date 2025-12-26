use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePublicIpRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP.
    pub public_ip_id: String,

    /// Public IP details.
    pub update_public_ip_details: UpdatePublicIpDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdatePublicIpRequest
pub struct UpdatePublicIpRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP.
    pub public_ip_id: String,

    /// Public IP details.
    pub update_public_ip_details: UpdatePublicIpDetails,
}

impl UpdatePublicIpRequest {
    /// Create a new UpdatePublicIpRequest with required fields
    pub fn new(required: UpdatePublicIpRequestRequired) -> Self {
        Self {
            public_ip_id: required.public_ip_id,

            update_public_ip_details: required.update_public_ip_details,

            if_match: None,
}
    }

    /// Set public_ip_id
    pub fn set_public_ip_id(mut self, value: String) -> Self {
        self.public_ip_id = value;
        self
    }

    /// Set update_public_ip_details
    pub fn set_update_public_ip_details(mut self, value: UpdatePublicIpDetails) -> Self {
        self.update_public_ip_details = value;
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


