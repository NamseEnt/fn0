use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgAttachmentResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgAttachment instance.
    pub drg_attachment: DrgAttachment,
}


/// Required fields for UpdateDrgAttachmentResponse
pub struct UpdateDrgAttachmentResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgAttachment instance.
    pub drg_attachment: DrgAttachment,
}

impl UpdateDrgAttachmentResponse {
    /// Create a new UpdateDrgAttachmentResponse with required fields
    pub fn new(required: UpdateDrgAttachmentResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            drg_attachment: required.drg_attachment,
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

    /// Set drg_attachment
    pub fn set_drg_attachment(mut self, value: DrgAttachment) -> Self {
        self.drg_attachment = value;
        self
    }
}


