use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A paravirtualized volume attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParavirtualizedVolumeAttachment {
    pub attachment_type: String,
}

/// Required fields for ParavirtualizedVolumeAttachment
pub struct ParavirtualizedVolumeAttachmentRequired {
    pub attachment_type: String,
}

impl ParavirtualizedVolumeAttachment {
    /// Create a new ParavirtualizedVolumeAttachment with required fields
    pub fn new(required: ParavirtualizedVolumeAttachmentRequired) -> Self {
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
