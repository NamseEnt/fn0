use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A resource that exists in the cloud network that you're querying.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSummary {
    /// The resource type name.
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

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags associated with this resource, if any. System tags are set by Oracle Cloud Infrastructure services. Each key is predefined and scoped to namespaces. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {orcl-cloud: {free-tier-retain: true}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context: Option<SearchContext>,

    /// Additional identifiers to use together in a \"Get\" request for a specified resource, only required for resource types that explicitly cannot be retrieved by using a single identifier, such as the resource's OCID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_context: Option<HashMap<String, serde_json::Value>>,

    /// Additional resource attribute fields of this resource that match queries with a return clause, if any. For example, if you ran a query to find the private IP addresses, public IP addresses, and isPrimary field of the VNIC attachment on instance resources, that field would be included in the ResourceSummary object as: {\"additionalDetails\": {\"attachedVnic\": [{\"publicIP\" : \"172.110.110.110\",\"privateIP\" : \"10.10.10.10\",\"isPrimary\" : true}, {\"publicIP\" : \"172.110.110.111\",\"privateIP\" : \"10.10.10.11\",\"isPrimary\" : false}]}. The structure of the additional details attribute fields depends on the matching resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<HashMap<String, serde_json::Value>>,
}

/// Required fields for ResourceSummary
pub struct ResourceSummaryRequired {
    /// The resource type name.
    pub resource_type: String,

    /// The unique identifier for this particular resource, usually an OCID.
    pub identifier: String,

    /// The OCID of the compartment that contains this resource.
    pub compartment_id: String,
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

    /// Set resource_type
    pub fn set_resource_type(mut self, value: String) -> Self {
        self.resource_type = value;
        self
    }

    /// Set identifier
    pub fn set_identifier(mut self, value: String) -> Self {
        self.identifier = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
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

    /// Set search_context
    pub fn set_search_context(mut self, value: Option<SearchContext>) -> Self {
        self.search_context = value;
        self
    }

    /// Set identity_context
    pub fn set_identity_context(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.identity_context = value;
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

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
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

    /// Set search_context (unwraps Option)
    pub fn with_search_context(mut self, value: SearchContext) -> Self {
        self.search_context = Some(value);
        self
    }

    /// Set identity_context (unwraps Option)
    pub fn with_identity_context(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.identity_context = Some(value);
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.additional_details = Some(value);
        self
    }
}
