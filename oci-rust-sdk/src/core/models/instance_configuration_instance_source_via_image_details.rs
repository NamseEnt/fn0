use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceSourceViaImageDetails {
    pub source_type: String,

    /// The size of the boot volume in GBs. The minimum value is 50 GB and the maximum value is 32,768 GB (32 TB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_size_in_g_bs: Option<i64>,

    /// The OCID of the image used to boot the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,

    /// The OCID of the Vault service key to assign as the master encryption key for the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// The number of volume performance units (VPUs) that will be applied to this volume per GB, representing the Block Volume service's elastic performance options. See [Block Volume Performance Levels](https://docs.oracle.com/iaas/Content/Block/Concepts/blockvolumeperformance.htm#perf_levels) for more information. <p> Allowed values: <p> {@code 10}: Represents Balanced option. <p> {@code 20}: Represents Higher Performance option. <p> {@code 30}-{@code 120}: Represents the Ultra High Performance option. <p> For performance autotune enabled volumes, it would be the Default(Minimum) VPUs/GB. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_vpus_per_g_b: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_source_image_filter_details:
        Option<InstanceConfigurationInstanceSourceImageFilterDetails>,
}

/// Required fields for InstanceConfigurationInstanceSourceViaImageDetails
pub struct InstanceConfigurationInstanceSourceViaImageDetailsRequired {
    pub source_type: String,
}

impl InstanceConfigurationInstanceSourceViaImageDetails {
    /// Create a new InstanceConfigurationInstanceSourceViaImageDetails with required fields
    pub fn new(required: InstanceConfigurationInstanceSourceViaImageDetailsRequired) -> Self {
        Self {
            source_type: required.source_type,

            boot_volume_size_in_g_bs: None,

            image_id: None,

            kms_key_id: None,

            boot_volume_vpus_per_g_b: None,

            instance_source_image_filter_details: None,
        }
    }

    /// Set boot_volume_size_in_g_bs
    pub fn set_boot_volume_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.boot_volume_size_in_g_bs = value;
        self
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: Option<String>) -> Self {
        self.image_id = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set boot_volume_vpus_per_g_b
    pub fn set_boot_volume_vpus_per_g_b(mut self, value: Option<i64>) -> Self {
        self.boot_volume_vpus_per_g_b = value;
        self
    }

    /// Set instance_source_image_filter_details
    pub fn set_instance_source_image_filter_details(
        mut self,
        value: Option<InstanceConfigurationInstanceSourceImageFilterDetails>,
    ) -> Self {
        self.instance_source_image_filter_details = value;
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

    /// Set image_id (unwraps Option)
    pub fn with_image_id(mut self, value: impl Into<String>) -> Self {
        self.image_id = Some(value.into());
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }

    /// Set boot_volume_vpus_per_g_b (unwraps Option)
    pub fn with_boot_volume_vpus_per_g_b(mut self, value: i64) -> Self {
        self.boot_volume_vpus_per_g_b = Some(value);
        self
    }

    /// Set instance_source_image_filter_details (unwraps Option)
    pub fn with_instance_source_image_filter_details(
        mut self,
        value: InstanceConfigurationInstanceSourceImageFilterDetails,
    ) -> Self {
        self.instance_source_image_filter_details = Some(value);
        self
    }
}
