use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachVnicResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VnicAttachment instance.
    pub vnic_attachment: VnicAttachment,
}


/// Required fields for AttachVnicResponse
pub struct AttachVnicResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VnicAttachment instance.
    pub vnic_attachment: VnicAttachment,
}

impl AttachVnicResponse {
    /// Create a new AttachVnicResponse with required fields
    pub fn new(required: AttachVnicResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            vnic_attachment: required.vnic_attachment,
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

    /// Set vnic_attachment
    pub fn set_vnic_attachment(mut self, value: VnicAttachment) -> Self {
        self.vnic_attachment = value;
        self
    }
}


