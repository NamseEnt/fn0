use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeAttachmentRequest {
    /// The OCID of the volume attachment.
    pub volume_attachment_id: String,
}

/// Required fields for GetVolumeAttachmentRequest
pub struct GetVolumeAttachmentRequestRequired {
    /// The OCID of the volume attachment.
    pub volume_attachment_id: String,
}

impl GetVolumeAttachmentRequest {
    /// Create a new GetVolumeAttachmentRequest with required fields
    pub fn new(required: GetVolumeAttachmentRequestRequired) -> Self {
        Self {
            volume_attachment_id: required.volume_attachment_id,
        }
    }

    /// Set volume_attachment_id
    pub fn set_volume_attachment_id(mut self, value: String) -> Self {
        self.volume_attachment_id = value;
        self
    }
}
