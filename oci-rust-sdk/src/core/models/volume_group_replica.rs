use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// An asynchronous replica of a volume group that can then be used to create a new volume group or recover a volume group. For more information, see [Volume Group Replication](https://docs.oracle.com/iaas/Content/Block/Concepts/volumegroupreplication.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroupReplica {
    /// The availability domain of the volume group replica.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume group replica.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID for the volume group replica.
    pub id: String,

    /// The current state of a volume group.
    pub lifecycle_state: VolumeGroupReplicaLifecycleState,

    /// The aggregate size of the volume group replica in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_g_bs: i64,

    /// The OCID of the source volume group.
    pub volume_group_id: String,

    /// The date and time the volume group replica was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// Volume replicas within this volume group replica.
    pub member_replicas: Vec<MemberReplica>,

    /// The date and time the volume group replica was last synced from the source volume group. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_last_synced: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for VolumeGroupReplica
pub struct VolumeGroupReplicaRequired {
    /// The availability domain of the volume group replica.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume group replica.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID for the volume group replica.
    pub id: String,

    /// The current state of a volume group.
    pub lifecycle_state: VolumeGroupReplicaLifecycleState,

    /// The aggregate size of the volume group replica in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_g_bs: i64,

    /// The OCID of the source volume group.
    pub volume_group_id: String,

    /// The date and time the volume group replica was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// Volume replicas within this volume group replica.
    pub member_replicas: Vec<MemberReplica>,

    /// The date and time the volume group replica was last synced from the source volume group. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_last_synced: DateTime<Utc>,
}

impl VolumeGroupReplica {
    /// Create a new VolumeGroupReplica with required fields
    pub fn new(required: VolumeGroupReplicaRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            size_in_g_bs: required.size_in_g_bs,

            volume_group_id: required.volume_group_id,

            time_created: required.time_created,

            member_replicas: required.member_replicas,

            time_last_synced: required.time_last_synced,

            defined_tags: None,

            freeform_tags: None,
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
    pub fn set_lifecycle_state(mut self, value: VolumeGroupReplicaLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: String) -> Self {
        self.volume_group_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set member_replicas
    pub fn set_member_replicas(mut self, value: Vec<MemberReplica>) -> Self {
        self.member_replicas = value;
        self
    }

    /// Set time_last_synced
    pub fn set_time_last_synced(mut self, value: DateTime<Utc>) -> Self {
        self.time_last_synced = value;
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
}
