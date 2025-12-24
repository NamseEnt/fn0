use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Summary of the lifecycle environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleEnvironmentSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle environment.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the lifecycle environment.
    pub compartment_id: String,

    /// A user-friendly name for the lifecycle environment.
    pub display_name: String,

    /// User-specified information about the lifecycle environment.
    pub description: String,

    /// User-specified list of lifecycle stages used within the lifecycle environment.
    pub stages: Vec<LifecycleStageSummary>,

    /// The CPU architecture of the managed instances in the lifecycle environment.
    pub arch_type: ArchType,

    /// The operating system of the managed instances in the lifecycle environment.
    pub os_family: OsFamily,

    /// The vendor of the operating system used by the managed instances in the lifecycle environment.
    pub vendor_name: VendorName,

    /// The current state of the lifecycle environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

    /// The location of managed instances attached to the lifecycle environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// The time the lifecycle environment was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The time the lifecycle environment was last modified (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_modified: Option<DateTime<Utc>>,

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

/// Required fields for LifecycleEnvironmentSummary
pub struct LifecycleEnvironmentSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle environment.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the lifecycle environment.
    pub compartment_id: String,

    /// A user-friendly name for the lifecycle environment.
    pub display_name: String,

    /// User-specified information about the lifecycle environment.
    pub description: String,

    /// User-specified list of lifecycle stages used within the lifecycle environment.
    pub stages: Vec<LifecycleStageSummary>,

    /// The CPU architecture of the managed instances in the lifecycle environment.
    pub arch_type: ArchType,

    /// The operating system of the managed instances in the lifecycle environment.
    pub os_family: OsFamily,

    /// The vendor of the operating system used by the managed instances in the lifecycle environment.
    pub vendor_name: VendorName,
}

impl LifecycleEnvironmentSummary {
    /// Create a new LifecycleEnvironmentSummary with required fields
    pub fn new(required: LifecycleEnvironmentSummaryRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            description: required.description,

            stages: required.stages,

            arch_type: required.arch_type,

            os_family: required.os_family,

            vendor_name: required.vendor_name,

            lifecycle_state: None,

            location: None,

            time_created: None,

            time_modified: None,

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
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: String) -> Self {
        self.description = value;
        self
    }

    /// Set stages
    pub fn set_stages(mut self, value: Vec<LifecycleStageSummary>) -> Self {
        self.stages = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = value;
        self
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
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

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }

    /// Set location (unwraps Option)
    pub fn with_location(mut self, value: ManagedInstanceLocation) -> Self {
        self.location = Some(value);
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
