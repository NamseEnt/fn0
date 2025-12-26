use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeGroupDetails {
    /// The availability domain of the volume group.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume group.
    pub compartment_id: String,

    pub source_details: VolumeGroupSourceFromVolumeGroupReplicaDetails,

    /// If provided, specifies the ID of the volume backup policy to assign to the newly created volume group. If omitted, no policy will be assigned.
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

    /// The list of volume group replicas that this volume group will be enabled to have in the specified destination availability domains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_replicas: Option<Vec<VolumeGroupReplicaDetails>>,

    /// The clusterPlacementGroup Id of the volume group for volume group placement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    /// The OCID of the Vault service key which is the master encryption key for the volume's cross region backups, which will be used in the destination region to encrypt the backup's encryption keys. For more information about the Vault service and encryption keys, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrc_kms_key_id: Option<String>,
}

/// Required fields for CreateVolumeGroupDetails
pub struct CreateVolumeGroupDetailsRequired {
    /// The availability domain of the volume group.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume group.
    pub compartment_id: String,

    pub source_details: VolumeGroupSourceFromVolumeGroupReplicaDetails,
}

impl CreateVolumeGroupDetails {
    /// Create a new CreateVolumeGroupDetails with required fields
    pub fn new(required: CreateVolumeGroupDetailsRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            source_details: required.source_details,

            backup_policy_id: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            volume_group_replicas: None,

            cluster_placement_group_id: None,

            xrc_kms_key_id: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
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

    /// Set source_details
    pub fn set_source_details(
        mut self,
        value: VolumeGroupSourceFromVolumeGroupReplicaDetails,
    ) -> Self {
        self.source_details = value;
        self
    }

    /// Set volume_group_replicas
    pub fn set_volume_group_replicas(
        mut self,
        value: Option<Vec<VolumeGroupReplicaDetails>>,
    ) -> Self {
        self.volume_group_replicas = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
        self
    }

    /// Set xrc_kms_key_id
    pub fn set_xrc_kms_key_id(mut self, value: Option<String>) -> Self {
        self.xrc_kms_key_id = value;
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

    /// Set volume_group_replicas (unwraps Option)
    pub fn with_volume_group_replicas(mut self, value: Vec<VolumeGroupReplicaDetails>) -> Self {
        self.volume_group_replicas = Some(value);
        self
    }

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }

    /// Set xrc_kms_key_id (unwraps Option)
    pub fn with_xrc_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.xrc_kms_key_id = Some(value.into());
        self
    }
}
