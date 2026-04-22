use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCrossConnectGroupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnectGroup instance.
    pub cross_connect_group: CrossConnectGroup,
}

/// Required fields for UpdateCrossConnectGroupResponse
pub struct UpdateCrossConnectGroupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnectGroup instance.
    pub cross_connect_group: CrossConnectGroup,
}

impl UpdateCrossConnectGroupResponse {
    /// Create a new UpdateCrossConnectGroupResponse with required fields
    pub fn new(required: UpdateCrossConnectGroupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            cross_connect_group: required.cross_connect_group,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set cross_connect_group
    pub fn set_cross_connect_group(mut self, value: CrossConnectGroup) -> Self {
        self.cross_connect_group = value;
        self
    }
}
