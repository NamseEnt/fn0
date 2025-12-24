use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A detachable boot volume device that contains the image used to boot a Compute instance. For more information, see [Overview of Boot Volumes](https://docs.oracle.com/iaas/Content/Block/Concepts/bootvolumes.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolume {
    /// The availability domain of the boot volume. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the boot volume.
    pub compartment_id: String,

    /// The boot volume's Oracle ID (OCID).
    pub id: String,

    /// The current state of a boot volume.
    pub lifecycle_state: BootVolumeLifecycleState,

    /// The size of the volume in MBs. The value must be a multiple of 1024. This field is deprecated. Please use sizeInGBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_m_bs: i64,

    /// The date and time the boot volume was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The image OCID used to create the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,

    /// Specifies whether the boot volume's data has finished copying from the source boot volume or boot volume backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hydrated: Option<bool>,

    /// The clusterPlacementGroup Id of the volume for volume placement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    /// The number of volume performance units (VPUs) that will be applied to this boot volume per GB, representing the Block Volume service's elastic performance options. See [Block Volume Performance Levels](https://docs.oracle.com/iaas/Content/Block/Concepts/blockvolumeperformance.htm#perf_levels) for more information. <p> Allowed values: <p> {@code 10}: Represents Balanced option. <p> {@code 20}: Represents Higher Performance option. <p> {@code 30}-{@code 120}: Represents the Ultra High Performance option. <p> For performance autotune enabled volumes, it would be the Default(Minimum) VPUs/GB. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpus_per_g_b: Option<i64>,

    /// The size of the boot volume in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<BootVolumeSourceFromBootVolumeBackupDetails>,

    /// The OCID of the source volume group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_id: Option<String>,

    /// The OCID of the Vault service master encryption key assigned to the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// Specifies whether the auto-tune performance is enabled for this boot volume. This field is deprecated. Use the {@code DetachedVolumeAutotunePolicy} instead to enable the volume for detached autotune.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_tune_enabled: Option<bool>,

    /// The number of Volume Performance Units per GB that this boot volume is effectively tuned to. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_tuned_vpus_per_g_b: Option<i64>,

    /// The list of boot volume replicas of this boot volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_replicas: Option<Vec<BootVolumeReplicaInfo>>,

    /// The list of autotune policies enabled for this volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autotune_policies: Option<Vec<AutotunePolicy>>,
}

/// Required fields for BootVolume
pub struct BootVolumeRequired {
    /// The availability domain of the boot volume. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the boot volume.
    pub compartment_id: String,

    /// The boot volume's Oracle ID (OCID).
    pub id: String,

    /// The current state of a boot volume.
    pub lifecycle_state: BootVolumeLifecycleState,

    /// The size of the volume in MBs. The value must be a multiple of 1024. This field is deprecated. Please use sizeInGBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_m_bs: i64,

    /// The date and time the boot volume was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,
}

impl BootVolume {
    /// Create a new BootVolume with required fields
    pub fn new(required: BootVolumeRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            size_in_m_bs: required.size_in_m_bs,

            time_created: required.time_created,

            defined_tags: None,

            system_tags: None,

            display_name: None,

            freeform_tags: None,

            image_id: None,

            is_hydrated: None,

            cluster_placement_group_id: None,

            vpus_per_g_b: None,

            size_in_g_bs: None,

            source_details: None,

            volume_group_id: None,

            kms_key_id: None,

            is_auto_tune_enabled: None,

            auto_tuned_vpus_per_g_b: None,

            boot_volume_replicas: None,

            autotune_policies: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: Option<String>) -> Self {
        self.image_id = value;
        self
    }

    /// Set is_hydrated
    pub fn set_is_hydrated(mut self, value: Option<bool>) -> Self {
        self.is_hydrated = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
        self
    }

    /// Set vpus_per_g_b
    pub fn set_vpus_per_g_b(mut self, value: Option<i64>) -> Self {
        self.vpus_per_g_b = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: BootVolumeLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set size_in_m_bs
    pub fn set_size_in_m_bs(mut self, value: i64) -> Self {
        self.size_in_m_bs = value;
        self
    }

    /// Set source_details
    pub fn set_source_details(
        mut self,
        value: Option<BootVolumeSourceFromBootVolumeBackupDetails>,
    ) -> Self {
        self.source_details = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: Option<String>) -> Self {
        self.volume_group_id = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set is_auto_tune_enabled
    pub fn set_is_auto_tune_enabled(mut self, value: Option<bool>) -> Self {
        self.is_auto_tune_enabled = value;
        self
    }

    /// Set auto_tuned_vpus_per_g_b
    pub fn set_auto_tuned_vpus_per_g_b(mut self, value: Option<i64>) -> Self {
        self.auto_tuned_vpus_per_g_b = value;
        self
    }

    /// Set boot_volume_replicas
    pub fn set_boot_volume_replicas(mut self, value: Option<Vec<BootVolumeReplicaInfo>>) -> Self {
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

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
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

    /// Set image_id (unwraps Option)
    pub fn with_image_id(mut self, value: impl Into<String>) -> Self {
        self.image_id = Some(value.into());
        self
    }

    /// Set is_hydrated (unwraps Option)
    pub fn with_is_hydrated(mut self, value: bool) -> Self {
        self.is_hydrated = Some(value);
        self
    }

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }

    /// Set vpus_per_g_b (unwraps Option)
    pub fn with_vpus_per_g_b(mut self, value: i64) -> Self {
        self.vpus_per_g_b = Some(value);
        self
    }

    /// Set size_in_g_bs (unwraps Option)
    pub fn with_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = Some(value);
        self
    }

    /// Set source_details (unwraps Option)
    pub fn with_source_details(
        mut self,
        value: BootVolumeSourceFromBootVolumeBackupDetails,
    ) -> Self {
        self.source_details = Some(value);
        self
    }

    /// Set volume_group_id (unwraps Option)
    pub fn with_volume_group_id(mut self, value: impl Into<String>) -> Self {
        self.volume_group_id = Some(value.into());
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }

    /// Set is_auto_tune_enabled (unwraps Option)
    pub fn with_is_auto_tune_enabled(mut self, value: bool) -> Self {
        self.is_auto_tune_enabled = Some(value);
        self
    }

    /// Set auto_tuned_vpus_per_g_b (unwraps Option)
    pub fn with_auto_tuned_vpus_per_g_b(mut self, value: i64) -> Self {
        self.auto_tuned_vpus_per_g_b = Some(value);
        self
    }

    /// Set boot_volume_replicas (unwraps Option)
    pub fn with_boot_volume_replicas(mut self, value: Vec<BootVolumeReplicaInfo>) -> Self {
        self.boot_volume_replicas = Some(value);
        self
    }

    /// Set autotune_policies (unwraps Option)
    pub fn with_autotune_policies(mut self, value: Vec<AutotunePolicy>) -> Self {
        self.autotune_policies = Some(value);
        self
    }
}
