use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A detachable block volume device that allows you to dynamically expand the storage capacity of an instance. For more information, see [Overview of Cloud Volume Storage](https://docs.oracle.com/iaas/Content/Block/Concepts/overview.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    /// The availability domain of the volume. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the volume.
    pub id: String,

    /// The current state of a volume.
    pub lifecycle_state: VolumeLifecycleState,

    /// The size of the volume in MBs. This field is deprecated. Use sizeInGBs instead. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_m_bs: i64,

    /// The date and time the volume was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Specifies whether the cloned volume's data has finished copying from the source volume or backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hydrated: Option<bool>,

    /// The OCID of the Vault service key which is the master encryption key for the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// The number of volume performance units (VPUs) that will be applied to this volume per GB, representing the Block Volume service's elastic performance options. See [Block Volume Performance Levels](https://docs.oracle.com/iaas/Content/Block/Concepts/blockvolumeperformance.htm#perf_levels) for more information. <p> Allowed values: <p> {@code 0}: Represents Lower Cost option. <p> {@code 10}: Represents Balanced option. <p> {@code 20}: Represents Higher Performance option. <p> {@code 30}-{@code 120}: Represents the Ultra High Performance option. <p> For performance autotune enabled volumes, It would be the Default(Minimum) VPUs/GB. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpus_per_g_b: Option<i64>,

    /// The clusterPlacementGroup Id of the volume for volume placement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    /// The size of the volume in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<VolumeSourceFromBlockVolumeReplicaDetails>,

    /// The OCID of the source volume group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_id: Option<String>,

    /// Specifies whether the auto-tune performance is enabled for this volume. This field is deprecated. Use the {@code DetachedVolumeAutotunePolicy} instead to enable the volume for detached autotune.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_tune_enabled: Option<bool>,

    /// The number of Volume Performance Units per GB that this volume is effectively tuned to. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_tuned_vpus_per_g_b: Option<i64>,

    /// The list of block volume replicas of this volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_volume_replicas: Option<Vec<BlockVolumeReplicaInfo>>,

    /// The list of autotune policies enabled for this volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autotune_policies: Option<Vec<AutotunePolicy>>,

    /// When set to true, enables SCSI Persistent Reservation (SCSI PR) for the volume. For more information, see [Persistent Reservations](https://docs.oracle.com/iaas/Content/Block/Concepts/persistent-reservations.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reservations_enabled: Option<bool>,
}

/// Required fields for Volume
pub struct VolumeRequired {
    /// The availability domain of the volume. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the volume.
    pub id: String,

    /// The current state of a volume.
    pub lifecycle_state: VolumeLifecycleState,

    /// The size of the volume in MBs. This field is deprecated. Use sizeInGBs instead. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_m_bs: i64,

    /// The date and time the volume was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,
}

impl Volume {
    /// Create a new Volume with required fields
    pub fn new(required: VolumeRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            size_in_m_bs: required.size_in_m_bs,

            time_created: required.time_created,

            defined_tags: None,

            freeform_tags: None,

            system_tags: None,

            is_hydrated: None,

            kms_key_id: None,

            vpus_per_g_b: None,

            cluster_placement_group_id: None,

            size_in_g_bs: None,

            source_details: None,

            volume_group_id: None,

            is_auto_tune_enabled: None,

            auto_tuned_vpus_per_g_b: None,

            block_volume_replicas: None,

            autotune_policies: None,

            is_reservations_enabled: None,
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

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set is_hydrated
    pub fn set_is_hydrated(mut self, value: Option<bool>) -> Self {
        self.is_hydrated = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: VolumeLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set vpus_per_g_b
    pub fn set_vpus_per_g_b(mut self, value: Option<i64>) -> Self {
        self.vpus_per_g_b = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
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
        value: Option<VolumeSourceFromBlockVolumeReplicaDetails>,
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

    /// Set block_volume_replicas
    pub fn set_block_volume_replicas(mut self, value: Option<Vec<BlockVolumeReplicaInfo>>) -> Self {
        self.block_volume_replicas = value;
        self
    }

    /// Set autotune_policies
    pub fn set_autotune_policies(mut self, value: Option<Vec<AutotunePolicy>>) -> Self {
        self.autotune_policies = value;
        self
    }

    /// Set is_reservations_enabled
    pub fn set_is_reservations_enabled(mut self, value: Option<bool>) -> Self {
        self.is_reservations_enabled = value;
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
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

    /// Set is_hydrated (unwraps Option)
    pub fn with_is_hydrated(mut self, value: bool) -> Self {
        self.is_hydrated = Some(value);
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

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }

    /// Set size_in_g_bs (unwraps Option)
    pub fn with_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = Some(value);
        self
    }

    /// Set source_details (unwraps Option)
    pub fn with_source_details(mut self, value: VolumeSourceFromBlockVolumeReplicaDetails) -> Self {
        self.source_details = Some(value);
        self
    }

    /// Set volume_group_id (unwraps Option)
    pub fn with_volume_group_id(mut self, value: impl Into<String>) -> Self {
        self.volume_group_id = Some(value.into());
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

    /// Set block_volume_replicas (unwraps Option)
    pub fn with_block_volume_replicas(mut self, value: Vec<BlockVolumeReplicaInfo>) -> Self {
        self.block_volume_replicas = Some(value);
        self
    }

    /// Set autotune_policies (unwraps Option)
    pub fn with_autotune_policies(mut self, value: Vec<AutotunePolicy>) -> Self {
        self.autotune_policies = Some(value);
        self
    }

    /// Set is_reservations_enabled (unwraps Option)
    pub fn with_is_reservations_enabled(mut self, value: bool) -> Self {
        self.is_reservations_enabled = Some(value);
        self
    }
}
