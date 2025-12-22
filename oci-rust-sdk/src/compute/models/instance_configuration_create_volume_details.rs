use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Creates a new block volume for instance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationCreateVolumeDetails {
    /// The availability domain of the volume.
    /// Example: Uocm:PHX-AD-1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// If provided, specifies the ID of the volume backup policy to assign to the newly
    /// created volume. If omitted, no policy will be assigned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_policy_id: Option<String>,

    /// The OCID of the compartment that contains the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Specifies whether the auto-tune performance is enabled for this boot volume.
    /// This field is deprecated. Use the autotune_policies instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_tune_enabled: Option<bool>,

    /// The list of block volume replicas to be enabled for this volume
    /// in the specified destination availability domains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_volume_replicas: Option<Vec<serde_json::Value>>,

    /// Defined tags for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable.
    /// Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The OCID of the Vault service key to assign as the master encryption key
    /// for the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// The number of volume performance units (VPUs) that will be applied to this volume per GB.
    /// Allowed values: 0 (Lower Cost), 10 (Balanced), 20 (Higher Performance), 30-120 (Ultra High Performance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpus_per_gb: Option<i64>,

    /// The clusterPlacementGroup Id of the volume for volume placement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    /// The size of the volume in GBs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    /// Volume source details (polymorphic type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<serde_json::Value>,

    /// The list of autotune policies enabled for this volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autotune_policies: Option<Vec<serde_json::Value>>,

    /// The OCID of the Vault service key which is the master encryption key for the
    /// block volume cross region backups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrc_kms_key_id: Option<String>,

    /// When set to true, enables SCSI Persistent Reservation (SCSI PR) for the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reservations_enabled: Option<bool>,
}

impl InstanceConfigurationCreateVolumeDetails {
    pub fn new() -> Self {
        Self {
            availability_domain: None,
            backup_policy_id: None,
            compartment_id: None,
            is_auto_tune_enabled: None,
            block_volume_replicas: None,
            defined_tags: None,
            display_name: None,
            freeform_tags: None,
            kms_key_id: None,
            vpus_per_gb: None,
            cluster_placement_group_id: None,
            size_in_g_bs: None,
            source_details: None,
            autotune_policies: None,
            xrc_kms_key_id: None,
            is_reservations_enabled: None,
        }
    }

    pub fn with_availability_domain(mut self, domain: impl Into<String>) -> Self {
        self.availability_domain = Some(domain.into());
        self
    }

    pub fn with_compartment_id(mut self, id: impl Into<String>) -> Self {
        self.compartment_id = Some(id.into());
        self
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn with_size_in_gbs(mut self, size: i64) -> Self {
        self.size_in_g_bs = Some(size);
        self
    }

    pub fn with_vpus_per_gb(mut self, vpus: i64) -> Self {
        self.vpus_per_gb = Some(vpus);
        self
    }

    pub fn with_backup_policy_id(mut self, id: impl Into<String>) -> Self {
        self.backup_policy_id = Some(id.into());
        self
    }

    pub fn with_kms_key_id(mut self, id: impl Into<String>) -> Self {
        self.kms_key_id = Some(id.into());
        self
    }

    pub fn with_cluster_placement_group_id(mut self, id: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(id.into());
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, availability_domain: Option<String>) -> Self {
        self.availability_domain = availability_domain;
        self
    }

    /// Set backup_policy_id
    pub fn set_backup_policy_id(mut self, backup_policy_id: Option<String>) -> Self {
        self.backup_policy_id = backup_policy_id;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, compartment_id: Option<String>) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    /// Set is_auto_tune_enabled
    pub fn set_is_auto_tune_enabled(mut self, is_auto_tune_enabled: Option<bool>) -> Self {
        self.is_auto_tune_enabled = is_auto_tune_enabled;
        self
    }

    /// Set block_volume_replicas
    pub fn set_block_volume_replicas(mut self, block_volume_replicas: Option<Vec<serde_json::Value>>) -> Self {
        self.block_volume_replicas = block_volume_replicas;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(mut self, defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.defined_tags = defined_tags;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, freeform_tags: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = freeform_tags;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, kms_key_id: Option<String>) -> Self {
        self.kms_key_id = kms_key_id;
        self
    }

    /// Set vpus_per_gb
    pub fn set_vpus_per_gb(mut self, vpus_per_gb: Option<i64>) -> Self {
        self.vpus_per_gb = vpus_per_gb;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, cluster_placement_group_id: Option<String>) -> Self {
        self.cluster_placement_group_id = cluster_placement_group_id;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, size_in_g_bs: Option<i64>) -> Self {
        self.size_in_g_bs = size_in_g_bs;
        self
    }

    /// Set source_details
    pub fn set_source_details(mut self, source_details: Option<serde_json::Value>) -> Self {
        self.source_details = source_details;
        self
    }

    /// Set autotune_policies
    pub fn set_autotune_policies(mut self, autotune_policies: Option<Vec<serde_json::Value>>) -> Self {
        self.autotune_policies = autotune_policies;
        self
    }

    /// Set xrc_kms_key_id
    pub fn set_xrc_kms_key_id(mut self, xrc_kms_key_id: Option<String>) -> Self {
        self.xrc_kms_key_id = xrc_kms_key_id;
        self
    }

    /// Set is_reservations_enabled
    pub fn set_is_reservations_enabled(mut self, is_reservations_enabled: Option<bool>) -> Self {
        self.is_reservations_enabled = is_reservations_enabled;
        self
    }
}

impl Default for InstanceConfigurationCreateVolumeDetails {
    fn default() -> Self {
        Self::new()
    }
}
