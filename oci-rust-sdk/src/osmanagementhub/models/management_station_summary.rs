use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementStationSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the management station.
    pub compartment_id: String,

    /// User-friendly name for the management station.
    pub display_name: String,

    /// Hostname of the management station.
    pub hostname: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance that is acting as the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the registration profile used for the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the scheduled job for the mirror sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_job_id: Option<String>,

    /// The date and time of the next scheduled mirror sync (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_next_execution: Option<DateTime<Utc>>,

    /// User-specified description of the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Current state of the mirror sync for the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_state: Option<OverallState>,

    /// Overall health status of the managment station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_state: Option<HealthState>,

    /// A decimal number representing the progress of the current mirror sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_percentage: Option<i64>,

    /// A decimal number representing the amount of mirror capacity used by the sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_capacity: Option<i64>,

    /// The current state of the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

    /// The location of the instance that is acting as the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

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

/// Required fields for ManagementStationSummary
pub struct ManagementStationSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the management station.
    pub compartment_id: String,

    /// User-friendly name for the management station.
    pub display_name: String,

    /// Hostname of the management station.
    pub hostname: String,
}

impl ManagementStationSummary {
    /// Create a new ManagementStationSummary with required fields
    pub fn new(required: ManagementStationSummaryRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            hostname: required.hostname,

            managed_instance_id: None,

            profile_id: None,

            scheduled_job_id: None,

            time_next_execution: None,

            description: None,

            overall_state: None,

            health_state: None,

            overall_percentage: None,

            mirror_capacity: None,

            lifecycle_state: None,

            location: None,

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

    /// Set managed_instance_id
    pub fn set_managed_instance_id(mut self, value: Option<String>) -> Self {
        self.managed_instance_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set profile_id
    pub fn set_profile_id(mut self, value: Option<String>) -> Self {
        self.profile_id = value;
        self
    }

    /// Set scheduled_job_id
    pub fn set_scheduled_job_id(mut self, value: Option<String>) -> Self {
        self.scheduled_job_id = value;
        self
    }

    /// Set time_next_execution
    pub fn set_time_next_execution(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_next_execution = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set hostname
    pub fn set_hostname(mut self, value: String) -> Self {
        self.hostname = value;
        self
    }

    /// Set overall_state
    pub fn set_overall_state(mut self, value: Option<OverallState>) -> Self {
        self.overall_state = value;
        self
    }

    /// Set health_state
    pub fn set_health_state(mut self, value: Option<HealthState>) -> Self {
        self.health_state = value;
        self
    }

    /// Set overall_percentage
    pub fn set_overall_percentage(mut self, value: Option<i64>) -> Self {
        self.overall_percentage = value;
        self
    }

    /// Set mirror_capacity
    pub fn set_mirror_capacity(mut self, value: Option<i64>) -> Self {
        self.mirror_capacity = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
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

    /// Set managed_instance_id (unwraps Option)
    pub fn with_managed_instance_id(mut self, value: impl Into<String>) -> Self {
        self.managed_instance_id = Some(value.into());
        self
    }

    /// Set profile_id (unwraps Option)
    pub fn with_profile_id(mut self, value: impl Into<String>) -> Self {
        self.profile_id = Some(value.into());
        self
    }

    /// Set scheduled_job_id (unwraps Option)
    pub fn with_scheduled_job_id(mut self, value: impl Into<String>) -> Self {
        self.scheduled_job_id = Some(value.into());
        self
    }

    /// Set time_next_execution (unwraps Option)
    pub fn with_time_next_execution(mut self, value: DateTime<Utc>) -> Self {
        self.time_next_execution = Some(value);
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set overall_state (unwraps Option)
    pub fn with_overall_state(mut self, value: OverallState) -> Self {
        self.overall_state = Some(value);
        self
    }

    /// Set health_state (unwraps Option)
    pub fn with_health_state(mut self, value: HealthState) -> Self {
        self.health_state = Some(value);
        self
    }

    /// Set overall_percentage (unwraps Option)
    pub fn with_overall_percentage(mut self, value: i64) -> Self {
        self.overall_percentage = Some(value);
        self
    }

    /// Set mirror_capacity (unwraps Option)
    pub fn with_mirror_capacity(mut self, value: i64) -> Self {
        self.mirror_capacity = Some(value);
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
