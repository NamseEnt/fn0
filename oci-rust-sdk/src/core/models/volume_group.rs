use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Specifies a volume group which is a collection of volumes. For more information, see [Volume Groups](https://docs.oracle.com/iaas/Content/Block/Concepts/volumegroups.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroup {
    /// The availability domain of the volume group.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume group.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID for the volume group.
    pub id: String,

    /// The current state of a volume group.
    pub lifecycle_state: VolumeGroupLifecycleState,

    /// The aggregate size of the volume group in MBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_m_bs: i64,

    /// The date and time the volume group was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// OCIDs for the volumes in this volume group.
    pub volume_ids: Vec<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The aggregate size of the volume group in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<VolumeGroupSourceFromVolumeGroupReplicaDetails>,

    /// Specifies whether the newly created cloned volume group's data has finished copying from the source volume group or backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hydrated: Option<bool>,

    /// The list of volume group replicas of this volume group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_replicas: Option<Vec<VolumeGroupReplicaInfo>>,
}

/// Required fields for VolumeGroup
pub struct VolumeGroupRequired {
    /// The availability domain of the volume group.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the volume group.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID for the volume group.
    pub id: String,

    /// The current state of a volume group.
    pub lifecycle_state: VolumeGroupLifecycleState,

    /// The aggregate size of the volume group in MBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size_in_m_bs: i64,

    /// The date and time the volume group was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// OCIDs for the volumes in this volume group.
    pub volume_ids: Vec<String>,
}

impl VolumeGroup {
    /// Create a new VolumeGroup with required fields
    pub fn new(required: VolumeGroupRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            size_in_m_bs: required.size_in_m_bs,

            time_created: required.time_created,

            volume_ids: required.volume_ids,

            defined_tags: None,

            freeform_tags: None,

            size_in_g_bs: None,

            source_details: None,

            is_hydrated: None,

            volume_group_replicas: None,
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
    pub fn set_lifecycle_state(mut self, value: VolumeGroupLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set size_in_m_bs
    pub fn set_size_in_m_bs(mut self, value: i64) -> Self {
        self.size_in_m_bs = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set source_details
    pub fn set_source_details(
        mut self,
        value: Option<VolumeGroupSourceFromVolumeGroupReplicaDetails>,
    ) -> Self {
        self.source_details = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set volume_ids
    pub fn set_volume_ids(mut self, value: Vec<String>) -> Self {
        self.volume_ids = value;
        self
    }

    /// Set is_hydrated
    pub fn set_is_hydrated(mut self, value: Option<bool>) -> Self {
        self.is_hydrated = value;
        self
    }

    /// Set volume_group_replicas
    pub fn set_volume_group_replicas(mut self, value: Option<Vec<VolumeGroupReplicaInfo>>) -> Self {
        self.volume_group_replicas = value;
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

    /// Set size_in_g_bs (unwraps Option)
    pub fn with_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = Some(value);
        self
    }

    /// Set source_details (unwraps Option)
    pub fn with_source_details(
        mut self,
        value: VolumeGroupSourceFromVolumeGroupReplicaDetails,
    ) -> Self {
        self.source_details = Some(value);
        self
    }

    /// Set is_hydrated (unwraps Option)
    pub fn with_is_hydrated(mut self, value: bool) -> Self {
        self.is_hydrated = Some(value);
        self
    }

    /// Set volume_group_replicas (unwraps Option)
    pub fn with_volume_group_replicas(mut self, value: Vec<VolumeGroupReplicaInfo>) -> Self {
        self.volume_group_replicas = Some(value);
        self
    }
}
