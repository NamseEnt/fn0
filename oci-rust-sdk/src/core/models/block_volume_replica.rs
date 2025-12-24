use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// An asynchronous replica of a block volume that can then be used to create a new block volume or recover a block volume. For more information, see [Overview of Cross-Region Volume Replication](https://docs.oracle.com/iaas/Content/Block/Concepts/volumereplication.htm) To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockVolumeReplica {
    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the block volume replica.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The block volume replica's Oracle ID (OCID).
    pub id: String,

    /// The current state of a block volume replica.
    pub lifecycle_state: BlockVolumeReplicaLifecycleState,

    /// The size of the source block volume, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_g_bs: i64,

    /// The date and time the block volume replica was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The date and time the block volume replica was last synced from the source block volume. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_last_synced: DateTime<Utc>,

    /// The OCID of the source block volume.
    pub block_volume_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The total size of the data transferred from the source block volume to the block volume replica, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_data_transferred_in_g_bs: Option<i64>,

    /// The OCID of the volume group replica.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_replica_id: Option<String>,

    /// The OCID of the Vault service key to assign as the master encryption key for the block volume replica, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,
}

/// Required fields for BlockVolumeReplica
pub struct BlockVolumeReplicaRequired {
    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the block volume replica.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The block volume replica's Oracle ID (OCID).
    pub id: String,

    /// The current state of a block volume replica.
    pub lifecycle_state: BlockVolumeReplicaLifecycleState,

    /// The size of the source block volume, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_g_bs: i64,

    /// The date and time the block volume replica was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The date and time the block volume replica was last synced from the source block volume. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_last_synced: DateTime<Utc>,

    /// The OCID of the source block volume.
    pub block_volume_id: String,
}

impl BlockVolumeReplica {
    /// Create a new BlockVolumeReplica with required fields
    pub fn new(required: BlockVolumeReplicaRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            size_in_g_bs: required.size_in_g_bs,

            time_created: required.time_created,

            time_last_synced: required.time_last_synced,

            block_volume_id: required.block_volume_id,

            defined_tags: None,

            freeform_tags: None,

            total_data_transferred_in_g_bs: None,

            volume_group_replica_id: None,

            kms_key_id: None,
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: BlockVolumeReplicaLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_last_synced
    pub fn set_time_last_synced(mut self, value: DateTime<Utc>) -> Self {
        self.time_last_synced = value;
        self
    }

    /// Set block_volume_id
    pub fn set_block_volume_id(mut self, value: String) -> Self {
        self.block_volume_id = value;
        self
    }

    /// Set total_data_transferred_in_g_bs
    pub fn set_total_data_transferred_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.total_data_transferred_in_g_bs = value;
        self
    }

    /// Set volume_group_replica_id
    pub fn set_volume_group_replica_id(mut self, value: Option<String>) -> Self {
        self.volume_group_replica_id = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
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

    /// Set total_data_transferred_in_g_bs (unwraps Option)
    pub fn with_total_data_transferred_in_g_bs(mut self, value: i64) -> Self {
        self.total_data_transferred_in_g_bs = Some(value);
        self
    }

    /// Set volume_group_replica_id (unwraps Option)
    pub fn with_volume_group_replica_id(mut self, value: impl Into<String>) -> Self {
        self.volume_group_replica_id = Some(value.into());
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }
}
