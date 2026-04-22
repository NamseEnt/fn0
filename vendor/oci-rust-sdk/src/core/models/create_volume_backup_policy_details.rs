use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Specifies the properties for creating user defined backup policy. For more information about user defined backup policies, see [User Defined Policies](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm#UserDefinedBackupPolicies) in [Policy-Based Backups](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeBackupPolicyDetails {
    /// The OCID of the compartment.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The paired destination region for copying scheduled backups to. Example: {@code us-ashburn-1}. See [Region Pairs](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm#RegionPairs) for details about paired regions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_region: Option<String>,

    /// The collection of schedules for the volume backup policy. See see [Schedules](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm#schedules) in [Policy-Based Backups](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm) for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedules: Option<Vec<VolumeBackupSchedule>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for CreateVolumeBackupPolicyDetails
pub struct CreateVolumeBackupPolicyDetailsRequired {
    /// The OCID of the compartment.
    pub compartment_id: String,
}

impl CreateVolumeBackupPolicyDetails {
    /// Create a new CreateVolumeBackupPolicyDetails with required fields
    pub fn new(required: CreateVolumeBackupPolicyDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: None,

            destination_region: None,

            schedules: None,

            defined_tags: None,

            freeform_tags: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set destination_region
    pub fn set_destination_region(mut self, value: Option<String>) -> Self {
        self.destination_region = value;
        self
    }

    /// Set schedules
    pub fn set_schedules(mut self, value: Option<Vec<VolumeBackupSchedule>>) -> Self {
        self.schedules = value;
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

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set destination_region (unwraps Option)
    pub fn with_destination_region(mut self, value: impl Into<String>) -> Self {
        self.destination_region = Some(value.into());
        self
    }

    /// Set schedules (unwraps Option)
    pub fn with_schedules(mut self, value: Vec<VolumeBackupSchedule>) -> Self {
        self.schedules = Some(value);
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
