use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines a type of resource that you can find with a search or query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceType {
    /// The unique name of the resource type, which matches the value returned as part of the ResourceSummary object.
    pub name: String,

    /// List of all the fields and their value type that are indexed for querying.
    pub fields: Vec<QueryableFieldDescription>,
}

/// Required fields for ResourceType
pub struct ResourceTypeRequired {
    /// The unique name of the resource type, which matches the value returned as part of the ResourceSummary object.
    pub name: String,

    /// List of all the fields and their value type that are indexed for querying.
    pub fields: Vec<QueryableFieldDescription>,
}

impl ResourceType {
    /// Create a new ResourceType with required fields
    pub fn new(required: ResourceTypeRequired) -> Self {
        Self {
            name: required.name,

            fields: required.fields,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set fields
    pub fn set_fields(mut self, value: Vec<QueryableFieldDescription>) -> Self {
        self.fields = value;
        self
    }
}
