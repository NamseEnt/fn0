use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a lifecycle stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleStageSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the lifecycle stage.
    pub compartment_id: String,

    /// The user-friendly name for the lifecycle stage.
    pub display_name: String,

    /// User-specified rank for the lifecycle stage. Rank determines the hierarchy of the lifecycle stages within the lifecycle environment. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub rank: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle environment that contains the lifecycle stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_environment_id: Option<String>,

    /// The user-friendly name for the lifecycle environment. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_environment_display_name: Option<String>,

    /// The operating system of the managed instances in the lifecycle stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    /// The CPU architecture of the managed instances in the lifecycle stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch_type: Option<ArchType>,

    /// The vendor of the operating system used by the managed instances in the lifecycle stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_name: Option<VendorName>,

    /// The list of managed instances associated with the lifecycle stage. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instances: Option<i64>,

    /// The location of managed instances associated with the lifecycle stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<SoftwareSourceDetails>,

    /// The time the lifecycle stage was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The time the lifecycle stage was last modified (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_modified: Option<DateTime<Utc>>,

    /// The current state of the lifecycle environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

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

/// Required fields for LifecycleStageSummary
pub struct LifecycleStageSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the lifecycle stage.
    pub compartment_id: String,

    /// The user-friendly name for the lifecycle stage.
    pub display_name: String,

    /// User-specified rank for the lifecycle stage. Rank determines the hierarchy of the lifecycle stages within the lifecycle environment. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub rank: i64,
}

impl LifecycleStageSummary {
    /// Create a new LifecycleStageSummary with required fields
    pub fn new(required: LifecycleStageSummaryRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            rank: required.rank,

            id: None,

            lifecycle_environment_id: None,

            lifecycle_environment_display_name: None,

            os_family: None,

            arch_type: None,

            vendor_name: None,

            managed_instances: None,

            location: None,

            software_source_id: None,

            time_created: None,

            time_modified: None,

            lifecycle_state: None,

            freeform_tags: None,

            defined_tags: None,

            system_tags: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
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

    /// Set lifecycle_environment_id
    pub fn set_lifecycle_environment_id(mut self, value: Option<String>) -> Self {
        self.lifecycle_environment_id = value;
        self
    }

    /// Set lifecycle_environment_display_name
    pub fn set_lifecycle_environment_display_name(mut self, value: Option<String>) -> Self {
        self.lifecycle_environment_display_name = value;
        self
    }

    /// Set rank
    pub fn set_rank(mut self, value: i64) -> Self {
        self.rank = value;
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

    /// Set managed_instances
    pub fn set_managed_instances(mut self, value: Option<i64>) -> Self {
        self.managed_instances = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: Option<SoftwareSourceDetails>) -> Self {
        self.software_source_id = value;
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
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
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

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set lifecycle_environment_id (unwraps Option)
    pub fn with_lifecycle_environment_id(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_environment_id = Some(value.into());
        self
    }

    /// Set lifecycle_environment_display_name (unwraps Option)
    pub fn with_lifecycle_environment_display_name(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_environment_display_name = Some(value.into());
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

    /// Set managed_instances (unwraps Option)
    pub fn with_managed_instances(mut self, value: i64) -> Self {
        self.managed_instances = Some(value);
        self
    }

    /// Set location (unwraps Option)
    pub fn with_location(mut self, value: ManagedInstanceLocation) -> Self {
        self.location = Some(value);
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: SoftwareSourceDetails) -> Self {
        self.software_source_id = Some(value);
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

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
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
