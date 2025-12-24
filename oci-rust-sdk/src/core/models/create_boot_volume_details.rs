use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBootVolumeDetails {
    /// The OCID of the compartment that contains the boot volume.
    pub compartment_id: String,

    pub source_details: BootVolumeSourceFromBootVolumeBackupDetails,

    /// The availability domain of the volume. Omissible for cloning a volume. The new volume will be created in the availability domain of the source volume. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// If provided, specifies the ID of the boot volume backup policy to assign to the newly created boot volume. If omitted, no policy will be assigned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_policy_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The OCID of the Vault service key to assign as the master encryption key for the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// The size of the volume in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    /// The clusterPlacementGroup Id of the volume for volume placement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    /// The number of volume performance units (VPUs) that will be applied to this volume per GB, representing the Block Volume service's elastic performance options. See [Block Volume Performance Levels](https://docs.oracle.com/iaas/Content/Block/Concepts/blockvolumeperformance.htm#perf_levels) for more information. <p> Allowed values: <p> {@code 10}: Represents the Balanced option. <p> {@code 20}: Represents the Higher Performance option. <p> {@code 30}-{@code 120}: Represents the Ultra High Performance option. <p> For performance autotune enabled volumes, it would be the Default(Minimum) VPUs/GB. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpus_per_g_b: Option<i64>,

    /// Specifies whether the auto-tune performance is enabled for this boot volume. This field is deprecated. Use the {@code DetachedVolumeAutotunePolicy} instead to enable the volume for detached autotune.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_tune_enabled: Option<bool>,

    /// The list of boot volume replicas to be enabled for this boot volume in the specified destination availability domains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_replicas: Option<Vec<BootVolumeReplicaDetails>>,

    /// The list of autotune policies to be enabled for this volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autotune_policies: Option<Vec<AutotunePolicy>>,

    /// The OCID of the Vault service key which is the master encryption key for the boot volume cross region backups, which will be used in the destination region to encrypt the backup's encryption keys. For more information about the Vault service and encryption keys, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrc_kms_key_id: Option<String>,
}

/// Required fields for CreateBootVolumeDetails
pub struct CreateBootVolumeDetailsRequired {
    /// The OCID of the compartment that contains the boot volume.
    pub compartment_id: String,

    pub source_details: BootVolumeSourceFromBootVolumeBackupDetails,
}

impl CreateBootVolumeDetails {
    /// Create a new CreateBootVolumeDetails with required fields
    pub fn new(required: CreateBootVolumeDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            source_details: required.source_details,

            availability_domain: None,

            backup_policy_id: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            kms_key_id: None,

            size_in_g_bs: None,

            cluster_placement_group_id: None,

            vpus_per_g_b: None,

            is_auto_tune_enabled: None,

            boot_volume_replicas: None,

            autotune_policies: None,

            xrc_kms_key_id: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set backup_policy_id
    pub fn set_backup_policy_id(mut self, value: Option<String>) -> Self {
        self.backup_policy_id = value;
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
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_g_bs = value;
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

    /// Set source_details
    pub fn set_source_details(
        mut self,
        value: BootVolumeSourceFromBootVolumeBackupDetails,
    ) -> Self {
        self.source_details = value;
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

    /// Set xrc_kms_key_id
    pub fn set_xrc_kms_key_id(mut self, value: Option<String>) -> Self {
        self.xrc_kms_key_id = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set backup_policy_id (unwraps Option)
    pub fn with_backup_policy_id(mut self, value: impl Into<String>) -> Self {
        self.backup_policy_id = Some(value.into());
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

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }

    /// Set size_in_g_bs (unwraps Option)
    pub fn with_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = Some(value);
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

    /// Set xrc_kms_key_id (unwraps Option)
    pub fn with_xrc_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.xrc_kms_key_id = Some(value.into());
        self
    }
}
