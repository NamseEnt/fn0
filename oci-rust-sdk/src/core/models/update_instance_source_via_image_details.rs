use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details for updating the instance source from an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceSourceViaImageDetails {
    /// The OCID of the image used to boot the instance.
    pub image_id: String,

    pub source_type: String,

    /// The size of the boot volume in GBs. Minimum value is 50 GB and maximum value is 32,768 GB (32 TB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_size_in_g_bs: Option<i64>,

    /// The OCID of the Vault service key to assign as the master encryption key for the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,
}

/// Required fields for UpdateInstanceSourceViaImageDetails
pub struct UpdateInstanceSourceViaImageDetailsRequired {
    /// The OCID of the image used to boot the instance.
    pub image_id: String,

    pub source_type: String,
}

impl UpdateInstanceSourceViaImageDetails {
    /// Create a new UpdateInstanceSourceViaImageDetails with required fields
    pub fn new(required: UpdateInstanceSourceViaImageDetailsRequired) -> Self {
        Self {
            image_id: required.image_id,

            source_type: required.source_type,

            boot_volume_size_in_g_bs: None,

            kms_key_id: None,
        }
    }

    /// Set boot_volume_size_in_g_bs
    pub fn set_boot_volume_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.boot_volume_size_in_g_bs = value;
        self
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: String) -> Self {
        self.image_id = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }

    /// Set boot_volume_size_in_g_bs (unwraps Option)
    pub fn with_boot_volume_size_in_g_bs(mut self, value: i64) -> Self {
        self.boot_volume_size_in_g_bs = Some(value);
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }
}
