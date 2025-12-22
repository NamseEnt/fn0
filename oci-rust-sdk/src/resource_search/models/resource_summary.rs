use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::search_context::SearchContext;

/// Required fields for ResourceSummary
pub struct ResourceSummaryRequired {
    /// The resource type name (e.g., "Instance", "VolumeAttachment")
    pub resource_type: String,
    /// The unique identifier for this particular resource, usually an OCID
    pub identifier: String,
    /// The OCID of the compartment that contains this resource
    pub compartment_id: String,
}

/// A resource that exists in the cloud network that you're querying.
///
/// Contains information about a single resource returned from a search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSummary {
    /// The resource type name (e.g., "Instance", "VolumeAttachment").
    pub resource_type: String,

    /// The unique identifier for this particular resource, usually an OCID.
    pub identifier: String,

    /// The OCID of the compartment that contains this resource.
    pub compartment_id: String,

    /// The time that this resource was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The display name (or name) of this resource, if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The availability domain where this resource exists, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The lifecycle state of this resource, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no
    /// predefined name, type, or namespace.
    ///
    /// Example: `{"Department": "Finance"}`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace.
    ///
    /// Example: `{"Operations": {"CostCenter": "42"}}`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags associated with this resource, if any. System tags are set by
    /// Oracle Cloud Infrastructure services. Each key is predefined and scoped to namespaces.
    ///
    /// Example: `{"orcl-cloud": {"free-tier-retain": true}}`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Additional search context showing what matched the query (when highlights are requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context: Option<SearchContext>,

    /// Additional identifiers to use together in a "Get" request for a specified resource.
    /// Only required for resource types that explicitly cannot be retrieved by using a single
    /// identifier, such as the resource's OCID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_context: Option<HashMap<String, serde_json::Value>>,

    /// Additional resource attribute fields of this resource that match queries with a return clause.
    ///
    /// For example, if you ran a query to find the private IP addresses, public IP addresses,
    /// and isPrimary field of the VNIC attachment on instance resources, that field would be
    /// included in the ResourceSummary object as:
    /// ```json
    /// {"additionalDetails": {"attachedVnic": [{"publicIP": "172.110.110.110", "privateIP": "10.10.10.10", "isPrimary": true}]}}
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<HashMap<String, serde_json::Value>>,
}

impl ResourceSummary {
    /// Create a new ResourceSummary with required fields
    pub fn new(required: ResourceSummaryRequired) -> Self {
        Self {
            resource_type: required.resource_type,
            identifier: required.identifier,
            compartment_id: required.compartment_id,
            time_created: None,
            display_name: None,
            availability_domain: None,
            lifecycle_state: None,
            freeform_tags: None,
            defined_tags: None,
            system_tags: None,
            search_context: None,
            identity_context: None,
            additional_details: None,
        }
    }

    // Setters for all fields (required + optional)
    pub fn set_resource_type(mut self, resource_type: String) -> Self {
        self.resource_type = resource_type;
        self
    }

    pub fn set_identifier(mut self, identifier: String) -> Self {
        self.identifier = identifier;
        self
    }

    pub fn set_compartment_id(mut self, compartment_id: String) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    pub fn set_time_created(mut self, time_created: Option<DateTime<Utc>>) -> Self {
        self.time_created = time_created;
        self
    }

    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    pub fn set_availability_domain(mut self, availability_domain: Option<String>) -> Self {
        self.availability_domain = availability_domain;
        self
    }

    pub fn set_lifecycle_state(mut self, lifecycle_state: Option<String>) -> Self {
        self.lifecycle_state = lifecycle_state;
        self
    }

    pub fn set_freeform_tags(mut self, freeform_tags: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = freeform_tags;
        self
    }

    pub fn set_defined_tags(mut self, defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.defined_tags = defined_tags;
        self
    }

    pub fn set_system_tags(mut self, system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.system_tags = system_tags;
        self
    }

    pub fn set_search_context(mut self, search_context: Option<SearchContext>) -> Self {
        self.search_context = search_context;
        self
    }

    pub fn set_identity_context(mut self, identity_context: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.identity_context = identity_context;
        self
    }

    pub fn set_additional_details(mut self, additional_details: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.additional_details = additional_details;
        self
    }

    // With methods for optional fields only
    pub fn with_time_created(mut self, time_created: DateTime<Utc>) -> Self {
        self.time_created = Some(time_created);
        self
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_availability_domain(mut self, availability_domain: impl Into<String>) -> Self {
        self.availability_domain = Some(availability_domain.into());
        self
    }

    pub fn with_lifecycle_state(mut self, lifecycle_state: impl Into<String>) -> Self {
        self.lifecycle_state = Some(lifecycle_state.into());
        self
    }

    pub fn with_freeform_tags(mut self, freeform_tags: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(freeform_tags);
        self
    }

    pub fn with_defined_tags(mut self, defined_tags: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
        self.defined_tags = Some(defined_tags);
        self
    }

    pub fn with_system_tags(mut self, system_tags: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
        self.system_tags = Some(system_tags);
        self
    }

    pub fn with_search_context(mut self, search_context: SearchContext) -> Self {
        self.search_context = Some(search_context);
        self
    }

    pub fn with_identity_context(mut self, identity_context: HashMap<String, serde_json::Value>) -> Self {
        self.identity_context = Some(identity_context);
        self
    }

    pub fn with_additional_details(mut self, additional_details: HashMap<String, serde_json::Value>) -> Self {
        self.additional_details = Some(additional_details);
        self
    }
}
