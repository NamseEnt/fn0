use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVnicAttachmentRequest {
    /// The OCID of the VNIC attachment.
    pub vnic_attachment_id: String,
}

/// Required fields for GetVnicAttachmentRequest
pub struct GetVnicAttachmentRequestRequired {
    /// The OCID of the VNIC attachment.
    pub vnic_attachment_id: String,
}

impl GetVnicAttachmentRequest {
    /// Create a new GetVnicAttachmentRequest with required fields
    pub fn new(required: GetVnicAttachmentRequestRequired) -> Self {
        Self {
            vnic_attachment_id: required.vnic_attachment_id,
        }
    }

    /// Set vnic_attachment_id
    pub fn set_vnic_attachment_id(mut self, value: String) -> Self {
        self.vnic_attachment_id = value;
        self
    }
}
