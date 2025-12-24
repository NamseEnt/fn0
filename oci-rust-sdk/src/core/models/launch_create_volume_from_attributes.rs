use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details of the volume to create for CreateVolume operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchCreateVolumeFromAttributes {
    /// The size of the volume in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_g_bs: i64,

    pub volume_creation_type: String,

    /// The OCID of the compartment that contains the volume. If not provided, it will be inherited from the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The OCID of the Vault service key to assign as the master encryption key for the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// The number of volume performance units (VPUs) that will be applied to this volume per GB, representing the Block Volume service's elastic performance options. See [Block Volume Performance Levels](https://docs.oracle.com/iaas/Content/Block/Concepts/blockvolumeperformance.htm#perf_levels) for more information. <p> Allowed values: <p> {@code 0}: Represents Lower Cost option. <p> {@code 10}: Represents Balanced option. <p> {@code 20}: Represents Higher Performance option. <p> {@code 30}-{@code 120}: Represents the Ultra High Performance option. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpus_per_g_b: Option<i64>,
}

/// Required fields for LaunchCreateVolumeFromAttributes
pub struct LaunchCreateVolumeFromAttributesRequired {
    /// The size of the volume in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_g_bs: i64,

    pub volume_creation_type: String,
}

impl LaunchCreateVolumeFromAttributes {
    /// Create a new LaunchCreateVolumeFromAttributes with required fields
    pub fn new(required: LaunchCreateVolumeFromAttributesRequired) -> Self {
        Self {
            size_in_g_bs: required.size_in_g_bs,

            volume_creation_type: required.volume_creation_type,

            compartment_id: None,

            display_name: None,

            kms_key_id: None,

            vpus_per_g_b: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set vpus_per_g_b
    pub fn set_vpus_per_g_b(mut self, value: Option<i64>) -> Self {
        self.vpus_per_g_b = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set volume_creation_type
    pub fn set_volume_creation_type(mut self, value: String) -> Self {
        self.volume_creation_type = value;
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }

    /// Set vpus_per_g_b (unwraps Option)
    pub fn with_vpus_per_g_b(mut self, value: i64) -> Self {
        self.vpus_per_g_b = Some(value);
        self
    }
}
