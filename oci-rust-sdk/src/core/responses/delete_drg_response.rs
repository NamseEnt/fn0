use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDrgResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}


/// Required fields for DeleteDrgResponse
pub struct DeleteDrgResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}

impl DeleteDrgResponse {
    /// Create a new DeleteDrgResponse with required fields
    pub fn new(required: DeleteDrgResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }
}


