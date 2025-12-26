use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeGroupDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// OCIDs for the volumes in this volume group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_ids: Option<Vec<String>>,

    /// The list of volume group replicas that this volume group will be updated to have in the specified destination availability domains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_replicas: Option<Vec<VolumeGroupReplicaDetails>>,
}

impl UpdateVolumeGroupDetails {
    /// Create a new UpdateVolumeGroupDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            volume_ids: None,

            volume_group_replicas: None,
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

    /// Set volume_ids
    pub fn set_volume_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.volume_ids = value;
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

    /// Set volume_ids (unwraps Option)
    pub fn with_volume_ids(mut self, value: Vec<String>) -> Self {
        self.volume_ids = Some(value);
        self
    }

    /// Set volume_group_replicas (unwraps Option)
    pub fn with_volume_group_replicas(mut self, value: Vec<VolumeGroupReplicaDetails>) -> Self {
        self.volume_group_replicas = Some(value);
        self
    }
}

impl Default for UpdateVolumeGroupDetails {
    fn default() -> Self {
        Self::new()
    }
}
