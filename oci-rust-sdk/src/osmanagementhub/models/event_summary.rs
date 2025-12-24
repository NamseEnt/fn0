use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides summary information about an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the event.
    pub id: String,

    /// Summary of the event.
    pub event_summary: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// Type of event.
    #[serde(rename = "type")]
    pub r#type: EventType,

    /// The date and time that the event was created (in [RFC 3339](https://tools.ietf.org/html/rfc3339) format). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the event occurred (in [RFC 3339](https://tools.ietf.org/html/rfc3339) format). <p> Example: 2016-08-25T21:10:29.600Z
    pub time_occurred: DateTime<Utc>,

    /// The current state of the event.
    pub lifecycle_state: String,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    pub freeform_tags: HashMap<String, String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    pub defined_tags: HashMap<String, HashMap<String, serde_json::Value>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance or resource where the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    /// The date and time that the event was updated (in [RFC 3339](https://tools.ietf.org/html/rfc3339) format). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// Describes the current state of the event in more detail. For example, the message can provide actionable information for a resource in the 'FAILED' state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<String>,

    /// Indicates whether the event occurred on a resource that is managed by the Autonomous Linux service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for EventSummary
pub struct EventSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the event.
    pub id: String,

    /// Summary of the event.
    pub event_summary: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// Type of event.
    pub r#type: EventType,

    /// The date and time that the event was created (in [RFC 3339](https://tools.ietf.org/html/rfc3339) format). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the event occurred (in [RFC 3339](https://tools.ietf.org/html/rfc3339) format). <p> Example: 2016-08-25T21:10:29.600Z
    pub time_occurred: DateTime<Utc>,

    /// The current state of the event.
    pub lifecycle_state: String,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    pub freeform_tags: HashMap<String, String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    pub defined_tags: HashMap<String, HashMap<String, serde_json::Value>>,
}

impl EventSummary {
    /// Create a new EventSummary with required fields
    pub fn new(required: EventSummaryRequired) -> Self {
        Self {
            id: required.id,

            event_summary: required.event_summary,

            compartment_id: required.compartment_id,

            r#type: required.r#type,

            time_created: required.time_created,

            time_occurred: required.time_occurred,

            lifecycle_state: required.lifecycle_state,

            freeform_tags: required.freeform_tags,

            defined_tags: required.defined_tags,

            resource_id: None,

            time_updated: None,

            lifecycle_details: None,

            is_managed_by_autonomous_linux: None,

            system_tags: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set event_summary
    pub fn set_event_summary(mut self, value: String) -> Self {
        self.event_summary = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: EventType) -> Self {
        self.r#type = value;
        self
    }

    /// Set resource_id
    pub fn set_resource_id(mut self, value: Option<String>) -> Self {
        self.resource_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
        self
    }

    /// Set time_occurred
    pub fn set_time_occurred(mut self, value: DateTime<Utc>) -> Self {
        self.time_occurred = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: String) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(mut self, value: Option<String>) -> Self {
        self.lifecycle_details = value;
        self
    }

    /// Set is_managed_by_autonomous_linux
    pub fn set_is_managed_by_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_managed_by_autonomous_linux = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
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

    /// Set resource_id (unwraps Option)
    pub fn with_resource_id(mut self, value: impl Into<String>) -> Self {
        self.resource_id = Some(value.into());
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
        self
    }

    /// Set lifecycle_details (unwraps Option)
    pub fn with_lifecycle_details(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_details = Some(value.into());
        self
    }

    /// Set is_managed_by_autonomous_linux (unwraps Option)
    pub fn with_is_managed_by_autonomous_linux(mut self, value: bool) -> Self {
        self.is_managed_by_autonomous_linux = Some(value);
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
