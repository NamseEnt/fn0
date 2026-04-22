use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUpgradeStatusResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.UpgradeStatus instance.
    pub upgrade_status: UpgradeStatus,
}

/// Required fields for GetUpgradeStatusResponse
pub struct GetUpgradeStatusResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.UpgradeStatus instance.
    pub upgrade_status: UpgradeStatus,
}

impl GetUpgradeStatusResponse {
    /// Create a new GetUpgradeStatusResponse with required fields
    pub fn new(required: GetUpgradeStatusResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            upgrade_status: required.upgrade_status,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set upgrade_status
    pub fn set_upgrade_status(mut self, value: UpgradeStatus) -> Self {
        self.upgrade_status = value;
        self
    }
}
