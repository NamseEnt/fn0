use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecurityListRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the security list.
    pub security_list_id: String,

    /// Updated details for the security list.
    pub update_security_list_details: UpdateSecurityListDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateSecurityListRequest
pub struct UpdateSecurityListRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the security list.
    pub security_list_id: String,

    /// Updated details for the security list.
    pub update_security_list_details: UpdateSecurityListDetails,
}

impl UpdateSecurityListRequest {
    /// Create a new UpdateSecurityListRequest with required fields
    pub fn new(required: UpdateSecurityListRequestRequired) -> Self {
        Self {
            security_list_id: required.security_list_id,

            update_security_list_details: required.update_security_list_details,

            if_match: None,
        }
    }

    /// Set security_list_id
    pub fn set_security_list_id(mut self, value: String) -> Self {
        self.security_list_id = value;
        self
    }

    /// Set update_security_list_details
    pub fn set_update_security_list_details(mut self, value: UpdateSecurityListDetails) -> Self {
        self.update_security_list_details = value;
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
