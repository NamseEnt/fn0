use serde::{Deserialize, Serialize};

use super::enums::*;

/// Details for creating an instance from an image or boot volume
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSourceDetails {
    /// The type of boot source
    pub source_type: SourceType,

    /// The OCID of the image used to boot the instance (required when sourceType = Image)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,

    /// The OCID of the boot volume used to boot the instance (required when sourceType = BootVolume)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_id: Option<String>,

    /// The size of the boot volume in GBs (optional, defaults to image size)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_size_in_g_bs: Option<i64>,
}

impl InstanceSourceDetails {
    /// Create a new instance with default source_type (Image)
    pub fn new() -> Self {
        Self {
            source_type: SourceType::Image,
            image_id: None,
            boot_volume_id: None,
            boot_volume_size_in_g_bs: None,
        }
    }

    /// Create source details from an image
    pub fn from_image(image_id: impl Into<String>) -> Self {
        Self {
            source_type: SourceType::Image,
            image_id: Some(image_id.into()),
            boot_volume_id: None,
            boot_volume_size_in_g_bs: None,
        }
    }

    /// Create source details from a boot volume
    pub fn from_boot_volume(boot_volume_id: impl Into<String>) -> Self {
        Self {
            source_type: SourceType::BootVolume,
            image_id: None,
            boot_volume_id: Some(boot_volume_id.into()),
            boot_volume_size_in_g_bs: None,
        }
    }

    /// Set the boot volume size
    pub fn with_boot_volume_size(mut self, size_in_gbs: i64) -> Self {
        self.boot_volume_size_in_g_bs = Some(size_in_gbs);
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, source_type: SourceType) -> Self {
        self.source_type = source_type;
        self
    }

    /// Set image_id
    pub fn set_image_id(mut self, image_id: Option<String>) -> Self {
        self.image_id = image_id;
        self
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, boot_volume_id: Option<String>) -> Self {
        self.boot_volume_id = boot_volume_id;
        self
    }

    /// Set boot_volume_size_in_g_bs
    pub fn set_boot_volume_size_in_g_bs(mut self, boot_volume_size_in_g_bs: Option<i64>) -> Self {
        self.boot_volume_size_in_g_bs = boot_volume_size_in_g_bs;
        self
    }
}

impl Default for InstanceSourceDetails {
    fn default() -> Self {
        Self::new()
    }
}
