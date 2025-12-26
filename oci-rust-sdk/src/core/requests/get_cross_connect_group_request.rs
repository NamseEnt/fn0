use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCrossConnectGroupRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect group.
    pub cross_connect_group_id: String,
}

/// Required fields for GetCrossConnectGroupRequest
pub struct GetCrossConnectGroupRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect group.
    pub cross_connect_group_id: String,
}

impl GetCrossConnectGroupRequest {
    /// Create a new GetCrossConnectGroupRequest with required fields
    pub fn new(required: GetCrossConnectGroupRequestRequired) -> Self {
        Self {
            cross_connect_group_id: required.cross_connect_group_id,
        }
    }

    /// Set cross_connect_group_id
    pub fn set_cross_connect_group_id(mut self, value: String) -> Self {
        self.cross_connect_group_id = value;
        self
    }
}
