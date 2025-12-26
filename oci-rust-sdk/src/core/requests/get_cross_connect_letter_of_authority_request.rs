use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCrossConnectLetterOfAuthorityRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,
}

/// Required fields for GetCrossConnectLetterOfAuthorityRequest
pub struct GetCrossConnectLetterOfAuthorityRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,
}

impl GetCrossConnectLetterOfAuthorityRequest {
    /// Create a new GetCrossConnectLetterOfAuthorityRequest with required fields
    pub fn new(required: GetCrossConnectLetterOfAuthorityRequestRequired) -> Self {
        Self {
            cross_connect_id: required.cross_connect_id,
        }
    }

    /// Set cross_connect_id
    pub fn set_cross_connect_id(mut self, value: String) -> Self {
        self.cross_connect_id = value;
        self
    }
}
