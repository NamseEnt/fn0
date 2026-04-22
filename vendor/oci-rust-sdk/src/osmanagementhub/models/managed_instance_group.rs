use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// An object that defines the managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroup {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance group.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance group.
    pub compartment_id: String,

    /// The current state of the managed instance group.
    pub lifecycle_state: ManagedInstanceGroupLifecycleState,

    /// A user-friendly name for the managed instance group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// User-specified information about the managed instance group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The time the managed instance group was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The time the managed instance group was last modified (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_modified: Option<DateTime<Utc>>,

    /// The operating system type of the instances in the managed instance group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    /// The CPU architecture of the instances in the managed instance group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch_type: Option<ArchType>,

    /// The vendor of the operating system used by the managed instances in the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_name: Option<VendorName>,

    /// The list of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that the managed instance group will use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_ids: Option<Vec<SoftwareSourceDetails>>,

    /// The list of software sources that the managed instance group will use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_sources: Option<Vec<SoftwareSourceDetails>>,

    /// The list of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) attached to the managed instance group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_ids: Option<Vec<String>>,

    /// The number of managed instances in the group. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_count: Option<i64>,

    /// The location of managed instances attached to the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// The number of scheduled jobs pending against the managed instance group. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_job_count: Option<i64>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Oracle Notifications service (ONS) topic. ONS is the channel used to send notifications to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_topic_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_settings: Option<AutonomousSettings>,

    /// Indicates whether the Autonomous Linux service manages the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for ManagedInstanceGroup
pub struct ManagedInstanceGroupRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance group.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance group.
    pub compartment_id: String,

    /// The current state of the managed instance group.
    pub lifecycle_state: ManagedInstanceGroupLifecycleState,
}

impl ManagedInstanceGroup {
    /// Create a new ManagedInstanceGroup with required fields
    pub fn new(required: ManagedInstanceGroupRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            lifecycle_state: required.lifecycle_state,

            display_name: None,

            description: None,

            time_created: None,

            time_modified: None,

            os_family: None,

            arch_type: None,

            vendor_name: None,

            software_source_ids: None,

            software_sources: None,

            managed_instance_ids: None,

            managed_instance_count: None,

            location: None,

            pending_job_count: None,

            notification_topic_id: None,

            autonomous_settings: None,

            is_managed_by_autonomous_linux: None,

            freeform_tags: None,

            defined_tags: None,

            system_tags: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
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

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_modified
    pub fn set_time_modified(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_modified = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ManagedInstanceGroupLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: Option<OsFamily>) -> Self {
        self.os_family = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: Option<ArchType>) -> Self {
        self.arch_type = value;
        self
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: Option<VendorName>) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set software_source_ids
    pub fn set_software_source_ids(mut self, value: Option<Vec<SoftwareSourceDetails>>) -> Self {
        self.software_source_ids = value;
        self
    }

    /// Set software_sources
    pub fn set_software_sources(mut self, value: Option<Vec<SoftwareSourceDetails>>) -> Self {
        self.software_sources = value;
        self
    }

    /// Set managed_instance_ids
    pub fn set_managed_instance_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.managed_instance_ids = value;
        self
    }

    /// Set managed_instance_count
    pub fn set_managed_instance_count(mut self, value: Option<i64>) -> Self {
        self.managed_instance_count = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
        self
    }

    /// Set pending_job_count
    pub fn set_pending_job_count(mut self, value: Option<i64>) -> Self {
        self.pending_job_count = value;
        self
    }

    /// Set notification_topic_id
    pub fn set_notification_topic_id(mut self, value: Option<String>) -> Self {
        self.notification_topic_id = value;
        self
    }

    /// Set autonomous_settings
    pub fn set_autonomous_settings(mut self, value: Option<AutonomousSettings>) -> Self {
        self.autonomous_settings = value;
        self
    }

    /// Set is_managed_by_autonomous_linux
    pub fn set_is_managed_by_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_managed_by_autonomous_linux = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set time_modified (unwraps Option)
    pub fn with_time_modified(mut self, value: DateTime<Utc>) -> Self {
        self.time_modified = Some(value);
        self
    }

    /// Set os_family (unwraps Option)
    pub fn with_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = Some(value);
        self
    }

    /// Set arch_type (unwraps Option)
    pub fn with_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = Some(value);
        self
    }

    /// Set vendor_name (unwraps Option)
    pub fn with_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = Some(value);
        self
    }

    /// Set software_source_ids (unwraps Option)
    pub fn with_software_source_ids(mut self, value: Vec<SoftwareSourceDetails>) -> Self {
        self.software_source_ids = Some(value);
        self
    }

    /// Set software_sources (unwraps Option)
    pub fn with_software_sources(mut self, value: Vec<SoftwareSourceDetails>) -> Self {
        self.software_sources = Some(value);
        self
    }

    /// Set managed_instance_ids (unwraps Option)
    pub fn with_managed_instance_ids(mut self, value: Vec<String>) -> Self {
        self.managed_instance_ids = Some(value);
        self
    }

    /// Set managed_instance_count (unwraps Option)
    pub fn with_managed_instance_count(mut self, value: i64) -> Self {
        self.managed_instance_count = Some(value);
        self
    }

    /// Set location (unwraps Option)
    pub fn with_location(mut self, value: ManagedInstanceLocation) -> Self {
        self.location = Some(value);
        self
    }

    /// Set pending_job_count (unwraps Option)
    pub fn with_pending_job_count(mut self, value: i64) -> Self {
        self.pending_job_count = Some(value);
        self
    }

    /// Set notification_topic_id (unwraps Option)
    pub fn with_notification_topic_id(mut self, value: impl Into<String>) -> Self {
        self.notification_topic_id = Some(value.into());
        self
    }

    /// Set autonomous_settings (unwraps Option)
    pub fn with_autonomous_settings(mut self, value: AutonomousSettings) -> Self {
        self.autonomous_settings = Some(value);
        self
    }

    /// Set is_managed_by_autonomous_linux (unwraps Option)
    pub fn with_is_managed_by_autonomous_linux(mut self, value: bool) -> Self {
        self.is_managed_by_autonomous_linux = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
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

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }
}
