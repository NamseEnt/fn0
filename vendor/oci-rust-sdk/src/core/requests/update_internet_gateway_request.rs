use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInternetGatewayRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the internet gateway.
    pub ig_id: String,

    /// Details for updating the internet gateway.
    pub update_internet_gateway_details: UpdateInternetGatewayDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateInternetGatewayRequest
pub struct UpdateInternetGatewayRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the internet gateway.
    pub ig_id: String,

    /// Details for updating the internet gateway.
    pub update_internet_gateway_details: UpdateInternetGatewayDetails,
}

impl UpdateInternetGatewayRequest {
    /// Create a new UpdateInternetGatewayRequest with required fields
    pub fn new(required: UpdateInternetGatewayRequestRequired) -> Self {
        Self {
            ig_id: required.ig_id,

            update_internet_gateway_details: required.update_internet_gateway_details,

            if_match: None,
        }
    }

    /// Set ig_id
    pub fn set_ig_id(mut self, value: String) -> Self {
        self.ig_id = value;
        self
    }

    /// Set update_internet_gateway_details
    pub fn set_update_internet_gateway_details(
        mut self,
        value: UpdateInternetGatewayDetails,
    ) -> Self {
        self.update_internet_gateway_details = value;
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
