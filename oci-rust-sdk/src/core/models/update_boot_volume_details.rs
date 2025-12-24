use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBootVolumeDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The size to resize the volume to in GBs. Has to be larger than the current size. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    /// The number of volume performance units (VPUs) that will be applied to this volume per GB, representing the Block Volume service's elastic performance options. See [Block Volume Performance Levels](https://docs.oracle.com/iaas/Content/Block/Concepts/blockvolumeperformance.htm#perf_levels) for more information. <p> Allowed values: <p> {@code 10}: Represents Balanced option. <p> {@code 20}: Represents Higher Performance option. <p> {@code 30}-{@code 120}: Represents the Ultra High Performance option. <p> For performance autotune enabled volumes, it would be the Default(Minimum) VPUs/GB. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpus_per_g_b: Option<i64>,

    /// Specifies whether the auto-tune performance is enabled for this boot volume. This field is deprecated. Use the {@code DetachedVolumeAutotunePolicy} instead to enable the volume for detached autotune.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_tune_enabled: Option<bool>,

    /// The list of boot volume replicas that this boot volume will be updated to have in the specified destination availability domains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_replicas: Option<Vec<BootVolumeReplicaDetails>>,

    /// The list of autotune policies to be enabled for this volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autotune_policies: Option<Vec<AutotunePolicy>>,
}

impl UpdateBootVolumeDetails {
    /// Create a new UpdateBootVolumeDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            size_in_g_bs: None,

            vpus_per_g_b: None,

            is_auto_tune_enabled: None,

            boot_volume_replicas: None,

            autotune_policies: None,
        }
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set vpus_per_g_b
    pub fn set_vpus_per_g_b(mut self, value: Option<i64>) -> Self {
        self.vpus_per_g_b = value;
        self
    }

    /// Set is_auto_tune_enabled
    pub fn set_is_auto_tune_enabled(mut self, value: Option<bool>) -> Self {
        self.is_auto_tune_enabled = value;
        self
    }

    /// Set boot_volume_replicas
    pub fn set_boot_volume_replicas(
        mut self,
        value: Option<Vec<BootVolumeReplicaDetails>>,
    ) -> Self {
        self.boot_volume_replicas = value;
        self
    }

    /// Set autotune_policies
    pub fn set_autotune_policies(mut self, value: Option<Vec<AutotunePolicy>>) -> Self {
        self.autotune_policies = value;
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set size_in_g_bs (unwraps Option)
    pub fn with_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = Some(value);
        self
    }

    /// Set vpus_per_g_b (unwraps Option)
    pub fn with_vpus_per_g_b(mut self, value: i64) -> Self {
        self.vpus_per_g_b = Some(value);
        self
    }

    /// Set is_auto_tune_enabled (unwraps Option)
    pub fn with_is_auto_tune_enabled(mut self, value: bool) -> Self {
        self.is_auto_tune_enabled = Some(value);
        self
    }

    /// Set boot_volume_replicas (unwraps Option)
    pub fn with_boot_volume_replicas(mut self, value: Vec<BootVolumeReplicaDetails>) -> Self {
        self.boot_volume_replicas = Some(value);
        self
    }

    /// Set autotune_policies (unwraps Option)
    pub fn with_autotune_policies(mut self, value: Vec<AutotunePolicy>) -> Self {
        self.autotune_policies = Some(value);
        self
    }
}

impl Default for UpdateBootVolumeDetails {
    fn default() -> Self {
        Self::new()
    }
}
