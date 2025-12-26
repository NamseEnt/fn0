use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCaptureFilterResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CaptureFilter instance.
    pub capture_filter: CaptureFilter,
}


/// Required fields for UpdateCaptureFilterResponse
pub struct UpdateCaptureFilterResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CaptureFilter instance.
    pub capture_filter: CaptureFilter,
}

impl UpdateCaptureFilterResponse {
    /// Create a new UpdateCaptureFilterResponse with required fields
    pub fn new(required: UpdateCaptureFilterResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            capture_filter: required.capture_filter,
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

    /// Set capture_filter
    pub fn set_capture_filter(mut self, value: CaptureFilter) -> Self {
        self.capture_filter = value;
        self
    }
}


