use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An Emulated volume attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatedVolumeAttachment {
    pub attachment_type: String,
}

/// Required fields for EmulatedVolumeAttachment
pub struct EmulatedVolumeAttachmentRequired {
    pub attachment_type: String,
}

impl EmulatedVolumeAttachment {
    /// Create a new EmulatedVolumeAttachment with required fields
    pub fn new(required: EmulatedVolumeAttachmentRequired) -> Self {
        Self {
            attachment_type: required.attachment_type,
        }
    }

    /// Set attachment_type
    pub fn set_attachment_type(mut self, value: String) -> Self {
        self.attachment_type = value;
        self
    }
}
