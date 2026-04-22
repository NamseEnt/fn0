use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInternetGatewayRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the internet gateway.
    pub ig_id: String,
}

/// Required fields for GetInternetGatewayRequest
pub struct GetInternetGatewayRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the internet gateway.
    pub ig_id: String,
}

impl GetInternetGatewayRequest {
    /// Create a new GetInternetGatewayRequest with required fields
    pub fn new(required: GetInternetGatewayRequestRequired) -> Self {
        Self {
            ig_id: required.ig_id,
        }
    }

    /// Set ig_id
    pub fn set_ig_id(mut self, value: String) -> Self {
        self.ig_id = value;
        self
    }
}
