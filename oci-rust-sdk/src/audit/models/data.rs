use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The payload of the event. Information within {@code data} comes from the resource emitting the event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    /// This value links multiple audit events that are part of the same API operation. For example, a long running API operations that emit an event at the start and the end of an operation would use the same value in this field for both events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_grouping_id: Option<String>,

    /// Name of the API operation that generated this event. <p> Example: {@code GetInstance}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment of the resource emitting the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The name of the compartment. This value is the friendly name associated with compartmentId. This value can change, but the service logs the value that appeared at the time of the audit event. <p> Example: {@code CompartmentA}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_name: Option<String>,

    /// The name of the resource emitting the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,

    /// An [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) or some other ID for the resource emitting the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    /// The availability domain where the resource resides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. Exists for cross-compatibility only. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Response>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_change: Option<StateChange>,

    /// A container object for attribues unique to the resource emitting the event. <p> Example: <p> ----- { \"imageId\": \"ocid1.image.oc1.phx.<unique_ID>\", \"shape\": \"VM.Standard1.1\", \"type\": \"CustomerVmi\" } -----
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<HashMap<String, serde_json::Value>>,
}

impl Data {
    /// Create a new Data
    pub fn new() -> Self {
        Self {
            event_grouping_id: None,

            event_name: None,

            compartment_id: None,

            compartment_name: None,

            resource_name: None,

            resource_id: None,

            availability_domain: None,

            freeform_tags: None,

            defined_tags: None,

            identity: None,

            request: None,

            response: None,

            state_change: None,

            additional_details: None,
        }
    }

    /// Set event_grouping_id
    pub fn set_event_grouping_id(mut self, value: Option<String>) -> Self {
        self.event_grouping_id = value;
        self
    }

    /// Set event_name
    pub fn set_event_name(mut self, value: Option<String>) -> Self {
        self.event_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set compartment_name
    pub fn set_compartment_name(mut self, value: Option<String>) -> Self {
        self.compartment_name = value;
        self
    }

    /// Set resource_name
    pub fn set_resource_name(mut self, value: Option<String>) -> Self {
        self.resource_name = value;
        self
    }

    /// Set resource_id
    pub fn set_resource_id(mut self, value: Option<String>) -> Self {
        self.resource_id = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
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

    /// Set identity
    pub fn set_identity(mut self, value: Option<Identity>) -> Self {
        self.identity = value;
        self
    }

    /// Set request
    pub fn set_request(mut self, value: Option<Request>) -> Self {
        self.request = value;
        self
    }

    /// Set response
    pub fn set_response(mut self, value: Option<Response>) -> Self {
        self.response = value;
        self
    }

    /// Set state_change
    pub fn set_state_change(mut self, value: Option<StateChange>) -> Self {
        self.state_change = value;
        self
    }

    /// Set additional_details
    pub fn set_additional_details(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.additional_details = value;
        self
    }

    /// Set event_grouping_id (unwraps Option)
    pub fn with_event_grouping_id(mut self, value: impl Into<String>) -> Self {
        self.event_grouping_id = Some(value.into());
        self
    }

    /// Set event_name (unwraps Option)
    pub fn with_event_name(mut self, value: impl Into<String>) -> Self {
        self.event_name = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set compartment_name (unwraps Option)
    pub fn with_compartment_name(mut self, value: impl Into<String>) -> Self {
        self.compartment_name = Some(value.into());
        self
    }

    /// Set resource_name (unwraps Option)
    pub fn with_resource_name(mut self, value: impl Into<String>) -> Self {
        self.resource_name = Some(value.into());
        self
    }

    /// Set resource_id (unwraps Option)
    pub fn with_resource_id(mut self, value: impl Into<String>) -> Self {
        self.resource_id = Some(value.into());
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
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

    /// Set identity (unwraps Option)
    pub fn with_identity(mut self, value: Identity) -> Self {
        self.identity = Some(value);
        self
    }

    /// Set request (unwraps Option)
    pub fn with_request(mut self, value: Request) -> Self {
        self.request = Some(value);
        self
    }

    /// Set response (unwraps Option)
    pub fn with_response(mut self, value: Response) -> Self {
        self.response = Some(value);
        self
    }

    /// Set state_change (unwraps Option)
    pub fn with_state_change(mut self, value: StateChange) -> Self {
        self.state_change = Some(value);
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.additional_details = Some(value);
        self
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}
