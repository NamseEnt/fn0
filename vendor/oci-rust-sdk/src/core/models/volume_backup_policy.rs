use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A policy for automatically creating volume backups according to a recurring schedule. Has a set of one or more schedules that control when and how backups are created. <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeBackupPolicy {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the volume backup policy.
    pub id: String,

    /// The collection of schedules that this policy will apply.
    pub schedules: Vec<VolumeBackupSchedule>,

    /// The date and time the volume backup policy was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The paired destination region for copying scheduled backups to. Example {@code us-ashburn-1}. See [Region Pairs](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm#RegionPairs) for details about paired regions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_region: Option<String>,

    /// The OCID of the compartment that contains the volume backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for VolumeBackupPolicy
pub struct VolumeBackupPolicyRequired {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the volume backup policy.
    pub id: String,

    /// The collection of schedules that this policy will apply.
    pub schedules: Vec<VolumeBackupSchedule>,

    /// The date and time the volume backup policy was created. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,
}

impl VolumeBackupPolicy {
    /// Create a new VolumeBackupPolicy with required fields
    pub fn new(required: VolumeBackupPolicyRequired) -> Self {
        Self {
            display_name: required.display_name,

            id: required.id,

            schedules: required.schedules,

            time_created: required.time_created,

            destination_region: None,

            compartment_id: None,

            defined_tags: None,

            freeform_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set schedules
    pub fn set_schedules(mut self, value: Vec<VolumeBackupSchedule>) -> Self {
        self.schedules = value;
        self
    }

    /// Set destination_region
    pub fn set_destination_region(mut self, value: Option<String>) -> Self {
        self.destination_region = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
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

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set destination_region (unwraps Option)
    pub fn with_destination_region(mut self, value: impl Into<String>) -> Self {
        self.destination_region = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
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
