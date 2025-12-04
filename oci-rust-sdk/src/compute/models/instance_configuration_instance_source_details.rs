use serde::{Deserialize, Serialize};

/// Instance source details (polymorphic based on sourceType).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "sourceType")]
pub enum InstanceConfigurationInstanceSourceDetails {
    /// Boot from an image.
    #[serde(rename = "image")]
    Image(InstanceConfigurationInstanceSourceViaImageDetails),

    /// Boot from a boot volume.
    #[serde(rename = "bootVolume")]
    BootVolume(InstanceConfigurationInstanceSourceViaBootVolumeDetails),
}

/// Instance source via image details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceSourceViaImageDetails {
    /// The size of the boot volume in GBs (50 GB to 32,768 GB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_size_in_g_bs: Option<i64>,

    /// The OCID of the image used to boot the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,

    /// The OCID of the Vault service key for the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// The number of volume performance units (VPUs) per GB (10, 20, or 30-120).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_vpus_per_gb: Option<i64>,

    /// Instance source image filter details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_source_image_filter_details: Option<super::InstanceConfigurationInstanceSourceImageFilterDetails>,
}

/// Instance source via boot volume details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceSourceViaBootVolumeDetails {
    /// The OCID of the boot volume used to boot the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_id: Option<String>,
}

impl InstanceConfigurationInstanceSourceDetails {
    /// Creates a new Image variant.
    pub fn from_image(image_id: impl Into<String>) -> Self {
        Self::Image(InstanceConfigurationInstanceSourceViaImageDetails {
            boot_volume_size_in_g_bs: None,
            image_id: Some(image_id.into()),
            kms_key_id: None,
            boot_volume_vpus_per_gb: None,
            instance_source_image_filter_details: None,
        })
    }

    /// Creates a new BootVolume variant.
    pub fn from_boot_volume(boot_volume_id: impl Into<String>) -> Self {
        Self::BootVolume(InstanceConfigurationInstanceSourceViaBootVolumeDetails {
            boot_volume_id: Some(boot_volume_id.into()),
        })
    }
}

impl InstanceConfigurationInstanceSourceViaImageDetails {
    pub fn new() -> Self {
        Self {
            boot_volume_size_in_g_bs: None,
            image_id: None,
            kms_key_id: None,
            boot_volume_vpus_per_gb: None,
            instance_source_image_filter_details: None,
        }
    }

    pub fn with_image_id(mut self, image_id: impl Into<String>) -> Self {
        self.image_id = Some(image_id.into());
        self
    }

    pub fn with_boot_volume_size_in_gbs(mut self, size: i64) -> Self {
        self.boot_volume_size_in_g_bs = Some(size);
        self
    }

    pub fn with_kms_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.kms_key_id = Some(key_id.into());
        self
    }

    pub fn with_boot_volume_vpus_per_gb(mut self, vpus: i64) -> Self {
        self.boot_volume_vpus_per_gb = Some(vpus);
        self
    }
}

impl Default for InstanceConfigurationInstanceSourceViaImageDetails {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceConfigurationInstanceSourceViaBootVolumeDetails {
    pub fn new() -> Self {
        Self {
            boot_volume_id: None,
        }
    }

    pub fn with_boot_volume_id(mut self, boot_volume_id: impl Into<String>) -> Self {
        self.boot_volume_id = Some(boot_volume_id.into());
        self
    }
}

impl Default for InstanceConfigurationInstanceSourceViaBootVolumeDetails {
    fn default() -> Self {
        Self::new()
    }
}
