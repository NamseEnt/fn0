use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConsoleHistoryContentResponse {
    /// The number of bytes remaining in the snapshot.
    pub opc_bytes_remaining: i64,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned string instance.
    pub value: String,
}

/// Required fields for GetConsoleHistoryContentResponse
pub struct GetConsoleHistoryContentResponseRequired {
    /// The number of bytes remaining in the snapshot.
    pub opc_bytes_remaining: i64,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned string instance.
    pub value: String,
}

impl GetConsoleHistoryContentResponse {
    /// Create a new GetConsoleHistoryContentResponse with required fields
    pub fn new(required: GetConsoleHistoryContentResponseRequired) -> Self {
        Self {
            opc_bytes_remaining: required.opc_bytes_remaining,

            opc_request_id: required.opc_request_id,

            value: required.value,
        }
    }

    /// Set opc_bytes_remaining
    pub fn set_opc_bytes_remaining(mut self, value: i64) -> Self {
        self.opc_bytes_remaining = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set value
    pub fn set_value(mut self, value: String) -> Self {
        self.value = value;
        self
    }
}
