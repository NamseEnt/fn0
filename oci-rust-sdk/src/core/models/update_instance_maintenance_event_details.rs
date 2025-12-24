use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Specifies the properties for updating maintenance due date.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceMaintenanceEventDetails {
    /// The beginning of the time window when Maintenance is scheduled to begin. The Maintenance will not begin before this time. <p> The timeWindowEnd is automatically calculated based on the maintenanceReason and the instanceAction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_window_start: Option<DateTime<Utc>>,

    /// One of the alternativeResolutionActions that was provided in the InstanceMaintenanceEvent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative_resolution_action: Option<InstanceMaintenanceAlternativeResolutionActions>,

    /// This field is only applicable when setting the alternativeResolutionAction. <p> For Instances that have local storage, this must be set to true to verify that the local storage will be deleted during the migration. For instances without, this parameter has no effect. <p> In cases where the local storage will be lost, this parameter must be set or the request will fail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_delete_local_storage: Option<bool>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

impl UpdateInstanceMaintenanceEventDetails {
    /// Create a new UpdateInstanceMaintenanceEventDetails
    pub fn new() -> Self {
        Self {
            time_window_start: None,

            alternative_resolution_action: None,

            can_delete_local_storage: None,

            display_name: None,

            defined_tags: None,

            freeform_tags: None,
        }
    }

    /// Set time_window_start
    pub fn set_time_window_start(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_window_start = value;
        self
    }

    /// Set alternative_resolution_action
    pub fn set_alternative_resolution_action(
        mut self,
        value: Option<InstanceMaintenanceAlternativeResolutionActions>,
    ) -> Self {
        self.alternative_resolution_action = value;
        self
    }

    /// Set can_delete_local_storage
    pub fn set_can_delete_local_storage(mut self, value: Option<bool>) -> Self {
        self.can_delete_local_storage = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
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

    /// Set time_window_start (unwraps Option)
    pub fn with_time_window_start(mut self, value: DateTime<Utc>) -> Self {
        self.time_window_start = Some(value);
        self
    }

    /// Set alternative_resolution_action (unwraps Option)
    pub fn with_alternative_resolution_action(
        mut self,
        value: InstanceMaintenanceAlternativeResolutionActions,
    ) -> Self {
        self.alternative_resolution_action = Some(value);
        self
    }

    /// Set can_delete_local_storage (unwraps Option)
    pub fn with_can_delete_local_storage(mut self, value: bool) -> Self {
        self.can_delete_local_storage = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
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

impl Default for UpdateInstanceMaintenanceEventDetails {
    fn default() -> Self {
        Self::new()
    }
}
