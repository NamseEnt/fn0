use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachVolumeResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeAttachment instance.
    pub volume_attachment: VolumeAttachment,
}


/// Required fields for AttachVolumeResponse
pub struct AttachVolumeResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeAttachment instance.
    pub volume_attachment: VolumeAttachment,
}

impl AttachVolumeResponse {
    /// Create a new AttachVolumeResponse with required fields
    pub fn new(required: AttachVolumeResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            volume_attachment: required.volume_attachment,
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

    /// Set volume_attachment
    pub fn set_volume_attachment(mut self, value: VolumeAttachment) -> Self {
        self.volume_attachment = value;
        self
    }
}


