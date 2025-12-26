use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBootVolumeAttachmentRequest {
    /// The OCID of the boot volume attachment.
    pub boot_volume_attachment_id: String,
}


/// Required fields for GetBootVolumeAttachmentRequest
pub struct GetBootVolumeAttachmentRequestRequired {
    /// The OCID of the boot volume attachment.
    pub boot_volume_attachment_id: String,
}

impl GetBootVolumeAttachmentRequest {
    /// Create a new GetBootVolumeAttachmentRequest with required fields
    pub fn new(required: GetBootVolumeAttachmentRequestRequired) -> Self {
        Self {
            boot_volume_attachment_id: required.boot_volume_attachment_id,
}
    }

    /// Set boot_volume_attachment_id
    pub fn set_boot_volume_attachment_id(mut self, value: String) -> Self {
        self.boot_volume_attachment_id = value;
        self
    }
}


