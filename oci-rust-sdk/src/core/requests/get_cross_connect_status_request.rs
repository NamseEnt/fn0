use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCrossConnectStatusRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,
}


/// Required fields for GetCrossConnectStatusRequest
pub struct GetCrossConnectStatusRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,
}

impl GetCrossConnectStatusRequest {
    /// Create a new GetCrossConnectStatusRequest with required fields
    pub fn new(required: GetCrossConnectStatusRequestRequired) -> Self {
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


