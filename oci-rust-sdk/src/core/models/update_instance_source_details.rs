use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details for updating the instance source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceSourceDetails {
    pub source_type: String,

    /// Whether to preserve the boot volume that was previously attached to the instance after a successful replacement of that boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_preserve_boot_volume_enabled: Option<bool>,
}

/// Required fields for UpdateInstanceSourceDetails
pub struct UpdateInstanceSourceDetailsRequired {
    pub source_type: String,
}

impl UpdateInstanceSourceDetails {
    /// Create a new UpdateInstanceSourceDetails with required fields
    pub fn new(required: UpdateInstanceSourceDetailsRequired) -> Self {
        Self {
            source_type: required.source_type,

            is_preserve_boot_volume_enabled: None,
        }
    }

    /// Set is_preserve_boot_volume_enabled
    pub fn set_is_preserve_boot_volume_enabled(mut self, value: Option<bool>) -> Self {
        self.is_preserve_boot_volume_enabled = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }

    /// Set is_preserve_boot_volume_enabled (unwraps Option)
    pub fn with_is_preserve_boot_volume_enabled(mut self, value: bool) -> Self {
        self.is_preserve_boot_volume_enabled = Some(value);
        self
    }
}
