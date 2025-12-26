use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateByoipRangeResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ByoipRange instance.
    pub byoip_range: ByoipRange,
}

/// Required fields for UpdateByoipRangeResponse
pub struct UpdateByoipRangeResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ByoipRange instance.
    pub byoip_range: ByoipRange,
}

impl UpdateByoipRangeResponse {
    /// Create a new UpdateByoipRangeResponse with required fields
    pub fn new(required: UpdateByoipRangeResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            byoip_range: required.byoip_range,
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

    /// Set byoip_range
    pub fn set_byoip_range(mut self, value: ByoipRange) -> Self {
        self.byoip_range = value;
        self
    }
}
